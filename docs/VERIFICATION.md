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
