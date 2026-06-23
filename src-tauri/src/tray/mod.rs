//! System-tray orchestration: watcher → item tracking → Tauri events + commands.

pub mod dbusmenu;
pub mod icon;
pub mod item;
pub mod watcher;

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::Mutex;
use zbus::Connection;

pub fn plugin_init() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri::plugin::Builder::new("system-tray")
        .setup(|app, _| {
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                let svc = TrayService::init(app.clone()).await;
                app.manage(svc);
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_tray_items,
            tray_item_activate,
            tray_menu_event,
            tray_refresh_menu,
        ])
        .build()
}

inventory::submit! {
    crate::registry::MantlePlugin { name: "system-tray", build: plugin_init }
}

use crate::tray::dbusmenu::TrayMenu;
use crate::tray::item::{read_item, TrayItem};
use crate::tray::watcher::{start_watcher, WatcherEvent};

// ── Snapshot sent to the frontend ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TraySnapshot {
    pub items: Vec<TrayItem>,
}

// ── Service ───────────────────────────────────────────────────────────────────

pub struct TrayService {
    items: Arc<Mutex<HashMap<String, TrayItem>>>,
    conn: Connection,
}

impl TrayService {
    pub async fn init(app: AppHandle) -> Arc<Self> {
        let conn = match Connection::session().await {
            Ok(c) => c,
            Err(e) => {
                log::warn!("tray service: no session bus: {e}");
                // Return a stub service.
                return Arc::new(Self {
                    items: Arc::new(Mutex::new(HashMap::new())),
                    conn: Connection::system().await.expect("fallback dbus"),
                });
            }
        };

        let items: Arc<Mutex<HashMap<String, TrayItem>>> = Arc::new(Mutex::new(HashMap::new()));
        let svc = Arc::new(Self {
            items: items.clone(),
            conn: conn.clone(),
        });

        let mut rx = match start_watcher(conn.clone()).await {
            Ok(r) => r,
            Err(e) => {
                log::warn!("tray watcher failed: {e}");
                return svc;
            }
        };

        let items2 = items.clone();
        let app2 = app.clone();
        let conn2 = conn.clone();
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                match event {
                    WatcherEvent::ItemAdded(service, obj_path) => {
                        let key = format!("{service}{obj_path}");
                        // Avoid duplicate tracking while we fetch.
                        {
                            let lock = items2.lock().await;
                            if lock.contains_key(&key) {
                                continue;
                            }
                        }

                        let item = read_item(&conn2, &key, &service, &obj_path).await;
                        {
                            let mut lock = items2.lock().await;
                            lock.insert(key.clone(), item);
                        }
                        emit_snapshot(&items2, &app2).await;

                        // Spawn a watcher task so item updates flow in live.
                        let items3 = items2.clone();
                        let app3 = app2.clone();
                        let conn3 = conn2.clone();
                        let key3 = key.clone();
                        tokio::spawn(async move {
                            crate::tray::item::watch_item(
                                conn3,
                                key3.clone(),
                                service.clone(),
                                obj_path.clone(),
                                move |updated| {
                                    let items4 = items3.clone();
                                    let app4 = app3.clone();
                                    let key4 = key3.clone();
                                    tokio::spawn(async move {
                                        items4.lock().await.insert(key4, updated);
                                        emit_snapshot(&items4, &app4).await;
                                    });
                                },
                            )
                            .await;
                        });
                    }
                    WatcherEvent::ItemRemoved(service, obj_path) => {
                        let key = format!("{service}{obj_path}");
                        items2.lock().await.remove(&key);
                        app2.emit("tray_item_removed", &key).ok();
                        emit_snapshot(&items2, &app2).await;
                    }
                }
            }
        });

        svc
    }

    pub async fn snapshot(&self) -> TraySnapshot {
        let lock = self.items.lock().await;
        TraySnapshot {
            items: lock.values().cloned().collect(),
        }
    }
}

async fn emit_snapshot(items: &Arc<Mutex<HashMap<String, TrayItem>>>, app: &AppHandle) {
    let snap = TraySnapshot {
        items: items.lock().await.values().cloned().collect(),
    };
    app.emit("tray_update", &snap).ok();
}

// ── Tauri commands ────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_tray_items(state: State<'_, Arc<TrayService>>) -> Result<TraySnapshot, String> {
    Ok(state.snapshot().await)
}

/// Left-click an SNI item → calls Activate(x, y).
#[tauri::command]
pub async fn tray_item_activate(
    key: String,
    x: i32,
    y: i32,
    state: State<'_, Arc<TrayService>>,
) -> Result<(), String> {
    let lock = state.items.lock().await;
    let item = lock.get(&key).ok_or_else(|| "item not found".to_owned())?;

    let (service, obj_path) = split_key(&key);
    let proxy = zbus::Proxy::new(&state.conn, service, obj_path, "org.kde.StatusNotifierItem")
        .await
        .map_err(|e| e.to_string())?;
    drop(lock);

    proxy
        .call_method("Activate", &(x, y))
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Send a DBusMenu event (e.g. "clicked") for a menu item.
#[tauri::command]
pub async fn tray_menu_event(
    key: String,
    item_id: i32,
    event: String,
    state: State<'_, Arc<TrayService>>,
) -> Result<(), String> {
    let (service, _obj_path) = split_key(&key);
    let menu_path = {
        let lock = state.items.lock().await;
        lock.get(&key)
            .and_then(|i| i.menu_path.clone())
            .ok_or_else(|| "item or menu not found".to_owned())?
    };
    crate::tray::dbusmenu::send_menu_event(&state.conn, &service, &menu_path, item_id, &event).await
}

/// Re-fetch the menu tree for an item (call before opening a context menu).
#[tauri::command]
pub async fn tray_refresh_menu(
    key: String,
    state: State<'_, Arc<TrayService>>,
) -> Result<Option<TrayMenu>, String> {
    let (service, _) = split_key(&key);
    let menu_path = {
        let lock = state.items.lock().await;
        lock.get(&key).and_then(|i| i.menu_path.clone())
    };
    let Some(mp) = menu_path else {
        return Ok(None);
    };
    let menu = crate::tray::dbusmenu::fetch_menu(&state.conn, &service, &mp)
        .await
        .ok();
    // Store updated menu.
    let mut lock = state.items.lock().await;
    if let Some(item) = lock.get_mut(&key) {
        item.menu = menu.clone();
    }
    Ok(menu)
}

fn split_key(key: &str) -> (&str, &str) {
    if let Some(idx) = key.find('/') {
        (&key[..idx], &key[idx..])
    } else {
        (key, "/StatusNotifierItem")
    }
}
