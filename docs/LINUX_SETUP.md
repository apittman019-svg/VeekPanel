# Nobara and Linux setup — M1/M2 developer utilities

Nobara is the first Linux target. The prototype has been built and run on Nobara
44 KDE as an ordinary user. Physical PCPanel verification is still pending.
These terminal steps are for M1/M2 development; the final desktop product must guide
setup without requiring terminal commands.

## Build

Install Rust using its [official installer](https://www.rust-lang.org/tools/install).
The repository's `rust-toolchain.toml` selects the tested version automatically.
On Nobara/Fedora, the native build prerequisites are:

```sh
sudo dnf install gcc pkgconf-pkg-config systemd-devel pipewire-devel clang-devel
cargo build --workspace --locked
cargo run -p veek-probe -- list
```

Ubuntu/Mint equivalents are `build-essential pkg-config libudev-dev libpipewire-0.3-dev libclang-dev`;
Arch uses `base-devel pkgconf systemd pipewire clang`. These broader distributions
are not physically validated by this session. PipeWire development headers and
libclang are needed for the M2 workspace build. To build only M1 without them use
`cargo build -p veek-probe --locked`. WebKit/Node/Tauri are not needed yet.

M2 runtime needs native libpipewire and the ordinary user-session server. Test tools
need `pipewire`, `pw-cat`, `pw-cli`, `pw-dump` and `pw-metadata` (Ubuntu `pipewire-bin`).
The audio integration script creates its own private server and never restarts the
desktop's service. See [audio diagnostics](AUDIO_VALIDATION.md).

## Known HID models (RGB/Mini/Pro)

`list` can enumerate a device even when opening its hidraw node is denied.
Install the scoped rule once if needed:

```sh
sudo install -m 0644 packaging/linux/70-veekpanel.rules /etc/udev/rules.d/70-veekpanel.rules
sudo udevadm control --reload-rules
```

Unplug/replug the panel while logged into the local desktop session, then run
`cargo run -p veek-probe -- watch --raw`. `uaccess` grants the active local user
access through logind/udev; it is not a headless-service permission policy.
Do not use `sudo` to run VeekPanel or make all hidraw devices world-writable.
The rule deliberately precedes the usual seat/uaccess rule ordering.

## Original/Maple serial

Original is not covered by HID rules. Compare `veek-probe list` before and after
connecting to establish the actual port. Prefer a persistent `/dev/serial/by-id/`
path if the adapter provides one. Generic serial candidates are not identified
PCPanels, and the utility will not open any unless explicitly selected:

```sh
cargo run -p veek-probe -- watch --serial /dev/serial/by-id/YOUR_ACTUAL_DEVICE --raw
```

If access is denied, inspect the actual node with `ls -l` and `udevadm info`.
After positively identifying the panel, an administrator can add a **device-specific**
tty udev rule using its observed VID, PID and unique serial (if available):

```text
# TEMPLATE ONLY: replace all placeholders with observed identifiers, then review.
SUBSYSTEM=="tty", ATTRS{idVendor}=="VVVV", ATTRS{idProduct}=="PPPP", ATTRS{serial}=="ACTUAL_SERIAL", TAG+="uaccess"
```

No stock Original identifiers are yet known, so no ready-made serial rule is
installed or claimed. A generic CH340 VID/PID-only rule can authorize unrelated
devices. If the unit has no unique serial, investigate a narrower physical-port
rule or guided access policy before shipping. Existing distro serial-group access
may suffice but grants broader access than a dedicated device rule. Do not disable
SELinux; diagnose actual denial and add only a justified rule if needed.

A changing `/dev/ttyUSBn` name breaks explicit-path reconnect; use by-id if available.
The prototype retries the selected path, not every new serial adapter. A silent
port may indicate a different stock protocol; inspect raw bytes rather than sending
guessed initialization or flashing replacement firmware.

## Verification and packaging boundary

`python3 tests/serial_pty.py target/release/veek-probe` exercises the serial transport
with an explicitly synthetic pseudo-terminal, including disconnect/reconnect.
It is not a physical check. Follow `HARDWARE_VALIDATION.md` when a panel is available.

Flatpak/AppImage and RPM/DEB are later work. Host udev ACLs, Flatpak device/USB portal
behavior, serial nodes and host PipeWire policy need an actual packaging spike.
No package, privileged helper, autostart entry, service or host permission changes
are created by building/running this prototype.
