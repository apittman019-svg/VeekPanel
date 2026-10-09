# Verification record

Initial implementation: 2026-10-03 UTC / 2026-10-02 America/New_York.
Current milestone status: **M1 basic Mini controls passed on one Windows 11 unit;
lifecycle/Nobara physical acceptance pending. M2 native audio, M3 core and an
M4 backend-connected desktop preview implemented; production acceptance pending.**
No physical PCPanel is available locally. The user corrected the friend's target
to **Mini 1.0** on 2026-10-04. Historical entries below retain the Original
assumption in effect at the time. See CONTINUATION_REQUEST.md for expanded scope.

## Local environment and completed checks

Nobara Linux 44, KDE Plasma Desktop Edition, x86_64. Rust/cargo 1.99.0.
User-scoped Rust was installed to build the project; no shell startup files or host
device permissions were modified. Native libudev development files already existed.

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` | Passed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed |
| `cargo test --workspace --locked` | 17 tests passed (10 protocol, 4 session, 3 CLI) |
| `cargo build --workspace --release --locked` | Passed on Nobara |
| Early Windows-native HID backend cross-target type check | Passed, then superseded: dependency inspection found synchronous writes report zero. Final implementation uses upstream HIDAPI Windows C; native Windows CI is required. |
| Malformed/truncated/oversized replay CLI inputs | Rejected with nonzero exit, bounded parsing |
| Nonexistent explicit serial port, 4-second live run | Reported failed opens, retried and exited normally at deadline |
| Original offline fixture | Correct independent knob and button labels; explicitly offline |
| Pro offline fixture | Correct knob/button and separate slider labels |
| Linux PTY integration | Passed actual serialport I/O with synthetic fragmented input, press/release, disconnect/reopen, parser reset and Ctrl+C |
| Live `list` | Ran successfully; 0 recognized PCPanel HID interfaces, serial candidates kept unverified |
| Live `watch --duration 5` with no panel | Waited and exited normally, no fabricated connection/control events |
| `udevadm verify packaging/linux/70-veekpanel.rules` | Passed syntax/semantic validation; rules not installed |
| Local Markdown links and `git diff --check` | Passed |
| Dependency metadata/license inventory | Recorded in `DEPENDENCIES.md` |
| No-device release process, 10-second sample | 10.10s elapsed, 0.04s user CPU + 0.06s system CPU, 3424 KiB max RSS; illustrative CLI sample only, not GUI/hardware performance |

Protocol tests include exact VID/PID matching, all accepted raw analog values and
indices, button polarity, Pro slider bounds, malformed/truncated/oversized HID,
serial chunk boundaries, invalid serial syntax, overflow recovery, partial EOF and
arbitrary byte inputs. Session tests cover timeouts, errors, short writes, invalid
reports followed by valid ones, no serial writes and decoder reset on new sessions.

## CI and additional verification

The workflow in `.github/workflows/ci.yml` runs formatting, lint, tests and release
builds on Windows and Ubuntu; Ubuntu also runs the synthetic serial integration.
[Run 37094820268](https://github.com/apittman019-svg/VeekPanel/actions/runs/37094820268)
passed both jobs for source commit `25ece7627d738463f1b3882b1e3ffdb4ac32663d`,
including the final upstream Windows HIDAPI C backend. All 17 tests ran on Windows
and Ubuntu; release builds passed on both. Ubuntu PTY reconnect passed as well.
The deprecated checkout action was then updated to a pinned v6 revision.
[Final workflow run 37094930940](https://github.com/apittman019-svg/VeekPanel/actions/runs/37094930940)
also passed both jobs on commit `8072086d9781281fb92a68f3ade5db6e5f6289e2`, with
the updated checkout action. The subsequent verification-record commit changes
documentation only and skips redundant CI.
A Windows runner is not a Windows 10/11 hardware validation.

`cargo doc --workspace --no-deps --locked` also passed locally. The repository is
public at [apittman019-svg/VeekPanel](https://github.com/apittman019-svg/VeekPanel).

## M1 continuation — friend testing support (2026-10-03)

The user clarified that the Original belongs to a friend, who will test a download
when convenient. No hardware was connected or physically validated in this continuation.
Added bounded raw serial capture (new directory, byte/time limits, no protocol writes,
no reconnect mixing), explicit unverified metadata, and a guided console test with
a physical-observation worksheet. CI stages downloadable Windows/Linux diagnostic
kits with dependency notices; these are M1 test utilities, not M6 installers.

Local Nobara checks passed: formatting, strict Clippy, 21 Rust tests, release build,
the existing serial PTY test, and a new synthetic capture/guide integration. That
integration preserves invalid bytes, replays captured data, records Ctrl+C and
disconnect, and produces the observation worksheet. Linux dependency-notice generation
passed.

[Continuation CI run 37141431162](https://github.com/apittman019-svg/VeekPanel/actions/runs/37141431162)
passed Windows and Ubuntu jobs for source commit
`f65350a4d1b25261410caddfc44504d1f64b44dd`: all 21 Rust tests, formatting,
strict Clippy, release builds, license collection and kit packaging. Ubuntu also
passed both synthetic PTY integration scripts. The 21 tests comprise 10 protocol,
4 session, 3 capture and 4 CLI tests.

Downloaded the final CI archives and checked archive integrity, required kit files,
exact `BUILD_COMMIT.txt`, dependency notices and included unmodified MPL source.
The packaged Linux executable ran on Nobara and passed both PTY scripts there.
Inspected the Windows PE imports after enabling static CRT linkage: no external
VCRUNTIME/MSVCP dependency remained. This inspection does not replace running the
kit on a friend's clean Windows machine or testing physical USB.

Published the verified archives and SHA-256 checksums as the unsigned experimental
[M1 diagnostic prerelease](https://github.com/apittman019-svg/VeekPanel/releases/tag/m1-probe-2026-10-03).
The release targets the source commit above; subsequent handoff edits are documentation
only. No hardware acceptance is inferred from publishing these downloads.

## Remaining limitations (unchanged physical gate)

- Stock Original VID/PID, USB bridge, serial grammar, button polarity, heartbeat,
  physical ordering and range are not verified. Adapter derives from a compatible
  MIT replacement-firmware reference, not a stock capture.
- Original automatic identity is unresolved. Explicit port selection is required;
  changed COM/tty paths require a new selection unless a stable by-id path is used.
- RGB/Mini/Pro are implemented from public references but physically untested.
- No tested LEDs, firmware query, silent-reset recovery, suspend/resume or multi-day
  reliability guarantee. `bcdDevice` is not promised as firmware version.
- Discovery/retry uses a 3-second portable scan; hotplug notifications are future
  work. Slow stdout can backpressure the bounded queue. No audio latency benchmark.
- No real permission-denied PCPanel, duplicate-interface or multi-panel checks.
  Rules are provided, not installed; normal-user access with actual hardware remains
  pending. No blanket serial rule is supplied without verified identifiers.
- M2 now provides native audio diagnostics (see below). No configuration persistence,
  groups/profiles, GUI, tray or full application package.

Next physical work must follow `HARDWARE_VALIDATION.md`, starting with the Original.

## M2 native audio continuation (2026-10-03)

The user authorized the next step after the M1 diagnostic release. Scope is now
native audio diagnostics plus the temporary one-knob/button path; original physical
M1 evidence is still pending. See [AUDIO_VALIDATION.md](AUDIO_VALIDATION.md).

Implemented Windows Core Audio and native PipeWire adapters, a shared contract with
mock tests, and a separate JSON diagnostic CLI. M1's hardware utility remains audio-free.
Local Nobara checks passed: formatting, strict Clippy, 30 Rust tests, workspace
release builds; Windows Core Audio cross-target type/lint checks also passed.

Read-only enumeration against the actual Nobara user server found four outputs,
two inputs and the current default output/input with their volume/mute values.
No user audio was mutated. Mutation tests used a private PipeWire daemon with
synthetic output/input objects and actual pw-cat playback/capture clients. They
exercise native volume/mute, default metadata, channel balance, external state,
stream lifecycle, restart/stale-ID rejection and synthetic PTY knob/button binding.
No physical PCPanel or audible hardware route was tested.

Build headers were extracted from the matching Nobara pipewire-devel 1.6.8 RPM
into `/tmp/veekpanel-pw-sdk`; local builds used its pkg-config path and the installed
native runtime. No host packages, permissions or desktop audio configuration were
changed. Standard build prerequisites are documented in LINUX_SETUP.md. Dependency
notice generation passed, including an exact-source MIT notice missing from the
cookie-factory registry archive. Final CI and artifact evidence follows.

The release-build integration passed on Nobara for both playback and recording
clients, as did both existing M1 PTY integrations. `cargo doc` and diff checks passed.
A 3-second read-only M2 watch on the live server used 0.00s user/0.00s system CPU
(at the timer's precision), 6888 KiB maximum RSS; this is a short diagnostic sample,
not a latency/production performance guarantee.

Initial M2 CI run 37175555119 passed Windows build/lint/30 tests, archive generation,
and native Core Audio initialization/enumeration with an empty endpoint list on the
hosted runner. It did not test Windows volume writes or real devices. Ubuntu exposed
an older pw-cat without `--raw`; the isolated test now uses portable WAV fixtures.
That corrected fixture passed on Nobara before resubmission to CI.

### Final M2 CI and published artifacts (2026-10-04)

[Run 37175760562](https://github.com/apittman019-svg/VeekPanel/actions/runs/37175760562)
passed both Windows and Ubuntu jobs for source
`237421c12bd453adc4994af0bb94e6c734c76098`. Each ran formatting, strict Clippy,
30 Rust tests, release builds and dependency notice/archive generation. Ubuntu
passed both M1 PTY tests and the corrected native PipeWire integration. Windows
passed native Core Audio startup/enumeration; its hosted runner had no endpoints,
so neither volume/mute writes nor audible hardware were validated there.

Downloaded both final M2 archives. Checked archive integrity, required files, exact
BUILD_COMMIT identity, dependency notices and included unmodified MPL source.
The Ubuntu-built Linux executable passed the complete isolated native audio test
on Nobara, including playback/recording clients and synthetic Original PTY binding.
The Windows PE import table has no external VCRUNTIME/MSVCP dependency; only OS
DLLs are imported. This is static inspection, not clean-Windows acceptance.

Published the unsigned experimental [M2 audio diagnostic release](https://github.com/apittman019-svg/VeekPanel/releases/tag/m2-audio-2026-10-04)
with both archives and SHA256SUMS.txt. GitHub's asset sizes/digests match every local
file; the release is public, non-draft and explicitly a prerelease targeting the
verified source commit. The subsequent handoff commit changes documentation only.

Next M2 acceptance work is interactive Windows volume/mute/session lifecycle tests,
physical Nobara audio route tests, and the friend's Original protocol/knob/button
trial. Read AUDIO_VALIDATION.md for the full matrix. M1 and M2 physical acceptance
remain open; no M3–M7 features were implemented or authorized in this continuation.

## Device-model correction and returned evidence (2026-10-04)

The user corrected the device identification from Original to **Mini 1.0** and
reported that they had figured out the hardware issue. No specific Mini HID control
results have been supplied yet; this is not recorded as physical acceptance.

The returned private `thing.zip` contains an Original-assumption serial capture:
zero bytes in `serial.bin`, only the header in `chunks.tsv`, final stop reason
`interrupted`, elapsed 24,892 ms, and an unfilled RESULTS worksheet. No code from
the archive was executed. This cannot establish a Mini protocol failure or pass.
The published START-HERE launcher selects the Original serial workflow; the Mini
should use the already implemented HID discovery/watch path. Documentation now
makes that distinction explicit. The private ZIP is excluded from Git.

## Mini-specific diagnostic kit implementation (2026-10-04)

Added `veek-probe test-mini` and Windows `TEST-MINI.cmd`. The command only selects
Mini HID VID:PID 0483:a3c4; unknown IDs, other models and ambiguous interfaces are
not opened. It gives timed physical-control prompts, preserves bounded reports
(including malformed input), decoded events and times, and writes an explicitly
unverified summary plus the physical worksheet. No COM selection is needed.
No-input captures are clearly flagged. Each run uses a new directory; disconnect
ends the capture for a separate reconnect trial. No audio, LED or firmware actions.

Local Nobara checks: package formatting, strict Clippy, release build and all
26 hardware/probe tests passed. New tests cover Mini-only selection/ambiguity,
malformed raw report preservation, empty/capped captures, all four knobs/buttons
via synthetic offline replay, and an exact nonexistent HID path that produces
finished no-input evidence without opening hardware. These are software tests,
not proof that the friend's physical Mini 1.0 works.


[Mini CI run 37212010661](https://github.com/apittman019-svg/VeekPanel/actions/runs/37212010661)
passed Windows and Ubuntu for `e703b4cf70eaa03c5168b5f16a4ffc72d6249e8a`:
formatting, strict Clippy, all 35 workspace tests, release builds, dependency
notices and packaging. Ubuntu passed both synthetic serial integrations and the
isolated PipeWire suite. The local serial/capture regression integrations also passed.

Downloaded both CI archives. Verified Windows ZIP integrity, TEST-MINI.cmd command,
checklist, included notices and exact BUILD_COMMIT. Windows PE imports only OS DLLs;
no external VCRUNTIME/MSVCP dependency was observed. This is not a clean-machine
physical test. Ran the downloaded Linux binary's Mini help and forced-nonexistent
HID path on Nobara: it returned a nonzero exit with finalized no-input evidence,
without opening a device. No physical Mini was attached.

Published [Mini diagnostic prerelease](https://github.com/apittman019-svg/VeekPanel/releases/tag/m1-mini-2026-10-04)
with Windows ZIP, Linux archive and SHA256SUMS. The public, non-draft prerelease
points to the verified source. GitHub asset sizes and SHA256 digests match local
files. Physical Mini 1.0 acceptance remains pending the friend's returned HID
capture and observations; the earlier empty serial ZIP does not count as evidence
of Mini behavior. This follow-up commit only records verification and skips CI.


## Reviewed physical Mini capture — Windows 11 (2026-10-04)

The friend returned the Mini kit's capture plus completed observations using source
`e703b4cf70eaa03c5168b5f16a4ffc72d6249e8a`. Independently replayed all 4,473 raw HID
reports (286,272 bytes): zero errors and exact agreement with every decoded event
in the returned log. All reports are 64 bytes with zero padding after byte 2.
Observed `0483:a3c4`, interface 0, usage `ff00:0001`, USB release `0200` (firmware
unknown). Duration stop, 90,915 ms including setup; 4,445 analog and 28 button reports.

Every knob reached raw 0 and 255. The worksheet confirms physical knobs 1–4 from
left to right, clockwise increasing, full travel, and press/release working for
each. Button press/release counts are 2/2, 7/7, 3/3, 2/2, with alternating edges
and each final state released. No missing/stuck/repeated/unexpected events were
reported. **Basic automatic detection and knob/button decoding passed for this
unit on Windows 11. No protocol change was needed.**

User identification is Mini 1.0; printed model/revision, firmware and Windows build
remain unknown. The competing-apps field is blank. Reconnect/rerun is marked Yes
without a specific outcome or second capture; in-process recovery is not proven.
Sleep/resume is explicitly NOT TESTED; Nobara physical behavior, audio actions,
Windows 10 and long-duration reliability remain untested by this evidence.

The raw ZIP remains private and unchanged. Added 16 exact, non-identifying 64-byte
reports (extrema and both button edges for each knob), preserving source order,
with [hashes/provenance](../tests/fixtures/mini-windows-2026-10-04.md) and a regression
replay check. It is a selected physical excerpt, not a continuous trace or a new
hardware trial. Historical “pending” entries above describe the earlier state.

Local Nobara checks for the physical excerpt: 27 hardware/probe tests passed,
including full-frame replay of the excerpt; package formatting, strict Clippy and
`git diff --check` passed. This replay is regression verification, not an additional
physical test. Unrelated audio/configuration work remains outside this evidence commit.


## Backend-driven desktop preview (2026-10-05)

Continued the existing working tree rather than redoing hardware investigation.
Nobara 44, Rust 1.99.0, Node 24 / pnpm 11.25.0. Native development headers and GUI
test tools were extracted into a user-owned cache; no host packages, device
permissions, SELinux policy or desktop services were modified.

Implemented schema-1 configuration with atomic saves, backups, migration and
conflict/lock protection; stable target selectors; independent analog/button
mappings; relative/equal audio groups; pickup and release-before-press; profiles;
a GUI-independent background controller; and a native Tauri/Svelte desktop preview.
Its explicit development Mini drives the same engine as HID/serial events and
controls actual native audio. The window has assignments, live target volume/mute
and hardware position, mixer, profiles, editable groups, appearance/device settings,
validated import/export, diagnostics and initial tray support. Old hardware input
is discarded at connection, mapping and profile boundaries.

Local verification on the final implementation of this increment:

- Root formatting, strict Clippy, release build and **49 tests passed**. Includes
  four config, six mapping-engine and two background-runtime tests; physical Mini
  fixture regression is preserved. One Linux-specific lifecycle test is additional
  to the 48 portable tests.
- Desktop Rust formatting/strict Clippy/build passed. Svelte/TypeScript check:
  **zero errors and warnings**; production frontend build passed.
- Existing isolated native PipeWire integration passed: outputs/inputs/app streams,
  volume/mute/channel balance, default changes, external notifications, disappearance,
  synthetic serial pickup/buttons, service restart and stale-ID rejection.
- New persistent-runtime integration passed against a private native server:
  saved Mini mappings, pickup/button edges, stale UI rejection, backups/relaunch,
  application identity surviving a new live stream ID after app relaunch, relative
  output/application group levels, service recovery and no stale writes.
- Native GUI automation passed with actual Tauri/WebKit/IPC, Xvfb and a private
  PipeWire server. Through the UI it saved knob-to-output and button-to-microphone
  mappings, changed native output to 200/255, toggled microphone mute, duplicated
  the profile and persisted dark/light settings. Native readback was independently
  checked with pw-dump. Screenshots inspected; no horizontal overflow at the tested
  1200px window. Synthetic endpoints/panel are explicit test inputs, not physical
  hardware or audible-output acceptance.

The earlier live Nobara audio pass in this continuation also passed: host default
output reduced by one percentage point and restored, secondary output/input mute
restored, an explicitly named unlinked pw-cat stream discovered with stable ID/binary,
volume/mute controlled, and disappearance observed. All changed endpoint state was
restored and verified. No desktop service restart was attempted. That result is
retained rather than unnecessarily mutating host audio again for identical checks.

M2 improvements include client-level application metadata fallback and narrowly
scoped handling for PipeWire objects disappearing during bind/destruction. Other
native errors remain visible; permission/write failures are not swallowed.

Remaining acceptance: interactive Windows audio/GUI, physical Mini-to-audio,
Nobara USB and physical lifecycle, actual tray/close behavior, accessibility/scaling,
performance/soak and distribution. Groups and device preferences are currently
shared across profiles; profile-specific overrides and richer identity editing
remain M3 work. This initial usable foundation does not mean all M3/M4 requirements
or M5 reliability have been accepted. Windows/Linux CI results are recorded below
once available; a Windows build alone is not interactive audio validation.


[Initial desktop CI run 37324159698](https://github.com/apittman019-svg/VeekPanel/actions/runs/37324159698)
for `f398f0f8d634a51c70a93c9ab37350b499a2fb70` passed Windows and Ubuntu core
checks (48 portable / 49 Linux tests), release diagnostic packaging and integrations.
Windows desktop formatting, frontend checks, strict native lint and build passed.
Ubuntu desktop also built, but its GUI harness initially failed before launching
the app: tauri-driver requires an absolute WebKitWebDriver path. The harness now
resolves the installed executable through PATH and fails immediately if missing;
the default-path invocation subsequently passed locally. The user's requested
removal of the brand tagline is included in `7668113`, with refreshed native
screenshots in docs/images. No new physical-device claims are made.


[Final CI run 37324994766](https://github.com/apittman019-svg/VeekPanel/actions/runs/37324994766)
**passed all four jobs** for `7668113eb3056acd81bf0c14a6c686c4bca4f64f`:
Windows/Ubuntu core formatting, strict lint, tests, release builds and diagnostic
packaging; Linux serial/native audio/persistent-runtime integrations; and both
native desktop builds with frontend type checking. Ubuntu's actual Tauri GUI
integration also passed after the driver-path correction. Windows ran 48 portable
tests; Linux ran those plus its PipeWire lifecycle test (49). Windows interactive
GUI/audio and physical Mini-to-audio are still untested. The subsequent handoff
commit contains only documentation and screenshots and skips redundant CI.

## Windows desktop installer and branch review (2026-10-07)

Reviewed the user's Antigravity worktree and preserved its local commit
`086288cc02bf4c25110582e49002a5276d312480` on public `gemini-test`. It adds
Linux/Wine cross-build scripts and produced a real NSIS setup EXE; inspection
confirmed an x64 Windows app inside its build. Its application source is the
unchanged `48ac784` desktop foundation. `QWEN-test` has no changes beyond that
base. These findings do not establish that the Wine-built installer was tested
on Windows. The existing unfinished schema-2 profile edits remain separate.

Published [Windows preview v0.1.0-preview.1](https://github.com/apittman019-svg/VeekPanel/releases/tag/v0.1.0-preview.1)
from `71a1c77d6d7fc84ea9580885fecc0c74f8f08bf9` using the native Windows
release pipeline. It packages schema 1, current-user NSIS installation, static
MSVC runtime, automatic missing-WebView2 download, project/dependency licenses,
applicable unmodified MPL dependency source and getting-started instructions.
The Wine builder remains preserved on `gemini-test`, rather than being required
by the native Windows pipeline.

[Installer CI 37573097127](https://github.com/apittman019-svg/VeekPanel/actions/runs/37573097127)
**passed** on GitHub's Windows Server 2025 runner:

- Locked frontend install, zero-error/warning Svelte check and production build;
  dependency-notice generation; locked native Rust release/NSIS build.
- Silent install and installed application/dependency-notice presence.
- Installed app opened a responsive native window and created configuration.
- Same-version reinstall preserved the configuration bytes.
- Uninstall removed the application and preserved configuration bytes.

Two packaging/test issues were resolved without changing app behavior: invoke
the pinned Tauri CLI through Node directly so PowerShell's shim does not swallow
the Cargo argument separator; wait for startup/configuration as well as the
window handle before testing saved defaults. The initial responsive window can
appear before Tauri's setup callback completes. The final test still fails if
initialization does not complete within 30 seconds.

[Standard CI 37572505739](https://github.com/apittman019-svg/VeekPanel/actions/runs/37572505739)
passed all four Windows/Linux core and desktop jobs for `e0c9f56` (the same
application and packaging implementation, before the smoke-test timing fix).
The release EXE was downloaded locally and its hash checked against the CI
checksum, then against GitHub's published asset digest:

- File: `VeekPanel_0.1.0_x64-setup.exe`, **2,474,378 bytes**.
- SHA-256: `d4bb646af11e3646caaf696be9e91c9b3af9ca440546e1aa83ef84ab042f6c16`.

This is an unsigned desktop preview. Windows 10/11 consumer clean installs,
missing-WebView2 download on a clean consumer machine, interactive Windows audio,
physical Mini-to-audio/lifecycle, upgrades across versions, Linux packages and
production/soak acceptance remain pending. Basic Windows Mini input evidence is
unchanged. Installer success does not close all M6 or physical milestone gates.


## Independent profile settings (2026-10-07)

Resumed the preserved profile edits after delivering the first installer. Schema 2
moves groups and preferred input/output into each profile. Schema 0/1 migrations
copy former shared settings into every profile and preserve the original bytes in
the normal backup. Malformed/future configurations and prior backups are unchanged
on rejection. Group IDs resolve within their owning profile. Duplicate copies
everything independently; new profiles start empty; switching clears editing drafts.

Local Nobara checks: **52 Rust tests passed**, workspace formatting and strict
Clippy passed; frontend check reported zero errors/warnings and production build
passed; native desktop built. Private native PipeWire integration confirmed distinct
profile destinations, same-ID groups with different membership, pickup reset on
switch, application relaunch matching, persistence/relaunch and service recovery.
Actual native Tauri GUI automation passed group/preference creation, full duplication,
independent edits and draft clearing after switches, plus existing simulated Mini
volume/microphone mute and dark/light checks. Native dark screenshot was inspected.
No physical USB or desktop audio was changed by these private-server tests.

Desktop version is 0.1.1. Windows CI now also installs the published 0.1.0 baseline
(verified by its pinned SHA-256), loads representative legacy settings, upgrades
the app, checks schema-2 migration and exact backup, and checks relaunch/reinstall/
uninstall preservation. CI and published artifact evidence will be recorded below
when completed. Consumer Windows and physical Mini-to-audio acceptance remain open.


## Live control readiness and accessible assignments (2026-10-07)

Reviewed current main `345f79f`, gemini-test `086288c`, and schema-2 continuation
`8e60df9`. Profile-settings CI [37574468939](https://github.com/apittman019-svg/VeekPanel/actions/runs/37574468939)
passed all four core/desktop jobs; Windows installer workflow
[37574468881](https://github.com/apittman019-svg/VeekPanel/actions/runs/37574468881)
also passed, including its legacy upgrade check. The next feature builds on
that verified branch, preserving the original installers and profile work.

Added `veek-core::feedback` with engine-owned pickup/controlling, button readiness,
offline/missing/blocked states, physical position and observed group-peak pickup
level. Runtime snapshots reconcile live target changes and native readback before
publishing feedback. External volume changes rearm pickup without a synthetic
hardware event. Failed/unconfirmed writes cannot publish active control.
The desktop explains pickup direction/level and independent button readiness,
retains partial-group warnings, and supports keyboard selection with accessible
labels, a submit form, and a selected-control live region.

Local Ubuntu 24.04 workspace, pinned Rust 1.99.0, Node 24/pnpm 11.25.0:
**56 Rust tests passed**, workspace formatting and strict Clippy passed; frontend
check reported zero errors/warnings and production build passed. Native desktop
build/formatting/strict Clippy and the full workspace release build passed. Python
integration scripts passed syntax checks. Four new regression tests cover readiness transitions,
external changes, unavailable/ambiguous/capability-limited targets, group pickup and
independent buttons, plus native-write rejection through the mocked runtime.
Build dependencies were extracted into a temporary sysroot. No user's desktop
audio, USB permissions or configuration was changed.

This workspace prohibits private sockets (`socket(AF_UNIX, SOCK_STREAM)` returns
EPERM), so local private PipeWire/native GUI integration cannot execute. The
existing CI tests were extended to check feedback/rearming through native PipeWire
and actual WebDriver keyboard selection. Git HTTPS push had no authenticated
credential; the connected GitHub integration returned HTTP 403, Resource not
accessible by integration, for repository writes. No remote feature branch or PR
was created and new CI could not be triggered. The complete change is committed
locally on codex/control-feedback and supplied as an apply-ready patch based on
8e60df93119a9cd2697d07dd0f2beca52f4f821d. Run native integrations/CI after importing
that patch; do not represent them as passed in this workspace. Manual screen-reader behavior, consumer Windows interaction, physical
PCPanel-to-audio/lifecycle, Nobara USB and full reliability acceptance remain open.

## Opt-in startup and background lifecycle (2026-10-07)

Recovered the saved control-feedback patch onto `8e60df9` as local `d7bfe86`.
No rewrite of the schema-2 profile foundation, Windows/Wine branch, native audio
adapters or hardware protocols. This increment adds:

- Machine-local startup registration only on an explicit UI action: HKCU Run on
  Windows; an atomic owned XDG autostart entry on Linux. Paths are quoted/rejected
  safely; writes are serialized/off-thread and verified by OS registration readback.
- Optional `settings.start_minimized`; earlier schema-2 files default false without
  rewriting. Config failures and missing/unconfirmed tray recovery show the window.
  Linux probes the StatusNotifier host off-thread before allowing hiding.
- First-plugin single-instance handoff; manual launches reveal/focus the existing
  window, duplicate login launches stay quiet; existing config-lock guard remains.
- Explicit shutdown joins the runtime owner, rather than relying on process teardown.

Local Ubuntu 24.04 remote workspace, pinned Rust 1.99.0:
**57 workspace Rust tests and 7 desktop Rust tests passed**. Both formatting checks,
strict workspace/desktop Clippy, frontend zero-error/zero-warning check, production
frontend build and locked native desktop debug build passed. The modified native
GUI integration script passed syntax compilation. Temporary Linux test directories
were the only startup files modified; no personal login registration was enabled.

CI now runs app unit tests on Windows/Linux and uses a private session bus for the
native Linux GUI test. Added GUI assertions cover scoped startup readback/removal,
start-minimized persistence and duplicate-launch handoff. This workspace still
denies private Unix sockets, so these new integrations have **not run here**.
Windows registry code, new Windows CI/installer behavior, actual login delivery,
tray lifecycle and shutdown under native failures need the home/CI/manual checks.
No new release or Windows installer is produced by this increment.

GitHub writes remain deferred at the user's request after the integration's 403.
The combined handoff patch includes feedback plus this increment against `8e60df9`;
follow docs/LOCAL_HANDOFF.md in an isolated home worktree. Do not assume a Git pull
will contain these local changes. Startup cleanup on uninstall/relocation remains
follow-up distribution work; disable the checkbox before removing this preview.
Physical Mini/audio, Nobara USB, consumer Windows, accessibility and soak gates
remain open.

## Scoped Windows startup cleanup and Linux tray recovery (2026-10-07)

Continued from local startup/background commit `961fb5d` on the schema-2 branch.
The current-source NSIS post-uninstall hook compares the exact quoted installed
executable plus `--autostart` before deleting only HKCU Run value
`org.veekpanel.desktop`. Install never enables it; other paths/changed commands
are preserved. Same-path reinstall and `/UPDATE` replacement retain opt-in.
An ordinary upgrade that uninstalls the old app clears the old owned entry;
re-enabling requires the user. Linux removal/relocation remains manual.

Integration uses the supported hook contract in pinned Tauri CLI 2.12.1 /
bundler 2.10.1 source. The stock template removes the product-name Run value,
whereas VeekPanel runtime registration uses the bundle identifier. No upstream
template was copied or replaced. The published 0.1.0 release is unchanged.

Linux tray monitoring repeats an uncached StatusNotifier-host probe off-thread
after a three-second idle interval, with a two-second D-Bus method timeout.
Observed loss reveals the window once and disables hiding; observed recovery
restores hiding eligibility without automatically hiding. UI reveals are queued
without waiting on the UI thread. Exit wakes and joins the monitor before runtime
shutdown. A method timeout is not a guarantee that every connection/authentication
step terminates in that period, and host presence does not prove icon rendering.

Local Ubuntu 24.04 remote workspace, pinned Rust 1.99.0:
**9 desktop Rust tests passed, 1 private-D-Bus integration test ignored**;
desktop formatting, strict all-target Clippy and locked native debug build passed.
The two new unit tests cover recovery transitions and prompt idle-worker shutdown
with probe ownership released. The private-bus fixture compiled and is wired into
Linux CI to exercise host-property changes and watcher loss/replacement, but did
not execute here because private Unix sockets remain prohibited.

NSIS 3.09 compiled the actual startup cleanup hook in an uninstaller section with
`-WX` (warnings as errors). The synthetic EXE was never executed. Windows smoke
assertions now cover a path with spaces, native manual/login duplicate handoff,
graceful shutdown/lock release, no implicit startup, owned-only cleanup, unrelated
and other-installation values, and `/UPDATE` removal/replacement preservation.
The published schema-1 baseline skips new single-instance assertions. These new
Windows checks have not executed here; PowerShell was reviewed but no local
PowerShell parser/runtime is available. The compiler check is not native acceptance.

Root Rust/frontend sources are unchanged in this follow-up; their 57-test,
lint/type/build evidence from the preceding entry remains applicable. No new
installer, release, GitHub push or CI result is claimed. The refreshed combined
handoff includes all local increments against `8e60df9`; follow LOCAL_HANDOFF.md.
Native Linux GUI/D-Bus, consumer Windows, login/reboot, physical Mini/audio,
Nobara USB and full accessibility/reliability soak acceptance remain open.

## Measured runtime and desktop optimization (2026-10-07)

The user prioritized optimization and reserved animations for an Astra pass.
Baseline local source `13a4903` was profiled with an explicitly injected mock
backend: 100 targets, 8 profiles and 100 alternating armed analog writes. A
baseline-only test patch and raw before/after data are retained in tests/perf.
Three isolated release-mode baseline runs used 498–500 audio snapshots for 100
confirmed writes; three final runs used 300 for the same 100 confirmed writes.
Burst p99 moved from 20.605–20.766 ms to 0.263–0.418 ms. Full state acquisition
previously copied data in 25.102–28.161 ms per 1,000 calls; the desktop's new shared
acquisition took 0.008–0.009 ms. These are fixture results, not physical latency,
native CPU/memory, UI rendering or an overall speedup claim. Scheduling/load
variation and exact reproduction are documented in PERFORMANCE.md.

Successful input/write commands reuse the complete verified native observation
for publication. Validation and readback remain mandatory; failures reconcile and
rearm. Immutable publications change only when observed state changes and are
distinct from saved-config revisions. Unchanged visible IPC replies omit runtime
and mapping data while still checking live machine-local tray/startup status.
Frontend merging preserves references and rejects missing/mismatched caches.
After command handling, native events are dispatched and the owner waits on the
command queue so follow-up requests wake it immediately. Idle scheduling remains
bounded to the same 20 ms interval; isolated requests/native calls can still wait.

Local Ubuntu 24.04 remote workspace, pinned Rust 1.99.0, Node 24/pnpm 11.25.0:
**59 workspace tests passed (1 probe ignored by default), 10 desktop tests passed
(1 private-D-Bus integration ignored), and 4 frontend merge tests passed**.
Strict workspace/desktop Clippy, both formatting checks, frontend zero-error/
zero-warning type check, frontend production build and locked native desktop debug
build passed. The release probe ran explicitly. After the final command-wakeup/
publication-lock change, runtime regressions, strict lint and native build passed
again. New tests cover complete readback and failed-readback invalidation, stable
idle publications and external updates without config revision changes, conditional
payload resync/desktop status, and frontend identity/cache/profile transitions.

No native GUI/private PipeWire/Windows CI result is claimed in this remote socket-
restricted environment. Their existing integration checks remain required, along
with Windows/Nobara CPU/RSS, startup, physical-input latency and multi-day soak.
GitHub writes remain deferred. The refreshed handoff includes the combined patch
and a scoped Git bundle preserving the local commits and performance baseline.

## Visual foundation / Astra boundary (2026-10-07)

The exact optimization source was restored from the published 8e60df9 base and
saved combined patch, then a new UI foundation was built. The source recovery
verified every restored UTF-8 blob against its Git SHA; four unchanged binary
assets remain on GitHub and are not included in the incremental visual patch.
This scratch checkout has synthetic history; the handoff patch targets the original
ffc6c43 optimization source, not that synthetic commit.

Frontend frozen-lockfile install, type check (0 errors / 0 warnings), production
build, all four snapshot tests and git diff --check passed. No dependencies or
backend code changed. Bundle sizes: CSS 18.44 kB / 4.88 kB gzip; JS 78.19 kB /
28.94 kB gzip. These do not establish idle resources or physical/native latency.

New original SVG icons, themed layered surfaces/sidebar, physical-position dial
and Pro fader visuals, active profiles and page/selection/press transitions retain
the native command contracts, unknown-value distinction, keyboard hooks and
reduced-motion fallback. Native/WebView layout, animation tuning, scaling and
Windows/Nobara GUI checks are unexecuted for this source; Cloud browser download
failed. Local Astra will perform them according to the user's agent split.
See UI_POLISH.md. No hardware, release, GitHub push or M4/M7 acceptance is claimed.

## Home handoff: native visual and lifecycle checks (2026-10-08)

Source: original backend history through `ffc6c43`, plus the imported visual patch
and local fixes on `codex/native-visual-polish`. Nobara 44 KDE, pinned Rust 1.99.0;
native GTK/WebKit under private Xvfb, D-Bus and PipeWire with disposable config.
The original profile checkout, main, Gemini and Qwen branches remain unchanged.

Passed:
- 59 workspace Rust tests; one optional performance probe remained ignored.
- 10 desktop tests; the one normally ignored private-D-Bus tray-host loss/recovery
  test was then run explicitly and passed on its disposable bus.
- Workspace and desktop strict Clippy and formatting; locked native desktop debug
  build and release runtime build.
- Private native PipeWire runtime integration: persistent mappings, simulated Mini
  pickup/buttons, stale UI rejection, application relaunch, profile-local groups/
  preferences, backups/relaunch, service recovery and no stale writes.
- Full native GUI regression: actual private audio writes/readback, pickup, mic mute,
  independent profiles/groups/preferences, cleared switch drafts, keyboard focus,
  opt-in private XDG startup creation/readback/removal, start-minimized persistence
  and duplicate normal/autostart launch handoff.
- 30 native visual captures at 1× and 30 at 2× with reduced motion. Mini/Pro themes,
  both window sizes, long names, absent targets and offline states had no horizontal
  overflow. 2× metrics confirmed devicePixelRatio=2 and reduced-motion=true;
  relevant computed animation/transition styles were disabled. See UI_POLISH.md.
- Final frontend type check: zero errors/warnings; four snapshot tests; production
  build; Python harness compilation and git diff whitespace check.
  Final CSS 18.64 kB (4.92 gzip), JS 78.58 kB (29.06 gzip); not runtime benchmarks.

Native review found and fixed profile-select overflow specific to WebKit. Technical
tray errors were moved behind a friendly banner's Diagnostics link. Unsupported
WebKit driver click/resize operations now fall back to X11 on the private test
display. One attempted overlapping test run failed at driver session creation;
sequential final runs passed. Private D-Bus emitted expected unavailable desktop
portal/systemd messages; no host service restart or startup change was performed.

Final local artifacts: `/tmp/veek-polish-final` (full functional + visual matrix),
`/tmp/veek-polish-hidpi` (2× reduced-motion matrix). Current backend tests passed
locally; earlier Windows CI for `8e60df9` is not evidence for these newer commits.
Current-source Windows/WebView2/installer tests, real desktop login/tray recovery,
physical Mini audio/lifecycle/Nobara, CPU/RSS/frame pacing and soak remain open.
No new installer release, GitHub push or milestone acceptance is claimed.

## Current nightly Windows gate review (2026-10-08)

Inspected authoritative project/verification/background/UI requirements, desktop
and startup implementation, NSIS hooks/config, both workflows and recent nightly
history. For exact source `94ed40539e2a881ba11b628b6b47afa12826924b`:

- [Core/desktop run 37727227802](https://github.com/apittman019-svg/VeekPanel/actions/runs/37727227802):
  Windows core job `113148061875` and Windows desktop job `113148062173` passed.
  This includes locked Rust tests/builds, formatting/strict Clippy, readonly native
  Core Audio enumeration and frontend check/tests/production build. Linux core
  job `113148062115` passed; Linux desktop job `113148062182` failed at native GUI
  profile creation with driver connection reset after its build/tests passed.
- [Installer run 37727227761](https://github.com/apittman019-svg/VeekPanel/actions/runs/37727227761),
  job `113148061663`, built the current-source NSIS installer on Windows Server 2025.
  Smoke failed immediately at `Get-ItemPropertyValue` for the absent opt-in HKCU
  startup value. Fresh install/launch/migration/reinstall/uninstall assertions did
  **not** execute; verified installer upload was correctly gated off.

Hardening fixes absent-value reads without hiding registry access errors. A
stdlib-only, loopback CDP helper observes the actual installed WebView2 Dashboard,
invokes real native snapshot/startup/save commands, and checks enable/readback/remove
idempotence without config changes. Native HWND visibility and the running owner
validate start-minimized safeguards, quiet duplicate autostart, manual relaunch and
available-tray close/reopen. Existing fresh/reinstall/update/legacy migration and
scoped uninstall checks remain intact. No production code or mock backend changed.

Cloud static checks: both PowerShell files parsed with tree-sitter-powershell
0.26.4 without errors; workflow YAML parsed. These are **static syntax checks**,
not PowerShell/native runtime acceptance. The workflow also runs the real Windows
PowerShell parser and retains a source/run/OS/result manifest plus smoke transcript
on success or failure. New native smoke results are pending execution.

Hosted Server CI cannot establish consumer Windows 10/11, missing-WebView2 clean
machine bootstrap, real tray-shell interactions, logout/login, interactive audio,
physical PCPanel, sleep/resume or soak acceptance. No release is authorized merely
by producing an installer. The prior Nobara native results and accepted Mini basic
input evidence remain valid within their original scopes.

Publication blocker: GitHub rejected creation of the hardening tree with HTTP 403,
`Resource not accessible by integration`. The connection can read the repository,
but this attempted write was denied. No commit was created, no ref changed and no
new native CI was triggered. Changes are delivered as a reviewable patch for local
Astra; native PowerShell, new installed WebView2/startup/background assertions and
the complete current-source installer smoke remain **unexecuted** for this patch.


### First native hardening attempt

Applied/pushed source `5cb896a532deb9e69e3f547801ad9b4e2d23197b`.
Run 37731482289 passed all jobs: Windows core 113161349335, Linux core
113161349488, Linux desktop 113161349595 and Windows desktop 113161349621.
The Linux GUI driver-connection failure did not reproduce; its complete native
smoke passed without a Linux code change. This does not establish flake elimination.

Installer run 37731482529/job 113161350507 passed native parsing, UI build,
NSIS construction and fresh native window/config ownership, then failed connecting
to the test-only WebView2 debugging port. Result/transcript retained in the
VeekPanel-Windows-validation artifact. The absent-startup-value guard is fixed.
Microsoft's WebView2 issue 5645 documents that elevated hosts ignore environment
switches starting with Runtime 150. The follow-up uses documented per-executable
HKLM debug policy only for elevated disposable CI, refuses pre-existing values,
and removes it in finally. No production hook or persistent product setting is
added. Actual rerun results remain pending; the runner's elevation is recorded.

### Completed hosted Windows smoke and recurring Linux driver loss

Reviewed GitHub job steps and full logs for exact nightly source
`d341f80b5372f0e2c601de1195f1c5eadecdf930` on 2026-10-08:

- [Installer run 37732328374](https://github.com/apittman019-svg/VeekPanel/actions/runs/37732328374),
  job `113164039343`, **passed the complete smoke** on Windows Server 2025.
  Actual installed WebView2 rendered Dashboard and returned native schema-2 state;
  startup enable/detect/remove was idempotent and did not rewrite config. Hidden
  startup policy, quiet duplicate autostart, manual recovery and available-tray
  close/reopen passed. Fresh install in a path with spaces, graceful shutdown,
  reinstall, `/UPDATE`, foreign-startup preservation, published 0.1.0 upgrade,
  both profiles' schema-1 migration, exact backup, relaunch and uninstall/settings
  preservation all passed. The log confirms temporary WebView2 policy removal
  after each elevated test session.
- Validation artifact `11530203374` has archive SHA256
  `0e4a77b7eabbd156aba7bd189f6356972dd9e4dbadc0d199d0103a512d6cce77`.
  Installer artifact `11529983920` contains setup EXE and SHA256SUMS; its **ZIP archive**
  digest is `05a39a35f4137dbc238fbbc97d7413a083924729622b61fe14f101a3ff6b9cb2`.
  This is not the EXE checksum. Both expire 2026-11-07; no new release was published.
- [Core/desktop run 37732328390](https://github.com/apittman019-svg/VeekPanel/actions/runs/37732328390):
  Windows core `113164039799`, Windows desktop `113164039769` and Linux core
  `113164039619` passed. Linux desktop `113164039793` passed frontend checks,
  Rust lint/tests/build, private-D-Bus tray recovery and NSIS hook compilation,
  then **failed** during native GUI integration. At native_smoke.py line 173,
  reading the preferred-output select after a profile switch lost the driver
  connection (`hyper` connection reset / Python `RemoteDisconnected`). This is
  a transport failure, not a failed preference-value assertion. The retained log
  does not establish whether the driver, WebKit, application or environment caused it.

This pass adds test-only request/result/error tracing without retries, captures
failure and owned helper exit status before diagnostic screenshot/session cleanup,
and retains the full native log, result JSON and available captures as
`VeekPanel-Linux-GUI-validation` for 30 days on success or failure. Pre-harness
failures still require the job log; absence of result.json is not success.
Failed HTTP mutations are never replayed. Product/backend code is unchanged.

Cloud Ubuntu 24.04 / Python 3.12.14: four isolated evidence regressions passed,
including lost mutation transport without replay and preservation of the original
failed command after successful screenshot collection. Python compilation,
workflow YAML parsing and git diff whitespace checks passed. Native Rust/WebKit,
PipeWire and Windows execution of this harness change have not run here; their
hosted/native follow-up remains required. The existing GUI transport failure is
still open. No M5/M6 acceptance, consumer Windows, actual login/tray-shell,
physical audio/HID, sleep/reboot or soak claim follows from hosted Windows success.


### Local completion and exact-source hosted evidence (2026-10-08)

Local Astra independently downloaded and reviewed the d341f80 Windows validation
transcript/result. WebView2 153 rendered the bundled Dashboard and returned native
schema 2; startup enable/readback/remove, duplicate ownership/background recovery,
graceful shutdown/lock release, fresh install, reinstall, update replacement,
legacy upgrade/migration/backup and scoped uninstall all passed. The runner was
elevated. Close/reopen is conditional on actual tray readiness; the transcript
reports the combined case, not a separate tray-branch value. This is not consumer
Explorer menu/login or standard-user/elevation-specific acceptance.

Downloaded installer artifact 11529983920 and independently verified the actual
`VeekPanel_0.1.1_x64-setup.exe` against SHA256SUMS:
`5900b85783186bb6a39dde634ea59a2c4d4e5bfd17f8927d08bb011d0805f7db`.
No release was published. The installer remains a CI artifact of d341f80.

The Linux evidence handoff checksum matched and applied cleanly on unchanged
d341f80. Local four-test regression suite, Python compilation, whitespace checks
and actual Nobara private native GUI smoke passed. Existing native binary was
reused because production app/UI/backend sources did not change. Local evidence
is `/tmp/veek-hardening-linux-local/result.json`; helper processes were alive before
normal cleanup, failure=null. Personal audio/startup and dirty profile docs stayed
untouched.

Committed/pushed evidence source `e76aaaceee1432e72932d5fcaed2d76bf419252c`.
[Run 37807217907](https://github.com/apittman019-svg/VeekPanel/actions/runs/37807217907)
**passed all four jobs**:

| Job | ID | Result |
| --- | --- | --- |
| Windows desktop | 113414524270 | Passed build/lint/tests and four evidence regressions |
| Linux core | 113414524553 | Passed, including private native audio/runtime integration |
| Windows core | 113414524570 | Passed |
| Linux desktop | 113414524577 | Passed, including four evidence regressions, private tray recovery and complete native GUI smoke |

Linux GUI artifact `11563438828`, archive SHA256
`7299d3beada1a0d3f354a56932349f4126e4b3afe65a9b0a97652a7f0e0c6878`,
was downloaded and inspected. It contains native.log, screenshots and result.json;
result identifies e76aaac/run 37807217907, status=passed, last_error=null,
failure=null and all three owned helpers alive before cleanup. The GUI reached
its final layout check after native audio, mappings, profile switching and startup
checks. The recurring driver failure did not reproduce on this run. Its cause
remains unknown; evidence capture is not a product or transport fix. There were
no mutation retries or weakened assertions.

No app/crates/UI/Windows packaging files changed between d341f80 and e76aaac, so
the passing installed Windows result covers the same production and installer
code; the final docs-only commit does not imply a separate installer execution.
Recommended next owner: Cloud Sol for review of this completed gate and the open
intermittent Linux driver issue, then local Astra for AppImage/package execution
when approved as the next increment. Order remains AppImage -> actual Nobara
package validation -> Flatpak. No main merge or release; physical, consumer,
real login/tray-shell, sleep/reboot, resource measurements and soak stay open.

### Cloud review and AppImage preparation (2026-10-08)

Reviewed nightly 0af5de925f0466009772c2b8a374c90be6b48482 and both intervening
commits. A Git comparison confirms app/UI/crates/Windows packaging and locks are
unchanged from Windows-verified d341f80. Independently fetched the latest four job
results and the full Linux job 113414524577 log: complete native GUI smoke passed,
and the log records artifact 11563438828 with the previously documented digest.
Its signed artifact-byte download returned HTTP 403 here; ZIP/result/screenshot
inspection remains Astra's attributed evidence, not this cloud review's work.

No specific production fix follows from intermittent driver resets at differing
requests. Portal/display warnings occur in the passing log too. The cause stays
open; preserve failing native evidence instead of mutation retries or speculative
workarounds. Selected next increment: separate AppImage bundle overlay, inert HID
resource and portable guide, then one bounded native package build/run with local
Astra. docs/APPIMAGE.md records the build recipe, license inventory, actual
AppImage-path startup check, compatibility scope and failure stop conditions.

Ubuntu 24.04 / Python 3.12.14 cloud preparation checks passed: base plus overlay
validated with jsonschema 4.26.0 against the exact Tauri CLI 2.12.1 schema; declared
checked-in resources exist; original app/build settings and production/default/
Windows source remained unchanged; git diff whitespace check passed. Generated
notices and their native-library inventory are not yet present. No Rust build,
AppImage creation/launch, new GUI run or hosted CI execution is claimed for this
patch. This adds packaging preparation, not M6 or physical/lifecycle acceptance.


### First local AppImage package pass (2026-10-08)

Preparation was applied on an isolated `codex/appimage-preparation` worktree at
`/run/media/PSSD2/VeekPanel-appimage`, based on unchanged nightly 0af5de9.
Source commits: 6f73b2d (preparation), c830725695fcdebf1e228a92a364827a49594a6e
(exact upstream license supplements for dlopen2, dlopen2_derive and
libappindicator-sys). Crate revisions and license hashes are in each supplement's
PROVENANCE.txt. No production Rust/UI/default/Windows config or lock changed.

Build host: Nobara 44 KDE x86_64, glibc 2.43, Rust 1.99.0, pinned Tauri CLI 2.12.1,
GTK 3.24.52, WebKitGTK 2.52.5 and PipeWire 1.6.8. Frozen frontend install/build and
fresh Rust/frontend notice generation passed after the omitted licenses were
supplied. Locked release compilation passed. The initial bundle attempt failed
because the old SDK pkg-config search override produced two library directories
where the bundler expected one. Host development packages already existed; no
system installation or permission change was needed. Removing PKG_CONFIG_PATH
for the bundle-only invocation resolved the error without changing the bundler.

Commands used the documented recipe plus local Node/Rust PATH and shared
CARGO_TARGET_DIR=/run/media/PSSD2/VeekPanel/app/target. Compilation used the existing
SDK pkg-config path; successful bundling used native host metadata:

```sh
cd app
# Existing release binary, unchanged production source; no dependency upgrade.
unset PKG_CONFIG_PATH
node ../ui/node_modules/@tauri-apps/cli/tauri.js bundle --config appimage.conf.json --bundles appimage
```

The repeat bundle-only step warned that the bundle-type marker was already absent
after the first attempt had patched the binary. It completed successfully; no
updater behavior is claimed. The built file is local-only at
`/run/media/PSSD2/VeekPanel/app/target/release/bundle/appimage/VeekPanel_0.1.1_amd64.AppImage`.
Size: 115263992 bytes (109.92 MiB). SHA256:
`1373bf5f20038027b200da6dd2ebac1bd087b16e9763a8bff7ac246896edde3c`.
Its source stamp is c830725 with an empty build diff. Subsequent changes affect
only audit/test/documentation; this exact artifact was tested and not repacked.

### Actual package execution

Copied the executable AppImage to `/tmp/VeekPanel AppImage trial/VeekPanel.AppImage`
and launched it normally through its runtime/FUSE, not an extracted/debug binary.
The existing native harness passed against that file with disposable configuration,
private D-Bus/Xvfb/PipeWire and explicitly simulated Mini input: real Dashboard,
private audio discovery/write/readback, pickup/mic mute, mapping/profile saves,
profile isolation, duplicate owners, tray-unavailable visible fallback, startup
opt-in/readback/removal and light/dark rendering. The startup Exec assertion used
the launched AppImage path itself, not its transient .mount path. Registration was
removed; this does not test login delivery or a user's permanent installation.

Added an opt-in package relaunch check (`VEEK_APPIMAGE_RELAUNCH=1`): native
WM_DELETE_WINDOW close, released config lock, then a new actual AppImage session
with identical saved config bytes/profile and absent startup registration. The
full targeted package run passed; no mutation retries or production hooks.
Tauri's existing automation support was used without adding release features.

```sh
VEEK_APPIMAGE_RELAUNCH=1 dbus-run-session -- python3 tests/gui/native_smoke.py '/tmp/VeekPanel AppImage trial/VeekPanel.AppImage' /tmp/veek-appimage-relaunch
```

A separate KDE `kioclient exec` file-handler launch opened the real AppImage on
the current Nobara desktop through Xwayland, with private D-Bus/config and no test
automation flag. Captured the rendered Dashboard, observed PipeWire connected,
verified hardware disabled/no mappings/no startup entry, then closed the owned
window gracefully. Host audio was read-only. This validates the KDE file-handler
route, not a manually observed Dolphin double-click, native Wayland, Explorer-like
tray integration or login. Screenshot and launch result are in
`/tmp/veek-appimage-desktop`; private package results/captures are in
`/tmp/veek-appimage-relaunch` and `/tmp/veek-appimage-private`.

### Audit and distribution stop

AppDir inspection found desktop/icon/AppRun, embedded-UI executable, Rust/frontend
notices, project guide/license and inert HID rule. The RPM/build-ID audit matched
210 native ELF libraries/helpers to 139 installed packages; it records modified
bundle hashes, preserved build IDs, source RPM identities, license expressions and
available original license texts. AppRun.wrapped is an external bundler artifact,
not RPM-owned; AppImage runtime reports type2-runtime commit 8f39b89. These require
separate tool-origin/license/source review. Hyphen 2.8.8's local package has no
license text available to this audit. Native non-ELF resources and source provision
for bundled copyleft components also require review. The audit exits nonzero for
known gaps instead of certifying completeness.

`packaging/linux/native_inventory.py AppDir fresh-output-directory` reproduces the
inventory; reviewed result is `/tmp/veek-appimage-native-audit-reviewed`. Native
notices were generated AFTER the tested artifact and are NOT embedded in it.
Do not redistribute this prototype. This bounded pass stops at that concrete
packaging-review gate; no public AppImage/release asset or Flatpak was produced.

The main executable requires GLIBC_2.39; bundled native libraries include
GLIBC_2.43 requirements. Nobara-only evidence cannot support Ubuntu/older-glibc
claims. libpipewire is host-resolved rather than bundled, so the tested host's
PipeWire library/SPA installation remains a prerequisite. Full native dependency,
source/license closure and choice of an older build baseline belong to the next
packaging review before repacking/distributing. Do not disguise this as a portable
or completed Linux installer.

No broad unchanged Windows/core/visual matrix was rerun. Python compilation and
whitespace checks passed. The audit's nonzero result is intentional evidence of
incomplete distribution review. No observed GUI driver loss occurred in these
package runs, but its historical cause remains open. Preserve physical Mini,
consumer Windows, actual login/desktop-menu/USB setup, native Wayland, relocation,
sleep/reboot, resource/soak gates. Original dirty work and native-polish checkout
were preserved; main and releases remain untouched.
### Nobara AppImage distribution contents and repack (2026-10-08)

The user's "keep going" superseded the preceding bounded-pass stop. Reviewed
unchanged origin/nightly 1fcb8df and preserved the original dirty README/Windows
installer docs plus the clean native-polish checkout. Packaging source is
`855d5531c12f10c1696b2c7b0673ae08648094cf`. No production/UI/audio/Windows/lock
changes; no public release/main merge. This packaging-only pass ran local native
checks instead of unchanged hosted Windows/core jobs; there is no new CI run/job.
Earlier Windows run 37732328374/job 113164039343 and full passing run 37807217907
remain the corresponding evidence for unchanged production source.

The bundler had copied the host's entire GSettings/GI metadata directories,
including unrelated applications. `stage_appdir.py` copies to a fresh staging
location, verifies resource bytes/owners, retains data from the already included
native source packages plus GTK's GNOME desktop schema dependency, and recompiles
schemas strictly. It omitted 132 unrelated data files and retained 113. It does
not prune native libraries, GTK input modules or accessibility code. All 212 ELF
files (app, launcher and 210 native helpers/libraries) are byte-identical to the
first tested c830725 package. Original AppDir/artifact remain intact.

Expanded inventory matches 210 native ELF files and 113 data files to 150 RPM
packages/127 source packages. Eight internal symlinks were checked. No unmatched
files or missing license texts remain in the reviewed staging payload. Hyphen's
original licenses/authors came from its exact source RPM with recorded hashes.
The old AppRun asset matches Tauri's published SHA256; the generated GTK hook's
input script matches pinned Tauri CLI 2.12.1 byte-for-byte. Original launcher,
GTK-plugin, linuxdeploy, runtime and static-dependency notices are now embedded.
The runtime Makefile also links mimalloc, so its notice is included despite the
upstream top-level license list omitting it.

Fetched and validated all 127 exact Fedora SRPM identities and payload digests,
recorded download URLs/SHA256s, and collected runtime commit 8f39b89's source/build
recipe/libfuse patch plus libfuse 3.15.0 and squashfuse 0.5.2 source archives. Both
dependency archive hashes match that runtime's pinned build script. No host package,
repository configuration, USB permission or service was changed. The source
collector rejects malformed paths, wrong versions and truncated RPMs (checked).
Source ZIP CRC verification passed. Engineering inventory/source provision is
not a legal certification or byte-reproducible runtime build: the AppRun mirror
has no exact source revision, and permissive static-library notice revisions are
not measurements of every embedded library version. These limits are documented.

Repacked the staged AppDir using the existing linuxdeploy AppImage output plugin,
with explicit LDAI_RUNTIME_FILE extracted from the prior tested image. Runtime
input: 944632 bytes, SHA256
`502fea1d14b4582c3acad1d9c2d23a86987f8930a10f95d5a26d2adf9accbd85`;
version 8f39b89e2ac31e1640b3d3f7e9a5108e6ce805fa. Embedded notices separately
record application c830725 and packaging 855d553. The final image was extracted
for inspection: every recorded library/resource/tool/project payload hash matched
its embedded inventory. This inspection was not substituted for actual execution.

Output directory: `/home/austinp/Downloads/gaem/VeekPanel-Nobara-Preview`.

| File | Bytes | SHA256 |
| --- | ---: | --- |
| VeekPanel-Nobara-x86_64.AppImage | 113891832 | f6a35230d68710323158941ef7000ee986bf94dc5400d3eccdf3e505fb97cdce |
| VeekPanel-corresponding-sources.zip | 884427727 | 0680aa8d3f8e64833896e44c4b69bc4310d358d84a8f02383df6a7176ca14efb |
| VeekPanel-source-855d553.tar.gz | 513124 | 38931c9dc20e2582a7327ee10b0b430dc480792231006de6f142e1bdeb26c160 |

The package and its source/notices companion are prepared for a Nobara 44 x86_64
test handoff. Share them together. This supersedes the missing-notice/source gate
for the new artifact only; the earlier 1373bf5 prototype remains superseded.
The AppImage is a portable preview, not an RPM/system installer or cross-distro
release. Required bundled-library glibc is still 2.43; PipeWire remains host-resolved.

Final artifact verification passed:

- Actual runtime/FUSE launch from `/tmp/VeekPanel Final AppImage/VeekPanel.AppImage`
  through the private Xvfb/D-Bus/PipeWire harness with VEEK_APPIMAGE_RELAUNCH=1.
  Real Dashboard/native IPC, private discovery/write/readback, simulated Mini
  pickup/mic mute, mappings/profiles/groups/preferences, startup opt-in/readback/
  removal using the AppImage path, duplicate owner, tray-unavailable fallback,
  dark/light, native close/lock release and byte-preserving saved-config relaunch.
- KDE file-handler launch of the final Downloads artifact on actual Nobara
  Xwayland, disposable config/private D-Bus, hardware disabled and host audio
  read-only. Screenshot inspected; native PipeWire connected, no mappings or
  startup registration. Owned window closed gracefully. This is not native Wayland,
  manual Dolphin double-click, actual desktop-shell tray or login acceptance.
- Python compilation/whitespace, source rejection cases, source archive CRC,
  original ELF byte comparison and extracted embedded inventory checks passed.

Evidence is retained under the output directory's `verification/`; original logs
are `/tmp/veek-appimage-855d553-smoke` and `/tmp/veek-appimage-855d553-desktop`.
The passing GUI result retains an expected WebDriver unsupported-click error from
the harness's existing supported fallback; it is not a connection loss. There
was no driver transport failure in this run. Its historical intermittent cause
remains open; do not claim that packaging changes fixed it.

Next: native Nobara package/desktop integration and explicitly isolated login/
tray testing; evaluate an older clean build baseline before broader Linux claims.
Flatpak remains subsequent. Physical Mini, host USB setup, consumer Windows/audio,
real login/tray, native Wayland, sleep/reboot, measured performance and soak gates
remain open. No physical acceptance, full M5/M6 completion or new Windows installer
is implied by this AppImage handoff.


## Optional Doom tab preparation (2026-10-09)

Based on nightly dfd3b166012e5a8d40fe1cca3279a3ae0bd414ab. User requested the
actual game as an extra tab. Added lazy, sandboxed Internet Archive shareware
player with Play/Stop, keyboard help, offline feedback and destruction on tab exit
or document-hidden/offline events. Exact frame-src allowance only; no backend,
config schema, hardware, native audio, lockfile or package dependency changed.

Cloud checks passed: pnpm ui check (zero errors/warnings), four existing ui tests,
production build (CSS 19.34 kB / 5.06 gzip; JS 81.57 kB / 30.25 gzip), Tauri JSON
parse and whitespace check. The exact published shareware embed returned HTTP 200.
Playwright browser installation failed with a truncated/non-ZIP download, so no
rendering/gameplay/input/fullscreen, native WebView2/WebKit, tray-hidden event or
remote-IPC-denial acceptance is claimed. No new native CI/build/installer/AppImage
result is implied. See DOOM.md for bounded local acceptance. Existing accepted
Windows/Nobara/physical evidence remains historical to its exact tested source.

### Local Doom integration (2026-10-09)

Verified bundle patch SHA256 b9e160d3cfe1b30e05bb6f83fc6e84a880a4362744b41ee814f038bf32578404
against unchanged nightly dfd3b16; applied in codex/doom-tab, preserving existing
worktrees and dirty personal docs. Frozen frontend install/check/four tests/build,
locked native Nobara debug build, Python compilation and whitespace checks passed.
Actual native Doom E1M1 gameplay/keyboard movement and non-silent private audio
were observed; Stop removed the iframe. Evidence: /tmp/veek-doom-policy, including
screenshots, result.json, audio-result.json and private game-audio.wav. Additional
lifecycle results and exact source/CI identifiers follow after completion.

No Rust/backend/config-schema/hardware/audio/lock/dependency changes. Remote native
capabilities remain absent; direct adversarial remote IPC testing was limited by
WebKit cross-frame automation timeouts. See DOOM.md for original failure evidence,
private audio routing and pending Windows/fullscreen/scaling/real-shell checks.
Old packages are unchanged and do not contain Doom. No release or main merge.
