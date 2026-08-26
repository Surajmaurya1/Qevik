use serde::{Serialize, Serializer};
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Search error: {0}")]
    Search(String),

    #[error("Indexer error: {0}")]
    Indexer(String),

    #[error("Hotkey error: {0}")]
    Hotkey(String),

    #[error("Launcher error: {0}")]
    Launcher(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("System error: {0}")]
    System(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
