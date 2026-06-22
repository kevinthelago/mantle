use serde::{Deserialize, Serialize};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, Networks, RefreshKind, System};
use tauri::{Manager, State};
use tokio::sync::Mutex;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_percent: f32,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub memory_percent: f32,
    pub net_rx_bytes: u64,
    pub net_tx_bytes: u64,
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

pub struct MetricsService {
    system: Mutex<System>,
    networks: Mutex<Networks>,
}

impl MetricsService {
    pub fn new() -> Self {
        let system = System::new_with_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::new().with_cpu_usage())
                .with_memory(MemoryRefreshKind::new().with_ram()),
        );
        Self {
            system: Mutex::new(system),
            networks: Mutex::new(Networks::new_with_refreshed_list()),
        }
    }

    pub async fn sample(&self) -> SystemMetrics {
        let mut sys = self.system.lock().await;
        // Two-pass CPU refresh: the first call initialises the counters; subsequent
        // calls compute usage over the delta since the last call. Because widget polls
        // are ≥2 s apart the interval is always large enough to be meaningful.
        sys.refresh_cpu_usage();
        sys.refresh_memory();

        let cpu_count = sys.cpus().len().max(1) as f32;
        let cpu_percent = sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>() / cpu_count;

        let used_kb = sys.used_memory();
        let total_kb = sys.total_memory();
        let memory_percent = if total_kb > 0 {
            used_kb as f32 / total_kb as f32 * 100.0
        } else {
            0.0
        };

        let mut nets = self.networks.lock().await;
        nets.refresh();
        let (net_rx_bytes, net_tx_bytes) =
            nets.iter()
                .fold((0u64, 0u64), |(rx, tx), (_, n)| {
                    (rx + n.received(), tx + n.transmitted())
                });

        SystemMetrics {
            cpu_percent,
            memory_used_mb: used_kb / 1024,
            memory_total_mb: total_kb / 1024,
            memory_percent,
            net_rx_bytes,
            net_tx_bytes,
        }
    }
}

impl Default for MetricsService {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn get_system_metrics(
    service: State<'_, MetricsService>,
) -> Result<SystemMetrics, String> {
    Ok(service.sample().await)
}

// ---------------------------------------------------------------------------
// Plugin — self-registers via inventory so lib.rs needs no per-service edits.
// ---------------------------------------------------------------------------

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("metrics")
        .invoke_handler(tauri::generate_handler![get_system_metrics])
        .setup(|app, _api| {
            app.manage(MetricsService::new());
            Ok(())
        })
        .build()
}

inventory::submit! {
    crate::registry::MantlePlugin {
        name: "metrics",
        build: || init::<tauri::Wry>(),
    }
}
