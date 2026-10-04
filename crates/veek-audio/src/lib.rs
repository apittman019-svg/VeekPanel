//! Native audio contract. IDs identify live objects, never persisted application mappings.
#![forbid(unsafe_code)]
use serde::Serialize;
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum Error {
    #[error("audio backend unavailable: {0}")]
    Unavailable(String),
    #[error("audio target disappeared: {0}")]
    Missing(String),
    #[error("stale audio generation; select the target again")]
    Stale,
    #[error("unsupported audio operation: {0}")]
    Unsupported(String),
    #[error("audio operation failed: {0}")]
    Native(String),
    #[error("volume must be finite and between 0 and 1")]
    InvalidVolume,
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Output,
    Input,
    Playback,
    Recording,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Target {
    pub id: String,
    pub name: String,
    pub kind: Kind,
    pub default: bool,
    /// Observed native scalar mapped into the backend's documented slider range.
    /// Can exceed 1 if another application enabled amplification. Writes never do.
    pub volume: Option<f32>,
    pub muted: Option<bool>,
    /// Stable hints for future matching, not a promise of persistent identity.
    pub identity: BTreeMap<String, String>,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Snapshot {
    pub backend: String,
    pub generation: u64,
    pub targets: Vec<Target>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Selection {
    pub generation: u64,
    pub id: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub enum Change {
    Volume(f32),
    Mute(bool),
}
impl Change {
    pub fn validate(self) -> Result<()> {
        if let Self::Volume(v) = self {
            if !v.is_finite() || !(0.0..=1.0).contains(&v) {
                return Err(Error::InvalidVolume);
            }
        }
        Ok(())
    }
    pub fn matches(self, target: &Target) -> bool {
        match self {
            Self::Volume(v) => target
                .volume
                .is_some_and(|actual| (actual - v).abs() <= 0.005),
            Self::Mute(m) => target.muted == Some(m),
        }
    }
}
#[derive(Debug, Serialize)]
pub struct Receipt {
    pub requested: Change,
    pub observed: Target,
    pub confirmed: bool,
}

/// Construct and use on one dedicated thread. Native COM/PipeWire objects never cross it.
/// A backend reports native errors and observations; it must not synthesize success.
pub trait Backend {
    fn name(&self) -> &'static str;
    fn snapshot(&mut self) -> Result<Vec<Target>>;
    fn write(&mut self, id: &str, change: Change) -> Result<()>;
    /// Dispatch subscriptions, sleeping at most this long. Snapshots reconcile the cache.
    fn wait(&mut self, timeout: Duration) -> Result<()>;
}
static GENERATION: AtomicU64 = AtomicU64::new(1);
pub struct Controller<B> {
    backend: B,
    generation: u64,
    valid: bool,
}
impl<B: Backend> Controller<B> {
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            generation: GENERATION.fetch_add(1, Ordering::Relaxed),
            valid: true,
        }
    }
    pub fn snapshot(&mut self) -> Result<Snapshot> {
        if !self.valid {
            return Err(Error::Stale);
        }
        match self.backend.snapshot() {
            Ok(mut targets) => {
                targets.sort_by(|a, b| a.id.cmp(&b.id));
                Ok(Snapshot {
                    backend: self.backend.name().into(),
                    generation: self.generation,
                    targets,
                })
            }
            Err(e) => {
                self.valid = false;
                Err(e)
            }
        }
    }
    pub fn select(&mut self, selector: &str) -> Result<Selection> {
        let snapshot = self.snapshot()?;
        let target = resolve(&snapshot, selector)?;
        Ok(Selection {
            generation: self.generation,
            id: target.id.clone(),
        })
    }
    pub fn apply(&mut self, selection: &Selection, change: Change) -> Result<Receipt> {
        change.validate()?;
        if !self.valid || selection.generation != self.generation {
            return Err(Error::Stale);
        }
        let snapshot = self.snapshot()?;
        let current = resolve(&snapshot, &selection.id)?;
        match change {
            Change::Volume(_) if current.volume.is_none() => {
                return Err(Error::Unsupported("target has no volume control".into()))
            }
            Change::Mute(_) if current.muted.is_none() => {
                return Err(Error::Unsupported("target has no mute control".into()))
            }
            _ => (),
        }
        if let Err(error) = self.backend.write(&selection.id, change) {
            if matches!(error, Error::Unavailable(_)) {
                self.valid = false;
            }
            return Err(error);
        }
        let observed = resolve(&self.snapshot()?, &selection.id)?.clone();
        Ok(Receipt {
            requested: change,
            confirmed: change.matches(&observed),
            observed,
        })
    }
    pub fn wait(&mut self, timeout: Duration) -> Result<()> {
        if !self.valid {
            return Err(Error::Stale);
        }
        if let Err(e) = self.backend.wait(timeout) {
            self.valid = false;
            return Err(e);
        }
        Ok(())
    }
}
pub fn resolve<'a>(snapshot: &'a Snapshot, selector: &str) -> Result<&'a Target> {
    let mut matches = snapshot.targets.iter().filter(|t| match selector {
        "default-output" => t.kind == Kind::Output && t.default,
        "default-input" => t.kind == Kind::Input && t.default,
        _ => t.id == selector,
    });
    let target = matches
        .next()
        .ok_or_else(|| Error::Missing(selector.into()))?;
    if matches.next().is_some() {
        return Err(Error::Native("ambiguous target; use an exact ID".into()));
    }
    Ok(target)
}

