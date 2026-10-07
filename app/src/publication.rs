//! Skip unchanged runtime payloads while always reporting machine-local status.
pub fn payload(
    published: &veek_runtime::PublishedState,
    known_version: Option<u64>,
    desktop: serde_json::Value,
) -> serde_json::Value {
    let changed = known_version != Some(published.version);
    let state = changed.then_some(published.state.as_ref());
    let mappings = state.map(|state| {
        state
            .audio
            .as_ref()
            .map(|audio| veek_core::statuses(&state.config, audio))
            .unwrap_or_default()
    });
    serde_json::json!({"version":published.version,"state":state,"mappings":mappings,"desktop":desktop})
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::BTreeMap, sync::Arc};

    #[test]
    fn unchanged_runtime_still_reports_desktop_changes_and_unknown_versions_resync() {
        let published = veek_runtime::PublishedState {
            version: 7,
            state: Arc::new(veek_runtime::State {
                config: veek_config::Config::default(),
                revision: 1,
                audio: None,
                audio_status: "Unavailable".into(),
                hardware_status: "Disabled".into(),
                controls: BTreeMap::new(),
                feedback: vec![],
                diagnostics: vec![],
            }),
        };
        let full = payload(&published, None, serde_json::json!({"tray_ready":true}));
        assert!(full["state"].is_object());
        assert!(full["mappings"].is_array());
        let unchanged = payload(&published, Some(7), serde_json::json!({"tray_ready":false}));
        assert!(unchanged["state"].is_null());
        assert!(unchanged["mappings"].is_null());
        assert_eq!(unchanged["desktop"]["tray_ready"], false);
        assert_eq!(unchanged["version"], 7);
        assert_eq!(
            payload(&published, Some(6), serde_json::Value::Null)["state"],
            full["state"]
        );
        assert_eq!(
            payload(&published, Some(8), serde_json::Value::Null)["state"],
            full["state"]
        );
        assert!(
            serde_json::to_vec(&unchanged).unwrap().len()
                < serde_json::to_vec(&full).unwrap().len() / 2
        );
    }
}
