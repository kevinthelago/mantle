//! Raw sway IPC over a Unix socket.
//!
//! Protocol: `b"i3-ipc"` magic (6 bytes) + u32 LE payload length
//!           + u32 LE message type + JSON payload.

use crate::config::schema::{FocusedWindow, Workspace, WorkspaceState};
use serde::Deserialize;
use serde_json::Value;
use tauri::Emitter;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

const MAGIC: &[u8; 6] = b"i3-ipc";
const MSG_RUN_COMMAND: u32 = 0;
const MSG_GET_WORKSPACES: u32 = 1;
const MSG_SUBSCRIBE: u32 = 2;
const MSG_GET_TREE: u32 = 4;
const EVT_WORKSPACE: u32 = 0x8000_0000;
const EVT_WINDOW: u32 = 0x8000_0003;

#[derive(Deserialize)]
struct RawWorkspace {
    num: i32,
    name: String,
    #[serde(default)]
    output: String,
    focused: bool,
    #[serde(default)]
    urgent: bool,
    representation: Option<String>,
}

async fn send_msg(stream: &mut UnixStream, msg_type: u32, payload: &[u8]) -> anyhow::Result<()> {
    let mut buf = Vec::with_capacity(14 + payload.len());
    buf.extend_from_slice(MAGIC);
    buf.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    buf.extend_from_slice(&msg_type.to_le_bytes());
    buf.extend_from_slice(payload);
    stream.write_all(&buf).await?;
    Ok(())
}

async fn recv_msg(stream: &mut UnixStream) -> anyhow::Result<(u32, Vec<u8>)> {
    let mut header = [0u8; 14];
    stream.read_exact(&mut header).await?;
    anyhow::ensure!(&header[..6] == MAGIC, "invalid sway IPC magic");
    let len = u32::from_le_bytes(header[6..10].try_into().unwrap()) as usize;
    let msg_type = u32::from_le_bytes(header[10..14].try_into().unwrap());
    let mut payload = vec![0u8; len];
    stream.read_exact(&mut payload).await?;
    Ok((msg_type, payload))
}

pub async fn get_workspaces(socket: &str) -> anyhow::Result<WorkspaceState> {
    let mut stream = UnixStream::connect(socket).await?;
    send_msg(&mut stream, MSG_GET_WORKSPACES, &[]).await?;
    let (_, data) = recv_msg(&mut stream).await?;
    let raw: Vec<RawWorkspace> = serde_json::from_slice(&data)?;

    let focused_output = raw.iter().find(|w| w.focused).map(|w| w.output.clone());
    let workspaces = raw
        .into_iter()
        .map(|w| Workspace {
            id: w.num,
            name: w.name,
            output: w.output,
            focused: w.focused,
            urgent: w.urgent,
            // sway omits representation for empty workspaces or sets it to ""
            empty: w.representation.as_deref().map_or(true, str::is_empty),
            representation: w.representation,
        })
        .collect();

    Ok(WorkspaceState {
        workspaces,
        focused_output,
    })
}

/// Recursively searches the sway tree for the focused `con`/`floating_con` node.
fn find_focused_window(node: &Value) -> Option<FocusedWindow> {
    let focused = node["focused"].as_bool().unwrap_or(false);
    let node_type = node["type"].as_str().unwrap_or("");

    if focused && (node_type == "con" || node_type == "floating_con") {
        let title = node["name"]
            .as_str()
            .filter(|s| !s.is_empty())
            .map(String::from);
        let app_id = node["app_id"]
            .as_str()
            .filter(|s| !s.is_empty())
            .map(String::from);
        if title.is_some() || app_id.is_some() {
            return Some(FocusedWindow { title, app_id });
        }
    }

    for key in &["nodes", "floating_nodes"] {
        if let Some(children) = node[key].as_array() {
            for child in children {
                if let Some(win) = find_focused_window(child) {
                    return Some(win);
                }
            }
        }
    }
    None
}

pub async fn get_focused_window(socket: &str) -> anyhow::Result<FocusedWindow> {
    let mut stream = UnixStream::connect(socket).await?;
    send_msg(&mut stream, MSG_GET_TREE, &[]).await?;
    let (_, data) = recv_msg(&mut stream).await?;
    let tree: Value = serde_json::from_slice(&data)?;
    Ok(find_focused_window(&tree).unwrap_or_default())
}

pub async fn run_command(socket: &str, command: &str) -> anyhow::Result<()> {
    let mut stream = UnixStream::connect(socket).await?;
    send_msg(&mut stream, MSG_RUN_COMMAND, command.as_bytes()).await?;
    recv_msg(&mut stream).await?;
    Ok(())
}

