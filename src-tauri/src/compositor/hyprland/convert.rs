/// Normalization layer: hyprland crate types → compositor types.
///
/// All conversion is pure (no I/O) to keep it testable in isolation.
use hyprland::data::{Client as HyprClient, Monitor as HyprMonitor, Workspace as HyprWorkspace};

use crate::compositor::{
    event::CompositorEvent,
    types::{Output, Window, Workspace},
};

// ── Workspace ─────────────────────────────────────────────────────────────────

/// Convert a Hyprland workspace to a normalized `Workspace`.
///
/// # Special workspace representation
/// Hyprland special workspaces (e.g. `special:magic`) have a **negative** ID
/// assigned by the compositor. The `is_special` flag is set; the `name` field
/// retains the full Hyprland name (e.g. `"special:magic"`) for display purposes.
pub fn workspace_from_hyprland(ws: &HyprWorkspace) -> Workspace {
    let is_special = ws.id < 0 || ws.name.starts_with("special:");
    Workspace {
        id: ws.id,
        name: ws.name.clone(),
        output: Some(ws.monitor.clone()),
        focused: false, // filled in by snapshot (compare with active workspace)
        urgent: false,  // Hyprland does not expose workspace urgency
        is_special,
        window_ids: Vec::new(), // populated from client list
    }
}

// ── Output / Monitor ──────────────────────────────────────────────────────────

pub fn output_from_hyprland(m: &HyprMonitor) -> Output {
    Output {
        name: m.name.clone(),
        make: Some(m.make.clone()).filter(|s| !s.is_empty()),
        model: Some(m.model.clone()).filter(|s| !s.is_empty()),
        width: m.width as u32,
        height: m.height as u32,
        refresh_hz: m.refresh_rate,
        x: m.x,
        y: m.y,
        active: true, // Hyprland only reports active monitors
        scale: m.scale,
        focused: m.focused,
    }
}

// ── Window / Client ───────────────────────────────────────────────────────────

/// Convert a Hyprland client to a normalized `Window`.
///
/// The client `address` (`"0x55f4a1b2c3d0"`) is parsed as a base-16 `u64`.
pub fn window_from_hyprland(c: &HyprClient) -> Window {
    Window {
        id: parse_address(&c.address.to_string()),
        title: c.title.clone(),
        app_id: Some(c.class.clone()).filter(|s| !s.is_empty()),
        workspace_id: Some(c.workspace.id),
        output: None,   // filled in by snapshot (map monitor id → name)
        focused: false, // filled in by snapshot (compare address with active client)
        floating: c.floating,
        fullscreen: c.fullscreen != hyprland::data::FullscreenMode::None,
        pid: Some(c.pid as u32),
    }
}

/// Parse a Hyprland hex address string (`"0x55f4..."`) to a `u64` window ID.
pub fn parse_address(addr: &str) -> u64 {
    let hex = addr.strip_prefix("0x").unwrap_or(addr);
    u64::from_str_radix(hex, 16).unwrap_or(0)
}

// ── Event data structs (from event listener callbacks) ────────────────────────

/// Normalized event from the Hyprland event listener.
/// Each variant maps 1-to-1 to a Hyprland IPC event.
pub enum HyprRawEvent {
    WorkspaceChanged(String),
    WorkspaceAdded(String),
    WorkspaceDeleted(String),
    ActiveWindowChanged {
        class: String,
        title: String,
    },
    WindowOpened {
        address: String,
        workspace_name: String,
        class: String,
        title: String,
    },
    WindowClosed(String),
    WindowMoved {
        address: String,
        workspace_name: String,
    },
    WindowTitleChanged(String),
    MonitorAdded(String),
    MonitorRemoved(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_hyprland_workspace(id: i32, name: &str, monitor: &str) -> HyprWorkspace {
        serde_json::from_value(serde_json::json!({
            "id": id,
            "name": name,
            "monitor": monitor,
            "monitorID": 0,
            "windows": 0,
            "hasfullscreen": false,
            "lastwindow": "0x0",
            "lastwindowtitle": ""
        }))
        .unwrap()
    }

    fn make_hyprland_special_workspace() -> HyprWorkspace {
        make_hyprland_workspace(-99, "special:magic", "eDP-1")
    }

    fn make_hyprland_monitor() -> HyprMonitor {
        serde_json::from_value(serde_json::json!({
            "id": 0,
            "name": "eDP-1",
            "description": "LG IPS eDP-1",
            "make": "LG",
            "model": "LG IPS",
            "serial": "0x000000",
            "width": 1920,
            "height": 1080,
            "refreshRate": 60.0,
            "x": 0,
            "y": 0,
            "activeWorkspace": {"id": 1, "name": "1"},
            "specialWorkspace": {"id": 0, "name": ""},
            "reserved": [0, 0, 0, 0],
            "scale": 1.0,
            "transform": 0,
            "focused": true,
            "dpmsStatus": true,
            "vrr": false,
            "activelyTearing": false
        }))
        .unwrap()
    }

    fn make_hyprland_client() -> HyprClient {
        serde_json::from_value(serde_json::json!({
            "address": "0x00000000002a",
            "mapped": true,
            "hidden": false,
            "at": [0, 0],
            "size": [800, 600],
            "workspace": {"id": 1, "name": "1"},
            "floating": false,
            "pseudo": false,
            "monitor": 0,
            "class": "firefox",
            "title": "My Window",
            "initialClass": "firefox",
            "initialTitle": "My Window",
            "pid": 1234,
            "xwayland": false,
            "pinned": false,
            "fullscreen": 0,
            "fullscreenClient": 0,
            "grouped": [],
            "swallowing": null,
            "focusHistoryID": 0,
            "inhibitingIdle": false
        }))
        .unwrap()
    }

    #[test]
    fn workspace_basic() {
        let ws = workspace_from_hyprland(&make_hyprland_workspace(1, "1", "eDP-1"));
        assert_eq!(ws.id, 1);
        assert_eq!(ws.name, "1");
        assert_eq!(ws.output.as_deref(), Some("eDP-1"));
        assert!(!ws.is_special);
    }

    #[test]
    fn special_workspace_detected() {
        let ws = workspace_from_hyprland(&make_hyprland_special_workspace());
        assert!(ws.is_special);
        assert!(ws.id < 0);
        assert!(ws.name.starts_with("special:"));
    }

    #[test]
    fn output_from_monitor() {
        let o = output_from_hyprland(&make_hyprland_monitor());
        assert_eq!(o.name, "eDP-1");
        assert_eq!(o.width, 1920);
        assert_eq!(o.height, 1080);
        assert_eq!(o.refresh_hz, 60.0);
        assert!(o.focused);
        assert_eq!(o.scale, 1.0);
    }

    #[test]
    fn window_from_client() {
        let w = window_from_hyprland(&make_hyprland_client());
        assert_eq!(w.id, 42); // 0x2a = 42
        assert_eq!(w.title, "My Window");
        assert_eq!(w.app_id.as_deref(), Some("firefox"));
        assert_eq!(w.workspace_id, Some(1));
        assert!(!w.floating);
        assert!(!w.fullscreen);
    }

    #[test]
    fn parse_address_strips_prefix() {
        assert_eq!(parse_address("0x2a"), 42);
        assert_eq!(parse_address("2a"), 42);
        assert_eq!(parse_address("0x0"), 0);
    }
}
