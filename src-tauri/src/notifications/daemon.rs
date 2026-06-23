use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use base64::Engine as _;
use tokio::sync::{broadcast, RwLock};
use tokio::task::AbortHandle;
use zbus::zvariant::{OwnedValue, Value};
use zbus::{interface, SignalContext};

use super::sanitize::sanitize_body;
use super::types::*;

const MAX_ACTIVE: usize = 50;
const DEFAULT_TIMEOUT_LOW_MS: u64 = 3_000;
const DEFAULT_TIMEOUT_NORMAL_MS: u64 = 5_000;
// Critical notifications are persistent by default (None).

struct ActiveEntry {
    notification: Notification,
    expiry_handle: Option<AbortHandle>,
}

pub struct ServerState {
    active: RwLock<HashMap<u32, ActiveEntry>>,
    paused: RwLock<HashSet<u32>>,
    id_seq: AtomicU32,
    pub event_tx: broadcast::Sender<NotificationEvent>,
    /// Held so expiry tasks can reconstruct a SignalContext.
    connection: std::sync::OnceLock<zbus::Connection>,
}

impl ServerState {
    pub fn new() -> (Arc<Self>, broadcast::Receiver<NotificationEvent>) {
        let (tx, rx) = broadcast::channel(256);
        let state = Arc::new(Self {
            active: RwLock::new(HashMap::new()),
            paused: RwLock::new(HashSet::new()),
            id_seq: AtomicU32::new(1),
            event_tx: tx,
            connection: std::sync::OnceLock::new(),
        });
        (state, rx)
    }

    pub fn set_connection(&self, conn: zbus::Connection) {
        let _ = self.connection.set(conn);
    }

    pub fn connection_ref(&self) -> Option<&zbus::Connection> {
        self.connection.get()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<NotificationEvent> {
        self.event_tx.subscribe()
    }

    pub async fn pause_expiry(&self, id: u32) {
        self.paused.write().await.insert(id);
    }

    pub async fn resume_expiry(&self, id: u32) {
        self.paused.write().await.remove(&id);
    }

    /// Returns snapshot of all active notifications for the bridge.
    pub async fn active_list(&self) -> Vec<Notification> {
        self.active
            .read()
            .await
            .values()
            .map(|e| e.notification.clone())
            .collect()
    }

    /// Close a notification from within an expiry task (no SignalContext available directly).
    pub async fn close_from_task(&self, id: u32, reason: ClosedReason) {
        let removed = {
            let mut active = self.active.write().await;
            active.remove(&id).map(|e| {
                if let Some(h) = e.expiry_handle {
                    h.abort();
                }
            })
        };
        if removed.is_some() {
            if let Some(conn) = self.connection.get() {
                let ctx =
                    SignalContext::new(conn, "/org/freedesktop/Notifications").expect("valid path");
                let _ = NotificationsServer::notification_closed(&ctx, id, reason as u32).await;
            }
            let _ = self.event_tx.send(NotificationEvent::Closed { id, reason });
        }
    }
}

pub struct NotificationsServer {
    pub state: Arc<ServerState>,
}

#[interface(name = "org.freedesktop.Notifications")]
impl NotificationsServer {
    async fn notify(
        &mut self,
        app_name: &str,
        replaces_id: u32,
        app_icon: &str,
        summary: &str,
        body: &str,
        actions: Vec<String>,
        hints: HashMap<String, OwnedValue>,
        expire_timeout: i32,
        #[zbus(signal_context)] _ctx: SignalContext<'_>,
    ) -> zbus::fdo::Result<u32> {
        // Rate-limit: drop if too many active notifications.
        {
            let active = self.state.active.read().await;
            if active.len() >= MAX_ACTIVE
                && (replaces_id == 0 || !active.contains_key(&replaces_id))
            {
                return Err(zbus::fdo::Error::Failed("notification queue full".into()));
            }
        }

        let urgency = decode_urgency(&hints);
        let expire_timeout_ms = resolve_timeout(expire_timeout, urgency);

        let notification = Notification {
            id: 0, // set below
            app_name: app_name.to_owned(),
            app_icon: app_icon.to_owned(),
            summary: summary.to_owned(),
            body: sanitize_body(body),
            actions,
            urgency,
            category: decode_str_hint(&hints, "category"),
            desktop_entry: decode_str_hint(&hints, "desktop-entry"),
            image: decode_image_data(&hints),
            image_path: decode_str_hint(&hints, "image-path")
                .or_else(|| decode_str_hint(&hints, "image_path")),
            resident: decode_bool_hint(&hints, "resident"),
            transient: decode_bool_hint(&hints, "transient"),
            expire_timeout_ms,
            created_at: unix_now(),
        };

        let id = if replaces_id > 0 {
            replaces_id
        } else {
            self.state.id_seq.fetch_add(1, Ordering::SeqCst)
        };

        let notification = Notification { id, ..notification };

        // Cancel any existing expiry for replaced notification.
        let was_replacing = {
            let mut active = self.state.active.write().await;
            if let Some(old) = active.remove(&id) {
                if let Some(h) = old.expiry_handle {
                    h.abort();
                }
                true
            } else {
                false
            }
        };

        // Spawn expiry task if needed.
        let expiry_handle = expire_timeout_ms.map(|ms| {
            let state = self.state.clone();
            tokio::spawn(async move {
                expiry_task(state, id, ms).await;
            })
            .abort_handle()
        });

        {
            let mut active = self.state.active.write().await;
            active.insert(
                id,
                ActiveEntry {
                    notification: notification.clone(),
                    expiry_handle,
                },
            );
        }

        let event = if was_replacing {
            NotificationEvent::Replaced {
                id,
                notification: notification.clone(),
            }
        } else {
            NotificationEvent::Added(notification.clone())
        };
        let _ = self.state.event_tx.send(event);

        Ok(id)
    }

