use serde::{Deserialize, Serialize};
use specta::Type;

use super::types::{Output, Window, Workspace};

/// Normalized event emitted by either backend.
///
/// The `type` field in the serialized JSON carries the variant tag so the
/// frontend can switch on a single field (`{"type":"window_focused",...}`).
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CompositorEvent {
    /// The active workspace changed.
    WorkspaceChanged { workspace: Workspace },
    /// A new workspace was created.
    WorkspaceAdded { workspace: Workspace },
    /// A workspace was destroyed (last window closed).
    WorkspaceRemoved { workspace_id: i32 },
    /// Keyboard focus moved to a different window.
    WindowFocused { window: Window },
    /// A new window was mapped.
    WindowOpened { window: Window },
    /// A window was unmapped/closed.
    WindowClosed { window_id: u64 },
    /// A window was moved to a different workspace.
    WindowMoved { window: Window },
    /// A window's title changed.
    WindowTitleChanged { window: Window },
    /// An output was connected.
    OutputAdded { output: Output },
    /// An output was disconnected.
    OutputRemoved { name: String },
    /// The compositor connection was lost.
    Disconnected,
    /// The supervisor is waiting before the next reconnect attempt.
    Reconnecting { attempt: u32 },
    /// A (re)connection succeeded.
    Connected,
}
