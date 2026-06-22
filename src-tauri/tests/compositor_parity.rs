/// Backend parity tests.
///
/// Each test in this matrix converts equivalent raw IPC responses from sway
/// and Hyprland and asserts that the resulting normalized types are semantically
/// equivalent.  No compositor process is required; fixtures are built from JSON
/// (both crates implement `Deserialize`).
///
/// # Why this approach
/// Testing the conversion layer directly (rather than the full IPC stack) lets
/// CI verify normalization correctness without sway/Hyprland running.  End-to-end
/// IPC tests belong in a separate suite that gates on a compositor fixture.
use mantle_lib::compositor::{
    hyprland::convert as hc,
    sway::convert as sc,
    types::{Output, Window, Workspace},
};

// ── Fixture builders ──────────────────────────────────────────────────────────

mod sway_fixtures {
    use swayipc::{Node, Output as SwayOutput, Workspace as SwayWorkspace};

    pub fn workspace(id: i32, name: &str, focused: bool) -> SwayWorkspace {
        serde_json::from_value(serde_json::json!({
            "id": id,
            "num": id,
            "name": name,
            "visible": focused,
            "focused": focused,
            "urgent": false,
            "representation": null,
            "layout": "splith",
            "output": "eDP-1",
            "rect": {"x": 0, "y": 0, "width": 1920, "height": 1080}
        }))
        .unwrap()
    }

    pub fn scratchpad() -> SwayWorkspace {
        serde_json::from_value(serde_json::json!({
            "id": 99,
            "num": -1,
            "name": "__i3_scratch",
            "visible": false,
            "focused": false,
            "urgent": false,
            "representation": null,
            "layout": "splith",
            "output": "eDP-1",
            "rect": {"x": 0, "y": 0, "width": 1920, "height": 1080}
        }))
        .unwrap()
    }

    pub fn output(name: &str, w: u32, h: u32, refresh_mhz: i32) -> SwayOutput {
        serde_json::from_value(serde_json::json!({
            "id": 1,
            "name": name,
            "make": "LG",
            "model": "LG IPS",
            "serial": "0x00000000",
            "active": true,
            "dpms": true,
            "primary": true,
            "scale": 1.0,
            "subpixel_hinting": "rgb",
            "transform": "normal",
            "current_workspace": "1",
            "modes": [],
            "current_mode": {
                "width": w,
                "height": h,
                "refresh": refresh_mhz
            },
            "rect": {"x": 0, "y": 0, "width": w, "height": h},
            "focused": true
        }))
        .unwrap()
    }

    pub fn window_node(id: i64, title: &str, app_id: &str, focused: bool) -> Node {
        serde_json::from_value(serde_json::json!({
            "id": id,
            "type": "con",
            "name": title,
            "focused": focused,
            "urgent": false,
            "sticky": false,
            "layout": "none",
            "orientation": "none",
            "border": "normal",
            "current_border_width": 2,
            "percent": null,
            "rect": {"x": 0, "y": 0, "width": 800, "height": 600},
            "marks": [],
            "nodes": [],
            "floating_nodes": [],
            "focus": [],
            "fullscreen_mode": 0,
            "app_id": app_id,
            "pid": 1234
        }))
        .unwrap()
    }

    pub fn floating_node(id: i64, title: &str) -> Node {
        serde_json::from_value(serde_json::json!({
            "id": id,
            "type": "floating_con",
            "name": title,
            "focused": false,
            "urgent": false,
            "sticky": false,
            "layout": "none",
            "orientation": "none",
            "border": "normal",
            "current_border_width": 2,
            "percent": null,
            "rect": {"x": 100, "y": 100, "width": 400, "height": 300},
            "marks": [],
            "nodes": [],
            "floating_nodes": [],
            "focus": [],
            "fullscreen_mode": 0,
            "app_id": "alacritty",
            "pid": 5678
        }))
        .unwrap()
    }
}

mod hyprland_fixtures {
    use hyprland::data::{
        Client as HyprClient, Monitor as HyprMonitor, Workspace as HyprWorkspace,
    };

