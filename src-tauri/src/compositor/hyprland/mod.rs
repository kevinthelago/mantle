pub mod convert;

use std::sync::Arc;

use async_trait::async_trait;
use futures::stream::BoxStream;
use hyprland::data::{Client, Clients, Monitor, Monitors, Workspace, Workspaces};
use hyprland::dispatch::{Dispatch, DispatchType, WindowIdentifier, WorkspaceIdentifierWithSpecial};
use hyprland::event_listener::EventListener;
use hyprland::shared::{HyprData, HyprDataActiveOptional, HyprDataVec};
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;
use tracing::{debug, warn};

use crate::compositor::{
    action::CompositorAction,
    backend::CompositorBackend,
    error::CompositorError,
    event::CompositorEvent,
    supervisor::Supervisor,
    types::{CompositorSnapshot, Output, Window as NWindow, Workspace as NWorkspace},
};

use convert::{output_from_hyprland, parse_address, window_from_hyprland, workspace_from_hyprland};

/// Hyprland backend.
///
/// Uses the `hyprland` crate's async data queries and event listener. The event
/// listener runs in a dedicated tokio task; events are forwarded through the
/// supervisor's broadcast channel.
pub struct HyprlandBackend {
    supervisor: Arc<Supervisor>,
    _supervisor_handle: Arc<JoinHandle<()>>,
}

impl HyprlandBackend {
    /// Connect to Hyprland and start the event supervisor.
    pub async fn connect() -> Result<Self, CompositorError> {
        let supervisor = Arc::new(Supervisor::new());

        let handle = supervisor.spawn(|| Box::pin(async move { connect_event_stream().await }));

        Ok(Self {
            supervisor,
            _supervisor_handle: Arc::new(handle),
        })
    }
}

