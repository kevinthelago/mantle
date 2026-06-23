use serde::{Deserialize, Serialize};
use specta::Type;

// ── Top-level ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct Config {
    pub general: GeneralConfig,
    /// One entry per physical output; the wildcard `"*"` matches all outputs
    /// not covered by a more specific entry.
    pub outputs: Vec<OutputConfig>,
    pub clock: ClockConfig,
    pub workspaces: WorkspacesConfig,
    #[serde(rename = "power_menu")]
    pub power_menu: PowerMenuConfig,
    pub notifications: NotificationsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            outputs: vec![OutputConfig::default()],
            clock: ClockConfig::default(),
            workspaces: WorkspacesConfig::default(),
            power_menu: PowerMenuConfig::default(),
            notifications: NotificationsConfig::default(),
        }
    }
}

// ── General ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct GeneralConfig {
    /// Gap in pixels between bar edge and screen edge.
    pub gap: i32,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self { gap: 0 }
    }
}

// ── Per-output bar config ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct OutputConfig {
    /// Output name (e.g. "DP-1", "HDMI-A-1") or "*" to match all.
    pub name: String,
    pub height: u32,
    pub layer: BarLayer,
    /// Whether to reserve space (exclusive zone) so windows don't overlap the bar.
    pub exclusive: bool,
    pub opacity: f64,
    pub keyboard: KeyboardMode,
    pub left: RegionConfig,
    pub center: RegionConfig,
    pub right: RegionConfig,
    pub theme: ThemeConfig,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            name: "*".into(),
            height: 36,
            layer: BarLayer::Top,
            exclusive: true,
            opacity: 0.95,
            keyboard: KeyboardMode::None,
            left: RegionConfig {
                modules: vec!["workspaces".into()],
            },
            center: RegionConfig {
                modules: vec!["clock".into()],
            },
            right: RegionConfig {
                modules: vec!["power-menu".into()],
            },
            theme: ThemeConfig::default(),
        }
    }
}

impl OutputConfig {
    /// Returns the config entry that best matches `output_name`.
    /// Prefers an exact name match over the wildcard.
    pub fn resolve<'a>(configs: &'a [OutputConfig], output_name: &str) -> &'a OutputConfig {
        configs
            .iter()
            .find(|c| c.name == output_name)
            .or_else(|| configs.iter().find(|c| c.name == "*"))
            .unwrap_or_else(|| &configs[0])
    }
}

// ── Module region ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default, Type)]
pub struct RegionConfig {
    pub modules: Vec<String>,
}

// ── Theme ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct ThemeConfig {
    pub background: String,
    pub foreground: String,
    pub border_radius: u32,
    pub font: String,
    pub font_size: u32,
    pub module_background: String,
    pub module_padding: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            background: "#1e1e2e".into(),
            foreground: "#cdd6f4".into(),
            border_radius: 8,
            font: "sans-serif".into(),
            font_size: 13,
            module_background: "transparent".into(),
            module_padding: "0 8px".into(),
        }
    }
}

// ── Enums ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Type)]
#[serde(rename_all = "lowercase")]
pub enum BarLayer {
    Background,
    Bottom,
    #[default]
    Top,
    Overlay,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Type)]
#[serde(rename_all = "lowercase")]
pub enum KeyboardMode {
    #[default]
    None,
    OnDemand,
    Exclusive,
}

// ── Module configs ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct ClockConfig {
    pub format: String,
    /// Update interval in milliseconds.
    pub interval: u64,
    pub timezone: Option<String>,
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            format: "%H:%M".into(),
            interval: 1000,
            timezone: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct WorkspacesConfig {
    pub show_empty: bool,
    pub show_names: bool,
    pub show_icons: bool,
    pub persistent: Vec<String>,
}

