# AppImage packaging preparation and bounded native handoff

## Review decision — 2026-10-08

Reviewed exact nightly `0af5de925f0466009772c2b8a374c90be6b48482` and intervening
e76aaac/0af5de9 changes. Production app, UI, crates and Windows packaging are
byte-identical to Windows-verified d341f80. The evidence helper preserves original
failures before screenshot/cleanup and does not retry requests. No new runtime
defect is established by this review.

GitHub run 37807217907 has four passing jobs. Its Linux native job log explicitly
reports the complete functional GUI smoke passed and artifact 11563438828 uploaded
with archive digest 7299d3beada1a0d3f354a56932349f4126e4b3afe65a9b0a97652a7f0e0c6878.
The result.json contents and local Nobara pass are recorded by Astra in
VERIFICATION.md. Artifact byte download was rejected with HTTP 403 in this cloud
workspace, so this review did not independently inspect its ZIP/screenshots.

The historical failures occurred at different driver requests; no failing
preference assertion or specific production fault is demonstrated. Headless
portal/display warnings also occur in the passing job and cannot alone identify
the cause. Keep the intermittent driver issue open and retain its evidence.
An additional broad rerun or speculative graphics workaround is not justified.
Proceed with one AppImage build and package-specific validation; if it fails,
triage that original failure before another run. A pass does not close the flake.

## Prepared change

`app/appimage.conf.json` is a separate, opt-in Tauri bundle overlay. Default
desktop and Windows installer configs remain unchanged. It includes the project
license, portable-preview guide, scoped HID rule as an inert resource, and the
generated AppImage-specific notices directory. It installs no host rule, startup
entry or launcher. No Rust/frontend dependency or runtime behavior changes.

`bundleMediaFramework` stays false: VeekPanel controls native PipeWire; the UI
does not play media. This flag is for WebView media playback, not a replacement
for PipeWire library/plugin packaging. Inspect the actual AppDir's libraries and
licenses after bundling; Cargo/frontend notices alone do not inventory the native
libraries included by linuxdeploy.

The existing startup composition already prefers Tauri's observed AppImage path
over current_exe. Validate it by launching the actual AppImage, not only an
extracted executable with a fabricated APPIMAGE variable.

## Local Astra: one packaging increment

1. Fetch nightly and inspect changes since 0af5de9. Use a clean isolated worktree
   based on that commit, or adapt after reviewing newer history. Preserve the
   active `/run/media/PSSD2/VeekPanel-native-polish` worktree and the unrelated
   dirty README/Windows installer docs in `/run/media/PSSD2/VeekPanel`. No reset,
   clean, main merge, release or Flatpak work. Review/apply the supplied patch.
2. Build one Linux x86_64 AppImage with the committed Rust/frontend locks and
   pinned Tauri CLI. Initially record the actual build OS/glibc and scope it to
   observed Nobara compatibility. A Nobara build is not evidence for older Ubuntu.
   Wider portability later needs an older compatible build baseline. Do not
   upgrade locks, replace the bundler, enable production debug hooks or install
   system permissions merely to get the build passing.
3. Inspect the AppDir: executable, AppRun, desktop/icon, bundled UI, declared
   resources/notices, linked PipeWire/WebKit/GTK/tray libraries, dynamically loaded
   SPA modules where needed, native-library licenses and loader requirements.
   Record the final package's source commit/diff, SHA256, size and build log.
   Do not redistribute an incomplete native license inventory.
4. Launch the actual AppImage as an ordinary user from a path with spaces, using
   disposable config and the existing private PipeWire fixtures. Verify Dashboard,
   real private discovery/write/readback, one representative pickup/mute mapping,
   save/quit/relaunch, duplicate ownership and missing-tray visible fallback.
   Check opt-in startup creates an Exec pointing at the permanent AppImage path,
   remove it, and verify removal. Do not modify personal audio or startup.
5. Perform one ordinary Nobara file-manager launch to establish actual mount/
   loader/rendering behavior; keep hardware disabled and audio read-only unless
   using private targets. If native automation cannot drive the packaged release,
   record a focused manual observation with captures instead of adding release
   debugging hooks or substituting an unbundled debug app as package evidence.
   Run native automation sequentially: ports 4454/4455 are fixed. An extraction
   fallback may help diagnose FUSE/loader problems but is separate from ordinary
   AppImage launch acceptance.
