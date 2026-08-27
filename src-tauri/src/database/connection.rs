use crate::error::{AppError, AppResult};
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;
use tracing::{error, info, warn};

pub fn get_database_path() -> PathBuf {
    let app_data = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());
    let dir = PathBuf::from(app_data).join("SpotlightForWindows");
    let _ = fs::create_dir_all(&dir);
    dir.join("spotlight.db")
}

pub fn remove_database_files() {
    let db_path = get_database_path();
    let wal_path = db_path.with_extension("db-wal");
    let shm_path = db_path.with_extension("db-shm");
    let _ = fs::remove_file(&db_path);
    let _ = fs::remove_file(&wal_path);
    let _ = fs::remove_file(&shm_path);
    warn!("Database files removed due to corruption auto-recovery.");
}

pub fn open_connection() -> AppResult<Connection> {
    match try_open_connection() {
        Ok(conn) => Ok(conn),
        Err(e) => {
            error!("Failed to open SQLite connection ({}). Performing auto-recovery...", e);
            remove_database_files();
            try_open_connection()
        }
    }
}

fn try_open_connection() -> AppResult<Connection> {
    let db_path = get_database_path();
    info!("Opening SQLite database at: {:?}", db_path);

    let conn = Connection::open(&db_path)
        .map_err(|e| AppError::Database(format!("Failed to open SQLite database: {}", e)))?;

    // Quick integrity check
    let check: Result<String, _> = conn.query_row("PRAGMA quick_check(1);", [], |row| row.get(0));
    if let Ok(res) = check {
        if res.to_lowercase() != "ok" {
            return Err(AppError::Database(format!("Integrity check failed: {}", res)));
        }
    }

    // Apply performance pragmas from Section 13
    conn.query_row("PRAGMA journal_mode = WAL", [], |_| Ok(()))
        .map_err(|e| AppError::Database(format!("Failed to configure SQLite pragmas: {}", e)))?;
    conn.query_row("PRAGMA mmap_size = 67108864", [], |_| Ok(()))
        .map_err(|e| AppError::Database(format!("Failed to configure SQLite pragmas: {}", e)))?;

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
