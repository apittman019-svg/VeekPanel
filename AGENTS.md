# VeekPanel — instructions for continuing agents

## Read first and preserve scope

This repository implements a modern replacement controller for existing PCPanel
hardware. The complete original brief is in `docs/PROJECT_REQUIREMENTS.md`.
Read this file, `docs/VERIFICATION.md`, `docs/HARDWARE_PROTOCOL.md` and
`docs/MILESTONES.md` before continuing. Update them when evidence or scope changes.

**Current authorization: continue all development that does not require physical
PCPanel hardware, including native audio, M3 mappings/configuration, background
architecture and the actual backend-connected M4 desktop GUI.** The superseding
request is preserved in `docs/CONTINUATION_REQUEST.md`. Do not use pending hardware
validation as a reason to stop software work. Nobara is the current development
machine; Windows remains a first-class product target. M1 physical validation is
partially validated: Mini basic discovery/controls passed on the friend's Windows
11 machine; lifecycle and Nobara physical checks remain pending. Never generalize
that result to untested models, revisions or platforms.

User clarifications that override ambiguous wording in the original brief:

- Corrected by the user on 2026-10-04: the friend's device is a **PCPanel Mini 1.0**,
  not the Original/Maple previously reported. It has four analog knobs and four
  independent push buttons. Use the Mini HID adapter (observed VID:PID `0483:a3c4`),
  not the Original serial guide. The returned Windows 11 capture confirms all four
  raw 0–255 ranges and button edges; the worksheet confirms left-to-right numbering
  and clockwise increase. Printed model/revision and firmware were marked unknown.
  See docs/HARDWARE_VALIDATION.md for the precise accepted checks and remaining gaps.
- This is mainly a **Windows 10/11 application**. It must also work on **Nobara**;
  Nobara is the first Linux validation target. Broader Linux compatibility remains
  planned, with PipeWire as the native direction.
- The user authorized publishing a **public `VeekPanel` GitHub repository** under
  their authenticated account. No credentials or private diagnostics go into Git.
- Real hardware is with the friend, who will download and test when convenient.
  Do not assume local access. Synthetic replay/PTY tests must be labeled as such.
  The experimental diagnostic kit is published as GitHub prerelease
  `m1-probe-2026-10-03` (source `f65350a4d1b25261410caddfc44504d1f64b44dd`).
  Mini-specific prerelease `m1-mini-2026-10-04` supersedes the Original launcher
  for this friend. Source: `e703b4cf70eaa03c5168b5f16a4ffc72d6249e8a`; Windows
  users run `TEST-MINI.cmd`. Both Windows/Ubuntu CI passed 35 tests for the kit.
  The returned real Windows 11 session has 4,473 reports, zero parse errors, all
  four full-range knobs and matching presses/releases. Reviewed 2026-10-04; an
  unchanged 16-report excerpt and provenance are in tests/fixtures. No adapter
  correction was needed. Reconnect/rerun was marked Yes without a specific result
  or second capture; sleep/resume was not tested. Next physical work is lifecycle
  and Nobara validation, not repeating already confirmed basic Windows inputs.

## Product requirements to preserve

1. Deliver a usable open-source daily-driver, native Windows/Linux experience:
   install → connect → detected → assign → use. No terminal for normal end users.
2. Communicate directly with original USB devices. Isolate HID/serial and model
   differences behind a hardware abstraction. Research before guessing packets.
3. Control system master, individual outputs, application sessions/streams, inputs
   and microphones; volume/mute and dynamic availability. Restore after relaunch,
   reboot, unplug/replug, sleep/resume, audio-service and device changes.
4. Map each analog input and each press independently. Support multiple targets,
   relative-volume groups, absent group members, profiles and profile switching.
   Plan mute actions, default input/output switching, media keys, shortcuts,
   explicitly configured local commands and future foreground-based switching.
5. Stable application metadata, never PID or PipeWire node ID alone. Windows path/
   package/session metadata; Linux app ID/name/binary/media metadata. Resolve
   ambiguity visibly and expose matching diagnostics.
6. Separate hardware, engine, audio, configuration, profile logic and UI. Platform
   checks belong in adapters/composition, not throughout shared logic.
7. Native Windows Core Audio and native PipeWire integration; Nobara is first-class.
   Use event subscriptions when available. UI must not block on enumeration/I/O.
8. Polished accessible UI later: rounded/squircle cards, generous spacing, clean
   type, subtle shadows/translucency, icons, animation, dark/light themes and
   scaling. Real device dashboard, connect screen, visual/drag-drop assignments
   and synchronized hardware/OS feedback. No mock UI in M1.
9. Background control when window closes; tray with open, current/switch profile,
   mic mute, status, settings, quit. Optional login startup and start-minimized.
10. Settings: general startup/tray/theme/update/scaling; hardware info/reconnect/
    LEDs/calibration; audio defaults/detection/missing-app handling; profile CRUD/
    import/export; diagnostics, logs, config folder and reset.
11. Human-readable schema-versioned config; atomic saves, validated imports,
    backups/migrations without silent destruction. Preserve platform-neutral
    intent for dual-boot profiles.
12. Understandable user errors plus technical logs. Handle missing/unsupported
    hardware, permission failures, bad config, disappearing apps/devices,
    Bluetooth loss, service restarts and reconnect. Never fake success.
13. Low idle CPU/memory, fast start and immediate input response; measure rather
    than claim. Preserve button edges while coalescing high-rate analog updates.
14. Windows installer/portable; Linux Flatpak/AppImage priorities with RPM/DEB
    possible. Test USB permissions/sandbox constraints and desktop integration.
