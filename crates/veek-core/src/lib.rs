//! Platform-independent identity resolution and input-to-action planning.
#![forbid(unsafe_code)]
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use veek_audio::{Change, Kind, Pickup, PressEdge, Selection, Snapshot, Target};
use veek_config::{Action, Config, Control, ControlKind, Selector};
use veek_hardware::protocol::ControlEvent;

mod feedback;
pub use feedback::{ControlFeedback, FeedbackPhase};

const ID_KEYS: &[&str] = &[
    "application.id",
    "application.path",
    "endpoint.id",
    "node.name",
    "application.process.binary",
];

/// Select the strongest available stable hint. Names/PIDs/live IDs are never saved.
pub fn selector_for(target: &Target) -> Result<Selector, String> {
    let device = matches!(target.kind, Kind::Output | Kind::Input);
    let key = ID_KEYS
        .iter()
        .find(|key| {
            device == matches!(**key, "endpoint.id" | "node.name")
                && target.identity.get(**key).is_some_and(|v| !v.is_empty())
        })
        .ok_or(
            "This target has no durable identity; use a default device or a different application",
        )?;
    Ok(Selector::Match {
        kind: target.kind,
        identities: BTreeMap::from([((*key).into(), target.identity[*key].clone())]),
    })
}

pub struct Resolved<'a> {
    pub targets: Vec<&'a Target>,
    pub relative: bool,
    pub warnings: Vec<String>,
}
/// Exact identity alternatives support dual-boot profiles. If a stronger key is
/// present but different, a weaker binary-name fallback must not override it.
pub fn resolve<'a>(
    config: &Config,
    selector: &Selector,
    snapshot: &'a Snapshot,
) -> Result<Resolved<'a>, String> {
    let profile = config.active().ok_or("Active profile no longer exists")?;
    let (targets, relative, warnings) = match selector {
        Selector::DefaultOutput | Selector::DefaultInput => {
            let kind = if matches!(selector, Selector::DefaultOutput) {
                Kind::Output
            } else {
                Kind::Input
            };
            (
                snapshot
                    .targets
                    .iter()
                    .filter(|t| t.kind == kind && t.default)
                    .collect::<Vec<_>>(),
                false,
                vec![],
            )
        }
        Selector::PreferredOutput => {
            return resolve(
                config,
                profile
                    .preferences
                    .output
                    .as_ref()
                    .unwrap_or(&Selector::DefaultOutput),
                snapshot,
            )
        }
        Selector::PreferredInput => {
            return resolve(
                config,
                profile
                    .preferences
                    .input
                    .as_ref()
                    .unwrap_or(&Selector::DefaultInput),
                snapshot,
            )
        }
        Selector::Match { kind, identities } => {
            let found = snapshot
                .targets
                .iter()
                .filter(|t| {
                    t.kind == *kind
                        && ID_KEYS.iter().find_map(|key| {
                            identities
                                .get(*key)
                                .zip(t.identity.get(*key))
                                .map(|(a, b)| a == b)
                        }) == Some(true)
                })
                .collect::<Vec<_>>();
            (found, false, vec![])
        }
        Selector::Group { id } => {
            let group = profile
                .groups
                .iter()
                .find(|g| &g.id == id)
                .ok_or("Audio group no longer exists")?;
            let mut unique = BTreeMap::new();
            let mut warnings = vec![];
            for (i, member) in group.members.iter().enumerate() {
                if matches!(member, Selector::Group { .. }) {
                    return Err("Nested groups are not supported".into());
                }
                match resolve(config, member, snapshot) {
                    Ok(found) if !found.targets.is_empty() => {
                        for t in found.targets {
                            unique.insert(&t.id, t);
                        }
                    }
                    Ok(_) => warnings.push(format!("Group member {} is unavailable", i + 1)),
                    Err(error) => return Err(format!("Group member {}: {error}", i + 1)),
                }
            }
            (unique.into_values().collect(), group.relative, warnings)
        }
    };
    // Multiple sessions of one application are intentional. Device ambiguity is not.
    if !matches!(selector, Selector::Group { .. })
        && targets.len() > 1
        && targets
            .iter()
            .any(|t| matches!(t.kind, Kind::Input | Kind::Output))
    {
        return Err(
            "More than one device matches this identity; choose an unambiguous device".into(),
        );
    }
    Ok(Resolved {
        targets,
        relative,
        warnings,
    })
}
#[derive(Default, Debug, Serialize)]
pub struct Effects {
    pub audio: Vec<(Selection, Change)>,
    pub profile: Option<String>,
    pub diagnostics: Vec<String>,
}
#[derive(Default)]
struct InputState {
    signature: (u64, Vec<String>),
    pickup: Pickup,
    button: PressEdge,
    ratios: Vec<f32>,
    expected: BTreeMap<String, f32>,
    position: Option<f32>,
}
impl InputState {
    fn synchronize(&mut self, snapshot: &Snapshot, targets: &[&Target]) {
        let signature = (
            snapshot.generation,
            targets.iter().map(|t| t.id.clone()).collect(),
        );
        if self.signature != signature {
            *self = Self {
                signature,
                ..Default::default()
            };
        } else if targets.iter().any(|t| {
            self.expected.get(&t.id).is_some_and(|before| {
                t.volume
                    .is_none_or(|v| !v.is_finite() || (before - v).abs() > 0.02)
            })
        }) {
            self.pickup = Pickup::default();
            self.expected.clear();
        }
    }
}
#[derive(Default)]
pub struct Engine {
    inputs: BTreeMap<Control, InputState>,
}
impl Engine {
    /// Call after config/profile changes, hardware reconnect, and audio loss.
    pub fn reset(&mut self) {
        self.inputs.clear();
    }
    pub fn handle(&mut self, config: &Config, event: ControlEvent, snapshot: &Snapshot) -> Effects {
        let mut effect = Effects::default();
        let (kind, index) = match event {
            ControlEvent::Analog { index, .. } => (ControlKind::Analog, index),
            ControlEvent::Button { index, .. } => (ControlKind::Button, index),
        };
        let limit = if kind == ControlKind::Analog {
            config.hardware.model.analog_count()
        } else {
            config.hardware.model.button_count()
        };
        if index >= limit {
            effect
                .diagnostics
                .push("Input is outside configured panel capabilities".into());
            return effect;
        }
        let control = Control {
            device: "primary".into(),
            kind,
            index,
        };
        let Some(profile) = config
            .profiles
            .iter()
            .find(|p| p.id == config.active_profile)
        else {
            return effect;
        };
        let Some(mapping) = profile.mappings.iter().find(|m| m.control == control) else {
            return effect;
        };
        let target = match &mapping.action {
            Action::Volume { target }
            | Action::ToggleMute { target }
            | Action::SetMute { target, .. } => Some(target),
            _ => None,
        };
        let resolved = if let Some(selector) = target {
            match resolve(config, selector, snapshot) {
                Ok(r) => r,
                Err(e) => {
                    self.inputs.remove(&control);
                    effect.diagnostics.push(e);
                    return effect;
                }
            }
        } else {
            Resolved {
                targets: vec![],
                relative: false,
                warnings: vec![],
            }
        };
        effect.diagnostics.extend(resolved.warnings);
        let state = self.inputs.entry(control).or_default();
        state.synchronize(snapshot, &resolved.targets);
        match (&mapping.action, event) {
            (Action::Volume { .. }, ControlEvent::Analog { raw, maximum, .. }) => {
                if maximum == 0 || raw > maximum {
                    effect.diagnostics.push("Invalid analog range".into());
                    return effect;
                }
                state.position = Some(f32::from(raw) / f32::from(maximum));
                if resolved.targets.is_empty() {
                    effect
                        .diagnostics
                        .push("Mapped audio target is unavailable".into());
                    return effect;
                }
                let volumes: Option<Vec<_>> = resolved
                    .targets
                    .iter()
                    .map(|t| t.volume.filter(|v| v.is_finite() && *v >= 0.))
                    .collect();
                let Some(volumes) = volumes else {
                    effect
                        .diagnostics
                        .push("A mapped target has no usable volume control".into());
                    return effect;
                };
                let peak = volumes.iter().copied().fold(0., f32::max);
                if resolved.relative && peak > 0. {
                    state.ratios = volumes.iter().map(|v| v / peak).collect();
                }
                let Some(value) = state
                    .pickup
                    .update(f32::from(raw) / f32::from(maximum), peak)
                else {
                    return effect;
                };
                for (i, t) in resolved.targets.iter().enumerate() {
                    let volume = if resolved.relative {
                        value * state.ratios.get(i).copied().unwrap_or(1.)
                    } else {
                        value
                    };
                    state.expected.insert(t.id.clone(), volume);
                    effect.audio.push((
                        Selection {
                            generation: snapshot.generation,
                            id: t.id.clone(),
                        },
                        Change::Volume(volume),
                    ));
                }
            }
            (action, ControlEvent::Button { pressed, .. }) => {
                if !state.button.update(pressed) {
                    return effect;
                }
                match action {
                    Action::ToggleMute { .. } | Action::SetMute { .. } => {
                        if resolved.targets.is_empty() {
                            effect
                                .diagnostics
                                .push("Mapped audio target is unavailable".into());
                            return effect;
                        }
                        if resolved.targets.iter().any(|t| t.muted.is_none()) {
                            effect
                                .diagnostics
                                .push("A mapped target has no mute control".into());
                            return effect;
                        }
                        let mute = if let Action::SetMute { muted, .. } = action {
                            *muted
                        } else {
                            !resolved.targets.iter().all(|t| t.muted == Some(true))
                        };
                        effect.audio = resolved
                            .targets
                            .iter()
                            .map(|t| {
                                (
                                    Selection {
                                        generation: snapshot.generation,
                                        id: t.id.clone(),
                                    },
                                    Change::Mute(mute),
                                )
                            })
                            .collect();
                    }
                    Action::SwitchProfile { id } => effect.profile = Some(id.clone()),
                    Action::NextProfile => {
                        let i = config
                            .profiles
                            .iter()
                            .position(|p| p.id == config.active_profile)
                            .unwrap_or(0);
                        effect.profile = config
                            .profiles
                            .get((i + 1) % config.profiles.len())
                            .map(|p| p.id.clone());
                    }
                    _ => (),
                }
            }
            _ => effect
                .diagnostics
                .push("Action does not support this input type".into()),
        }
        effect
    }
}
#[derive(Serialize)]
pub struct MappingStatus {
    pub control: Control,
    pub targets: Vec<String>,
    pub messages: Vec<String>,
}
pub fn statuses(config: &Config, snapshot: &Snapshot) -> Vec<MappingStatus> {
    config
        .profiles
        .iter()
        .find(|p| p.id == config.active_profile)
        .into_iter()
        .flat_map(|p| &p.mappings)
        .map(|mapping| {
            let mut status = MappingStatus {
                control: mapping.control.clone(),
                targets: vec![],
                messages: vec![],
            };
            if let Action::Volume { target }
            | Action::ToggleMute { target }
            | Action::SetMute { target, .. } = &mapping.action
            {
                match resolve(config, target, snapshot) {
                    Ok(r) => {
                        status.targets = r.targets.iter().map(|t| t.id.clone()).collect();
                        status.messages = r.warnings;
                        if status.targets.is_empty() {
                            status
                                .messages
                                .push("Unavailable; waiting for matching audio".into());
                        }
                    }
                    Err(e) => status.messages.push(e),
                }
            }
            status
        })
        .collect()
}
/// Coalesce only adjacent analog runs. A button edge is an ordering barrier.
pub fn coalesce(events: Vec<ControlEvent>) -> Vec<ControlEvent> {
    let mut out = vec![];
    let mut run = vec![];
    let flush = |run: &mut Vec<ControlEvent>, out: &mut Vec<ControlEvent>| {
        let mut seen = BTreeSet::new();
        let mut keep = vec![];
        for event in run.drain(..).rev() {
            if let ControlEvent::Analog { index, .. } = event {
                if seen.insert(index) {
                    keep.push(event);
                }
            }
        }
        out.extend(keep.into_iter().rev());
    };
    for e in events {
        if matches!(e, ControlEvent::Button { .. }) {
            flush(&mut run, &mut out);
            out.push(e);
        } else {
            run.push(e);
        }
    }
    flush(&mut run, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use veek_config::{Group, Mapping, Profile};
    fn target(id: &str, volume: f32) -> Target {
        Target {
            id: id.into(),
            name: id.into(),
            kind: Kind::Playback,
            default: false,
            volume: Some(volume),
            muted: Some(false),
            identity: BTreeMap::from([("application.id".into(), id.into())]),
        }
    }
    fn snap() -> Snapshot {
        Snapshot {
            backend: "explicit test mock".into(),
            generation: 1,
            targets: vec![target("a", 0.4), target("b", 0.2)],
        }
    }
    fn mapping(c: &mut Config, kind: ControlKind, index: u8, action: Action) {
        c.profiles[0].mappings.push(Mapping {
            control: Control {
                device: "primary".into(),
                kind,
                index,
            },
            action,
        });
    }
    fn analog(raw: u8) -> ControlEvent {
        ControlEvent::Analog {
            index: 0,
            raw,
            maximum: 100,
        }
    }
    fn button(pressed: bool) -> ControlEvent {
        ControlEvent::Button { index: 0, pressed }
    }
    fn apply(snapshot: &mut Snapshot, e: Effects) {
        for (s, change) in e.audio {
            let t = snapshot.targets.iter_mut().find(|t| t.id == s.id).unwrap();
            match change {
                Change::Volume(v) => t.volume = Some(v),
                Change::Mute(m) => t.muted = Some(m),
            }
        }
    }
    #[test]
    fn relaunch_matches_metadata_not_live_id_and_stronger_identity_wins() {
        let c = Config::default();
        let mut s = snap();
        let selector = selector_for(&s.targets[0]).unwrap();
        s.targets[0].id = "replacement-live-id".into();
        assert_eq!(
            resolve(&c, &selector, &s).unwrap().targets[0].id,
            "replacement-live-id"
        );
        let mixed = Selector::Match {
            kind: Kind::Playback,
            identities: BTreeMap::from([
                ("application.id".into(), "wrong".into()),
                ("application.process.binary".into(), "same".into()),
            ]),
        };
        s.targets[0]
            .identity
            .insert("application.process.binary".into(), "same".into());
        assert!(resolve(&c, &mixed, &s).unwrap().targets.is_empty());
        assert!(selector_for(&Target {
            identity: BTreeMap::from([("process.id".into(), "123".into())]),
            ..target("no-id", 0.2)
        })
        .is_err());
    }
    #[test]
    fn relative_group_preserves_balance_including_zero_and_skips_absent_member() {
        let mut c = Config::default();
        let mut s = snap();
        let selectors = s
            .targets
            .iter()
            .map(|t| selector_for(t).unwrap())
            .chain([selector_for(&target("absent", 0.2)).unwrap()])
            .collect();
        c.profiles[0].groups.push(Group {
            id: "mix".into(),
            name: "Mix".into(),
            members: selectors,
            relative: true,
        });
        mapping(
            &mut c,
            ControlKind::Analog,
            0,
            Action::Volume {
                target: Selector::Group { id: "mix".into() },
            },
        );
        let mut e = Engine::default();
        assert!(e.handle(&c, analog(10), &s).audio.is_empty());
        let effect = e.handle(&c, analog(60), &s);
        assert_eq!(effect.diagnostics.len(), 1);
        apply(&mut s, effect);
        assert_eq!(s.targets[0].volume, Some(0.6));
        assert_eq!(s.targets[1].volume, Some(0.3));
        let effect = e.handle(&c, analog(0), &s);
        apply(&mut s, effect);
        let effect = e.handle(&c, analog(40), &s);
        apply(&mut s, effect);
        assert_eq!(s.targets[1].volume, Some(0.2));
    }
    #[test]
    fn default_change_generation_and_external_volume_rearm_pickup() {
        let mut c = Config::default();
        let mut s = snap();
        s.targets[0].kind = Kind::Output;
        s.targets[0].default = true;
        mapping(
            &mut c,
            ControlKind::Analog,
            0,
            Action::Volume {
                target: Selector::DefaultOutput,
            },
        );
        let mut e = Engine::default();
        assert!(e.handle(&c, analog(10), &s).audio.is_empty());
        let effects = e.handle(&c, analog(60), &s);
        apply(&mut s, effects);
        s.generation += 1;
        assert!(e.handle(&c, analog(70), &s).audio.is_empty());
        e.reset();
        assert!(e.handle(&c, analog(20), &s).audio.is_empty());
        let effect = e.handle(&c, analog(80), &s);
        apply(&mut s, effect);
        s.targets[0].volume = Some(0.2);
        assert!(e.handle(&c, analog(75), &s).audio.is_empty());
        s.targets[0].default = false;
        s.targets[1].kind = Kind::Output;
        s.targets[1].default = true;
        assert!(e.handle(&c, analog(20), &s).audio.is_empty());
    }
    #[test]
    fn buttons_require_release_and_profiles_do_not_replay_held_presses() {
        let mut c = Config::default();
        let s = snap();
        mapping(
            &mut c,
            ControlKind::Button,
            0,
            Action::ToggleMute {
                target: selector_for(&s.targets[0]).unwrap(),
            },
        );
        let mut e = Engine::default();
        assert!(e.handle(&c, button(true), &s).audio.is_empty());
        e.handle(&c, button(false), &s);
        assert_eq!(
            e.handle(&c, button(true), &s).audio[0].1,
            Change::Mute(true)
        );
        assert!(e.handle(&c, button(true), &s).audio.is_empty());
        c.profiles.push(Profile {
            id: "next".into(),
            name: "Next".into(),
            mappings: vec![],
            groups: vec![],
            preferences: Default::default(),
        });
        c.profiles[0].mappings[0].action = Action::NextProfile;
        e.reset();
        assert!(e.handle(&c, button(true), &s).profile.is_none());
        e.handle(&c, button(false), &s);
        assert_eq!(
            e.handle(&c, button(true), &s).profile.as_deref(),
            Some("next")
        );
    }
    #[test]
    fn ambiguous_devices_fail_closed_and_unavailable_preferences_never_fall_back() {
        let mut c = Config::default();
        let mut s = snap();
        for t in &mut s.targets {
            t.kind = Kind::Output;
            t.default = true;
            t.identity = BTreeMap::from([("node.name".into(), "duplicate".into())]);
        }
        assert!(resolve(&c, &Selector::DefaultOutput, &s).is_err());
        assert!(resolve(&c, &selector_for(&s.targets[0]).unwrap(), &s).is_err());
        c.profiles[0].preferences.output = Some(Selector::Match {
            kind: Kind::Output,
            identities: BTreeMap::from([("node.name".into(), "missing".into())]),
        });
        assert!(resolve(&c, &Selector::PreferredOutput, &s)
            .unwrap()
            .targets
            .is_empty());
    }
    #[test]
    fn profile_group_and_preference_resolution_stay_independent() {
        let mut c = Config::default();
        let mut s = snap();
        for t in &mut s.targets {
            t.kind = Kind::Output;
            t.identity = BTreeMap::from([("node.name".into(), t.id.clone())]);
        }
        c.profiles[0].preferences.output = Some(selector_for(&s.targets[0]).unwrap());
        c.profiles[0].groups.push(Group {
            id: "mix".into(),
            name: "First mix".into(),
            members: vec![Selector::PreferredOutput],
            relative: true,
        });
        let mut other = c.profiles[0].clone();
        other.id = "second".into();
        other.preferences.output = Some(selector_for(&s.targets[1]).unwrap());
        other.groups[0].relative = false;
        c.profiles.push(other);
        c.validate().unwrap();
        let selector = Selector::Group { id: "mix".into() };
        let first = resolve(&c, &selector, &s).unwrap();
        assert_eq!(first.targets[0].id, "a");
        assert!(first.relative);
        c.active_profile = "second".into();
        let second = resolve(&c, &selector, &s).unwrap();
        assert_eq!(second.targets[0].id, "b");
        assert!(!second.relative);
        s.targets.pop();
        assert!(resolve(&c, &selector, &s).unwrap().targets.is_empty());
        c.active_profile = "default".into();
        assert_eq!(resolve(&c, &selector, &s).unwrap().targets[0].id, "a");
    }
    #[test]
    fn feedback_tracks_pickup_and_rearms_on_observed_changes_without_writing() {
        let mut c = Config::default();
        let mut s = snap();
        mapping(
            &mut c,
            ControlKind::Analog,
            0,
            Action::Volume {
                target: selector_for(&s.targets[0]).unwrap(),
            },
        );
        let mut e = Engine::default();
        assert_eq!(
            e.feedback(&c, Some(&s))[0].phase,
            FeedbackPhase::AwaitingInput
        );
        assert!(e.handle(&c, analog(10), &s).audio.is_empty());
        let f = e.feedback(&c, Some(&s)).remove(0);
        assert_eq!(f.phase, FeedbackPhase::Pickup);
        assert_eq!(f.position, Some(0.1));
        assert_eq!(f.target_volume, Some(0.4));
        let effects = e.handle(&c, analog(60), &s);
        apply(&mut s, effects);
        assert_eq!(
            e.feedback(&c, Some(&s))[0].phase,
            FeedbackPhase::Controlling
        );
        s.targets[0].volume = Some(0.2);
        assert_eq!(e.feedback(&c, Some(&s))[0].phase, FeedbackPhase::Pickup);
        // Publishing feedback must not synthesize movement or acquire control.
        assert!(e.handle(&c, analog(55), &s).audio.is_empty());
        assert!(e.handle(&c, analog(20), &s).audio.len() == 1);
        s.targets[0].id = "new-session".into();
        assert_eq!(
            e.feedback(&c, Some(&s))[0].phase,
            FeedbackPhase::AwaitingInput
        );
        s.generation += 1;
        assert!(e.handle(&c, analog(70), &s).audio.is_empty());
        e.reset();
        assert_eq!(e.feedback(&c, Some(&s))[0].position, None);
    }
    #[test]
    fn feedback_reports_offline_missing_ambiguous_and_unsupported_targets() {
        let mut c = Config::default();
        let mut s = snap();
        mapping(
            &mut c,
            ControlKind::Analog,
            0,
            Action::Volume {
                target: selector_for(&s.targets[0]).unwrap(),
            },
        );
        let mut e = Engine::default();
        assert_eq!(e.feedback(&c, None)[0].phase, FeedbackPhase::AudioOffline);
        s.targets[0].volume = None;
        assert_eq!(e.feedback(&c, Some(&s))[0].phase, FeedbackPhase::Blocked);
        s.targets[0].volume = Some(1.5);
        assert_eq!(e.feedback(&c, Some(&s))[0].phase, FeedbackPhase::Blocked);
        s.targets.clear();
        assert_eq!(
            e.feedback(&c, Some(&s))[0].phase,
            FeedbackPhase::TargetUnavailable
        );
        s = snap();
        assert_eq!(
            e.feedback(&c, Some(&s))[0].phase,
            FeedbackPhase::AwaitingInput
        );
        c.profiles[0].mappings[0].action = Action::Volume {
            target: Selector::DefaultOutput,
        };
        for t in &mut s.targets {
            t.kind = Kind::Output;
            t.default = true;
        }
        let f = e.feedback(&c, Some(&s)).remove(0);
        assert_eq!(f.phase, FeedbackPhase::Blocked);
        assert!(f.message.unwrap().contains("More than one device"));
    }
    #[test]
    fn feedback_preserves_group_peak_and_independent_button_readiness() {
        let mut c = Config::default();
        let mut s = snap();
        c.profiles[0].groups.push(Group {
            id: "mix".into(),
            name: "Mix".into(),
            relative: true,
            members: s.targets.iter().map(|t| selector_for(t).unwrap()).collect(),
        });
        mapping(
            &mut c,
            ControlKind::Analog,
            0,
            Action::Volume {
                target: Selector::Group { id: "mix".into() },
            },
        );
        mapping(
            &mut c,
            ControlKind::Button,
            0,
            Action::ToggleMute {
                target: selector_for(&s.targets[1]).unwrap(),
            },
        );
        let mut e = Engine::default();
        assert_eq!(e.feedback(&c, Some(&s))[0].target_volume, Some(0.4));
        assert_eq!(
            e.feedback(&c, Some(&s))[1].phase,
            FeedbackPhase::WaitingRelease
        );
        e.handle(&c, button(false), &s);
        assert_eq!(e.feedback(&c, Some(&s))[1].phase, FeedbackPhase::Ready);
        e.handle(&c, analog(0), &s);
        assert_eq!(e.feedback(&c, Some(&s))[1].phase, FeedbackPhase::Ready);
        let effects = e.handle(&c, button(true), &s);
        apply(&mut s, effects);
        assert_eq!(
            e.feedback(&c, Some(&s))[1].phase,
            FeedbackPhase::WaitingRelease
        );
        assert_eq!(s.targets[1].muted, Some(true));
        e.handle(&c, button(false), &s);
        assert_eq!(e.feedback(&c, Some(&s))[1].phase, FeedbackPhase::Ready);
        c.profiles[0].mappings[1].action = Action::NextProfile;
        e.reset();
        e.handle(&c, button(false), &s);
        assert_eq!(e.feedback(&c, None)[1].phase, FeedbackPhase::Ready);
    }
    #[test]
    fn coalescing_preserves_button_barriers_and_last_analog_order() {
        assert_eq!(
            coalesce(vec![
                analog(1),
                analog(2),
                button(false),
                analog(3),
                button(true),
                button(false)
            ]),
            vec![
                analog(2),
                button(false),
                analog(3),
                button(true),
                button(false)
            ]
        );
    }
}
