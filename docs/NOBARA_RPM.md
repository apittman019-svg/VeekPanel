# Native Nobara RPM preview

The native RPM build includes the current desktop UI (quick assignments and Doom),
launcher/icon, Rust/frontend notices and applicable source, and the scoped Mini,
Pro and RGB udev rule. Native libraries are system dependencies, not copied into
the RPM. The recipe uses RPM's ELF scanner for exact symbol/ABI dependencies plus
explicit tray/PipeWire/media/udev requirements. Preview release ordering uses the
source commit timestamp before its hash. This is a Nobara 44 x86_64 preview only.

Build from the checked-out source with the pinned Rust/Node/pnpm environment:

```sh
python3 packaging/linux/build_rpm.py /absolute/path/to/fresh-output
python3 tests/gui/rpm_smoke.py /absolute/path/to/package.rpm /absolute/path/to/fresh-evidence
```

The build performs a frozen frontend install/build, locked Rust release build,
license collection and pinned Tauri bundling. Each output has the source commit,
tracked working diff, generated dependency config, checksums and build receipt.
This reproduces the procedure, not a promise of byte-identical RPM archives.

## Current result

Source 7b8ecbfe, application source a4d0ab8, produced
VeekPanel-0.1.1-0.preview.1791605701.g7b8ecbfe.nobara44.x86_64.rpm.
SHA256: a21289ba1ebd00ca8788590a7aa09c3e3bea6e748cbc93be059d78becf21d5e3.
Output: /home/austinp/Downloads/gaem/VeekPanel-Nobara-RPM-Verified.

Passed: all declared dependencies resolve on this Nobara host; package notices,
launcher/icon and exact scoped rule are present; desktop-file validation; isolated
install/reinstall and file verification; actual packaged native GUI/private audio,
quick assignment handlers, preserved button mappings, profile edits, startup
registration, duplicate ownership, clean close/relaunch and saved settings.

The original nested `rpm --root` namespace fixture failed with SQLite Basenames
WAL/savepoint errors during reinstall and duplicate database entries on removal.
The corrected harness runs RPM directly inside the disposable filesystem with
read-only RPM/runtime mounts, no host package database, and plugins disabled.
The complete install/reinstall/native GUI/close/relaunch/uninstall sequence now
PASSES. Configuration and a separate user-owned udev rule survive removal; the
package-owned executable, launcher and rule are removed. Original failure evidence
is retained under verification/; the passing result is under
verification-direct-root/. The package bytes were unchanged between these tests.

No host package or USB permission was installed. Host dependency availability was
checked separately from the empty test RPM database (--nodeps); this does not
prove clean-machine package-manager dependency solving. Next: normal Nobara
desktop install/login/tray/physical USB acceptance, then Flatpak. Broader Linux,
native Wayland, sleep and soak remain open. Do not treat this as completed M6.