6. Stop after this package-specific pass. Do not rerun broad unchanged Windows,
   core or 60-scenario visual matrices. If failure recurs, retain result.json,
   exact failing request, native.log, screenshot availability and helper status;
   additionally identify whether the app/driver/WebKit child exited, with version
   and crash evidence if available. Do not replay mutation requests or weaken
   assertions. Return the original failure and smallest supported fix proposal.

Hardware access, desktop-menu integration and real login are recorded as separate
pending acceptance. Without physical Mini, observe discovery/errors only and do
not claim HID success. Actual login cycles require an explicitly isolated test
account/session; in-app registration readback does not establish delivery. Broader
Nobara package validation follows this first build/run pass; Flatpak follows later.

## Build commands for the clean worktree

Run from its repository root; the notice generators require fresh destinations.
If a destination already exists, inspect and use a fresh worktree rather than
deleting unrelated output. Native/bundler prerequisites must be checked locally.

```sh
pnpm --dir ui install --frozen-lockfile
pnpm --dir ui build
cd app
python3 ../tests/manual/license_inventory.py ../appimage-notices/rust
cd ..
python3 packaging/windows/frontend_notices.py appimage-notices/frontend
git rev-parse HEAD > appimage-notices/BUILD_COMMIT.txt
git diff HEAD --binary > appimage-notices/BUILD_DIFF.patch
cd app
node ../ui/node_modules/@tauri-apps/cli/tauri.js build --config appimage.conf.json --bundles appimage -- --locked
```

The existing frontend notice generator is platform-neutral despite its directory
name; reuse it without a Windows pipeline refactor. Expected output directory:
`app/target/release/bundle/appimage`. Actual file creation/run is pending.

## Source references and verification limits

