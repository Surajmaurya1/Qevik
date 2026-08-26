use crate::error::{AppError, AppResult};

pub const DEFAULT_SHORTCUT: &str = "Alt+Space";

pub fn validate_shortcut(shortcut: &str) -> AppResult<()> {
    let trimmed = shortcut.trim();
    if trimmed.is_empty() {
        return Err(AppError::Hotkey("Shortcut cannot be empty".into()));
    }

    // Supported modifiers: Alt, Ctrl, Shift, Super (Win)
    let parts: Vec<&str> = trimmed.split('+').map(|s| s.trim()).collect();
    if parts.len() < 2 {
        return Err(AppError::Hotkey(
            "Shortcut must contain at least one modifier (e.g., Alt+Space, Ctrl+Space)".into(),
        ));
    }

    Ok(())
}
