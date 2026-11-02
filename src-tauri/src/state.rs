use crate::lyrics::LyricsManager;
use crate::media::MediaSource;
use crate::settings::AppSettings;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub media_source: Arc<dyn MediaSource>,
    pub lyrics_manager: Arc<LyricsManager>,
    pub settings: Arc<Mutex<AppSettings>>,
    pub settings_path: PathBuf,
}

