use crate::error::{AppError, AppResult};
use rusqlite::{params, Connection};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct HistoryRecord {
    #[allow(dead_code)]
    pub id: i64,
    #[allow(dead_code)]
    pub query: String,
    pub result_id: String,
    pub result_type: String,
    pub result_name: String,
    #[allow(dead_code)]
    pub launched_at: i64,
}

pub fn record_launch_history(
    conn: &Connection,
    query: &str,
    result_id: &str,
    result_type: &str,
    result_name: &str,
) -> AppResult<()> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    conn.execute(
        "INSERT INTO history (query, result_id, result_type, result_name, launched_at)
         VALUES (?1, ?2, ?3, ?4, ?5);",
        params![query, result_id, result_type, result_name, now],
    )
    .map_err(|e| AppError::Database(format!("Record history error: {}", e)))?;

    // Cap history table at 10,000 rows per Section 13
    let _ = conn.execute(
        "DELETE FROM history WHERE id IN (
            SELECT id FROM history ORDER BY launched_at DESC LIMIT -1 OFFSET 10000
        );",
        [],
    );

    Ok(())
}

pub fn get_recent_history(conn: &Connection, limit: usize) -> AppResult<Vec<HistoryRecord>> {
    let mut stmt = conn
        .prepare_cached(
            "SELECT result_id, result_type, MAX(result_name) as result_name, MAX(launched_at) as launched_at
             FROM history
             WHERE result_name NOT LIKE 'file_%' AND result_name NOT LIKE 'app_%' AND result_name NOT LIKE 'folder_%'
             GROUP BY result_id, result_type
             ORDER BY launched_at DESC
             LIMIT ?1;",
        )
        .map_err(|e| AppError::Database(format!("Prepare get_recent error: {}", e)))?;

    let rows = stmt
        .query_map(params![limit as i64], |row| {
            Ok(HistoryRecord {
                id: 0,
                query: String::new(),
                result_id: row.get(0)?,
                result_type: row.get(1)?,
                result_name: row.get(2)?,
                launched_at: row.get(3)?,
            })
        })
        .map_err(|e| AppError::Database(format!("Query get_recent error: {}", e)))?;

    let mut results = Vec::new();
    for rec in rows.flatten() {
        results.push(rec);
    }

    Ok(results)
}
