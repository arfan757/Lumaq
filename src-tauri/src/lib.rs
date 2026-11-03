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

#[cfg(windows)]
use std::sync::atomic::{AtomicBool, Ordering};
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
            info!("Second instance launched; showing existing desktop widget");
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_skip_taskbar(true);
                #[cfg(windows)]
                apply_desktop_pinning(app, &window);
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
                let _ = window.show();
                let _ = window.center();
                let _ = window.set_skip_taskbar(true);
                let _ = window.set_ignore_cursor_events(settings.click_through);

                // Desktop widget: sit at the bottom of the z-order instead of
                // floating over other applications, and never steal focus.
                #[cfg(windows)]
                apply_desktop_pinning(&app_handle, &window);

                info!(
                    "Main window initialized: visible={:?}, position={:?}",
                    window.is_visible(),
                    window.outer_position()
                );

                // Clicking the widget (or the taskbar) raises it; drop it back
                // to the desktop as soon as it gains focus.
                #[cfg(windows)]
                {
                    let win_for_focus = window.clone();
                    let handle_for_focus = app_handle.clone();
                    window.on_window_event(move |event| {
                        if let tauri::WindowEvent::Focused(true) = event {
                            apply_desktop_pinning(&handle_for_focus, &win_for_focus);
                        }
                    });

                    // "Show Desktop" (Win+D / taskbar button) does not minimize
                    // windows: it raises the desktop above everything, covering
                    // our bottom-most widget. Detect that state and temporarily
                    // lift the widget above the desktop, then drop it back down
                    // as soon as a real application regains the foreground.
                    let win_for_watch = window.clone();
                    let handle_for_watch = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        let Ok(hwnd) = win_for_watch.hwnd() else {
                            return;
                        };
                        let raw_hwnd = hwnd.0 as usize;
                        let mut desktop_shown = false;

                        loop {
                            tokio::time::sleep(std::time::Duration::from_millis(250)).await;

                            let always_on_top = handle_for_watch
                                .state::<AppState>()
                                .settings
                                .lock()
                                .map(|s| s.always_on_top)
                                .unwrap_or(false);

                            if always_on_top {
                                if desktop_shown {
                                    desktop_shown = false;
                                    DESKTOP_SHOWN.store(false, Ordering::Relaxed);
                                }
                                continue;
                            }

                            if !desktop_shown {
                                if desktop_covers_window(raw_hwnd) {
                                    desktop_shown = true;
                                    DESKTOP_SHOWN.store(true, Ordering::Relaxed);
                                    set_window_topmost(raw_hwnd, true);
                                }
                            } else if foreground_is_foreign_non_desktop() {
                                desktop_shown = false;
                                DESKTOP_SHOWN.store(false, Ordering::Relaxed);
                                set_window_topmost(raw_hwnd, false);
                            }
                        }
                    });
                }

                // Multi-monitor sanity check: ensure window is within monitor bounds
                let win_clone = window.clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                    validate_window_bounds(&win_clone);
                });
            } else {
                warn!("Main webview window 'main' not found in setup!");
            }

            // Event forwarding loop with robust Lagged handling so receivers never die
            let handle_np = app_handle.clone();
            let lyrics_for_np = Arc::clone(&lyrics_manager);
            #[cfg(windows)]
            tauri::async_runtime::spawn(async move {
                loop {
                    match np_rx.recv().await {
                        Ok(np_opt) => {
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
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                            tracing::debug!(
                                "NowPlaying broadcast channel lagged by {} messages",
                                skipped
                            );
                            continue;
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                            tracing::info!("NowPlaying broadcast channel closed");
                            break;
                        }
                    }
                }
            });

            #[cfg(windows)]
            let handle_pb = app_handle.clone();
            #[cfg(windows)]
            tauri::async_runtime::spawn(async move {
                loop {
                    match pb_rx.recv().await {
                        Ok(pb) => {
                            let _ = handle_pb.emit("lumaq://playback", &pb);
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                            tracing::debug!(
                                "Playback broadcast channel lagged by {} messages",
                                skipped
                            );
                            continue;
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                            tracing::info!("Playback broadcast channel closed");
                            break;
                        }
                    }
                }
            });

            let handle_lyrics = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    match lyrics_rx.recv().await {
                        Ok(lyr) => {
                            let _ = handle_lyrics.emit("lumaq://lyrics", &lyr);
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                            tracing::debug!(
                                "Lyrics broadcast channel lagged by {} messages",
                                skipped
                            );
                            continue;
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                            tracing::info!("Lyrics broadcast channel closed");
                            break;
                        }
                    }
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

    let tray_icon = tauri::image::Image::from_bytes(include_bytes!("../icons/tray.png"))?;

    let tray = TrayIconBuilder::new()
        .icon(tray_icon)
        .tooltip("Lumaq")
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
                            let _ = window.set_skip_taskbar(true);
                            #[cfg(windows)]
                            apply_desktop_pinning(app, &window);
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
                    #[cfg(windows)]
                    apply_desktop_pinning(app, &window);
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
                            let _ = window.set_skip_taskbar(true);
                            #[cfg(windows)]
                            apply_desktop_pinning(app, &window);
                        }
                    }
                }
            }
        })
        .build(app)?;

    // TrayIcon is ref-counted: it disappears when the last handle is dropped.
    app.manage(tray);

    Ok(())
}

