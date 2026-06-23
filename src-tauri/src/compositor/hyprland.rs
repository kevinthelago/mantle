//! Raw Hyprland IPC over Unix sockets.
//!
//! Command socket (`.socket.sock`): send plain command string, read response until EOF.
//! JSON output: prefix command with `j/` (e.g. `j/workspaces`).
//! Event socket (`.socket2.sock`): newline-delimited `EVENT>>DATA` lines.

use crate::config::schema::{FocusedWindow, Workspace, WorkspaceState};
use serde::Deserialize;
use tauri::Emitter;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

fn socket_path(filename: &str) -> anyhow::Result<std::path::PathBuf> {
    let sig = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
        .map_err(|_| anyhow::anyhow!("HYPRLAND_INSTANCE_SIGNATURE not set"))?;
    let runtime = dirs::runtime_dir().ok_or_else(|| anyhow::anyhow!("XDG_RUNTIME_DIR not set"))?;
    Ok(runtime.join("hypr").join(sig).join(filename))
}

/// Send `command` to the Hyprland IPC command socket and return the raw response bytes.
async fn ipc_request(path: &std::path::Path, command: &str) -> anyhow::Result<Vec<u8>> {
    let mut stream = UnixStream::connect(path).await?;
    stream.write_all(command.as_bytes()).await?;
    // Signal end-of-command by shutting down the write half.
    stream.shutdown().await?;
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).await?;
    Ok(buf)
}

#[derive(Deserialize)]
struct RawHyprWorkspace {
    id: i32,
    name: String,
    monitor: String,
    windows: u32,
}

pub async fn get_workspaces() -> anyhow::Result<WorkspaceState> {
    let path = socket_path(".socket.sock")?;

    let ws_data = ipc_request(&path, "j/workspaces").await?;
    let raw: Vec<RawHyprWorkspace> = serde_json::from_slice(&ws_data)?;

    let active_data = ipc_request(&path, "j/activeworkspace").await?;
    let active: serde_json::Value =
        serde_json::from_slice(&active_data).unwrap_or(serde_json::Value::Null);
    let focused_ws_id = active["id"].as_i64().map(|i| i as i32);
    let focused_output = active["monitor"].as_str().map(String::from);

    let workspaces = raw
        .into_iter()
        .map(|w| {
            let focused = focused_ws_id.map_or(false, |id| id == w.id);
            Workspace {
                id: w.id,
                name: w.name,
                output: w.monitor,
                focused,
                urgent: false, // Hyprland's workspace list has no urgency field
                empty: w.windows == 0,
                representation: None,
            }
        })
        .collect();

    Ok(WorkspaceState {
        workspaces,
        focused_output,
    })
}

pub async fn get_focused_window() -> anyhow::Result<FocusedWindow> {
    let path = socket_path(".socket.sock")?;
    let data = ipc_request(&path, "j/activewindow").await?;
    let val: serde_json::Value = serde_json::from_slice(&data)?;
    // Empty workspace → {} from Hyprland
    if !val.is_object() || val.as_object().map_or(true, |m| m.is_empty()) {
        return Ok(FocusedWindow::default());
    }
    Ok(FocusedWindow {
        title: val["title"]
            .as_str()
            .filter(|s| !s.is_empty())
            .map(String::from),
        app_id: val["class"]
            .as_str()
            .filter(|s| !s.is_empty())
            .map(String::from),
    })
}

/// Dispatch a sway-style command to Hyprland (prefixes with `dispatch `).
pub async fn dispatch(command: &str) -> anyhow::Result<()> {
    let path = socket_path(".socket.sock")?;
    ipc_request(&path, &format!("dispatch {command}")).await?;
    Ok(())
}

/// Long-running task: reads `.socket2.sock` event lines and emits Tauri events.
/// Reconnects with exponential backoff on failure.
pub async fn event_loop(app: tauri::AppHandle) {
    let mut backoff_secs = 1u64;
    loop {
        match run_event_loop(&app).await {
            Ok(()) => tracing::info!("hyprland event loop closed; reconnecting"),
            Err(e) => {
                tracing::warn!("hyprland event loop error: {e:#}; reconnecting in {backoff_secs}s")
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(backoff_secs)).await;
        backoff_secs = (backoff_secs * 2).min(30);
    }
}

async fn run_event_loop(app: &tauri::AppHandle) -> anyhow::Result<()> {
    let path = socket_path(".socket2.sock")?;
    let stream = UnixStream::connect(&path).await?;
    let mut lines = BufReader::new(stream).lines();

    while let Some(line) = lines.next_line().await? {
        if line.starts_with("workspace>>") {
            if let Ok(state) = get_workspaces().await {
                app.emit("workspace-state", &state).ok();
            }
        } else if let Some(data) = line.strip_prefix("activewindow>>") {
            // Format: "class,title" — title may contain commas; split at first comma only.
            let focused = if let Some((class, title)) = data.split_once(',') {
                FocusedWindow {
                    title: if title.is_empty() {
                        None
                    } else {
                        Some(title.to_string())
                    },
                    app_id: if class.is_empty() {
                        None
                    } else {
                        Some(class.to_string())
                    },
                }
            } else {
                FocusedWindow::default()
            };
            app.emit("focused-window", &focused).ok();
        } else if line.starts_with("openwindow>>") || line.starts_with("closewindow>>") {
            // Window count changed — refresh workspace emptiness state.
            if let Ok(state) = get_workspaces().await {
                app.emit("workspace-state", &state).ok();
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activewindow_event_splits_at_first_comma() {
        let data = "firefox,mozilla - GitHub, Inc.";
        let (class, title) = data.split_once(',').unwrap();
        assert_eq!(class, "firefox");
        assert_eq!(title, "mozilla - GitHub, Inc.");
    }

    #[test]
    fn activewindow_event_title_preserves_commas() {
        let data = "kitty,nvim: src/main.rs, 120:5";
        let (class, title) = data.split_once(',').unwrap();
        assert_eq!(class, "kitty");
        assert_eq!(title, "nvim: src/main.rs, 120:5");
    }

    #[test]
    fn activewindow_event_no_comma_yields_default() {
        let data = "";
        let focused = if let Some((class, title)) = data.split_once(',') {
            FocusedWindow {
                title: if title.is_empty() {
                    None
                } else {
                    Some(title.to_string())
                },
                app_id: if class.is_empty() {
                    None
                } else {
                    Some(class.to_string())
                },
            }
        } else {
            FocusedWindow::default()
        };
        assert!(focused.title.is_none());
        assert!(focused.app_id.is_none());
    }

    #[test]
    fn raw_hypr_workspace_deserializes() {
        let json = r#"{"id":3,"name":"code","monitor":"DP-1","windows":2,"hasfullscreen":false}"#;
        let raw: RawHyprWorkspace = serde_json::from_str(json).unwrap();
        assert_eq!(raw.id, 3);
        assert_eq!(raw.name, "code");
        assert_eq!(raw.monitor, "DP-1");
        assert_eq!(raw.windows, 2);
    }

    #[test]
    fn workspace_empty_when_windows_zero() {
        let json = r#"{"id":1,"name":"1","monitor":"eDP-1","windows":0,"hasfullscreen":false}"#;
        let raw: RawHyprWorkspace = serde_json::from_str(json).unwrap();
        assert!(raw.windows == 0);
    }

    #[test]
    fn activewindow_empty_object_is_default() {
        let val: serde_json::Value = serde_json::from_str("{}").unwrap();
        let is_empty_obj = val.is_object() && val.as_object().map_or(true, |m| m.is_empty());
        assert!(is_empty_obj);
    }
}
