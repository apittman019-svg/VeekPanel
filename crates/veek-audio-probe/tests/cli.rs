use std::process::Command;
fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_veek-audio-probe"))
        .args(args)
        .output()
        .unwrap()
}
#[test]
fn invalid_commands_fail_before_native_access() {
    for args in [
        vec!["set", "--target", "default-output"],
        vec!["set", "--target", "default-output", "--volume", "NaN"],
        vec!["set", "--target", "default-output", "--volume", "inf"],
        vec!["set", "--target", "default-output", "--volume", "101"],
        vec![
            "set",
            "--target",
            "default-output",
            "--volume",
            "50",
            "--mute",
            "on",
        ],
        vec!["bind", "--target", "default-output"],
        vec![
            "bind",
            "--target",
            "default-output",
            "--serial",
            "FAKE",
            "--hid-path",
            "FAKE",
        ],
        vec![
            "bind",
            "--target",
            "default-output",
            "--serial",
            "FAKE",
            "--knob",
            "0",
        ],
        vec!["watch", "--duration", "0"],
    ] {
        let output = run(&args);
        assert_eq!(
            output.status.code(),
            Some(2),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
#[test]
fn help_is_explicit_about_scope_and_writes() {
    for command in [
        vec!["--help"],
        vec!["set", "--help"],
        vec!["bind", "--help"],
    ] {
        assert!(run(&command).status.success());
    }
    let output = run(&["--help"]);
    assert!(String::from_utf8_lossy(&output.stdout).contains("no saved mappings or GUI"));
}
