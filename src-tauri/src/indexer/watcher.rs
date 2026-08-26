use crate::core::state::AppState;
use crate::database::files::delete_file_by_path;
use crate::indexer::files::FileIndexer;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, warn};

pub struct FilesystemWatcher;

impl FilesystemWatcher {
    /// Starts watching the default user directories on a dedicated background thread.
    pub fn start_watching(state: Arc<AppState>) {
        std::thread::spawn(move || {
            let (tx, rx) = std::sync::mpsc::channel();

            let mut watcher = match RecommendedWatcher::new(tx, Config::default()) {
                Ok(w) => w,
                Err(e) => {
                    error!("Failed to initialize filesystem watcher: {}", e);
                    return;
                }
            };

            let targets = FileIndexer::get_default_target_directories();
            for (path, _) in targets {
                if path.exists() {
                    let _ = watcher.watch(&path, RecursiveMode::Recursive);
                }
            }

            info!("Filesystem watcher active for user directories.");

            let mut batch_queue: HashSet<PathBuf> = HashSet::new();
            let mut last_flush = std::time::Instant::now();

            loop {
                // Collect events with timeout
                if let Ok(Ok(event)) = rx.recv_timeout(Duration::from_millis(500)) {
                    Self::process_event(event, &mut batch_queue);
                }

                // Batch flush every 500ms or when reaching batch size
                if (!batch_queue.is_empty() && last_flush.elapsed() >= Duration::from_millis(500))
                    || batch_queue.len() >= 100
                {
                    let paths_to_process: Vec<PathBuf> = batch_queue.drain().collect();
                    last_flush = std::time::Instant::now();

                    let state_clone = state.clone();
                    tauri::async_runtime::spawn(async move {
                        let db = state_clone.db.lock().await;
                        for path in paths_to_process {
                            if !path.exists() {
                                let _ = delete_file_by_path(&db, &path.to_string_lossy());
                                debug!("Watcher processed deletion: {:?}", path);
                            }
                        }
                    });
                }
            }
        });
    }

    fn process_event(event: Event, queue: &mut HashSet<PathBuf>) {
        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                for path in event.paths {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        let lower = ext.to_lowercase();
                        if lower == "tmp" || lower == "log" || lower == "bak" {
                            continue;
                        }
                    }
                    queue.insert(path);
                }
            }
            _ => {}
        }
    }
}
