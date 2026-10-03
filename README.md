# VeekPanel

A Windows-first, open-source PCPanel controller project with first-class Nobara/Linux support.
**Current scope: research and Milestone 1 hardware prototype only.** No audio control,
desktop UI, profiles, tray service or installer is implemented yet.

The owner uses the **Original PCPanel**, whose four knobs also act as four independent
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

Offline checks (these use explicitly synthetic fixtures):

```sh
cargo run -p veek-probe -- replay --model original tests/fixtures/original.txt
cargo run -p veek-probe -- replay --model pro tests/fixtures/pro.hex
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all --check
# Linux-only synthetic serial integration:
python3 tests/serial_pty.py
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
