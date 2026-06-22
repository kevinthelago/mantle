//! Layer-shell surface manager — Path A implementation.
//!
//! Applies gtk-layer-shell to a Tauri WebviewWindow *before* it is shown.
//! The window must be created with `visible: false` in tauri.conf.json so it
//! does not realize as an xdg-toplevel before we claim it as a layer surface.
//!
//! # Decision record
//! Path A (Tauri + gtk-layer-shell) was chosen over Path B (bare gtk-rs).
//! Tauri's `WebviewWindow::gtk_window()` is available in the `setup` hook
//! before `win.show()` is called, giving us the correct pre-map timing window.
//! If Tauri's window lifecycle ever fights this assumption, fall back to Path B.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Platform-agnostic surface config types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayerHint {
    Background,
    Bottom,
    Top,
    Overlay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnchorEdge {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyboardHint {
    /// Bar surfaces — keyboard focus is never captured.
    None,
    /// Launcher / popups — focus on demand.
    OnDemand,
    /// Full-screen overlays — exclusive keyboard grab.
    Exclusive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Margins {
    pub top: i32,
    pub bottom: i32,
    pub left: i32,
    pub right: i32,
}

impl Default for Margins {
    fn default() -> Self {
        Self {
            top: 0,
            bottom: 0,
            left: 0,
            right: 0,
        }
    }
}

/// Complete configuration for a single layer-shell surface.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceConfig {
    /// wlr-layer-shell namespace (visible to compositor; use reverse-DNS style).
    pub namespace: String,
    pub layer: LayerHint,
    pub anchors: Vec<AnchorEdge>,
    /// Exclusive zone in pixels. -1 = auto (computed from surface size).
    pub exclusive_zone: i32,
    pub margins: Margins,
    pub keyboard_mode: KeyboardHint,
    /// GDK monitor name to pin to. None = follow the first available output.
    pub output: Option<String>,
}

impl SurfaceConfig {
    /// Default config for the main bar: top-anchored, full-width, auto exclusive zone.
    pub fn default_bar() -> Self {
        Self {
            namespace: "mantle-bar".to_string(),
            layer: LayerHint::Top,
            anchors: vec![AnchorEdge::Top, AnchorEdge::Left, AnchorEdge::Right],
            exclusive_zone: -1,
            margins: Margins::default(),
            keyboard_mode: KeyboardHint::None,
            output: None,
        }
    }
}

// ---------------------------------------------------------------------------
// LayerShellManager
// ---------------------------------------------------------------------------

pub struct LayerShellManager;

impl LayerShellManager {
    pub fn new() -> Self {
        Self
    }

    /// Apply layer-shell properties to `win` and then show it.
    ///
    /// Must be called from Tauri's `setup` hook before the window is visible.
    /// On non-Linux platforms this is a no-op (returns Ok, window stays hidden).
    pub fn apply(&self, win: &tauri::WebviewWindow, config: &SurfaceConfig) -> Result<(), String> {
        #[cfg(target_os = "linux")]
        return self.apply_linux(win, config);

        #[cfg(not(target_os = "linux"))]
        {
            let _ = (win, config);
            log::warn!("layer-shell is only supported on Linux; skipping apply()");
            Ok(())
        }
    }

    #[cfg(target_os = "linux")]
    fn apply_linux(
        &self,
        win: &tauri::WebviewWindow,
        config: &SurfaceConfig,
    ) -> Result<(), String> {
        use gtk::prelude::*;
        use gtk_layer_shell::Edge;

        let gtk_win = win.gtk_window().map_err(|e| e.to_string())?;

        // --- Init layer shell (must happen before show) ---
        gtk_layer_shell::init_for_window(&gtk_win);
        gtk_layer_shell::set_namespace(&gtk_win, &config.namespace);
        gtk_layer_shell::set_layer(&gtk_win, self.layer_hint_to_gtk(&config.layer));

        // --- Anchors ---
        for edge in &config.anchors {
            gtk_layer_shell::set_anchor(&gtk_win, self.edge_to_gtk(edge), true);
        }

        // --- Exclusive zone ---
        if config.exclusive_zone == -1 {
            gtk_layer_shell::auto_exclusive_zone_enable(&gtk_win);
        } else {
            gtk_layer_shell::set_exclusive_zone(&gtk_win, config.exclusive_zone);
        }

        // --- Margins ---
        gtk_layer_shell::set_margin(&gtk_win, Edge::Top, config.margins.top);
        gtk_layer_shell::set_margin(&gtk_win, Edge::Bottom, config.margins.bottom);
        gtk_layer_shell::set_margin(&gtk_win, Edge::Left, config.margins.left);
        gtk_layer_shell::set_margin(&gtk_win, Edge::Right, config.margins.right);

        // --- Keyboard mode ---
        gtk_layer_shell::set_keyboard_mode(
            &gtk_win,
            self.keyboard_hint_to_gtk(&config.keyboard_mode),
        );

        // --- Output pin ---
        if let Some(ref output_name) = config.output {
            if let Some(display) = gdk::Display::default() {
                let n = display.n_monitors();
                for i in 0..n {
                    if let Some(monitor) = display.monitor(i) {
                        if monitor.model().as_deref() == Some(output_name.as_str()) {
                            gtk_layer_shell::set_monitor(&gtk_win, &monitor);
                            break;
                        }
                    }
                }
            }
        }

        // --- Show — at this point the surface is a layer surface, not xdg-toplevel ---
        win.show().map_err(|e| e.to_string())?;

        log::info!(
            "layer-shell applied: namespace={} layer={:?}",
            config.namespace,
            config.layer,
        );

        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn layer_hint_to_gtk(&self, hint: &LayerHint) -> gtk_layer_shell::Layer {
        use gtk_layer_shell::Layer;
        match hint {
            LayerHint::Background => Layer::Background,
            LayerHint::Bottom => Layer::Bottom,
            LayerHint::Top => Layer::Top,
            LayerHint::Overlay => Layer::Overlay,
        }
    }

    #[cfg(target_os = "linux")]
    fn edge_to_gtk(&self, edge: &AnchorEdge) -> gtk_layer_shell::Edge {
        use gtk_layer_shell::Edge;
        match edge {
            AnchorEdge::Top => Edge::Top,
            AnchorEdge::Bottom => Edge::Bottom,
            AnchorEdge::Left => Edge::Left,
            AnchorEdge::Right => Edge::Right,
        }
    }

    #[cfg(target_os = "linux")]
    fn keyboard_hint_to_gtk(&self, hint: &KeyboardHint) -> gtk_layer_shell::KeyboardMode {
        use gtk_layer_shell::KeyboardMode;
        match hint {
            KeyboardHint::None => KeyboardMode::None,
            KeyboardHint::OnDemand => KeyboardMode::OnDemand,
            KeyboardHint::Exclusive => KeyboardMode::Exclusive,
        }
    }

    /// Enumerate GDK monitors and return their model names.
    /// Use for multi-monitor setup and hotplug-driven output selection.
    #[cfg(target_os = "linux")]
    pub fn list_monitors(&self) -> Vec<String> {
        use gtk::prelude::*;
        let display = match gdk::Display::default() {
            Some(d) => d,
            None => return vec![],
        };
        (0..display.n_monitors())
            .filter_map(|i| display.monitor(i))
            .filter_map(|m| m.model())
            .map(|s| s.to_string())
            .collect()
    }

    #[cfg(not(target_os = "linux"))]
    pub fn list_monitors(&self) -> Vec<String> {
        vec![]
    }
}

impl Default for LayerShellManager {
    fn default() -> Self {
        Self::new()
    }
}
