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

- Keep the VeekPanel brand plain: the user asked to remove the small tagline
  beneath the software name. Do not restore it.
- 2026-10-05: the user explicitly permits directly Apple-like visuals. The previous
  inspiration-only language was generated for them and is not their restriction.
- Keep each development pass focused on a concrete deliverable. Reuse established
  evidence and run relevant checks; do not repeatedly troubleshoot accepted hardware
  behavior or rerun broad checks without changes or an unresolved failure.

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
  an empty endpoint list: interactive Windows audio writes and physical PCPanel-to-audio
  checks remain pending. Basic Mini hardware input was validated subsequently. Do not equate the release with milestone acceptance.
- `crates/veek-config`: schema 2, profile-owned groups/preferences, validated imports, migrations, lock/conflict checks,
  atomic saves and backups. `crates/veek-core`: stable selectors, independent analog/
  button actions, groups, pickup, profile effects and ordered event coalescing.
- `crates/veek-runtime`: shared background owner and local JSON-lines test harness.
  Native backend/hardware reconnect rearms controls; simulated input requires Mock mode.
- `app` (separate Cargo workspace/lockfile) and `ui` (pnpm/Svelte/TypeScript) implement
  the actual Tauri desktop preview, wired to runtime IPC. Read docs/CORE_AND_DESKTOP.md.
  Compile ui/dist before the native app. No web server or fake audio in the app.
  Full M3/M4 acceptance, tray/startup reliability and distribution remain pending.
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
3. **M3 (initial implementation, acceptance pending):** stable identity, mapping engine, groups, profiles, versioned
   persistent config/migrations and reboot/reconnect semantics.
4. **M4 (native preview, acceptance pending):** real desktop UI, onboarding/configuration, live feedback,
   settings/diagnostics and accessibility.
5. **M5 (initial tray/background foundation, soak/startup pending):** tray/startup/background lifecycle and reliability soak.
6. **M6 (Windows preview installer; broader packaging pending):** installers/packages, USB permissions, updates and clean installs.
7. **M7 (initial measured optimization; native/polish acceptance pending):** performance, UX/accessibility and visual polish.

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


## Current software handoff (2026-10-05)

The M3 runtime and native M4 preview are implemented, not merely planned. Local
Nobara verification passed 49 tests plus actual native GUI/private-PipeWire
integration, including simulated Mini volume/mic mute, application relaunch matching,
relative groups, saved profiles and audio-service recovery. See the newest entries
in docs/VERIFICATION.md for exact source/CI evidence before repeating checks.

Next software increments: profile-specific group/device-preference overrides
(currently shared), clearer pickup/action feedback and accessible assignment UX,
then background lifecycle and distribution. The user permits directly Apple-like
visuals but wants the product name without a slogan underneath. Keep physical
Mini lifecycle/Nobara/knob-to-audio and interactive Windows audio acceptance open;
do not restart basic Windows Mini input research.


## Installer priority supersedes profile work (2026-10-05)

The user prioritized a downloadable Windows installer for the friend before more
feature work. The installer uses the verified schema-1 desktop app and preserves
the aligned, tagline-free brand. See docs/WINDOWS_INSTALLER.md and the newest
verification entry. New schema-2 profile edits are unfinished in the original
working tree and deliberately excluded from the installer; inspect them before
resuming, and do not imply their migration/integration passed. Installer work was
isolated in the codex/windows-installer worktree so those edits remain intact.

The user also supplied Antigravity work on `gemini-test`, reviewed 2026-10-07.
Commit `086288cc02bf4c25110582e49002a5276d312480` adds a Linux/Wine Windows
installer builder and produced an actual setup EXE. It was found locally and
pushed to preserve the work. Application behavior is unchanged from `48ac784`.
`QWEN-test` contains no commits beyond that same base. The release pipeline uses
native Windows CI, dependency notices and installation smoke checks; the Wine
builder remains on its branch. Do not confuse a produced EXE with a tested
Windows installation or physical PCPanel acceptance.

