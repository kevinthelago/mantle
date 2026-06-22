pub mod action;
pub mod backend;
pub mod bridge;
pub mod error;
pub mod event;
pub mod plugin;
pub mod types;

// These modules use Unix domain sockets and only compile on Unix targets.
#[cfg(unix)]
pub mod auto_detect;
#[cfg(unix)]
pub mod hyprland;
#[cfg(unix)]
pub mod supervisor;
#[cfg(unix)]
pub mod sway;

#[cfg(unix)]
pub use auto_detect::create_backend;
pub use backend::CompositorBackend;
pub use bridge::CompositorState;
pub use error::CompositorError;
pub use event::CompositorEvent;
pub use types::{CompositorSnapshot, Output, Window, Workspace};
