/// A self-registering Tauri plugin descriptor.
///
/// Other streams declare their plugins by calling `inventory::submit!` with this
/// struct. `setup()` in this module iterates all submitted plugins and installs
/// them into the Tauri builder — so `main.rs` and `lib.rs` never change per service.
///
/// # Example (in another crate/module):
/// ```rust
/// inventory::submit! {
///     MantlePlugin {
///         name: "my-service",
///         build: || my_service_plugin::init(),
///     }
/// }
/// ```
pub struct MantlePlugin {
    /// Human-readable name used for logging.
    pub name: &'static str,
    /// Factory that produces the Tauri plugin.
    pub build: fn() -> tauri::plugin::TauriPlugin<tauri::Wry>,
}

inventory::collect!(MantlePlugin);

/// Fold all registered plugins into a Tauri builder.
pub fn setup(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    inventory::iter::<MantlePlugin>.into_iter().fold(builder, |b, plugin| {
        log::info!("registering plugin: {}", plugin.name);
        b.plugin((plugin.build)())
    })
}
