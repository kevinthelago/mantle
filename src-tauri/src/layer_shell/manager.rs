#![allow(dead_code)] // All items used on Linux only; silence non-Linux dead-code warnings.

//! LayerShellManager — turns a GTK window into a wlr-layer-shell surface.
//!
//! Path A (preferred): Tauri creates the window with `visible: false`, then
//! `apply_to_window` is called on the `gtk::ApplicationWindow` obtained via
//! `WebviewWindow::gtk_window()` BEFORE `window.show()`.  The window must
//! never realize as an xdg-toplevel first — hence `visible: false` in
//! tauri.conf.json.
//!
//! Path B (fallback): drop Tauri for that surface; use gtk-rs + webkit2gtk +
//! gtk-layer-shell directly.  Choose only if Tauri's window lifecycle fights
//! the pre-map init in the spike (tracked in docs/adr/layer-shell.md).

/// Which wlr-layer-shell layer to use.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Layer {
    Background,
    Bottom,
    Top,
    Overlay,
}

/// How the surface handles keyboard events.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyboardMode {
    None,
    OnDemand,
    Exclusive,
}

/// How the exclusive zone is determined.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExclusiveZone {
    /// Reserve space automatically based on the surface size.
    Auto,
    /// Reserve exactly this many pixels.
    Fixed(i32),
    /// Do not reserve any space (overlapping surface).
    None,
}

/// All parameters for a single layer-shell surface.
#[derive(Debug, Clone)]
pub struct SurfaceConfig {
    pub layer: Layer,
    /// `[top, right, bottom, left]` — true = anchored to that edge.
    pub anchors: [bool; 4],
    pub exclusive_zone: ExclusiveZone,
    /// Margin in pixels: `(top, right, bottom, left)`.
    pub margins: (i32, i32, i32, i32),
    pub keyboard_mode: KeyboardMode,
    pub namespace: String,
}

impl SurfaceConfig {
    /// Preset for a full-width top bar of the given pixel height.
    pub fn top_bar(height: i32, namespace: impl Into<String>) -> Self {
        Self {
            layer: Layer::Top,
            anchors: [true, true, false, true], // top, right, bottom=false, left
            exclusive_zone: ExclusiveZone::Fixed(height),
            margins: (0, 0, 0, 0),
            keyboard_mode: KeyboardMode::None,
            namespace: namespace.into(),
        }
    }
}

/// Information about a physical monitor.
#[derive(Debug, Clone)]
pub struct MonitorInfo {
    pub index: u32,
    /// Human-readable model name.
    pub name: String,
    /// Connector name as reported by the compositor (e.g. "DP-1").
    pub connector: Option<String>,
}

pub struct LayerShellManager;

impl LayerShellManager {
    /// Apply layer-shell properties to `win`.
    ///
    /// **Must be called before `win.show()`** — the window must not have
    /// realized as an xdg-toplevel yet.
    ///
    /// On non-Linux platforms this is a compile-time no-op.
    #[cfg(target_os = "linux")]
    pub fn apply_to_window(
        win: &gtk::ApplicationWindow,
        config: &SurfaceConfig,
        monitor: Option<&gdk::Monitor>,
    ) {
        use gtk::prelude::*;
        use gtk_layer_shell::{Edge, KeyboardMode as GtkKb, Layer as GtkLayer};

        gtk_layer_shell::init_for_window(win);
        gtk_layer_shell::set_namespace(win, &config.namespace);

        gtk_layer_shell::set_layer(
            win,
            match config.layer {
                Layer::Background => GtkLayer::Background,
                Layer::Bottom => GtkLayer::Bottom,
                Layer::Top => GtkLayer::Top,
                Layer::Overlay => GtkLayer::Overlay,
            },
        );

        let edges = [Edge::Top, Edge::Right, Edge::Bottom, Edge::Left];
        for (edge, &anchored) in edges.iter().zip(config.anchors.iter()) {
            gtk_layer_shell::set_anchor(win, edge, anchored);
        }

        match config.exclusive_zone {
            ExclusiveZone::Auto => gtk_layer_shell::auto_exclusive_zone_enable(win),
            ExclusiveZone::Fixed(px) => gtk_layer_shell::set_exclusive_zone(win, px),
            ExclusiveZone::None => gtk_layer_shell::set_exclusive_zone(win, 0),
        }

        let (mt, mr, mb, ml) = config.margins;
        gtk_layer_shell::set_margin(win, Edge::Top, mt);
        gtk_layer_shell::set_margin(win, Edge::Right, mr);
        gtk_layer_shell::set_margin(win, Edge::Bottom, mb);
        gtk_layer_shell::set_margin(win, Edge::Left, ml);

        gtk_layer_shell::set_keyboard_mode(
            win,
            match config.keyboard_mode {
                KeyboardMode::None => GtkKb::None,
                KeyboardMode::OnDemand => GtkKb::OnDemand,
                KeyboardMode::Exclusive => GtkKb::Exclusive,
            },
        );

        if let Some(mon) = monitor {
            gtk_layer_shell::set_monitor(win, mon);
        }
    }

    /// Enumerate all connected GDK monitors.  Returns empty vec on non-Linux.
    pub fn list_monitors() -> Vec<MonitorInfo> {
        #[cfg(target_os = "linux")]
        {
            use gtk::prelude::*;
            if gtk::init().is_err() {
                return vec![];
            }
            let display = match gdk::Display::default() {
                Some(d) => d,
                None => return vec![],
            };
            (0..display.n_monitors())
                .filter_map(|i| display.monitor(i).map(|m| (i as u32, m)))
                .map(|(idx, mon)| MonitorInfo {
                    index: idx,
                    name: mon
                        .model()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| format!("monitor-{idx}")),
                    connector: mon.connector().map(|s| s.to_string()),
                })
                .collect()
        }
        #[cfg(not(target_os = "linux"))]
        {
            vec![]
        }
    }
}
