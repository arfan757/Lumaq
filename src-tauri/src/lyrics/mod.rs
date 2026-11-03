pub mod cache;
pub mod lrc;
pub mod lrclib;
pub mod normalize;

use cache::{CachedLyrics, LyricsDiskCache};
use lrc::LyricLine;
use lrclib::{FetchResult, LrclibClient};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tokio::time::{sleep, Duration};
use tracing::{debug, info};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum LyricsState {
    Loading {
        track_key: String,
    },
    Synced {
        track_key: String,
        lines: Vec<LyricLine>,
    },
    Plain {
        track_key: String,
        text: String,
    },
    Instrumental {
        track_key: String,
    },
    NotFound {
        track_key: String,
    },
    Error {
        track_key: String,
        message: String,
    },
}

impl LyricsState {
    pub fn track_key(&self) -> &str {
        match self {
            LyricsState::Loading { track_key } => track_key,
            LyricsState::Synced { track_key, .. } => track_key,
            LyricsState::Plain { track_key, .. } => track_key,
            LyricsState::Instrumental { track_key } => track_key,
            LyricsState::NotFound { track_key } => track_key,
            LyricsState::Error { track_key, .. } => track_key,
        }
    }
}

pub struct LyricsManager {
    client: Arc<LrclibClient>,
    cache: Arc<Mutex<LyricsDiskCache>>,
    current_request_id: Arc<AtomicU64>,
    current_state: Arc<Mutex<Option<LyricsState>>>,
    event_tx: broadcast::Sender<LyricsState>,
}

impl LyricsManager {
    pub fn new(app_data_dir: PathBuf) -> (Self, broadcast::Receiver<LyricsState>) {
        let (event_tx, rx) = broadcast::channel(32);
        let manager = Self {
            client: Arc::new(LrclibClient::new()),
            cache: Arc::new(Mutex::new(LyricsDiskCache::new(app_data_dir))),
            current_request_id: Arc::new(AtomicU64::new(0)),
            current_state: Arc::new(Mutex::new(None)),
            event_tx,
        };

        (manager, rx)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<LyricsState> {
        self.event_tx.subscribe()
    }

    pub async fn current_lyrics(&self) -> Option<LyricsState> {
        let guard = self.current_state.lock().await;
        guard.clone()
    }

    pub async fn on_track_change(
        &self,
        track_key: String,
        title: String,
        artist: String,
        album: String,
        duration_ms: u64,
        is_ad: bool,
    ) {
        let req_id = self.current_request_id.fetch_add(1, Ordering::SeqCst) + 1;

        if is_ad {
            let state = LyricsState::Plain {
                track_key,
                text: "Ad playing".to_string(),
            };
            {
                let mut guard = self.current_state.lock().await;
                *guard = Some(state.clone());
            }
            let _ = self.event_tx.send(state);
            return;
        }

        // 1. Emit loading state
        let loading_state = LyricsState::Loading {
            track_key: track_key.clone(),
        };
        {
            let mut guard = self.current_state.lock().await;
            *guard = Some(loading_state.clone());
        }
        let _ = self.event_tx.send(loading_state);

        // 2. Check disk cache first
        {
            let cache = self.cache.lock().await;
            if let Some(cached) = cache.get(&track_key) {
                let state = match cached {
                    CachedLyrics::Synced { lines } => LyricsState::Synced {
                        track_key: track_key.clone(),
                        lines,
                    },
                    CachedLyrics::Plain { text } => LyricsState::Plain {
                        track_key: track_key.clone(),
                        text,
                    },
                    CachedLyrics::Instrumental => LyricsState::Instrumental {
                        track_key: track_key.clone(),
                    },
                    CachedLyrics::NotFound => LyricsState::NotFound {
                        track_key: track_key.clone(),
                    },
                };
                {
                    let mut guard = self.current_state.lock().await;
                    *guard = Some(state.clone());
                }
                let _ = self.event_tx.send(state);
                return;
            }
        }

        // 3. Debounce 300 ms so rapid track skips don't spam the API
        let client = Arc::clone(&self.client);
        let cache_lock = Arc::clone(&self.cache);
        let req_id_atomic = Arc::clone(&self.current_request_id);
        let state_lock = Arc::clone(&self.current_state);
        let tx = self.event_tx.clone();

        tokio::spawn(async move {
            sleep(Duration::from_millis(300)).await;

            // Check if this request is still current
            if req_id_atomic.load(Ordering::SeqCst) != req_id {
                debug!(
                    "Cancelled stale lyrics request {} for track {}",
                    req_id, track_key
                );
                return;
            }

            info!("Fetching lyrics from LRCLIB for: {} by {}", title, artist);
            let res = client
                .fetch_lyrics(&title, &artist, &album, duration_ms)
                .await;

            // Check again after network await
            if req_id_atomic.load(Ordering::SeqCst) != req_id {
                debug!(
                    "Discarding stale lyrics response {} for track {}",
                    req_id, track_key
                );
                return;
            }

            let state = match res {
                FetchResult::Synced(lines) => {
                    let cache = cache_lock.lock().await;
                    cache.put(
                        &track_key,
                        CachedLyrics::Synced {
                            lines: lines.clone(),
                        },
                    );
                    LyricsState::Synced { track_key, lines }
                }
                FetchResult::Plain(text) => {
                    let cache = cache_lock.lock().await;
                    cache.put(&track_key, CachedLyrics::Plain { text: text.clone() });
                    LyricsState::Plain { track_key, text }
                }
                FetchResult::Instrumental => {
                    let cache = cache_lock.lock().await;
                    cache.put(&track_key, CachedLyrics::Instrumental);
                    LyricsState::Instrumental { track_key }
                }
                FetchResult::NotFound => {
                    let cache = cache_lock.lock().await;
                    cache.put(&track_key, CachedLyrics::NotFound);
                    LyricsState::NotFound { track_key }
                }
                FetchResult::Error(msg) => LyricsState::Error {
                    track_key,
                    message: msg,
                },
            };

            {
                let mut guard = state_lock.lock().await;
                *guard = Some(state.clone());
            }
            let _ = tx.send(state);
        });
    }
}
