//! Native PipeWire backend. All proxies and callbacks are owned by one thread.
#![cfg(target_os = "linux")]
#![forbid(unsafe_code)]
use pipewire as pw;
use pw::{
    spa::{
        self,
        param::ParamType,
        pod::{
            deserialize::PodDeserializer, serialize::PodSerializer, Object, Pod, Property, Value,
            ValueArray,
        },
    },
    types::ObjectType,
};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    io::Cursor,
    rc::Rc,
    time::{Duration, Instant},
};
use veek_audio::{balanced_channels, Backend, Change, Error, Kind, Result, Target};

struct Entry {
    target: Target,
    name: String,
    channels: Vec<f32>,
    scalar: Option<f32>,
    writable: bool,
    _listener: pw::node::NodeListener,
    node: pw::node::Node,
}
struct Metadata {
    _listener: pw::metadata::MetadataListener,
    _proxy: pw::metadata::Metadata,
}
#[derive(Default)]
struct State {
    nodes: BTreeMap<u32, Entry>,
    metadata: BTreeMap<u32, Metadata>,
    defaults: BTreeMap<String, String>,
    cookie: u32,
    error: Option<String>,
}
pub struct PipeWire {
    _registry_listener: pw::registry::Listener,
    _core_listener: pw::core::Listener,
    state: Rc<RefCell<State>>,
    _registry: pw::registry::RegistryRc,
    core: pw::core::CoreRc,
    mainloop: pw::main_loop::MainLoopRc,
}
fn native(e: impl std::fmt::Display) -> Error {
    Error::Unavailable(e.to_string())
}
impl PipeWire {
    pub fn connect() -> Result<Self> {
        pw::init();
        let mainloop = pw::main_loop::MainLoopRc::new(None).map_err(native)?;
        let context = pw::context::ContextRc::new(&mainloop, None).map_err(native)?;
        let core = context
            .connect_rc(Some(
                pw::properties::properties! {"application.name"=>"VeekPanel audio diagnostics"},
            ))
            .map_err(native)?;
        let registry = core.get_registry_rc().map_err(native)?;
        let state = Rc::new(RefCell::new(State::default()));
        let info_state = Rc::downgrade(&state);
        let error_state = Rc::downgrade(&state);
        let core_listener = core
            .add_listener_local()
            .info(move |info| {
                if let Some(s) = info_state.upgrade() {
                    s.borrow_mut().cookie = info.cookie();
                }
            })
            .error(move |id, _, res, message| {
                if let Some(s) = error_state.upgrade() {
                    s.borrow_mut().error = Some(format!("PipeWire object {id}: {message} ({res})"));
                }
            })
            .register();
        let weak = Rc::downgrade(&state);
        let removed = weak.clone();
        let registry_weak = registry.downgrade();
        let listener = registry
            .add_listener_local()
            .global(move |global| {
                let (Some(state), Some(registry)) = (weak.upgrade(), registry_weak.upgrade())
                else {
                    return;
                };
                let Some(props) = global.props.as_ref() else {
                    return;
                };
                let id = global.id;
                if global.type_ == ObjectType::Node {
                    let kind = match props.get("media.class") {
                        Some("Audio/Sink") => Kind::Output,
                        Some("Audio/Source") => Kind::Input,
                        Some("Stream/Output/Audio") => Kind::Playback,
                        Some("Stream/Input/Audio") => Kind::Recording,
                        _ => return,
                    };
                    let Some(serial) = props.get("object.serial") else {
                        return;
                    };
                    let node: pw::node::Node = match registry.bind(global) {
                        Ok(n) => n,
                        Err(e) => {
                            state.borrow_mut().error = Some(e.to_string());
                            return;
                        }
                    };
                    let identity = props
                        .iter()
                        .filter(|(k, _)| {
                            [
                                "application.id",
                                "application.name",
                                "application.process.binary",
                                "application.process.id",
                                "node.name",
                                "media.name",
                                "object.serial",
                                "device.id",
                            ]
                            .contains(k)
                        })
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                        .collect();
                    let name = props.get("node.name").unwrap_or("").to_string();
                    let target = Target {
                        id: serial.to_string(),
                        name: props
                            .get("node.description")
                            .or_else(|| props.get("application.name"))
                            .or_else(|| props.get("node.nick"))
                            .unwrap_or(&name)
                            .to_string(),
                        kind,
                        default: false,
                        volume: None,
                        muted: None,
                        identity,
                    };
                    let info_state = Rc::downgrade(&state);
                    let param_state = info_state.clone();
                    let listener = node
                        .add_listener_local()
                        .info(move |info| {
                            if let Some(s) = info_state.upgrade() {
                                if let Some(e) = s.borrow_mut().nodes.get_mut(&id) {
                                    if !info.params().is_empty() {
                                        e.writable = info.params().iter().any(|p| {
                                            p.id() == ParamType::Props
                                                && p.flags()
                                                    .contains(spa::param::ParamInfoFlags::WRITE)
                                        });
                                    }
                                    if let Some(props) = info.props() {
                                        for key in [
                                            "application.id",
                                            "application.name",
                                            "application.process.binary",
                                            "media.name",
                                        ] {
                                            if let Some(value) = props.get(key) {
                                                e.target.identity.insert(key.into(), value.into());
                                            }
                                        }
                                    }
                                }
                            }
                        })
                        .param(move |_, param_id, _, _, pod| {
                            if param_id != ParamType::Props {
                                return;
                            }
                            let Some(pod) = pod else { return };
                            // Reject oversized native properties before allocating a deserialized tree.
                            if pod.as_bytes().len() > 65536 {
                                return;
                            }
                            if let Ok((_, Value::Object(object))) =
                                PodDeserializer::deserialize_any_from(pod.as_bytes())
                            {
                                if let Some(s) = param_state.upgrade() {
                                    if let Some(e) = s.borrow_mut().nodes.get_mut(&id) {
                                        read_props(e, object);
                                    }
                                }
                            }
                        })
                        .register();
                    state.borrow_mut().nodes.insert(
                        id,
                        Entry {
                            target,
                            name,
                            channels: Vec::new(),
                            scalar: None,
                            writable: false,
                            _listener: listener,
                            node,
                        },
                    );
                    state.borrow().nodes[&id]
                        .node
                        .subscribe_params(&[ParamType::Props]);
                } else if global.type_ == ObjectType::Metadata
                    && props.get("metadata.name") == Some("default")
                {
                    let metadata: pw::metadata::Metadata = match registry.bind(global) {
                        Ok(m) => m,
                        Err(e) => {
                            state.borrow_mut().error = Some(e.to_string());
                            return;
                        }
                    };
                    let weak = Rc::downgrade(&state);
                    let listener = metadata
                        .add_listener_local()
                        .property(move |subject, key, _, value| {
                            if subject != 0 {
                                return 0;
                            }
                            if let Some(s) = weak.upgrade() {
                                let mut s = s.borrow_mut();
                                match key {
                                    Some(key @ ("default.audio.sink" | "default.audio.source")) => {
                                        let name = value
                                            .and_then(|v| {
                                                serde_json::from_str::<serde_json::Value>(v).ok()
                                            })
                                            .and_then(|v| {
                                                v.get("name")?.as_str().map(str::to_owned)
                                            });
                                        if let Some(name) = name {
                                            s.defaults.insert(key.into(), name);
                                        } else {
                                            s.defaults.remove(key);
                                        }
                                    }
                                    None => s.defaults.clear(),
                                    _ => (),
                                }
                            }
                            0
                        })
                        .register();
                    state.borrow_mut().metadata.insert(
                        id,
                        Metadata {
                            _listener: listener,
                            _proxy: metadata,
                        },
                    );
                }
            })
            .global_remove(move |id| {
                if let Some(s) = removed.upgrade() {
                    let mut s = s.borrow_mut();
                    s.nodes.remove(&id);
                    if s.metadata.remove(&id).is_some() {
                        s.defaults.clear();
                    }
                }
            })
            .register();
        let mut result = Self {
            _registry_listener: listener,
            _core_listener: core_listener,
            state,
            _registry: registry,
            core,
            mainloop,
        };
        // First roundtrip discovers objects; second completes subscriptions created in callbacks.
        result.roundtrip()?;
        result.roundtrip()?;
        Ok(result)
    }
    fn check(&self) -> Result<()> {
        if let Some(e) = &self.state.borrow().error {
            Err(Error::Unavailable(e.clone()))
        } else {
            Ok(())
        }
    }
    fn roundtrip(&mut self) -> Result<()> {
        self.check()?;
        let done = Rc::new(std::cell::Cell::new(false));
        let done2 = done.clone();
        let seq = self.core.sync(0).map_err(native)?;
        let _listener = self
            .core
            .add_listener_local()
            .done(move |id, s| {
                if id == pw::core::PW_ID_CORE && s == seq {
                    done2.set(true)
                }
            })
            .register();
        let deadline = Instant::now() + Duration::from_secs(2);
        while !done.get() {
            if Instant::now() >= deadline {
                return Err(Error::Unavailable("PipeWire response timed out".into()));
            }
            self.wait(Duration::from_millis(25))?;
        }
        self.check()
    }
}
fn read_props(entry: &mut Entry, object: Object) {
    for p in object.properties {
        match (p.key, p.value) {
            (spa::sys::SPA_PROP_channelVolumes, Value::ValueArray(ValueArray::Float(ch)))
                if !ch.is_empty()
                    && ch.len() <= 64
                    && ch.iter().all(|v| v.is_finite() && *v >= 0.0) =>
            {
                entry.channels = ch
            }
            (spa::sys::SPA_PROP_volume, Value::Float(v)) if v.is_finite() && v >= 0.0 => {
                entry.scalar = Some(v)
            }
            (spa::sys::SPA_PROP_mute, Value::Bool(v)) => entry.target.muted = Some(v),
            _ => (),
        }
    }
    // Match the cubic slider convention used by desktop PipeWire mixers.
    let peak = entry
        .channels
        .iter()
        .copied()
        .reduce(f32::max)
        .or(entry.scalar);
    entry.target.volume = peak.map(f32::cbrt);
}
impl Backend for PipeWire {
    fn name(&self) -> &'static str {
        "PipeWire (native)"
    }
    fn snapshot(&mut self) -> Result<Vec<Target>> {
        self.roundtrip()?;
        let state = self.state.borrow();
        Ok(state
            .nodes
            .values()
            .map(|entry| {
                let mut target = entry.target.clone();
                target.id = format!("pw:{}:{}", state.cookie, target.id);
                let key = match target.kind {
                    Kind::Output => "default.audio.sink",
                    Kind::Input => "default.audio.source",
                    _ => "",
                };
                target.default = state.defaults.get(key) == Some(&entry.name);
                target
            })
            .collect())
    }
    fn write(&mut self, id: &str, change: Change) -> Result<()> {
        change.validate()?;
        self.roundtrip()?;
        {
            let state = self.state.borrow();
            let e = state
                .nodes
                .values()
                .find(|e| format!("pw:{}:{}", state.cookie, e.target.id) == id)
                .ok_or_else(|| Error::Missing(id.into()))?;
            if !e.writable {
                return Err(Error::Unsupported("PipeWire Props are not writable".into()));
            }
            let property = match change {
                Change::Mute(m) => Property::new(spa::sys::SPA_PROP_mute, Value::Bool(m)),
                Change::Volume(v) if !e.channels.is_empty() => Property::new(
                    spa::sys::SPA_PROP_channelVolumes,
                    Value::ValueArray(ValueArray::Float(balanced_channels(
                        &e.channels,
                        v.powi(3),
                    )?)),
                ),
                Change::Volume(v) if e.scalar.is_some() => {
                    Property::new(spa::sys::SPA_PROP_volume, Value::Float(v.powi(3)))
                }
                _ => return Err(Error::Unsupported("no volume property".into())),
            };
            let object = Value::Object(Object {
                type_: spa::sys::SPA_TYPE_OBJECT_Props,
                id: ParamType::Props.as_raw(),
                properties: vec![property],
            });
            let (bytes, _) = PodSerializer::serialize(Cursor::new(Vec::new()), &object)
                .map_err(|e| Error::Native(e.to_string()))?;
            let bytes = bytes.into_inner();
            let pod = Pod::from_bytes(&bytes)
                .ok_or_else(|| Error::Native("could not encode PipeWire Props".into()))?;
            e.node.set_param(ParamType::Props, 0, pod);
            e.node.enum_params(0, Some(ParamType::Props), 0, u32::MAX);
        }
        self.roundtrip()
    }
    fn wait(&mut self, timeout: Duration) -> Result<()> {
        self.check()?;
        if self.mainloop.loop_().iterate(pw::loop_::Timeout::Finite(
            timeout.min(Duration::from_secs(1)),
        )) < 0
        {
            return Err(Error::Unavailable("PipeWire event loop failed".into()));
        }
        self.check()
    }
}
