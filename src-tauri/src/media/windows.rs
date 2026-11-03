use super::{MediaSource, NowPlaying, PlaybackState, PlaybackStatus, RepeatMode};
use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::broadcast;
use tracing::{info, warn};
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession, GlobalSystemMediaTransportControlsSessionManager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus,
};
use windows::Media::MediaPlaybackAutoRepeatMode;
use windows::Storage::Streams::{DataReader, InputStreamOptions};

enum MediaCommand {
    PlayPause,
    Next,
    Prev,
    Seek(u64),
    ToggleShuffle,
    CycleRepeat,
    SetPreference(String),
}

pub struct WindowsMediaSource {
    cmd_tx: Sender<MediaCommand>,
    cached_now_playing: Arc<Mutex<Option<NowPlaying>>>,
    cached_playback: Arc<Mutex<PlaybackState>>,
    _is_running: Arc<AtomicBool>,
}

impl WindowsMediaSource {
    pub fn new() -> (
        Self,
        broadcast::Receiver<Option<NowPlaying>>,
        broadcast::Receiver<PlaybackState>,
    ) {
        let (np_tx, np_rx) = broadcast::channel(32);
        let (pb_tx, pb_rx) = broadcast::channel(32);
        let (cmd_tx, cmd_rx) = mpsc::channel::<MediaCommand>();

        let cached_now_playing = Arc::new(Mutex::new(None));
        let cached_playback = Arc::new(Mutex::new(PlaybackState::default()));
        let is_running = Arc::new(AtomicBool::new(true));

        let np_cache = Arc::clone(&cached_now_playing);
        let pb_cache = Arc::clone(&cached_playback);
        let running_flag = Arc::clone(&is_running);

        std::thread::Builder::new()
            .name("lumaq-gsmtc".to_string())
            .spawn(move || {
                Self::run_gsmtc_loop(cmd_rx, np_tx, pb_tx, np_cache, pb_cache, running_flag);
            })
            .expect("failed to spawn GSMTC thread");

        let source = Self {
            cmd_tx,
            cached_now_playing,
            cached_playback,
            _is_running: is_running,
        };

        (source, np_rx, pb_rx)
    }

