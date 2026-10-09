# Mapping, persistence and desktop preview

This is the first working M3/M4 implementation, not a completed daily-driver or
M5–M7 acceptance. The friend's basic Mini inputs passed on Windows 11; physical
Nobara/lifecycle and knob-to-audio trials remain separate.

## Build and run

Use pinned Rust 1.99, Node 24, and pnpm 11.25.0. The core Rust workspace and desktop
application have separate committed Cargo.lock files; ui/pnpm-lock.yaml pins the
frontend. Linux needs the existing PipeWire/libudev development packages plus GTK3,
WebKitGTK 4.1 and an AppIndicator library. Ubuntu package names are in desktop CI;
Nobara uses webkit2gtk4.1-devel and libappindicator-gtk3-devel. Windows requires the
MSVC build tools/SDK and WebView2 runtime. No administrator privileges at runtime.

```sh
pnpm --dir ui install --frozen-lockfile
pnpm --dir ui check
pnpm --dir ui build
cargo build --manifest-path app/Cargo.toml --locked
# Linux:
./app/target/debug/veekpanel
# Windows: app\target\debug\veekpanel.exe
```

The native window uses bundled local frontend assets and narrow Tauri commands.
It does not host a remote-control HTTP service. Native audio objects remain on a
background owner thread; IPC mutations use a blocking-task pool, not the UI thread.
The background state and engine readiness refresh at 250 ms, UI at 400 ms while visible. Hardware reads
have 100 ms timeouts; native event waits are bounded to 20 ms. These are scheduling
bounds, not measured input-latency guarantees for blocking OS APIs.

Runtime publications are immutable and versioned separately from configuration
edits. Unchanged observations retain the same publication; desktop polling omits
unchanged runtime/mapping payloads and preserves the UI's object references.
Tray/startup status is still checked each visible refresh. Successful writes reuse
their complete native readback for feedback instead of immediately enumerating
again. Follow-up UI requests wake the owner after native events are dispatched;
idle waits keep their existing 20 ms limit. See [performance evidence](PERFORMANCE.md).

## Use the preview

- Dashboard lists real outputs, inputs and active application/recording streams.
  Its sliders/mute buttons change actual native audio and require readback.
- Mini/RGB/Pro can use automatic model/VID/PID HID discovery. More than one matching
  interface is an explicit ambiguity; an exact HID path can be configured in
  settings. Original serial always needs its explicit port. No generic serial scan.
- Without a panel, explicitly choose Development panel. Its simulated controls
  send the same ControlEvent type into the engine; the audio backend remains real.
  The simulated sliders send an event when released. The first position never writes;
  move again across the current audio volume to pick up control. A simulated press
  emits release, press, release, so it cannot remain held after a mouse cancellation.
- Select a knob with Tab/Enter or left/right arrows, Home and End, choose its
  rotation target and independent button action, and save through the assignment
  form. Selection, target labels and live readiness have accessible descriptions.
- Each assigned rotation reports awaiting movement, pickup direction/level, or
  active control from the engine. Buttons independently report release-required
  or ready state. Missing/ambiguous/unsupported audio and partial-group warnings
  remain visible; external volume changes rearm pickup before the next movement.
  Feedback is published after native readback; an unconfirmed write cannot report
  successful control. A selected-control live region announces readiness changes.
- Groups cover several targets; relative groups preserve observed volume ratios.
  Missing members are skipped with diagnostics. Group/device ambiguity fails closed.
- Profiles support create, duplicate, rename, delete and switch. Each profile owns
  its mappings, audio groups and preferred input/output. Duplication copies all of
  these independently; a new profile starts empty and follows system defaults.
  Buttons can toggle mute or switch/cycle profiles. Schema imports also support
  explicit set-mute. Switching profiles clears unsaved assignment/group drafts.
- Settings cover theme, connection/model, preferred input/output, close-to-tray,
  start-in-tray, machine-local login registration and
  JSON configuration import/export. Imported settings apply after validation.
