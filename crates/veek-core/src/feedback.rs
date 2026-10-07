//! Live input readiness belongs to the engine, never to UI guesses about knob positions.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackPhase {
    AwaitingInput,
    Pickup,
    Controlling,
    WaitingRelease,
    Ready,
    AudioOffline,
    TargetUnavailable,
    Blocked,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ControlFeedback {
    pub control: Control,
    pub phase: FeedbackPhase,
    pub position: Option<f32>,
    /// Peak volume is the pickup level for both relative and equal groups.
    pub target_volume: Option<f32>,
    pub message: Option<String>,
}

impl Engine {
    /// Reconcile observed audio before publishing readiness. This only rearms state;
    /// it never invents a hardware event, acquires pickup or requests an audio write.
    pub fn feedback(
        &mut self,
        config: &Config,
        snapshot: Option<&Snapshot>,
    ) -> Vec<ControlFeedback> {
        config
            .active()
            .into_iter()
            .flat_map(|p| &p.mappings)
            .map(|mapping| {
                let control = &mapping.control;
                let mut feedback = ControlFeedback {
                    control: control.clone(),
                    phase: FeedbackPhase::AwaitingInput,
                    position: None,
                    target_volume: None,
                    message: None,
                };
                let selector = match &mapping.action {
                    Action::Volume { target }
                    | Action::ToggleMute { target }
                    | Action::SetMute { target, .. } => Some(target),
                    _ => None,
                };
                let targets = if let Some(selector) = selector {
                    let Some(snapshot) = snapshot else {
                        self.inputs.remove(control);
                        feedback.phase = FeedbackPhase::AudioOffline;
                        return feedback;
                    };
                    match resolve(config, selector, snapshot) {
                        Ok(r) if !r.targets.is_empty() => r.targets,
                        result => {
                            self.inputs.remove(control);
                            feedback.phase = match result {
                                Err(message) => {
                                    feedback.message = Some(message);
                                    FeedbackPhase::Blocked
                                }
                                _ => FeedbackPhase::TargetUnavailable,
                            };
                            return feedback;
                        }
                    }
                } else {
                    vec![]
                };
                let state = self.inputs.entry(control.clone()).or_default();
                // Profile actions must remain usable through an audio outage.
                if selector.is_some() {
                    state.synchronize(snapshot.expect("resolved audio has a snapshot"), &targets);
                }
                if matches!(mapping.action, Action::Volume { .. }) {
                    let volumes: Option<Vec<_>> = targets
                        .iter()
                        .map(|t| t.volume.filter(|v| v.is_finite() && *v >= 0.))
                        .collect();
                    feedback.position = state.position;
                    match volumes {
                        Some(volumes) => {
                            let peak = volumes.into_iter().fold(0., f32::max);
                            feedback.target_volume = Some(peak);
                            if peak > 1. {
                                state.pickup = Pickup::default();
                                state.expected.clear();
                                feedback.phase = FeedbackPhase::Blocked;
                                feedback.message = Some(
                                    "Lower target volume to 100% or less to pick up control".into(),
                                );
                            } else {
                                feedback.phase = if state.pickup.is_active() {
                                    FeedbackPhase::Controlling
                                } else if state.position.is_some() {
                                    FeedbackPhase::Pickup
                                } else {
                                    FeedbackPhase::AwaitingInput
                                };
                            }
                        }
                        None => {
                            state.pickup = Pickup::default();
                            state.expected.clear();
                            feedback.phase = FeedbackPhase::Blocked;
                            feedback.message =
                                Some("A mapped target has no usable volume control".into());
                        }
                    }
                } else if selector.is_some() && targets.iter().any(|t| t.muted.is_none()) {
                    state.button = PressEdge::default();
                    feedback.phase = FeedbackPhase::Blocked;
                    feedback.message = Some("A mapped target has no mute control".into());
                } else {
                    feedback.phase = if state.button.is_ready() {
                        FeedbackPhase::Ready
                    } else {
                        FeedbackPhase::WaitingRelease
                    };
                }
                feedback
            })
            .collect()
    }
}
