use std::sync::Arc;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager, State};
use tokio::sync::Mutex;
use zbus::{proxy, Connection};

// ── Snapshot ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct NetworkSnapshot {
    pub available: bool,
    pub state: NetworkState,
    pub connectivity: ConnectivityState,
    pub connection_type: ConnectionType,
    pub ssid: Option<String>,
    pub signal_strength: Option<u8>,
    pub interface: Option<String>,
    pub ip4_address: Option<String>,
}

impl NetworkSnapshot {
    fn unavailable() -> Self {
        Self {
            available: false,
            state: NetworkState::Unknown,
            connectivity: ConnectivityState::Unknown,
            connection_type: ConnectionType::None,
            ssid: None,
            signal_strength: None,
            interface: None,
            ip4_address: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum NetworkState {
    Unknown,
    Asleep,
    Disconnected,
    Disconnecting,
    Connecting,
    ConnectedLocal,
    ConnectedSite,
    ConnectedGlobal,
}

impl NetworkState {
    fn from_u32(v: u32) -> Self {
        match v {
            10 => Self::Asleep,
            20 => Self::Disconnected,
            30 => Self::Disconnecting,
            40 => Self::Connecting,
            50 => Self::ConnectedLocal,
            60 => Self::ConnectedSite,
            70 => Self::ConnectedGlobal,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ConnectivityState {
    Unknown,
    None,
    Portal,
    Limited,
    Full,
}

impl ConnectivityState {
    fn from_u32(v: u32) -> Self {
        match v {
            1 => Self::None,
            2 => Self::Portal,
            3 => Self::Limited,
            4 => Self::Full,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ConnectionType {
    None,
    Wired,
    Wifi,
    Vpn,
    Other(String),
}

// ── D-Bus proxies ─────────────────────────────────────────────────────────────

#[proxy(
    interface = "org.freedesktop.NetworkManager",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager"
)]
trait NetworkManager {
    #[zbus(property)]
    fn state(&self) -> zbus::Result<u32>;
    #[zbus(property)]
    fn connectivity(&self) -> zbus::Result<u32>;
    #[zbus(property)]
    fn primary_connection(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
    #[zbus(signal)]
    fn state_changed(&self, state: u32) -> zbus::Result<()>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.Connection.Active",
    default_service = "org.freedesktop.NetworkManager"
)]
trait ActiveConnection {
    #[zbus(property, name = "Type")]
    fn connection_type(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn devices(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.Device",
    default_service = "org.freedesktop.NetworkManager"
)]
trait NMDevice {
    #[zbus(property)]
    fn ip_interface(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn ip4_config(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.Device.Wireless",
    default_service = "org.freedesktop.NetworkManager"
)]
trait WirelessDevice {
    #[zbus(property)]
    fn active_access_point(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.AccessPoint",
    default_service = "org.freedesktop.NetworkManager"
)]
trait AccessPoint {
    #[zbus(property)]
    fn ssid(&self) -> zbus::Result<Vec<u8>>;
    #[zbus(property)]
    fn strength(&self) -> zbus::Result<u8>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.IP4Config",
    default_service = "org.freedesktop.NetworkManager"
)]
trait IP4Config {
    #[zbus(property)]
    fn address_data(
        &self,
    ) -> zbus::Result<Vec<std::collections::HashMap<String, zbus::zvariant::OwnedValue>>>;
}

// ── Service ───────────────────────────────────────────────────────────────────

pub struct NetworkService {
    snapshot: Arc<Mutex<NetworkSnapshot>>,
}

impl NetworkService {
    pub async fn init(app: AppHandle) -> Arc<Self> {
        let snapshot = Arc::new(Mutex::new(NetworkSnapshot::unavailable()));
        let svc = Arc::new(Self { snapshot: snapshot.clone() });
        let app2 = app.clone();
        tokio::spawn(async move {
            if let Err(e) = run_network_loop(app2, snapshot).await {
                log::warn!("network service: {e}");
            }
        });
        svc
    }

    pub async fn snapshot(&self) -> NetworkSnapshot {
        self.snapshot.lock().await.clone()
    }
}

/// Build a network snapshot; any sub-step failure is silently swallowed.
async fn build_snapshot(nm: &NetworkManagerProxy<'_>, conn: &Connection) -> NetworkSnapshot {
    let state = NetworkState::from_u32(nm.state().await.unwrap_or(0));
    let connectivity = ConnectivityState::from_u32(nm.connectivity().await.unwrap_or(0));

    let base = NetworkSnapshot {
        available: true,
        state,
        connectivity,
        connection_type: ConnectionType::None,
        ssid: None,
        signal_strength: None,
        interface: None,
        ip4_address: None,
    };

    // Delegate to a fallible inner helper so we can use `?` freely.
    enrich_snapshot(base, nm, conn).await.unwrap_or_else(|snap| snap)
}

/// Returns `Ok(enriched)` or `Err(partial)` — either way the caller uses the value.
async fn enrich_snapshot(
    mut snap: NetworkSnapshot,
    nm: &NetworkManagerProxy<'_>,
    conn: &Connection,
) -> Result<NetworkSnapshot, NetworkSnapshot> {
    macro_rules! ok_or_partial {
        ($expr:expr) => {
            match $expr {
                Ok(v) => v,
                Err(_) => return Err(snap),
            }
        };
    }

    // ── Primary active connection ─────────────────────────────────────────────
    let primary_path = ok_or_partial!(nm.primary_connection().await);
    if primary_path.as_str() == "/" {
        return Err(snap);
    }

    let active_builder = ok_or_partial!(ActiveConnectionProxy::builder(conn).path(primary_path));
    let active = ok_or_partial!(active_builder.build().await);

    snap.connection_type = match active.connection_type().await.unwrap_or_default().as_str() {
        "802-3-ethernet" => ConnectionType::Wired,
        "802-11-wireless" => ConnectionType::Wifi,
        "vpn" => ConnectionType::Vpn,
        "" => ConnectionType::None,
        other => ConnectionType::Other(other.to_owned()),
    };

    let mut devices = ok_or_partial!(active.devices().await);
    if devices.is_empty() {
        return Err(snap);
    }
    let dev_path = devices.swap_remove(0);

    // ── Device: IP interface + IPv4 ───────────────────────────────────────────
    if let Ok(dev_builder) = NMDeviceProxy::builder(conn).path(dev_path.clone()) {
        if let Ok(dev) = dev_builder.build().await {
            snap.interface = dev.ip_interface().await.ok().filter(|s| !s.is_empty());
            if let Ok(ip4_path) = dev.ip4_config().await {
                if ip4_path.as_str() != "/" {
                    if let Ok(ip4_builder) = IP4ConfigProxy::builder(conn).path(ip4_path) {
                        if let Ok(ip4) = ip4_builder.build().await {
                            if let Ok(addrs) = ip4.address_data().await {
                                snap.ip4_address = addrs
                                    .first()
                                    .and_then(|m| m.get("address"))
                                    .and_then(|v| v.downcast_ref::<str>().ok())
                                    .map(ToOwned::to_owned);
                            }
                        }
                    }
                }
            }
        }
    }

    // ── Wireless: SSID + signal ───────────────────────────────────────────────
    if snap.connection_type == ConnectionType::Wifi {
        if let Ok(wb) = WirelessDeviceProxy::builder(conn).path(dev_path) {
            if let Ok(wdev) = wb.build().await {
                if let Ok(ap_path) = wdev.active_access_point().await {
                    if ap_path.as_str() != "/" {
                        if let Ok(ab) = AccessPointProxy::builder(conn).path(ap_path) {
                            if let Ok(ap) = ab.build().await {
                                snap.ssid = ap
                                    .ssid()
                                    .await
                                    .ok()
                                    .and_then(|b| String::from_utf8(b).ok())
                                    .filter(|s| !s.is_empty());
                                snap.signal_strength = ap.strength().await.ok();
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(snap)
}

async fn run_network_loop(
    app: AppHandle,
    snapshot: Arc<Mutex<NetworkSnapshot>>,
) -> zbus::Result<()> {
    let conn = Connection::system().await?;
    let nm = NetworkManagerProxy::new(&conn).await?;

    let initial = build_snapshot(&nm, &conn).await;
    {
        let mut lock = snapshot.lock().await;
        *lock = initial.clone();
    }
    app.emit("network_update", &initial).ok();

    let mut state_stream = nm.receive_state_changed().await?;
    let mut primary_stream = nm.receive_primary_connection_changed().await;

    loop {
        tokio::select! {
            Some(_) = state_stream.next() => {}
            Some(_) = primary_stream.next() => {}
        }
        let snap = build_snapshot(&nm, &conn).await;
        {
            let mut lock = snapshot.lock().await;
            *lock = snap.clone();
        }
        app.emit("network_update", &snap).ok();
    }
}

// ── Tauri commands ────────────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
pub async fn get_network_snapshot(
    state: State<'_, Arc<NetworkService>>,
) -> Result<NetworkSnapshot, String> {
    Ok(state.snapshot().await)
}
