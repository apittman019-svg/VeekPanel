//! Versioned user configuration and atomic, conflict-aware persistence.
#![forbid(unsafe_code)]
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};
use veek_audio::Kind;
pub const SCHEMA: u32 = 1;
pub const MAX_BYTES: u64 = 1_048_576;
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("configuration I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid configuration: {0}")]
    Invalid(String),
    #[error("unsupported configuration schema {0}; file was not changed")]
    Future(u64),
    #[error("another VeekPanel instance owns this configuration")]
    Locked,
    #[error("configuration changed outside the application; refusing to overwrite it")]
    Conflict,
}
pub type Result<T> = std::result::Result<T, Error>;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Selector {
    DefaultOutput,
    DefaultInput,
    PreferredOutput,
    PreferredInput,
    /// Exact durable alternatives (for example Linux app ID and Windows executable path).
    Match {
        kind: Kind,
        identities: BTreeMap<String, String>,
    },
    Group {
        id: String,
    },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Volume { target: Selector },
    ToggleMute { target: Selector },
    SetMute { target: Selector, muted: bool },
    SwitchProfile { id: String },
    NextProfile,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlKind {
    Analog,
    Button,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Control {
    pub device: String,
    pub kind: ControlKind,
    pub index: u8,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mapping {
    pub control: Control,
    pub action: Action,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub mappings: Vec<Mapping>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub members: Vec<Selector>,
    pub relative: bool,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HardwareMode {
    #[default]
    Disabled,
    Mock,
    Serial,
    Hid,
    AutoHid,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Panel {
    Original,
    Rgb,
    #[default]
    Mini,
    Pro,
}
impl Panel {
    pub fn analog_count(self) -> u8 {
        if self == Self::Pro {
            9
        } else {
            4
        }
    }
    pub fn button_count(self) -> u8 {
        if self == Self::Pro {
            5
        } else {
            4
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hardware {
    pub mode: HardwareMode,
    pub model: Panel,
    pub address: String,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preferences {
    pub output: Option<Selector>,
    pub input: Option<Selector>,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub theme: Theme,
    pub close_to_tray: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub schema_version: u32,
    pub active_profile: String,
    pub profiles: Vec<Profile>,
    pub groups: Vec<Group>,
    pub hardware: Hardware,
    pub preferences: Preferences,
    pub settings: Settings,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA,
            active_profile: "default".into(),
            profiles: vec![Profile {
                id: "default".into(),
                name: "Everyday".into(),
                mappings: vec![],
            }],
            groups: vec![],
            hardware: Hardware::default(),
            preferences: Preferences::default(),
            settings: Settings::default(),
        }
    }
}
fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(message.into())
}
fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 80
        && id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))
}
fn valid_name(name: &str) -> bool {
    !name.trim().is_empty() && name.len() <= 160 && !name.chars().any(char::is_control)
}
const KEYS: &[&str] = &[
    "application.id",
    "application.path",
    "application.process.binary",
    "endpoint.id",
    "node.name",
];
impl Config {
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != SCHEMA {
            return Err(Error::Future(self.schema_version.into()));
        }
        if self.profiles.is_empty() || self.profiles.len() > 64 || self.groups.len() > 128 {
            return Err(invalid("expected 1..64 profiles and at most 128 groups"));
        }
        let profiles: BTreeSet<_> = self.profiles.iter().map(|p| p.id.as_str()).collect();
        let groups: BTreeSet<_> = self.groups.iter().map(|g| g.id.as_str()).collect();
        if profiles.len() != self.profiles.len()
            || groups.len() != self.groups.len()
            || !profiles.contains(self.active_profile.as_str())
        {
            return Err(invalid("duplicate IDs or missing active profile"));
        }
        let check = |selector: &Selector, allow_group: bool| -> Result<()> {
            match selector {
                Selector::Match { kind, identities } => {
                    if identities.is_empty() || identities.len() > 5 {
                        return Err(invalid("a durable identity is required"));
                    }
                    for (key, value) in identities {
                        if !KEYS.contains(&key.as_str())
                            || value.is_empty()
                            || value.len() > 4096
                            || value.contains('\0')
                        {
                            return Err(invalid("invalid identity; PID and live node/session IDs cannot be persisted"));
                        }
                        let device = matches!(kind, Kind::Input | Kind::Output);
                        if device != matches!(key.as_str(), "endpoint.id" | "node.name") {
                            return Err(invalid("identity key does not match target kind"));
                        }
                    }
                }
                Selector::Group { id } if !allow_group || !groups.contains(id.as_str()) => {
                    return Err(invalid("missing/nested audio group"))
                }
                _ => (),
            }
            Ok(())
        };
        for group in &self.groups {
            if !valid_id(&group.id)
                || !valid_name(&group.name)
                || group.members.is_empty()
                || group.members.len() > 64
            {
                return Err(invalid("invalid audio group"));
            }
            for member in &group.members {
                check(member, false)?;
            }
        }
        for profile in &self.profiles {
            if !valid_id(&profile.id) || !valid_name(&profile.name) || profile.mappings.len() > 128
            {
                return Err(invalid("invalid profile"));
            }
            let mut controls = BTreeSet::new();
            for mapping in &profile.mappings {
                let c = &mapping.control;
                if c.device != "primary"
                    || c.index >= if c.kind == ControlKind::Analog { 9 } else { 5 }
                    || !controls.insert(c)
                {
                    return Err(invalid("invalid/duplicate physical control mapping"));
                }
                match &mapping.action {
                    Action::Volume { target } => {
                        if c.kind != ControlKind::Analog {
                            return Err(invalid("volume requires an analog control"));
                        }
                        check(target, true)?;
                    }
                    Action::ToggleMute { target } | Action::SetMute { target, .. } => {
                        if c.kind != ControlKind::Button {
                            return Err(invalid("mute action requires a button"));
                        }
                        check(target, true)?;
                    }
                    Action::SwitchProfile { id } => {
                        if c.kind != ControlKind::Button || !profiles.contains(id.as_str()) {
                            return Err(invalid("invalid profile-switch action"));
                        }
                    }
                    Action::NextProfile => {
                        if c.kind != ControlKind::Button {
                            return Err(invalid("profile action requires a button"));
                        }
                    }
                }
            }
        }
        for (preference, kind) in [
            (&self.preferences.output, Kind::Output),
            (&self.preferences.input, Kind::Input),
        ] {
            if let Some(selector) = preference {
                check(selector, false)?;
                if !matches!(selector,Selector::Match{kind:k,..} if *k==kind) {
                    return Err(invalid(
                        "device preference must name an output/input device",
                    ));
                }
            }
        }
        if self.hardware.address.len() > 4096 || self.hardware.address.contains('\0') {
            return Err(invalid("invalid hardware address"));
        }
        if matches!(self.hardware.mode, HardwareMode::Serial | HardwareMode::Hid)
            && self.hardware.address.trim().is_empty()
        {
            return Err(invalid("an explicit hardware address is required"));
        }
        if self.hardware.mode == HardwareMode::Serial && self.hardware.model != Panel::Original {
            return Err(invalid("serial mode is only implemented for Original"));
        }
        if matches!(
            self.hardware.mode,
            HardwareMode::Hid | HardwareMode::AutoHid
        ) && self.hardware.model == Panel::Original
        {
            return Err(invalid(
                "Original HID identity is unverified; select its serial port explicitly",
            ));
        }
        Ok(())
    }
}
/// Version zero is the documented pre-release profile-only format. No input commands
/// are executed during migration/import. Unknown/newer versions are left untouched.
pub fn decode(bytes: &[u8]) -> Result<(Config, bool)> {
    if bytes.len() as u64 > MAX_BYTES {
        return Err(invalid("configuration exceeds 1 MiB"));
    }
    let mut value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| invalid(e.to_string()))?;
    let version = value
        .get("schema_version")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| invalid("missing schema_version"))?;
    let migrated = version == 0;
    if migrated {
        let root = value
            .as_object_mut()
            .ok_or_else(|| invalid("expected object"))?;
        for (key, data) in [
            (
                "hardware",
                serde_json::to_value(Hardware::default()).unwrap(),
            ),
            (
                "preferences",
                serde_json::to_value(Preferences::default()).unwrap(),
            ),
            (
                "settings",
                serde_json::to_value(Settings::default()).unwrap(),
            ),
        ] {
            root.entry(key).or_insert(data);
        }
        root.insert("schema_version".into(), SCHEMA.into());
    } else if version != SCHEMA as u64 {
        return Err(Error::Future(version));
    }
    let config: Config = serde_json::from_value(value).map_err(|e| invalid(e.to_string()))?;
    config.validate()?;
    Ok((config, migrated))
}
pub struct Store {
    path: PathBuf,
    expected: Option<Vec<u8>>,
    _lock: File,
}
impl Store {
    pub fn open(path: impl Into<PathBuf>) -> Result<(Self, Config)> {
        let path = path.into();
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        fs::create_dir_all(parent)?;
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path.with_extension("lock"))?;
        lock.try_lock().map_err(|_| Error::Locked)?;
        let expected = read_optional(&path)?;
        let (config, migrated) = if let Some(bytes) = &expected {
            decode(bytes)?
        } else {
            (Config::default(), false)
        };
        let mut store = Self {
            path,
            expected,
            _lock: lock,
        };
        if migrated || store.expected.is_none() {
            store.save(&config)?;
        }
        Ok((store, config))
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn save(&mut self, config: &Config) -> Result<()> {
        config.validate()?;
        if read_optional(&self.path)? != self.expected {
            return Err(Error::Conflict);
        }
        let mut bytes = serde_json::to_vec_pretty(config).map_err(|e| invalid(e.to_string()))?;
        bytes.push(b'\n');
        if bytes.len() as u64 > MAX_BYTES {
            return Err(invalid("configuration exceeds 1 MiB"));
        }
        if let Some(previous) = &self.expected {
            atomic_write(&self.path.with_extension("json.bak"), previous)?;
        }
        atomic_write(&self.path, &bytes)?;
        self.expected = Some(bytes);
        Ok(())
    }
}
fn read_optional(path: &Path) -> Result<Option<Vec<u8>>> {
    match File::open(path) {
        Ok(file) => {
            let mut bytes = Vec::new();
            file.take(MAX_BYTES + 1).read_to_end(&mut bytes)?;
            if bytes.len() as u64 > MAX_BYTES {
                return Err(invalid("configuration exceeds 1 MiB"));
            }
            Ok(Some(bytes))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}
fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut temp = tempfile::NamedTempFile::new_in(parent)?;
    temp.write_all(bytes)?;
    temp.as_file().sync_all()?;
    temp.persist(path).map_err(|e| Error::Io(e.error))?;
    #[cfg(unix)]
    File::open(parent)?.sync_all()?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn migration_backup_and_future_rejection() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let mut v = serde_json::to_value(Config::default()).unwrap();
        v["schema_version"] = 0.into();
        v.as_object_mut().unwrap().remove("hardware");
        let original = serde_json::to_vec(&v).unwrap();
        fs::write(&path, &original).unwrap();
        let (store, c) = Store::open(&path).unwrap();
        assert_eq!(c.schema_version, SCHEMA);
        assert_eq!(fs::read(path.with_extension("json.bak")).unwrap(), original);
        drop(store);
        v["schema_version"] = 99.into();
        let bytes = serde_json::to_vec(&v).unwrap();
        fs::write(&path, &bytes).unwrap();
        assert!(matches!(Store::open(&path), Err(Error::Future(99))));
        assert_eq!(fs::read(&path).unwrap(), bytes);
    }
    #[test]
    fn lock_conflict_and_backup() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let (mut store, mut c) = Store::open(&path).unwrap();
        assert!(matches!(Store::open(&path), Err(Error::Locked)));
        let before = fs::read(&path).unwrap();
        c.profiles[0].name = "Changed".into();
        store.save(&c).unwrap();
        assert_eq!(fs::read(path.with_extension("json.bak")).unwrap(), before);
        fs::write(&path, b"external edit").unwrap();
        assert!(matches!(store.save(&c), Err(Error::Conflict)));
        assert_eq!(fs::read(&path).unwrap(), b"external edit");
    }
    #[test]
    fn invalid_config_never_replaced() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        fs::write(&path, b"broken").unwrap();
        assert!(Store::open(&path).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"broken");
        let mut c = Config::default();
        c.profiles[0].mappings.push(Mapping {
            control: Control {
                device: "primary".into(),
                kind: ControlKind::Analog,
                index: 0,
            },
            action: Action::Volume {
                target: Selector::Match {
                    kind: Kind::Playback,
                    identities: BTreeMap::from([("process.id".into(), "42".into())]),
                },
            },
        });
        assert!(c.validate().is_err());
    }
    #[test]
    fn groups_no_cycles_and_profile_references() {
        let mut c = Config::default();
        c.groups.push(Group {
            id: "cycle".into(),
            name: "Invalid".into(),
            members: vec![Selector::Group { id: "cycle".into() }],
            relative: true,
        });
        assert!(c.validate().is_err());
    }
}
