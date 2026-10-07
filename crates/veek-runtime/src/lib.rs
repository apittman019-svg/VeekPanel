//! Background ownership of configuration, native audio and reconnecting hardware.
#![forbid(unsafe_code)]
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};
use veek_audio::{Backend, Change, Controller, Selection, Snapshot};
use veek_config::{Config, Hardware, HardwareMode, Panel, Store};
use veek_core::Engine;
use veek_hardware::{
    protocol::ControlEvent,
    transport::{self, Address, Session},
    Model,
};

#[derive(Clone, Serialize)]
pub struct State {
    pub config: Config,
    pub revision: u64,
    pub audio: Option<Snapshot>,
    pub audio_status: String,
    pub hardware_status: String,
    pub controls: BTreeMap<String, u8>,
    pub diagnostics: Vec<String>,
}
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Command {
    Save { revision: u64, config: Config },
    Activate { id: String },
    Volume { selection: Selection, value: f32 },
    Mute { selection: Selection, muted: bool },
    MockAnalog { index: u8, raw: u8 },
    MockButton { index: u8, pressed: bool },
    Import { revision: u64, json: String },
}
struct Request {
    command: Command,
    deadline: Instant,
    reply: SyncSender<Result<(), String>>,
}
#[derive(Clone)]
pub struct Handle {
    tx: SyncSender<Request>,
    state: Arc<Mutex<State>>,
}
impl Handle {
    pub fn state(&self) -> State {
        self.state.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }
    pub fn request(&self, command: Command) -> Result<(), String> {
        let (tx, rx) = mpsc::sync_channel(1);
        self.tx
            .try_send(Request {
                command,
                deadline: Instant::now() + Duration::from_secs(10),
                reply: tx,
            })
            .map_err(|_| {
                "Background queue is full or closed; refresh before retrying".to_string()
            })?;
        rx.recv_timeout(Duration::from_secs(10)).map_err(|_|"Background response timed out; refresh state before retrying (a native call may still be completing)".to_string())?
    }
}
pub struct Runtime {
    pub handle: Handle,
    stop: Arc<AtomicBool>,
    join: Option<thread::JoinHandle<()>>,
}
impl Drop for Runtime {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}
#[cfg(target_os = "linux")]
fn connect() -> veek_audio::Result<Box<dyn Backend>> {
    Ok(Box::new(veek_audio_linux::PipeWire::connect()?))
}
#[cfg(windows)]
fn connect() -> veek_audio::Result<Box<dyn Backend>> {
    Ok(Box::new(veek_audio_windows::CoreAudio::connect()?))
}

