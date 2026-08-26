use crate::core::state::AppState;
use crate::database::apps::upsert_applications;
use crate::database::files::upsert_files;
use crate::database::folders::upsert_folders;
use crate::error::AppResult;
use crate::indexer::apps::AppIndexer;
use crate::indexer::files::FileIndexer;
use crate::indexer::watcher::FilesystemWatcher;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, info};

pub struct IndexManager;

impl IndexManager {
    /// Start full initial indexing in background thread without blocking UI.
    pub fn start_background_indexing(state: Arc<AppState>) {
        let state_for_watcher = state.clone();

        tauri::async_runtime::spawn(async move {
            info!("Starting background indexing pass for apps, files, and folders...");

            // Set indexing status flag
            {
                let mut status = state.index_status.write().await;
                status.is_indexing = true;
            }

            match tauri::async_runtime::spawn_blocking(
                move || -> AppResult<(usize, usize, usize, i64)> {
                    // 1. Scan applications
                    let apps = AppIndexer::scan_all_sources()?;
                    let app_count = apps.len();

                    // 2. Scan default user files and folders
                    let file_result = FileIndexer::scan_default_directories()?;
                    let file_count = file_result.files.len();
                    let folder_count = file_result.folders.len();

                    // 3. Commit to SQLite
                    let mut conn = crate::database::connection::open_connection()?;
                    crate::database::migrations::run_migrations(&mut conn)?;

                    upsert_applications(&mut conn, &apps)?;
                    upsert_files(&mut conn, &file_result.files)?;
                    upsert_folders(&mut conn, &file_result.folders)?;

                    let _ = conn.execute_batch(
                        "INSERT INTO applications_fts(applications_fts) VALUES('rebuild');
                     INSERT INTO files_fts(files_fts) VALUES('rebuild');
                     INSERT INTO folders_fts(folders_fts) VALUES('rebuild');",
                    );

                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs() as i64;

                    Ok((app_count, file_count, folder_count, now))
                },
            )
            .await
            {
                Ok(Ok((app_count, file_count, folder_count, timestamp))) => {
                    info!(
                        "Background indexing complete: {} apps, {} files, {} folders.",
                        app_count, file_count, folder_count
                    );
                    let mut status = state.index_status.write().await;
                    status.is_indexing = false;
                    status.total_applications = app_count;
                    status.total_files = file_count;
                    status.total_folders = folder_count;
                    status.last_indexed_at = Some(timestamp);

                    // Start incremental filesystem watcher
                    FilesystemWatcher::start_watching(state_for_watcher);
                }
                Ok(Err(e)) => {
                    error!("Indexing pass failed: {}", e);
                    let mut status = state.index_status.write().await;
                    status.is_indexing = false;
                }
                Err(e) => {
                    error!("Join error in indexing task: {}", e);
                    let mut status = state.index_status.write().await;
                    status.is_indexing = false;
                }
            }
        });
    }
}