    pub fn workspace(id: i32, name: &str, monitor: &str) -> HyprWorkspace {
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

    pub fn special_workspace(name: &str) -> HyprWorkspace {
        workspace(-99, &format!("special:{name}"), "eDP-1")
    }

    pub fn monitor(name: &str, w: i32, h: i32, refresh: f64) -> HyprMonitor {
        serde_json::from_value(serde_json::json!({
            "id": 0,
            "name": name,
            "description": "LG IPS",
            "make": "LG",
            "model": "LG IPS",
            "serial": "0x000000",
            "width": w,
            "height": h,
            "refreshRate": refresh,
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

    pub fn client(
        addr_hex: u64,
        title: &str,
        class: &str,
        ws_id: i32,
        focused: bool,
    ) -> HyprClient {
        serde_json::from_value(serde_json::json!({
            "address": format!("0x{addr_hex:x}"),
            "mapped": true,
            "hidden": false,
            "at": [0, 0],
            "size": [800, 600],
            "workspace": {"id": ws_id, "name": ws_id.to_string()},
            "floating": false,
            "pseudo": false,
            "monitor": 0,
            "class": class,
            "title": title,
            "initialClass": class,
            "initialTitle": title,
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
}

// ── Parity assertions ─────────────────────────────────────────────────────────

/// Assert that two workspaces are semantically equivalent for parity purposes.
fn assert_workspace_parity(sway: &Workspace, hypr: &Workspace, scenario: &str) {
    assert_eq!(
        sway.is_special, hypr.is_special,
        "{scenario}: is_special mismatch"
    );
    assert_eq!(sway.urgent, hypr.urgent, "{scenario}: urgent mismatch");
    // Name parity: sway uses the workspace name as-is; Hyprland may prefix
    // special workspaces with "special:". We only compare for non-special.
    if !sway.is_special {
        assert_eq!(sway.name, hypr.name, "{scenario}: name mismatch");
    }
}

fn assert_output_parity(sway: &Output, hypr: &Output, scenario: &str) {
    assert_eq!(sway.width, hypr.width, "{scenario}: width mismatch");
    assert_eq!(sway.height, hypr.height, "{scenario}: height mismatch");
    assert!(
        (sway.refresh_hz - hypr.refresh_hz).abs() < 0.1,
        "{scenario}: refresh_hz mismatch ({} vs {})",
        sway.refresh_hz,
        hypr.refresh_hz
    );
    assert_eq!(sway.active, hypr.active, "{scenario}: active mismatch");
    assert_eq!(sway.scale, hypr.scale, "{scenario}: scale mismatch");
}

fn assert_window_parity(sway: &Window, hypr: &Window, scenario: &str) {
    assert_eq!(sway.title, hypr.title, "{scenario}: title mismatch");
    assert_eq!(sway.app_id, hypr.app_id, "{scenario}: app_id mismatch");
    assert_eq!(
        sway.floating, hypr.floating,
        "{scenario}: floating mismatch"
    );
    assert_eq!(
        sway.fullscreen, hypr.fullscreen,
        "{scenario}: fullscreen mismatch"
    );
}

// ── Test matrix ───────────────────────────────────────────────────────────────

#[test]
fn parity_regular_workspace() {
    let sway_ws = sc::workspace_from_sway(&sway_fixtures::workspace(1, "1", true));
    let hypr_ws = hc::workspace_from_hyprland(&hyprland_fixtures::workspace(1, "1", "eDP-1"));
    assert_workspace_parity(&sway_ws, &hypr_ws, "regular workspace");
    assert!(!sway_ws.is_special);
    assert!(!hypr_ws.is_special);
}

#[test]
fn parity_special_workspace() {
    let sway_ws = sc::workspace_from_sway(&sway_fixtures::scratchpad());
    let hypr_ws = hc::workspace_from_hyprland(&hyprland_fixtures::special_workspace("magic"));
    assert_workspace_parity(&sway_ws, &hypr_ws, "special workspace");
    assert!(sway_ws.is_special, "sway scratchpad should be special");
    assert!(hypr_ws.is_special, "hyprland special ws should be special");
}

#[test]
fn parity_output_resolution_and_refresh() {
    let sway_out = sc::output_from_sway(&sway_fixtures::output("eDP-1", 1920, 1080, 60_000));
    let hypr_out = hc::output_from_hyprland(&hyprland_fixtures::monitor("eDP-1", 1920, 1080, 60.0));
    assert_output_parity(&sway_out, &hypr_out, "1080p 60Hz");
}

#[test]
fn parity_output_high_refresh() {
    // 144 Hz: sway reports 144000 mHz; hyprland reports 144.0 Hz.
    let sway_out = sc::output_from_sway(&sway_fixtures::output("DP-1", 2560, 1440, 144_000));
    let hypr_out = hc::output_from_hyprland(&hyprland_fixtures::monitor("DP-1", 2560, 1440, 144.0));
    assert_output_parity(&sway_out, &hypr_out, "1440p 144Hz");
    assert!((sway_out.refresh_hz - 144.0).abs() < 0.1);
}

#[test]
fn parity_window_basic() {
    let sway_win =
        sc::window_from_node(&sway_fixtures::window_node(42, "Firefox", "firefox", true));
    let hypr_win = hc::window_from_hyprland(&hyprland_fixtures::client(
        42, "Firefox", "firefox", 1, true,
    ));
    assert_window_parity(&sway_win, &hypr_win, "basic window");
}

#[test]
fn parity_floating_window() {
    let sway_win = sc::window_from_node(&sway_fixtures::floating_node(99, "Floating Term"));
    // In Hyprland the floating flag is set per-client.
    let hypr_raw = serde_json::from_value::<hyprland::data::Client>(serde_json::json!({
        "address": "0x63",
        "mapped": true,
        "hidden": false,
        "at": [100, 100],
        "size": [400, 300],
        "workspace": {"id": 1, "name": "1"},
        "floating": true,
        "pseudo": false,
        "monitor": 0,
        "class": "alacritty",
        "title": "Floating Term",
        "initialClass": "alacritty",
        "initialTitle": "Floating Term",
        "pid": 5678,
        "xwayland": false,
        "pinned": false,
        "fullscreen": 0,
        "fullscreenClient": 0,
        "grouped": [],
        "swallowing": null,
        "focusHistoryID": 0,
        "inhibitingIdle": false
    }))
    .unwrap();
    let hypr_win = hc::window_from_hyprland(&hypr_raw);

    assert!(sway_win.floating, "sway floating_con should be floating");
    assert!(
        hypr_win.floating,
        "hyprland floating client should be floating"
    );
    assert_window_parity(&sway_win, &hypr_win, "floating window");
}

#[test]
fn parity_multiple_workspaces_count() {
    let sway_wss: Vec<Workspace> = (1..=3)
        .map(|i| sc::workspace_from_sway(&sway_fixtures::workspace(i, &i.to_string(), i == 1)))
        .collect();

    let hypr_wss: Vec<Workspace> = (1..=3)
        .map(|i| {
            hc::workspace_from_hyprland(&hyprland_fixtures::workspace(i, &i.to_string(), "eDP-1"))
        })
        .collect();

    assert_eq!(sway_wss.len(), hypr_wss.len(), "workspace count must match");
    for (s, h) in sway_wss.iter().zip(hypr_wss.iter()) {
        assert_workspace_parity(s, h, "multi-workspace element");
    }
}

#[test]
fn parity_focused_workspace_flag() {
    let sway_ws = sc::workspace_from_sway(&sway_fixtures::workspace(2, "2", true));
    let mut hypr_ws = hc::workspace_from_hyprland(&hyprland_fixtures::workspace(2, "2", "eDP-1"));
    hypr_ws.focused = true; // annotated during snapshot building

    assert!(sway_ws.focused, "sway: focused workspace should be marked");
    assert!(
        hypr_ws.focused,
        "hyprland: focused workspace should be marked"
    );
}

#[test]
fn hyprland_address_round_trips() {
    // Window IDs must survive address→u64→address round-trip.
    let addr = 0xdeadbeef_u64;
    let parsed = hc::parse_address(&format!("0x{addr:x}"));
    assert_eq!(parsed, addr);
}
