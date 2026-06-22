use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::{collect_commands, Builder};

/// Typed error propagated across the bridge boundary.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BridgeError {
    pub code: String,
    pub message: String,
}

impl BridgeError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self { code: code.into(), message: message.into() }
    }
}

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for BridgeError {}

// ---------------------------------------------------------------------------
// Snapshot commands — return current state immediately.
// Convention: async fn foo(...) -> Result<T, BridgeError>
// ---------------------------------------------------------------------------

/// Health-check / smoke-test — useful for integration tests.
#[tauri::command]
#[specta::specta]
pub async fn ping(message: String) -> Result<String, BridgeError> {
    Ok(format!("pong: {message}"))
}

/// Returns the Mantle version string.
#[tauri::command]
#[specta::specta]
pub async fn get_version() -> Result<String, BridgeError> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

// ---------------------------------------------------------------------------
// Builder factory — called from lib.rs to wire invoke handler + export.
// ---------------------------------------------------------------------------

pub fn build_commands() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new().commands(collect_commands![ping, get_version])
}
