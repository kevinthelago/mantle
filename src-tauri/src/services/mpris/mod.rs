mod player;

use player::{active_player, extract_i64, extract_string, MprisPlayerProxy};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::Mutex;
use zbus::Connection;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum PlaybackStatus {
    Playing,
    Paused,
    Stopped,
}

impl PlaybackStatus {
    fn from_str(s: &str) -> Self {
        match s {
            "Playing" => Self::Playing,
            "Paused" => Self::Paused,
            _ => Self::Stopped,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub art_url: Option<String>,
    pub length_us: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub player_name: String,
    pub playback_status: PlaybackStatus,
    pub position_us: i64,
    pub volume: f64,
    pub metadata: PlayerMetadata,
    pub can_next: bool,
    pub can_prev: bool,
    pub can_pause: bool,
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

/// Shared D-Bus session handle; `Clone`-able so the watcher task and the
/// managed Tauri state can share the same connection without Arc at the
/// call-site.
#[derive(Clone)]
pub struct MprisService {
    conn: Arc<Mutex<Option<Connection>>>,
}

impl MprisService {
    pub fn new() -> Self {
        Self {
            conn: Arc::new(Mutex::new(None)),
        }
    }

    async fn connection(&self) -> anyhow::Result<Connection> {
        let mut guard = self.conn.lock().await;
        if guard.is_none() {
            let conn = Connection::session().await?;
            *guard = Some(conn);
        }
        Ok(guard.as_ref().unwrap().clone())
    }

    pub async fn get_state(&self) -> anyhow::Result<Option<PlayerState>> {
        let conn = self.connection().await?;
        let Some(name) = active_player(&conn).await? else {
            return Ok(None);
        };

        let short_name = name
            .strip_prefix("org.mpris.MediaPlayer2.")
            .unwrap_or(&name)
            .to_string();

        let proxy = MprisPlayerProxy::builder(&conn)
            .destination(name.as_str())?
            .build()
            .await?;

        let status_str = proxy.playback_status().await.unwrap_or_default();
        let meta_map = proxy.metadata().await.unwrap_or_default();
        let position_us = proxy.position().await.unwrap_or(0);
        let volume = proxy.volume().await.unwrap_or(1.0);
        let can_next = proxy.can_go_next().await.unwrap_or(false);
        let can_prev = proxy.can_go_previous().await.unwrap_or(false);
        let can_pause = proxy.can_pause().await.unwrap_or(false);

        Ok(Some(PlayerState {
            player_name: short_name,
            playback_status: PlaybackStatus::from_str(&status_str),
            position_us,
            volume,
            metadata: PlayerMetadata {
                title: extract_string(&meta_map, "xesam:title"),
                artist: extract_string(&meta_map, "xesam:artist"),
                album: extract_string(&meta_map, "xesam:album"),
                art_url: extract_string(&meta_map, "mpris:artUrl"),
                length_us: extract_i64(&meta_map, "mpris:length"),
            },
            can_next,
            can_prev,
            can_pause,
        }))
    }

    pub async fn play_pause(&self) -> anyhow::Result<()> {
        let conn = self.connection().await?;
        let Some(name) = active_player(&conn).await? else {
            return Ok(());
        };
        MprisPlayerProxy::builder(&conn)
            .destination(name.as_str())?
            .build()
            .await?
            .play_pause()
            .await?;
        Ok(())
    }

    pub async fn next(&self) -> anyhow::Result<()> {
        let conn = self.connection().await?;
        let Some(name) = active_player(&conn).await? else {
            return Ok(());
        };
        MprisPlayerProxy::builder(&conn)
            .destination(name.as_str())?
            .build()
            .await?
            .next()
            .await?;
        Ok(())
    }

    pub async fn prev(&self) -> anyhow::Result<()> {
        let conn = self.connection().await?;
        let Some(name) = active_player(&conn).await? else {
            return Ok(());
        };
        MprisPlayerProxy::builder(&conn)
            .destination(name.as_str())?
            .build()
            .await?
            .previous()
            .await?;
        Ok(())
    }
}

impl Default for MprisService {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn get_player_state(
    service: State<'_, MprisService>,
) -> Result<Option<PlayerState>, String> {
    service.get_state().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn mpris_play_pause(service: State<'_, MprisService>) -> Result<(), String> {
    service.play_pause().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn mpris_next(service: State<'_, MprisService>) -> Result<(), String> {
    service.next().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn mpris_prev(service: State<'_, MprisService>) -> Result<(), String> {
    service.prev().await.map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Background watcher — emits mpris:state-changed to the frontend.
// ---------------------------------------------------------------------------

async fn watch_mpris(app: AppHandle, service: MprisService) {
    let mut last_state: Option<PlayerState> = None;
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(2));
    loop {
        interval.tick().await;
        let new_state = service.get_state().await.ok().flatten();
        let changed = match (&last_state, &new_state) {
            (None, None) => false,
            (Some(_), None) | (None, Some(_)) => true,
            (Some(old), Some(new)) => {
                old.playback_status != new.playback_status
                    || old.metadata.title != new.metadata.title
                    || old.metadata.artist != new.metadata.artist
            }
        };
        if changed {
            let _ = app.emit("mpris:state-changed", &new_state);
            last_state = new_state;
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin — self-registers via inventory so lib.rs needs no per-service edits.
// ---------------------------------------------------------------------------

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("mpris")
        .invoke_handler(tauri::generate_handler![
            get_player_state,
            mpris_play_pause,
            mpris_next,
            mpris_prev,
        ])
        .setup(|app, _api| {
            let service = MprisService::new();
            app.manage(service.clone());
            let handle = app.clone();
            tauri::async_runtime::spawn(watch_mpris(handle, service));
            Ok(())
        })
        .build()
}

inventory::submit! {
    crate::registry::MantlePlugin {
        name: "mpris",
        build: || init::<tauri::Wry>(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playback_status_parses_known_values() {
        assert_eq!(PlaybackStatus::from_str("Playing"), PlaybackStatus::Playing);
        assert_eq!(PlaybackStatus::from_str("Paused"), PlaybackStatus::Paused);
        assert_eq!(PlaybackStatus::from_str("Stopped"), PlaybackStatus::Stopped);
    }

    #[test]
    fn playback_status_unknown_string_becomes_stopped() {
        assert_eq!(PlaybackStatus::from_str("unknown"), PlaybackStatus::Stopped);
        assert_eq!(PlaybackStatus::from_str(""), PlaybackStatus::Stopped);
    }
}
