use tauri::{AppHandle, Manager};
use tracing::info;

pub fn on_app_ready(app: &AppHandle, is_startup: bool) {
    info!("Spotlight core initialized. is_startup: {}", is_startup);

    if let Some(window) = app.get_webview_window("main") {
        if !is_startup {
            // Show launcher window on manual startup
            crate::windows::window::show_launcher(&window);
        } else {
            // Keep window hidden in background on Windows autostart
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
