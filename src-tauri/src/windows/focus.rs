use tauri::WebviewWindow;

pub fn set_foreground_focus(window: &WebviewWindow) {
    let _ = window.set_focus();
}
