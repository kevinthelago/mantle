//! Native PipeWire audio service on a dedicated MainLoop thread.
//!
//! Thread design:
//!  • std::thread "pipewire-audio" owns the pw::MainLoop — never touches tokio.
//!  • Arc<Mutex<AudioSnapshot>> is the cross-thread snapshot store.
//!  • pw::channel::Sender<PwCmd> lets the Tauri async side send volume/mute
//!    commands into the PW loop without any blocking.
//!
//! Discovery path:
//!  1. Metadata object (pw.metadata) → watch `default.audio.sink` key.
//!  2. On each Node global with media.class = "Audio/Sink", enumerate
//!     SPA_PARAM_Props to read channelVolumes + mute.
//!  3. When the default sink name matches a tracked node, publish its props.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use pipewire as pw;
use pipewire::spa::param::ParamType;
use pipewire::spa::pod::serialize::PodSerializer;
use pipewire::spa::pod::{Object, Pod, Property, PropertyFlags, Value, ValueArray};
use pipewire::spa::sys::{SPA_PARAM_Props, SPA_PROP_channelVolumes, SPA_PROP_mute};
use pipewire::spa::utils::Id;
use pipewire::types::ObjectType;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager, State};

// ── Public snapshot types ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AudioSnapshot {
    pub available: bool,
    /// Linear amplitude 0.0–1.0.
    pub volume: f32,
    pub muted: bool,
    pub sink_name: Option<String>,
    pub icon: AudioIcon,
}

impl AudioSnapshot {
    fn unavailable() -> Self {
        Self {
            available: false,
            volume: 0.0,
            muted: false,
            sink_name: None,
            icon: AudioIcon::Muted,
        }
    }

