# Verification record

Initial implementation: 2026-10-03 UTC / 2026-10-02 America/New_York.
Current milestone status: **M1 physical acceptance pending; M2 audio diagnostics implemented.**
No physical PCPanel was available; the user confirmed the target is a friend's Original
with four pushable knobs. M2 was subsequently authorized; M3–M7 are not started.

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
