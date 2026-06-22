// Mirror of src-tauri/src/config/schema.rs — keep in sync.

export type BarLayer = "background" | "bottom" | "top" | "overlay";
export type KeyboardMode = "none" | "on_demand" | "exclusive";
export type NotificationPosition =
  | "top-right"
  | "top-left"
  | "bottom-right"
  | "bottom-left"
  | "top-center"
  | "bottom-center";
export type PowerAction =
  | "lock"
  | "logout"
  | "suspend"
  | "hibernate"
  | "hybrid_sleep"
  | "reboot"
  | "poweroff";

export interface ThemeConfig {
  background: string;
  foreground: string;
  border_radius: number;
  font: string;
  font_size: number;
  module_background: string;
  module_padding: string;
}

export interface RegionConfig {
  modules: string[];
}

export interface OutputConfig {
  name: string;
  height: number;
  layer: BarLayer;
  exclusive: boolean;
  opacity: number;
  keyboard: KeyboardMode;
  left: RegionConfig;
  center: RegionConfig;
  right: RegionConfig;
  theme: ThemeConfig;
}

export interface ClockConfig {
  format: string;
  interval: number;
  timezone: string | null;
}

export interface WorkspacesConfig {
  show_empty: boolean;
  show_names: boolean;
  show_icons: boolean;
  persistent: string[];
}

export interface PowerMenuConfig {
  actions: PowerAction[];
}

export interface NotificationsConfig {
  position: NotificationPosition;
  max_visible: number;
  timeout: number;
}

export interface GeneralConfig {
  gap: number;
}

export interface Config {
  general: GeneralConfig;
  outputs: OutputConfig[];
  clock: ClockConfig;
  workspaces: WorkspacesConfig;
  power_menu: PowerMenuConfig;
  notifications: NotificationsConfig;
}

// ── Compositor event payloads (contract with the compositor stream) ────────────

export interface Workspace {
  id: number;
  name: string;
  output: string;
  focused: boolean;
  urgent: boolean;
  empty: boolean;
  representation?: string;
}

export interface WorkspaceState {
  workspaces: Workspace[];
  focused_output: string | null;
}

export interface FocusedWindow {
  title: string | null;
  app_id: string | null;
}
