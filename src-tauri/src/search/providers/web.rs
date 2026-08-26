use crate::search::query::{ResultType, SearchCandidate};

pub struct WebProvider;

impl WebProvider {
    pub fn search(query: &str) -> Option<SearchCandidate> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return None;
        }

        let url = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            trimmed.to_string()
        } else {
            format!("https://www.google.com/search?q={}", urlencoding(trimmed))
        };

        Some(SearchCandidate {
            id: "web_search".into(),
            result_type: ResultType::Web,
            display_name: format!("Search web for '{}'", trimmed),
            subtitle: url.clone(),
            target_path: url,
            icon_id: None,
            base_score: 0.1,
        })
    }
}

fn urlencoding(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}
