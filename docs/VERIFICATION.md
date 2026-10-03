# Verification record

Initial implementation: 2026-10-03 UTC / 2026-10-02 America/New_York.
Current milestone status: **M1 prototype implemented; physical acceptance pending.**
No physical PCPanel was available; the user confirmed the target is a friend's Original
with four pushable knobs. M2–M7 are not started.

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
passed. Windows build/test and kit results for this continuation are recorded after CI.

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
- No audio backend, configuration persistence, groups/profiles, GUI, tray or package.

Next physical work must follow `HARDWARE_VALIDATION.md`, starting with the Original.
