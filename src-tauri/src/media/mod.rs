pub mod stub;
#[cfg(windows)]
pub mod windows;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PlaybackStatus {
    Playing,
    Paused,
    Stopped,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RepeatMode {
    Off,
    Track,
    List,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NowPlaying {
    pub track_key: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_ms: u64,
    pub artwork: Option<String>,
    pub is_ad: bool,
    pub source_app: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackState {
    pub status: PlaybackStatus,
    pub position_ms: u64,
    pub rate: f64,
    pub shuffle: Option<bool>,
    pub repeat: Option<RepeatMode>,
    pub can_play_pause: bool,
    pub can_next: bool,
    pub can_prev: bool,
    pub can_seek: bool,
    pub can_shuffle: bool,
    pub can_repeat: bool,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            status: PlaybackStatus::None,
            position_ms: 0,
            rate: 1.0,
            shuffle: None,
            repeat: None,
            can_play_pause: false,
            can_next: false,
            can_prev: false,
            can_seek: false,
            can_shuffle: false,
            can_repeat: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MediaEvent {
    NowPlaying(Option<NowPlaying>),
    Playback(PlaybackState),
}

#[async_trait::async_trait]
pub trait MediaSource: Send + Sync {
    async fn current_now_playing(&self) -> Option<NowPlaying>;
    async fn current_playback(&self) -> PlaybackState;
    async fn play_pause(&self) -> Result<(), String>;
    async fn next(&self) -> Result<(), String>;
    async fn prev(&self) -> Result<(), String>;
    async fn seek(&self, position_ms: u64) -> Result<(), String>;
    async fn toggle_shuffle(&self) -> Result<(), String>;
    async fn cycle_repeat(&self) -> Result<(), String>;
    fn set_source_preference(&self, preference: String);
}

