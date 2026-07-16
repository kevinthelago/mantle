use crate::layer_shell::manager::{ExclusiveZone, KeyboardMode, Layer, SurfaceConfig};
use crate::registry::MantlePlugin;

use super::LauncherState;

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

/// Apply layer-shell to the launcher window **without showing it**.
///
/// The launcher starts hidden and is summoned on demand via [`toggle_launcher`]
/// (bound to a global shortcut in `lib.rs`).  Layer-shell must be initialized
/// before the window is first mapped, so this runs at startup even though the
/// surface stays hidden until the first toggle — otherwise the launcher's
/// exclusive keyboard grab would steal all input the moment mantle starts.
///
/// Called from `lib.rs` setup on Linux after all plugins are wired.
pub fn setup_surface(win: &tauri::WebviewWindow) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        let gtk_win = win.gtk_window().map_err(|e| e.to_string())?;
        let config = launcher_surface_config();
        crate::layer_shell::LayerShellManager::apply_to_window(&gtk_win, &config, None);
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = win;
    }
    Ok(())
}

/// Toggle the launcher window's visibility.
///
/// Shows and focuses the launcher when hidden — its layer surface takes an
/// exclusive keyboard grab so the user can type immediately — and hides it when
/// visible.  Wired to a global shortcut in `lib.rs`; the frontend's Escape and
/// launch handlers hide the window via the `close_launcher` command.
pub fn toggle_launcher<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    use tauri::Manager;
    let Some(win) = app.get_webview_window("launcher") else {
        log::warn!("toggle_launcher: no launcher window");
        return;
    };
    if win.is_visible().unwrap_or(false) {
        let _ = win.hide();
    } else {
        let _ = win.show();
        let _ = win.set_focus();
    }
}
