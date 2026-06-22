pub mod loader;
pub mod schema;
pub mod watcher;

pub use schema::{Config, OutputConfig, PowerAction};

use std::sync::Arc;
use tokio::sync::{watch, RwLock};

/// Shared app state: the live `Config` updated by the file watcher.
#[derive(Clone)]
pub struct ConfigState {
    inner: Arc<RwLock<Config>>,
    rx: watch::Receiver<Config>,
}

impl ConfigState {
    pub fn new(config: Config, rx: watch::Receiver<Config>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(config)),
            rx,
        }
    }

    pub async fn get(&self) -> Config {
        self.inner.read().await.clone()
    }

    /// Subscribe to config change notifications.
    #[allow(dead_code)]
    pub fn subscribe(&self) -> watch::Receiver<Config> {
        self.rx.clone()
    }

    /// Spawn a background task that keeps `inner` in sync with the watcher.
    pub fn start_sync_loop(&self) {
        let inner = self.inner.clone();
        let mut rx = self.rx.clone();
        tauri::async_runtime::spawn(async move {
            while rx.changed().await.is_ok() {
                let new_cfg = rx.borrow().clone();
                *inner.write().await = new_cfg;
            }
        });
    }
}
