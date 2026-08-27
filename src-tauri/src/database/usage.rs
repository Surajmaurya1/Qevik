use crate::error::{AppError, AppResult};
use rusqlite::{params, Connection};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct UsageRecord {
    #[allow(dead_code)]
    pub result_id: String,
    #[allow(dead_code)]
    pub result_type: String,
    pub launch_count: i64,
    pub last_launched_at: i64,
}

pub fn increment_usage(conn: &Connection, result_id: &str, result_type: &str) -> AppResult<()> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    conn.execute(
        "INSERT INTO usage (result_id, result_type, launch_count, last_launched_at)
         VALUES (?1, ?2, 1, ?3)
         ON CONFLICT(result_id, result_type) DO UPDATE SET
            launch_count = launch_count + 1,
            last_launched_at = ?3;",
        params![result_id, result_type, now],
    )
    .map_err(|e| AppError::Database(format!("Increment usage error: {}", e)))?;

    Ok(())
}

#[allow(dead_code)]
pub fn get_usage(
    conn: &Connection,
    result_id: &str,
    result_type: &str,
) -> AppResult<Option<UsageRecord>> {
    let mut stmt = conn
        .prepare_cached(
            "SELECT result_id, result_type, launch_count, last_launched_at
             FROM usage
             WHERE result_id = ?1 AND result_type = ?2;",
        )
        .map_err(|e| AppError::Database(format!("Prepare get_usage error: {}", e)))?;

    let mut rows = stmt
        .query(params![result_id, result_type])
        .map_err(|e| AppError::Database(format!("Query get_usage error: {}", e)))?;

    if let Some(row) = rows
        .next()
        .map_err(|e| AppError::Database(format!("Row error: {}", e)))?
    {
        Ok(Some(UsageRecord {
            result_id: row.get(0).map_err(|e| AppError::Database(e.to_string()))?,
            result_type: row.get(1).map_err(|e| AppError::Database(e.to_string()))?,
            launch_count: row.get(2).map_err(|e| AppError::Database(e.to_string()))?,
            last_launched_at: row.get(3).map_err(|e| AppError::Database(e.to_string()))?,
        }))
    } else {
        Ok(None)
    }
}

pub fn get_all_usage(conn: &Connection) -> AppResult<Vec<UsageRecord>> {
    let mut stmt = conn
        .prepare_cached(
            "SELECT result_id, result_type, launch_count, last_launched_at
             FROM usage;",
        )
        .map_err(|e| AppError::Database(format!("Prepare get_all_usage error: {}", e)))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(UsageRecord {
                result_id: row.get(0)?,
                result_type: row.get(1)?,
                launch_count: row.get(2)?,
                last_launched_at: row.get(3)?,
            })
        })
        .map_err(|e| AppError::Database(format!("Query get_all_usage error: {}", e)))?;

    let mut records = Vec::new();
    for r in rows {
        if let Ok(rec) = r {
            records.push(rec);
        }
    }
    Ok(records)
}