    fn run_gsmtc_loop(
        cmd_rx: Receiver<MediaCommand>,
        np_tx: broadcast::Sender<Option<NowPlaying>>,
        pb_tx: broadcast::Sender<PlaybackState>,
        np_cache: Arc<Mutex<Option<NowPlaying>>>,
        pb_cache: Arc<Mutex<PlaybackState>>,
        is_running: Arc<AtomicBool>,
    ) {
        info!("GSMTC background thread started");

        let manager = match GlobalSystemMediaTransportControlsSessionManager::RequestAsync() {
            Ok(op) => match op.get() {
                Ok(m) => m,
                Err(e) => {
                    warn!("Failed to get GSMTC SessionManager: {:?}", e);
                    return;
                }
            },
            Err(e) => {
                warn!("Failed to request GSMTC SessionManager: {:?}", e);
                return;
            }
        };

        info!("GSMTC SessionManager initialized successfully");

        let mut preference = "spotify-first".to_string();
        let mut last_stable_key = String::new();
        let mut pending_key = String::new();
        let mut pending_since = Instant::now();
        let mut last_resync = Instant::now();
        let mut last_emitted = PlaybackState::default();
        let mut last_emitted_position: i64 = -1;
        let mut pending_seek: Option<(u64, Instant)> = None;
        let mut thumb_key = String::new();
        let mut thumb_data: Option<String> = None;

        while is_running.load(Ordering::Relaxed) {
            // 1. Wake up immediately when a command arrives so controls feel instant.
            let mut mutated = false;

            match cmd_rx.recv_timeout(Duration::from_millis(150)) {
                Ok(cmd) => {
                    let (m, hint) =
                        Self::execute_command(cmd, &manager, &mut preference, &pb_cache);
                    mutated |= m;
                    if let Some(ms) = hint {
                        pending_seek = Some((ms, Instant::now()));
                    }
                    while let Ok(cmd) = cmd_rx.try_recv() {
                        let (m, hint) =
                            Self::execute_command(cmd, &manager, &mut preference, &pb_cache);
                        mutated |= m;
                        if let Some(ms) = hint {
                            pending_seek = Some((ms, Instant::now()));
                        }
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }

            // 2. Poll active session
            let target_session = Self::find_target_session(&manager, &preference);

            if let Some(session) = target_session {
                if let Ok((now_playing, mut playback)) =
                    Self::read_session_state(&session, &mut thumb_key, &mut thumb_data)
                {
                    // Hold the requested position briefly until the player catches up.
                    if let Some((hint, since)) = pending_seek {
                        let close = (playback.position_ms as i64 - hint as i64).abs() <= 1500;
                        if close || since.elapsed() > Duration::from_secs(2) {
                            pending_seek = None;
                        } else {
                            playback.position_ms = hint;
                        }
                    }

                    if let Some(ref np) = now_playing {
                        let key = np.track_key.clone();
                        if key != last_stable_key {
                            // Non-blocking debounce for transient track metadata.
                            if pending_key != key {
                                pending_key = key.clone();
                                pending_since = Instant::now();
                            }
                            if pending_since.elapsed() >= Duration::from_millis(200) {
                                last_stable_key = key.clone();
                            }
                        } else {
                            pending_key.clear();
                        }

                        if key == last_stable_key {
                            let mut curr_np = np_cache.lock().unwrap();
                            let changed = curr_np.as_ref() != Some(np);
                            if changed {
                                *curr_np = Some(np.clone());
                                let _ = np_tx.send(Some(np.clone()));
                            }
                        }
                    }

                    // Keep the shared cache fresh for command handlers.
                    {
                        let mut cached = pb_cache.lock().unwrap();
                        *cached = playback.clone();
                    }

                    let status_changed = last_emitted.status != playback.status
                        || last_emitted.shuffle != playback.shuffle
                        || last_emitted.repeat != playback.repeat;
                    let position_delta =
                        (playback.position_ms as i64 - last_emitted_position).abs();
                    let should_resync = playback.status == PlaybackStatus::Playing
                        && last_resync.elapsed() >= Duration::from_secs(2);
                    let should_emit =
                        mutated || status_changed || should_resync || position_delta >= 500;

                    if should_emit {
                        last_emitted = playback.clone();
                        last_emitted_position = playback.position_ms as i64;
                        last_resync = Instant::now();
                        let _ = pb_tx.send(playback);
                    }
                }
            } else {
                // No session active
                pending_key.clear();
                last_stable_key.clear();
                pending_seek = None;
                last_emitted = PlaybackState::default();
                last_emitted_position = -1;

                let mut curr_np = np_cache.lock().unwrap();
                if curr_np.is_some() {
                    *curr_np = None;
                    let _ = np_tx.send(None);
                }
                let mut curr_pb = pb_cache.lock().unwrap();
                if curr_pb.status != PlaybackStatus::None {
                    *curr_pb = PlaybackState::default();
                    let _ = pb_tx.send(PlaybackState::default());
                }
            }
        }
    }

    /// Executes a media command immediately and reports whether it mutated
    /// playback plus an optional seek position hint.
    fn execute_command(
        cmd: MediaCommand,
        manager: &GlobalSystemMediaTransportControlsSessionManager,
        preference: &mut String,
        pb_cache: &Arc<Mutex<PlaybackState>>,
    ) -> (bool, Option<u64>) {
        match cmd {
            MediaCommand::SetPreference(p) => {
                *preference = p;
                (false, None)
            }
            other => {
                let session = match Self::find_target_session(manager, preference) {
                    Some(s) => s,
                    None => return (false, None),
                };

                let mut seek_hint = None;
                match other {
                    MediaCommand::PlayPause => {
                        let _ = session.TryTogglePlayPauseAsync().and_then(|op| op.get());
                    }
                    MediaCommand::Next => {
                        let _ = session.TrySkipNextAsync().and_then(|op| op.get());
                    }
                    MediaCommand::Prev => {
                        let _ = session.TrySkipPreviousAsync().and_then(|op| op.get());
                    }
                    MediaCommand::Seek(ms) => {
                        let ticks = (ms * 10_000) as i64;
                        let _ = session
                            .TryChangePlaybackPositionAsync(ticks)
                            .and_then(|op| op.get());
                        seek_hint = Some(ms);
                    }
                    MediaCommand::ToggleShuffle => {
                        let curr_shuffle = pb_cache.lock().unwrap().shuffle.unwrap_or(false);
                        let _ = session
                            .TryChangeShuffleActiveAsync(!curr_shuffle)
                            .and_then(|op| op.get());
                    }
                    MediaCommand::CycleRepeat => {
                        let curr_repeat =
                            pb_cache.lock().unwrap().repeat.unwrap_or(RepeatMode::Off);
                        let next_mode = match curr_repeat {
                            RepeatMode::Off => MediaPlaybackAutoRepeatMode::List,
                            RepeatMode::List => MediaPlaybackAutoRepeatMode::Track,
                            RepeatMode::Track => MediaPlaybackAutoRepeatMode::None,
                        };
                        let _ = session
                            .TryChangeAutoRepeatModeAsync(next_mode)
                            .and_then(|op| op.get());
                    }
                    MediaCommand::SetPreference(_) => {}
                }

                (true, seek_hint)
            }
        }
    }

    fn find_target_session(
        manager: &GlobalSystemMediaTransportControlsSessionManager,
        preference: &str,
    ) -> Option<GlobalSystemMediaTransportControlsSession> {
        let sessions = match manager.GetSessions() {
            Ok(s) => s,
            Err(_) => return manager.GetCurrentSession().ok(),
        };

        let count = sessions.Size().unwrap_or(0);
        if count == 0 {
            return manager.GetCurrentSession().ok();
        }

        if preference == "spotify-first" {
            for i in 0..count {
                if let Ok(session) = sessions.GetAt(i) {
                    if let Ok(app_id) = session.SourceAppUserModelId() {
                        let app_str = app_id.to_string().to_lowercase();
                        if app_str.contains("spotify") {
                            return Some(session);
                        }
                    }
                }
            }
        }

        if let Ok(curr) = manager.GetCurrentSession() {
            return Some(curr);
        }

        sessions.GetAt(0).ok()
    }

    fn read_session_state(
        session: &GlobalSystemMediaTransportControlsSession,
        thumb_key: &mut String,
        thumb_data: &mut Option<String>,
    ) -> Result<(Option<NowPlaying>, PlaybackState), String> {
        let source_app = session
            .SourceAppUserModelId()
            .map(|s| s.to_string())
            .unwrap_or_else(|_| "Unknown".to_string());

        let media_props = session
            .TryGetMediaPropertiesAsync()
            .map_err(|e| format!("{:?}", e))?
            .get()
            .map_err(|e| format!("{:?}", e))?;

        let title = media_props
            .Title()
            .map(|s| s.to_string())
            .unwrap_or_default();
        let artist = media_props
            .Artist()
            .map(|s| s.to_string())
            .unwrap_or_default();
        let album = media_props
            .AlbumTitle()
            .map(|s| s.to_string())
            .unwrap_or_default();

        let timeline = session.GetTimelineProperties().ok();
        let duration_ms = timeline
            .as_ref()
            .and_then(|t| t.EndTime().ok())
            .map(|d| (d.Duration / 10_000) as u64)
            .unwrap_or(0);

        let position_ticks = timeline
            .as_ref()
            .and_then(|t| t.Position().ok())
            .map(|p| p.Duration)
            .unwrap_or(0);

        let last_updated = timeline
            .as_ref()
            .and_then(|t| t.LastUpdatedTime().ok())
            .map(|u| u.UniversalTime)
            .unwrap_or(0);

        let playback_info = session.GetPlaybackInfo().ok();
        let status = playback_info
            .as_ref()
            .and_then(|p| p.PlaybackStatus().ok())
            .map(|s| match s {
                GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing => {
                    PlaybackStatus::Playing
                }
                GlobalSystemMediaTransportControlsSessionPlaybackStatus::Paused => {
                    PlaybackStatus::Paused
                }
                GlobalSystemMediaTransportControlsSessionPlaybackStatus::Stopped => {
                    PlaybackStatus::Stopped
                }
                _ => PlaybackStatus::None,
            })
            .unwrap_or(PlaybackStatus::None);

        let rate = playback_info
            .as_ref()
            .and_then(|p| p.PlaybackRate().ok())
            .and_then(|r| r.Value().ok())
            .unwrap_or(1.0);

        let mut position_ms = (position_ticks / 10_000) as u64;
        if status == PlaybackStatus::Playing && last_updated > 0 {
            let win_epoch_now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
                / 100
                + 116_444_736_000_000_000;

            if win_epoch_now > last_updated as u128 {
                let elapsed_ms = ((win_epoch_now - last_updated as u128) / 10_000) as f64;
                position_ms = (position_ms as f64 + elapsed_ms * rate) as u64;
            }
        }

        if duration_ms > 0 {
            position_ms = position_ms.min(duration_ms);
        }

        let controls = playback_info.as_ref().and_then(|p| p.Controls().ok());
        let can_play_pause = controls
            .as_ref()
            .and_then(|c| c.IsPlayPauseToggleEnabled().ok())
            .unwrap_or(true);
        let can_next = controls
            .as_ref()
            .and_then(|c| c.IsNextEnabled().ok())
            .unwrap_or(true);
        let can_prev = controls
            .as_ref()
            .and_then(|c| c.IsPreviousEnabled().ok())
            .unwrap_or(true);
        let can_seek = controls
            .as_ref()
            .and_then(|c| c.IsPlaybackPositionEnabled().ok())
            .unwrap_or(true);
        let can_shuffle = controls
            .as_ref()
            .and_then(|c| c.IsShuffleEnabled().ok())
            .unwrap_or(false);
        let can_repeat = controls
            .as_ref()
            .and_then(|c| c.IsRepeatEnabled().ok())
            .unwrap_or(false);

        let shuffle = playback_info
            .as_ref()
            .and_then(|p| p.IsShuffleActive().ok())
            .and_then(|s| s.Value().ok());

        let repeat = playback_info
            .as_ref()
            .and_then(|p| p.AutoRepeatMode().ok())
            .and_then(|r| r.Value().ok())
            .map(|r| match r {
                MediaPlaybackAutoRepeatMode::None => RepeatMode::Off,
                MediaPlaybackAutoRepeatMode::Track => RepeatMode::Track,
                MediaPlaybackAutoRepeatMode::List => RepeatMode::List,
                _ => RepeatMode::Off,
            });

        let is_ad = title.eq_ignore_ascii_case("Advertisement")
            || (artist.eq_ignore_ascii_case("Spotify") && duration_ms > 0 && duration_ms <= 60_000);

        let track_key = format!(
            "{:x}",
            md5_hash(&format!("{}|{}|{}|{}", title, artist, album, duration_ms))
        );

        // Artwork is expensive to read and base64-encode, so only refresh it
        // when the track changes.
        let thumbnail = if *thumb_key == track_key && thumb_data.is_some() {
            thumb_data.clone()
        } else {
            let parsed = if let Ok(stream_ref) = media_props.Thumbnail() {
                Self::read_thumbnail_base64_sync(stream_ref)
            } else {
                None
            };
            if parsed.is_some() {
                *thumb_key = track_key.clone();
                *thumb_data = parsed.clone();
            }
            parsed
        };

        let now_playing = if title.is_empty() && artist.is_empty() {
            None
        } else {
            Some(NowPlaying {
                track_key,
                title,
                artist,
                album,
                duration_ms,
                artwork: thumbnail,
                is_ad,
                source_app,
            })
        };

        let playback_state = PlaybackState {
            status,
            position_ms,
            rate,
            shuffle,
            repeat,
            can_play_pause,
            can_next,
            can_prev,
            can_seek,
            can_shuffle,
            can_repeat,
        };

        Ok((now_playing, playback_state))
    }

    fn read_thumbnail_base64_sync(
        stream_ref: windows::Storage::Streams::IRandomAccessStreamReference,
    ) -> Option<String> {
        let stream = stream_ref.OpenReadAsync().ok()?.get().ok()?;
        let size = stream.Size().ok()? as u32;
        if size == 0 {
            return None;
        }

        let reader = DataReader::CreateDataReader(&stream).ok()?;
        reader
            .SetInputStreamOptions(InputStreamOptions::None)
            .ok()?;
        reader.LoadAsync(size).ok()?.get().ok()?;

        let mut bytes = vec![0u8; size as usize];
        reader.ReadBytes(&mut bytes).ok()?;

        let mime = if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
            "image/png"
        } else {
            "image/jpeg"
        };

        use base64::engine::general_purpose::STANDARD;
        use base64::Engine;
        let b64 = STANDARD.encode(&bytes);

        Some(format!("data:{};base64,{}", mime, b64))
    }
}

fn md5_hash(input: &str) -> u128 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut s = DefaultHasher::new();
    input.hash(&mut s);
    s.finish() as u128
}

