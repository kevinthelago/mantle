use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::types::Notification;

const MAX_HISTORY: usize = 200;

#[derive(Debug)]
pub struct NotificationHistory {
    entries: RwLock<Vec<Notification>>,
    path: PathBuf,
}

impl NotificationHistory {
    pub async fn load(path: PathBuf) -> Arc<Self> {
        let entries = Self::read_from_disk(&path).await.unwrap_or_default();
        Arc::new(Self {
            entries: RwLock::new(entries),
            path,
        })
    }

    async fn read_from_disk(path: &PathBuf) -> Option<Vec<Notification>> {
        let bytes = tokio::fs::read(path).await.ok()?;
        serde_json::from_slice(&bytes).ok()
    }

    pub async fn push(&self, notification: Notification) {
        let mut entries = self.entries.write().await;
        // Replace existing entry with same id if present.
        if let Some(pos) = entries.iter().position(|n| n.id == notification.id) {
            entries[pos] = notification;
        } else {
            entries.insert(0, notification);
            if entries.len() > MAX_HISTORY {
                entries.truncate(MAX_HISTORY);
            }
        }
        drop(entries);
        self.persist().await;
    }

    pub async fn remove(&self, id: u32) {
        let mut entries = self.entries.write().await;
        entries.retain(|n| n.id != id);
        drop(entries);
        self.persist().await;
    }

    pub async fn clear(&self) {
        *self.entries.write().await = Vec::new();
        self.persist().await;
    }

    pub async fn list(&self) -> Vec<Notification> {
        self.entries.read().await.clone()
    }

    async fn persist(&self) {
        let entries = self.entries.read().await;
        if let Ok(json) = serde_json::to_vec_pretty(&*entries) {
            if let Some(parent) = self.path.parent() {
                let _ = tokio::fs::create_dir_all(parent).await;
            }
            let _ = tokio::fs::write(&self.path, json).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU32, Ordering};

    use crate::notifications::types::Urgency;

    use super::*;

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn test_path() -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        std::env::temp_dir().join(format!("mantle-test-history-{n}.json"))
    }

    fn dummy(id: u32) -> Notification {
        Notification {
            id,
            app_name: "test".into(),
            app_icon: "".into(),
            summary: format!("Summary {id}"),
            body: "".into(),
            actions: vec![],
            urgency: Urgency::Normal,
            category: None,
            desktop_entry: None,
            image: None,
            image_path: None,
            resident: false,
            transient: false,
            expire_timeout_ms: None,
            created_at: 0,
        }
    }

    #[tokio::test]
    async fn push_adds_entry() {
        let h = NotificationHistory::load(test_path()).await;
        h.push(dummy(1)).await;
        assert_eq!(h.list().await.len(), 1);
        assert_eq!(h.list().await[0].id, 1);
    }

    #[tokio::test]
    async fn push_replaces_existing_id() {
        let h = NotificationHistory::load(test_path()).await;
        h.push(dummy(1)).await;
        let mut updated = dummy(1);
        updated.summary = "Updated".into();
        h.push(updated).await;
        let list = h.list().await;
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].summary, "Updated");
    }

    #[tokio::test]
    async fn push_inserts_at_front() {
        let h = NotificationHistory::load(test_path()).await;
        h.push(dummy(1)).await;
        h.push(dummy(2)).await;
        assert_eq!(h.list().await[0].id, 2);
    }

    #[tokio::test]
    async fn remove_deletes_entry() {
        let h = NotificationHistory::load(test_path()).await;
        h.push(dummy(1)).await;
        h.push(dummy(2)).await;
        h.remove(1).await;
        let list = h.list().await;
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, 2);
    }

    #[tokio::test]
    async fn remove_noop_for_unknown_id() {
        let h = NotificationHistory::load(test_path()).await;
        h.push(dummy(1)).await;
        h.remove(99).await;
        assert_eq!(h.list().await.len(), 1);
    }

    #[tokio::test]
    async fn clear_empties_list() {
        let h = NotificationHistory::load(test_path()).await;
        h.push(dummy(1)).await;
        h.push(dummy(2)).await;
        h.clear().await;
        assert!(h.list().await.is_empty());
    }

    #[tokio::test]
    async fn truncates_at_max_history() {
        let h = NotificationHistory::load(test_path()).await;
        for i in 0..=(MAX_HISTORY as u32 + 10) {
            h.push(dummy(i)).await;
        }
        assert_eq!(h.list().await.len(), MAX_HISTORY);
    }
}
