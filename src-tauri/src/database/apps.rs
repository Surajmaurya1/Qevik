use crate::error::{AppError, AppResult};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationRecord {
    pub id: String,
    pub display_name: String,
    pub exe_path: String,
    pub shortcut_path: Option<String>,
    pub arguments: Option<String>,
    pub icon_path: Option<String>,
    pub icon_index: i32,
    pub source: String,
    pub indexed_at: i64,
    pub updated_at: i64,
}

pub fn upsert_applications(
    conn: &mut Connection,
    apps: &[ApplicationRecord],
) -> AppResult<usize> {
    let tx = conn
        .transaction()
        .map_err(|e| AppError::Database(format!("Transaction start failed: {}", e)))?;

    let mut count = 0;
    {
        let mut stmt = tx
            .prepare_cached(
                "INSERT INTO applications (
                    id, display_name, exe_path, shortcut_path, arguments,
                    icon_path, icon_index, source, indexed_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                ON CONFLICT(exe_path) DO UPDATE SET
                    display_name = excluded.display_name,
                    shortcut_path = excluded.shortcut_path,
                    arguments = excluded.arguments,
                    icon_path = excluded.icon_path,
                    icon_index = excluded.icon_index,
                    updated_at = excluded.updated_at;",
            )
            .map_err(|e| AppError::Database(format!("Prepare error: {}", e)))?;

        for app in apps {
            stmt.execute(params![
                app.id,
                app.display_name,
                app.exe_path,
                app.shortcut_path,
                app.arguments,
                app.icon_path,
                app.icon_index,
                app.source,
                app.indexed_at,
                app.updated_at,
            ])
            .map_err(|e| AppError::Database(format!("Insert app error: {}", e)))?;
            count += 1;
        }
    }

    tx.commit()
        .map_err(|e| AppError::Database(format!("Transaction commit failed: {}", e)))?;

    Ok(count)
}

pub fn build_fts_query(raw: &str) -> Option<String> {
    let tokens: Vec<String> = raw
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| format!("\"{}\"*", t.replace('"', "\"\"")))
        .collect();

    if tokens.is_empty() {
        None
    } else {
        Some(tokens.join(" "))
    }
}

pub fn search_applications_fts(
    conn: &Connection,
    query: &str,
    limit: usize,
) -> AppResult<Vec<ApplicationRecord>> {
    let like_pattern = format!("%{}%", query.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_"));
    let fts_query = build_fts_query(query);

    let mut results = Vec::new();

    if let Some(fts_q) = fts_query {
        let mut stmt = conn
            .prepare_cached(
                "SELECT DISTINCT a.id, a.display_name, a.exe_path, a.shortcut_path, a.arguments,
                        a.icon_path, a.icon_index, a.source, a.indexed_at, a.updated_at
                 FROM applications a
                 WHERE a.rowid IN (
                     SELECT rowid FROM applications_fts WHERE applications_fts MATCH ?1
                 )
                 OR a.display_name LIKE ?2 ESCAPE '\\'
                 OR a.exe_path LIKE ?2 ESCAPE '\\'
                 LIMIT ?3;",
            )
            .map_err(|e| AppError::Database(format!("Prepare FTS query error: {}", e)))?;

        let rows = stmt
            .query_map(params![fts_q, like_pattern, limit as i64], |row| {
                Ok(ApplicationRecord {
                    id: row.get(0)?,
                    display_name: row.get(1)?,
                    exe_path: row.get(2)?,
                    shortcut_path: row.get(3)?,
                    arguments: row.get(4)?,
                    icon_path: row.get(5)?,
                    icon_index: row.get(6)?,
                    source: row.get(7)?,
                    indexed_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            })
            .map_err(|e| AppError::Database(format!("Query map error: {}", e)))?;

        for row in rows.flatten() {
            results.push(row);
        }
    } else {
        let mut stmt = conn
            .prepare_cached(
                "SELECT id, display_name, exe_path, shortcut_path, arguments,
                        icon_path, icon_index, source, indexed_at, updated_at
                 FROM applications
                 WHERE display_name LIKE ?1 ESCAPE '\\' OR exe_path LIKE ?1 ESCAPE '\\'
                 LIMIT ?2;",
            )
            .map_err(|e| AppError::Database(format!("Prepare LIKE query error: {}", e)))?;

        let rows = stmt
            .query_map(params![like_pattern, limit as i64], |row| {
                Ok(ApplicationRecord {
                    id: row.get(0)?,
                    display_name: row.get(1)?,
                    exe_path: row.get(2)?,
                    shortcut_path: row.get(3)?,
                    arguments: row.get(4)?,
                    icon_path: row.get(5)?,
                    icon_index: row.get(6)?,
                    source: row.get(7)?,
                    indexed_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            })
            .map_err(|e| AppError::Database(format!("Query LIKE error: {}", e)))?;

        for row in rows.flatten() {
            results.push(row);
        }
    }

    Ok(results)
}

pub fn get_application_by_id_or_path(
    conn: &Connection,
    id_or_path: &str,
) -> AppResult<Option<ApplicationRecord>> {
    let mut stmt = conn
        .prepare_cached(
            "SELECT id, display_name, exe_path, shortcut_path, arguments,
                    icon_path, icon_index, source, indexed_at, updated_at
             FROM applications
             WHERE id = ?1 OR exe_path = ?1 OR shortcut_path = ?1
             LIMIT 1;",
        )
        .map_err(|e| AppError::Database(format!("Prepare get_application error: {}", e)))?;

    let mut rows = stmt
        .query_map(params![id_or_path], |row| {
            Ok(ApplicationRecord {
                id: row.get(0)?,
                display_name: row.get(1)?,
                exe_path: row.get(2)?,
                shortcut_path: row.get(3)?,
                arguments: row.get(4)?,
                icon_path: row.get(5)?,
                icon_index: row.get(6)?,
                source: row.get(7)?,
                indexed_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })
        .map_err(|e| AppError::Database(format!("Query get_application error: {}", e)))?;

    if let Some(Ok(app)) = rows.next() {
        Ok(Some(app))
    } else {
        Ok(None)
    }
}


pub fn get_all_applications(conn: &Connection) -> AppResult<Vec<ApplicationRecord>> {
    let mut stmt = conn
        .prepare_cached(
            "SELECT id, display_name, exe_path, shortcut_path, arguments,
                    icon_path, icon_index, source, indexed_at, updated_at
             FROM applications
             ORDER BY display_name ASC;",
        )
        .map_err(|e| AppError::Database(format!("Prepare get_all error: {}", e)))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ApplicationRecord {
                id: row.get(0)?,
                display_name: row.get(1)?,
                exe_path: row.get(2)?,
                shortcut_path: row.get(3)?,
                arguments: row.get(4)?,
                icon_path: row.get(5)?,
                icon_index: row.get(6)?,
                source: row.get(7)?,
                indexed_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })
        .map_err(|e| AppError::Database(format!("Query get_all error: {}", e)))?;

    let mut results = Vec::new();
    for row in rows {
        if let Ok(app) = row {
            results.push(app);
        }
    }

    Ok(results)
}

pub fn count_applications(conn: &Connection) -> AppResult<usize> {
    let count: usize = conn
        .query_row("SELECT COUNT(*) FROM applications;", [], |r| r.get(0))
        .map_err(|e| AppError::Database(format!("Count applications error: {}", e)))?;
    Ok(count)
}
