use crate::database::apps::ApplicationRecord;
use crate::error::AppResult;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

pub struct AppIndexer;

impl AppIndexer {
    /// Discover all applications from standard Windows locations.
    pub fn scan_all_sources() -> AppResult<Vec<ApplicationRecord>> {
        let mut apps: HashMap<String, ApplicationRecord> = HashMap::new();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        // 1. User & System Start Menu (recursive)
        for (source_name, path) in Self::get_start_menu_sources() {
            if path.exists() {
                debug!("Scanning start menu source {}: {:?}", source_name, path);
                Self::scan_start_menu(&path, &source_name, now, &mut apps);
            }
        }

        // 2. Desktop (shallow - only top level to avoid project build dirs)
        for (source_name, path) in Self::get_desktop_sources() {
            if path.exists() {
                debug!("Scanning desktop source {}: {:?}", source_name, path);
                Self::scan_desktop(&path, &source_name, now, &mut apps);
            }
        }

        // 3. User Programs directory (%LOCALAPPDATA%\Programs, depth 2)
        if let Ok(local_app_data) = env::var("LOCALAPPDATA") {
            let programs_dir = PathBuf::from(&local_app_data).join("Programs");
            if programs_dir.exists() {
                debug!("Scanning user programs: {:?}", programs_dir);
                Self::scan_directory_bounded(&programs_dir, "UserPrograms", 0, 2, now, &mut apps);
            }

            // 4. WindowsApps execution aliases (%LOCALAPPDATA%\Microsoft\WindowsApps, depth 1)
            let windows_apps_dir = PathBuf::from(&local_app_data).join("Microsoft").join("WindowsApps");
            if windows_apps_dir.exists() {
                debug!("Scanning WindowsApps aliases: {:?}", windows_apps_dir);
                Self::scan_directory_bounded(&windows_apps_dir, "WindowsApps", 0, 1, now, &mut apps);
            }
        }

        // 5. Windows System32 & Windows core utilities
        Self::scan_system_utilities(now, &mut apps);

        // 6. Registry App Paths
        #[cfg(windows)]
        {
            Self::scan_registry_app_paths(now, &mut apps);
        }

        let result: Vec<ApplicationRecord> = apps.into_values().collect();
        info!("Discovered {} installed applications.", result.len());
        Ok(result)
    }

    fn get_start_menu_sources() -> Vec<(String, PathBuf)> {
        let mut sources = Vec::new();

        if let Ok(app_data) = env::var("APPDATA") {
            sources.push((
                "StartMenuUser".into(),
                PathBuf::from(app_data)
                    .join("Microsoft")
                    .join("Windows")
                    .join("Start Menu")
                    .join("Programs"),
            ));
        }

        if let Ok(program_data) = env::var("ProgramData") {
            sources.push((
                "StartMenuSystem".into(),
                PathBuf::from(program_data)
                    .join("Microsoft")
                    .join("Windows")
                    .join("Start Menu")
                    .join("Programs"),
            ));
        }

        sources
    }

    fn get_desktop_sources() -> Vec<(String, PathBuf)> {
        let mut sources = Vec::new();

        if let Ok(user_profile) = env::var("USERPROFILE") {
            sources.push((
                "Desktop".into(),
                PathBuf::from(user_profile).join("Desktop"),
            ));
        }

        if let Ok(public_dir) = env::var("PUBLIC") {
            sources.push((
                "PublicDesktop".into(),
                PathBuf::from(public_dir).join("Desktop"),
            ));
        }

        sources
    }

