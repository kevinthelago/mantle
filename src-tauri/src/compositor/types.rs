use serde::{Deserialize, Serialize};
use specta::Type;

/// A compositor workspace.
///
/// # Normalization notes
/// - **Sway**: workspaces have positive numeric `id`. The scratchpad appears as
///   `__i3_scratch`; it is normalized to `is_special = true`.
/// - **Hyprland**: regular workspaces have positive `id`; special workspaces
///   (e.g. `special:magic`) carry a negative `id` assigned by the compositor.
///   Both are represented uniformly here with `is_special = true`.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Workspace {
    /// Compositor-native ID. Negative on Hyprland for special workspaces.
    pub id: i32,
    pub name: String,
    /// Output (monitor) this workspace lives on, if known.
    pub output: Option<String>,
    pub focused: bool,
    pub urgent: bool,
    /// `true` for Hyprland special workspaces and the sway scratchpad.
    pub is_special: bool,
    /// IDs of windows currently in this workspace.
    pub window_ids: Vec<u64>,
}

/// A compositor window (Wayland surface / X11 window).
///
/// # Normalization notes
/// - **Sway**: `id` is the sway container ID cast to `u64`.
/// - **Hyprland**: `id` is the hex client address parsed to `u64` (strip `0x`, parse base-16).
/// - `app_id`: Wayland `app_id` when available; falls back to X11 window class.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Window {
    /// Stable per-backend window identifier. See normalization notes.
    pub id: u64,
    pub title: String,
    /// Wayland app_id or X11 class, whichever is present.
    pub app_id: Option<String>,
    pub workspace_id: Option<i32>,
    pub output: Option<String>,
    pub focused: bool,
    pub floating: bool,
    pub fullscreen: bool,
    pub pid: Option<u32>,
}

/// A physical output (monitor) as reported by the compositor.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Output {
    pub name: String,
    pub make: Option<String>,
    pub model: Option<String>,
    /// Pixel width of the active mode.
    pub width: u32,
    /// Pixel height of the active mode.
    pub height: u32,
    /// Refresh rate in Hz (e.g. 144.0).
    pub refresh_hz: f64,
    /// Position on the global compositor canvas.
    pub x: i32,
    pub y: i32,
    /// Whether the output is currently enabled.
    pub active: bool,
    pub scale: f64,
    /// Whether this output is the one currently receiving keyboard focus.
    pub focused: bool,
}

/// A point-in-time snapshot of compositor state.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CompositorSnapshot {
    pub workspaces: Vec<Workspace>,
    pub windows: Vec<Window>,
    pub outputs: Vec<Output>,
    /// The currently-focused window, if any.
    pub focused_window: Option<Window>,
    /// ID of the workspace that currently has focus.
    pub active_workspace_id: Option<i32>,
}