/// Long-running task: subscribes to workspace/window events and emits Tauri events.
/// Reconnects with exponential backoff on failure.
pub async fn event_loop(socket: String, app: tauri::AppHandle) {
    let mut backoff_secs = 1u64;
    loop {
        match subscribe_and_relay(&socket, &app).await {
            Ok(()) => tracing::info!("sway event loop closed; reconnecting"),
            Err(e) => {
                tracing::warn!("sway event loop error: {e:#}; reconnecting in {backoff_secs}s")
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(backoff_secs)).await;
        backoff_secs = (backoff_secs * 2).min(30);
    }
}

async fn subscribe_and_relay(socket: &str, app: &tauri::AppHandle) -> anyhow::Result<()> {
    let mut stream = UnixStream::connect(socket).await?;

    let topics = serde_json::json!(["workspace", "window"]);
    send_msg(&mut stream, MSG_SUBSCRIBE, &serde_json::to_vec(&topics)?).await?;
    let (_, ack) = recv_msg(&mut stream).await?;
    let ack_val: Value = serde_json::from_slice(&ack)?;
    anyhow::ensure!(
        ack_val["success"].as_bool() == Some(true),
        "sway subscription rejected"
    );

    loop {
        let (evt_type, data) = recv_msg(&mut stream).await?;
        let json: Value = serde_json::from_slice(&data).unwrap_or(Value::Null);

        if evt_type == EVT_WORKSPACE {
            if let Ok(state) = get_workspaces(socket).await {
                app.emit("workspace-state", &state).ok();
            }
        } else if evt_type == EVT_WINDOW {
            let change = json["change"].as_str().unwrap_or("");
            if matches!(change, "title" | "focus" | "close") {
                if change == "close" {
                    // Closing a window may empty a workspace
                    if let Ok(state) = get_workspaces(socket).await {
                        app.emit("workspace-state", &state).ok();
                    }
                }
                let container = &json["container"];
                let title = container["name"]
                    .as_str()
                    .filter(|s| !s.is_empty())
                    .map(String::from);
                let app_id = container["app_id"]
                    .as_str()
                    .filter(|s| !s.is_empty())
                    .map(String::from);
                app.emit("focused-window", &FocusedWindow { title, app_id })
                    .ok();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn find_focused_window_in_nested_tree() {
        let tree = json!({
            "type": "root",
            "focused": false,
            "nodes": [{
                "type": "output",
                "focused": false,
                "nodes": [{
                    "type": "workspace",
                    "focused": false,
                    "nodes": [{
                        "type": "con",
                        "focused": true,
                        "name": "Terminal",
                        "app_id": "kitty",
                        "nodes": [],
                        "floating_nodes": []
                    }],
                    "floating_nodes": []
                }],
                "floating_nodes": []
            }],
            "floating_nodes": []
        });
        let win = find_focused_window(&tree).unwrap();
        assert_eq!(win.title.as_deref(), Some("Terminal"));
        assert_eq!(win.app_id.as_deref(), Some("kitty"));
    }

    #[test]
    fn find_focused_window_floating_con() {
        let tree = json!({
            "type": "root",
            "focused": false,
            "nodes": [],
            "floating_nodes": [{
                "type": "floating_con",
                "focused": true,
                "name": "Floating App",
                "app_id": "floatapp",
                "nodes": [],
                "floating_nodes": []
            }]
        });
        let win = find_focused_window(&tree).unwrap();
        assert_eq!(win.app_id.as_deref(), Some("floatapp"));
    }

    #[test]
    fn find_focused_window_returns_none_on_empty_tree() {
        let tree = json!({
            "type": "root",
            "focused": false,
            "nodes": [],
            "floating_nodes": []
        });
        assert!(find_focused_window(&tree).is_none());
    }

    #[test]
    fn find_focused_window_skips_workspace_node() {
        // A focused workspace node should NOT match — only con/floating_con.
        let tree = json!({
            "type": "root",
            "focused": false,
            "nodes": [{
                "type": "workspace",
                "focused": true,
                "name": "1",
                "nodes": [],
                "floating_nodes": []
            }],
            "floating_nodes": []
        });
        assert!(find_focused_window(&tree).is_none());
    }

    #[test]
    fn workspace_empty_flag_from_representation() {
        // None or "" → empty; anything else → not empty
        assert!(None::<&str>.map_or(true, str::is_empty));
        assert!(Some("").map_or(true, str::is_empty));
        assert!(!Some("H[kitty]").map_or(true, str::is_empty));
    }

    #[test]
    fn raw_workspace_deserializes() {
        let json = r#"{
            "num": 2,
            "name": "2",
            "output": "DP-1",
            "focused": false,
            "urgent": true,
            "representation": "H[term browser]"
        }"#;
        let raw: RawWorkspace = serde_json::from_str(json).unwrap();
        assert_eq!(raw.num, 2);
        assert_eq!(raw.output, "DP-1");
        assert!(raw.urgent);
        assert!(!raw.focused);
        assert_eq!(raw.representation.as_deref(), Some("H[term browser]"));
    }
}
