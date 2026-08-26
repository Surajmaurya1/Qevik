use tauri::{AppHandle, Manager};
use tracing::info;

pub fn on_app_ready(app: &AppHandle, is_startup: bool) {
    info!("Spotlight core initialized. is_startup: {}", is_startup);
    
    // If not started in background --startup mode, we can optionally keep window hidden until hotkey
    if let Some(window) = app.get_webview_window("main") {
        if !is_startup {
            // Keep window hidden initially; hotkey will show it
            let _ = window.hide();
        }
    }
}

pub fn graceful_shutdown(app: &AppHandle) {
    info!("Initiating graceful shutdown of Spotlight core...");
    // 1. Hide windows
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    // 2. Exit cleanly
    app.exit(0);
}