Windows desktop prerelease `v0.1.0-preview.1` is published from `71a1c77`.
Installer CI `37573097127` passed installation, responsive window/config creation,
reinstall and uninstall preserving settings on Windows Server 2025. See
docs/VERIFICATION.md for its exact digest and remaining consumer/hardware gates.
The Windows packaging CLI must run through `node .../tauri.js` in PowerShell to
preserve `-- --locked`; the smoke test waits for configuration after window creation.


## Profile continuation (2026-10-07)

The Windows installer priority was delivered, and the user authorized continued
development from the original brief. The formerly unfinished profile work is now
implemented and locally verified: schema 2, independent groups/device preferences,
complete profile duplication and cleared editing drafts on profile switches.
Legacy schema 0/1 migrations preserve behavior in every profile and back up the
original bytes. Hardware and appearance/tray settings remain global.

Local checks passed 52 Rust tests, strict workspace lint, frontend type/build
checks, private native PipeWire routing/relaunch/recovery, and actual Tauri GUI
profile editing/duplication/switching. Windows upgrade verification is being added
for desktop 0.1.1; check the newest VERIFICATION.md entry for final CI/release status.
The earlier schema-1 statements describe the 0.1.0 release, not current source.
Next work: clearer pickup/action feedback and accessible assignment UX, then
background lifecycle/startup reliability. Keep physical and soak gates open.


## Control feedback continuation (2026-10-07)

Reviewed main `345f79f`, gemini-test `086288c`, and the newer profile-settings
branch `8e60df9`. The profile branch passed all four core/desktop CI jobs and
the Windows installer/legacy-upgrade smoke workflow. Continue from that schema-2
foundation instead of reimplementing it. Linux/Wine packaging on gemini-test
produces a Windows installer; Linux desktop packages still remain pending.

The next M4 increment adds engine-owned pickup/readiness feedback, independent
button release state, external-volume reconciliation, and accessible assignment
navigation/form/status descriptions. No schema or platform-adapter rewrite.
See the newest VERIFICATION.md entry for this increment's actual checks.
Next software module: opt-in startup and background lifecycle/single-instance
behavior, followed by reliability and Linux distribution. Preserve physical,
consumer Windows and accessibility/soak acceptance gates.

## Startup/background continuation (2026-10-07)

The control-feedback patch was recovered unchanged onto the schema-2 foundation.
This increment adds explicit machine-local per-user startup registration, optional
global start-in-tray, duplicate-launch handoff and background-owner shutdown.
Linux requires an observed StatusNotifier host before hiding; missing/unconfirmed
tray or config initialization failures leave the window visible. No runtime imports
or profile operations change login registration. See docs/CORE_AND_DESKTOP.md and
docs/BACKGROUND_VALIDATION.md for behavior, compatibility and open acceptance.

Local Linux checks passed 57 workspace tests and 7 desktop tests, strict lint,
frontend checks/build and the native desktop build. Private sockets remain blocked;
the new GUI assertions and Windows CI are not executed here. GitHub writes remain
deferred, with a combined patch and docs/LOCAL_HANDOFF.md for the home checkout.
Next: execute native/Windows lifecycle checks, fix installer startup cleanup, then
reliability/Linux distribution. Keep physical, reboot and multi-day soak gates open.

## Installer cleanup and tray recovery continuation (2026-10-07)

Current-source Windows installers now remove only their exact owned HKCU login
command on ordinary uninstall. Same-path reinstall and `/UPDATE` preserve opt-in;
normal uninstall-based upgrades clear it. The published 0.1.0 installer is unchanged.
Linux continuously probes uncached tray-host state off-thread: observed loss reveals
once and disables hiding; recovery restores hiding eligibility without auto-hiding.
Exit wakes and joins the tray monitor before the background runtime.

