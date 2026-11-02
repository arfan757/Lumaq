pub mod commands;
pub mod lyrics;
pub mod media;
pub mod settings;
pub mod state;

use commands::*;
use lyrics::LyricsManager;
#[cfg(not(windows))]
use media::stub::StubMediaSource;
#[cfg(windows)]
use media::windows::WindowsMediaSource;
use media::MediaSource;
use settings::AppSettings;
use state::AppState;

use std::sync::{Arc, Mutex};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Emitter, Manager, Position};
use tracing::{info, warn};

pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter("info,lumaq=debug")
        .init();

    info!("Starting Lumaq application backend");

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            info!("Second instance launched; focusing existing window");
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            // Setup System Tray
            setup_system_tray(app)?;

            let app_handle = app.handle().clone();
            let app_data_dir = app.path().app_data_dir().unwrap_or_else(|_| {
                dirs::data_dir()
                    .unwrap_or_else(|| std::path::PathBuf::from("."))
                    .join("lumaq")
            });
            let _ = std::fs::create_dir_all(&app_data_dir);
            let settings_path = app_data_dir.join("settings.json");
            let settings = AppSettings::load_or_default(&settings_path);

            // Initialize media source and lyrics manager
            #[cfg(windows)]
            let (media_source_impl, mut np_rx, mut pb_rx) = WindowsMediaSource::new();
            #[cfg(not(windows))]
            let media_source_impl = StubMediaSource::new();

            let media_source: Arc<dyn MediaSource> = Arc::new(media_source_impl);
            let (lyrics_mgr_impl, mut lyrics_rx) = LyricsManager::new(app_data_dir);
            let lyrics_manager = Arc::new(lyrics_mgr_impl);

            let app_state = AppState {
                media_source: Arc::clone(&media_source),
                lyrics_manager: Arc::clone(&lyrics_manager),
                settings: Arc::new(Mutex::new(settings.clone())),
                settings_path,
            };

            app.manage(app_state);

            // Main window configuration
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_always_on_top(settings.always_on_top);
                let _ = window.set_ignore_cursor_events(settings.click_through);

                // Multi-monitor sanity check: ensure window is within monitor bounds
                let win_clone = window.clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                    validate_window_bounds(&win_clone);
                });
            }

            // Event forwarding loop
            let handle_np = app_handle.clone();
            let lyrics_for_np = Arc::clone(&lyrics_manager);
            #[cfg(windows)]
            tauri::async_runtime::spawn(async move {
                while let Ok(np_opt) = np_rx.recv().await {
                    let _ = handle_np.emit("lumaq://now-playing", &np_opt);
                    if let Some(np) = np_opt {
                        lyrics_for_np
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
                }
            });

            #[cfg(windows)]
            let handle_pb = app_handle.clone();
            #[cfg(windows)]
            tauri::async_runtime::spawn(async move {
                while let Ok(pb) = pb_rx.recv().await {
                    let _ = handle_pb.emit("lumaq://playback", &pb);
                }
            });

            let handle_lyrics = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                while let Ok(lyr) = lyrics_rx.recv().await {
                    let _ = handle_lyrics.emit("lumaq://lyrics", &lyr);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_snapshot,
            media_play_pause,
            media_next,
            media_prev,
            media_seek,
            media_toggle_shuffle,
            media_cycle_repeat,
            lyrics_refetch,
            get_settings,
            set_settings,
            set_click_through,
            quit
        ])
        .run(tauri::generate_context!())
        .expect("error while running lumaq application");
}

fn setup_system_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let show_hide = MenuItem::with_id(app, "toggle_visible", "Show / Hide", true, None::<&str>)?;
    let click_through = MenuItem::with_id(
        app,
        "toggle_click_through",
        "Toggle Click-through (Recoverable)",
        true,
        None::<&str>,
    )?;
    let always_on_top = MenuItem::with_id(
        app,
        "toggle_always_on_top",
        "Always on Top",
        true,
        None::<&str>,
    )?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit Lumaq", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[&show_hide, &click_through, &always_on_top, &quit_item],
    )?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "toggle_visible" => {
                if let Some(window) = app.get_webview_window("main") {
                    if let Ok(visible) = window.is_visible() {
                        if visible {
                            let _ = window.hide();
                        } else {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
            }
            "toggle_click_through" => {
                if let Some(window) = app.get_webview_window("main") {
                    let state = app.state::<AppState>();
                    let new_val = {
                        let mut guard = state.settings.lock().unwrap();
                        guard.click_through = !guard.click_through;
                        let _ = guard.save(&state.settings_path);
                        guard.click_through
                    };
                    let _ = window.set_ignore_cursor_events(new_val);
                    info!("Click-through toggled from tray: {}", new_val);
                }
            }
            "toggle_always_on_top" => {
                if let Some(window) = app.get_webview_window("main") {
                    let state = app.state::<AppState>();
                    let new_val = {
                        let mut guard = state.settings.lock().unwrap();
                        guard.always_on_top = !guard.always_on_top;
                        let _ = guard.save(&state.settings_path);
                        guard.always_on_top
                    };
                    let _ = window.set_always_on_top(new_val);
                    info!("Always-on-top toggled from tray: {}", new_val);
                }
            }
            "quit" => {
                info!("Quit clicked from system tray");
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if let Ok(visible) = window.is_visible() {
                        if visible {
                            let _ = window.hide();
                        } else {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

fn validate_window_bounds(window: &tauri::WebviewWindow) {
    let position = match window.outer_position() {
        Ok(p) => p,
        Err(_) => return,
    };

    let monitors = match window.available_monitors() {
        Ok(m) => m,
        Err(_) => return,
    };

    let is_on_any_monitor = monitors.iter().any(|m| {
        let m_pos = m.position();
        let m_size = m.size();
        position.x >= m_pos.x - 50
            && position.x < (m_pos.x + m_size.width as i32)
            && position.y >= m_pos.y - 50
            && position.y < (m_pos.y + m_size.height as i32)
    });

    if !is_on_any_monitor {
        warn!("Saved window position is off-screen! Resetting to bottom-left of primary monitor.");
        if let Ok(Some(primary)) = window.primary_monitor() {
            let m_pos = primary.position();
            let m_size = primary.size();
            let target_x = m_pos.x + 40;
            let target_y = m_pos.y + (m_size.height as i32) - 360;
            let _ = window.set_position(Position::Physical(tauri::PhysicalPosition {
                x: target_x,
                y: target_y,
            }));
        }
    }
}
