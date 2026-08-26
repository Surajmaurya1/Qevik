use crate::error::{AppError, AppResult};
use rusqlite::Connection;
use tracing::info;

const INITIAL_MIGRATION: &str = include_str!("../../migrations/0001_initial.sql");

pub fn run_migrations(conn: &mut Connection) -> AppResult<()> {
    info!("Running database migrations...");

    // Ensure metadata table exists to track migrations
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS metadata (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );",
    )
    .map_err(|e| AppError::Database(format!("Failed to create metadata table: {}", e)))?;

    // Check if initial migration has already run
    let already_run: bool = conn
        .query_row(
            "SELECT 1 FROM metadata WHERE key = 'migration_0001'",
            [],
            |_| Ok(true),
        )
        .unwrap_or(false);

    if !already_run {
        let tx = conn
            .transaction()
            .map_err(|e| AppError::Database(format!("Transaction error: {}", e)))?;

        tx.execute_batch(INITIAL_MIGRATION)
            .map_err(|e| AppError::Database(format!("Failed executing 0001_initial.sql: {}", e)))?;

        tx.execute(
            "INSERT INTO metadata (key, value) VALUES ('migration_0001', 'applied')",
            [],
        )
        .map_err(|e| AppError::Database(format!("Failed recording migration: {}", e)))?;

        tx.commit()
            .map_err(|e| AppError::Database(format!("Failed committing migration: {}", e)))?;

        info!("Migration 0001_initial.sql applied successfully.");
    }

    // Ensure all FTS triggers exist on existing databases
    conn.execute_batch(
        "
        CREATE TRIGGER IF NOT EXISTS applications_ai AFTER INSERT ON applications BEGIN
          INSERT INTO applications_fts(rowid, display_name, exe_path) VALUES (new.rowid, new.display_name, new.exe_path);
        END;
        CREATE TRIGGER IF NOT EXISTS applications_ad AFTER DELETE ON applications BEGIN
          INSERT INTO applications_fts(applications_fts, rowid, display_name, exe_path) VALUES('delete', old.rowid, old.display_name, old.exe_path);
        END;
        CREATE TRIGGER IF NOT EXISTS applications_au AFTER UPDATE ON applications BEGIN
          INSERT INTO applications_fts(applications_fts, rowid, display_name, exe_path) VALUES('delete', old.rowid, old.display_name, old.exe_path);
          INSERT INTO applications_fts(rowid, display_name, exe_path) VALUES (new.rowid, new.display_name, new.exe_path);
        END;

        CREATE TRIGGER IF NOT EXISTS files_ai AFTER INSERT ON files BEGIN
          INSERT INTO files_fts(rowid, name, display_name, path) VALUES (new.rowid, new.name, new.display_name, new.path);
        END;
        CREATE TRIGGER IF NOT EXISTS files_ad AFTER DELETE ON files BEGIN
          INSERT INTO files_fts(files_fts, rowid, name, display_name, path) VALUES('delete', old.rowid, old.name, old.display_name, old.path);
        END;
        CREATE TRIGGER IF NOT EXISTS files_au AFTER UPDATE ON files BEGIN
          INSERT INTO files_fts(files_fts, rowid, name, display_name, path) VALUES('delete', old.rowid, old.name, old.display_name, old.path);
          INSERT INTO files_fts(rowid, name, display_name, path) VALUES (new.rowid, new.name, new.display_name, new.path);
        END;

        CREATE TRIGGER IF NOT EXISTS folders_ai AFTER INSERT ON folders BEGIN
          INSERT INTO folders_fts(rowid, name, path) VALUES (new.rowid, new.name, new.path);
        END;
        CREATE TRIGGER IF NOT EXISTS folders_ad AFTER DELETE ON folders BEGIN
          INSERT INTO folders_fts(folders_fts, rowid, name, path) VALUES('delete', old.rowid, old.name, old.path);
        END;
        CREATE TRIGGER IF NOT EXISTS folders_au AFTER UPDATE ON folders BEGIN
          INSERT INTO folders_fts(folders_fts, rowid, name, path) VALUES('delete', old.rowid, old.name, old.path);
          INSERT INTO folders_fts(rowid, name, path) VALUES (new.rowid, new.name, new.path);
        END;

        -- Clean up any legacy history rows that recorded IDs as names
        DELETE FROM history WHERE result_name LIKE 'file_%' OR result_name LIKE 'app_%' OR result_name LIKE 'folder_%';
        ",
    )
    .map_err(|e| AppError::Database(format!("Failed to ensure FTS triggers: {}", e)))?;

    Ok(())
}
