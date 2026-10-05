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
        vec!["test-mini", "--duration", "0"],
        vec!["test-mini", "--wait", "0"],
        vec![
            "capture",
            "--serial",
            "COM3",
            "--output",
            "unused",
            "--max-bytes",
            "0",
        ],
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

#[test]
fn guided_test_stops_on_closed_input_before_opening_hardware() {
    let output = Command::new(env!("CARGO_BIN_EXE_veek-probe"))
        .arg("test-original")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("Input closed"));
    assert!(!String::from_utf8(output.stdout)
        .unwrap()
        .contains("CAPTURE_OPENED"));
}

#[test]
fn mini_missing_exact_path_saves_clear_empty_evidence_without_opening_hardware() {
    let directory = std::env::temp_dir().join(format!(
        "veek-mini-cli-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let output = Command::new(env!("CARGO_BIN_EXE_veek-probe"))
        .args([
            "test-mini",
            "--wait",
            "1",
            "--duration",
            "1",
            "--hid-path",
            "veek-test-nonexistent-hid-path",
            "--output",
        ])
        .arg(&directory)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let summary = std::fs::read_to_string(directory.join("SUMMARY.txt")).unwrap();
    assert!(summary.contains("NO INPUT RECEIVED"));
    let metadata = std::fs::read_to_string(directory.join("metadata.txt")).unwrap();
    assert!(metadata.contains("hardware_validation=unverified"));
    assert!(metadata.contains("status=finished"));
    assert!(!metadata.contains("veek-test-nonexistent-hid-path"));
    assert!(directory.join("RESULTS.txt").exists());
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn mini_replay_covers_four_knobs_and_independent_button_edges() {
    let output = Command::new(env!("CARGO_BIN_EXE_veek-probe"))
        .args(["replay", "--model", "mini"])
        .arg(fixture("mini.hex"))
        .output()
        .unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("OFFLINE REPLAY"));
    for i in 1..=4 {
        assert!(text.contains(&format!("KNOB_{i} = 0 (raw=0/255)")));
        assert!(text.contains(&format!("KNOB_{i} = 100 (raw=255/255)")));
        assert!(text.contains(&format!("KNOB_{i}_PRESS = TRUE")));
        assert!(text.contains(&format!("KNOB_{i}_PRESS = FALSE")));
    }
}

#[test]
fn physical_mini_windows_excerpt_retains_full_frames_and_all_control_types() {
    let path = fixture("mini-windows-2026-10-04.hex");
    let input = std::fs::read_to_string(&path).unwrap();
    let reports: Vec<_> = input
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
    assert_eq!(reports.len(), 16);
    assert!(reports
        .iter()
        .all(|line| line.split_whitespace().count() == 64));
    let output = Command::new(env!("CARGO_BIN_EXE_veek-probe"))
        .args(["replay", "--model", "mini"])
        .arg(path)
        .output()
        .unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("OFFLINE REPLAY"));
    assert_eq!(text.lines().count(), 17);
    for knob in 1..=4 {
        for event in [
            format!("KNOB_{knob} = 0 (raw=0/255)"),
            format!("KNOB_{knob} = 100 (raw=255/255)"),
            format!("KNOB_{knob}_PRESS = TRUE"),
            format!("KNOB_{knob}_PRESS = FALSE"),
        ] {
            assert!(
                text.contains(&event),
                "missing physical fixture event: {event}"
            );
        }
    }
}
