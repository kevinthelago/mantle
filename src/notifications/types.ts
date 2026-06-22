export type Urgency = "low" | "normal" | "critical";

export interface NotificationImage {
  width: number;
  height: number;
  rowstride: number;
  has_alpha: boolean;
  bits_per_sample: number;
  channels: number;
  /** PNG bytes, base64-encoded */
  data_b64: string;
}

export interface Notification {
  id: number;
  app_name: string;
  app_icon: string;
  summary: string;
  /** FDO markup subset — safe to dangerouslySetInnerHTML */
  body: string;
  /** Alternating [key, label] pairs */
  actions: string[];
  urgency: Urgency;
  category?: string;
  desktop_entry?: string;
  image?: NotificationImage;
  image_path?: string;
  resident: boolean;
  transient: boolean;
  /** undefined → persistent; number → auto-close after ms */
  expire_timeout_ms?: number;
  created_at: number;
}

export type ClosedReason =
  | "expired"
  | "dismissed"
  | "close_notification"
  | "undefined";

export type NotificationEvent =
  | { type: "Added"; data: Notification }
  | { type: "Replaced"; data: { id: number; notification: Notification } }
  | { type: "Closed"; data: { id: number; reason: ClosedReason } }
  | { type: "ActionInvoked"; data: { id: number; action_key: string } }
  | { type: "DndChanged"; data: { enabled: boolean } };

/** Parsed action pairs for rendering. */
export interface NotificationAction {
  key: string;
  label: string;
}

export function parseActions(raw: string[]): NotificationAction[] {
  const out: NotificationAction[] = [];
  for (let i = 0; i + 1 < raw.length; i += 2) {
    out.push({ key: raw[i], label: raw[i + 1] });
  }
  return out;
}
