/// Compositor bridge: the Tauri managed-state wrapper and event relay.
///
/// The `CompositorState` lives in Tauri's `Manager` and is accessed by
/// command handlers in `plugin.rs`.
use std::sync::Arc;

use futures::StreamExt;
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;
use tracing::error;

use super::backend::CompositorBackend;

/// Tauri managed state for the compositor.
///
/// Wraps the active backend in an `Arc<RwLock<…>>` so it can be accessed
/// from multiple concurrent command invocations without holding the lock
/// longer than necessary.
pub struct CompositorState {
    pub(crate) backend: Arc<RwLock<Box<dyn CompositorBackend>>>,
}

impl CompositorState {
    pub fn new(backend: Box<dyn CompositorBackend>) -> Self {
        Self {
            backend: Arc::new(RwLock::new(backend)),
        }
    }

    /// Spawn the event relay task.
    ///
    /// The relay subscribes to the backend's event stream and forwards each
    /// `CompositorEvent` to all Tauri frontend listeners on the
    /// `"compositor://event"` channel as a JSON payload.
    pub fn spawn_event_relay(&self, app: AppHandle) {
        let backend_arc = Arc::clone(&self.backend);
        tokio::spawn(async move {
            let stream = {
                let backend = backend_arc.read().await;
                backend.subscribe()
            };

            let mut stream = stream;
            while let Some(event) = stream.next().await {
                if let Err(e) = app.emit("compositor://event", &event) {
                    error!("compositor relay: emit failed: {e}");
                }
            }
        });
    }
}
