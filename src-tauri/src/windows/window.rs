use tauri::{AppHandle, Emitter, Manager, WebviewWindow};
use tracing::{error, info};

#[cfg(windows)]
use windows::Win32::Foundation::POINT;
#[cfg(windows)]
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromPoint, HMONITOR, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

/// Centers the window on the monitor that currently contains the cursor.
pub fn position_window_on_active_monitor(window: &WebviewWindow) {
    #[cfg(windows)]
    unsafe {
        let mut cursor_point = POINT::default();
        if GetCursorPos(&mut cursor_point).is_ok() {
            let monitor: HMONITOR = MonitorFromPoint(cursor_point, MONITOR_DEFAULTTONEAREST);
            let mut monitor_info = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };

            if GetMonitorInfoW(monitor, &mut monitor_info).as_bool() {
                let rc_work = monitor_info.rcWork;
                let monitor_width = rc_work.right - rc_work.left;
                let monitor_height = rc_work.bottom - rc_work.top;

                let window_width = 640;
                let window_height = 480;

                // Center horizontally, position in top-third vertically
                let x = rc_work.left + (monitor_width - window_width) / 2;
                let y = rc_work.top + (monitor_height - window_height) / 3;

                let _ = window
                    .set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }));
                return;
            }
        }
    }

    // Fallback: standard center
    let _ = window.center();
}

/// Reveal and focus the launcher window.
pub fn show_launcher(window: &WebviewWindow) {
    position_window_on_active_monitor(window);
    let _ = window.show();
    let _ = window.set_focus();
    let _ = window.emit("launcher-shown", ());
    info!("Launcher opened on active monitor");
}

/// Toggle launcher window visibility.
pub fn toggle_launcher(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        match window.is_visible() {
            Ok(true) => {
                let _ = window.hide();
            }
            Ok(false) => {
                show_launcher(&window);
            }
            Err(e) => {
                error!("Failed to query window visibility: {}", e);
            }
        }
    }
}