impl Default for WorkspacesConfig {
    fn default() -> Self {
        Self {
            show_empty: false,
            show_names: true,
            show_icons: false,
            persistent: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct PowerMenuConfig {
    pub actions: Vec<PowerAction>,
}

impl Default for PowerMenuConfig {
    fn default() -> Self {
        Self {
            actions: vec![
                PowerAction::Lock,
                PowerAction::Logout,
                PowerAction::Suspend,
                PowerAction::Hibernate,
                PowerAction::Reboot,
                PowerAction::Poweroff,
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
#[serde(rename_all = "lowercase")]
pub enum PowerAction {
    Lock,
    Logout,
    Suspend,
    Hibernate,
    HybridSleep,
    Reboot,
    Poweroff,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct NotificationsConfig {
    pub position: NotificationPosition,
    pub max_visible: u32,
    /// Dismiss timeout in milliseconds; 0 = never auto-dismiss.
    pub timeout: u64,
}

impl Default for NotificationsConfig {
    fn default() -> Self {
        Self {
            position: NotificationPosition::TopRight,
            max_visible: 5,
            timeout: 5000,
        }
    }
}

// ── Compositor event payloads (contract with compositor stream) ───────────────
// These types are stubs — the compositor stream owns the implementations that
// populate them.  Defined here so the bridge can generate typed TS bindings.

#[derive(Debug, Clone, Serialize, Deserialize, Default, Type)]
pub struct Workspace {
    pub id: i32,
    pub name: String,
    pub output: String,
    pub focused: bool,
    pub urgent: bool,
    pub empty: bool,
    pub representation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, Type)]
pub struct WorkspaceState {
    pub workspaces: Vec<Workspace>,
    pub focused_output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, Type)]
pub struct FocusedWindow {
    pub title: Option<String>,
    pub app_id: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default, Type)]
#[serde(rename_all = "kebab-case")]
pub enum NotificationPosition {
    #[default]
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
    TopCenter,
    BottomCenter,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_expected_values() {
        let cfg = Config::default();
        assert_eq!(cfg.clock.format, "%H:%M");
        assert_eq!(cfg.clock.interval, 1000);
        assert_eq!(cfg.clock.timezone, None);
        assert_eq!(cfg.outputs.len(), 1);
        assert_eq!(cfg.outputs[0].name, "*");
        assert_eq!(cfg.outputs[0].height, 36);
        assert!(cfg.outputs[0].exclusive);
        assert!(cfg.workspaces.show_names);
        assert!(!cfg.workspaces.show_empty);
    }

    #[test]
    fn config_toml_round_trips() {
        let original = Config::default();
        let toml_str = toml::to_string_pretty(&original).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.clock.format, original.clock.format);
        assert_eq!(parsed.clock.interval, original.clock.interval);
        assert_eq!(parsed.outputs.len(), original.outputs.len());
        assert_eq!(parsed.outputs[0].name, original.outputs[0].name);
        assert_eq!(parsed.outputs[0].height, original.outputs[0].height);
        assert_eq!(parsed.general.gap, original.general.gap);
    }

    #[test]
    fn output_config_resolve_exact_match_wins() {
        let wildcard = OutputConfig {
            name: "*".into(),
            height: 36,
            ..OutputConfig::default()
        };
        let specific = OutputConfig {
            name: "DP-1".into(),
            height: 50,
            ..OutputConfig::default()
        };
        let configs = vec![wildcard, specific];
        assert_eq!(OutputConfig::resolve(&configs, "DP-1").height, 50);
        assert_eq!(OutputConfig::resolve(&configs, "HDMI-A-1").height, 36);
    }

    #[test]
    fn output_config_resolve_first_entry_fallback() {
        let only = OutputConfig {
            name: "eDP-1".into(),
            height: 32,
            ..OutputConfig::default()
        };
        let configs = vec![only];
        assert_eq!(OutputConfig::resolve(&configs, "DP-1").height, 32);
    }

    #[test]
    fn power_action_json_roundtrip() {
        let actions = [
            PowerAction::Lock,
            PowerAction::Logout,
            PowerAction::Suspend,
            PowerAction::Hibernate,
            PowerAction::HybridSleep,
            PowerAction::Reboot,
            PowerAction::Poweroff,
        ];
        for action in &actions {
            let json = serde_json::to_string(action).unwrap();
            let back: PowerAction = serde_json::from_str(&json).unwrap();
            assert_eq!(&back, action);
        }
    }

    #[test]
    fn power_menu_default_actions_are_sensible() {
        let cfg = PowerMenuConfig::default();
        assert!(cfg.actions.contains(&PowerAction::Reboot));
        assert!(cfg.actions.contains(&PowerAction::Poweroff));
        assert!(cfg.actions.contains(&PowerAction::Lock));
        assert!(!cfg.actions.contains(&PowerAction::HybridSleep));
    }

    #[test]
    fn notification_position_kebab_case_serde() {
        let pos = NotificationPosition::TopRight;
        let s = serde_json::to_string(&pos).unwrap();
        assert_eq!(s, r#""top-right""#);
        let back: NotificationPosition =
            serde_json::from_str(r#""bottom-left""#).unwrap();
        assert!(matches!(back, NotificationPosition::BottomLeft));
    }
}
