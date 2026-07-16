//! StatusNotifierWatcher: own the name or co-host against an existing watcher.
//!
//! If we win `org.kde.StatusNotifierWatcher`:
//!   • Serve the interface ourselves (object server).
//!   • Track items; emit StatusNotifierItem{Registered,Unregistered} signals.
//!
//! If the name is already owned (KDE Plasma, etc.):
//!   • Call RegisterStatusNotifierHost on the existing watcher.
//!   • Subscribe to its StatusNotifierItemRegistered/Unregistered signals.
//!   • Seed from RegisteredStatusNotifierItems property.

use std::collections::HashMap;
use std::sync::Arc;

use futures_util::StreamExt;
use tokio::sync::{mpsc, Mutex};
use zbus::fdo::RequestNameFlags;
use zbus::{fdo, interface, proxy, Connection, MessageHeader, SignalContext};

use crate::tray::item::parse_sni_key;

// ── Events from watcher to the TrayService ────────────────────────────────────

#[derive(Debug, Clone)]
pub enum WatcherEvent {
    ItemAdded(String, String), // (service, object_path)
    ItemRemoved(String, String),
}

// ── Co-host proxy (when someone else owns the name) ───────────────────────────

#[proxy(
    interface = "org.kde.StatusNotifierWatcher",
    default_service = "org.kde.StatusNotifierWatcher",
    default_path = "/StatusNotifierWatcher"
)]
trait StatusNotifierWatcher {
    fn register_status_notifier_host(&self, service: &str) -> zbus::Result<()>;
    fn register_status_notifier_item(&self, service: &str) -> zbus::Result<()>;

    #[zbus(property)]
    fn registered_status_notifier_items(&self) -> zbus::Result<Vec<String>>;

    #[zbus(signal)]
    fn status_notifier_item_registered(&self, service: &str) -> zbus::Result<()>;
    #[zbus(signal)]
    fn status_notifier_item_unregistered(&self, service: &str) -> zbus::Result<()>;
}

// ── Object-server implementation (when we own the name) ───────────────────────

pub struct WatcherImpl {
    items: Arc<Mutex<HashMap<String, ()>>>,
    hosts: Arc<Mutex<Vec<String>>>,
    event_tx: mpsc::Sender<WatcherEvent>,
    conn: Connection,
}

#[interface(name = "org.kde.StatusNotifierWatcher")]
impl WatcherImpl {
    async fn register_status_notifier_host(
        &self,
        service: &str,
        #[zbus(signal_context)] ctx: SignalContext<'_>,
    ) -> fdo::Result<()> {
        self.hosts.lock().await.push(service.to_owned());
        WatcherImpl::status_notifier_host_registered(&ctx).await?;
        Ok(())
    }

    async fn register_status_notifier_item(
        &self,
        service: &str,
        #[zbus(header)] hdr: MessageHeader<'_>,
        #[zbus(signal_context)] ctx: SignalContext<'_>,
    ) -> fdo::Result<()> {
        let sender = hdr
            .sender()
            .ok_or_else(|| fdo::Error::Failed("no sender".into()))?
            .as_str()
            .to_owned();
        let (svc, path) = parse_sni_key(service, &sender);
        let key = format!("{svc}{path}");
        self.items.lock().await.insert(key.clone(), ());
        self.event_tx
            .send(WatcherEvent::ItemAdded(svc, path))
            .await
            .ok();
        WatcherImpl::status_notifier_item_registered(&ctx, &key).await?;
        Ok(())
    }

    #[zbus(property)]
    async fn registered_status_notifier_items(&self) -> Vec<String> {
        self.items.lock().await.keys().cloned().collect()
    }

    #[zbus(property)]
    async fn is_status_notifier_host_registered(&self) -> bool {
        !self.hosts.lock().await.is_empty()
    }

    #[zbus(property)]
    async fn protocol_version(&self) -> i32 {
        0
    }