Local checks passed 9 desktop tests with 1 private-D-Bus integration ignored,
strict desktop lint/formatting, locked native build and the actual NSIS hook's
compile-only check with warnings as errors. Windows smoke coverage is extended;
native Windows/D-Bus/GUI execution remains pending. See the newest verification
entry. Import the refreshed combined handoff before continuing on the home PC.
Next: native lifecycle verification, Linux distribution and measured performance.
Keep GitHub writes deferred and physical/reboot/soak acceptance open.

## Optimization continuation (2026-10-07)

The user prioritized optimization and plans an Astra pass for animations. The first
increment reuses complete verified native readback, shares unchanged immutable
runtime publications, omits unchanged runtime/mapping IPC payloads, and wakes
follow-up UI requests without adding idle polling. Native adapters, button barriers
and pickup/generation checks retain their contracts. See docs/PERFORMANCE.md and
the newest verification entry for measured results and limitations.

Local checks passed 59 workspace tests (1 performance probe ignored by default),
10 desktop tests (1 private-D-Bus test ignored), 4 frontend merge tests, strict
lint/formatting, frontend check/build and the native debug build. The release
performance probe ran explicitly against mock audio. Native GUI/PipeWire and
Windows/Nobara CPU/memory/physical latency remain pending. Do not convert these
software timings into physical claims or reopen the accepted basic Windows Mini
input investigation. GitHub remains deferred; use the refreshed combined handoff.

## Visual work and agent handoff (2026-10-07)

The latest user instruction supersedes the previous Astra-only animation plan:
Cloud 6.1 Sol handles architecture, optimization strategy, coordinated refactors,
code review and deciding the next task. Hand off when work needs local execution,
GUI/animation tuning, native hardware/platform behavior or home Git state.
Local Astra with Full Access owns those execution-heavy passes. The user accepts
some rendering cost for an awesome appearance. Preserve real backend wiring,
pickup/button safety, accessibility/reduced motion and the plain brand.
The new compiled visual foundation and its unexecuted native/visual checks are
documented in docs/UI_POLISH.md. Continue locally from evidence rather than
assuming compilation proves native layout or animation quality.

## Local native handback (2026-10-08)

The ZIP development bundle was verified/imported on `codex/native-visual-polish`
in `/run/media/PSSD2/VeekPanel-native-polish`; original history is retained through
`ffc6c43`. The visual patch is applied and native WebKit long-profile overflow is
fixed. Friendly tray-error text links to local diagnostic details. Original main,
Qwen/Gemini branches and dirty profile-checkout installer docs remain preserved.

Nobara checks now passed: 59 workspace tests, 10 desktop tests plus explicit private
D-Bus tray recovery, strict lint/format, native build, private PipeWire integration,
full GUI functional/startup-registration/duplicate-launch regression, keyboard
selection and 30 visual scenarios each at 100% and 200% (reduced motion verified).
Frontend check/build and four tests passed. Read the newest VERIFICATION.md and
UI_POLISH.md entries before rerunning. Native tests use fixed driver ports: run
sequentially. No physical/Windows/reboot/soak or frame-pacing acceptance is implied.

This completes the scoped local native handoff. Return the source and evidence for
Cloud Sol review/next-step selection. GitHub pushes, merges and releases remain
deferred; do not publish the pending 0.1.1 installer as part of this pass.

## Nightly publication authorized (2026-10-08)

The user explicitly requested a nightly branch containing all updated work,
including the installer and UI. This supersedes the earlier GitHub deferral for
this integration/publication. `nightly` starts from native-polish commit 08ade89
and retains the complete backend/profile/installer history. Main and older work
branches remain preserved. Use the native Windows installer pipeline; the old
Gemini Wine builder remains historical rather than replacing current-user setup,
notices or startup cleanup. Qwen has no additional application changes.

Installer triggers now include frontend and shared-crate changes as well as app
and packaging changes. Publish only a Windows-verified prerelease; retain exact
commit/checksum evidence and physical/platform limitations. A nightly branch does
not establish physical acceptance or create a scheduled automation.

## Current-source Windows hardening (2026-10-08)