Configuration was checked against the exact
[Tauri CLI 2.12.1 schema](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.12.1/crates/tauri-cli/config.schema.json).
The [AppImage guide](https://v2.tauri.app/distribute/appimage/) explains Linux build
baseline limitations and media bundling; consulted 2026-10-08. This preparation
does not establish a build, native bundle completeness, FUSE execution, USB access,
ordinary-user login, frame pacing, consumer Windows, sleep/reboot or soak acceptance.


## First local AppImage package pass (2026-10-08)

Preparation was applied on an isolated `codex/appimage-preparation` worktree at
`/run/media/PSSD2/VeekPanel-appimage`, based on unchanged nightly 0af5de9.
Source commits: 6f73b2d (preparation), c830725695fcdebf1e228a92a364827a49594a6e
(exact upstream license supplements for dlopen2, dlopen2_derive and
libappindicator-sys). Crate revisions and license hashes are in each supplement's
PROVENANCE.txt. No production Rust/UI/default/Windows config or lock changed.

Build host: Nobara 44 KDE x86_64, glibc 2.43, Rust 1.99.0, pinned Tauri CLI 2.12.1,
GTK 3.24.52, WebKitGTK 2.52.5 and PipeWire 1.6.8. Frozen frontend install/build and
fresh Rust/frontend notice generation passed after the omitted licenses were
supplied. Locked release compilation passed. The initial bundle attempt failed
because the old SDK pkg-config search override produced two library directories
where the bundler expected one. Host development packages already existed; no
system installation or permission change was needed. Removing PKG_CONFIG_PATH
for the bundle-only invocation resolved the error without changing the bundler.

Commands used the documented recipe plus local Node/Rust PATH and shared
CARGO_TARGET_DIR=/run/media/PSSD2/VeekPanel/app/target. Compilation used the existing
SDK pkg-config path; successful bundling used native host metadata:

```sh
cd app
# Existing release binary, unchanged production source; no dependency upgrade.
unset PKG_CONFIG_PATH
node ../ui/node_modules/@tauri-apps/cli/tauri.js bundle --config appimage.conf.json --bundles appimage
```

The repeat bundle-only step warned that the bundle-type marker was already absent
after the first attempt had patched the binary. It completed successfully; no
updater behavior is claimed. The built file is local-only at
`/run/media/PSSD2/VeekPanel/app/target/release/bundle/appimage/VeekPanel_0.1.1_amd64.AppImage`.
Size: 115263992 bytes (109.92 MiB). SHA256:
`1373bf5f20038027b200da6dd2ebac1bd087b16e9763a8bff7ac246896edde3c`.
Its source stamp is c830725 with an empty build diff. Subsequent changes affect
only audit/test/documentation; this exact artifact was tested and not repacked.

### Actual package execution

Copied the executable AppImage to `/tmp/VeekPanel AppImage trial/VeekPanel.AppImage`
and launched it normally through its runtime/FUSE, not an extracted/debug binary.
The existing native harness passed against that file with disposable configuration,
private D-Bus/Xvfb/PipeWire and explicitly simulated Mini input: real Dashboard,
private audio discovery/write/readback, pickup/mic mute, mapping/profile saves,
profile isolation, duplicate owners, tray-unavailable visible fallback, startup
opt-in/readback/removal and light/dark rendering. The startup Exec assertion used
the launched AppImage path itself, not its transient .mount path. Registration was
removed; this does not test login delivery or a user's permanent installation.

Added an opt-in package relaunch check (`VEEK_APPIMAGE_RELAUNCH=1`): native
WM_DELETE_WINDOW close, released config lock, then a new actual AppImage session
with identical saved config bytes/profile and absent startup registration. The
full targeted package run passed; no mutation retries or production hooks.
Tauri's existing automation support was used without adding release features.

```sh
VEEK_APPIMAGE_RELAUNCH=1 dbus-run-session -- python3 tests/gui/native_smoke.py '/tmp/VeekPanel AppImage trial/VeekPanel.AppImage' /tmp/veek-appimage-relaunch
```

A separate KDE `kioclient exec` file-handler launch opened the real AppImage on
the current Nobara desktop through Xwayland, with private D-Bus/config and no test
automation flag. Captured the rendered Dashboard, observed PipeWire connected,
verified hardware disabled/no mappings/no startup entry, then closed the owned
window gracefully. Host audio was read-only. This validates the KDE file-handler
route, not a manually observed Dolphin double-click, native Wayland, Explorer-like
tray integration or login. Screenshot and launch result are in
`/tmp/veek-appimage-desktop`; private package results/captures are in
`/tmp/veek-appimage-relaunch` and `/tmp/veek-appimage-private`.

### Audit and distribution stop

AppDir inspection found desktop/icon/AppRun, embedded-UI executable, Rust/frontend
notices, project guide/license and inert HID rule. The RPM/build-ID audit matched
210 native ELF libraries/helpers to 139 installed packages; it records modified
bundle hashes, preserved build IDs, source RPM identities, license expressions and
available original license texts. AppRun.wrapped is an external bundler artifact,
not RPM-owned; AppImage runtime reports type2-runtime commit 8f39b89. These require
separate tool-origin/license/source review. Hyphen 2.8.8's local package has no
license text available to this audit. Native non-ELF resources and source provision
for bundled copyleft components also require review. The audit exits nonzero for
known gaps instead of certifying completeness.

`packaging/linux/native_inventory.py AppDir fresh-output-directory` reproduces the
inventory; reviewed result is `/tmp/veek-appimage-native-audit-reviewed`. Native
notices were generated AFTER the tested artifact and are NOT embedded in it.
Do not redistribute this prototype. This bounded pass stops at that concrete
packaging-review gate; no public AppImage/release asset or Flatpak was produced.

The main executable requires GLIBC_2.39; bundled native libraries include
GLIBC_2.43 requirements. Nobara-only evidence cannot support Ubuntu/older-glibc
claims. libpipewire is host-resolved rather than bundled, so the tested host's
PipeWire library/SPA installation remains a prerequisite. Full native dependency,
source/license closure and choice of an older build baseline belong to the next
packaging review before repacking/distributing. Do not disguise this as a portable
or completed Linux installer.

No broad unchanged Windows/core/visual matrix was rerun. Python compilation and
whitespace checks passed. The audit's nonzero result is intentional evidence of
incomplete distribution review. No observed GUI driver loss occurred in these
package runs, but its historical cause remains open. Preserve physical Mini,
consumer Windows, actual login/desktop-menu/USB setup, native Wayland, relocation,
sleep/reboot, resource/soak gates. Original dirty work and native-polish checkout
were preserved; main and releases remain untouched.
