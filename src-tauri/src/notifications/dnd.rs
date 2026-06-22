use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use super::types::NotificationEvent;

#[derive(Debug)]
pub struct DndState {
    enabled: RwLock<bool>,
    event_tx: broadcast::Sender<NotificationEvent>,
}

impl DndState {
    pub fn new(event_tx: broadcast::Sender<NotificationEvent>) -> Arc<Self> {
        Arc::new(Self {
            enabled: RwLock::new(false),
            event_tx,
        })
    }

    pub async fn is_enabled(&self) -> bool {
        *self.enabled.read().await
    }

    pub async fn set_enabled(&self, enabled: bool) {
        *self.enabled.write().await = enabled;
        let _ = self.event_tx.send(NotificationEvent::DndChanged { enabled });
    }
}