    fn scan_start_menu(
        dir: &Path,
        source: &str,
        now: i64,
        apps: &mut HashMap<String, ApplicationRecord>,
    ) {
        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if path.is_dir() {
                if !name.starts_with('.') && !Self::is_ignored_dir_name(&name) {
                    Self::scan_start_menu(&path, source, now, apps);
                }
            } else if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    if ext_lower == "lnk" {
                        let file_stem = path
                            .file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| "Unknown App".into());

                        if Self::is_ignored_shortcut_name(&file_stem) {
                            continue;
                        }

                        let path_str = path.to_string_lossy().to_string();
                        let key = file_stem.to_lowercase();
                        let id = format!("app_{:x}", simple_hash(&path_str));

                        let record = ApplicationRecord {
                            id,
                            display_name: file_stem,
                            exe_path: path_str.clone(),
                            shortcut_path: Some(path_str),
                            arguments: None,
                            icon_path: None,
                            icon_index: 0,
                            source: source.to_string(),
                            indexed_at: now,
                            updated_at: now,
                        };

                        apps.insert(key, record);
                    }
                }
            }
        }
    }

    fn scan_desktop(
        dir: &Path,
        source: &str,
        now: i64,
        apps: &mut HashMap<String, ApplicationRecord>,
    ) {
        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    if ext_lower == "lnk" || ext_lower == "exe" {
                        let file_stem = path
                            .file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| "Unknown App".into());

                        if Self::is_ignored_executable_name(&file_stem)
                            || Self::is_ignored_shortcut_name(&file_stem)
                        {
                            continue;
                        }

                        let path_str = path.to_string_lossy().to_string();
                        let key = file_stem.to_lowercase();

                        // Prefer existing Start Menu shortcut if present
                        if apps.contains_key(&key) && ext_lower == "exe" {
                            continue;
                        }

                        let id = format!("app_{:x}", simple_hash(&path_str));
                        let record = ApplicationRecord {
                            id,
                            display_name: clean_app_name(&file_stem),
                            exe_path: path_str.clone(),
                            shortcut_path: if ext_lower == "lnk" {
                                Some(path_str)
                            } else {
                                None
                            },
                            arguments: None,
                            icon_path: None,
                            icon_index: 0,
                            source: source.to_string(),
                            indexed_at: now,
                            updated_at: now,
                        };

                        apps.insert(key, record);
                    }
                }
            }
        }
    }

    fn scan_directory_bounded(
        dir: &Path,
        source: &str,
        depth: usize,
        max_depth: usize,
        now: i64,
        apps: &mut HashMap<String, ApplicationRecord>,
    ) {
        if depth > max_depth {
            return;
        }

        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if path.is_dir() {
                if !name.starts_with('.') && !Self::is_ignored_dir_name(&name) {
                    Self::scan_directory_bounded(&path, source, depth + 1, max_depth, now, apps);
                }
            } else if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    if ext_lower == "exe" || ext_lower == "lnk" {
                        let file_stem = path
                            .file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| "Unknown App".into());

                        if Self::is_ignored_executable_name(&file_stem)
                            || Self::is_ignored_shortcut_name(&file_stem)
                        {
                            continue;
                        }

                        let path_str = path.to_string_lossy().to_string();
                        let display_name = clean_app_name(&file_stem);
                        let key = display_name.to_lowercase();

                        if !apps.contains_key(&key) {
                            let id = format!("app_{:x}", simple_hash(&path_str));
                            let record = ApplicationRecord {
                                id,
                                display_name,
                                exe_path: path_str.clone(),
                                shortcut_path: if ext_lower == "lnk" {
                                    Some(path_str)
                                } else {
                                    None
                                },
                                arguments: None,
                                icon_path: None,
                                icon_index: 0,
                                source: source.to_string(),
                                indexed_at: now,
                                updated_at: now,
                            };

                            apps.insert(key, record);
                        }
                    }
                }
            }
        }
    }

    fn scan_system_utilities(now: i64, apps: &mut HashMap<String, ApplicationRecord>) {
        let system_root = env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into());
        let sys32 = PathBuf::from(&system_root).join("System32");

        let common_tools = [
            ("notepad.exe", "Notepad"),
            ("calc.exe", "Calculator"),
            ("mspaint.exe", "Paint"),
            ("taskmgr.exe", "Task Manager"),
            ("cmd.exe", "Command Prompt"),
            ("powershell.exe", "Windows PowerShell"),
            ("regedit.exe", "Registry Editor"),
            ("mstsc.exe", "Remote Desktop Connection"),
            ("SnippingTool.exe", "Snipping Tool"),
            ("control.exe", "Control Panel"),
            ("cleanmgr.exe", "Disk Cleanup"),
            ("charmap.exe", "Character Map"),
            ("explorer.exe", "File Explorer"),
            ("write.exe", "WordPad"),
            ("resmon.exe", "Resource Monitor"),
            ("perfmon.exe", "Performance Monitor"),
            ("eventvwr.exe", "Event Viewer"),
            ("dxdiag.exe", "DirectX Diagnostic Tool"),
            ("magnify.exe", "Magnifier"),
        ];

        for (exe_file, display_name) in common_tools {
            let key = display_name.to_lowercase();
            if apps.contains_key(&key) {
                continue;
            }

            let mut target_path = sys32.join(exe_file);
            if !target_path.exists() {
                target_path = PathBuf::from(&system_root).join(exe_file);
            }

            if target_path.exists() {
                let path_str = target_path.to_string_lossy().to_string();
                let id = format!("app_{:x}", simple_hash(&path_str));

                apps.insert(
                    key,
                    ApplicationRecord {
                        id,
                        display_name: display_name.into(),
                        exe_path: path_str,
                        shortcut_path: None,
                        arguments: None,
                        icon_path: None,
                        icon_index: 0,
                        source: "System32".into(),
                        indexed_at: now,
                        updated_at: now,
                    },
                );
            }
        }
    }

    #[cfg(windows)]
    fn scan_registry_app_paths(now: i64, apps: &mut HashMap<String, ApplicationRecord>) {
        use windows::core::PCWSTR;
        use windows::Win32::System::Registry::{
            RegCloseKey, RegEnumKeyExW, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER,
            HKEY_LOCAL_MACHINE, KEY_READ, REG_SZ,
        };

        let roots = [(HKEY_LOCAL_MACHINE, "HKLM"), (HKEY_CURRENT_USER, "HKCU")];
        for (root_key, source_name) in roots {
            let subkey: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\App Paths"
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            let mut h_key = HKEY::default();
            let status = unsafe {
                RegOpenKeyExW(
                    root_key,
                    PCWSTR::from_raw(subkey.as_ptr()),
                    0,
                    KEY_READ,
                    &mut h_key,
                )
            };

            if status.is_ok() {
                let mut index = 0;
                let mut name_buf = [0u16; 260];
                loop {
                    let mut name_len = name_buf.len() as u32;
                    let enum_status = unsafe {
                        RegEnumKeyExW(
                            h_key,
                            index,
                            windows::core::PWSTR::from_raw(name_buf.as_mut_ptr()),
                            &mut name_len,
                            None,
                            windows::core::PWSTR::null(),
                            None,
                            None,
                        )
                    };

                    if enum_status.is_err() {
                        break;
                    }

                    let app_key_name = String::from_utf16_lossy(&name_buf[..name_len as usize]);
                    index += 1;

                    let app_subkey: Vec<u16> = format!(
                        "Software\\Microsoft\\Windows\\CurrentVersion\\App Paths\\{}",
                        app_key_name
                    )
                    .encode_utf16()
                    .chain(std::iter::once(0))
                    .collect();

                    let mut h_app_key = HKEY::default();
                    let app_status = unsafe {
                        RegOpenKeyExW(
                            root_key,
                            PCWSTR::from_raw(app_subkey.as_ptr()),
                            0,
                            KEY_READ,
                            &mut h_app_key,
                        )
                    };

                    if app_status.is_ok() {
                        let mut path_buf = [0u16; 512];
                        let mut path_len = (path_buf.len() * 2) as u32;
                        let mut val_type = REG_SZ;
                        let val_status = unsafe {
                            RegQueryValueExW(
                                h_app_key,
                                PCWSTR::null(),
                                None,
                                Some(&mut val_type),
                                Some(path_buf.as_mut_ptr() as *mut u8),
                                Some(&mut path_len),
                            )
                        };

                        unsafe {
                            let _ = RegCloseKey(h_app_key);
                        }

                        if val_status.is_ok() && (path_len / 2) > 1 {
                            let raw_path = String::from_utf16_lossy(
                                &path_buf[..(path_len / 2).saturating_sub(1) as usize],
                            )
                            .trim_matches('"')
                            .to_string();

                            let target_path = PathBuf::from(&raw_path);
                            if target_path.exists() && target_path.is_file() {
                                let file_stem = target_path
                                    .file_stem()
                                    .map(|s| s.to_string_lossy().to_string())
                                    .unwrap_or_else(|| app_key_name.clone());

                                if Self::is_ignored_executable_name(&file_stem) {
                                    continue;
                                }

                                let display_name = clean_app_name(&file_stem);
                                let key = display_name.to_lowercase();

                                if !apps.contains_key(&key) {
                                    let path_str = target_path.to_string_lossy().to_string();
                                    let id = format!("app_{:x}", simple_hash(&path_str));
                                    apps.insert(
                                        key,
                                        ApplicationRecord {
                                            id,
                                            display_name,
                                            exe_path: path_str,
                                            shortcut_path: None,
                                            arguments: None,
                                            icon_path: None,
                                            icon_index: 0,
                                            source: format!("AppPaths_{}", source_name),
                                            indexed_at: now,
                                            updated_at: now,
                                        },
                                    );
                                }
                            }
                        }
                    }
                }

                unsafe {
                    let _ = RegCloseKey(h_key);
                }
            }
        }
    }

    fn is_ignored_dir_name(name: &str) -> bool {
        let lower = name.to_lowercase();
        lower.starts_with('.')
            || lower == "node_modules"
            || lower == "target"
            || lower == ".cargo"
            || lower == "dist"
            || lower == "build"
            || lower == "bin"
            || lower == "obj"
            || lower == "venv"
            || lower == "__pycache__"
            || lower == "$recycle.bin"
            || lower == "system volume information"
    }

    fn is_ignored_executable_name(name: &str) -> bool {
        let lower = name.to_lowercase();
        lower.starts_with("uninstall")
            || lower.starts_with("unins0")
            || lower.contains("crashpad")
            || lower.contains("helper")
            || lower.contains("daemon")
            || lower.contains("build-script")
            || lower.contains("build_script")
            || lower == "esbuild"
            || lower == "windows-kill"
            || lower == "vc_redist"
            || lower == "vcredist"
    }

    fn is_ignored_shortcut_name(name: &str) -> bool {
        let lower = name.to_lowercase();
        lower.starts_with("uninstall")
            || lower.contains("documentation")
            || lower.contains("release notes")
            || lower.contains("user manual")
            || lower.contains("what is new")
            || lower.contains("read me")
            || lower.contains("readme")
    }
}

