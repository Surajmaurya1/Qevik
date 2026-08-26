use crate::core::state::{AppSettings, AppState, IndexStatus};
use crate::search::engine::SearchEngine;
use crate::search::query::ResultType;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tauri::{AppHandle, Manager, State};
use tracing::{error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultDto {
    pub id: String,
    pub result_type: String,
    pub display_name: String,
    pub subtitle: String,
    pub score: f64,
    pub icon_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponseDto {
    pub results: Vec<SearchResultDto>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchResponseDto {
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconResponseDto {
    pub data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfoDto {
    pub name: String,
    pub version: String,
    pub tauri_version: String,
}

fn convert_result_type_to_str(rt: &ResultType) -> &'static str {
    match rt {
        ResultType::App => "app",
        ResultType::File => "file",
        ResultType::Folder => "folder",
        ResultType::Command => "command",
        ResultType::Calculator => "calculator",
        ResultType::Web => "web",
    }
}

#[tauri::command]
pub async fn search(
    query: String,
    state: State<'_, Arc<AppState>>,
) -> Result<SearchResponseDto, String> {
    let start = Instant::now();
    let settings = state.settings.read().await;
    let max_results = settings.max_results;
    drop(settings);

    match SearchEngine::execute(&state, &query, max_results).await {
        Ok(results) => {
            let dtos = results
                .into_iter()
                .map(|r| SearchResultDto {
                    id: r.id,
                    result_type: convert_result_type_to_str(&r.result_type).to_string(),
                    display_name: r.display_name,
                    subtitle: r.subtitle,
                    score: r.score,
                    icon_id: r.icon_id,
                })
                .collect();

            Ok(SearchResponseDto {
                results: dtos,
                duration_ms: start.elapsed().as_millis() as u64,
            })
        }
        Err(e) => {
            error!("Search error: {}", e);
            Err(e.to_string())
        }
    }
}

#[cfg(windows)]
fn execute_target(
    target_path: &str,
    arguments: Option<&str>,
    is_folder: bool,
    is_web: bool,
) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let wide_op: Vec<u16> = OsStr::new("open")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let wide_path: Vec<u16> = OsStr::new(target_path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let wide_args: Option<Vec<u16>> = arguments.map(|a| {
        OsStr::new(a)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    });

    unsafe {
        let instance = ShellExecuteW(
            None,
            PCWSTR::from_raw(wide_op.as_ptr()),
            PCWSTR::from_raw(wide_path.as_ptr()),
            wide_args
                .as_ref()
                .map(|a| PCWSTR::from_raw(a.as_ptr()))
                .unwrap_or(PCWSTR::null()),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        );

        if (instance.0 as isize) > 32 {
            return Ok(());
        }
    }

    // Fallback handlers if ShellExecute returns error code <= 32
    if is_folder || std::path::Path::new(target_path).is_dir() {
        let mut cmd = std::process::Command::new("explorer.exe");
        cmd.arg(target_path);
        if let Err(e) = cmd.spawn() {
            return Err(format!("Failed to open folder: {}", e));
        }
        return Ok(());
    }

    if is_web {
        let mut cmd = std::process::Command::new("cmd.exe");
        cmd.args(["/C", "start", "", target_path]);
        if let Err(e) = cmd.spawn() {
            return Err(format!("Failed to open URL: {}", e));
        }
        return Ok(());
    }

    let mut cmd = std::process::Command::new("cmd.exe");
    cmd.args(["/C", "start", "", target_path]);
    if let Some(args) = arguments {
        cmd.arg(args);
    }
    if let Err(e) = cmd.spawn() {
        return Err(format!("Failed to launch file/app: {}", e));
    }

    Ok(())
}

#[tauri::command]
pub async fn launch(
    id: String,
    result_type: String,
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<LaunchResponseDto, String> {
    info!("Launch request: id={}, type={}", id, result_type);

    // Hide launcher window before launching per Section 8
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }

    // Record usage in DB
    {
        let db = state.db.lock().await;
        let _ = crate::database::usage::increment_usage(&db, &id, &result_type);
        let _ = crate::database::history::record_launch_history(&db, "", &id, &result_type, &id);
    }

    // Resolve target path and launch
    #[cfg(windows)]
    {
        let db = state.db.lock().await;
        let mut target_path = id.clone();
        let mut arguments: Option<String> = None;
        let is_folder = result_type == "folder";
        let is_web = result_type == "web";

        match result_type.as_str() {
            "app" => {
                if let Ok(Some(app_rec)) =
                    crate::database::apps::get_application_by_id_or_path(&db, &id)
                {
                    target_path = if let Some(shortcut) = app_rec.shortcut_path {
                        if std::path::Path::new(&shortcut).exists() {
                            shortcut
                        } else {
                            app_rec.exe_path
                        }
                    } else {
                        app_rec.exe_path
                    };
                    arguments = app_rec.arguments;
                }
            }
            "file" => {
                if let Ok(Some(file_rec)) = crate::database::files::get_file_by_id_or_path(&db, &id)
                {
                    target_path = file_rec.path;
                }
            }
            "folder" => {
                if let Ok(Some(folder_rec)) =
                    crate::database::folders::get_folder_by_id_or_path(&db, &id)
                {
                    target_path = folder_rec.path;
                }
            }
            "web" => {
                target_path = id.clone();
            }
            _ => {}
        }
        drop(db);

        if let Err(e) = execute_target(&target_path, arguments.as_deref(), is_folder, is_web) {
            error!("Launch failed for {}: {}", target_path, e);
            return Ok(LaunchResponseDto {
                success: false,
                error: Some(e),
            });
        }
    }

    Ok(LaunchResponseDto {
        success: true,
        error: None,
    })
}

#[tauri::command]
pub async fn get_icon(_id: String) -> Result<IconResponseDto, String> {
    Ok(IconResponseDto { data: None })
}

#[tauri::command]
pub async fn hide_launcher(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    Ok(())
}

#[tauri::command]
pub async fn get_settings(state: State<'_, Arc<AppState>>) -> Result<AppSettings, String> {
    let settings = state.settings.read().await;
    Ok(settings.clone())
}

#[tauri::command]
pub async fn update_settings(
    settings: AppSettings,
    state: State<'_, Arc<AppState>>,
) -> Result<bool, String> {
    {
        let mut current = state.settings.write().await;
        *current = settings.clone();
    }
    let db = state.db.lock().await;
    let _ = crate::database::settings::save_settings_to_db(&db, &settings);
    Ok(true)
}

#[tauri::command]
pub async fn get_recent_results(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<SearchResultDto>, String> {
    let db = state.db.lock().await;
    match crate::database::history::get_recent_history(&db, 8) {
        Ok(history) => {
            let results = history
                .into_iter()
                .map(|h| SearchResultDto {
                    id: h.result_id,
                    result_type: h.result_type,
                    display_name: h.result_name,
                    subtitle: "Recent launch".into(),
                    score: 1.0,
                    icon_id: None,
                })
                .collect();
            Ok(results)
        }
        Err(_) => Ok(vec![]),
    }
}

#[tauri::command]
pub async fn get_index_status(state: State<'_, Arc<AppState>>) -> Result<IndexStatus, String> {
    let status = state.index_status.read().await;
    Ok(status.clone())
}

#[tauri::command]
pub async fn rebuild_index(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    info!("Rebuild index triggered via IPC");
    crate::indexer::manager::IndexManager::start_background_indexing(state.inner().clone());
    Ok(())
}

#[tauri::command]
pub async fn get_app_info() -> Result<AppInfoDto, String> {
    Ok(AppInfoDto {
        name: "Spotlight for Windows".into(),
        version: "1.0.0".into(),
        tauri_version: "2.2.0".into(),
    })
}
