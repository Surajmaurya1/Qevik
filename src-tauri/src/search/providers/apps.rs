use crate::database::apps::{search_applications_fts, ApplicationRecord};
use crate::error::AppResult;
use crate::search::query::{ResultType, SearchCandidate};
use rusqlite::Connection;

pub struct AppsProvider;

impl AppsProvider {
    /// In-memory cache search for sub-millisecond response
    pub fn search_cache(
        apps: &[ApplicationRecord],
        query: &str,
        limit: usize,
    ) -> Vec<SearchCandidate> {
        let q_lower = query.to_lowercase();
        let tokens: Vec<&str> = q_lower.split_whitespace().collect();

        let mut matches: Vec<(&ApplicationRecord, u32)> = Vec::new();

        for app in apps {
            let name_lower = app.display_name.to_lowercase();
            let mut score: u32 = 0;

            if name_lower == q_lower {
                score = 1000;
            } else if name_lower.starts_with(&q_lower) {
                score = 500;
            } else if tokens.iter().all(|t| name_lower.contains(t)) {
                score = 200;
            } else {
                let exe_lower = app.exe_path.to_lowercase();
                if tokens.iter().all(|t| exe_lower.contains(t)) {
                    score = 100;
                }
            }

            if score > 0 {
                matches.push((app, score));
            }
        }

        matches.sort_by_key(|a| std::cmp::Reverse(a.1));

        matches
            .into_iter()
            .take(limit)
            .map(|(a, _)| SearchCandidate {
                id: a.id.clone(),
                result_type: ResultType::App,
                display_name: a.display_name.clone(),
                subtitle: a.exe_path.clone(),
                target_path: a
                    .shortcut_path
                    .clone()
                    .unwrap_or_else(|| a.exe_path.clone()),
                icon_id: a.icon_path.clone(),
                base_score: 0.90,
            })
            .collect()
    }

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
