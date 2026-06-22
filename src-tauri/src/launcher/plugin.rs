use crate::layer_shell::manager::{ExclusiveZone, KeyboardMode, Layer, SurfaceConfig};
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
        layer: Layer::Overlay,
        // No edge anchoring — the window floats centred at its declared size.
        anchors: [false, false, false, false],
        exclusive_zone: ExclusiveZone::None,
        margins: (0, 0, 0, 0),
        keyboard_mode: KeyboardMode::Exclusive,
    }
}

/// Apply layer-shell and show the launcher window.
///
/// Called from `lib.rs` setup on Linux after all plugins are wired.
pub fn setup_surface(win: &tauri::WebviewWindow) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        let gtk_win = win.gtk_window().map_err(|e| e.to_string())?;
        let config = launcher_surface_config();
        crate::layer_shell::LayerShellManager::apply_to_window(&gtk_win, &config, None);
        win.show().map_err(|e| e.to_string())?;
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = win;
    }
    Ok(())
}