fn clean_app_name(raw: &str) -> String {
    let lower = raw.to_lowercase();
    match lower.as_str() {
        "notepad" => "Notepad".into(),
        "calc" => "Calculator".into(),
        "mspaint" => "Paint".into(),
        "taskmgr" => "Task Manager".into(),
        "cmd" => "Command Prompt".into(),
        "powershell" => "Windows PowerShell".into(),
        "regedit" => "Registry Editor".into(),
        "mstsc" => "Remote Desktop Connection".into(),
        "snippingtool" => "Snipping Tool".into(),
        "control" => "Control Panel".into(),
        "cleanmgr" => "Disk Cleanup".into(),
        "charmap" => "Character Map".into(),
        "explorer" => "File Explorer".into(),
        "write" => "WordPad".into(),
        "resmon" => "Resource Monitor".into(),
        "perfmon" => "Performance Monitor".into(),
        "eventvwr" => "Event Viewer".into(),
        "chrome" => "Google Chrome".into(),
        "brave" => "Brave".into(),
        "msedge" => "Microsoft Edge".into(),
        "devenv" => "Visual Studio".into(),
        "code" => "Visual Studio Code".into(),
        "winword" => "Microsoft Word".into(),
        "excel" => "Microsoft Excel".into(),
        "powerpnt" => "Microsoft PowerPoint".into(),
        "msaccess" => "Microsoft Access".into(),
        "acrord32" => "Adobe Reader".into(),
        "vlc" => "VLC media player".into(),
        _ => raw.to_string(),
    }
}

fn simple_hash(s: &str) -> u64 {
    let mut hash: u64 = 5381;
    for b in s.bytes() {
        hash = ((hash << 5).wrapping_add(hash)).wrapping_add(b as u64);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_all_sources() {
        let apps = AppIndexer::scan_all_sources().unwrap();
        println!("Indexed {} applications", apps.len());
        assert!(!apps.is_empty(), "App scan should find applications on Windows");

        // Verify Notepad and Antigravity or common apps are discovered
        let names: Vec<String> = apps.iter().map(|a| a.display_name.to_lowercase()).collect();
        assert!(
            names.iter().any(|n| n.contains("notepad")),
            "Notepad should be discovered"
        );
    }
}


