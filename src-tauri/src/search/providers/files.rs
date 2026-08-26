use crate::database::files::search_files_fts;
use crate::error::AppResult;
use crate::search::query::{ResultType, SearchCandidate};
use rusqlite::Connection;

pub struct FilesProvider;

impl FilesProvider {
    pub fn search(conn: &Connection, query: &str, limit: usize) -> AppResult<Vec<SearchCandidate>> {
        let files = search_files_fts(conn, query, limit)?;
        let candidates = files
            .into_iter()
            .map(|f| SearchCandidate {
                id: f.id,
                result_type: ResultType::File,
                display_name: f.name,
                subtitle: f.path.clone(),
                target_path: f.path,
                icon_id: None,
                base_score: 0.50,
            })
            .collect();
        Ok(candidates)
    }
}