pub fn start(path: PathBuf) -> Result<Runtime, String> {
    start_with(path, connect)
}
fn start_with<F>(path: PathBuf, connect: F) -> Result<Runtime, String>
where
    F: Fn() -> veek_audio::Result<Box<dyn Backend>> + Send + 'static,
{
    let (store, config) = Store::open(path).map_err(|e| e.to_string())?;
    let state = Arc::new(Mutex::new(State {
        config,
        revision: 1,
        audio: None,
        audio_status: "Connecting".into(),
        hardware_status: "Starting".into(),
        controls: BTreeMap::new(),
        diagnostics: vec![],
    }));
    let (tx, rx) = mpsc::sync_channel(64);
    let stop = Arc::new(AtomicBool::new(false));
    let shared = state.clone();
    let signal = stop.clone();
    let join = thread::Builder::new()
        .name("veek-background".into())
        .spawn(move || run(store, shared, rx, signal, connect))
        .map_err(|e| e.to_string())?;
    Ok(Runtime {
        handle: Handle { tx, state },
        stop,
        join: Some(join),
    })
}
struct Native(Box<dyn Backend>);
impl Backend for Native {
    fn name(&self) -> &'static str {
        self.0.name()
    }
    fn snapshot(&mut self) -> veek_audio::Result<Vec<veek_audio::Target>> {
        self.0.snapshot()
    }
    fn write(&mut self, id: &str, c: Change) -> veek_audio::Result<()> {
        self.0.write(id, c)
    }
    fn wait(&mut self, d: Duration) -> veek_audio::Result<()> {
        self.0.wait(d)
    }
}
fn note(state: &mut State, message: String) {
    if state.diagnostics.last() != Some(&message) {
        state.diagnostics.push(message);
        if state.diagnostics.len() > 80 {
            state.diagnostics.remove(0);
        }
    }
}
fn model(panel: Panel) -> Model {
    match panel {
        Panel::Original => Model::Original,
        Panel::Rgb => Model::Rgb,
        Panel::Mini => Model::Mini,
        Panel::Pro => Model::Pro,
    }
}
struct Reader {
    stop: Arc<AtomicBool>,
    join: Option<thread::JoinHandle<()>>,
    rx: Receiver<HardwareMessage>,
}
enum HardwareMessage {
    Status(String),
    Input(ControlEvent, Instant),
}
impl Drop for Reader {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(j) = self.join.take() {
            let _ = j.join();
        }
    }
}
fn send(tx: &SyncSender<HardwareMessage>, stop: &AtomicBool, mut message: HardwareMessage) -> bool {
    loop {
        if stop.load(Ordering::Relaxed) {
            return false;
        }
        match tx.try_send(message) {
            Ok(()) => return true,
            Err(mpsc::TrySendError::Full(m)) => {
                message = m;
                thread::sleep(Duration::from_millis(2));
            }
            Err(_) => return false,
        }
    }
}
fn reader(hardware: Hardware) -> Result<Reader, String> {
    let (tx, rx) = mpsc::sync_channel(256);
    let stop = Arc::new(AtomicBool::new(false));
    let signal = stop.clone();
    let join = thread::Builder::new()
        .name("veek-panel".into())
        .spawn(move || {
            if matches!(hardware.mode, HardwareMode::Disabled | HardwareMode::Mock) {
                let label = if hardware.mode == HardwareMode::Mock {
                    "Development panel: simulated input, real audio"
                } else {
                    "Hardware disabled"
                };
                let _ = send(&tx, &signal, HardwareMessage::Status(label.into()));
                return;
            }
            while !signal.load(Ordering::Relaxed) {
                let result = (|| -> Result<(), String> {
                    let wanted = model(hardware.model);
                    let device = if hardware.mode == HardwareMode::Serial {
                        transport::DeviceDescriptor::selected_original(hardware.address.clone())
                    } else {
                        let mut api = hidapi::HidApi::new().map_err(|e| e.to_string())?;
                        let mut found = transport::discover_hid(&mut api)
                            .map_err(|e| e.to_string())?
                            .into_iter()
                            .filter(|d| {
                                d.model == wanted
                                    && match &d.address {
                                        Address::Hid(p) => {
                                            hardware.mode == HardwareMode::AutoHid
                                                || p.to_string_lossy() == hardware.address
                                        }
                                        Address::Serial(_) => false,
                                    }
                            })
                            .collect::<Vec<_>>();
                        if found.len() != 1 {
                            return Err(if found.is_empty() {
                                "Panel disconnected; waiting for matching hardware"
                            } else {
                                "Multiple matching HID interfaces; select an exact path in settings"
                            }
                            .into());
                        }
                        found.remove(0)
                    };
                    let mut session = Session::new(
                        wanted,
                        transport::open(&device).map_err(|e| e.to_string())?,
                        true,
                    )
                    .map_err(|e| e.to_string())?;
                    if !send(
                        &tx,
                        &signal,
                        HardwareMessage::Status(format!("Connected: {wanted}")),
                    ) {
                        return Ok(());
                    }
                    while !signal.load(Ordering::Relaxed) {
                        let batch = session.read().map_err(|e| e.to_string())?;
                        for event in batch.events {
                            match event {
                                Ok(e) => {
                                    if !send(
                                        &tx,
                                        &signal,
                                        HardwareMessage::Input(e, Instant::now()),
                                    ) {
                                        return Ok(());
                                    }
                                }
                                Err(e) => {
                                    return Err(format!(
                                    "Hardware protocol error: {e}; reconnecting with input rearmed"
                                ))
                                }
                            }
                        }
                    }
                    Ok(())
                })();
                if let Err(error) = result {
                    if !send(&tx, &signal, HardwareMessage::Status(error)) {
                        break;
                    }
                }
                for _ in 0..30 {
                    if signal.load(Ordering::Relaxed) {
                        break;
                    }
                    thread::sleep(Duration::from_millis(100));
                }
            }
        })
        .map_err(|e| e.to_string())?;
    Ok(Reader {
        stop,
        join: Some(join),
        rx,
    })
}
fn save(
    store: &mut Store,
    state: &mut State,
    config: Config,
    revision: u64,
    engine: &mut Engine,
) -> Result<(), String> {
    if revision != state.revision {
        return Err("Configuration changed; reload before saving".into());
    }
    store.save(&config).map_err(|e| e.to_string())?;
    state.config = config;
    state.revision += 1;
    engine.reset();
    Ok(())
}
fn input(
    state: &mut State,
    store: &mut Store,
    engine: &mut Engine,
    audio: &mut Option<Controller<Native>>,
    event: ControlEvent,
) -> Result<(), String> {
    match event {
        ControlEvent::Analog { index, raw, .. } => {
            state.controls.insert(format!("analog_{index}"), raw);
        }
        ControlEvent::Button { index, pressed } => {
            state
                .controls
                .insert(format!("button_{index}"), u8::from(pressed));
        }
    };
    // Profile buttons remain available while audio is offline; generation 0 has no targets.
    let snapshot = if let Some(c) = audio {
        c.snapshot().map_err(|e| e.to_string())?
    } else {
        Snapshot {
            backend: "Unavailable".into(),
            generation: 0,
            targets: vec![],
        }
    };
    let effects = engine.handle(&state.config, event, &snapshot);
    for message in effects.diagnostics {
        note(state, message);
    }
    if let Some(id) = effects.profile {
        let mut config = state.config.clone();
        config.active_profile = id;
        save(store, state, config, state.revision, engine)?;
    }
    for (selection, change) in effects.audio {
        let receipt = audio
            .as_mut()
            .ok_or("Audio is unavailable")?
            .apply(&selection, change)
            .map_err(|e| e.to_string())?;
        if !receipt.confirmed {
            engine.reset();
            return Err("Audio change was not confirmed; input has been rearmed".into());
        }
    }
    Ok(())
}
fn command(
    cmd: Command,
    state: &mut State,
    store: &mut Store,
    engine: &mut Engine,
    audio: &mut Option<Controller<Native>>,
) -> Result<(), String> {
    match cmd {
        Command::Save { config, revision } => save(store, state, config, revision, engine),
        Command::Import { json, revision } => {
            let (config, _) = veek_config::decode(json.as_bytes()).map_err(|e| e.to_string())?;
            save(store, state, config, revision, engine)
        }
        Command::Activate { id } => {
            let mut config = state.config.clone();
            config.active_profile = id;
            save(store, state, config, state.revision, engine)
        }
        Command::Volume { selection, value } => write(audio, &selection, Change::Volume(value)),
        Command::Mute { selection, muted } => write(audio, &selection, Change::Mute(muted)),
        Command::MockAnalog { index, raw } => {
            if state.config.hardware.mode != HardwareMode::Mock {
                return Err("Enable development panel mode before sending simulated input".into());
            }
            let max = model(state.config.hardware.model).maximum();
            if index >= state.config.hardware.model.analog_count() || raw > max {
                return Err("Invalid simulated analog input".into());
            }
            input(
                state,
                store,
                engine,
                audio,
                ControlEvent::Analog {
                    index,
                    raw,
                    maximum: max,
                },
            )
        }
        Command::MockButton { index, pressed } => {
            if state.config.hardware.mode != HardwareMode::Mock {
                return Err("Enable development panel mode before sending simulated input".into());
            }
            if index >= state.config.hardware.model.button_count() {
                return Err("Invalid simulated button input".into());
            }
            input(
                state,
                store,
                engine,
                audio,
                ControlEvent::Button { index, pressed },
            )
        }
    }
}
fn write(
    audio: &mut Option<Controller<Native>>,
    selection: &Selection,
    change: Change,
) -> Result<(), String> {
    let receipt = audio
        .as_mut()
        .ok_or("Audio is unavailable")?
        .apply(selection, change)
        .map_err(|e| e.to_string())?;
    if !receipt.confirmed {
        return Err("Audio change was not confirmed by native readback".into());
    }
    Ok(())
}
fn run<F>(
    mut store: Store,
    shared: Arc<Mutex<State>>,
    rx: Receiver<Request>,
    stop: Arc<AtomicBool>,
    connect: F,
) where
    F: Fn() -> veek_audio::Result<Box<dyn Backend>>,
{
    let mut state = shared.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let mut engine = Engine::default();
    let mut audio = None;
    let mut retry = Instant::now();
    let mut refresh = Instant::now();
    let mut input_after = Instant::now();
    let mut input_revision = state.revision;
    let mut hardware = state.config.hardware.clone();
    let mut reader = reader(hardware.clone())
        .map_err(|e| note(&mut state, e))
        .ok();
    while !stop.load(Ordering::Relaxed) {
        if audio.is_none() && Instant::now() >= retry {
            match connect() {
                Ok(b) => {
                    audio = Some(Controller::new(Native(b)));
                    engine.reset();
                    input_after = Instant::now();
                    refresh = Instant::now();
                }
                Err(e) => {
                    state.audio_status = e.to_string();
                    note(&mut state, e.to_string());
                    retry = Instant::now() + Duration::from_secs(3);
                }
            }
        }
        let mut dirty = false;
        for request in rx.try_iter().take(32) {
            let result = if Instant::now() > request.deadline {
                Err("Command expired without execution".into())
            } else {
                command(
                    request.command,
                    &mut state,
                    &mut store,
                    &mut engine,
                    &mut audio,
                )
            };
            if let Err(e) = &result {
                note(&mut state, e.clone());
                engine.reset();
            }
            // Publish saved state before acknowledging, so an immediate UI refresh sees it.
            *shared.lock().unwrap_or_else(|e| e.into_inner()) = state.clone();
            let _ = request.reply.try_send(result);
            dirty = true;
        }
        if input_revision != state.revision {
            // Reports collected before a saved mapping/profile change belong to the old intent.
            input_after = Instant::now();
            input_revision = state.revision;
        }
        if hardware != state.config.hardware {
            reader.take();
            engine.reset();
            state.controls.clear();
            hardware = state.config.hardware.clone();
            reader = crate::reader(hardware.clone())
                .map_err(|e| note(&mut state, e))
                .ok();
        }
        let mut inputs = vec![];
        if let Some(r) = &reader {
            for message in r.rx.try_iter().take(256) {
                match message {
                    HardwareMessage::Status(s) => {
                        // A connection boundary invalidates queued input from the old session.
                        inputs.clear();
                        state.hardware_status = s;
                        state.controls.clear();
                        engine.reset();
                        dirty = true;
                    }
                    HardwareMessage::Input(e, at) => {
                        if at < input_after {
                            continue;
                        }
                        if at.elapsed() > Duration::from_millis(250) {
                            inputs.clear();
                            engine.reset();
                            note(
                                &mut state,
                                "Hardware input backlog expired; pickup/button state rearmed"
                                    .into(),
                            );
                        } else {
                            inputs.push(e);
                        }
                    }
                }
            }
        }
        for e in veek_core::coalesce(inputs) {
            if let Err(err) = input(&mut state, &mut store, &mut engine, &mut audio, e) {
                note(&mut state, err);
                engine.reset();
            }
            dirty = true;
            if state.revision != input_revision {
                // A profile button also invalidates the remainder of this captured batch.
                input_after = Instant::now();
                input_revision = state.revision;
                break;
            }
        }
        if dirty || Instant::now() >= refresh {
            if let Some(c) = &mut audio {
                match c.snapshot() {
                    Ok(s) => {
                        state.audio = Some(s);
                        state.audio_status = "Connected".into();
                    }
                    Err(e) => {
                        state.audio = None;
                        state.audio_status = e.to_string();
                        note(&mut state, e.to_string());
                        audio = None;
                        engine.reset();
                        retry = Instant::now() + Duration::from_secs(3);
                    }
                }
            }
            *shared.lock().unwrap_or_else(|e| e.into_inner()) = state.clone();
            refresh = Instant::now() + Duration::from_millis(250);
        }
        if let Some(c) = &mut audio {
            if let Err(e) = c.wait(Duration::from_millis(20)) {
                state.audio = None;
                state.audio_status = e.to_string();
                note(&mut state, e.to_string());
                audio = None;
                engine.reset();
                retry = Instant::now() + Duration::from_secs(3);
                refresh = Instant::now();
            }
        } else {
            thread::sleep(Duration::from_millis(20));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use veek_audio::{Kind, Target};
    struct Mock(Arc<Mutex<Target>>);
    impl Backend for Mock {
        fn name(&self) -> &'static str {
            "explicit test mock"
        }
        fn snapshot(&mut self) -> veek_audio::Result<Vec<Target>> {
            Ok(vec![self.0.lock().unwrap().clone()])
        }
        fn wait(&mut self, d: Duration) -> veek_audio::Result<()> {
            thread::sleep(d);
            Ok(())
        }
        fn write(&mut self, id: &str, c: Change) -> veek_audio::Result<()> {
            let mut t = self.0.lock().unwrap();
            if id != t.id {
                return Err(veek_audio::Error::Missing(id.into()));
            }
            match c {
                Change::Volume(v) => t.volume = Some(v),
                Change::Mute(m) => t.muted = Some(m),
            };
            Ok(())
        }
    }
    fn target() -> Target {
        Target {
            id: "test-live-output".into(),
            name: "Explicit test output".into(),
            kind: Kind::Output,
            default: true,
            volume: Some(0.5),
            muted: Some(false),
            identity: BTreeMap::from([("node.name".into(), "test-output".into())]),
        }
    }
    fn wait_for(handle: &Handle, condition: impl Fn(&State) -> bool) {
        let start = Instant::now();
        while !condition(&handle.state()) {
            assert!(start.elapsed() < Duration::from_secs(3));
            thread::sleep(Duration::from_millis(10));
        }
    }
    #[test]
    fn background_persists_mappings_and_rejects_stale_ui_and_unarmed_input() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let t = Arc::new(Mutex::new(target()));
        let shared = t.clone();
        let runtime = start_with(path.clone(), move || Ok(Box::new(Mock(shared.clone())))).unwrap();
        let h = &runtime.handle;
        wait_for(h, |s| s.audio.is_some());
        assert!(h
            .request(Command::MockButton {
                index: 0,
                pressed: true
            })
            .is_err());
        let before = h.state();
        let mut c = before.config.clone();
        c.hardware.mode = HardwareMode::Mock;
        c.profiles[0].mappings.push(veek_config::Mapping {
            control: veek_config::Control {
                device: "primary".into(),
                kind: veek_config::ControlKind::Analog,
                index: 0,
            },
            action: veek_config::Action::Volume {
                target: veek_config::Selector::DefaultOutput,
            },
        });
        h.request(Command::Save {
            revision: before.revision,
            config: c.clone(),
        })
        .unwrap();
        wait_for(h, |s| s.hardware_status.starts_with("Development panel"));
        assert!(h
            .request(Command::Save {
                revision: before.revision,
                config: c.clone()
            })
            .is_err());
        h.request(Command::MockAnalog { index: 0, raw: 10 })
            .unwrap();
        assert_eq!(t.lock().unwrap().volume, Some(0.5));
        h.request(Command::MockAnalog { index: 0, raw: 200 })
            .unwrap();
        assert!((t.lock().unwrap().volume.unwrap() - 200. / 255.).abs() < 0.001);
        let stale = Selection {
            generation: h.state().audio.as_ref().unwrap().generation + 1,
            id: "test-live-output".into(),
        };
        assert!(h
            .request(Command::Mute {
                selection: stale,
                muted: true
            })
            .is_err());
        assert_eq!(t.lock().unwrap().muted, Some(false));
        drop(runtime);
        let (store, loaded) = Store::open(path).unwrap();
        assert_eq!(loaded, c);
        drop(store);
    }
    #[test]
    fn offline_profile_actions_still_work_and_corrupt_config_is_never_reset() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let runtime = start_with(path.clone(), || {
            Err(veek_audio::Error::Unavailable("test outage".into()))
        })
        .unwrap();
        let s = runtime.handle.state();
        let mut c = s.config;
        c.profiles.push(veek_config::Profile {
            id: "second".into(),
            name: "Second".into(),
            mappings: vec![],
            groups: vec![],
            preferences: Default::default(),
        });
        runtime
            .handle
            .request(Command::Save {
                revision: s.revision,
                config: c,
            })
            .unwrap();
        runtime
            .handle
            .request(Command::Activate {
                id: "second".into(),
            })
            .unwrap();
        assert_eq!(runtime.handle.state().config.active_profile, "second");
        drop(runtime);
        std::fs::write(&path, b"damaged").unwrap();
        assert!(
            start_with(path.clone(), || Err(veek_audio::Error::Unavailable(
                "test".into()
            )))
            .is_err()
        );
        assert_eq!(std::fs::read(path).unwrap(), b"damaged");
    }
}
