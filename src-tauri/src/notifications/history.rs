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
