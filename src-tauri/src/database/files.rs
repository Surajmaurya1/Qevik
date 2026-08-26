use crate::error::{AppError, AppResult};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRecord {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub extension: Option<String>,
    pub path: String,
    pub parent_dir: String,
    pub size_bytes: u64,
    pub modified_at: i64,
    pub indexed_at: i64,
    pub is_hidden: bool,
    pub is_system: bool,
}

pub fn upsert_files(conn: &mut Connection, files: &[FileRecord]) -> AppResult<usize> {
    let tx = conn
        .transaction()
        .map_err(|e| AppError::Database(format!("Transaction start error: {}", e)))?;

    let mut count = 0;
    {
        let mut stmt = tx
            .prepare_cached(
                "INSERT INTO files (
                    id, name, display_name, extension, path, parent_dir,
                    size_bytes, modified_at, indexed_at, is_hidden, is_system
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                ON CONFLICT(path) DO UPDATE SET
                    name = excluded.name,
                    display_name = excluded.display_name,
                    extension = excluded.extension,
                    parent_dir = excluded.parent_dir,
                    size_bytes = excluded.size_bytes,
                    modified_at = excluded.modified_at,
                    indexed_at = excluded.indexed_at,
                    is_hidden = excluded.is_hidden,
                    is_system = excluded.is_system;",
            )
            .map_err(|e| AppError::Database(format!("Prepare insert files error: {}", e)))?;

        for file in files {
            stmt.execute(params![
                file.id,
                file.name,
                file.display_name,
                file.extension,
                file.path,
                file.parent_dir,
                file.size_bytes as i64,
                file.modified_at,
                file.indexed_at,
                if file.is_hidden { 1 } else { 0 },
                if file.is_system { 1 } else { 0 },
            ])
            .map_err(|e| AppError::Database(format!("Execute file insert error: {}", e)))?;
            count += 1;
        }
    }

    tx.commit()
        .map_err(|e| AppError::Database(format!("Transaction commit error: {}", e)))?;

    Ok(count)
}

pub fn delete_file_by_path(conn: &Connection, path: &str) -> AppResult<usize> {
    let rows = conn
        .execute("DELETE FROM files WHERE path = ?1;", params![path])
        .map_err(|e| AppError::Database(format!("Delete file error: {}", e)))?;
    Ok(rows)
}

pub fn search_files_fts(
    conn: &Connection,
    query: &str,
    limit: usize,
) -> AppResult<Vec<FileRecord>> {
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
                "SELECT DISTINCT f.id, f.name, f.display_name, f.extension, f.path,
                        f.parent_dir, f.size_bytes, f.modified_at, f.indexed_at,
                        f.is_hidden, f.is_system
                 FROM files f
                 WHERE f.rowid IN (
                     SELECT rowid FROM files_fts WHERE files_fts MATCH ?1
                 )
                 OR f.name LIKE ?2 ESCAPE '\\'
                 OR f.path LIKE ?2 ESCAPE '\\'
                 LIMIT ?3;",
            )
            .map_err(|e| AppError::Database(format!("Prepare FTS files error: {}", e)))?;

        let rows = stmt
            .query_map(params![fts_q, like_pattern, limit as i64], |row| {
                let is_hidden_int: i32 = row.get(9)?;
                let is_system_int: i32 = row.get(10)?;
                let size_int: i64 = row.get(6)?;

                Ok(FileRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    display_name: row.get(2)?,
                    extension: row.get(3)?,
                    path: row.get(4)?,
                    parent_dir: row.get(5)?,
                    size_bytes: size_int as u64,
                    modified_at: row.get(7)?,
                    indexed_at: row.get(8)?,
                    is_hidden: is_hidden_int != 0,
                    is_system: is_system_int != 0,
                })
            })
            .map_err(|e| AppError::Database(format!("Query files error: {}", e)))?;

        for row in rows.flatten() {
            results.push(row);
        }
    } else {
        let mut stmt = conn
            .prepare_cached(
                "SELECT f.id, f.name, f.display_name, f.extension, f.path,
                        f.parent_dir, f.size_bytes, f.modified_at, f.indexed_at,
                        f.is_hidden, f.is_system
                 FROM files f
                 WHERE f.name LIKE ?1 ESCAPE '\\' OR f.path LIKE ?1 ESCAPE '\\'
                 LIMIT ?2;",
            )
            .map_err(|e| AppError::Database(format!("Prepare LIKE files error: {}", e)))?;

        let rows = stmt
            .query_map(params![like_pattern, limit as i64], |row| {
                let is_hidden_int: i32 = row.get(9)?;
                let is_system_int: i32 = row.get(10)?;
                let size_int: i64 = row.get(6)?;

                Ok(FileRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    display_name: row.get(2)?,
                    extension: row.get(3)?,
                    path: row.get(4)?,
                    parent_dir: row.get(5)?,
                    size_bytes: size_int as u64,
                    modified_at: row.get(7)?,
                    indexed_at: row.get(8)?,
                    is_hidden: is_hidden_int != 0,
                    is_system: is_system_int != 0,
                })
            })
            .map_err(|e| AppError::Database(format!("Query LIKE files error: {}", e)))?;

        for row in rows.flatten() {
            results.push(row);
        }
    }

    Ok(results)
}

pub fn get_file_by_id_or_path(
    conn: &Connection,
    id_or_path: &str,
) -> AppResult<Option<FileRecord>> {
    let mut stmt = conn
        .prepare_cached(
            "SELECT id, name, display_name, extension, path,
                    parent_dir, size_bytes, modified_at, indexed_at,
                    is_hidden, is_system
             FROM files
             WHERE id = ?1 OR path = ?1
             LIMIT 1;",
        )
        .map_err(|e| AppError::Database(format!("Prepare get_file error: {}", e)))?;

    let mut rows = stmt
        .query_map(params![id_or_path], |row| {
            let is_hidden_int: i32 = row.get(9)?;
            let is_system_int: i32 = row.get(10)?;
            let size_int: i64 = row.get(6)?;

            Ok(FileRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                display_name: row.get(2)?,
                extension: row.get(3)?,
                path: row.get(4)?,
                parent_dir: row.get(5)?,
                size_bytes: size_int as u64,
                modified_at: row.get(7)?,
                indexed_at: row.get(8)?,
                is_hidden: is_hidden_int != 0,
                is_system: is_system_int != 0,
            })
        })
        .map_err(|e| AppError::Database(format!("Query get_file error: {}", e)))?;

    if let Some(Ok(file)) = rows.next() {
        Ok(Some(file))
    } else {
        Ok(None)
    }
}

#[allow(dead_code)]
pub fn count_files(conn: &Connection) -> AppResult<usize> {
    let count: usize = conn
        .query_row("SELECT COUNT(*) FROM files;", [], |r| r.get(0))
        .map_err(|e| AppError::Database(format!("Count files error: {}", e)))?;
    Ok(count)
}
