use crate::search::query::{ResultType, SearchCandidate};

pub struct CalculatorProvider;

impl CalculatorProvider {
    pub fn evaluate(expr: &str) -> Option<SearchCandidate> {
        let cleaned = expr.trim();
        if cleaned.is_empty() {
            return None;
        }

        match evalexpr::eval(cleaned) {
            Ok(value) => {
                let val_str = value.to_string();
                Some(SearchCandidate {
                    id: format!("calc:{}", val_str),
                    result_type: ResultType::Calculator,
                    display_name: val_str.clone(),
                    subtitle: format!("= {} (Press Enter to copy)", cleaned),
                    target_path: val_str,
                    icon_id: None,
                    base_score: 2.0,
                })
            }
            Err(_) => None,
        }
    }
}
