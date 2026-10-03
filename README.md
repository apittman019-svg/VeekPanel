# VeekPanel

A Windows-first, open-source PCPanel controller project with first-class Nobara/Linux support.
**Current scope: research and Milestone 1 hardware prototype only.** No audio control,
desktop UI, profiles, tray service or installer is implemented yet.

The target is a friend's **Original PCPanel**, whose four knobs also act as four independent
push buttons. Its experimental serial adapter is included; stock-hardware validation
is still pending. There was no physical PCPanel available during this implementation.

## Hardware status

| Model | Discovery / transport | Status |
| --- | --- | --- |
| Original / Maple Wood Edition | Explicit serial port, 9600 baud | Experimental, based on compatible replacement firmware; unique USB identity and stock firmware unverified |
| RGB | HID `04d8:eb52` | Implemented from public protocol references; physical test pending |
| Mini | HID `0483:a3c4` | Implemented from public protocol references; physical test pending |
| Pro | HID `0483:a3c5` | Implemented from public protocol references; physical test pending |

Do not mistake replayed fixtures or a successful build for hardware compatibility.
The Milestone 1 acceptance gate remains open until physical controls are checked.

## Friend testing / downloadable diagnostic kit

The [GitHub releases page](https://github.com/apittman019-svg/VeekPanel/releases)
provides experimental M1 diagnostic builds when published. These are console test
utilities, not the finished application or installers. For Windows, extract the
whole ZIP and double-click `START-HERE.cmd`; no Rust installation is needed. It
runs `veek-probe test-original`, compares device listings before/after connection,
asks for an explicit serial port, and collects 60 seconds of raw input. Follow the
included README and fill in `RESULTS.txt` before returning the capture folder.
The friend owns the device and will test when convenient; no physical pass is claimed.

Raw captures include unrecognized bytes. A replay/parser error is useful evidence,
not a reason to flash firmware. Nothing is uploaded automatically. Captured metadata
always says hardware validation is unverified. Builds are currently unsigned.

## Run the developer utility

Install [Rust](https://www.rust-lang.org/tools/install) and the prerequisites for
[Windows](docs/WINDOWS_SETUP.md) or [Nobara/Linux](docs/LINUX_SETUP.md).
The toolchain is pinned in `rust-toolchain.toml`; `Cargo.lock` pins dependencies.

```sh
cargo build --workspace --locked
cargo run -p veek-probe -- list
cargo run -p veek-probe -- watch
```

For the Original, identify its port by comparing `list` before/after plugging it in:

```sh
# Windows example (use your actual COM port):
cargo run -p veek-probe -- watch --serial COM3 --raw
# Nobara example (prefer the actual persistent /dev/serial/by-id/... path):
cargo run -p veek-probe -- watch --serial /dev/ttyUSB0 --raw
```

`watch` retries failed connections every three seconds, reports parser errors,
handles Ctrl+C and supports `--duration 10`. `--no-init` suppresses the HID
state-request write; Original serial never receives protocol writes. Opening some
serial boards can reset them. Unrecognized serial adapters are never auto-opened.
`list` prints device identifiers/paths for local diagnosis; review before sharing.

Collect evidence from one explicit Original serial connection:

```sh
mkdir captures
cargo run -p veek-probe -- capture --serial COM3 --output captures/trial-1 --duration 60
```

Use the actual port (Linux paths also work). The output parent must exist; the trial
directory must not exist. Raw bytes are capped at 1 MiB by default (`--max-bytes` can
set 1 byte through 16 MiB), and duration at 1–300 seconds. Capture stops on disconnect
instead of mixing sessions. `serial.bin` is directly usable with Original replay;
`chunks.tsv` records host read timings/boundaries; `metadata.txt` records stop reason.
Caps exclude the bounded hex/metadata representation, which adds roughly 2x raw size
plus a line per read. Port paths and USB serial numbers are not stored in metadata.

Offline checks (these use explicitly synthetic fixtures):

```sh
cargo run -p veek-probe -- replay --model original tests/fixtures/original.txt
cargo run -p veek-probe -- replay --model pro tests/fixtures/pro.hex
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all --check
# Linux-only synthetic serial integration:
python3 tests/serial_pty.py
python3 tests/capture_pty.py
```

Example **offline** output:

```text
KNOB_1 = 73 (raw=73/100)
KNOB_2 = 42 (raw=42/100)
KNOB_3_PRESS = TRUE
KNOB_3_PRESS = FALSE
```

These are physical positions, not OS volume readings. See [verification](docs/VERIFICATION.md)
and the [real-hardware checklist](docs/HARDWARE_VALIDATION.md).

## Project map

- [AGENTS.md](AGENTS.md): requirements, scope gates and instructions for future sessions.
- [Research and stack decision](docs/RESEARCH.md), [protocol](docs/HARDWARE_PROTOCOL.md),
  [architecture](docs/ARCHITECTURE.md), [milestones](docs/MILESTONES.md).
- `crates/veek-hardware`: model descriptors, pure parsers, transport and session boundary.
- `crates/veek-probe`: hardware diagnostics CLI; no GUI dependencies.
- `tests`: synthetic fixtures and Linux PTY integration; Rust tests live with crates.
- `packaging/linux`: narrowly scoped HID udev rules, not a finished package.

MIT for original VeekPanel code; [research provenance](docs/RESEARCH.md) records upstream
licenses. No affiliation with PCPanel; no firmware or vendor binaries are redistributed.
