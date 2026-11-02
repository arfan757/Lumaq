use super::{MediaSource, NowPlaying, PlaybackState};
use async_trait::async_trait;

pub struct StubMediaSource;

impl Default for StubMediaSource {
    fn default() -> Self {
        Self::new()
    }
}

impl StubMediaSource {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl MediaSource for StubMediaSource {
    async fn current_now_playing(&self) -> Option<NowPlaying> {
        None
    }

    async fn current_playback(&self) -> PlaybackState {
        PlaybackState::default()
    }

    async fn play_pause(&self) -> Result<(), String> {
        Err("Media controls not supported on this platform".to_string())
    }

    async fn next(&self) -> Result<(), String> {
        Err("Media controls not supported on this platform".to_string())
    }

    async fn prev(&self) -> Result<(), String> {
        Err("Media controls not supported on this platform".to_string())
    }

    async fn seek(&self, _position_ms: u64) -> Result<(), String> {
        Err("Media controls not supported on this platform".to_string())
    }

    async fn toggle_shuffle(&self) -> Result<(), String> {
        Err("Media controls not supported on this platform".to_string())
    }

    async fn cycle_repeat(&self) -> Result<(), String> {
        Err("Media controls not supported on this platform".to_string())
    }

    fn set_source_preference(&self, _preference: String) {}
}
