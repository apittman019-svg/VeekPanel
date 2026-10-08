# Windows desktop preview installer

The published 0.1.0 installer packages the verified schema-1 desktop app. Current
0.1.1 source includes schema-2 profiles and the later feedback/startup work;
the newest startup cleanup increment has not been released. This is the GUI,
not the older console diagnostic kit.

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

Preview `v0.1.0-preview.1` was published from `71a1c77` after
[Windows installer CI passed](https://github.com/apittman019-svg/VeekPanel/actions/runs/37573097127).
GitHub's Windows Server 2025 runner verified the checks above; consumer Windows
10/11 and physical hardware acceptance remain separate.

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
node ../ui/node_modules/@tauri-apps/cli/tauri.js build --config installer.conf.json --bundles nsis -- --locked
```

The generated setup EXE is under `app/target/release/bundle/nsis`.
Signing, automatic updates, Linux packages and broad clean-machine/upgrade/lifecycle
acceptance remain pending. Basic Windows Mini hardware input evidence is unchanged.

References: [Tauri Windows installers](https://v2.tauri.app/distribute/windows-installer/)
and [bundle configuration](https://v2.tauri.app/reference/config/#bundleconfig),
consulted 2026-10-05. No vendor binaries or firmware are redistributed.

## Current-source startup cleanup (2026-10-07)

Installation never enables login startup. The app's explicit opt-in writes HKCU
Run value `org.veekpanel.desktop` as `"<installed path>\veekpanel.exe" --autostart`.
The supported NSIS post-uninstall hook removes that value only if it exactly
matches this installation's command. Missing, changed or other-installation
entries are preserved; no machine-wide registration is touched. Configuration
is retained. Registry removal failure is reported in installer details and exit status.

Same-path reinstall and `/UPDATE` removal/replacement preserve the existing opt-in.
A normal upgrade that uninstalls the old app invokes ordinary cleanup, so the user
must re-enable startup afterward. Disable startup before relocating the executable;
exact matching intentionally does not interpret aliases or arbitrary commands.
The published 0.1.0 installer remains unchanged.

Hook integration was checked against pinned Tauri CLI 2.12.1 / bundler 2.10.1 source.
The actual hook passes an NSIS compile-only check with warnings as errors. Updated
Windows smoke tests cover scoped registry cleanup, preservation and native
single-instance/shutdown behavior. Those new Windows checks are pending CI;
compiler success is not Windows installer acceptance.

## Current nightly hosted validation (2026-10-08)

The latest exact-source evidence and failures are in VERIFICATION.md. Current
Windows core/desktop compilation and installer construction are separate from
installed-app acceptance; neither older preview success nor a new EXE closes it.

The installer workflow now parses smoke scripts with native PowerShell before
building. On a disposable hosted Windows user, its installed-app smoke uses
WebView2's process-local debugging environment variable, or a guarded temporary
per-executable HKLM debugging policy when the hosted runner is elevated, and an
ephemeral loopback CDP port. It requires the actual bundled Dashboard and native schema-2
state, and invokes the existing startup/save commands rather than substituting a
mock backend. It checks opt-in enable/detect/remove against HKCU, config ownership,
quiet duplicate autostart, manual recovery from a background start, and close/reopen
when the app reports a ready tray. It restores browser arguments and removes its
owned temporary policy afterward; no debugging capability is embedded in the release.

Fresh install in a path with spaces, same-version reinstall, `/UPDATE` replacement,
published 0.1.0 upgrade with a synthetic legacy config, exact migration backup,
relaunch preservation and scoped uninstall remain required. These smoke scripts
are CI-only and refuse existing configuration/startup entries; do not run against
a personal install. Audio services are enabled only on the disposable runner, and
hardware stays disabled throughout. No audio-write/physical claim follows.

`VeekPanel-Windows-validation` retains the transcript and source/run/OS/result JSON
for 30 days, including failures after smoke starts. Installer/hash upload still
requires the entire smoke to pass. A timeout or pre-smoke build failure must be
read from the job log; artifact presence alone is not acceptance. No release is
automatically published.

Exact source `d341f80` passed all installed-app assertions in
[run 37732328374](https://github.com/apittman019-svg/VeekPanel/actions/runs/37732328374),
job `113164039343`, including temporary policy removal, upgrade/migration and
uninstall preservation. The verified 0.1.1 installer is available as a CI artifact;
the public release remains 0.1.0. See VERIFICATION.md for artifact IDs/digests and
scope. Consumer Windows, missing-WebView2 bootstrap, physical hardware/audio,
actual login/Explorer tray and soak remain separate acceptance gates.