User priority is release hardening on `nightly`, with no main merge or public
release just because an artifact builds. Current-source `94ed40539e2a881ba11b628b6b47afa12826924b`
passed Windows core and desktop CI, and built NSIS, but installer smoke stopped at
its absent-startup registry guard. Older installer success does not close this gate.
See the newest VERIFICATION.md entry for exact runs and subsequent results.

The Windows smoke now handles absent opt-in normally and uses test-only loopback
WebView2 CDP on a disposable hosted runner to inspect the bundled UI and call real
native startup/config commands. It checks quiet duplicate autostart, manual recovery
from start-minimized, available-tray close/reopen, config ownership and existing
fresh/reinstall/update/migration/uninstall preservation cases. No production debug
hook or dependency was added. Native smoke results are recorded below.
Do not claim consumer Windows, actual login, tray-shell, physical audio/HID or soak
acceptance from hosted CI. Preserve accepted Nobara visuals and Mini input evidence.
After current-source Windows validation, prefer AppImage -> native Nobara -> Flatpak;
actual package/build/run, GUI, PipeWire/HID and login cycles belong to local Astra.

Local Astra verified the supplied patch checksum and clean application against
unchanged nightly 94ed405, then applied it in the clean nightly worktree. The
original profile working tree remains dirty and untouched. The cloud HTTP 403 was
specific to its integration; local publication uses the existing authorized CLI.
Native hosted results for the applied changes are recorded below.

## Hosted Windows result and Linux GUI evidence (2026-10-08)

Exact nightly source `d341f80b5372f0e2c601de1195f1c5eadecdf930` passed the entire
Windows installer smoke, including installed WebView2/native startup/background,
reinstall/update, schema-1 upgrade and scoped uninstall. Windows core/desktop and
Linux core also passed. Read the newest VERIFICATION.md entry for run/job/artifact
IDs and the remaining consumer/physical/login/soak limits; no release was published.

Linux desktop passed build/lint/tests and private tray recovery, then lost its
WebKit driver connection while reading a preferred output after switching profiles.
This resembles the earlier connection reset but its cause is not established.
Do not weaken assertions, blindly replay mutations or claim a product fix from
another successful rerun. The harness now retains command/error/process evidence
before screenshot/cleanup, and CI uploads native logs/results/captures even on
failure. Four isolated evidence regressions and subsequent native execution passed
(see completion below). Next: review retained Linux native evidence, resolve the cause, then
continue AppImage -> native Nobara package validation -> Flatpak with local Astra.


## Release-hardening completion (2026-10-08)

Windows hardening source d341f80 passed installer run 37732328374/job 113164039343.
The later test/docs-only e76aaac passed all four jobs in run 37807217907; Linux
native GUI job 113414524577 also passed and its retained result has no transport
error. The same harness passed locally on Nobara. Read the newest VERIFICATION.md
entry for exact hashes, artifact IDs and limits. No production code changed after
the Windows-verified source. The earlier Linux connection failures remain
unexplained; diagnostic retention must not be described as a fix.

Stop this focused pass here. Recommended next agent: Cloud Sol for review/next-step
selection and intermittent Linux driver triage, then local Astra for AppImage,
native Nobara package testing and later Flatpak. Preserve current Windows evidence,
accepted UI/physical scopes, and all personal dirty work. No main merge or release
was performed or is authorized by this hardening task.

## AppImage preparation / next native boundary (2026-10-08)

Cloud review of 0af5de9 confirms no production/Windows packaging changes after
verified d341f80 and the full passing e76aaac Linux job log. Artifact bytes were
unavailable here; Astra's recorded ZIP inspection remains attributed to Astra.
The intermittent driver cause is open, but current evidence supports no specific
product fix or broad rerun before a bounded AppImage packaging increment.

A separate app/appimage.conf.json and portable guide prepare bundling without
changing default/Windows builds or host permissions. Read docs/APPIMAGE.md for
the exact native handoff, notices/build scope and stop conditions. Config/schema
checks are preparation only; actual package build/run is pending with local Astra.
Retain original failures without mutation replay. No main merge/public release,
Flatpak work or physical/consumer/login/reboot/soak acceptance is implied.