- The initial tray supports Open, Next profile and Quit. Closing hides the window
  only when close-to-tray is selected and tray creation succeeded (Linux also
  requires an observed StatusNotifier host). Actual desktop
  tray visibility, sleep/resume and Windows tray behavior still need manual trials.
- Diagnostics show local observed identities/errors. Copy redacted report deliberately
  excludes names, paths, USB serials and raw error text. A configuration export can
  contain paths; it is not a redacted diagnostic report.

## Configuration and identities

The app stores config.json under Tauri's per-user app_config_dir for
org.veekpanel.desktop (Linux: $XDG_CONFIG_HOME/org.veekpanel.desktop, or
~/.config/org.veekpanel.desktop). File ownership is guarded by an advisory lock;
a second owner fails visibly rather than competing over saves. Malformed/newer
configurations are not replaced. A temporary file is synced and atomically persisted;
config.json.bak preserves the preceding version. External file edits cause a
conflict error; UI revisions prevent stale editors overwriting newer configuration.

Schema 2 stores mappings, groups and device preferences inside each profile.
Hardware connection and appearance/tray settings remain application-wide. A group
ID resolves only within the active profile; identical IDs in duplicated profiles
can have different members. Preferred-device selectors use the active profile
without changing the OS default device.

Schema 1 (the 0.1.0 desktop preview) and schema 0 migrate automatically: shared
groups/preferences are copied into every profile to preserve existing behavior.
The original bytes are backed up to config.json.bak before migration; subsequent
saves use that file for the preceding configuration as usual. Invalid or newer
files, including their existing backup, are left unchanged. Schema 0 allows omitted
hardware/preferences/settings and inserts their defaults; schema 1 still requires
those fields. Schema 0 was a documented prerelease format, not a public application.
Older app versions cannot read schema 2; downgrading requires a schema-1 backup. Unknown fields/actions are rejected. Imports
cannot contain executable commands because that action type is not implemented.

Application matching uses exact application.id, application.path or process.binary;
devices use endpoint.id or node.name. The strongest configured key present on a
target wins; a mismatching strong key cannot be bypassed by a weak binary fallback.
Multiple matching app streams are intentional; multiple device matches are an
ambiguity. Never persist a PID or live node/session ID. Cross-platform alternative
identities can be authored/imported, but real dual-boot matching is not yet verified.
Preferences name an exact device and wait if it is absent; they do not silently
fall back. System-default selectors intentionally follow native default changes.

Every input carries its model's raw range. Pickup/button state resets after config,
profile, matching-target set, connection generation or backend failure changes.
No queued audio writes are replayed after service recovery. A new hardware reader
owns a new parser; old reader queues are dropped on hardware setting changes.
Input is bounded to 256 messages, coalesces analog runs without crossing buttons,
and discards input at a connection boundary or after a 250 ms backlog, rearming
controls. Native writes can partially affect a group before a later member fails;
the error is shown, remaining writes stop, and pickup is rearmed. No transactional
multi-device promise is made. Relative ratios survive an in-session zero-volume
position; after process restart with every target at zero the fallback is equal.

