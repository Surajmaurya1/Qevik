use crate::core::state::AppState;
use crate::database::usage::UsageRecord;
use crate::error::AppResult;
use crate::search::parser::{QueryMode, QueryParser};
use crate::search::providers::apps::AppsProvider;
use crate::search::providers::calculator::CalculatorProvider;
use crate::search::providers::commands::CommandsProvider;
use crate::search::providers::files::FilesProvider;
use crate::search::providers::folders::FoldersProvider;
use crate::search::providers::web::WebProvider;
use crate::search::query::{ResultType, SearchResult};
use crate::search::ranking::Ranker;
use std::collections::HashMap;
use std::sync::Arc;

pub struct SearchEngine;

impl SearchEngine {
    /// Coordinates query parsing, provider fan-out, and ranking.
    pub async fn execute(
        state: &Arc<AppState>,
        query: &str,
        max_results: usize,
    ) -> AppResult<Vec<SearchResult>> {
        let mode = QueryParser::parse(query);

        match mode {
            QueryMode::Empty => {
                let db = state.db.lock().await;
                let recent = crate::database::history::get_recent_history(&db, max_results)?;
                let results = recent
                    .into_iter()
                    .map(|h| SearchResult {
                        id: h.result_id,
                        result_type: ResultType::App,
                        display_name: h.result_name,
                        subtitle: "Recently opened".into(),
                        score: 1.0,
                        icon_id: None,
                    })
                    .collect();
                Ok(results)
            }

            QueryMode::Calculator(expr) => {
                if let Some(candidate) = CalculatorProvider::evaluate(&expr) {
                    let ranked = Ranker::rank_all(
                        vec![candidate],
                        &expr,
                        &HashMap::new(),
                        max_results,
                    );
                    Ok(ranked)
                } else {
                    Ok(vec![])
                }
            }


            QueryMode::Command(cmd) => {
                let candidates = CommandsProvider::search(&cmd);
                let ranked = Ranker::rank_all(candidates, &cmd, &HashMap::new(), max_results);
                Ok(ranked)
            }

            QueryMode::Web(url) => {
                if let Some(candidate) = WebProvider::search(&url) {
                    Ok(vec![SearchResult {
                        id: candidate.id,
                        result_type: ResultType::Web,
                        display_name: candidate.display_name,
                        subtitle: candidate.subtitle,
                        score: 1.0,
                        icon_id: None,
                    }])
                } else {
                    Ok(vec![])
                }
            }

            QueryMode::General(q) => {
                let mut candidates = Vec::new();

                // 1. Math check
                if let Some(calc) = CalculatorProvider::evaluate(&q) {
                    candidates.push(calc);
                }

                // 2. Apps, files, folders from SQLite
                {
                    let db = state.db.lock().await;

                    if let Ok(mut apps) = AppsProvider::search(&db, &q, 25) {
                        candidates.append(&mut apps);
                    }

                    if q.len() >= 2 {
                        if let Ok(mut files) = FilesProvider::search(&db, &q, 25) {
                            candidates.append(&mut files);
                        }

                        if let Ok(mut folders) = FoldersProvider::search(&db, &q, 15) {
                            candidates.append(&mut folders);
                        }
                    }
                }

                // 3. Built-in commands
                let mut commands = CommandsProvider::search(&q);
                candidates.append(&mut commands);

                // 4. Web search fallback candidate
                let settings = state.settings.read().await;
                if settings.enable_web_search {
                    if let Some(web) = WebProvider::search(&q) {
                        candidates.push(web);
                    }
                }

                // 5. Ranking
                let usage_map = HashMap::<String, UsageRecord>::new();
                let ranked = Ranker::rank_all(candidates, &q, &usage_map, max_results);

                Ok(ranked)
            }
        }
    }
}
