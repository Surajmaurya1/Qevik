use crate::core::state::AppState;
use crate::database::apps::upsert_applications;
use crate::database::files::upsert_files;
use crate::database::folders::upsert_folders;
use crate::error::AppResult;
use crate::indexer::apps::AppIndexer;
use crate::indexer::files::FileIndexer;
use crate::indexer::watcher::FilesystemWatcher;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{error, info};

pub struct IndexManager;

impl IndexManager {
    /// Start full initial indexing in background thread without blocking UI.
    pub fn start_background_indexing(state: Arc<AppState>) {
        let state_clone = state.clone();

        tauri::async_runtime::spawn(async move {
            info!("Starting staggered background indexing pass...");

            // Set indexing status flag
            {
                let mut status = state_clone.index_status.write().await;
                status.is_indexing = true;
            }

            // Phase 1: Immediate Application Indexing & In-Memory Cache Population
            let apps_result = tauri::async_runtime::spawn_blocking(
                move || -> AppResult<Vec<crate::database::apps::ApplicationRecord>> {
                    let apps = AppIndexer::scan_all_sources()?;
                    let mut conn = crate::database::connection::open_connection()?;
                    crate::database::migrations::run_migrations(&mut conn)?;
                    upsert_applications(&mut conn, &apps)?;
                    let _ = conn.execute_batch(
                        "INSERT INTO applications_fts(applications_fts) VALUES('rebuild');",
                    );
                    Ok(apps)
                },
            )
            .await;

            if let Ok(Ok(apps)) = apps_result {
                let app_count = apps.len();
                {
                    let mut cache = state_clone.app_cache.write().await;
                    *cache = apps;
                }
                {
                    let mut status = state_clone.index_status.write().await;
                    status.total_applications = app_count;
                }
                info!(
                    "Phase 1: In-memory app cache populated with {} applications.",
                    app_count
                );
            }

            // Phase 2: Staggered File & Folder Indexing (delayed by 2 seconds so startup is 100% instant)
            tokio::time::sleep(Duration::from_millis(2000)).await;

            let custom_dirs = {
                let s = state_clone.settings.read().await;
                s.indexed_directories.clone()
            };

            let state_for_watcher = state_clone.clone();
            let files_result =
                tauri::async_runtime::spawn_blocking(move || -> AppResult<(usize, usize, i64)> {
                    let file_result = FileIndexer::scan_all_directories(&custom_dirs)?;
                    let file_count = file_result.files.len();
                    let folder_count = file_result.folders.len();

                    let mut conn = match crate::database::connection::open_connection() {
                        Ok(c) => c,
                        Err(e) => {
                            tracing::warn!("Initial connection in file indexer failed: {}. Resetting...", e);
                            crate::database::connection::remove_database_files();
                            crate::database::connection::open_connection()?
                        }
                    };

                    if let Err(e) = crate::database::migrations::run_migrations(&mut conn) {
                        tracing::warn!("Migrations failed in file indexer: {}. Resetting DB...", e);
                        drop(conn);
                        crate::database::connection::remove_database_files();
                        let mut fresh_conn = crate::database::connection::open_connection()?;
                        crate::database::migrations::run_migrations(&mut fresh_conn)?;
                        conn = fresh_conn;
                    }

                    if let Err(e) = upsert_files(&mut conn, &file_result.files) {
                        tracing::warn!("upsert_files failed: {}. Retrying with fresh DB...", e);
                        drop(conn);
                        crate::database::connection::remove_database_files();
                        let mut fresh_conn = crate::database::connection::open_connection()?;
                        crate::database::migrations::run_migrations(&mut fresh_conn)?;
                        upsert_files(&mut fresh_conn, &file_result.files)?;
                        upsert_folders(&mut fresh_conn, &file_result.folders)?;
                        conn = fresh_conn;
                    } else {
                        let _ = upsert_folders(&mut conn, &file_result.folders);
                    }

                    let _ = conn.execute_batch(
                        "INSERT INTO files_fts(files_fts) VALUES('rebuild');
                         INSERT INTO folders_fts(folders_fts) VALUES('rebuild');",
                    );

                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs() as i64;

                    Ok((file_count, folder_count, now))
                })
                .await;

            match files_result {
                Ok(Ok((file_count, folder_count, timestamp))) => {
                    info!(
                        "Phase 2 complete: {} files, {} folders indexed.",
                        file_count, folder_count
                    );
                    let mut status = state_clone.index_status.write().await;
                    status.is_indexing = false;
                    status.total_files = file_count;
                    status.total_folders = folder_count;
                    status.last_indexed_at = Some(timestamp);

                    // Start incremental filesystem watcher
                    FilesystemWatcher::start_watching(state_for_watcher);
                }
                Ok(Err(e)) => {
                    error!("File indexing pass failed: {}", e);
                    let mut status = state_clone.index_status.write().await;
                    status.is_indexing = false;
                }
                Err(e) => {
                    error!("Join error in file indexing task: {}", e);
                    let mut status = state_clone.index_status.write().await;
                    status.is_indexing = false;
                }
            }
        });
    }
}
