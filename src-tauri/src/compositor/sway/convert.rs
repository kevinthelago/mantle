/// Normalization layer: swayipc types → compositor types.
///
/// All conversion is pure (no I/O) to keep it testable in isolation.
use swayipc::{EventType, Node, NodeType, Output as SwayOutput, Workspace as SwayWorkspace};

use crate::compositor::{
    event::CompositorEvent,
    types::{Output, Window, Workspace},
};

// ── Workspace ─────────────────────────────────────────────────────────────────

pub fn workspace_from_sway(ws: &SwayWorkspace) -> Workspace {
    Workspace {
        id: ws.id,
        name: ws.name.clone(),
        output: Some(ws.output.clone()),
        focused: ws.focused,
        urgent: ws.urgent,
        // sway represents the scratchpad as "__i3_scratch"
        is_special: ws.name == "__i3_scratch",
        window_ids: Vec::new(), // populated separately from the tree
    }
}

// ── Output ────────────────────────────────────────────────────────────────────

pub fn output_from_sway(o: &SwayOutput) -> Output {
    let (width, height, refresh_hz) = o
        .current_mode
        .as_ref()
        .map(|m| {
            (
                m.width as u32,
                m.height as u32,
                // sway reports refresh in millihertz (e.g. 60000 = 60 Hz)
                m.refresh as f64 / 1000.0,
            )
        })
        .unwrap_or((0, 0, 0.0));

    Output {
        name: o.name.clone(),
        make: Some(o.make.clone()).filter(|s| !s.is_empty()),
        model: Some(o.model.clone()).filter(|s| !s.is_empty()),
        width,
        height,
        refresh_hz,
        x: o.rect.x,
        y: o.rect.y,
        active: o.active,
        scale: o.scale.unwrap_or(1.0),
        // swayipc < 3.x did not expose `focused` on Output; derive from
        // the focused workspace's output name when building the snapshot.
        focused: false,
    }
}

// ── Window (Node) ─────────────────────────────────────────────────────────────

/// Convert a sway container `Node` to a `Window`.
///
/// Expects `node` to be a leaf container (actual window), not a split container.
pub fn window_from_node(node: &Node) -> Window {
    let app_id = node
        .app_id
        .clone()
        .or_else(|| {
            node.window_properties
                .as_ref()
                .and_then(|p| p.class.clone())
        });

    let title = node
        .name
        .clone()
        .or_else(|| {
            node.window_properties
                .as_ref()
                .and_then(|p| p.title.clone())
        })
        .unwrap_or_default();

    let fullscreen = node
        .fullscreen_mode
        .map(|m| m != 0)
        .unwrap_or(false);

    Window {
        id: node.id as u64,
        title,
        app_id,
        workspace_id: None, // filled in by the snapshot builder
        output: None,       // filled in by the snapshot builder
        focused: node.focused,
        floating: node.node_type == NodeType::FloatingCon,
        fullscreen,
        pid: node.pid.map(|p| p as u32),
    }
}

/// Recursively collect all leaf windows from a sway tree.
pub fn collect_windows(node: &Node, out: &mut Vec<Window>) {
    let is_leaf = node.nodes.is_empty() && node.floating_nodes.is_empty();
    let is_window = matches!(
        node.node_type,
        NodeType::Con | NodeType::FloatingCon
    ) && is_leaf;

    if is_window {
        out.push(window_from_node(node));
    }

    for child in node.nodes.iter().chain(node.floating_nodes.iter()) {
        collect_windows(child, out);
    }
}

// ── Events ────────────────────────────────────────────────────────────────────

