use async_trait::async_trait;
use futures::stream::BoxStream;

use super::{
    action::CompositorAction,
    error::CompositorError,
    event::CompositorEvent,
    types::CompositorSnapshot,
};

/// Abstraction over sway and Hyprland.
///
/// Both backends expose the same three operations so no compositor-specific
/// code leaks into the rest of the application.
///
/// # Threading
/// Implementations must be `Send + Sync` so they can live in Tauri's managed
/// state and be called from multiple command handlers concurrently.
#[async_trait]
pub trait CompositorBackend: Send + Sync {
    /// Return a point-in-time snapshot of the full compositor state.
    async fn snapshot(&self) -> Result<CompositorSnapshot, CompositorError>;

    /// Subscribe to a live stream of compositor events.
    ///
    /// The stream terminates when the backend shuts down. Callers that need
    /// persistent delivery should use the supervisor-wrapped handle, which
    /// transparently reconnects and resumes the stream.
    fn subscribe(&self) -> BoxStream<'static, CompositorEvent>;

    /// Send an action to the compositor (focus, close, exec, …).
    async fn dispatch(&self, action: CompositorAction) -> Result<(), CompositorError>;

    /// Human-readable backend identifier for logging and diagnostics.
    fn name(&self) -> &'static str;
}
