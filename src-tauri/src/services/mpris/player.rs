use std::collections::HashMap;
use std::ops::Deref;
use zbus::{proxy, Connection, Result as ZbusResult};
use zbus::zvariant::{Array, OwnedValue, Value};

pub const MPRIS_PREFIX: &str = "org.mpris.MediaPlayer2.";

#[proxy(
    interface = "org.mpris.MediaPlayer2.Player",
    default_path = "/org/mpris/MediaPlayer2"
)]
pub trait MprisPlayer {
    fn play_pause(&self) -> ZbusResult<()>;
    fn next(&self) -> ZbusResult<()>;
    fn previous(&self) -> ZbusResult<()>;

    #[zbus(property)]
    fn playback_status(&self) -> ZbusResult<String>;

    #[zbus(property)]
    fn metadata(&self) -> ZbusResult<HashMap<String, OwnedValue>>;

    #[zbus(property)]
    fn volume(&self) -> ZbusResult<f64>;

    #[zbus(property)]
    fn position(&self) -> ZbusResult<i64>;

    #[zbus(property)]
    fn can_go_next(&self) -> ZbusResult<bool>;

    #[zbus(property)]
    fn can_go_previous(&self) -> ZbusResult<bool>;

    #[zbus(property)]
    fn can_pause(&self) -> ZbusResult<bool>;

    #[zbus(property)]
    fn can_play(&self) -> ZbusResult<bool>;
}

pub async fn list_players(conn: &Connection) -> ZbusResult<Vec<String>> {
    let dbus = zbus::fdo::DBusProxy::new(conn).await?;
    let names = dbus.list_names().await?;
    Ok(names
        .into_iter()
        .filter(|n| n.starts_with(MPRIS_PREFIX))
        .map(|n| n.to_string())
        .collect())
}

/// Returns the active (Playing) player's bus name, or the first listed one, or None.
pub async fn active_player(conn: &Connection) -> ZbusResult<Option<String>> {
    let players = list_players(conn).await?;
    if players.is_empty() {
        return Ok(None);
    }
    for name in &players {
        let proxy = MprisPlayerProxy::builder(conn)
            .destination(name.as_str())?
            .build()
            .await?;
        if let Ok(status) = proxy.playback_status().await {
            if status == "Playing" {
                return Ok(Some(name.clone()));
            }
        }
    }
    Ok(Some(players.into_iter().next().unwrap()))
}

/// Extract a string from an MPRIS metadata map entry.
/// Handles both plain strings (`s`) and arrays of strings (`as`).
pub fn extract_string(meta: &HashMap<String, OwnedValue>, key: &str) -> Option<String> {
    let owned = meta.get(key)?;
    match owned.deref() {
        Value::Str(s) => Some(s.to_string()),
        Value::Array(arr) => first_string_from_array(arr),
        _ => None,
    }
}

fn first_string_from_array(arr: &Array<'_>) -> Option<String> {
    // In zvariant 4.x, Array::get returns Result<Option<Value<'_>>, Error>
    match arr.get(0) {
        Ok(Some(Value::Str(s))) => Some(s.to_string()),
        _ => None,
    }
}

/// Extract an i64 from an MPRIS metadata map entry.
pub fn extract_i64(meta: &HashMap<String, OwnedValue>, key: &str) -> Option<i64> {
    let owned = meta.get(key)?;
    match owned.deref() {
        Value::I64(n) => Some(*n),
        _ => None,
    }
}
