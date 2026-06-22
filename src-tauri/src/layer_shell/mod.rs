pub mod manager;

// Items are used under #[cfg(target_os = "linux")] only.
#[allow(unused_imports)]
pub use manager::{ExclusiveZone, KeyboardMode, Layer, LayerShellManager, SurfaceConfig};