## Local AppImage result (2026-10-08)

The isolated AppImage pass built and executed an actual 109.92 MiB package from
c830725. Private native GUI/audio/startup/duplicate/relaunch checks passed, including
AppImage-path startup and graceful lock release; a KDE file-handler launch rendered
on the Nobara Xwayland desktop with isolated config and read-only host audio.
See docs/APPIMAGE.md and the newest VERIFICATION.md entry for exact SHA256/paths,
build environment correction and limits. No production/Windows/lock changes.

The artifact is LOCAL-ONLY, not cleared for redistribution: native inventory covers
210 ELF files/139 RPM packages, but hyphen license text, AppRun/runtime provenance,
non-ELF resources and corresponding-source obligations remain unresolved. Generated
native notices are not embedded in the tested artifact. Bundled libraries require
glibc 2.43; only this Nobara host was tested and PipeWire libraries remain host-side.
Stop this increment here. Next agent: Cloud Sol to review native distribution
closure/build baseline and produce a bounded packaging plan; then local Astra to
repack and perform fuller Nobara package validation. Flatpak stays later. Preserve
all earlier physical/lifecycle limits; no main merge or public release.

## AppImage continuation result (2026-10-08)

The user explicitly said "keep going", superseding the earlier pass's stop/Cloud
review requirement. Packaging source 855d553 has a repacked, tested Nobara 44 x86_64
AppImage with embedded native notices and a collected companion of all 127 exact
native SRPMs plus runtime/libfuse/squashfuse source. Its staged payload omits 132
unrelated host schemas/typelibs; all 212 ELF files remain unchanged. Native audit
now covers data, symlinks and pinned external artifacts; no missing origin/text
remains for this payload. Read the newest VERIFICATION.md and docs/APPIMAGE.md
for hashes, source/launcher reproducibility limits, commands and actual results.

The new artifact/source companion is in
/home/austinp/Downloads/gaem/VeekPanel-Nobara-Preview. Private packaged GUI/audio/
startup/ownership/relaunch and actual KDE file-handler Xwayland launch passed.
This replaces the earlier missing-notice/source stop for this artifact, not the
older local-only prototype. Share the companion sources/notices with the AppImage.
No public release, main merge, Windows rebuild or host installation was performed.
Nobara 44/glibc 2.43 only; host PipeWire required. Older Linux is not validated.

Next concrete task: native Nobara integration/packaging, isolated desktop/login/
tray acceptance, then Flatpak; assess a clean older build baseline for broader
AppImage distribution. Local Astra owns native checks, Cloud Sol can review the
focused packaging changes. Do not restart basic Mini or Windows smoke work.
Preserve personal dirty work and all physical/consumer/login/soak limits. Linux
GUI driver transport loss did not recur; its historical cause remains unknown.


## Optional Doom tab (2026-10-09)

The user explicitly requested the actual Doom game as a fun extra tab. A separate
Doom.svelte embeds Internet Archive's original shareware episode only after Play;
Stop/tab exit/document-hidden/offline removes it. No backend/config/hardware or
package dependencies change. Only the exact embed URL is added to frame-src;
remote native capabilities and parent message bridges are not enabled. See
docs/DOOM.md for provider, lifetime behavior and native acceptance limits.
Frontend checks/four tests/build pass; actual gameplay/native frames remain
unverified in cloud. Do not replace accepted release evidence with a gameplay claim.

Local continuation: checksum-verified Doom patch is applied in codex/doom-tab.
Actual Nobara native E1M1 gameplay, keyboard movement, non-silent private audio and
Stop were observed; frontend check/four tests/build and locked native build passed.
The optional live-provider harness uses private PipeWire/Pulse and WirePlumber's
policy-only profile, never physical hardware or personal audio settings. Read the
newest DOOM.md/VERIFICATION.md for final lifecycle and CI results/limits. Older
installers/AppImage do not contain this feature; no new release is authorized.
