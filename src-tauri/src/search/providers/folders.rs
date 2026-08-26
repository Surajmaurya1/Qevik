use crate::database::folders::search_folders_fts;
use crate::error::AppResult;
use crate::search::query::{ResultType, SearchCandidate};
use rusqlite::Connection;

pub struct FoldersProvider;

impl FoldersProvider {
    pub fn search(conn: &Connection, query: &str, limit: usize) -> AppResult<Vec<SearchCandidate>> {
        let folders = search_folders_fts(conn, query, limit)?;
        let candidates = folders
            .into_iter()
            .map(|f| SearchCandidate {
                id: f.id,
                result_type: ResultType::Folder,
                display_name: f.name,
                subtitle: f.path.clone(),
                target_path: f.path,
                icon_id: None,
                base_score: 0.60,
            })
            .collect();
        Ok(candidates)
    }
}
