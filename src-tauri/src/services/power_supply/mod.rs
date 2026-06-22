use std::sync::Arc;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager, State};
use tokio::sync::Mutex;
use zbus::{proxy, Connection};

pub fn plugin_init() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri::plugin::Builder::new("power-supply")
        .setup(|app, _| {
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                let svc = PowerSupplyService::init(app.clone()).await;
                app.manage(svc);
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_battery_snapshot])
        .build()
}

inventory::submit! {
    crate::registry::MantlePlugin { name: "power-supply", build: plugin_init }
}

// ── Snapshot ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BatterySnapshot {
    pub available: bool,
    pub present: bool,
    pub percentage: f64,
    pub state: BatteryState,
    /// Seconds until empty (0 when charging or unknown)
    pub time_to_empty: i64,
    /// Seconds until full (0 when discharging or unknown)
    pub time_to_full: i64,
}

impl BatterySnapshot {
    fn unavailable() -> Self {
        Self {
            available: false,
            present: false,
            percentage: 0.0,
            state: BatteryState::Unknown,
            time_to_empty: 0,
            time_to_full: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum BatteryState {
    Unknown,
    Charging,
    Discharging,
    Empty,
    FullyCharged,
    PendingCharge,
    PendingDischarge,
}

impl BatteryState {
    fn from_u32(v: u32) -> Self {
        match v {
            1 => Self::Charging,
            2 => Self::Discharging,
            3 => Self::Empty,
            4 => Self::FullyCharged,
            5 => Self::PendingCharge,
            6 => Self::PendingDischarge,
            _ => Self::Unknown,
        }
    }
}

// ── D-Bus proxies ─────────────────────────────────────────────────────────────

#[proxy(
    interface = "org.freedesktop.UPower",
    default_service = "org.freedesktop.UPower",
    default_path = "/org/freedesktop/UPower"
)]
trait UPower {
    fn enumerate_devices(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;
}

#[proxy(
    interface = "org.freedesktop.UPower.Device",
    default_service = "org.freedesktop.UPower"
)]
trait UPowerDevice {
    #[zbus(property, name = "Type")]
    fn device_type(&self) -> zbus::Result<u32>;
    #[zbus(property)]
    fn percentage(&self) -> zbus::Result<f64>;
    #[zbus(property)]
    fn state(&self) -> zbus::Result<u32>;
    #[zbus(property)]
    fn time_to_empty(&self) -> zbus::Result<i64>;
    #[zbus(property)]
    fn time_to_full(&self) -> zbus::Result<i64>;
    #[zbus(property)]
    fn is_present(&self) -> zbus::Result<bool>;
}

// ── Service ───────────────────────────────────────────────────────────────────

pub struct PowerSupplyService {
    snapshot: Arc<Mutex<BatterySnapshot>>,
}

impl PowerSupplyService {
    pub async fn init(app: AppHandle) -> Arc<Self> {
        let snapshot = Arc::new(Mutex::new(BatterySnapshot::unavailable()));
        let svc = Arc::new(Self { snapshot: snapshot.clone() });
        let app2 = app.clone();
        tokio::spawn(async move {
            if let Err(e) = run_battery_loop(app2, snapshot).await {
                log::warn!("battery service: {e}");
            }
        });
        svc
    }

    pub async fn snapshot(&self) -> BatterySnapshot {
        self.snapshot.lock().await.clone()
    }
}

async fn find_battery_path(
    conn: &Connection,
) -> zbus::Result<Option<zbus::zvariant::OwnedObjectPath>> {
    let upower = UPowerProxy::new(conn).await?;
    for path in upower.enumerate_devices().await? {
        let dev = UPowerDeviceProxy::builder(conn)
            .path(path.clone())?
            .build()
            .await?;
        if dev.device_type().await? == 2 {
            return Ok(Some(path));
        }
    }
    Ok(None)
}

async fn read_snapshot(
    dev: &UPowerDeviceProxy<'_>,
) -> zbus::Result<BatterySnapshot> {
    Ok(BatterySnapshot {
        available: true,
        present: dev.is_present().await?,
        percentage: dev.percentage().await?,
        state: BatteryState::from_u32(dev.state().await?),
        time_to_empty: dev.time_to_empty().await?,
        time_to_full: dev.time_to_full().await?,
    })
}

async fn run_battery_loop(
    app: AppHandle,
    snapshot: Arc<Mutex<BatterySnapshot>>,
) -> zbus::Result<()> {
    let conn = Connection::system().await?;

    let Some(path) = find_battery_path(&conn).await? else {
        log::info!("no battery device found via UPower");
        return Ok(());
    };

    let dev = UPowerDeviceProxy::builder(&conn)
        .path(path)?
        .build()
        .await?;

    let initial = read_snapshot(&dev).await?;
    {
        let mut lock = snapshot.lock().await;
        *lock = initial.clone();
    }
    app.emit("battery_update", &initial).ok();

    let mut pct = dev.receive_percentage_changed().await;
    let mut state = dev.receive_state_changed().await;
    let mut tte = dev.receive_time_to_empty_changed().await;
    let mut ttf = dev.receive_time_to_full_changed().await;
    let mut present = dev.receive_is_present_changed().await;

    loop {
        tokio::select! {
            Some(_) = pct.next() => {}
            Some(_) = state.next() => {}
            Some(_) = tte.next() => {}
            Some(_) = ttf.next() => {}
            Some(_) = present.next() => {}
        }
        if let Ok(snap) = read_snapshot(&dev).await {
            let mut lock = snapshot.lock().await;
            *lock = snap.clone();
            drop(lock);
            app.emit("battery_update", &snap).ok();
        }
    }
}

// ── Tauri commands ────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_battery_snapshot(
    state: State<'_, Arc<PowerSupplyService>>,
) -> Result<BatterySnapshot, String> {
    Ok(state.snapshot().await)
}
