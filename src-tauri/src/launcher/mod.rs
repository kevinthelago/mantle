pub mod commands;
pub mod desktop_entry;
pub mod plugin;
pub mod search;
pub mod service;
pub mod types;
pub mod usage_store;

use std::sync::Arc;
use tokio::sync::RwLock;

pub use plugin::{setup_surface, toggle_launcher};
pub use service::LauncherService;
pub use types::*;

pub struct LauncherState(pub Arc<RwLock<LauncherService>>);

impl LauncherState {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(LauncherService::new())))
    }
}

impl Default for LauncherState {
    fn default() -> Self {
        Self::new()
    }
}
