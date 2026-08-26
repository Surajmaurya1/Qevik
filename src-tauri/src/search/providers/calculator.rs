use crate::search::query::{ResultType, SearchCandidate};

pub struct CalculatorProvider;

impl CalculatorProvider {
    pub fn evaluate(expr: &str) -> Option<SearchCandidate> {
        let cleaned = expr.trim();
        if cleaned.is_empty() {
            return None;
        }

        match evalexpr::eval(cleaned) {
            Ok(value) => Some(SearchCandidate {
                id: "calc_eval".into(),
                result_type: ResultType::Calculator,
                display_name: value.to_string(),
                subtitle: format!("= {}", cleaned),
                target_path: value.to_string(),
                icon_id: None,
                base_score: 2.0,
            }),
            Err(_) => None,
        }
    }
}
