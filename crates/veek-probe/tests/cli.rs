use std::{path::PathBuf, process::Command};
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(name)
}

#[test]
fn original_replay_is_labeled_offline_and_preserves_push_buttons() {
    let out = Command::new(env!("CARGO_BIN_EXE_veek-probe"))
        .args(["replay", "--model", "original"])
        .arg(fixture("original.txt"))
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    for expected in [
        "OFFLINE REPLAY",
        "KNOB_1 = 73",
        "KNOB_2 = 42",
        "KNOB_3_PRESS = TRUE",
        "KNOB_3_PRESS = FALSE",
        "HEARTBEAT",
    ] {
        assert!(stdout.contains(expected), "{stdout}");
    }
    assert!(!stdout.contains("CONNECTED"));
}

#[test]
fn pro_replay_maps_sliders_separately() {
    let out = Command::new(env!("CARGO_BIN_EXE_veek-probe"))
        .args(["replay", "--model", "pro"])
        .arg(fixture("pro.hex"))
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("SLIDER_1 = 100"));
    assert!(stdout.contains("SLIDER_4 = 0"));
}

#[test]
fn invalid_arguments_and_missing_files_fail() {
    for args in [
        vec!["watch", "--duration", "0"],
        vec!["replay", "--model", "unknown", "x"],
        vec!["replay", "--model", "original", "no-such-file"],
    ] {
        assert!(!Command::new(env!("CARGO_BIN_EXE_veek-probe"))
            .args(args)
            .output()
            .unwrap()
            .status
            .success());
    }
}
