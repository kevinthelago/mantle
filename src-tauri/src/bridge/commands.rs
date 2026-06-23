use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{Emitter, State};
use tauri_specta::{collect_commands, Builder};

use crate::config::{
    schema::{ClockConfig, Config, FocusedWindow, OutputConfig, PowerAction, WorkspaceState},
    ConfigState,
};

/// Typed error propagated across the bridge boundary.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BridgeError {
    pub code: String,
    pub message: String,
}

impl BridgeError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for BridgeError {}

impl From<anyhow::Error> for BridgeError {
    fn from(e: anyhow::Error) -> Self {
        BridgeError::new("internal", e.to_string())
    }
}

// ── Core commands ─────────────────────────────────────────────────────────────

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

// ── Config commands (bar-shell stream) ───────────────────────────────────────

#[tauri::command]
#[specta::specta]
pub async fn get_config(state: State<'_, ConfigState>) -> Result<Config, BridgeError> {
    Ok(state.get().await)
}

#[tauri::command]
#[specta::specta]
pub async fn get_output_config(
    output: String,
    state: State<'_, ConfigState>,
) -> Result<OutputConfig, BridgeError> {
    let cfg = state.get().await;
    Ok(OutputConfig::resolve(&cfg.outputs, &output).clone())
}

#[tauri::command]
#[specta::specta]
pub async fn get_clock_config(state: State<'_, ConfigState>) -> Result<ClockConfig, BridgeError> {
    Ok(state.get().await.clock)
}

/// Return the current time formatted per the user's format string.
#[tauri::command]
#[specta::specta]
pub async fn clock_tick(state: State<'_, ConfigState>) -> Result<String, BridgeError> {
    let cfg = state.get().await;
    Ok(format_time(&cfg.clock.format))
}

fn format_time(fmt: &str) -> String {
    let utc_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format_time_at(fmt, utc_secs)
}

fn format_time_at(fmt: &str, utc_secs: u64) -> String {
    let secs = utc_secs % 86400;
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    fmt.replace("%H", &format!("{h:02}"))
        .replace("%M", &format!("{m:02}"))
        .replace("%S", &format!("{s:02}"))
        .replace(
            "%I",
            &format!("{:02}", if h % 12 == 0 { 12 } else { h % 12 }),
        )
        .replace("%p", if h < 12 { "AM" } else { "PM" })
}

#[cfg(test)]
mod tests {
    use super::*;

    // 14:30:00 UTC = 52 200 seconds into the day.
    const SAMPLE: u64 = 14 * 3600 + 30 * 60;

    #[test]
    fn format_24h_hm() {
        assert_eq!(format_time_at("%H:%M", SAMPLE), "14:30");
    }

    #[test]
    fn format_24h_hms() {
        assert_eq!(format_time_at("%H:%M:%S", SAMPLE), "14:30:00");
    }

    #[test]
    fn format_12h_pm() {
        assert_eq!(format_time_at("%I:%M %p", SAMPLE), "02:30 PM");
    }

    #[test]
    fn format_midnight_12h() {
        assert_eq!(format_time_at("%I:%M %p", 0), "12:00 AM");
    }

    #[test]
    fn format_noon_12h() {
        // Noon = 12 * 3600
        assert_eq!(format_time_at("%I:%M %p", 43200), "12:00 PM");
    }

    #[test]
    fn format_no_placeholders() {
        assert_eq!(format_time_at("Mantle", SAMPLE), "Mantle");
    }

    #[test]
    fn build_commands_compiles() {
        let _ = build_commands();
    }
}

#[tauri::command]
#[specta::specta]
pub async fn reload_config(
    app: tauri::AppHandle,
    _state: State<'_, ConfigState>,
) -> Result<Config, BridgeError> {
    let path = crate::config::loader::config_path();
    let new_cfg = crate::config::loader::reload(&path)
        .map_err(|e| BridgeError::new("config-reload", e.to_string()))?;
    app.emit("config-changed", &new_cfg).ok();
    Ok(new_cfg)
}

// ── Power menu (bar-shell stream) ─────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
pub async fn power_action(action: PowerAction) -> Result<(), BridgeError> {
    crate::services::power::execute(&action)
        .await
        .map_err(|e| BridgeError::new("power-action", e.to_string()))
}

// ── Compositor commands ───────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
pub async fn get_workspace_state() -> Result<WorkspaceState, BridgeError> {
    #[cfg(target_os = "linux")]
    {
        return crate::compositor::query_workspace_state()
            .await
            .map_err(|e| BridgeError::new("compositor", e.to_string()));
    }
    #[cfg(not(target_os = "linux"))]
    Ok(WorkspaceState::default())
}

#[tauri::command]
#[specta::specta]
pub async fn get_focused_window() -> Result<FocusedWindow, BridgeError> {
    #[cfg(target_os = "linux")]
    {
        return crate::compositor::query_focused_window()
            .await
            .map_err(|e| BridgeError::new("compositor", e.to_string()));
    }
    #[cfg(not(target_os = "linux"))]
    Ok(FocusedWindow::default())
}

#[tauri::command]
#[specta::specta]
pub async fn compositor_dispatch(command: String) -> Result<(), BridgeError> {
    #[cfg(target_os = "linux")]
    {
        return crate::compositor::dispatch_command(&command)
            .await
            .map_err(|e| BridgeError::new("compositor", e.to_string()));
    }
    #[cfg(not(target_os = "linux"))]
    {
        drop(command);
        Ok(())
    }
}

// ── Builder factory ───────────────────────────────────────────────────────────

pub fn build_commands() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new().commands(collect_commands![
        ping,
        get_version,
        // Config
        get_config,
        get_output_config,
        get_clock_config,
        clock_tick,
        reload_config,
        // Power
        power_action,
        // Compositor stubs
        get_workspace_state,
        get_focused_window,
        compositor_dispatch,
        // Styling loader
        crate::load_styling
    ])
}