#[cfg(windows)]
static DESKTOP_SHOWN: AtomicBool = AtomicBool::new(false);

#[cfg(windows)]
fn hwnd_from_raw(raw: usize) -> windows::Win32::Foundation::HWND {
    windows::Win32::Foundation::HWND(raw as *mut std::ffi::c_void)
}

#[cfg(windows)]
fn window_class_name(hwnd: windows::Win32::Foundation::HWND) -> String {
    use windows::Win32::UI::WindowsAndMessaging::GetClassNameW;

    let mut buffer = [0u16; 256];
    let len = unsafe { GetClassNameW(hwnd, &mut buffer) };
    if len <= 0 {
        return String::new();
    }
    String::from_utf16_lossy(&buffer[..len as usize])
}

#[cfg(windows)]
fn is_desktop_class(class_name: &str) -> bool {
    matches!(
        class_name,
        "Progman" | "WorkerW" | "SHELLDLL_DefView" | "SysListView32"
    )
}

/// True when the point at the center of our window is covered by the desktop
/// instead of an application (i.e. "Show Desktop" is active).
#[cfg(windows)]
fn desktop_covers_window(raw_hwnd: usize) -> bool {
    use windows::Win32::Foundation::POINT;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetAncestor, GetWindowRect, GetWindowThreadProcessId, WindowFromPoint, GA_ROOT,
    };

    unsafe {
        let hwnd = hwnd_from_raw(raw_hwnd);
        let mut rect = windows::Win32::Foundation::RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_err() {
            return false;
        }

        let point = POINT {
            x: (rect.left + rect.right) / 2,
            y: (rect.top + rect.bottom) / 2,
        };
        let under = WindowFromPoint(point);
        if under.0.is_null() {
            return false;
        }

        let root = GetAncestor(under, GA_ROOT);
        let root = if root.0.is_null() { under } else { root };

        let mut pid = 0u32;
        GetWindowThreadProcessId(root, Some(&mut pid));
        if pid == std::process::id() {
            return false;
        }

        is_desktop_class(&window_class_name(root))
    }
}

/// True when the foreground window belongs to another process and is not the
/// desktop (used to detect that "Show Desktop" has ended).
#[cfg(windows)]
fn foreground_is_foreign_non_desktop() -> bool {
    use windows::Win32::UI::WindowsAndMessaging::{
        GetAncestor, GetForegroundWindow, GetWindowThreadProcessId, GA_ROOT,
    };

    unsafe {
        let foreground = GetForegroundWindow();
        if foreground.0.is_null() {
            return false;
        }

        let mut pid = 0u32;
        GetWindowThreadProcessId(foreground, Some(&mut pid));
        if pid == std::process::id() {
            return false;
        }

        let root = GetAncestor(foreground, GA_ROOT);
        let root = if root.0.is_null() { foreground } else { root };
        !is_desktop_class(&window_class_name(root))
    }
}

#[cfg(windows)]
fn set_window_topmost(raw_hwnd: usize, topmost: bool) {
    use windows::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, HWND_BOTTOM, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOOWNERZORDER,
        SWP_NOSIZE,
    };

    let insert_after = if topmost { HWND_TOPMOST } else { HWND_BOTTOM };
    let _ = unsafe {
        SetWindowPos(
            hwnd_from_raw(raw_hwnd),
            insert_after,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOOWNERZORDER,
        )
    };
}

/// Push the widget to the bottom of the normal window z-order so it behaves
/// like a desktop widget instead of floating over other applications.
#[cfg(windows)]
pub(crate) fn send_to_desktop(hwnd_raw: *mut std::ffi::c_void) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, HWND_BOTTOM, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOOWNERZORDER, SWP_NOSIZE,
    };

    let result = unsafe {
        SetWindowPos(
            HWND(hwnd_raw),
            HWND_BOTTOM,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOOWNERZORDER,
        )
    };

    if let Err(err) = result {
        warn!("Failed to pin window to the desktop: {}", err);
    }
}

#[cfg(windows)]
pub(crate) fn apply_manual_desktop_pin(hwnd_raw: *mut std::ffi::c_void) {
    if DESKTOP_SHOWN.load(Ordering::Relaxed) {
        set_window_topmost(hwnd_raw as usize, true);
    } else {
        send_to_desktop(hwnd_raw);
    }
}

#[cfg(windows)]
fn pin_window_to_desktop(window: &tauri::WebviewWindow) {
    if let Ok(hwnd) = window.hwnd() {
        send_to_desktop(hwnd.0);
    }
}

#[cfg(windows)]
fn apply_desktop_pinning(app: &tauri::AppHandle, window: &tauri::WebviewWindow) {
    let always_on_top = app
        .state::<AppState>()
        .settings
        .lock()
        .map(|s| s.always_on_top)
        .unwrap_or(false);

    if always_on_top {
        let _ = window.set_always_on_top(true);
    } else if DESKTOP_SHOWN.load(Ordering::Relaxed) {
        // Stay above the desktop while "Show Desktop" is active.
        if let Ok(hwnd) = window.hwnd() {
            set_window_topmost(hwnd.0 as usize, true);
        }
    } else {
        let _ = window.set_always_on_top(false);
        pin_window_to_desktop(window);
    }
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
