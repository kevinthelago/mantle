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
        let _ = self
            .event_tx
            .send(NotificationEvent::DndChanged { enabled });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_dnd() -> (Arc<DndState>, broadcast::Receiver<NotificationEvent>) {
        let (tx, rx) = broadcast::channel(16);
        let dnd = DndState::new(tx);
        (dnd, rx)
    }

    #[tokio::test]
    async fn starts_disabled() {
        let (dnd, _rx) = make_dnd();
        assert!(!dnd.is_enabled().await);
    }

    #[tokio::test]
    async fn enable_sets_state_and_emits_event() {
        let (dnd, mut rx) = make_dnd();
        dnd.set_enabled(true).await;
        assert!(dnd.is_enabled().await);
        let event = rx.try_recv().expect("event was sent");
        assert!(matches!(
            event,
            NotificationEvent::DndChanged { enabled: true }
        ));
    }

    #[tokio::test]
    async fn disable_sets_state_and_emits_event() {
        let (dnd, mut rx) = make_dnd();
        dnd.set_enabled(true).await;
        let _ = rx.try_recv();
        dnd.set_enabled(false).await;
        assert!(!dnd.is_enabled().await);
        let event = rx.try_recv().expect("event was sent");
        assert!(matches!(
            event,
            NotificationEvent::DndChanged { enabled: false }
        ));
    }

    #[tokio::test]
    async fn toggle_emits_each_change() {
        let (dnd, mut rx) = make_dnd();
        dnd.set_enabled(true).await;
        dnd.set_enabled(false).await;
        dnd.set_enabled(true).await;
        let mut events = vec![];
        while let Ok(e) = rx.try_recv() {
            events.push(e);
        }
        assert_eq!(events.len(), 3);
    }
}
