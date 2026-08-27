use crate::error::AppResult;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub hotkey: String,
    pub theme: String,
    pub start_with_windows: bool,
    pub max_results: usize,
    pub enable_calculator: bool,
    pub enable_web_search: bool,
    pub indexed_directories: Vec<String>,
    pub ignored_extensions: Vec<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            hotkey: "Alt+Space".to_string(),
            theme: "dark".to_string(),
            start_with_windows: false,
            max_results: 12,
            enable_calculator: true,
            enable_web_search: false,
            indexed_directories: vec![],
            ignored_extensions: vec!["tmp".into(), "log".into(), "bak".into()],
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IndexStatus {
    pub is_indexing: bool,
    pub total_applications: usize,
    pub total_files: usize,
    pub total_folders: usize,
    pub last_indexed_at: Option<i64>,
}

pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub settings: Arc<RwLock<AppSettings>>,
    pub index_status: Arc<RwLock<IndexStatus>>,
    pub app_cache: Arc<RwLock<Vec<crate::database::apps::ApplicationRecord>>>,
    #[allow(dead_code)]
    pub search_cancel: Arc<AtomicBool>,
}

impl AppState {
    pub fn new() -> AppResult<Self> {
        let mut conn = crate::database::connection::open_connection()?;
        
        if let Err(e) = crate::database::migrations::run_migrations(&mut conn) {
            tracing::warn!("Migration failed on initial connection: {}. Recreating database...", e);
            drop(conn);
            crate::database::connection::remove_database_files();
            let mut fresh_conn = crate::database::connection::open_connection()?;
            crate::database::migrations::run_migrations(&mut fresh_conn)?;
            conn = fresh_conn;
        }

        // Load settings from database if previously saved
        let initial_settings =
            crate::database::settings::load_settings_from_db(&conn)?.unwrap_or_default();

        // Initial in-memory app cache load
        let initial_apps = crate::database::apps::get_all_applications(&conn).unwrap_or_default();

        Ok(Self {
            db: Arc::new(Mutex::new(conn)),
            settings: Arc::new(RwLock::new(initial_settings)),
            index_status: Arc::new(RwLock::new(IndexStatus::default())),
            app_cache: Arc::new(RwLock::new(initial_apps)),
            search_cancel: Arc::new(AtomicBool::new(false)),
        })
    }
}
