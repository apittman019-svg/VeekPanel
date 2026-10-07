# VeekPanel – Windows Installer Build

This directory contains scripts for cross-compiling VeekPanel for Windows and
producing an NSIS installer, all from a Linux host (Nobara or similar).

## Prerequisites

Install these on the Linux build machine:

| Tool       | Purpose                              | Install                           |
|------------|--------------------------------------|-----------------------------------|
| Rust       | Compiler toolchain                   | `rustup`                          |
| Node.js    | UI build (Vite)                      | System package or `nvm`           |
| pnpm       | JS package manager                   | `npm i -g pnpm`                   |
| Wine       | Runs NSIS under Linux                | `dnf install wine` / distro pkg   |
| Python 3   | Path translation in makensis wrapper | Pre-installed on most distros     |
| curl, unzip| Download NSIS archive                | Pre-installed on most distros     |

The build script will automatically install these Cargo tools if missing:
- `cargo-xwin` – cross-compilation for MSVC targets
- `cargo-tauri` (tauri-cli) – Tauri bundling CLI

## Building the Installer

From the repository root:

```bash
bash packaging/windows/build-installer.sh
```

The script performs these steps in order:

1. Verifies required tools (`node`, `pnpm`, `cargo`, `wine`)
2. Installs `cargo-xwin` if missing
3. Downloads NSIS 3.11 to `~/.local/share/nsis/` if missing
4. Fetches the `nsis_tauri_utils` plugin
5. Creates a `makensis` wrapper script that translates POSIX paths to Wine Z: paths
6. Builds the Svelte/Vite UI frontend (`ui/dist/`)
7. Cross-compiles the Windows binary with `cargo xwin build`
8. Bundles the installer with `cargo tauri bundle --bundles nsis`
9. Verifies the installer output and prints its path + SHA-256

## Output

On success, the NSIS installer is created at:

```
app/target/x86_64-pc-windows-msvc/release/bundle/nsis/VeekPanel_0.1.0_x64-setup.exe
```

## What the Installer Provides

- Standard Windows install/uninstall experience (NSIS)
- Supports per-user and per-machine installation (`installMode: "both"`)
- Start Menu shortcut
- Desktop shortcut (for silent/passive installs)
- Clean uninstall via Add/Remove Programs
- Handles paths with spaces correctly

## Known Limitations

- **Not code-signed:** The installer is unsigned. Windows SmartScreen may warn on
  first run. A custom signing command can be configured in `tauri.conf.json` under
  `bundler > windows > sign_command`.
- **Cross-compilation warnings:** Tauri emits warnings about cross-platform
  compilation being experimental. The resulting installer is functional.
- **LNK4099 linker warnings:** Harmless warnings about missing MSVC `.pdb` debug
  info files from the Windows SDK. These do not affect the binary.

## Windows-Native Build (Alternative)

A PowerShell script `build-installer.ps1` also exists for building directly on
a Windows host. It follows a similar flow but uses native MSVC tooling instead
of `cargo-xwin` and Wine.
