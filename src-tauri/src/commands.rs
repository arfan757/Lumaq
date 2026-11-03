use crate::lyrics::LyricsState;
use crate::media::{NowPlaying, PlaybackState};
use crate::settings::AppSettings;
use crate::state::AppState;
use serde::Serialize;
use tauri::{AppHandle, State, Window};
use tracing::info;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub now_playing: Option<NowPlaying>,
    pub playback: PlaybackState,
    pub lyrics: Option<LyricsState>,
    pub settings: AppSettings,
}

#[tauri::command]
pub async fn get_snapshot(state: State<'_, AppState>) -> Result<Snapshot, String> {
    let now_playing = state.media_source.current_now_playing().await;
    let playback = state.media_source.current_playback().await;
    let lyrics = state.lyrics_manager.current_lyrics().await;
    let settings = { state.settings.lock().unwrap().clone() };

    Ok(Snapshot {
        now_playing,
        playback,
        lyrics,
        settings,
    })
}

#[tauri::command]
pub async fn media_play_pause(state: State<'_, AppState>) -> Result<(), String> {
    state.media_source.play_pause().await
}

#[tauri::command]
pub async fn media_next(state: State<'_, AppState>) -> Result<(), String> {
    state.media_source.next().await
}

#[tauri::command]
pub async fn media_prev(state: State<'_, AppState>) -> Result<(), String> {
    state.media_source.prev().await
}

#[tauri::command]
pub async fn media_seek(state: State<'_, AppState>, position_ms: u64) -> Result<(), String> {
    state.media_source.seek(position_ms).await
}

#[tauri::command]
pub async fn media_toggle_shuffle(state: State<'_, AppState>) -> Result<(), String> {
    state.media_source.toggle_shuffle().await
}

#[tauri::command]
pub async fn media_cycle_repeat(state: State<'_, AppState>) -> Result<(), String> {
    state.media_source.cycle_repeat().await
}

#[tauri::command]
pub async fn lyrics_refetch(state: State<'_, AppState>) -> Result<(), String> {
    if let Some(np) = state.media_source.current_now_playing().await {
        state
            .lyrics_manager
            .on_track_change(
                np.track_key,
                np.title,
                np.artist,
                np.album,
                np.duration_ms,
                np.is_ad,
            )
            .await;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let settings = { state.settings.lock().unwrap().clone() };
    Ok(settings)
}

#[tauri::command]
pub async fn set_settings(
    state: State<'_, AppState>,
    window: Window,
    new_settings: AppSettings,
) -> Result<AppSettings, String> {
    let mut validated = new_settings;
    validated.validate_and_clamp();

    {
        let mut guard = state.settings.lock().unwrap();
        *guard = validated.clone();
        let _ = guard.save(&state.settings_path);
    }

    // Apply always_on_top. When disabled, keep the widget pinned to the
    // desktop instead of letting it float over other applications.
    let _ = window.set_always_on_top(validated.always_on_top);
    #[cfg(windows)]
    if !validated.always_on_top {
        if let Ok(hwnd) = window.hwnd() {
            crate::apply_manual_desktop_pin(hwnd.0);
        }
    }

    // Apply click_through
    let _ = window.set_ignore_cursor_events(validated.click_through);

    // Apply source preference
    let pref_str = match validated.source_preference {
        crate::settings::SourcePreference::SpotifyFirst => "spotify-first".to_string(),
        crate::settings::SourcePreference::Any => "any".to_string(),
    };
    state.media_source.set_source_preference(pref_str);

    Ok(validated)
}

#[tauri::command]
pub async fn set_click_through(
    state: State<'_, AppState>,
    window: Window,
    enabled: bool,
) -> Result<(), String> {
    {
        let mut guard = state.settings.lock().unwrap();
        guard.click_through = enabled;
        let _ = guard.save(&state.settings_path);
    }
    let _ = window.set_ignore_cursor_events(enabled);
    Ok(())
}

#[tauri::command]
pub fn quit(app: AppHandle) {
    info!("Quit command invoked, terminating application gracefully");
    app.exit(0);
}