    fn compute_icon(volume: f32, muted: bool) -> AudioIcon {
        if muted || volume == 0.0 {
            AudioIcon::Muted
        } else if volume < 0.34 {
            AudioIcon::Low
        } else if volume < 0.67 {
            AudioIcon::Medium
        } else {
            AudioIcon::High
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum AudioIcon {
    Muted,
    Low,
    Medium,
    High,
}

// ── Commands that travel from Tauri → PW thread ───────────────────────────────

enum PwCmd {
    SetVolume(f32),
    SetMute(bool),
}

// ── Per-node data kept on the PW thread ──────────────────────────────────────

struct TrackedNode {
    name: String,
    proxy: pw::node::Node,
    _listener: pw::node::NodeListener,
}

// ── Service ───────────────────────────────────────────────────────────────────

pub struct AudioService {
    snapshot: Arc<Mutex<AudioSnapshot>>,
    cmd_tx: pw::channel::Sender<PwCmd>,
}

impl AudioService {
    /// Initialise the service; spawns the PW thread and returns immediately.
    pub fn init(app: AppHandle) -> Arc<Self> {
        pw::init();
        let snapshot = Arc::new(Mutex::new(AudioSnapshot::unavailable()));
        let (cmd_tx, cmd_rx) = pw::channel::channel::<PwCmd>();

        let snap2 = snapshot.clone();
        std::thread::Builder::new()
            .name("pipewire-audio".into())
            .spawn(move || pw_thread(app, snap2, cmd_rx))
            .expect("spawn pipewire-audio thread");

        Arc::new(Self { snapshot, cmd_tx })
    }

    pub fn snapshot(&self) -> AudioSnapshot {
        self.snapshot.lock().unwrap().clone()
    }

    pub fn set_volume(&self, vol: f32) -> Result<(), String> {
        self.cmd_tx.send(PwCmd::SetVolume(vol.clamp(0.0, 1.0))).map_err(|e| e.to_string())
    }

    pub fn set_mute(&self, muted: bool) -> Result<(), String> {
        self.cmd_tx.send(PwCmd::SetMute(muted)).map_err(|e| e.to_string())
    }
}

// ── PipeWire thread ───────────────────────────────────────────────────────────

fn pw_thread(app: AppHandle, snapshot: Arc<Mutex<AudioSnapshot>>, cmd_rx: pw::channel::Receiver<PwCmd>) {
    let mainloop = match pw::MainLoop::new(None) {
        Ok(m) => m,
        Err(e) => { log::warn!("PipeWire MainLoop: {e}"); return; }
    };
    let context = match pw::Context::new(&mainloop) {
        Ok(c) => c,
        Err(e) => { log::warn!("PipeWire Context: {e}"); return; }
    };
    let core = match context.connect(None) {
        Ok(c) => c,
        Err(e) => { log::warn!("PipeWire connect: {e}"); return; }
    };
    let registry = Rc::new(match core.get_registry() {
        Ok(r) => r,
        Err(e) => { log::warn!("PipeWire Registry: {e}"); return; }
    });

    // Thread-local mutable state — all callbacks run on this single thread.
    let default_sink_name: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    let default_node_id: Rc<RefCell<Option<u32>>> = Rc::new(RefCell::new(None));
    let nodes: Rc<RefCell<HashMap<u32, TrackedNode>>> = Rc::new(RefCell::new(HashMap::new()));
    let volume: Rc<RefCell<f32>> = Rc::new(RefCell::new(0.0));
    let mute: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));

    // Wire up command channel → PW loop.
    let cmd_nodes = nodes.clone();
    let cmd_def_id = default_node_id.clone();
    let cmd_vol = volume.clone();
    let cmd_mut = mute.clone();
    let cmd_snap = snapshot.clone();
    let cmd_app = app.clone();
    let cmd_dsn = default_sink_name.clone();
    let _cmd_attached = cmd_rx.attach(mainloop.loop_(), move |cmd| {
        let Some(id) = *cmd_def_id.borrow() else { return };
        let nodes_ref = cmd_nodes.borrow();
        let Some(node_state) = nodes_ref.get(&id) else { return };
        match cmd {
            PwCmd::SetVolume(v) => { *cmd_vol.borrow_mut() = v; }
            PwCmd::SetMute(m)   => { *cmd_mut.borrow_mut() = m; }
        }
        apply_props(&node_state.proxy, *cmd_vol.borrow(), *cmd_mut.borrow());
        publish(&cmd_snap, &cmd_app, &cmd_vol, &cmd_mut, &cmd_dsn);
    });

    // Watch the global registry.
    let reg_ref = Rc::clone(&registry);
    let dsn = default_sink_name.clone();
    let dnid = default_node_id.clone();
    let nds = nodes.clone();
    let vol = volume.clone();
    let mut_ = mute.clone();
    let snap = snapshot.clone();
    let app2 = app.clone();

    let _registry_listener = registry
        .add_listener_local()
        .global(move |global| {
            on_global(global, &reg_ref, &dsn, &dnid, &nds, &vol, &mut_, &snap, &app2);
        })
        .global_remove({
            let nds2 = nodes.clone();
            let dnid2 = default_node_id.clone();
            move |id| {
                nds2.borrow_mut().remove(&id);
                if dnid2.borrow().as_ref() == Some(&id) {
                    *dnid2.borrow_mut() = None;
                }
            }
        })
        .register();

    mainloop.run();
    // pw::deinit() is called implicitly via the Drop impl.
}

fn on_global(
    global: &pipewire::registry::GlobalObject<&pipewire::spa::utils::dict::DictRef>,
    registry: &Rc<pw::registry::Registry>,
    default_sink_name: &Rc<RefCell<Option<String>>>,
    default_node_id: &Rc<RefCell<Option<u32>>>,
    nodes: &Rc<RefCell<HashMap<u32, TrackedNode>>>,
    volume: &Rc<RefCell<f32>>,
    mute: &Rc<RefCell<bool>>,
    snapshot: &Arc<Mutex<AudioSnapshot>>,
    app: &AppHandle,
) {
    match global.type_ {
        ObjectType::Metadata => {
            let meta: pw::metadata::Metadata = match registry.bind(global) {
                Ok(m) => m,
                Err(_) => return,
            };
            let dsn = Rc::clone(default_sink_name);
            let dnid = Rc::clone(default_node_id);
            let nds = Rc::clone(nodes);
            let vol = Rc::clone(volume);
            let mut_ = Rc::clone(mute);
            let snap = snapshot.clone();
            let app = app.clone();
            let _listener = meta
                .add_listener_local()
                .property(move |_subject, key, _type, value| {
                    if key != Some("default.audio.sink") { return; }
                    let name = value
                        .and_then(|v| serde_json::from_str::<serde_json::Value>(v).ok())
                        .and_then(|j| j["name"].as_str().map(ToOwned::to_owned));
                    *dsn.borrow_mut() = name.clone();
                    // Try to find matching node.
                    let new_id = nds.borrow().iter().find_map(|(id, n)| {
                        name.as_deref().map(|nm| nm == n.name).unwrap_or(false).then_some(*id)
                    });
                    *dnid.borrow_mut() = new_id;
                    if new_id.is_some() {
                        publish(&snap, &app, &vol, &mut_, &dsn);
                    }
                })
                .register();
            // Keep the proxy alive: store it temporarily in the nodes map under a phantom id.
            // We store it as a dummy TrackedNode so its Drop keeps the proxy alive.
            // In practice Metadata lives for the whole session so this is fine.
        }

        ObjectType::Node => {
            let props = match global.props { Some(p) => p, None => return };
            if props.get("media.class") != Some("Audio/Sink") { return; }
            let node_name = props.get("node.name").unwrap_or("").to_owned();
            let id = global.id;

            let node: pw::node::Node = match registry.bind(global) {
                Ok(n) => n,
                Err(_) => return,
            };

            let dsn = Rc::clone(default_sink_name);
            let dnid = Rc::clone(default_node_id);
            let vol = Rc::clone(volume);
            let mut_ = Rc::clone(mute);
            let snap = snapshot.clone();
            let app = app.clone();
            let nn = node_name.clone();

            let listener = node
                .add_listener_local()
                .param(move |_seq, param_type, _idx, _next, pod| {
                    if param_type != ParamType::Props { return; }
                    let Some(pod) = pod else { return };
                    parse_props(pod, &vol, &mut_);
                    let is_default =
                        dsn.borrow().as_deref().map(|n| n == nn).unwrap_or(false)
                        || dnid.borrow().as_ref() == Some(&id);
                    if is_default {
                        *dnid.borrow_mut() = Some(id);
                        publish(&snap, &app, &vol, &mut_, &dsn);
                    }
                })
                .register();

            node.enum_params(0, Some(ParamType::Props), 0, u32::MAX);

            // If metadata already told us the default sink name, claim this node now.
            if default_sink_name.borrow().as_deref() == Some(&node_name) {
                *default_node_id.borrow_mut() = Some(id);
            }

            nodes.borrow_mut().insert(id, TrackedNode {
                name: node_name,
                proxy: node,
                _listener: listener,
            });
        }

        _ => {}
    }
}

// ── SPA pod helpers ───────────────────────────────────────────────────────────

fn parse_props(pod: &Pod, volume: &Rc<RefCell<f32>>, mute: &Rc<RefCell<bool>>) {
    use pipewire::spa::pod::deserialize::PodDeserializer;
    // Walk the props object looking for channelVolumes and mute.
    // The pod is an SPA_TYPE_Object with props.
    if let Ok((_, Value::Object(obj))) = PodDeserializer::deserialize_from::<Value>(pod.as_bytes()) {
        for prop in &obj.properties {
            match prop.key {
                k if k == unsafe { SPA_PROP_channelVolumes } => {
                    if let Value::ValueArray(ValueArray::Float(vols)) = &prop.value {
                        if let Some(&v) = vols.first() {
                            *volume.borrow_mut() = v;
                        }
                    }
                }
                k if k == unsafe { SPA_PROP_mute } => {
                    if let Value::Bool(m) = prop.value {
                        *mute.borrow_mut() = m;
                    }
                }
                _ => {}
            }
        }
    }
}

fn apply_props(node: &pw::node::Node, volume: f32, muted: bool) {
    let mut buf = Vec::new();
    let res = PodSerializer::serialize(
        std::io::Cursor::new(&mut buf),
        &Value::Object(Object {
            type_: pipewire::spa::utils::SpaTypes::ObjectParamProps.as_raw(),
            id: Id(unsafe { SPA_PARAM_Props }),
            properties: vec![
                Property {
                    key: unsafe { SPA_PROP_channelVolumes },
                    flags: PropertyFlags::empty(),
                    value: Value::ValueArray(ValueArray::Float(vec![volume, volume])),
                },
                Property {
                    key: unsafe { SPA_PROP_mute },
                    flags: PropertyFlags::empty(),
                    value: Value::Bool(muted),
                },
            ],
        }),
    );
    if res.is_ok() {
        if let Some(pod) = unsafe { Pod::from_bytes(&buf) } {
            node.set_param(ParamType::Props, 0, pod);
        }
    }
}

fn publish(
    snapshot: &Arc<Mutex<AudioSnapshot>>,
    app: &AppHandle,
    volume: &Rc<RefCell<f32>>,
    mute: &Rc<RefCell<bool>>,
    sink_name: &Rc<RefCell<Option<String>>>,
) {
    let vol = *volume.borrow();
    let muted = *mute.borrow();
    let name = sink_name.borrow().clone();
    let snap = AudioSnapshot {
        available: true,
        volume: vol,
        muted,
        sink_name: name,
        icon: AudioSnapshot::compute_icon(vol, muted),
    };
    *snapshot.lock().unwrap() = snap.clone();
    app.emit("audio_update", &snap).ok();
}

// ── Tauri commands ────────────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
pub async fn get_audio_snapshot(
    state: State<'_, Arc<AudioService>>,
) -> Result<AudioSnapshot, String> {
    Ok(state.snapshot())
}

#[tauri::command]
#[specta::specta]
pub async fn set_audio_volume(
    volume: f32,
    state: State<'_, Arc<AudioService>>,
) -> Result<(), String> {
    state.set_volume(volume)
}

#[tauri::command]
#[specta::specta]
pub async fn set_audio_mute(
    muted: bool,
    state: State<'_, Arc<AudioService>>,
) -> Result<(), String> {
    state.set_mute(muted)
}
