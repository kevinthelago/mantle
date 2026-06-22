pub mod convert;

use std::sync::Arc;

use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::{debug, warn};

use crate::compositor::{
    action::CompositorAction,
    backend::CompositorBackend,
    error::CompositorError,
    event::CompositorEvent,
    supervisor::Supervisor,
    types::{CompositorSnapshot, Output, Workspace},
};

use convert::{collect_windows, event_from_sway, output_from_sway, workspace_from_sway};

/// sway backend.
///
/// Uses the synchronous `swayipc` API bridged to tokio via `spawn_blocking`:
/// - Snapshot queries run in a blocking thread pool.
/// - The event subscription runs in a dedicated blocking thread that feeds a
///   channel; the supervisor drains the channel and broadcasts to subscribers.
pub struct SwayBackend {
    /// Shared connection used for IPC queries (get_workspaces, run_command, …).
    /// Guarded by a mutex because `Connection` is not `Send + Sync` by itself.
    query: Arc<Mutex<swayipc::Connection>>,
    supervisor: Arc<Supervisor>,
    /// Keep the supervisor loop alive for the lifetime of the backend.
    _supervisor_handle: Arc<JoinHandle<()>>,
}

impl SwayBackend {
    /// Connect to sway and start the event supervisor.
    pub async fn connect() -> Result<Self, CompositorError> {
        let conn = tokio::task::spawn_blocking(swayipc::Connection::new)
            .await
            .map_err(|_| CompositorError::TaskPanicked)??;

        let query = Arc::new(Mutex::new(conn));
        let supervisor = Arc::new(Supervisor::new());

        let handle =
            supervisor.spawn(move || Box::pin(async move { connect_event_stream().await }));

        Ok(Self {
            query,
            supervisor,
            _supervisor_handle: Arc::new(handle),
        })
    }
}

/// Open a new connection and return a stream of normalized compositor events.
/// Returns `None` on connection failure (supervisor will backoff and retry).
async fn connect_event_stream() -> Option<impl futures::Stream<Item = CompositorEvent> + Unpin> {
    use convert::SWAY_EVENT_TYPES;
    use tokio::sync::mpsc;
    use tokio_stream::wrappers::ReceiverStream;

    let (tx, rx) = mpsc::channel::<CompositorEvent>(256);

    tokio::task::spawn_blocking(move || {
        let conn = match swayipc::Connection::new() {
            Ok(c) => c,
            Err(e) => {
                warn!("sway: event connection failed: {e}");
                return;
            }
        };

        let events = match conn.subscribe(SWAY_EVENT_TYPES) {
            Ok(s) => s,
            Err(e) => {
                warn!("sway: subscribe failed: {e}");
                return;
            }
        };

        for raw in events {
            match raw {
                Ok(event) => {
                    if let Some(normalized) = event_from_sway(event) {
                        if tx.blocking_send(normalized).is_err() {
                            break;
                        }
                    }
                }
                Err(e) => {
                    warn!("sway: event error: {e}");
                    break;
                }
            }
        }
        debug!("sway: event loop exited");
    });

    Some(ReceiverStream::new(rx))
}

#[async_trait]
impl CompositorBackend for SwayBackend {
    async fn snapshot(&self) -> Result<CompositorSnapshot, CompositorError> {
        let mut conn = self.query.lock().await;

        // Run all three queries while holding the lock (one connection at a time).
        let (sway_workspaces, sway_outputs, tree) = tokio::task::block_in_place(|| {
            let ws = conn.get_workspaces()?;
            let out = conn.get_outputs()?;
            let tree = conn.get_tree()?;
            Ok::<_, swayipc::Error>((ws, out, tree))
        })?;

        // --- Workspaces ---
        let mut workspaces: Vec<Workspace> =
            sway_workspaces.iter().map(workspace_from_sway).collect();

        // --- Outputs (mark which one has focus via the focused workspace) ---
        let focused_ws_output = sway_workspaces
            .iter()
            .find(|w| w.focused)
            .map(|w| w.output.as_str());

        let outputs: Vec<Output> = sway_outputs
            .iter()
            .map(|o| {
                let mut out = output_from_sway(o);
                out.focused = Some(o.name.as_str()) == focused_ws_output;
                out
            })
            .collect();

        // --- Windows ---
        let mut all_windows = Vec::new();
        collect_windows(&tree, &mut all_windows);

        // Walk the tree once more to annotate each window with its workspace
        // and output. We find the workspace ancestor by walking sway's tree path.
        annotate_windows_from_tree(&tree, &mut all_windows, &sway_workspaces, &sway_outputs);

        // --- Wire window_ids into each workspace ---
        for ws in &mut workspaces {
            ws.window_ids = all_windows
                .iter()
                .filter(|w| w.workspace_id == Some(ws.id))
                .map(|w| w.id)
                .collect();
        }

        let focused_window = all_windows.iter().find(|w| w.focused).cloned();
        let active_workspace_id = sway_workspaces.iter().find(|w| w.focused).map(|w| w.id);

        Ok(CompositorSnapshot {
            workspaces,
            windows: all_windows,
            outputs,
            focused_window,
            active_workspace_id,
        })
    }