pub fn event_from_sway(raw: swayipc::Event) -> Option<CompositorEvent> {
    match raw {
        swayipc::Event::Workspace(e) => match e.change {
            swayipc::WorkspaceChange::Focus => {
                let ws = e.current.as_ref()?;
                Some(CompositorEvent::WorkspaceChanged {
                    workspace: node_to_workspace(ws),
                })
            }
            swayipc::WorkspaceChange::Init => {
                let ws = e.current.as_ref()?;
                Some(CompositorEvent::WorkspaceAdded {
                    workspace: node_to_workspace(ws),
                })
            }
            swayipc::WorkspaceChange::Empty => {
                let ws = e.current.as_ref()?;
                Some(CompositorEvent::WorkspaceRemoved {
                    workspace_id: ws.id as i32,
                })
            }
            _ => None,
        },

        swayipc::Event::Window(e) => match e.change {
            swayipc::WindowChange::New => Some(CompositorEvent::WindowOpened {
                window: window_from_node(&e.container),
            }),
            swayipc::WindowChange::Close => Some(CompositorEvent::WindowClosed {
                window_id: e.container.id as u64,
            }),
            swayipc::WindowChange::Focus => Some(CompositorEvent::WindowFocused {
                window: window_from_node(&e.container),
            }),
            swayipc::WindowChange::Title => Some(CompositorEvent::WindowTitleChanged {
                window: window_from_node(&e.container),
            }),
            swayipc::WindowChange::Move => Some(CompositorEvent::WindowMoved {
                window: window_from_node(&e.container),
            }),
            swayipc::WindowChange::Floating => {
                // Re-emit as a window moved (state changed but workspace unchanged)
                Some(CompositorEvent::WindowMoved {
                    window: window_from_node(&e.container),
                })
            }
            _ => None,
        },

        swayipc::Event::Output(_) => {
            // Output change events don't carry the full output struct;
            // callers should re-snapshot outputs on receiving this.
            None
        }

        swayipc::Event::Shutdown(_) => Some(CompositorEvent::Disconnected),

        _ => None,
    }
}

/// Convert a workspace `Node` (from workspace-change events) into a `Workspace`.
fn node_to_workspace(node: &Node) -> Workspace {
    let name = node.name.clone().unwrap_or_default();
    Workspace {
        id: node.id as i32,
        is_special: name == "__i3_scratch",
        name,
        output: None,
        focused: node.focused,
        urgent: node.urgent,
        window_ids: Vec::new(),
    }
}

/// The event types the sway IPC subscription should request.
pub const SWAY_EVENT_TYPES: &[EventType] = &[
    EventType::Workspace,
    EventType::Window,
    EventType::Output,
    EventType::Shutdown,
];

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sway_workspace() -> SwayWorkspace {
        serde_json::from_value(serde_json::json!({
            "id": 1,
            "num": 1,
            "name": "1",
            "visible": true,
            "focused": true,
            "urgent": false,
            "representation": null,
            "layout": "splith",
            "output": "eDP-1",
            "rect": {"x": 0, "y": 0, "width": 1920, "height": 1080}
        }))
        .unwrap()
    }

    fn make_sway_scratchpad() -> SwayWorkspace {
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

    fn make_sway_output() -> SwayOutput {
        serde_json::from_value(serde_json::json!({
            "id": 1,
            "name": "eDP-1",
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
                "width": 1920,
                "height": 1080,
                "refresh": 60000
            },
            "rect": {"x": 0, "y": 0, "width": 1920, "height": 1080},
            "focused": true
        }))
        .unwrap()
    }

    fn make_sway_node_window() -> Node {
        serde_json::from_value(serde_json::json!({
            "id": 42,
            "type": "con",
            "name": "My Window",
            "focused": true,
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
            "app_id": "firefox",
            "pid": 1234
        }))
        .unwrap()
    }

    #[test]
    fn workspace_basic_fields() {
        let ws = workspace_from_sway(&make_sway_workspace());
        assert_eq!(ws.id, 1);
        assert_eq!(ws.name, "1");
        assert_eq!(ws.output.as_deref(), Some("eDP-1"));
        assert!(ws.focused);
        assert!(!ws.is_special);
    }

    #[test]
    fn scratchpad_is_special() {
        let ws = workspace_from_sway(&make_sway_scratchpad());
        assert!(ws.is_special);
        assert_eq!(ws.name, "__i3_scratch");
    }

    #[test]
    fn output_refresh_converts_from_millihertz() {
        let o = output_from_sway(&make_sway_output());
        assert_eq!(o.refresh_hz, 60.0);
        assert_eq!(o.width, 1920);
        assert_eq!(o.height, 1080);
        assert_eq!(o.name, "eDP-1");
    }

    #[test]
    fn window_from_node_basic() {
        let w = window_from_node(&make_sway_node_window());
        assert_eq!(w.id, 42);
        assert_eq!(w.title, "My Window");
        assert_eq!(w.app_id.as_deref(), Some("firefox"));
        assert!(w.focused);
        assert!(!w.fullscreen);
    }
}
