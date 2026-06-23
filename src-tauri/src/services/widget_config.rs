use serde::{Deserialize, Serialize};
use tauri::Emitter;

/// Mirrors `WidgetConfig` in src/widgets/types.ts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetConfig {
    pub id: String,
    pub anchor: String,
    pub visible: bool,
}

/// Mirrors `WidgetLayerConfig` in src/widgets/types.ts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetLayerConfig {
    pub widgets: Vec<WidgetConfig>,
}

impl Default for WidgetLayerConfig {
    fn default() -> Self {
        Self {
            widgets: vec![
                WidgetConfig {
                    id: "calendar".into(),
                    anchor: "top-right".into(),
                    visible: true,
                },
                WidgetConfig {
                    id: "system-monitor".into(),
                    anchor: "top-right".into(),
                    visible: true,
                },
                WidgetConfig {
                    id: "media-controls".into(),
                    anchor: "bottom-right".into(),
                    visible: true,
                },
            ],
        }
    }
}

#[tauri::command]
pub async fn get_widget_config() -> Result<WidgetLayerConfig, String> {
    Ok(WidgetLayerConfig::default())
}

/// Emit the toggle event so WidgetLayer can show/hide itself.
#[tauri::command]
pub async fn toggle_widget_layer<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<(), String> {
    app.emit("widget-layer:toggle", ())
        .map_err(|e| e.to_string())
}

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("widget-config")
        .invoke_handler(tauri::generate_handler![
            get_widget_config,
            toggle_widget_layer
        ])
        .build()
}

inventory::submit! {
    crate::registry::MantlePlugin {
        name: "widget-config",
        build: || init::<tauri::Wry>(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_three_widgets() {
        let cfg = WidgetLayerConfig::default();
        assert_eq!(cfg.widgets.len(), 3);
        assert_eq!(cfg.widgets[0].id, "calendar");
        assert_eq!(cfg.widgets[1].id, "system-monitor");
        assert_eq!(cfg.widgets[2].id, "media-controls");
    }

    #[test]
    fn all_default_widgets_are_visible() {
        assert!(WidgetLayerConfig::default()
            .widgets
            .iter()
            .all(|w| w.visible));
    }

    #[test]
    fn default_anchors_are_set() {
        let cfg = WidgetLayerConfig::default();
        assert_eq!(cfg.widgets[0].anchor, "top-right");
        assert_eq!(cfg.widgets[2].anchor, "bottom-right");
    }
}