#[async_trait]
impl MediaSource for WindowsMediaSource {
    async fn current_now_playing(&self) -> Option<NowPlaying> {
        self.cached_now_playing.lock().unwrap().clone()
    }

    async fn current_playback(&self) -> PlaybackState {
        self.cached_playback.lock().unwrap().clone()
    }

    async fn play_pause(&self) -> Result<(), String> {
        self.cmd_tx
            .send(MediaCommand::PlayPause)
            .map_err(|e| format!("{:?}", e))
    }

    async fn next(&self) -> Result<(), String> {
        self.cmd_tx
            .send(MediaCommand::Next)
            .map_err(|e| format!("{:?}", e))
    }

    async fn prev(&self) -> Result<(), String> {
        self.cmd_tx
            .send(MediaCommand::Prev)
            .map_err(|e| format!("{:?}", e))
    }

    async fn seek(&self, position_ms: u64) -> Result<(), String> {
        self.cmd_tx
            .send(MediaCommand::Seek(position_ms))
            .map_err(|e| format!("{:?}", e))
    }

    async fn toggle_shuffle(&self) -> Result<(), String> {
        self.cmd_tx
            .send(MediaCommand::ToggleShuffle)
            .map_err(|e| format!("{:?}", e))
    }

    async fn cycle_repeat(&self) -> Result<(), String> {
        self.cmd_tx
            .send(MediaCommand::CycleRepeat)
            .map_err(|e| format!("{:?}", e))
    }

    fn set_source_preference(&self, preference: String) {
        let _ = self.cmd_tx.send(MediaCommand::SetPreference(preference));
    }
}