    async fn close_notification(
        &mut self,
        id: u32,
        #[zbus(signal_context)] ctx: SignalContext<'_>,
    ) -> zbus::fdo::Result<()> {
        let removed = {
            let mut active = self.state.active.write().await;
            active.remove(&id).map(|e| {
                if let Some(h) = e.expiry_handle {
                    h.abort();
                }
            })
        };
        if removed.is_some() {
            let _ =
                Self::notification_closed(&ctx, id, ClosedReason::CloseNotification as u32).await;
            let _ = self.state.event_tx.send(NotificationEvent::Closed {
                id,
                reason: ClosedReason::CloseNotification,
            });
        }
        Ok(())
    }

    fn get_capabilities(&self) -> Vec<String> {
        vec![
            "actions".into(),
            "body".into(),
            "body-markup".into(),
            "icon-static".into(),
            "persistence".into(),
        ]
    }

    fn get_server_information(&self) -> (&str, &str, &str, &str) {
        // (name, vendor, version, spec_version)
        ("mantle", "mantle", "0.1.0", "1.2")
    }

    #[zbus(signal)]
    async fn notification_closed(ctx: &SignalContext<'_>, id: u32, reason: u32)
        -> zbus::Result<()>;

    #[zbus(signal)]
    pub async fn action_invoked(
        ctx: &SignalContext<'_>,
        id: u32,
        action_key: &str,
    ) -> zbus::Result<()>;
}

// ── expiry task ─────────────────────────────────────────────────────────────

async fn expiry_task(state: Arc<ServerState>, id: u32, total_ms: u64) {
    let tick = Duration::from_millis(50);
    let mut remaining = total_ms;

    loop {
        if remaining == 0 {
            break;
        }
        let paused = state.paused.read().await.contains(&id);
        if paused {
            tokio::time::sleep(tick).await;
            continue;
        }
        let sleep_ms = remaining.min(50);
        tokio::time::sleep(Duration::from_millis(sleep_ms)).await;
        remaining = remaining.saturating_sub(sleep_ms);
    }

    // Check notification is still alive before closing (it may have been replaced).
    let still_active = state.active.read().await.contains_key(&id);
    if still_active {
        state.close_from_task(id, ClosedReason::Expired).await;
    }
}

// ── hint decoders ────────────────────────────────────────────────────────────

fn decode_urgency(hints: &HashMap<String, OwnedValue>) -> Urgency {
    hints
        .get("urgency")
        .and_then(|v| match v.deref() {
            Value::U8(0) => Some(Urgency::Low),
            Value::U8(1) => Some(Urgency::Normal),
            Value::U8(2) => Some(Urgency::Critical),
            _ => None,
        })
        .unwrap_or(Urgency::Normal)
}

fn decode_str_hint(hints: &HashMap<String, OwnedValue>, key: &str) -> Option<String> {
    hints.get(key).and_then(|v| match v.deref() {
        Value::Str(s) => Some(s.to_string()),
        _ => None,
    })
}

fn decode_bool_hint(hints: &HashMap<String, OwnedValue>, key: &str) -> bool {
    hints
        .get(key)
        .and_then(|v| match v.deref() {
            Value::Bool(b) => Some(*b),
            _ => None,
        })
        .unwrap_or(false)
}

/// Decode `image-data` hint: (iiibiiay) → PNG → base64.
fn decode_image_data(hints: &HashMap<String, OwnedValue>) -> Option<NotificationImage> {
    let raw = hints
        .get("image-data")
        .or_else(|| hints.get("image_data"))
        .or_else(|| hints.get("icon_data"))?;

    let structure = match raw.deref() {
        Value::Structure(s) => s,
        _ => return None,
    };

    let fields = structure.fields();
    if fields.len() < 7 {
        return None;
    }

    let width = match &fields[0] {
        Value::I32(v) => *v,
        _ => return None,
    };
    let height = match &fields[1] {
        Value::I32(v) => *v,
        _ => return None,
    };
    let rowstride = match &fields[2] {
        Value::I32(v) => *v,
        _ => return None,
    };
    let has_alpha = match &fields[3] {
        Value::Bool(v) => *v,
        _ => return None,
    };
    let bits_per_sample = match &fields[4] {
        Value::I32(v) => *v,
        _ => return None,
    };
    let channels = match &fields[5] {
        Value::I32(v) => *v,
        _ => return None,
    };
    let pixel_bytes: Vec<u8> = match &fields[6] {
        Value::Array(arr) => arr
            .iter()
            .filter_map(|v| if let Value::U8(b) = v { Some(*b) } else { None })
            .collect(),
        _ => return None,
    };

    // Encode raw pixels as PNG for transport.
    let png_bytes = encode_raw_to_png(
        &pixel_bytes,
        width as u32,
        height as u32,
        rowstride as u32,
        has_alpha,
        bits_per_sample as u32,
        channels as u32,
    )?;

    Some(NotificationImage {
        width,
        height,
        rowstride,
        has_alpha,
        bits_per_sample,
        channels,
        data_b64: base64::engine::general_purpose::STANDARD.encode(&png_bytes),
    })
}

fn encode_raw_to_png(
    pixels: &[u8],
    width: u32,
    height: u32,
    rowstride: u32,
    has_alpha: bool,
    _bits: u32,
    channels: u32,
) -> Option<Vec<u8>> {
    use image::{ExtendedColorType, ImageEncoder};

    let color_type = match (channels, has_alpha) {
        (4, true) => ExtendedColorType::Rgba8,
        (3, false) => ExtendedColorType::Rgb8,
        _ => return None,
    };

    // Re-pack rows using rowstride (may have padding).
    let row_bytes = (width * channels) as usize;
    let mut packed = Vec::with_capacity(row_bytes * height as usize);
    for row in 0..height as usize {
        let start = row * rowstride as usize;
        let end = start + row_bytes;
        if end > pixels.len() {
            return None;
        }
        packed.extend_from_slice(&pixels[start..end]);
    }

    let mut png_buf = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut png_buf);
    encoder
        .write_image(&packed, width, height, color_type)
        .ok()?;
    Some(png_buf)
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn resolve_timeout(expire_timeout: i32, urgency: Urgency) -> Option<u64> {
    match expire_timeout {
        0 => None,
        -1 => match urgency {
            Urgency::Critical => None,
            Urgency::Normal => Some(DEFAULT_TIMEOUT_NORMAL_MS),
            Urgency::Low => Some(DEFAULT_TIMEOUT_LOW_MS),
        },
        n if n > 0 => Some(n as u64),
        _ => Some(DEFAULT_TIMEOUT_NORMAL_MS),
    }
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// OwnedValue derefs to Value<'_> in zbus 4.
use std::ops::Deref;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_timeout_zero_means_persistent() {
        assert_eq!(resolve_timeout(0, Urgency::Normal), None);
        assert_eq!(resolve_timeout(0, Urgency::Critical), None);
    }

    #[test]
    fn resolve_timeout_negative_one_uses_urgency_defaults() {
        assert_eq!(resolve_timeout(-1, Urgency::Critical), None);
        assert_eq!(
            resolve_timeout(-1, Urgency::Normal),
            Some(DEFAULT_TIMEOUT_NORMAL_MS),
        );
        assert_eq!(
            resolve_timeout(-1, Urgency::Low),
            Some(DEFAULT_TIMEOUT_LOW_MS),
        );
    }

    #[test]
    fn resolve_timeout_positive_value_is_passed_through() {
        assert_eq!(resolve_timeout(3000, Urgency::Normal), Some(3000));
        assert_eq!(resolve_timeout(1, Urgency::Critical), Some(1));
    }

    #[test]
    fn resolve_timeout_negative_other_falls_back_to_normal() {
        // Any negative value other than -1 (spec violation) → normal default.
        assert_eq!(
            resolve_timeout(-5, Urgency::Low),
            Some(DEFAULT_TIMEOUT_NORMAL_MS),
        );
    }

    #[test]
    fn unix_now_is_nonzero() {
        assert!(unix_now() > 0);
    }
}