/// Build a Hyprland event stream and return it.
/// Returns `None` on connection failure.
async fn connect_event_stream() -> Option<impl futures::Stream<Item = CompositorEvent> + Unpin> {
    use tokio_stream::wrappers::ReceiverStream;

    let (tx, rx) = mpsc::channel::<CompositorEvent>(256);

    let tx_workspace_changed = tx.clone();
    let tx_workspace_added = tx.clone();
    let tx_workspace_deleted = tx.clone();
    let tx_window_focus = tx.clone();
    let tx_window_opened = tx.clone();
    let tx_window_closed = tx.clone();
    let tx_window_moved = tx.clone();
    let tx_window_title = tx.clone();
    let tx_monitor_added = tx.clone();
    let tx_monitor_removed = tx.clone();

    let mut listener = EventListener::new();

    listener.add_workspace_change_handler(move |name| {
        // Re-fetch the workspace to build a full struct; fall back to a
        // minimal version if the IPC call fails.
        let event = CompositorEvent::WorkspaceChanged {
            workspace: NWorkspace {
                id: 0,
                name: name.to_string(),
                output: None,
                focused: true,
                urgent: false,
                is_special: name.to_string().starts_with("special:"),
                window_ids: Vec::new(),
            },
        };
        let _ = tx_workspace_changed.try_send(event);
    });

    listener.add_workspace_added_handler(move |name| {
        let event = CompositorEvent::WorkspaceAdded {
            workspace: NWorkspace {
                id: 0,
                name: name.to_string(),
                output: None,
                focused: false,
                urgent: false,
                is_special: name.to_string().starts_with("special:"),
                window_ids: Vec::new(),
            },
        };
        let _ = tx_workspace_added.try_send(event);
    });

    listener.add_workspace_deleted_handler(move |name| {
        // Hyprland only gives the name; emit ID=0 as sentinel (frontend re-syncs via snapshot).
        let event = CompositorEvent::WorkspaceRemoved { workspace_id: 0 };
        let _ = tx_workspace_deleted.try_send(event);
    });

    listener.add_active_window_changed_handler(move |opt| {
        if let Some(data) = opt {
            let event = CompositorEvent::WindowFocused {
                window: NWindow {
                    id: parse_address(&data.address.to_string()),
                    title: data.title.clone(),
                    app_id: Some(data.class.clone()),
                    workspace_id: None,
                    output: None,
                    focused: true,
                    floating: false,
                    fullscreen: false,
                    pid: None,
                },
            };
            let _ = tx_window_focus.try_send(event);
        }
    });

    listener.add_window_opened_handler(move |data| {
        let is_special = data.workspace_name.starts_with("special:");
        let event = CompositorEvent::WindowOpened {
            window: NWindow {
                id: parse_address(&data.window_address.to_string()),
                title: data.window_title.clone(),
                app_id: Some(data.window_class.clone()),
                workspace_id: None,
                output: None,
                focused: false,
                floating: false,
                fullscreen: false,
                pid: None,
            },
        };
        let _ = tx_window_opened.try_send(event);
    });

    listener.add_window_closed_handler(move |addr| {
        let event = CompositorEvent::WindowClosed {
            window_id: parse_address(&addr.to_string()),
        };
        let _ = tx_window_closed.try_send(event);
    });

    listener.add_window_moved_handler(move |data| {
        let event = CompositorEvent::WindowMoved {
            window: NWindow {
                id: parse_address(&data.window_address.to_string()),
                title: String::new(),
                app_id: None,
                workspace_id: None,
                output: None,
                focused: false,
                floating: false,
                fullscreen: false,
                pid: None,
            },
        };
        let _ = tx_window_moved.try_send(event);
    });

    listener.add_window_title_changed_handler(move |addr| {
        let event = CompositorEvent::WindowTitleChanged {
            window: NWindow {
                id: parse_address(&addr.to_string()),
                title: String::new(),
                app_id: None,
                workspace_id: None,
                output: None,
                focused: false,
                floating: false,
                fullscreen: false,
                pid: None,
            },
        };
        let _ = tx_window_title.try_send(event);
    });

    listener.add_monitor_added_handler(move |name| {
        let event = CompositorEvent::OutputAdded {
            output: Output {
                name: name.clone(),
                make: None,
                model: None,
                width: 0,
                height: 0,
                refresh_hz: 0.0,
                x: 0,
                y: 0,
                active: true,
                scale: 1.0,
                focused: false,
            },
        };
        let _ = tx_monitor_added.try_send(event);
    });

    listener.add_monitor_removed_handler(move |name| {
        let event = CompositorEvent::OutputRemoved { name: name.clone() };
        let _ = tx_monitor_removed.try_send(event);
    });

    tokio::task::spawn_blocking(move || {
        if let Err(e) = listener.start_listener() {
            warn!("hyprland: event listener exited: {e}");
        }
        debug!("hyprland: event loop exited");
    });

    Some(tokio_stream::wrappers::ReceiverStream::new(rx))
}

#[async_trait]
impl CompositorBackend for HyprlandBackend {
    async fn snapshot(&self) -> Result<CompositorSnapshot, CompositorError> {
        // Fetch everything concurrently.
        let (workspaces_raw, clients_raw, monitors_raw) = tokio::try_join!(
            async { Workspaces::get_async().await.map_err(|e| CompositorError::HyprlandIpc(e.to_string())) },
            async { Clients::get_async().await.map_err(|e| CompositorError::HyprlandIpc(e.to_string())) },
            async { Monitors::get_async().await.map_err(|e| CompositorError::HyprlandIpc(e.to_string())) },
        )?;

        let active_client = Client::get_active_async()
            .await
            .ok()
            .flatten();

        // --- Outputs ---
        let outputs: Vec<Output> = monitors_raw.iter().map(output_from_hyprland).collect();

        // Find the focused workspace ID from the focused monitor.
        let focused_monitor = monitors_raw.iter().find(|m| m.focused);
        let active_workspace_id = focused_monitor.map(|m| m.active_workspace.id);

        // Build a monitor_id → output name map for window annotation.
        let monitor_id_to_name: std::collections::HashMap<u8, String> = monitors_raw
            .iter()
            .map(|m| (m.id, m.name.clone()))
            .collect();

        // Also include special workspaces (one per monitor if active).
        let mut workspaces: Vec<NWorkspace> = workspaces_raw
            .iter()
            .map(|ws| {
                let mut nws = workspace_from_hyprland(ws);
                nws.focused = Some(ws.id) == active_workspace_id;
                nws
            })
            .collect();

        // Inject special workspaces from monitor metadata.
        for monitor in &monitors_raw {
            let sp = &monitor.special_workspace;
            if sp.id != 0 {
                workspaces.push(NWorkspace {
                    id: sp.id,
                    name: sp.name.clone(),
                    output: Some(monitor.name.clone()),
                    focused: false,
                    urgent: false,
                    is_special: true,
                    window_ids: Vec::new(),
                });
            }
        }

        // --- Windows ---
        let active_addr = active_client.as_ref().map(|c| c.address.to_string());
        let mut windows: Vec<NWindow> = clients_raw
            .iter()
            .map(|c| {
                let mut w = window_from_hyprland(c);
                w.output = monitor_id_to_name.get(&(c.monitor as u8)).cloned();
                w.focused = active_addr.as_deref() == Some(&c.address.to_string());
                w
            })
            .collect();

        // Wire window_ids into workspaces.
        for ws in &mut workspaces {
            ws.window_ids = windows
                .iter()
                .filter(|w| w.workspace_id == Some(ws.id))
                .map(|w| w.id)
                .collect();
        }

        let focused_window = windows.iter().find(|w| w.focused).cloned();

        Ok(CompositorSnapshot {
            workspaces,
            windows,
            outputs,
            focused_window,
            active_workspace_id,
        })
    }

