//! Windows Core Audio. COM is initialized and released on the owning MTA worker.
#![cfg(windows)]
#![allow(non_snake_case)]
use std::{
    collections::{BTreeMap, BTreeSet},
    marker::PhantomData,
    rc::Rc,
    sync::{mpsc, Arc, Condvar, Mutex},
    time::Duration,
};
use veek_audio::{Backend, Change, Error, Kind, Result, Target};
use windows::{
    core::{implement, AgileReference, Interface, BOOL, GUID, PCWSTR, PWSTR},
    Win32::{
        Devices::FunctionDiscovery::PKEY_Device_FriendlyName,
        Foundation::{CloseHandle, PROPERTYKEY},
        Media::Audio::{Endpoints::*, *},
        System::{
            Com::{
                StructuredStorage::{PropVariantClear, PropVariantToStringAlloc},
                *,
            },
            Threading::*,
        },
    },
};
const CONTEXT: GUID = GUID::from_u128(0x92537656_c69f_4a28_899d_bbe89003cfa0);
fn native(e: windows::core::Error) -> Error {
    Error::Unavailable(format!("Core Audio: {e}"))
}
#[derive(Default)]
struct Wake {
    changed: Mutex<bool>,
    signal: Condvar,
}
impl Wake {
    fn notify(&self) -> windows::core::Result<()> {
        if let Ok(mut value) = self.changed.lock() {
            *value = true;
            self.signal.notify_one();
        }
        Ok(())
    }
    fn wait(&self, duration: Duration) {
        if let Ok(value) = self.changed.lock() {
            if let Ok((mut value, _)) = self
                .signal
                .wait_timeout_while(value, duration, |changed| !*changed)
            {
                *value = false;
            }
        }
    }
}
#[implement(
    IMMNotificationClient,
    IAudioEndpointVolumeCallback,
    IAudioSessionEvents
)]
struct Notifications(Arc<Wake>);
impl IMMNotificationClient_Impl for Notifications_Impl {
    fn OnDeviceStateChanged(&self, _: &PCWSTR, _: DEVICE_STATE) -> windows::core::Result<()> {
        self.0.notify()
    }
    fn OnDeviceAdded(&self, _: &PCWSTR) -> windows::core::Result<()> {
        self.0.notify()
    }
    fn OnDeviceRemoved(&self, _: &PCWSTR) -> windows::core::Result<()> {
        self.0.notify()
    }
    fn OnDefaultDeviceChanged(
        &self,
        _: EDataFlow,
        _: ERole,
        _: &PCWSTR,
    ) -> windows::core::Result<()> {
        self.0.notify()
    }
    fn OnPropertyValueChanged(&self, _: &PCWSTR, _: &PROPERTYKEY) -> windows::core::Result<()> {
        self.0.notify()
    }
}
impl IAudioEndpointVolumeCallback_Impl for Notifications_Impl {
    fn OnNotify(&self, _: *mut AUDIO_VOLUME_NOTIFICATION_DATA) -> windows::core::Result<()> {
        self.0.notify()
    }
}
impl IAudioSessionEvents_Impl for Notifications_Impl {
    fn OnDisplayNameChanged(&self, _: &PCWSTR, _: *const GUID) -> windows::core::Result<()> {
        self.0.notify()
    }
    fn OnIconPathChanged(&self, _: &PCWSTR, _: *const GUID) -> windows::core::Result<()> {
        self.0.notify()
    }
    fn OnSimpleVolumeChanged(&self, _: f32, _: BOOL, _: *const GUID) -> windows::core::Result<()> {
        self.0.notify()
    }
    fn OnChannelVolumeChanged(
        &self,
        _: u32,
        _: *const f32,
        _: u32,
        _: *const GUID,
    ) -> windows::core::Result<()> {
        self.0.notify()
    }
    fn OnGroupingParamChanged(&self, _: *const GUID, _: *const GUID) -> windows::core::Result<()> {
        self.0.notify()
    }
    fn OnStateChanged(&self, _: AudioSessionState) -> windows::core::Result<()> {
        self.0.notify()
    }
    fn OnSessionDisconnected(&self, _: AudioSessionDisconnectReason) -> windows::core::Result<()> {
        self.0.notify()
    }
}
#[implement(IAudioSessionNotification)]
struct SessionCreated {
    wake: Arc<Wake>,
    sender: mpsc::SyncSender<AgileReference<IAudioSessionControl>>,
}
impl IAudioSessionNotification_Impl for SessionCreated_Impl {
    fn OnSessionCreated(
        &self,
        session: windows::core::Ref<IAudioSessionControl>,
    ) -> windows::core::Result<()> {
        // Retain a marshaled reference until enumeration on the owning MTA. Never block a callback.
        if let Some(session) = session.as_ref() {
            if let Ok(reference) = AgileReference::new(session) {
                let _ = self.sender.try_send(reference);
            }
        }
        self.wake.notify()
    }
}
struct Apartment(PhantomData<Rc<()>>);
impl Apartment {
    fn new() -> Result<Self> {
        // SAFETY: this backend is constructed on a dedicated worker and !Send. Every
        // successful CoInitializeEx, including S_FALSE, is balanced by its guard.
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED)
                .ok()
                .map_err(native)?;
        }
        Ok(Self(PhantomData))
    }
}
impl Drop for Apartment {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}
struct Endpoint {
    device: IMMDevice,
    volume: IAudioEndpointVolume,
    callback: IAudioEndpointVolumeCallback,
    manager: IAudioSessionManager2,
    created: IAudioSessionNotification,
    pending: mpsc::Receiver<AgileReference<IAudioSessionControl>>,
    kind: Kind,
}
impl Drop for Endpoint {
    fn drop(&mut self) {
        // SAFETY: unregister the exact interfaces registered on these live objects.
        unsafe {
            let _ = self.volume.UnregisterControlChangeNotify(&self.callback);
            let _ = self.manager.UnregisterSessionNotification(&self.created);
        }
    }
}
struct Session {
    endpoint: String,
    control: IAudioSessionControl,
    volume: ISimpleAudioVolume,
    callback: IAudioSessionEvents,
}
impl Drop for Session {
    fn drop(&mut self) {
        unsafe {
            let _ = self
                .control
                .UnregisterAudioSessionNotification(&self.callback);
        }
    }
}
pub struct CoreAudio {
    sessions: BTreeMap<String, Session>,
    endpoints: BTreeMap<String, Endpoint>,
    enumerator: IMMDeviceEnumerator,
    callback: IMMNotificationClient,
    wake: Arc<Wake>,
    // Declared last: all COM objects must be released before CoUninitialize.
    _apartment: Apartment,
}
impl Drop for CoreAudio {
    fn drop(&mut self) {
        unsafe {
            let _ = self
                .enumerator
                .UnregisterEndpointNotificationCallback(&self.callback);
        }
    }
}
fn owned_string(value: PWSTR) -> String {
    // SAFETY: only CoTaskMem-allocated out strings from Core Audio/props enter here.
    // Copy before freeing; null is allowed by CoTaskMemFree.
    unsafe {
        let text = value.to_string().unwrap_or_default();
        CoTaskMemFree(Some(value.0.cast()));
        text
    }
}
fn device_name(device: &IMMDevice) -> String {
    // SAFETY: COM returns an initialized PROPVARIANT. Clear it on success or conversion failure.
    unsafe {
        let Ok(store) = device.OpenPropertyStore(STGM_READ) else {
            return "Audio endpoint".into();
        };
        let Ok(mut value) = store.GetValue(&PKEY_Device_FriendlyName) else {
            return "Audio endpoint".into();
        };
        let name = PropVariantToStringAlloc(&value)
            .map(owned_string)
            .unwrap_or_else(|_| "Audio endpoint".into());
        let _ = PropVariantClear(&mut value);
        name
    }
}
fn process_path(pid: u32) -> Option<String> {
    if pid == 0 {
        return None;
    }
    // SAFETY: handle is always closed, and Windows receives a buffer and its actual length.
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buffer = vec![0u16; 32768];
        let mut length = buffer.len() as u32;
        let result = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &mut length,
        );
        let _ = CloseHandle(handle);
        result.ok()?;
        Some(String::from_utf16_lossy(&buffer[..length as usize]))
    }
}
impl CoreAudio {
    pub fn connect() -> Result<Self> {
        let apartment = Apartment::new()?;
        // SAFETY: COM initialized on this thread, typed activation and no aggregation.
        let enumerator: IMMDeviceEnumerator =
            unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) }.map_err(native)?;
        let wake = Arc::new(Wake::default());
        let callback: IMMNotificationClient = Notifications(wake.clone()).into();
        unsafe { enumerator.RegisterEndpointNotificationCallback(&callback) }.map_err(native)?;
        Ok(Self {
            sessions: BTreeMap::new(),
            endpoints: BTreeMap::new(),
            enumerator,
            callback,
            wake,
            _apartment: apartment,
        })
    }
    fn endpoint(&self, device: IMMDevice, kind: Kind) -> Result<Endpoint> {
        // SAFETY: every interface is activated on the COM worker. Registration is
        // unwound on partial failure; the Endpoint guard unregisters on normal drop.
        unsafe {
            let volume: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None).map_err(native)?;
            let manager: IAudioSessionManager2 =
                device.Activate(CLSCTX_ALL, None).map_err(native)?;
            let callback: IAudioEndpointVolumeCallback = Notifications(self.wake.clone()).into();
            let (sender, pending) = mpsc::sync_channel(256);
            let created: IAudioSessionNotification = SessionCreated {
                wake: self.wake.clone(),
                sender,
            }
            .into();
            volume
                .RegisterControlChangeNotify(&callback)
                .map_err(native)?;
            if let Err(e) = manager.RegisterSessionNotification(&created) {
                let _ = volume.UnregisterControlChangeNotify(&callback);
                return Err(native(e));
            }
            Ok(Endpoint {
                device,
                volume,
                callback,
                manager,
                created,
                pending,
                kind,
            })
        }
    }
    fn reconcile(&mut self) -> Result<()> {
        // SAFETY: all pointers are typed COM objects on this MTA; string out-params
        // are copied/freed, callbacks only signal and marshal newly created sessions.
        unsafe {
            let mut live = BTreeSet::new();
            for (flow, kind) in [(eRender, Kind::Output), (eCapture, Kind::Input)] {
                let devices = self
                    .enumerator
                    .EnumAudioEndpoints(flow, DEVICE_STATE_ACTIVE)
                    .map_err(native)?;
                for i in 0..devices.GetCount().map_err(native)? {
                    let device = devices.Item(i).map_err(native)?;
                    let id = owned_string(device.GetId().map_err(native)?);
                    live.insert(id.clone());
                    if !self.endpoints.contains_key(&id) {
                        let endpoint = self.endpoint(device, kind)?;
                        self.endpoints.insert(id, endpoint);
                    }
                }
            }
            self.sessions.retain(|_, s| live.contains(&s.endpoint));
            self.endpoints.retain(|id, _| live.contains(id));
            let mut controls = Vec::new();
            for (id, endpoint) in &self.endpoints {
                // GetCount after registering enables notifications per Core Audio contract.
                let enumerator = endpoint.manager.GetSessionEnumerator().map_err(native)?;
                for i in 0..enumerator.GetCount().map_err(native)? {
                    controls.push((id.clone(), enumerator.GetSession(i).map_err(native)?));
                }
                for reference in endpoint.pending.try_iter() {
                    controls.push((id.clone(), reference.resolve().map_err(native)?));
                }
            }
            for (endpoint, control) in controls {
                if control.GetState().map_err(native)? == AudioSessionStateExpired {
                    continue;
                }
                let control2: IAudioSessionControl2 = control.cast().map_err(native)?;
                let instance =
                    owned_string(control2.GetSessionInstanceIdentifier().map_err(native)?);
                let id = format!("session:{}:{instance}", endpoint);
                if let std::collections::btree_map::Entry::Vacant(entry) = self.sessions.entry(id) {
                    let volume: ISimpleAudioVolume = control.cast().map_err(native)?;
                    let callback: IAudioSessionEvents = Notifications(self.wake.clone()).into();
                    control
                        .RegisterAudioSessionNotification(&callback)
                        .map_err(native)?;
                    entry.insert(Session {
                        endpoint,
                        control,
                        volume,
                        callback,
                    });
                }
            }
            self.sessions.retain(|_, s| {
                s.control
                    .GetState()
                    .is_ok_and(|state| state != AudioSessionStateExpired)
            });
        }
        Ok(())
    }
}
impl Backend for CoreAudio {
    fn name(&self) -> &'static str {
        "Windows Core Audio"
    }
    fn snapshot(&mut self) -> Result<Vec<Target>> {
        self.reconcile()?;
        let mut targets = Vec::new();
        // SAFETY: COM worker only; reads have no system mutations. Scalar APIs preserve
        // native endpoint/session channel relationships. No undocumented policy APIs.
        unsafe {
            let default_output = self
                .enumerator
                .GetDefaultAudioEndpoint(eRender, eMultimedia)
                .ok()
                .and_then(|d| d.GetId().ok())
                .map(owned_string);
            let default_input = self
                .enumerator
                .GetDefaultAudioEndpoint(eCapture, eMultimedia)
                .ok()
                .and_then(|d| d.GetId().ok())
                .map(owned_string);
            for (id, e) in &self.endpoints {
                targets.push(Target {
                    id: format!("endpoint:{id}"),
                    name: device_name(&e.device),
                    kind: e.kind,
                    default: Some(id)
                        == if e.kind == Kind::Output {
                            default_output.as_ref()
                        } else {
                            default_input.as_ref()
                        },
                    volume: Some(e.volume.GetMasterVolumeLevelScalar().map_err(native)?),
                    muted: Some(e.volume.GetMute().map_err(native)?.as_bool()),
                    identity: BTreeMap::from([("endpoint.id".into(), id.clone())]),
                });
            }
            for (id, s) in &self.sessions {
                let c: IAudioSessionControl2 = s.control.cast().map_err(native)?;
                let pid = c.GetProcessId().unwrap_or(0);
                let mut identity = BTreeMap::from([
                    ("endpoint.id".into(), s.endpoint.clone()),
                    ("process.id".into(), pid.to_string()),
                ]);
                if let Ok(value) = c.GetSessionIdentifier() {
                    identity.insert("session.identifier".into(), owned_string(value));
                }
                let path = process_path(pid);
                if let Some(path) = &path {
                    identity.insert("application.path".into(), path.clone());
                }
                let display = s
                    .control
                    .GetDisplayName()
                    .map(owned_string)
                    .unwrap_or_default();
                let name = if !display.is_empty() {
                    display
                } else if let Some(path) = path {
                    path.rsplit('\\').next().unwrap_or(&path).into()
                } else if c.IsSystemSoundsSession() == windows::core::HRESULT(0) {
                    "System sounds".into()
                } else {
                    format!("Audio session (PID {pid}; identity unavailable)")
                };
                let kind = if self.endpoints[&s.endpoint].kind == Kind::Output {
                    Kind::Playback
                } else {
                    Kind::Recording
                };
                targets.push(Target {
                    id: id.clone(),
                    name,
                    kind,
                    default: false,
                    volume: Some(s.volume.GetMasterVolume().map_err(native)?),
                    muted: Some(s.volume.GetMute().map_err(native)?.as_bool()),
                    identity,
                });
            }
        }
        Ok(targets)
    }
    fn write(&mut self, id: &str, change: Change) -> Result<()> {
        change.validate()?;
        self.reconcile()?;
        // SAFETY: scalar validated before FFI; context pointer is a static GUID;
        // lookup uses the exact live endpoint/session instance, never PID matching.
        unsafe {
            if let Some(e) = id
                .strip_prefix("endpoint:")
                .and_then(|id| self.endpoints.get(id))
            {
                match change {
                    Change::Volume(v) => e.volume.SetMasterVolumeLevelScalar(v, &CONTEXT),
                    Change::Mute(m) => e.volume.SetMute(m, &CONTEXT),
                }
                .map_err(native)
            } else if let Some(s) = self.sessions.get(id) {
                match change {
                    Change::Volume(v) => s.volume.SetMasterVolume(v, &CONTEXT),
                    Change::Mute(m) => s.volume.SetMute(m, &CONTEXT),
                }
                .map_err(native)
            } else {
                Err(Error::Missing(id.into()))
            }
        }
    }
    fn wait(&mut self, timeout: Duration) -> Result<()> {
        self.wake.wait(timeout);
        Ok(())
    }
}