15. No admin/root for runtime. Elevate only installation steps that need it.
    No remote arbitrary execution. Imported commands require explicit opt-in.
16. Diagnostics: model, USB IDs, firmware only if actually known, status, raw
    controls, backend, apps/devices/inputs, identities and errors. Future copied
    reports remove unnecessary personal data. Current CLI output is not redacted.
17. Automated tests: hardware parsers now; configuration/migrations, matching,
    mappings, profiles and groups with mocks when implemented. Keep hardware
    integration separate from default tests. Never use fixtures as physical proof.

## Stack and code boundaries

- Rust workspace, pinned toolchain and lockfile; original VeekPanel code is MIT.
- `crates/veek-hardware`: pure protocol/model layer and mockable HID/serial session.
- `crates/veek-probe`: local CLI for list/watch/offline replay; scoped M1 orchestration.
  `capture` saves bounded raw serial evidence; `test-original` guides the friend
  through explicit port selection and a 60-second capture. Neither certifies hardware.
  `test-mini` auto-detects only Mini HID, gives timed prompts and saves bounded raw
  reports/events plus a summary and checklist. `TEST-MINI.cmd` launches it; the
  Original-only `START-HERE.cmd` must not be used for the friend's Mini 1.0.
  `tests/manual` contains the diagnostic-kit launchers, checklists and notice generator.
  Prebuilt diagnostic utilities are M1/M2 test deliverables, not M6 installers or
  releases of the finished desktop application. Keep them visibly experimental.
- `crates/veek-audio`: shared target/change contract, generation checks, readback,
  balance/pickup/button diagnostic logic and mock tests.
- `crates/veek-audio-windows`: native Core Audio on an MTA owner thread. FFI unsafe
  is confined here with ownership/lifetime comments; shared code forbids unsafe.
- `crates/veek-audio-linux`: native PipeWire with registry/node/metadata subscriptions.
- `crates/veek-audio-probe`: M2 list/watch/set and explicit temporary one-knob/button
  bind. No saved profiles/mappings. Read `docs/AUDIO_VALIDATION.md` before changing it.
  Watch reconnects read-only; bind stops on loss and requires explicit rearming.
  Published prerelease `m2-audio-2026-10-04` uses source
  `237421c12bd453adc4994af0bb94e6c734c76098`. Windows/Ubuntu CI passed 30 tests;
  packaged Linux native integration passed on Nobara. Windows CI only enumerated
  an empty endpoint list: interactive Windows writes and all physical PCPanel
  checks remain pending. Do not equate the release with milestone acceptance.
- Future core/config crates and Tauri 2 + Svelte UI remain unimplemented.
- HID IDs: RGB `04d8:eb52`, Mini `0483:a3c4`, Pro `0483:a3c5`.
- Original serial is **experimental**: 9600 8N1; `v<0..3>x<0..100>` and
  `b<0..3> <0|1>` lines; inferred active-low presses; periodic `pong`. Source is
  MIT replacement firmware, not verified stock firmware. Unknown unique USB ID.
- Do not auto-open generic serial ports. Explicit `--serial` is the safe M1 fallback.
  Original never receives host commands. HID sends only documented state request,
  with `--no-init` available. No LED writes or firmware flashing in M1.
- Preserve raw values, separate press/release, bounded buffers/channels, clean
  handle ownership, finite read timeouts, fresh decoder state on reconnect.
- No code/assets from vendor binaries or unlicensed repositories. Record source
  revisions/licenses. Consulted protocol facts do not authorize copying GPL Java
  implementation into MIT files; this project has no imported upstream source.

## Implementation plan

1. **M1 (physical acceptance pending):** investigate → pure decoders → descriptors/transport → list/watch
   utility → mock/PTY tests → real Mini 1.0 validation on Windows/Nobara → positive
   automatic identity → per-model compatibility evidence. Physical gate pending.
2. **M2 (current):** audio contract/backends; physical knob → master then app;
   subscriptions, input/output mute, service recovery and mock backend.
3. **M3 (not started):** stable identity, mapping engine, groups, profiles, versioned
   persistent config/migrations and reboot/reconnect semantics.
4. **M4 (not started):** real desktop UI, onboarding/configuration, live feedback,
   settings/diagnostics and accessibility.
5. **M5 (not started):** tray/startup/background lifecycle and reliability soak.
6. **M6 (not started):** installers/packages, USB permissions, updates and clean installs.
7. **M7 (not started):** measured performance, UX/accessibility and visual polish.

Detailed acceptance criteria and risks live in `docs/MILESTONES.md`,
`docs/ARCHITECTURE.md` and `docs/RESEARCH.md`.

## Verification and handoff commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --workspace --release --locked
cargo run -p veek-probe -- list
cargo run -p veek-probe -- watch --duration 5
python3 tests/serial_pty.py target/release/veek-probe
python3 tests/capture_pty.py target/release/veek-probe
python3 tests/audio/pipewire_integration.py target/release/veek-audio-probe
```

All Python integration commands above are Linux-only. Audio integration uses a
private PipeWire server for routine unattended mutation tests. The latest request
also authorizes explicit live Nobara audio integration: document selected targets,
limit changes, and restore prior state; do not restart the user's desktop services.
Windows CI tests actual Windows build/runtime without physical USB. Do not run speculative commands against unrelated
hardware. Do not install host permissions or disable SELinux to get tests passing.
Document what ran, OS/toolchain, actual results, and unresolved checks in
`docs/VERIFICATION.md`; use `docs/HARDWARE_VALIDATION.md` for future physical trials.
