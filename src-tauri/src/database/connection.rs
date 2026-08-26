use crate::error::{AppError, AppResult};
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;
use tracing::info;

pub fn get_database_path() -> PathBuf {
    let app_data = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());
    let dir = PathBuf::from(app_data).join("SpotlightForWindows");
    let _ = fs::create_dir_all(&dir);
    dir.join("spotlight.db")
}

pub fn open_connection() -> AppResult<Connection> {
    let db_path = get_database_path();
    info!("Opening SQLite database at: {:?}", db_path);

    let conn = Connection::open(&db_path)
        .map_err(|e| AppError::Database(format!("Failed to open SQLite database: {}", e)))?;

    // Apply performance pragmas from Section 13
    // PRAGMAs that return result rows must be queried rather than executed
    conn.query_row("PRAGMA journal_mode = WAL", [], |_| Ok(()))
        .map_err(|e| AppError::Database(format!("Failed to configure SQLite pragmas: {}", e)))?;
    conn.query_row("PRAGMA mmap_size = 67108864", [], |_| Ok(()))
        .map_err(|e| AppError::Database(format!("Failed to configure SQLite pragmas: {}", e)))?;

    // PRAGMAs that perform actions without returning rows
    conn.execute_batch(
        "
        PRAGMA synchronous = NORMAL;
        PRAGMA foreign_keys = ON;
        PRAGMA temp_store = MEMORY;
        PRAGMA cache_size = -8000;
        ",
    )
    .map_err(|e| AppError::Database(format!("Failed to configure SQLite pragmas: {}", e)))?;

    Ok(conn)
}