    #[zbus(signal)]
    async fn status_notifier_item_registered(
        ctx: &SignalContext<'_>,
        service: &str,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn status_notifier_item_unregistered(
        ctx: &SignalContext<'_>,
        service: &str,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn status_notifier_host_registered(ctx: &SignalContext<'_>) -> zbus::Result<()>;
}

impl WatcherImpl {
    /// Watch NameOwnerChanged to detect when an item's service disappears.
    pub async fn watch_name_changes(self: Arc<Self>) {
        let conn = self.conn.clone();
        let event_tx = self.event_tx.clone();
        let items = self.items.clone();
        tokio::spawn(async move {
            let Ok(dbus) = zbus::fdo::DBusProxy::new(&conn).await else {
                return;
            };
            let Ok(mut stream) = dbus.receive_name_owner_changed().await else {
                return;
            };
            while let Some(signal) = stream.next().await {
                let Ok(args) = signal.args() else { continue };
                if args.new_owner().is_none() {
                    let lost = args.name().to_owned();
                    let mut lock = items.lock().await;
                    let removed: Vec<String> = lock
                        .keys()
                        .filter(|k| k.starts_with(&*lost))
                        .cloned()
                        .collect();
                    for key in removed {
                        lock.remove(&key);
                        let (svc, path) = parse_sni_key(&key, "");
                        event_tx
                            .send(WatcherEvent::ItemRemoved(svc, path))
                            .await
                            .ok();
                    }
                }
            }
        });
    }
}

// ── Public entry point ────────────────────────────────────────────────────────

/// Try to own `org.kde.StatusNotifierWatcher`; if taken, co-host instead.
/// Returns a channel that yields `WatcherEvent`s.
pub async fn start_watcher(conn: Connection) -> Result<mpsc::Receiver<WatcherEvent>, zbus::Error> {
    let (tx, rx) = mpsc::channel(64);

    let reply = conn
        .request_name_with_flags(
            "org.kde.StatusNotifierWatcher",
            RequestNameFlags::DoNotQueue.into(),
        )
        .await;

    match reply {
        Ok(zbus::fdo::RequestNameReply::PrimaryOwner)
        | Ok(zbus::fdo::RequestNameReply::AlreadyOwner) => {
            run_as_primary_watcher(conn, tx).await?;
        }
        _ => {
            run_as_co_host(conn, tx).await?;
        }
    }

    Ok(rx)
}

async fn run_as_primary_watcher(
    conn: Connection,
    tx: mpsc::Sender<WatcherEvent>,
) -> Result<(), zbus::Error> {
    let items = Arc::new(Mutex::new(HashMap::new()));
    let hosts = Arc::new(Mutex::new(Vec::new()));

    let watcher = Arc::new(WatcherImpl {
        items: items.clone(),
        hosts: hosts.clone(),
        event_tx: tx,
        conn: conn.clone(),
    });

    conn.object_server()
        .at("/StatusNotifierWatcher", (*watcher).clone())
        .await?;

    // Register ourselves as a host too.
    let own_name = conn
        .unique_name()
        .map(|n| n.as_str().to_owned())
        .unwrap_or_default();
    let ctx = SignalContext::new(&conn, "/StatusNotifierWatcher")?;
    hosts.lock().await.push(own_name);
    WatcherImpl::status_notifier_host_registered(&ctx)
        .await
        .ok();

    // Watch for disappearing services.
    watcher.watch_name_changes().await;

    log::info!("StatusNotifierWatcher: primary owner");
    Ok(())
}

async fn run_as_co_host(
    conn: Connection,
    tx: mpsc::Sender<WatcherEvent>,
) -> Result<(), zbus::Error> {
    let proxy = StatusNotifierWatcherProxy::new(&conn).await?;

    // Register as a host.
    let own_name = conn
        .unique_name()
        .map(|n| n.as_str().to_owned())
        .unwrap_or_default();
    proxy.register_status_notifier_host(&own_name).await.ok();

    // Seed from existing items.
    if let Ok(existing) = proxy.registered_status_notifier_items().await {
        for raw in existing {
            let (svc, path) = parse_sni_key(&raw, "");
            tx.send(WatcherEvent::ItemAdded(svc, path)).await.ok();
        }
    }

    // Watch for new/removed items.
    let tx2 = tx.clone();
    let mut added_stream = proxy.receive_status_notifier_item_registered().await?;
    let mut removed_stream = proxy.receive_status_notifier_item_unregistered().await?;

    tokio::spawn(async move {
        loop {
            tokio::select! {
                Some(sig) = added_stream.next() => {
                    if let Ok(args) = sig.args() {
                        let (svc, path) = parse_sni_key(args.service(), "");
                        tx.send(WatcherEvent::ItemAdded(svc, path)).await.ok();
                    }
                }
                Some(sig) = removed_stream.next() => {
                    if let Ok(args) = sig.args() {
                        let (svc, path) = parse_sni_key(args.service(), "");
                        tx2.send(WatcherEvent::ItemRemoved(svc, path)).await.ok();
                    }
                }
            }
        }
    });

    log::info!("StatusNotifierWatcher: co-host mode");
    Ok(())
}

// Required because WatcherImpl is served via object_server (needs Clone).
impl Clone for WatcherImpl {
    fn clone(&self) -> Self {
        Self {
            items: self.items.clone(),
            hosts: self.hosts.clone(),
            event_tx: self.event_tx.clone(),
            conn: self.conn.clone(),
        }
    }
}
