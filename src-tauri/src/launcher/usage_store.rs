use anyhow::Result;
use sled::Db;
use std::path::PathBuf;

/// Persists per-app launch counts in an embedded sled database.
pub struct UsageStore {
    pub(crate) db: Db,
}

impl UsageStore {
    pub fn open() -> Result<Self> {
        let path = store_path();
        let db = sled::open(&path)?;
        Ok(Self { db })
    }

    /// Return the launch count for the given app id.
    pub fn count(&self, app_id: &str) -> u32 {
        self.db
            .get(app_id.as_bytes())
            .ok()
            .flatten()
            .and_then(|v| {
                let bytes: [u8; 4] = v.as_ref().try_into().ok()?;
                Some(u32::from_le_bytes(bytes))
            })
            .unwrap_or(0)
    }

    /// Increment the launch count for the given app id and flush.
    pub fn increment(&self, app_id: &str) -> Result<()> {
        let current = self.count(app_id);
        let next = current.saturating_add(1).to_le_bytes();
        self.db.insert(app_id.as_bytes(), &next)?;
        self.db.flush()?;
        Ok(())
    }

    /// Return all stored (id, count) pairs.
    pub fn all_counts(&self) -> Vec<(String, u32)> {
        self.db
            .iter()
            .filter_map(|r| {
                let (k, v) = r.ok()?;
                let id = String::from_utf8(k.to_vec()).ok()?;
                let bytes: [u8; 4] = v.as_ref().try_into().ok()?;
                Some((id, u32::from_le_bytes(bytes)))
            })
            .collect()
    }
}

fn store_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("mantle")
        .join("launcher_usage")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn open_temp_store() -> (UsageStore, TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let db = sled::open(dir.path().join("usage")).unwrap();
        (UsageStore { db }, dir)
    }

    #[test]
    fn increment_and_count() {
        let (store, _dir) = open_temp_store();
        assert_eq!(store.count("app.firefox"), 0);
        store.increment("app.firefox").unwrap();
        store.increment("app.firefox").unwrap();
        assert_eq!(store.count("app.firefox"), 2);
    }

    #[test]
    fn all_counts_returns_all() {
        let (store, _dir) = open_temp_store();
        store.increment("a").unwrap();
        store.increment("b").unwrap();
        store.increment("b").unwrap();
        let counts = store.all_counts();
        assert_eq!(counts.len(), 2);
        let b = counts.iter().find(|(id, _)| id == "b").unwrap();
        assert_eq!(b.1, 2);
    }
}
