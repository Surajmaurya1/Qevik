use crate::database::apps::search_applications_fts;
use crate::error::AppResult;
use crate::search::query::{ResultType, SearchCandidate};
use rusqlite::Connection;

pub struct AppsProvider;

impl AppsProvider {
    pub fn search(conn: &Connection, query: &str, limit: usize) -> AppResult<Vec<SearchCandidate>> {
        let apps = search_applications_fts(conn, query, limit)?;
        let candidates = apps
            .into_iter()
            .map(|a| SearchCandidate {
                id: a.id,
                result_type: ResultType::App,
                display_name: a.display_name,
                subtitle: a.exe_path.clone(),
                target_path: a.shortcut_path.unwrap_or(a.exe_path),
                icon_id: a.icon_path,
                base_score: 0.85,
            })
            .collect();
        Ok(candidates)
    }
}
