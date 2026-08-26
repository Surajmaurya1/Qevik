use crate::error::{AppError, AppResult};
use crate::hotkey::shortcuts::{validate_shortcut, DEFAULT_SHORTCUT};
use crate::windows::window::toggle_launcher;
use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use tracing::{info, warn};

pub struct HotkeyManager;

impl HotkeyManager {
    /// Register the global hotkey on startup.
    pub fn register_default(app: &AppHandle) -> AppResult<()> {
        Self::register(app, DEFAULT_SHORTCUT)
    }

    /// Register a specific shortcut string.
    pub fn register(app: &AppHandle, shortcut_str: &str) -> AppResult<()> {
        validate_shortcut(shortcut_str)?;

        let shortcut: Shortcut = shortcut_str.parse().map_err(|e| {
            AppError::Hotkey(format!("Invalid shortcut format '{}': {}", shortcut_str, e))
        })?;

        let app_handle = app.clone();
        let plugin = app.global_shortcut();

        // If already registered, unregister first
        if plugin.is_registered(shortcut) {
            let _ = plugin.unregister(shortcut);
        }

        match plugin.on_shortcut(shortcut, move |_app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                toggle_launcher(&app_handle);
            }
        }) {
            Ok(_) => {
                info!("Global hotkey successfully registered: {}", shortcut_str);
                Ok(())
            }
            Err(e) => {
                warn!(
                    "Failed to register global hotkey '{}': {}. Hotkey may conflict with another application.",
                    shortcut_str, e
                );
                Err(AppError::Hotkey(format!("Registration failed: {}", e)))
            }
        }
    }

    /// Change hotkey to a new combination.
    #[allow(dead_code)]
    pub fn update(app: &AppHandle, old_shortcut: &str, new_shortcut: &str) -> AppResult<()> {
        validate_shortcut(new_shortcut)?;

        let plugin = app.global_shortcut();
        if let Ok(old) = old_shortcut.parse::<Shortcut>() {
            let _ = plugin.unregister(old);
        }

        Self::register(app, new_shortcut)
    }
}
