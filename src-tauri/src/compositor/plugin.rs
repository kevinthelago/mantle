/// Tauri plugin that initialises the compositor backend and wires commands.
///
/// Commands are defined here (alongside `generate_handler!`) to avoid
/// the cross-module `#[macro_export]` visibility issue that occurs when
/// Tauri 2's `generate_handler!` looks for its generated helper macros.
use tauri::{
    plugin::{Builder as PluginBuilder, TauriPlugin},
    Runtime, State,
};
use tracing::error;

use super::{
    action::CompositorAction, bridge::CompositorState, event::CompositorEvent,
    types::CompositorSnapshot,
};

// ── Commands ──────────────────────────────────────────────────────────────────

/// Fetch the current compositor snapshot (workspaces, windows, outputs).
#[tauri::command]
pub async fn compositor_snapshot(
    state: State<'_, CompositorState>,
) -> Result<CompositorSnapshot, String> {
    state
        .backend
        .read()
        .await
        .snapshot()
        .await
        .map_err(|e| e.to_string())
}

/// Dispatch a compositor action (focus workspace, exec, …).
#[tauri::command]
pub async fn compositor_dispatch(
    state: State<'_, CompositorState>,
    action: CompositorAction,
) -> Result<(), String> {
    state
        .backend
        .read()
        .await
        .dispatch(action)
        .await
        .map_err(|e| e.to_string())
}

/// Return the name of the active backend (`"sway"` or `"hyprland"`).
#[tauri::command]
pub async fn compositor_backend_name(state: State<'_, CompositorState>) -> Result<String, String> {
    Ok(state.backend.read().await.name().to_owned())
}

// ── Plugin factory ────────────────────────────────────────────────────────────

/// Build the compositor Tauri plugin.
///
/// Usage:
/// ```rust
/// tauri::Builder::default()
///     .plugin(compositor::plugin::init())
///     .run(tauri::generate_context!())
///     .unwrap();
/// ```
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    PluginBuilder::new("compositor")
        .invoke_handler(tauri::generate_handler![
            compositor_snapshot,
            compositor_dispatch,
            compositor_backend_name,
        ])
        .setup(|app, _| {
            let app_handle = app.clone();
            tauri::async_runtime::spawn(async move {
                #[cfg(unix)]
                {
                    use super::auto_detect::create_backend;
                    use tauri::Emitter;
                    match create_backend().await {
                        Ok(backend) => {
                            let state = CompositorState::new(backend);
                            state.spawn_event_relay(app_handle.clone());
                            app_handle.manage(state);
                        }
                        Err(e) => {
                            error!("compositor backend init failed: {e}");
                            let _ = app_handle
                                .emit("compositor://event", CompositorEvent::Disconnected);
                        }
                    }
                }
                #[cfg(not(unix))]
                {
                    use tauri::Emitter;
                    error!("compositor: not on a Unix system — sway/Hyprland unavailable");
                    let _ = app_handle.emit("compositor://event", CompositorEvent::Disconnected);
                }
            });
            Ok(())
        })
        .build()
}

// ── Registry self-registration ────────────────────────────────────────────────

// Register with the core MantlePlugin inventory — installed by registry::setup()
// without requiring changes to lib.rs.
inventory::submit! {
    crate::registry::MantlePlugin {
        name: "compositor",
        build: || init::<tauri::Wry>(),
    }
}

// ── TypeScript binding export (dev-only, called from a build script or test) ─

/// Generate TypeScript bindings for all compositor commands to `bindings_path`.
///
/// Called during development to keep `src/bindings/compositor.ts` in sync.
/// Core's build pipeline calls this for each sub-module and concatenates the results.
#[cfg(feature = "generate_bindings")]
pub fn export_bindings(bindings_path: &str) {
    use specta_typescript::{BigIntExportBehavior, Typescript};
    use tauri_specta::{collect_commands, Builder};

    Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            compositor_snapshot,
            compositor_dispatch,
            compositor_backend_name,
        ])
        .export(Typescript::default(), bindings_path)
        .expect("failed to export compositor TypeScript bindings");
}
