//! SNI item tracking: read properties, watch for changes, resolve icons.

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use specta::Type;
use zbus::zvariant::OwnedObjectPath;
use zbus::Connection;

use crate::tray::dbusmenu::TrayMenu;
use crate::tray::icon::resolve_icon;

// ── SNI item snapshot ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TrayItem {
    /// Stable key identifying the item (service + object path, no separator).
    pub key: String,
    pub id: String,
    pub title: String,
    pub status: SniStatus,
    pub category: String,
    /// Resolved icon: `data:image/png;base64,...` or `file:///...`.
    pub icon: Option<String>,
    pub tooltip: Option<String>,
    pub menu_path: Option<String>,
    pub menu: Option<TrayMenu>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum SniStatus {
    Passive,
    Active,
    NeedsAttention,
}

impl SniStatus {
    fn from_str(s: &str) -> Self {
        match s {
            "Passive" => Self::Passive,
            "NeedsAttention" => Self::NeedsAttention,
            _ => Self::Active,
        }
    }
}

// ── D-Bus proxy for org.kde.StatusNotifierItem ────────────────────────────────

type PixmapList = Vec<(i32, i32, Vec<u8>)>;
type ToolTipTuple = (String, PixmapList, String, String);

#[zbus::proxy(interface = "org.kde.StatusNotifierItem")]
trait StatusNotifierItem {
    #[zbus(property, name = "Id")]
    fn item_id(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn title(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn status(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn category(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn icon_name(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn icon_theme_path(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn icon_pixmap(&self) -> zbus::Result<PixmapList>;
    #[zbus(property)]
    fn tool_tip(&self) -> zbus::Result<ToolTipTuple>;
    #[zbus(property)]
    fn menu(&self) -> zbus::Result<OwnedObjectPath>;

    fn activate(&self, x: i32, y: i32) -> zbus::Result<()>;
    fn secondary_activate(&self, x: i32, y: i32) -> zbus::Result<()>;
    fn context_menu(&self, x: i32, y: i32) -> zbus::Result<()>;
}

// ── Key helpers ───────────────────────────────────────────────────────────────

/// Split a raw SNI registration string into (service_name, object_path).
///
/// The spec says the service argument may be:
///   "org.kde.StatusNotifierItem-1234-1"        → bare service name
///   "org.kde.StatusNotifierItem-1234-1/path"   → service + path
/// When no service is given (rare), `sender` (the D-Bus unique name) is used.
pub fn parse_sni_key(raw: &str, sender: &str) -> (String, String) {
    if let Some(slash) = raw.find('/') {
        let svc = &raw[..slash];
        let path = &raw[slash..];
        (
            if svc.is_empty() { sender.to_owned() } else { svc.to_owned() },
            path.to_owned(),
        )
    } else if raw.is_empty() {
        (sender.to_owned(), "/StatusNotifierItem".to_owned())
    } else {
        (raw.to_owned(), "/StatusNotifierItem".to_owned())
    }
}

// ── Read + watch ──────────────────────────────────────────────────────────────

/// Build a full `TrayItem` snapshot by reading all SNI properties.
/// Any property failure is silently defaulted (graceful degrade).
pub async fn read_item(
    conn: &Connection,
    key: &str,
    service: &str,
    obj_path: &str,
) -> TrayItem {
    let proxy = match StatusNotifierItemProxy::builder(conn)
        .destination(service)
        .and_then(|b| b.path(obj_path))
    {
        Ok(builder) => match builder.build().await {
            Ok(p) => p,
            Err(_) => return empty_item(key),
        },
        Err(_) => return empty_item(key),
    };

    let id = proxy.item_id().await.unwrap_or_default();
    let title = proxy.title().await.unwrap_or_default();
    let status = SniStatus::from_str(&proxy.status().await.unwrap_or_default());
    let category = proxy.category().await.unwrap_or_default();

    let icon_name = proxy.icon_name().await.ok().filter(|s| !s.is_empty());
    let icon_theme_path = proxy.icon_theme_path().await.ok().filter(|s| !s.is_empty());
    let icon_pixmap: Option<PixmapList> = proxy.icon_pixmap().await.ok();

    let icon = resolve_icon(
        icon_name.as_deref(),
        icon_theme_path.as_deref(),
        icon_pixmap.as_deref(),
        22,
    );

    let tooltip = proxy
        .tool_tip()
        .await
        .ok()
        .and_then(|(_, _, t, body)| {
            if t.is_empty() && body.is_empty() {
                None
            } else if body.is_empty() {
                Some(t)
            } else {
                Some(format!("{t}\n{body}"))
            }
        });

    let menu_path = proxy
        .menu()
        .await
        .ok()
        .filter(|p| p.as_str() != "/" && !p.as_str().is_empty())
        .map(|p| p.as_str().to_owned());

    let menu = if let Some(ref mp) = menu_path {
        crate::tray::dbusmenu::fetch_menu(conn, service, mp).await.ok()
    } else {
        None
    };

    TrayItem { key: key.to_owned(), id, title, status, category, icon, tooltip, menu_path, menu }
}

fn empty_item(key: &str) -> TrayItem {
    TrayItem {
        key: key.to_owned(),
        id: String::new(),
        title: String::new(),
        status: SniStatus::Passive,
        category: String::new(),
        icon: None,
        tooltip: None,
        menu_path: None,
        menu: None,
    }
}

/// Subscribe to PropertiesChanged on an SNI item and call `on_change` whenever
/// any property changes.  Returns when the item's service disappears.
pub async fn watch_item(
    conn: Connection,
    key: String,
    service: String,
    obj_path: String,
    on_change: impl Fn(TrayItem) + Send + 'static,
) {
    let props_proxy = match zbus::fdo::PropertiesProxy::builder(&conn)
        .destination(&service)
        .and_then(|b| b.path(&obj_path))
    {
        Ok(b) => match b.build().await {
            Ok(p) => p,
            Err(_) => return,
        },
        Err(_) => return,
    };

    let Ok(mut changes) = props_proxy.receive_properties_changed().await else { return };

    while changes.next().await.is_some() {
        let item = read_item(&conn, &key, &service, &obj_path).await;
        on_change(item);
    }
}
