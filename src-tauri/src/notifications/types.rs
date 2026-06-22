use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Urgency {
    Low = 0,
    Normal = 1,
    Critical = 2,
}

impl Default for Urgency {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationImage {
    pub width: i32,
    pub height: i32,
    pub rowstride: i32,
    pub has_alpha: bool,
    pub bits_per_sample: i32,
    pub channels: i32,
    /// PNG-encoded bytes, base64 for JSON serialization
    pub data_b64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: u32,
    pub app_name: String,
    pub app_icon: String,
    pub summary: String,
    /// Sanitized body (FDO markup subset only)
    pub body: String,
    /// Alternating [key, label, key, label, …] pairs
    pub actions: Vec<String>,
    pub urgency: Urgency,
    pub category: Option<String>,
    pub desktop_entry: Option<String>,
    pub image: Option<NotificationImage>,
    pub image_path: Option<String>,
    pub resident: bool,
    pub transient: bool,
    /// None → persistent; Some(ms) → auto-close after ms
    pub expire_timeout_ms: Option<u64>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClosedReason {
    Expired = 1,
    Dismissed = 2,
    CloseNotification = 3,
    Undefined = 4,
}

/// Events broadcast to connected frontends via Tauri event bus.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum NotificationEvent {
    Added(Notification),
    Replaced { id: u32, notification: Notification },
    Closed { id: u32, reason: ClosedReason },
    ActionInvoked { id: u32, action_key: String },
    DndChanged { enabled: bool },
}
