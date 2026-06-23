pub mod bridge;
pub mod daemon;
pub mod dnd;
pub mod history;
pub mod sanitize;
pub mod types;

use std::sync::Arc;

pub use bridge::{
    clear_notification_history, dismiss_notification, get_active_notifications, get_dnd_enabled,
    get_notification_history, invoke_action, pause_notification_expiry, resume_notification_expiry,
    set_dnd_enabled, spawn_event_relay,
};
pub use daemon::{NotificationsServer, ServerState};
pub use dnd::DndState;
pub use history::NotificationHistory;
pub use types::NotificationEvent;

/// Start the FDO notification daemon and return the shared state handles.
pub async fn start_daemon(
    data_dir: std::path::PathBuf,
) -> Result<(Arc<ServerState>, Arc<DndState>, Arc<NotificationHistory>), zbus::Error> {
    let (server_state, _rx) = ServerState::new();
    let dnd = DndState::new(server_state.event_tx.clone());
    let history_path = data_dir.join("notifications").join("history.json");
    let history = NotificationHistory::load(history_path).await;

    let server = NotificationsServer {
        state: server_state.clone(),
    };

    let conn = zbus::connection::Builder::session()?
        .name("org.freedesktop.Notifications")?
        .serve_at("/org/freedesktop/Notifications", server)?
        .build()
        .await
        .map_err(|e| {
            if e.to_string().contains("already owned") {
                zbus::Error::Failure(
                    "Another notification daemon is already running. \
                     Stop mako/dunst/swaync before starting mantle."
                        .into(),
                )
            } else {
                e
            }
        })?;

    server_state.set_connection(conn);
    Ok((server_state, dnd, history))
}

/// Tauri plugin that starts the notification daemon and registers all commands.
/// Self-registers into the service registry via inventory so lib.rs stays unchanged.
pub fn plugin_init() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri::plugin::Builder::new("notifications")
        .setup(|app, _| {
            // Initialize the notifications window as a layer-shell overlay surface
            // so toast popups float above all other surfaces.
            #[cfg(target_os = "linux")]
            {
                use crate::layer_shell::manager::{AnchorEdge, KeyboardHint, LayerHint, Margins};
                use crate::layer_shell::{LayerShellManager, SurfaceConfig};
                use tauri::Manager;

                if let Some(win) = app.get_webview_window("notifications") {
                    let cfg = SurfaceConfig {
                        namespace: "mantle-notifications".to_string(),
                        layer: LayerHint::Overlay,
                        anchors: vec![AnchorEdge::Top, AnchorEdge::Right],
                        exclusive_zone: 0,
                        margins: Margins {
                            top: 40,
                            bottom: 0,
                            left: 0,
                            right: 0,
                        },
                        keyboard_mode: KeyboardHint::None,
                        output: None,
                    };
                    if let Err(e) = LayerShellManager::new().apply(&win, &cfg) {
                        log::warn!("Failed to apply layer-shell to notifications window: {e}");
                    }
                } else {
                    log::warn!("notifications window not found in tauri.conf.json");
                }
            }

            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                use tauri::Manager;
                let data_dir = app
                    .path()
                    .app_data_dir()
                    .expect("failed to resolve app data dir");
                match start_daemon(data_dir).await {
                    Ok((server, dnd, history)) => {
                        app.manage(server.clone());
                        app.manage(dnd);
                        app.manage(history.clone());
                        spawn_event_relay(app, server, history);
                    }
                    Err(e) => {
                        log::error!("notification daemon failed to start: {e}");
                        std::process::exit(1);
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_active_notifications,
            get_notification_history,
            get_dnd_enabled,
            dismiss_notification,
            invoke_action,
            pause_notification_expiry,
            resume_notification_expiry,
            set_dnd_enabled,
            clear_notification_history,
        ])
        .build()
}

// Self-register into the service registry — lib.rs needs no changes.
inventory::submit! {
    crate::registry::MantlePlugin {
        name: "notifications",
        build: plugin_init,
    }
}
