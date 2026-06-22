use tracing::info;

use super::{
    backend::CompositorBackend, error::CompositorError, hyprland::HyprlandBackend,
    sway::SwayBackend,
};

/// Which compositor was detected from the environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Detected {
    Sway,
    Hyprland,
}

/// Detect the running compositor from environment variables.
///
/// - `SWAYSOCK` → sway is running.
/// - `HYPRLAND_INSTANCE_SIGNATURE` → Hyprland is running.
///
/// If both are set, sway takes precedence (unusual, but be explicit).
pub fn detect() -> Option<Detected> {
    if std::env::var_os("SWAYSOCK").is_some() {
        Some(Detected::Sway)
    } else if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
        Some(Detected::Hyprland)
    } else {
        None
    }
}

/// Create and connect the appropriate backend for the running compositor.
pub async fn create_backend() -> Result<Box<dyn CompositorBackend>, CompositorError> {
    match detect() {
        Some(Detected::Sway) => {
            info!("auto-detected sway (SWAYSOCK set)");
            Ok(Box::new(SwayBackend::connect().await?))
        }
        Some(Detected::Hyprland) => {
            info!("auto-detected Hyprland (HYPRLAND_INSTANCE_SIGNATURE set)");
            Ok(Box::new(HyprlandBackend::connect().await?))
        }
        None => Err(CompositorError::Unsupported),
    }
}

#[cfg(test)]
mod tests {
    // Detection logic depends on env vars; verified via integration tests that
    // set SWAYSOCK / HYPRLAND_INSTANCE_SIGNATURE before calling detect().
}
