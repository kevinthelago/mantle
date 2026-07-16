pub mod bridge;
pub mod daemon;
pub mod dnd;
pub mod history;
pub mod sanitize;
pub mod types;

use std::sync::Arc;

pub use bridge::spawn_event_relay;
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
                use crate::layer_shell::{
                    ExclusiveZone, KeyboardMode, Layer, LayerShellManager, SurfaceConfig,
                };
                use tauri::Manager;

                if let Some(win) = app.get_webview_window("notifications") {
                    let gtk_win = win
                        .gtk_window()
                        .expect("failed to get GTK window for notifications");
                    let cfg = SurfaceConfig {
                        namespace: "mantle-notifications".to_string(),
                        layer: Layer::Overlay,
                        anchors: [true, true, false, false], // top, right
                        exclusive_zone: ExclusiveZone::None,
                        margins: (40, 0, 0, 0), // 40 px top margin (clears the bar)
                        keyboard_mode: KeyboardMode::None,
                    };
                    LayerShellManager::apply_to_window(&gtk_win, &cfg, None);
                    win.show().expect("failed to show notifications window");
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
            bridge::get_active_notifications,
            bridge::get_notification_history,
            bridge::get_dnd_enabled,
            bridge::dismiss_notification,
            bridge::invoke_action,
            bridge::pause_notification_expiry,
            bridge::resume_notification_expiry,
            bridge::set_dnd_enabled,
            bridge::clear_notification_history,
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
