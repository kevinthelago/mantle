use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

use super::daemon::ServerState;
use super::dnd::DndState;
use super::history::NotificationHistory;
use super::types::{ClosedReason, Notification, NotificationEvent};

// ── read commands ─────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_active_notifications(
    server: State<'_, Arc<ServerState>>,
) -> Result<Vec<Notification>, String> {
    Ok(server.active_list().await)
}

#[tauri::command]
pub async fn get_notification_history(
    history: State<'_, Arc<NotificationHistory>>,
) -> Result<Vec<Notification>, String> {
    Ok(history.list().await)
}

#[tauri::command]
pub async fn get_dnd_enabled(dnd: State<'_, Arc<DndState>>) -> Result<bool, String> {
    Ok(dnd.is_enabled().await)
}

// ── write commands ────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn dismiss_notification(
    id: u32,
    server: State<'_, Arc<ServerState>>,
    history: State<'_, Arc<NotificationHistory>>,
) -> Result<(), String> {
    server.close_from_task(id, ClosedReason::Dismissed).await;
    history.remove(id).await;
    Ok(())
}

#[tauri::command]
pub async fn invoke_action(
    id: u32,
    action_key: String,
    server: State<'_, Arc<ServerState>>,
    app: AppHandle,
) -> Result<(), String> {
    // Emit ActionInvoked over D-Bus (the originating app listens for this).
    if let Some(conn) = server.connection_ref() {
        let ctx = zbus::SignalContext::new(conn, "/org/freedesktop/Notifications")
            .map_err(|e| e.to_string())?;
        super::daemon::NotificationsServer::action_invoked(&ctx, id, &action_key)
            .await
            .map_err(|e| e.to_string())?;
    }
    let _ = server
        .event_tx
        .send(NotificationEvent::ActionInvoked { id, action_key });
    Ok(())
}

#[tauri::command]
pub async fn pause_notification_expiry(
    id: u32,
    server: State<'_, Arc<ServerState>>,
) -> Result<(), String> {
    server.pause_expiry(id).await;
    Ok(())
}

#[tauri::command]
pub async fn resume_notification_expiry(
    id: u32,
    server: State<'_, Arc<ServerState>>,
) -> Result<(), String> {
    server.resume_expiry(id).await;
    Ok(())
}

#[tauri::command]
pub async fn set_dnd_enabled(enabled: bool, dnd: State<'_, Arc<DndState>>) -> Result<(), String> {
    dnd.set_enabled(enabled).await;
    Ok(())
}

#[tauri::command]
pub async fn clear_notification_history(
    history: State<'_, Arc<NotificationHistory>>,
) -> Result<(), String> {
    history.clear().await;
    Ok(())
}

// ── event relay ───────────────────────────────────────────────────────────────

/// Spawns a task that relays ServerState broadcast events to the Tauri frontend
/// via `app.emit("notification-event", payload)`.
pub fn spawn_event_relay(
    app: AppHandle,
    server: Arc<ServerState>,
    history: Arc<NotificationHistory>,
) {
    let mut rx = server.subscribe();
    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    // Persist to history on add/replace; remove on transient close.
                    match &event {
                        NotificationEvent::Added(n)
                        | NotificationEvent::Replaced {
                            notification: n, ..
                        } => {
                            if !n.transient {
                                history.push(n.clone()).await;
                            }
                        }
                        _ => {}
                    }
                    let _ = app.emit("notification-event", &event);
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    log::warn!("notification event relay lagged, dropped {n} events");
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    });
}
