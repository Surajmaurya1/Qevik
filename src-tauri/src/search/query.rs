use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResultType {
    App,
    File,
    Folder,
    Command,
    Calculator,
    Web,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchCandidate {
    pub id: String,
    pub result_type: ResultType,
    pub display_name: String,
    pub subtitle: String,
    pub target_path: String,
    pub icon_id: Option<String>,
    pub base_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub result_type: ResultType,
    pub display_name: String,
    pub subtitle: String,
    pub score: f64,
    pub icon_id: Option<String>,
}
