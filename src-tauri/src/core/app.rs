use crate::commands;
use crate::core::process::LaunchArgs;
use crate::core::state::AppState;
use crate::hotkey::manager::HotkeyManager;
use crate::indexer::manager::IndexManager;
use crate::tray::manager::create_tray;
use std::sync::Arc;
use tauri::{App, Manager};
use tracing::{error, info};

pub fn create_app() -> tauri::Builder<tauri::Wry> {
    let builder = tauri::Builder::default();

    builder
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            info!("Second instance detected. Revealing primary instance window.");
            if let Some(window) = app.get_webview_window("main") {
                crate::windows::window::show_launcher(&window);
            }
        }))

        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            setup_app(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::search,
            commands::launch,
            commands::get_icon,
            commands::hide_launcher,
            commands::get_settings,
            commands::update_settings,
            commands::get_recent_results,
            commands::get_index_status,
            commands::rebuild_index,
            commands::get_app_info,
        ])
}

fn setup_app(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let args = LaunchArgs::parse();

    // 1. Initialize managed AppState
    let state = match AppState::new() {
        Ok(s) => Arc::new(s),
        Err(e) => {
            error!("Failed to initialize AppState: {}", e);
            return Err(Box::new(e));
        }
    };
    app.manage(state.clone());

    // 2. Create system tray icon and menu
    create_tray(app.handle())?;

    // 3. Register global hotkey (Alt + Space by default)
    if let Err(e) = HotkeyManager::register_default(app.handle()) {
        error!("Global hotkey registration failed: {}", e);
    }

    // 4. Setup lifecycle behavior
    crate::core::lifecycle::on_app_ready(app.handle(), args.is_startup);

    // 5. Trigger initial background application index scan
    IndexManager::start_background_indexing(state);

    Ok(())
}
