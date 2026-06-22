use super::{loader, schema::Config};
use notify::{
    Config as NotifyConfig, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, watch};

/// Spawn a background task that watches the config file for changes and pushes
/// updated `Config` values through the returned `watch::Receiver`.
///
/// The receiver's initial value is the config as of startup.
pub fn spawn(config_path: PathBuf, initial: Config) -> watch::Receiver<Config> {
    let (tx, rx) = watch::channel(initial);
    let tx = Arc::new(tx);

    tauri::async_runtime::spawn(async move {
        if let Err(e) = watch_loop(config_path, tx).await {
            log::error!("config watcher exited: {e:#}");
        }
    });

    rx
}

async fn watch_loop(path: PathBuf, tx: Arc<watch::Sender<Config>>) -> anyhow::Result<()> {
    // Bridge: notify uses std mpsc; we need a tokio-friendly channel.
    let (bridge_tx, mut bridge_rx) = mpsc::channel::<notify::Result<Event>>(32);

    // Spawn a dedicated OS thread for the blocking std::mpsc::Receiver.
    let (notify_tx, notify_rx) = std::sync::mpsc::channel::<notify::Result<Event>>();
    std::thread::spawn(move || {
        for event in notify_rx.into_iter() {
            if bridge_tx.blocking_send(event).is_err() {
                break;
            }
        }
    });

    let mut watcher = RecommendedWatcher::new(
        notify_tx,
        NotifyConfig::default().with_poll_interval(Duration::from_secs(2)),
    )?;

    // Watch the parent directory to catch atomic saves (write-to-tmp + rename).
    let watch_dir = path.parent().unwrap_or(&path);
    watcher.watch(watch_dir, RecursiveMode::NonRecursive)?;
    log::debug!("watching {} for config changes", path.display());

    while let Some(event_result) = bridge_rx.recv().await {
        match event_result {
            Ok(event) if is_write_to(&event, &path) => {
                // Debounce: wait for the editor's write to fully flush.
                tokio::time::sleep(Duration::from_millis(80)).await;
                match loader::reload(&path) {
                    Ok(new_cfg) => {
                        log::info!("config hot-reloaded from {}", path.display());
                        let _ = tx.send(new_cfg);
                    }
                    Err(e) => log::warn!("config reload failed (keeping previous): {e:#}"),
                }
            }
            Ok(_) => {}
            Err(e) => log::warn!("notify error: {e}"),
        }
    }

    Ok(())
}

fn is_write_to(event: &Event, target: &PathBuf) -> bool {
    matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_))
        && event.paths.iter().any(|p| p == target)
}