## Test surfaces and limits

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --workspace --release --locked
python3 tests/audio/runtime_integration.py target/release/veek-runtime
cargo fmt --manifest-path app/Cargo.toml --check
cargo clippy --manifest-path app/Cargo.toml --all-targets --locked -- -D warnings
# Native Linux UI automation needs WebKitWebDriver and Xvfb:
cargo test --manifest-path app/Cargo.toml --locked
dbus-run-session -- python3 tests/gui/native_smoke.py app/target/debug/veekpanel /tmp/veek-ui-evidence
```

The runtime integration and GUI test launch a private PipeWire server and temporary
config. Synthetic panel input changes only private synthetic audio endpoints.
The GUI test drives the actual native window/IPC and saves screenshots, not a mocked
browser frontend. The local JSON-lines veek-runtime --config PATH harness exists
for these tests; it exposes no network listener and is not the end-user interface.

Remaining product work includes multiple panels, richer identity editing, foreground
profiles, default-device switching/media/shortcut/opt-in command actions, LEDs,
login/reboot acceptance, automatic updates, Linux installers, signed distribution, accessibility/
scaling review, low-latency and idle baselines, long-run reliability and clean Windows
interaction. Only implemented controls appear in the preview. Do not mark M3/M4 or
M5 complete based solely on this initial feature set and synthetic integration.

API references: [Tauri commands](https://v2.tauri.app/develop/calling-rust/),
[Tauri native WebDriver tests](https://v2.tauri.app/develop/tests/webdriver/),
[Svelte](https://svelte.dev/docs/svelte/overview). Consulted 2026-10-04.

## Startup and background increment (2026-10-07)

Login startup defaults off. Only the explicit Settings action writes OS registration:
Windows uses the current user's Run key, value `org.veekpanel.desktop`, with a quoted
executable plus `--autostart`; Linux atomically writes
`$XDG_CONFIG_HOME/autostart/org.veekpanel.desktop.desktop` (or the per-user default).
Linux AppImage registration points to the AppImage, not its temporary mount.
There are no shell commands, machine-wide registry changes or elevated runtime.
Paths that cannot be represented safely fail visibly. Windows Run commands longer
than 260 UTF-16 units are rejected; Linux paths containing `%` or `=` are rejected.
Registration changes are serialized, run off the UI thread and require readback.
The UI reports **registered**, not guaranteed delivery: OS settings/policy can override it.
Changed entries require an explicit repair/removal; foreign Linux entries are not overwritten.

The global `settings.start_minimized` defaults false for existing schema-2 files,
which are not rewritten on load. A save/import may include this optional field;
pre-increment schema-2 builds reject it as unknown. Keep a compatible backup before
downgrading. OS login registration is never exported, imported or automatically repaired.

The main window starts unshown until initialization decides whether it should appear.
Configuration failures or missing tray recovery show it. Linux checks the desktop's
StatusNotifier host off-thread before allowing hiding. An uncached probe repeats
after a three-second idle interval, with a two-second D-Bus method timeout.
Legacy-only/unconfirmed trays leave the window visible and disable close-to-tray.
Observed host loss reveals the window once; host recovery enables tray hiding again
without hiding the window automatically. This check is not proof that the icon is
visibly rendered, and the method timeout does not bound every native connection step.
Shutdown wakes and joins the monitor before releasing the background runtime.

The first Tauri plugin enforces a single instance. A manual second launch reveals,
unminimizes and focuses the existing window; a duplicate `--autostart` launch stays
quiet. Linux requires a desktop session D-Bus; isolated tests use `dbus-run-session`.
The configuration lock remains an independent guard if handoff fails. Quit explicitly
joins the background owner before process exit, releasing config/hardware/audio.

Current-source Windows installers remove only the exact startup command belonging
to that installation on ordinary uninstall. Same-path reinstall and `/UPDATE`
replacement preserve the opt-in; an upgrade that ordinarily uninstalls the old
version clears its entry and requires explicit re-enabling. Other paths/changed
commands are preserved. This hook is not present in the published 0.1.0 installer.
Use Settings to disable startup before relocating a build or removing a Linux
preview. Native installer checks, actual login delivery, desktop shell recovery,
consumer Windows and Nobara lifecycle/soak remain pending.
See [background validation](BACKGROUND_VALIDATION.md) and [local handoff](LOCAL_HANDOFF.md).

Protocol references consulted 2026-10-07:
[Tauri single instance](https://v2.tauri.app/plugin/single-instance/),
[Windows Run keys](https://learn.microsoft.com/en-us/windows/win32/setupapi/run-and-runonce-registry-keys),
[XDG Exec quoting](https://specifications.freedesktop.org/desktop-entry/latest/exec-variables.html),
[KDE tray interface](https://github.com/KDE/plasma-workspace/blob/master/xembed-sni-proxy/org.kde.StatusNotifierWatcher.xml).


Linux GUI automation now calls WebKitWebDriver directly with its native browser
capability and Tauri's existing automation environment setting. This removes
tauri-driver's failing HTTP forwarding hop; no production debug feature is added.
The same native GUI/audio/profile/startup assertions and no-mutation-retry policy
remain. A local full pass is not proof of the historical reset's root cause.