    fn subscribe(&self) -> BoxStream<'static, CompositorEvent> {
        self.supervisor.subscribe()
    }

    async fn dispatch(&self, action: CompositorAction) -> Result<(), CompositorError> {
        let cmd = sway_command(action);
        let mut conn = self.query.lock().await;
        tokio::task::block_in_place(|| {
            conn.run_command(&cmd)?;
            Ok(())
        })
    }

    fn name(&self) -> &'static str {
        "sway"
    }
}

/// Build the sway IPC command string for a `CompositorAction`.
fn sway_command(action: CompositorAction) -> String {
    match action {
        CompositorAction::FocusWorkspace { id } => format!("workspace {id}"),
        CompositorAction::FocusWindow { id } => format!("[con_id={id}] focus"),
        CompositorAction::MoveWindowToWorkspace {
            window_id,
            workspace_id,
        } => format!("[con_id={window_id}] move container to workspace {workspace_id}"),
        CompositorAction::CloseWindow { id } => format!("[con_id={id}] kill"),
        CompositorAction::ToggleFloating { id } => format!("[con_id={id}] floating toggle"),
        CompositorAction::ToggleFullscreen { id } => format!("[con_id={id}] fullscreen toggle"),
        CompositorAction::Exec { command } => format!("exec {command}"),
        CompositorAction::SetLayout { layout } => format!("layout {layout}"),
    }
}

/// Walk the sway tree, annotating each already-collected `Window` with the
/// workspace ID and output name found along its ancestry path.
fn annotate_windows_from_tree(
    root: &swayipc::Node,
    windows: &mut Vec<crate::compositor::types::Window>,
    workspaces: &[swayipc::Workspace],
    outputs: &[swayipc::Output],
) {
    // Build a map from container id → (workspace_id, output_name).
    let mut id_to_ctx: std::collections::HashMap<i64, (i32, String)> =
        std::collections::HashMap::new();
    walk_for_context(root, None, None, &mut id_to_ctx);

    for w in windows.iter_mut() {
        if let Some((ws_id, output_name)) = id_to_ctx.get(&(w.id as i64)) {
            w.workspace_id = Some(*ws_id);
            w.output = Some(output_name.clone());
        }
    }
}

fn walk_for_context(
    node: &swayipc::Node,
    current_ws: Option<i32>,
    current_output: Option<&str>,
    out: &mut std::collections::HashMap<i64, (i32, String)>,
) {
    use swayipc::NodeType;

    let ws = if node.node_type == NodeType::Workspace {
        Some(node.id as i32)
    } else {
        current_ws
    };

    let output_name: Option<String> = if node.node_type == NodeType::Output {
        node.name.clone()
    } else {
        current_output.map(|s| s.to_owned())
    };

    let is_leaf = node.nodes.is_empty() && node.floating_nodes.is_empty();
    if matches!(node.node_type, NodeType::Con | NodeType::FloatingCon) && is_leaf {
        if let (Some(ws_id), Some(out_name)) = (ws, &output_name) {
            out.insert(node.id, (ws_id, out_name.clone()));
        }
    }

    for child in node.nodes.iter().chain(node.floating_nodes.iter()) {
        walk_for_context(child, ws, output_name.as_deref(), out);
    }
}