/// Preserve relative channel balance, never amplify past the requested peak.
/// All-zero channels have no recoverable balance; restore them equally.
pub fn balanced_channels(channels: &[f32], peak: f32) -> Result<Vec<f32>> {
    Change::Volume(peak).validate()?;
    if channels.is_empty() || channels.iter().any(|v| !v.is_finite() || *v < 0.0) {
        return Err(Error::Unsupported("invalid channel volumes".into()));
    }
    let old = channels.iter().copied().fold(0.0_f32, f32::max);
    Ok(channels
        .iter()
        .map(|v| if old > 0.0 { v / old * peak } else { peak })
        .collect())
}

/// Temporary M2 diagnostic pickup, not a persistent mapping engine.
#[derive(Default)]
pub struct Pickup {
    previous: Option<f32>,
    applied: Option<f32>,
    caught: bool,
}
impl Pickup {
    pub fn update(&mut self, position: f32, observed: f32) -> Option<f32> {
        if Change::Volume(position).validate().is_err() || !observed.is_finite() {
            return None;
        }
        if self.applied.is_some_and(|a| (a - observed).abs() > 0.02) {
            self.caught = false;
            self.applied = None;
        }
        let previous = self.previous.replace(position)?;
        if (previous - position).abs() < 0.0001 {
            return None;
        }
        if !self.caught
            && ((previous - observed) * (position - observed) <= 0.0
                || (position - observed).abs() <= 0.02)
        {
            self.caught = true;
        }
        if self.caught {
            self.applied = Some(position);
            Some(position)
        } else {
            None
        }
    }
}
#[derive(Default)]
pub struct PressEdge {
    released: bool,
}
impl PressEdge {
    /// Startup/held presses are ignored until a release is observed.
    pub fn update(&mut self, pressed: bool) -> bool {
        if !pressed {
            self.released = true;
            false
        } else {
            std::mem::take(&mut self.released)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Mock {
        targets: Vec<Target>,
        fail: bool,
        writes: usize,
        ignore: bool,
    }
    impl Backend for Mock {
        fn name(&self) -> &'static str {
            "mock"
        }
        fn snapshot(&mut self) -> Result<Vec<Target>> {
            if self.fail {
                Err(Error::Unavailable("mock disconnect".into()))
            } else {
                Ok(self.targets.clone())
            }
        }
        fn wait(&mut self, _: Duration) -> Result<()> {
            Ok(())
        }
        fn write(&mut self, id: &str, c: Change) -> Result<()> {
            self.writes += 1;
            if !self.ignore {
                let t = self
                    .targets
                    .iter_mut()
                    .find(|t| t.id == id)
                    .ok_or_else(|| Error::Missing(id.into()))?;
                match c {
                    Change::Volume(v) => t.volume = Some(v),
                    Change::Mute(m) => t.muted = Some(m),
                }
            }
            Ok(())
        }
    }
    fn mock() -> Mock {
        Mock {
            targets: vec![Target {
                id: "endpoint".into(),
                name: "Synthetic".into(),
                kind: Kind::Output,
                default: true,
                volume: Some(0.5),
                muted: Some(false),
                identity: BTreeMap::new(),
            }],
            fail: false,
            writes: 0,
            ignore: false,
        }
    }
    #[test]
    fn finite_range_and_balance() {
        for v in [f32::NAN, f32::INFINITY, -0.1, 1.1] {
            assert!(Change::Volume(v).validate().is_err());
        }
        assert_eq!(balanced_channels(&[0.2, 0.4], 0.8).unwrap(), vec![0.4, 0.8]);
        assert_eq!(balanced_channels(&[0., 0.], 0.2).unwrap(), vec![0.2, 0.2]);
    }
    #[test]
    fn stale_missing_and_unsupported_never_write() {
        let mut c = Controller::new(mock());
        let s = c.select("default-output").unwrap();
        assert!(matches!(
            c.apply(
                &Selection {
                    generation: s.generation + 1,
                    id: s.id.clone()
                },
                Change::Mute(true)
            ),
            Err(Error::Stale)
        ));
        c.backend.targets.clear();
        assert!(matches!(
            c.apply(&s, Change::Mute(true)),
            Err(Error::Missing(_))
        ));
        assert_eq!(c.backend.writes, 0);
    }
    #[test]
    fn observed_receipts_and_external_changes() {
        let mut c = Controller::new(mock());
        let s = c.select("default-output").unwrap();
        assert!(c.apply(&s, Change::Volume(0.2)).unwrap().confirmed);
        c.backend.targets[0].muted = Some(true);
        assert_eq!(c.snapshot().unwrap().targets[0].muted, Some(true));
        c.backend.ignore = true;
        assert!(!c.apply(&s, Change::Mute(false)).unwrap().confirmed);
    }
    #[test]
    fn disconnect_invalidates_generation() {
        let mut c = Controller::new(mock());
        let s = c.select("default-output").unwrap();
        c.backend.fail = true;
        assert!(c.snapshot().is_err());
        c.backend.fail = false;
        assert_eq!(c.apply(&s, Change::Mute(true)).unwrap_err(), Error::Stale);
        let mut next = Controller::new(mock());
        assert_eq!(
            next.apply(&s, Change::Mute(true)).unwrap_err(),
            Error::Stale
        );
    }
    #[test]
    fn capability_missing_and_ambiguous_defaults() {
        let mut m = mock();
        m.targets[0].volume = None;
        let mut c = Controller::new(m);
        let s = c.select("default-output").unwrap();
        assert!(matches!(
            c.apply(&s, Change::Volume(0.1)),
            Err(Error::Unsupported(_))
        ));
        let mut duplicate = c.backend.targets[0].clone();
        duplicate.id = "another".into();
        c.backend.targets.push(duplicate);
        assert!(c.select("default-output").is_err());
    }
    #[test]
    fn pickup_ignores_initial_state_and_rearms_on_external_change() {
        let mut p = Pickup::default();
        assert_eq!(p.update(0.9, 0.5), None);
        assert_eq!(p.update(0.8, 0.5), None);
        assert_eq!(p.update(0.4, 0.5), Some(0.4));
        assert_eq!(p.update(0.3, 0.4), Some(0.3));
        assert_eq!(p.update(0.2, 0.7), None);
        assert_eq!(p.update(0.8, 0.7), Some(0.8));
    }
    #[test]
    fn press_requires_release_and_deduplicates() {
        let mut p = PressEdge::default();
        assert!(!p.update(true));
        assert!(!p.update(false));
        assert!(p.update(true));
        assert!(!p.update(true));
        assert!(!p.update(false));
        assert!(p.update(true));
    }
}
