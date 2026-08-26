use crate::core::state::AppSettings;
use crate::error::{AppError, AppResult};
use rusqlite::{params, Connection};

pub fn load_settings_from_db(conn: &Connection) -> AppResult<Option<AppSettings>> {
    let mut stmt = conn
        .prepare_cached("SELECT value FROM settings WHERE key = 'app_settings';")
        .map_err(|e| AppError::Database(format!("Prepare load_settings error: {}", e)))?;

    let mut rows = stmt
        .query([])
        .map_err(|e| AppError::Database(format!("Query load_settings error: {}", e)))?;

    if let Some(row) = rows.next().map_err(|e| AppError::Database(e.to_string()))? {
        let json_str: String = row.get(0).map_err(|e| AppError::Database(e.to_string()))?;
        let settings: AppSettings = serde_json::from_str(&json_str)
            .map_err(|e| AppError::Config(format!("Failed to parse settings JSON: {}", e)))?;
        Ok(Some(settings))
    } else {
        Ok(None)
    }
}

pub fn save_settings_to_db(conn: &Connection, settings: &AppSettings) -> AppResult<()> {
    let json_str = serde_json::to_string(settings)
        .map_err(|e| AppError::Config(format!("Failed to serialize settings: {}", e)))?;

    conn.execute(
        "INSERT INTO settings (key, value) VALUES ('app_settings', ?1)
         ON CONFLICT(key) DO UPDATE SET value = ?1;",
        params![json_str],
    )
    .map_err(|e| AppError::Database(format!("Save settings error: {}", e)))?;

    Ok(())
}
