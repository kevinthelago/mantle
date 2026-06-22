use crate::layer_shell::manager::{KeyboardHint, LayerHint, SurfaceConfig};
use crate::layer_shell::LayerShellManager;
use crate::registry::MantlePlugin;

use super::LauncherState;

/// Self-register the launcher as a plugin in the Mantle registry.
///
/// `inventory::submit!` runs at binary startup; `registry::setup` folds all
/// registered plugins into the Tauri builder before the app runs.
inventory::submit! {
    MantlePlugin {
        name: "launcher",
        build: init,
    }
}

pub fn init() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri::plugin::Builder::new("launcher")
        .invoke_handler(tauri::generate_handler![
            super::commands::search_apps,
            super::commands::launch_app,
            super::commands::run_command,
            super::commands::close_launcher,
            super::commands::get_history,
        ])
        .setup(|app, _| {
            use tauri::Manager;
            app.manage(LauncherState::new());
            Ok(())
        })
        .build()
}

/// Surface config for the app-launcher window: Overlay layer + exclusive keyboard.
pub fn launcher_surface_config() -> SurfaceConfig {
    SurfaceConfig {
        namespace: "mantle-launcher".to_string(),
        layer: LayerHint::Overlay,
        // No edge anchoring — the window floats centred at its declared size.
        anchors: vec![],
        exclusive_zone: 0,
        margins: Default::default(),
        keyboard_mode: KeyboardHint::Exclusive,
        output: None,
    }
}

/// Apply layer-shell and show the launcher window.
///
/// Called from `lib.rs` setup on Linux after all plugins are wired.
pub fn setup_surface(win: &tauri::WebviewWindow) -> Result<(), String> {
    LayerShellManager::new().apply(win, &launcher_surface_config())
}
