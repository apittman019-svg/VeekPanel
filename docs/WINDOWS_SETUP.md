# Windows setup — primary platform

Target Windows 10 and 11. M1/M2 are developer console utilities, not an installer or
finished desktop application. GitHub CI builds/tests on a Windows runner; actual
Windows 10/11 hardware compatibility is a separate acceptance gate.

For friend testing without Rust, use the experimental Windows diagnostic ZIP from
the repository releases page. Extract it fully, read README.txt, then double-click
START-HERE.cmd. This launches the guided `test-original` command. Its raw capture
and observation checklist prepare a physical trial; no support is presumed. The
kit includes dependency notices and unmodified MPL dependency source. It is unsigned.

## Build and run

Install [Rust](https://www.rust-lang.org/tools/install) with the MSVC toolchain and
Visual Studio Build Tools' Desktop development with C++ workload/Windows SDK.
The repository pins the Rust version. WebView2/Node/Tauri are not needed for M1/M2.

From PowerShell in the repository:

```powershell
cargo build --workspace --release --locked
.\target\release\veek-probe.exe list
.\target\release\veek-probe.exe watch --raw
```

Close official/community PCPanel software first to avoid competing reads/writes.
Known RGB/Mini/Pro IDs use Windows HID; do not replace their HID driver with WinUSB
or libusb. No administrator runtime is required. `--no-init` disables the HID
state-request write for diagnostics. Ctrl+C stops; `--duration 10` bounds a test.

## Original/Maple (the friend's hardware)

The Original is experimental serial support, not one of the HID models. Use Device
Manager and `list` before/after plugging in to identify its actual COM port:

```powershell
.\target\release\veek-probe.exe watch --serial COM3 --raw
```

Replace `COM3` with the observed port. Some legacy units may need a USB serial
bridge driver. Verify the hardware ID first and use a trusted signed manufacturer/
Windows Update source; this project does not bundle a guessed driver. A generic
bridge ID does not prove PCPanel identity. Opening serial can reset some boards.

The adapter uses 9600 8N1 with no flow control and no host protocol writes. Its
grammar comes from compatible replacement firmware, not a validated stock capture.
Opening a port prints `OPENED_UNVERIFIED`. Valid control and heartbeat traffic
prints `PROTOCOL_MATCH`, which still is not proof of device identity.

Move all four knobs and push/release each separately. Record exact raw events,
indices, range endpoints and physical order with `HARDWARE_VALIDATION.md`.
Reconnect retries the same COM path. If Windows assigns another port, select it
again; durable Original auto-identification is pending hardware evidence.

## Checks

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
.\target\release\veek-probe.exe replay --model original tests/fixtures/original.txt
```

Replay is explicitly offline. The Linux pseudo-terminal test is not applicable on
Windows. No audio changes, tray operation, startup registration or installer are
part of M1. Follow the milestone plan before adding any of them.

## M2 native audio

`veek-audio-probe.exe list` enumerates active endpoints and live sessions using
Core Audio. `watch --duration 30` observes native changes. Explicit set/bind commands
change audio; see [audio diagnostics and manual acceptance](AUDIO_VALIDATION.md).
The M2 kit's LIST-AUDIO.cmd is read-only. It is separate from M1's START-HERE.cmd.
CI uses static CRT linkage for portable experimental binaries; they are unsigned.
