# Windows desktop preview installer

The installer packages the verified schema-1 desktop app. Profile-specific
schema-2 work was left separate when the user prioritized an installable build.
This is the GUI, not the older console diagnostic kit.

Download the setup EXE from the [Windows preview release](https://github.com/apittman019-svg/VeekPanel/releases/tag/v0.1.0-preview.1).
Run it, finish setup, and open VeekPanel from Start. Close other PCPanel software,
connect the Mini, click **Detect my Mini**, choose rotation/press assignments and
save. Move a knob through the current volume to pick up control.

Windows 10/11 x64; per-user installation. No terminal, Rust, Node or development
tools required on the destination PC. Missing WebView2 is downloaded automatically,
so an internet connection may be needed during setup. This preview is unsigned;
Windows may display an unknown-publisher/SmartScreen warning. Use the repository's
release asset and SHA256SUMS rather than copies from elsewhere.

Uninstall through Windows Settings > Apps. Configuration is retained at
`%APPDATA%\org.veekpanel.desktop` for reinstall. Close the app before upgrading.
The software starts with hardware disabled and no mappings; installation itself
does not change audio. Development panel mode is explicit and controls real audio.

## Reproduce and verify

The `Windows desktop installer` workflow uses the committed Rust/frontend locks,
pinned Tauri CLI, static MSVC runtime, NSIS current-user mode and the separate
`app/installer.conf.json` bundle configuration. Normal CLI/desktop builds keep
working without generated packaging resources. Runtime Rust dependency notices
and original MPL source (where applicable), bundled frontend licenses, project
license, getting-started guide and source commit are included in the installed app.

The workflow builds on Windows, then tests silent install, installed-file/notices
presence, a responsive native window, configuration creation, reinstall and
uninstall with configuration preserved. These are automated Windows runner checks,
not proof of physical Mini audio actions or clean consumer-machine acceptance.
Check docs/VERIFICATION.md for the actual run results and release hashes.

Local Windows reproduction after Rust/Node/pnpm prerequisites:

```powershell
pnpm --dir ui install --frozen-lockfile
pnpm --dir ui build
cd app
python ../tests/manual/license_inventory.py ../desktop-notices/rust
cd ..
python packaging/windows/frontend_notices.py desktop-notices/frontend
cd app
$env:RUSTFLAGS = '-C target-feature=+crt-static'
../ui/node_modules/.bin/tauri build --config installer.conf.json --bundles nsis -- --locked
```

The generated setup EXE is under `app/target/release/bundle/nsis`.
Signing, automatic updates, Linux packages and broad clean-machine/upgrade/lifecycle
acceptance remain pending. Basic Windows Mini hardware input evidence is unchanged.

References: [Tauri Windows installers](https://v2.tauri.app/distribute/windows-installer/)
and [bundle configuration](https://v2.tauri.app/reference/config/#bundleconfig),
consulted 2026-10-05. No vendor binaries or firmware are redistributed.
