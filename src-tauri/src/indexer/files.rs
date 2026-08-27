use crate::database::files::FileRecord;
use crate::database::folders::FolderRecord;
use crate::error::AppResult;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

pub struct FileIndexer;

pub struct IndexScanResult {
    pub files: Vec<FileRecord>,
    pub folders: Vec<FolderRecord>,
}

impl FileIndexer {
    /// Scan all default user directories plus custom user directories up to depth limits.
    pub fn scan_all_directories(custom_dirs: &[String]) -> AppResult<IndexScanResult> {
        let mut targets = Self::get_default_target_directories();
        for dir in custom_dirs {
            let p = PathBuf::from(dir);
            if p.exists() && p.is_dir() {
                targets.push((p, 3));
            }
        }

        let mut all_files = Vec::new();
        let mut all_folders = Vec::new();

        for (dir_path, max_depth) in targets {
            if dir_path.exists() {
                debug!(
                    "Scanning directory {:?} with max depth {}",
                    dir_path, max_depth
                );
                Self::scan_recursive(&dir_path, 0, max_depth, &mut all_files, &mut all_folders);
            }
        }

        info!(
            "Discovered {} files and {} folders.",
            all_files.len(),
            all_folders.len()
        );

        Ok(IndexScanResult {
            files: all_files,
            folders: all_folders,
        })
    }

    /// Scan default directories only.
    #[allow(dead_code)]
    pub fn scan_default_directories() -> AppResult<IndexScanResult> {
        Self::scan_all_directories(&[])
    }

    pub fn inspect_file(path: &Path) -> Option<FileRecord> {
        if !path.is_file() {
            return None;
        }

        let file_name = path.file_name()?.to_string_lossy().to_string();
        if Self::is_ignored_name(&file_name) {
            return None;
        }

        let ext = path.extension().map(|e| e.to_string_lossy().to_string());
        if let Some(ref e) = ext {
            if Self::is_ignored_extension(e) {
                return None;
            }
        }

        let metadata = fs::metadata(path).ok()?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(now);

        let display_name = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| file_name.clone());

        let parent = path.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
        let file_id = format!("file_{:x}", simple_hash(&path.to_string_lossy()));

        Some(FileRecord {
            id: file_id,
            name: file_name,
            display_name,
            extension: ext,
            path: path.to_string_lossy().to_string(),
            parent_dir: parent,
            size_bytes: metadata.len(),
            modified_at: modified,
            indexed_at: now,
            is_hidden: false,
            is_system: false,
        })
    }

    pub fn inspect_folder(path: &Path) -> Option<FolderRecord> {
        if !path.is_dir() {
            return None;
        }

        let name = path.file_name()?.to_string_lossy().to_string();
        if Self::is_ignored_name(&name) {
            return None;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let parent = path.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
        let folder_id = format!("folder_{:x}", simple_hash(&path.to_string_lossy()));

        Some(FolderRecord {
            id: folder_id,
            name,
            path: path.to_string_lossy().to_string(),
            parent_dir: parent,
            indexed_at: now,
        })
    }

    pub fn get_default_target_directories() -> Vec<(PathBuf, usize)> {
        let mut list = Vec::new();
        if let Ok(user_profile) = env::var("USERPROFILE") {
            let base = PathBuf::from(user_profile);
            // Desktop: depth 1
            list.push((base.join("Desktop"), 1));
            // Documents: depth 4
            list.push((base.join("Documents"), 4));
            // Downloads: depth 1
            list.push((base.join("Downloads"), 1));
            // Pictures: depth 3
            list.push((base.join("Pictures"), 3));
            // Videos: depth 3
            list.push((base.join("Videos"), 3));
            // Music: depth 3
            list.push((base.join("Music"), 3));
        }
        list
    }

    fn scan_recursive(
        dir: &Path,
        current_depth: usize,
        max_depth: usize,
        files: &mut Vec<FileRecord>,
        folders: &mut Vec<FolderRecord>,
    ) {
        if current_depth > max_depth {
            return;
        }

        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(e) => {
                warn!("Cannot read directory {:?}: {}", dir, e);
                return;
            }
        };

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = entry.file_name().to_string_lossy().to_string();

            // Ignored directory names per Section 11
            if Self::is_ignored_name(&file_name) {
                continue;
            }

            if path.is_dir() {
                let folder_id = format!("folder_{:x}", simple_hash(&path.to_string_lossy()));
                let parent = dir.to_string_lossy().to_string();

                folders.push(FolderRecord {
                    id: folder_id,
                    name: file_name,
                    path: path.to_string_lossy().to_string(),
                    parent_dir: parent,
                    indexed_at: now,
                });

                Self::scan_recursive(&path, current_depth + 1, max_depth, files, folders);
            } else if path.is_file() {
                let ext = path.extension().map(|e| e.to_string_lossy().to_string());

                if let Some(ref e) = ext {
                    if Self::is_ignored_extension(e) {
                        continue;
                    }
                }

                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                let modified = entry
                    .metadata()
                    .and_then(|m| m.modified())
                    .ok()
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(now);

                let display_name = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| file_name.clone());

                let file_id = format!("file_{:x}", simple_hash(&path.to_string_lossy()));
                let parent = dir.to_string_lossy().to_string();

                files.push(FileRecord {
                    id: file_id,
                    name: file_name,
                    display_name,
                    extension: ext,
                    path: path.to_string_lossy().to_string(),
                    parent_dir: parent,
                    size_bytes: size,
                    modified_at: modified,
                    indexed_at: now,
                    is_hidden: false,
                    is_system: false,
                });
            }
        }
    }

    fn is_ignored_name(name: &str) -> bool {
        let lower = name.to_lowercase();
        lower.starts_with('.')
            || lower == "node_modules"
            || lower == "__pycache__"
            || lower == "$recycle.bin"
            || lower == "system volume information"
            || lower == "desktop.ini"
            || lower == "thumbs.db"
    }

    fn is_ignored_extension(ext: &str) -> bool {
        let lower = ext.to_lowercase();
        lower == "tmp" || lower == "log" || lower == "bak" || lower == "temp"
    }
}

fn simple_hash(s: &str) -> u64 {
    let mut hash: u64 = 5381;
    for b in s.bytes() {
        hash = ((hash << 5).wrapping_add(hash)).wrapping_add(b as u64);
    }
    hash
}