    fn subscribe(&self) -> BoxStream<'static, CompositorEvent> {
        self.supervisor.subscribe()
    }

    async fn dispatch(&self, action: CompositorAction) -> Result<(), CompositorError> {
        match action {
            CompositorAction::FocusWorkspace { id } => {
                Dispatch::call_async(DispatchType::Workspace(
                    WorkspaceIdentifierWithSpecial::Id(id),
                ))
                .await
                .map_err(|e| CompositorError::HyprlandIpc(e.to_string()))
            }
            CompositorAction::FocusWindow { id } => {
                let addr = format!("0x{id:x}");
                Dispatch::call_async(DispatchType::FocusWindow(WindowIdentifier::Address(
                    addr.into(),
                )))
                .await
                .map_err(|e| CompositorError::HyprlandIpc(e.to_string()))
            }
            CompositorAction::MoveWindowToWorkspace {
                window_id,
                workspace_id,
            } => {
                let addr = format!("0x{window_id:x}");
                Dispatch::call_async(DispatchType::MoveToWorkspaceSilent(
                    WorkspaceIdentifierWithSpecial::Id(workspace_id),
                    Some(WindowIdentifier::Address(addr.into())),
                ))
                .await
                .map_err(|e| CompositorError::HyprlandIpc(e.to_string()))
            }
            CompositorAction::CloseWindow { id } => {
                let addr = format!("0x{id:x}");
                Dispatch::call_async(DispatchType::CloseWindow(WindowIdentifier::Address(
                    addr.into(),
                )))
                .await
                .map_err(|e| CompositorError::HyprlandIpc(e.to_string()))
            }
            CompositorAction::ToggleFloating { id } => {
                let addr = format!("0x{id:x}");
                Dispatch::call_async(DispatchType::ToggleFloating(Some(
                    WindowIdentifier::Address(addr.into()),
                )))
                .await
                .map_err(|e| CompositorError::HyprlandIpc(e.to_string()))
            }
            CompositorAction::ToggleFullscreen { id } => {
                Dispatch::call_async(DispatchType::FullscreenWindow(
                    hyprland::dispatch::FullscreenType::Real,
                ))
                .await
                .map_err(|e| CompositorError::HyprlandIpc(e.to_string()))
            }
            CompositorAction::Exec { command } => {
                Dispatch::call_async(DispatchType::Exec(command))
                    .await
                    .map_err(|e| CompositorError::HyprlandIpc(e.to_string()))
            }
            CompositorAction::SetLayout { .. } => {
                // Hyprland does not expose a generic layout switch via IPC; no-op.
                Ok(())
            }
        }
    }

    fn name(&self) -> &'static str {
        "hyprland"
    }
}
