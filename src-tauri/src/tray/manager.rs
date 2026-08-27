use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};
use tracing::info;

pub fn create_tray(app: &AppHandle) -> Result<TrayIcon, tauri::Error> {
    let open_item = MenuItem::with_id(app, "open", "Open Spotlight", true, None::<&str>)?;
    let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let reindex_item = MenuItem::with_id(app, "reindex", "Re-index Files", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit Spotlight", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[&open_item, &settings_item, &reindex_item, &quit_item],
    )?;

    let tray = TrayIconBuilder::with_id("spotlight-tray")
        .tooltip("Spotlight for Windows")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => {
                if let Some(window) = app.get_webview_window("main") {
                    crate::windows::window::show_launcher(&window);
                }
            }
            "settings" => {
                info!("Settings requested from tray");
                if let Some(window) = app.get_webview_window("main") {
                    use tauri::Emitter;
                    crate::windows::window::show_launcher(&window);
                    let _ = window.emit("open-settings", ());
                }
            }
            "reindex" => {
                info!("Manual re-index triggered from tray");
                if let Some(state) = app.try_state::<std::sync::Arc<crate::core::state::AppState>>() {
                    crate::indexer::manager::IndexManager::start_background_indexing(state.inner().clone());
                }
            }
            "quit" => {
                crate::core::lifecycle::graceful_shutdown(app);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button, .. } = event {
                if button == tauri::tray::MouseButton::Left {
                    let app = tray.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        if window.is_visible().unwrap_or(false) {
                            let _ = window.hide();
                        } else {
                            crate::windows::window::show_launcher(&window);
                        }
                    }
                }
            }
        })
        .build(app)?;

    Ok(tray)
}
