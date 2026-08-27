use crate::error::{AppError, AppResult};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderRecord {
    pub id: String,
    pub name: String,
    pub path: String,
    pub parent_dir: String,
    pub indexed_at: i64,
}

pub fn upsert_folders(conn: &mut Connection, folders: &[FolderRecord]) -> AppResult<usize> {
    match execute_upsert_folders(conn, folders) {
        Ok(c) => Ok(c),
        Err(e) => {
            let err_str = e.to_string();
            if err_str.contains("malformed")
                || err_str.contains("fts")
                || err_str.contains("disk image")
            {
                tracing::warn!("Self-healing folders FTS index due to: {}", err_str);
                let _ = conn.execute_batch(
                    "DROP TABLE IF EXISTS folders_fts;
                     CREATE VIRTUAL TABLE IF NOT EXISTS folders_fts USING fts5(
                         name, path, content='folders', content_rowid='rowid'
                     );
                     INSERT INTO folders_fts(folders_fts) VALUES('rebuild');",
                );
                execute_upsert_folders(conn, folders)
            } else {
                Err(e)
            }
        }
    }
}

fn execute_upsert_folders(conn: &mut Connection, folders: &[FolderRecord]) -> AppResult<usize> {
    let tx = conn
        .transaction()
        .map_err(|e| AppError::Database(format!("Transaction start error: {}", e)))?;

    let mut count = 0;
    {
        let mut stmt = tx
            .prepare_cached(
                "INSERT INTO folders (id, name, path, parent_dir, indexed_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(path) DO UPDATE SET
                    name = excluded.name,
                    parent_dir = excluded.parent_dir,
                    indexed_at = excluded.indexed_at;",
            )
            .map_err(|e| AppError::Database(format!("Prepare insert folders error: {}", e)))?;

        for folder in folders {
            stmt.execute(params![
                folder.id,
                folder.name,
                folder.path,
                folder.parent_dir,
                folder.indexed_at,
            ])
            .map_err(|e| AppError::Database(format!("Execute folder insert error: {}", e)))?;
            count += 1;
        }
    }

    tx.commit()
        .map_err(|e| AppError::Database(format!("Transaction commit error: {}", e)))?;

    Ok(count)
}

pub fn search_folders_fts(
    conn: &Connection,
    query: &str,
    limit: usize,
) -> AppResult<Vec<FolderRecord>> {
    let like_pattern = format!(
        "%{}%",
        query
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    );
    let fts_query = crate::database::apps::build_fts_query(query);

    let mut results = Vec::new();

    if let Some(fts_q) = fts_query {
        let mut stmt = conn
            .prepare_cached(
                "SELECT DISTINCT f.id, f.name, f.path, f.parent_dir, f.indexed_at
                 FROM folders f
                 WHERE f.rowid IN (
                     SELECT rowid FROM folders_fts WHERE folders_fts MATCH ?1
                 )
                 OR f.name LIKE ?2 ESCAPE '\\'
                 OR f.path LIKE ?2 ESCAPE '\\'
                 LIMIT ?3;",
            )
            .map_err(|e| AppError::Database(format!("Prepare FTS folders error: {}", e)))?;

        let rows = stmt
            .query_map(params![fts_q, like_pattern, limit as i64], |row| {
                Ok(FolderRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    parent_dir: row.get(3)?,
                    indexed_at: row.get(4)?,
                })
            })
            .map_err(|e| AppError::Database(format!("Query folders error: {}", e)))?;

        for row in rows.flatten() {
            results.push(row);
        }
    } else {
        let mut stmt = conn
            .prepare_cached(
                "SELECT f.id, f.name, f.path, f.parent_dir, f.indexed_at
                 FROM folders f
                 WHERE f.name LIKE ?1 ESCAPE '\\' OR f.path LIKE ?1 ESCAPE '\\'
                 LIMIT ?2;",
            )
            .map_err(|e| AppError::Database(format!("Prepare LIKE folders error: {}", e)))?;

        let rows = stmt
            .query_map(params![like_pattern, limit as i64], |row| {
                Ok(FolderRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    parent_dir: row.get(3)?,
                    indexed_at: row.get(4)?,
                })
            })
            .map_err(|e| AppError::Database(format!("Query LIKE folders error: {}", e)))?;

        for row in rows.flatten() {
            results.push(row);
        }
    }

    Ok(results)
}

pub fn get_folder_by_id_or_path(
    conn: &Connection,
    id_or_path: &str,
) -> AppResult<Option<FolderRecord>> {
    let mut stmt = conn
        .prepare_cached(
            "SELECT id, name, path, parent_dir, indexed_at
             FROM folders
             WHERE id = ?1 OR path = ?1
             LIMIT 1;",
        )
        .map_err(|e| AppError::Database(format!("Prepare get_folder error: {}", e)))?;

    let mut rows = stmt
        .query_map(params![id_or_path], |row| {
            Ok(FolderRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                parent_dir: row.get(3)?,
                indexed_at: row.get(4)?,
            })
        })
        .map_err(|e| AppError::Database(format!("Query get_folder error: {}", e)))?;

    if let Some(Ok(folder)) = rows.next() {
        Ok(Some(folder))
    } else {
        Ok(None)
    }
}

#[allow(dead_code)]
pub fn count_folders(conn: &Connection) -> AppResult<usize> {
    let count: usize = conn
        .query_row("SELECT COUNT(*) FROM folders;", [], |r| r.get(0))
        .map_err(|e| AppError::Database(format!("Count folders error: {}", e)))?;
    Ok(count)
}
