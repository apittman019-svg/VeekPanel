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
