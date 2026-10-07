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
pub const SCHEMA: u32 = 2;
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
    pub groups: Vec<Group>,
    pub preferences: Preferences,
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
    pub hardware: Hardware,
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
                groups: vec![],
                preferences: Preferences::default(),
            }],
            hardware: Hardware::default(),
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
    pub fn active(&self) -> Option<&Profile> {
        self.profiles.iter().find(|p| p.id == self.active_profile)
    }
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != SCHEMA {
            return Err(Error::Future(self.schema_version.into()));
        }
        let profiles = profile_ids(
            &self.active_profile,
            self.profiles.iter().map(|p| p.id.as_str()),
        )?;
        for profile in &self.profiles {
            validate_profile(
                &profile.id,
                &profile.name,
                &profile.mappings,
                &profile.groups,
                &profile.preferences,
                &profiles,
            )?;
        }
        validate_hardware(&self.hardware)
    }
}
fn profile_ids<'a>(
    active: &str,
    ids: impl ExactSizeIterator<Item = &'a str>,
) -> Result<BTreeSet<&'a str>> {
    let count = ids.len();
    if count == 0 || count > 64 {
        return Err(invalid("expected 1..64 profiles"));
    }
    let profiles: BTreeSet<_> = ids.collect();
    if profiles.len() != count || !profiles.contains(active) {
        return Err(invalid("duplicate profile IDs or missing active profile"));
    }
    Ok(profiles)
}
fn validate_profile(
    id: &str,
    name: &str,
    mappings: &[Mapping],
    audio_groups: &[Group],
    preferences: &Preferences,
    profiles: &BTreeSet<&str>,
) -> Result<()> {
    if !valid_id(id) || !valid_name(name) || mappings.len() > 128 {
        return Err(invalid("invalid profile"));
    }
    if audio_groups.len() > 128 {
        return Err(invalid("expected at most 128 groups per profile"));
    }
    let groups: BTreeSet<_> = audio_groups.iter().map(|g| g.id.as_str()).collect();
    if groups.len() != audio_groups.len() {
        return Err(invalid("duplicate audio group IDs within a profile"));
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
                        return Err(invalid(
                            "invalid identity; PID and live node/session IDs cannot be persisted",
                        ));
                    }
                    let device = matches!(kind, Kind::Input | Kind::Output);
                    if device != matches!(key.as_str(), "endpoint.id" | "node.name") {
                        return Err(invalid("identity key does not match target kind"));
                    }
                }
            }
            Selector::Group { id } if !allow_group || !groups.contains(id.as_str()) => {
                return Err(invalid("missing/nested audio group in this profile"));
            }
            _ => (),
        }
        Ok(())
    };
    for group in audio_groups {
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
    let mut controls = BTreeSet::new();
    for mapping in mappings {
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
    for (preference, kind) in [
        (&preferences.output, Kind::Output),
        (&preferences.input, Kind::Input),
    ] {
        if let Some(selector) = preference {
            check(selector, false)?;
            if !matches!(selector, Selector::Match{kind: k, ..} if *k == kind) {
                return Err(invalid(
                    "device preference must name an output/input device",
                ));
            }
        }
    }
    Ok(())
}
fn validate_hardware(hardware: &Hardware) -> Result<()> {
    if hardware.address.len() > 4096 || hardware.address.contains('\0') {
        return Err(invalid("invalid hardware address"));
    }
    if matches!(hardware.mode, HardwareMode::Serial | HardwareMode::Hid)
        && hardware.address.trim().is_empty()
    {
        return Err(invalid("an explicit hardware address is required"));
    }
    if hardware.mode == HardwareMode::Serial && hardware.model != Panel::Original {
        return Err(invalid("serial mode is only implemented for Original"));
    }
    if matches!(hardware.mode, HardwareMode::Hid | HardwareMode::AutoHid)
        && hardware.model == Panel::Original
    {
        return Err(invalid(
            "Original HID identity is unverified; select its serial port explicitly",
        ));
    }
    Ok(())
}
// Decode the legacy shape separately: profile-local fields in a schema-0/1 file
// are rejected rather than silently replaced by shared data during migration.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyProfile {
    id: String,
    name: String,
    mappings: Vec<Mapping>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyConfig {
    schema_version: u32,
    active_profile: String,
    profiles: Vec<LegacyProfile>,
    groups: Vec<Group>,
    // Schema 0 allowed these sections to be omitted. Schema 1 checks presence
    // before decoding, retaining its original required-field contract.
    #[serde(default)]
    hardware: Hardware,
    #[serde(default)]
    preferences: Preferences,
    #[serde(default)]
    settings: Settings,
}
impl LegacyConfig {
    fn migrate(self) -> Result<Config> {
        if self.schema_version > 1 {
            return Err(Error::Future(self.schema_version.into()));
        }
        let profiles = profile_ids(
            &self.active_profile,
            self.profiles.iter().map(|p| p.id.as_str()),
        )?;
        // Validate the entire old document before copying any shared settings.
        for profile in &self.profiles {
            validate_profile(
                &profile.id,
                &profile.name,
                &profile.mappings,
                &self.groups,
                &self.preferences,
                &profiles,
            )?;
        }
        validate_hardware(&self.hardware)?;
        Ok(Config {
            schema_version: SCHEMA,
            active_profile: self.active_profile,
            profiles: self
                .profiles
                .into_iter()
                .map(|p| Profile {
                    id: p.id,
                    name: p.name,
                    mappings: p.mappings,
                    groups: self.groups.clone(),
                    preferences: self.preferences.clone(),
                })
                .collect(),
            hardware: self.hardware,
            settings: self.settings,
        })
    }
}
/// Versions zero and one stored groups/device preferences globally. Migration
/// copies them into every profile, preserving behavior and allowing later edits
/// independently. No input commands are executed; invalid/future files are untouched.
pub fn decode(bytes: &[u8]) -> Result<(Config, bool)> {
    if bytes.len() as u64 > MAX_BYTES {
        return Err(invalid("configuration exceeds 1 MiB"));
    }
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| invalid(e.to_string()))?;
    let version = value
        .get("schema_version")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| invalid("missing schema_version"))?;
    let config = match version {
        0 | 1 => {
            if version == 1 {
                for key in ["hardware", "preferences", "settings"] {
                    if value.get(key).is_none() {
                        return Err(invalid(format!("missing field `{key}`")));
                    }
                }
            }
            let legacy: LegacyConfig =
                serde_json::from_slice(bytes).map_err(|e| invalid(e.to_string()))?;
            legacy.migrate()?
        }
        v if v == SCHEMA as u64 => {
            serde_json::from_slice(bytes).map_err(|e| invalid(e.to_string()))?
        }
        other => return Err(Error::Future(other)),
    };
    config.validate()?;
    Ok((config, version != SCHEMA as u64))
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
    fn legacy(version: u32) -> serde_json::Value {
        serde_json::json!({
            "schema_version": version, "active_profile": "one",
            "profiles": [
                {"id":"one","name":"One","mappings":[{"control":{"device":"primary","kind":"analog","index":0},"action":{"type":"volume","target":{"type":"group","id":"mix"}}}]},
                {"id":"two","name":"Two","mappings":[]}
            ],
            "groups":[{"id":"mix","name":"My mix","members":[{"type":"preferred_output"}],"relative":true}],
            "preferences":{"output":{"type":"match","kind":"output","identities":{"node.name":"saved-output"}},"input":null},
            "hardware":{"mode":"disabled","model":"mini","address":""},
            "settings":{"theme":"dark","close_to_tray":false}
        })
    }
    #[test]
    fn legacy_migration_preserves_every_profile_and_original_backup() {
        for version in [0, 1] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("config.json");
            let mut value = legacy(version);
            if version == 0 {
                value.as_object_mut().unwrap().remove("hardware");
            }
            let original = serde_json::to_vec(&value).unwrap();
            fs::write(&path, &original).unwrap();
            let (store, mut c) = Store::open(&path).unwrap();
            assert_eq!(c.schema_version, SCHEMA);
            assert_eq!(fs::read(path.with_extension("json.bak")).unwrap(), original);
            assert_eq!(c.profiles[0].groups, c.profiles[1].groups);
            assert_eq!(c.profiles[0].preferences, c.profiles[1].preferences);
            assert_eq!(c.profiles[0].mappings.len(), 1);
            assert_eq!(c.settings.theme, Theme::Dark);
            assert_eq!(c.active_profile, "one");
            let (reloaded, migrated) = decode(&fs::read(&path).unwrap()).unwrap();
            assert!(!migrated);
            assert_eq!(c, reloaded);
            c.profiles[1].groups[0].name = "Independent".into();
            c.profiles[1].preferences.output = None;
            assert_eq!(c.profiles[0].groups[0].name, "My mix");
            assert!(c.profiles[0].preferences.output.is_some());
            drop(store);
        }
    }
    #[test]
    fn malformed_legacy_and_future_files_preserve_file_and_backup() {
        let mut future = legacy(99);
        future["unexpected"] = true.into();
        let mut unknown = legacy(1);
        unknown["profiles"][0]["groups"] = serde_json::json!([]);
        let mut missing = legacy(1);
        missing.as_object_mut().unwrap().remove("preferences");
        let mut bad_reference = legacy(1);
        bad_reference["groups"] = serde_json::json!([]);
        for value in [future, unknown, missing, bad_reference] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("config.json");
            let bytes = serde_json::to_vec(&value).unwrap();
            fs::write(&path, &bytes).unwrap();
            fs::write(path.with_extension("json.bak"), b"previous backup").unwrap();
            assert!(Store::open(&path).is_err());
            assert_eq!(fs::read(&path).unwrap(), bytes);
            assert_eq!(
                fs::read(path.with_extension("json.bak")).unwrap(),
                b"previous backup"
            );
        }
    }
    #[test]
    fn group_ids_and_references_are_local_to_each_profile() {
        let (mut c, _) = decode(&serde_json::to_vec(&legacy(1)).unwrap()).unwrap();
        c.profiles[1].mappings = c.profiles[0].mappings.clone();
        c.profiles[1].groups[0].members = vec![Selector::DefaultInput];
        c.validate().unwrap(); // Same group ID in two independent profiles is valid.
        c.profiles[1].groups.clear();
        assert!(c.validate().is_err()); // Cannot resolve against the other profile.
        c.profiles[1].mappings.clear();
        c.validate().unwrap();
        let duplicate = c.profiles[0].groups[0].clone();
        c.profiles[0].groups.push(duplicate);
        assert!(c.validate().is_err());
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
        c.profiles[0].groups.push(Group {
            id: "cycle".into(),
            name: "Invalid".into(),
            members: vec![Selector::Group { id: "cycle".into() }],
            relative: true,
        });
        assert!(c.validate().is_err());
    }
}
