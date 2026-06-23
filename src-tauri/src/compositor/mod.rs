//! Compositor IPC bridge — detects sway or Hyprland at runtime and delegates.
//!
//! All public functions are no-ops on non-Linux platforms so the bridge commands
//! compile and return sensible defaults everywhere.

#[cfg(target_os = "linux")]
mod hyprland;
#[cfg(target_os = "linux")]
mod sway;

#[cfg(target_os = "linux")]
use crate::config::schema::{FocusedWindow, WorkspaceState};

/// Returns the current workspace list from the running compositor.
/// Falls back to an empty [`WorkspaceState`] when no supported compositor is detected.
#[cfg(target_os = "linux")]
pub async fn query_workspace_state() -> anyhow::Result<WorkspaceState> {
    if let Ok(socket) = std::env::var("SWAYSOCK") {
        return sway::get_workspaces(&socket).await;
    }
    if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
        return hyprland::get_workspaces().await;
    }
    Ok(WorkspaceState::default())
}

/// Returns the currently focused window from the running compositor.
#[cfg(target_os = "linux")]
pub async fn query_focused_window() -> anyhow::Result<FocusedWindow> {
    if let Ok(socket) = std::env::var("SWAYSOCK") {
        return sway::get_focused_window(&socket).await;
    }
    if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
        return hyprland::get_focused_window().await;
    }
    Ok(FocusedWindow::default())
}

/// Dispatches a sway-style command (e.g. `"workspace 1"`) to the running compositor.
/// Hyprland commands are automatically prefixed with `dispatch `.
#[cfg(target_os = "linux")]
pub async fn dispatch_command(command: &str) -> anyhow::Result<()> {
    if let Ok(socket) = std::env::var("SWAYSOCK") {
        return sway::run_command(&socket, command).await;
    }
    if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
        return hyprland::dispatch(command).await;
    }
    Ok(())
}

/// Spawns a background task that subscribes to compositor events and emits
/// `workspace-state` and `focused-window` Tauri events on changes.
#[cfg(target_os = "linux")]
pub fn spawn_event_relay(app: tauri::AppHandle) {
    if let Ok(socket) = std::env::var("SWAYSOCK") {
        tauri::async_runtime::spawn(sway::event_loop(socket, app));
    } else if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
        tauri::async_runtime::spawn(hyprland::event_loop(app));
    }
}
