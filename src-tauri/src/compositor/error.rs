use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompositorError {
    #[cfg(unix)]
    #[error("sway IPC error: {0}")]
    SwayIpc(#[from] swayipc::Error),

    #[error("Hyprland IPC error: {0}")]
    HyprlandIpc(String),

    #[error("compositor connection lost")]
    Disconnected,

    #[error("no supported compositor detected (set SWAYSOCK or HYPRLAND_INSTANCE_SIGNATURE)")]
    Unsupported,

    #[error("background task panicked")]
    TaskPanicked,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Lets `CompositorError` be used as the `Err` variant in Tauri commands
/// (`Result<T, String>` is what Tauri serializes to the frontend).
impl From<CompositorError> for String {
    fn from(e: CompositorError) -> Self {
        e.to_string()
    }
}
