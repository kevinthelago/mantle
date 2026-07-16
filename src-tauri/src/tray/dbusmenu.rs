//! Parse and interact with com.canonical.dbusmenu trees.

use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use zbus::zvariant::{OwnedValue, Value};
use zbus::{proxy, Connection};

// ── Menu types exposed to the frontend ───────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TrayMenu {
    pub revision: u32,
    pub root: MenuItem,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct MenuItem {
    pub id: i32,
    pub label: String,
    pub enabled: bool,
    pub visible: bool,
    pub icon_name: Option<String>,
    pub item_type: MenuItemType,
    pub toggle_type: MenuToggleType,
    pub toggle_state: ToggleState,
    pub children: Vec<MenuItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum MenuItemType {
    Standard,
    Separator,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum MenuToggleType {
    None,
    Checkmark,
    Radio,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ToggleState {
    Off,
    On,
    Indeterminate,
}

// ── D-Bus proxy ───────────────────────────────────────────────────────────────

type PropMap = HashMap<String, OwnedValue>;
// GetLayout returns (revision: u32, layout: (i32, PropMap, Vec<OwnedValue>))
type LayoutTuple = (i32, PropMap, Vec<OwnedValue>);

#[proxy(interface = "com.canonical.dbusmenu")]
trait DBusMenu {
    fn get_layout(
        &self,
        parent_id: i32,
        recursion_depth: i32,
        property_names: &[&str],
    ) -> zbus::Result<(u32, LayoutTuple)>;

    fn event(
        &self,
        id: i32,
        event_id: &str,
        data: &zbus::zvariant::Value<'_>,
        timestamp: u32,
    ) -> zbus::Result<()>;

    fn about_to_show(&self, id: i32) -> zbus::Result<bool>;

    #[zbus(signal)]
    fn layout_updated(&self, revision: u32, parent: i32) -> zbus::Result<()>;

    #[zbus(signal)]
    fn items_properties_updated(
        &self,
        updated_props: Vec<(i32, PropMap)>,
        removed_props: Vec<(i32, Vec<String>)>,
    ) -> zbus::Result<()>;
}

// ── Public API ────────────────────────────────────────────────────────────────

pub async fn fetch_menu(
    conn: &Connection,
    service: &str,
    menu_path: &str,
) -> Result<TrayMenu, String> {
    let proxy = DBusMenuProxy::builder(conn)
        .destination(service)
        .map_err(|e| e.to_string())?
        .path(menu_path)
        .map_err(|e| e.to_string())?
        .build()
        .await
        .map_err(|e| e.to_string())?;

    let (revision, layout) = proxy
        .get_layout(0, -1, &[])
        .await
        .map_err(|e| e.to_string())?;

    let root = parse_item(&layout);
    Ok(TrayMenu { revision, root })
}

pub async fn send_menu_event(
    conn: &Connection,
    service: &str,
    menu_path: &str,
    item_id: i32,
    event: &str,
) -> Result<(), String> {
    let proxy = DBusMenuProxy::builder(conn)
        .destination(service)
        .map_err(|e| e.to_string())?
        .path(menu_path)
        .map_err(|e| e.to_string())?
        .build()
        .await
        .map_err(|e| e.to_string())?;

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as u32;

    proxy
        .event(item_id, event, &Value::U32(0), ts)
        .await
        .map_err(|e| e.to_string())
}

// ── Parsing ───────────────────────────────────────────────────────────────────

fn parse_item(layout: &LayoutTuple) -> MenuItem {
    let (id, props, children_raw) = layout;

    let label = str_prop(props, "label").unwrap_or_default();
    let enabled = bool_prop(props, "enabled").unwrap_or(true);
    let visible = bool_prop(props, "visible").unwrap_or(true);
    let icon_name = str_prop(props, "icon-name");
    let item_type = match str_prop(props, "type").as_deref() {
        Some("separator") => MenuItemType::Separator,
        _ => MenuItemType::Standard,
    };
    let toggle_type = match str_prop(props, "toggle-type").as_deref() {
        Some("checkmark") => MenuToggleType::Checkmark,
        Some("radio") => MenuToggleType::Radio,
        _ => MenuToggleType::None,
    };
    let toggle_state = match i32_prop(props, "toggle-state") {
        Some(0) => ToggleState::Off,
        Some(1) => ToggleState::On,
        _ => ToggleState::Indeterminate,
    };

    let children = children_raw.iter().filter_map(parse_child_value).collect();

    MenuItem {
        id: *id,
        label,
        enabled,
        visible,
        icon_name,
        item_type,
        toggle_type,
        toggle_state,
        children,
    }
}

/// Each child in `av` is a D-Bus struct `(i32, a{sv}, av)` boxed as a Variant.
fn parse_child_value(v: &OwnedValue) -> Option<MenuItem> {
    // Dereference through OwnedValue → Value and extract the inner structure.
    let inner: &Value = &**v;
    let s = match inner {
        Value::Structure(s) => s,
        _ => return None,
    };
    let fields = s.fields();
    if fields.len() < 3 {
        return None;
    }

    let id = match &fields[0] {
        Value::I32(n) => *n,
        _ => return None,
    };

    // a{sv}: zvariant represents this as Value::Dict — convert to PropMap.
    let props: PropMap = match &fields[1] {
        Value::Dict(d) => {
            d.iter()
                .filter_map(|(k, v)| {
                    let key = match k {
                        Value::Str(s) => s.to_string(),
                        _ => return None,
                    };
                    // Wrap the value reference into an OwnedValue by going through serde.
                    // zvariant's OwnedValue::from<Value> for non-'static lifetimes isn't
                    // directly available, so we use try_into with the static lifetime variant.
                    let owned: OwnedValue = OwnedValue::try_from(v.clone()).ok()?;
                    Some((key, owned))
                })
                .collect()
        }
        _ => HashMap::new(),
    };

    // av: each element is again an OwnedValue wrapping a struct.
    let children_raw: Vec<OwnedValue> = match &fields[2] {
        Value::Array(arr) => arr
            .iter()
            .filter_map(|item| OwnedValue::try_from(item.clone()).ok())
            .collect(),
        _ => vec![],
    };

    Some(parse_item(&(id, props, children_raw)))
}

// ── Property helpers ──────────────────────────────────────────────────────────

fn str_prop(props: &PropMap, key: &str) -> Option<String> {
    let ov = props.get(key)?;
    let inner: &Value = &**ov;
    match inner {
        Value::Str(s) => Some(s.to_string()),
        _ => None,
    }
}

fn bool_prop(props: &PropMap, key: &str) -> Option<bool> {
    let ov = props.get(key)?;
    let inner: &Value = &**ov;
    match inner {
        Value::Bool(b) => Some(*b),
        _ => None,
    }
}

fn i32_prop(props: &PropMap, key: &str) -> Option<i32> {
    let ov = props.get(key)?;
    let inner: &Value = &**ov;
    match inner {
        Value::I32(n) => Some(*n),
        _ => None,
    }
}
