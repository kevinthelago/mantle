use serde::{Deserialize, Serialize};
use specta::Type;

/// An action the frontend can request the compositor to perform.
///
/// Serialized with a `type` tag so the Tauri command payload is self-describing.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CompositorAction {
    /// Switch focus to the workspace with the given ID.
    FocusWorkspace { id: i32 },
    /// Raise and focus a window by its normalized ID.
    FocusWindow { id: u64 },
    /// Move a window to a different workspace.
    MoveWindowToWorkspace { window_id: u64, workspace_id: i32 },
    /// Close (kill) a window.
    CloseWindow { id: u64 },
    /// Toggle floating state on a window.
    ToggleFloating { id: u64 },
    /// Toggle fullscreen state on a window.
    ToggleFullscreen { id: u64 },
    /// Spawn a process via the compositor.
    Exec { command: String },
    /// Change the layout mode of the focused container (sway: "splith"/"splitv"/etc.).
    SetLayout { layout: String },
}
