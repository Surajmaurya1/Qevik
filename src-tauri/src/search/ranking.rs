use crate::database::usage::UsageRecord;
use crate::search::query::{ResultType, SearchCandidate, SearchResult};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Ranker;

impl Ranker {
    /// Rank all candidates according to the Section 9 scoring model.
    pub fn rank_all(
        candidates: Vec<SearchCandidate>,
        query: &str,
        usage_map: &HashMap<String, UsageRecord>,
        limit: usize,
    ) -> Vec<SearchResult> {
        let q_lower = query.to_lowercase();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let mut scored: Vec<SearchResult> = candidates
            .into_iter()
            .map(|c| {
                let (mut text_relevance, mut match_bonus) =
                    Self::compute_text_score(&c.display_name, &q_lower);
                if !c.subtitle.is_empty() {
                    let (sub_rel, sub_bonus) = Self::compute_text_score(&c.subtitle, &q_lower);
                    if (sub_rel * 0.6) > text_relevance {
                        text_relevance = sub_rel * 0.6;
                        match_bonus = sub_bonus * 0.4;
                    }
                }

                let type_bonus = Self::compute_type_priority_bonus(&c.result_type);

                let (usage_score, recency_score) = if let Some(usage) = usage_map.get(&c.id) {
                    let u_score = ((usage.launch_count as f64) / 50.0).min(1.0) * 0.5;
                    let hours = ((now - usage.last_launched_at).max(0) as f64) / 3600.0;
                    let r_score = (0.3 - (hours / 168.0) * 0.3).max(0.0);
                    (u_score, r_score)
                } else {
                    (0.0, 0.0)
                };

                let final_score =
                    text_relevance + match_bonus + type_bonus + usage_score + recency_score;

                SearchResult {
                    id: c.id,
                    result_type: c.result_type,
                    display_name: c.display_name,
                    subtitle: c.subtitle,
                    score: final_score,
                    icon_id: c.icon_id,
                }
            })
            .collect();

        // Sort descending by score; break ties alphabetically & by type priority
        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    a.display_name
                        .to_lowercase()
                        .cmp(&b.display_name.to_lowercase())
                })
        });

        scored.truncate(limit);
        scored
    }

    fn compute_text_score(name: &str, query: &str) -> (f64, f64) {
        let n_lower = name.to_lowercase();

        // Exact match
        if n_lower == query {
            return (1.0, 0.3);
        }

        // Prefix match
        if n_lower.starts_with(query) {
            return (0.85, 0.2);
        }

        // Token prefix match
        for token in n_lower.split(|c: char| !c.is_alphanumeric()) {
            if !token.is_empty() && token.starts_with(query) {
                return (0.70, 0.1);
            }
        }

        // Fuzzy match via character contains
        if n_lower.contains(query) {
            return (0.50, 0.0);
        }

        (0.2, 0.0)
    }

    fn compute_type_priority_bonus(result_type: &ResultType) -> f64 {
        match result_type {
            ResultType::Calculator => 0.25,
            ResultType::App => 0.20,
            ResultType::Command => 0.15,
            ResultType::Folder => 0.10,
            ResultType::File => 0.05,
            ResultType::Web => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match_score() {
        let (relevance, bonus) = Ranker::compute_text_score("Notepad", "notepad");
        assert_eq!(relevance, 1.0);
        assert_eq!(bonus, 0.3);
    }

    #[test]
    fn test_prefix_match_score() {
        let (relevance, bonus) = Ranker::compute_text_score("Visual Studio Code", "visual");
        assert_eq!(relevance, 0.85);
        assert_eq!(bonus, 0.2);
    }

    #[test]
    fn test_type_priority_bonus() {
        assert_eq!(
            Ranker::compute_type_priority_bonus(&ResultType::Calculator),
            0.25
        );
        assert_eq!(Ranker::compute_type_priority_bonus(&ResultType::App), 0.20);
        assert_eq!(
            Ranker::compute_type_priority_bonus(&ResultType::Command),
            0.15
        );
        assert_eq!(
            Ranker::compute_type_priority_bonus(&ResultType::Folder),
            0.10
        );
        assert_eq!(Ranker::compute_type_priority_bonus(&ResultType::File), 0.05);
    }

    #[test]
    fn test_rank_ordering() {
        let candidates = vec![
            SearchCandidate {
                id: "file1".into(),
                result_type: ResultType::File,
                display_name: "Code Notes.txt".into(),
                subtitle: "".into(),
                target_path: "".into(),
                icon_id: None,
                base_score: 0.5,
            },
            SearchCandidate {
                id: "app1".into(),
                result_type: ResultType::App,
                display_name: "Code".into(),
                subtitle: "".into(),
                target_path: "".into(),
                icon_id: None,
                base_score: 0.85,
            },
        ];

        let ranked = Ranker::rank_all(candidates, "code", &HashMap::new(), 10);
        assert_eq!(ranked.len(), 2);
        assert_eq!(ranked[0].id, "app1"); // App should rank above file for exact match
    }
}
