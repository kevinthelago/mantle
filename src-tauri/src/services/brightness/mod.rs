use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager, State};
use tokio::sync::Mutex;
use zbus::{proxy, Connection};

pub fn plugin_init() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri::plugin::Builder::new("brightness")
        .setup(|app, _| {
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                let svc = BrightnessService::init(app.clone()).await;
                app.manage(svc);
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_brightness_snapshot,
            set_brightness,
        ])
        .build()
}

inventory::submit! {
    crate::registry::MantlePlugin { name: "brightness", build: plugin_init }
}

// ── Snapshot ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BrightnessSnapshot {
    pub available: bool,
    pub device: String,
    pub brightness: u32,
    pub max_brightness: u32,
    /// 0.0–1.0
    pub percentage: f32,
}

impl BrightnessSnapshot {
    fn unavailable() -> Self {
        Self {
            available: false,
            device: String::new(),
            brightness: 0,
            max_brightness: 1,
            percentage: 0.0,
        }
    }
}

// ── D-Bus proxy for logind ────────────────────────────────────────────────────

#[proxy(
    interface = "org.freedesktop.login1.Session",
    default_service = "org.freedesktop.login1",
    default_path = "/org/freedesktop/login1/session/auto"
)]
trait LogindSession {
    fn set_brightness(&self, subsystem: &str, name: &str, brightness: u32) -> zbus::Result<()>;
}

// ── sysfs helpers ─────────────────────────────────────────────────────────────

fn find_backlight_device() -> Option<(PathBuf, String)> {
    let base = Path::new("/sys/class/backlight");
    std::fs::read_dir(base).ok()?.find_map(|entry| {
        let entry = entry.ok()?;
        let name = entry.file_name().to_string_lossy().into_owned();
        Some((entry.path(), name))
    })
}

fn read_sysfs_u32(path: &Path) -> Option<u32> {
    std::fs::read_to_string(path)
        .ok()?
        .trim()
        .parse()
        .ok()
}

fn read_brightness(dev_path: &Path) -> Option<BrightnessSnapshot> {
    let name = dev_path.file_name()?.to_string_lossy().into_owned();
    let brightness = read_sysfs_u32(&dev_path.join("brightness"))?;
    let max_brightness = read_sysfs_u32(&dev_path.join("max_brightness"))?;
    let percentage = brightness as f32 / max_brightness.max(1) as f32;
    Some(BrightnessSnapshot {
        available: true,
        device: name,
        brightness,
        max_brightness,
        percentage,
    })
}

// ── Service ───────────────────────────────────────────────────────────────────

pub struct BrightnessService {
    snapshot: Arc<Mutex<BrightnessSnapshot>>,
    dev_path: Option<PathBuf>,
}

impl BrightnessService {
    pub async fn init(app: AppHandle) -> Arc<Self> {
        let dev_path = find_backlight_device().map(|(p, _)| p);
        let initial = dev_path
            .as_deref()
            .and_then(read_brightness)
            .unwrap_or_else(BrightnessSnapshot::unavailable);
        let snapshot = Arc::new(Mutex::new(initial));
        let svc = Arc::new(Self {
            snapshot: snapshot.clone(),
            dev_path: dev_path.clone(),
        });
        if dev_path.is_some() {
            let app2 = app.clone();
            tokio::spawn(async move {
                run_brightness_poll(app2, snapshot, dev_path.unwrap()).await;
            });
        }
        svc
    }

    pub async fn snapshot(&self) -> BrightnessSnapshot {
        self.snapshot.lock().await.clone()
    }

    pub fn dev_path(&self) -> Option<&Path> {
        self.dev_path.as_deref()
    }
}

/// Poll sysfs every 500 ms — logind SetBrightness is the only writer we know of,
/// and sysfs doesn't generate inotify events reliably for backlight files.
async fn run_brightness_poll(
    app: AppHandle,
    snapshot: Arc<Mutex<BrightnessSnapshot>>,
    dev_path: PathBuf,
) {
    let mut last: Option<u32> = None;
    let mut interval = tokio::time::interval(Duration::from_millis(500));
    loop {
        interval.tick().await;
        if let Some(snap) = read_brightness(&dev_path) {
            if last != Some(snap.brightness) {
                last = Some(snap.brightness);
                let mut lock = snapshot.lock().await;
                *lock = snap.clone();
                drop(lock);
                app.emit("brightness_update", &snap).ok();
            }
        }
    }
}

// ── Tauri commands ────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_brightness_snapshot(
    state: State<'_, Arc<BrightnessService>>,
) -> Result<BrightnessSnapshot, String> {
    Ok(state.snapshot().await)
}

/// Set brightness as a fraction 0.0–1.0 via logind (no root required).
#[tauri::command]
pub async fn set_brightness(
    fraction: f32,
    state: State<'_, Arc<BrightnessService>>,
) -> Result<(), String> {
    let snap = state.snapshot().await;
    if !snap.available {
        return Err("brightness service unavailable".into());
    }
    let target = (fraction.clamp(0.0, 1.0) * snap.max_brightness as f32).round() as u32;
    let conn = Connection::system().await.map_err(|e| e.to_string())?;
    let session = LogindSessionProxy::new(&conn)
        .await
        .map_err(|e| e.to_string())?;
    session
        .set_brightness("backlight", &snap.device, target)
        .await
        .map_err(|e| e.to_string())
}
