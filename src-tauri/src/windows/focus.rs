use tauri::WebviewWindow;

#[allow(dead_code)]
pub fn set_foreground_focus(window: &WebviewWindow) {
    let _ = window.set_focus();
}
