use tauri::State;

use crate::launcher::types::SearchResult;
use crate::launcher::LauncherState;

type CmdResult<T> = Result<T, String>;

#[tauri::command]
pub async fn search_apps(
    query: String,
    state: State<'_, LauncherState>,
) -> CmdResult<Vec<SearchResult>> {
    let svc = state.0.read().await;
    Ok(svc.search(&query))
}

#[tauri::command]
pub async fn launch_app(
    id: String,
    state: State<'_, LauncherState>,
) -> CmdResult<()> {
    let svc = state.0.read().await;
    svc.launch(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn run_command(
    cmd: String,
    state: State<'_, LauncherState>,
) -> CmdResult<()> {
    let svc = state.0.read().await;
    svc.run_command(&cmd).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn close_launcher(window: tauri::WebviewWindow) -> CmdResult<()> {
    window.hide().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_history(
    state: State<'_, LauncherState>,
) -> CmdResult<Vec<(String, u32)>> {
    let svc = state.0.read().await;
    Ok(svc.usage_counts())
}
