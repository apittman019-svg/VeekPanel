# Background and startup validation

This is an M5 development increment, not completed lifecycle acceptance.

## Automated checks

- Workspace test: old schema-2 config defaults `start_minimized` to false without
  rewriting; first save backs up original bytes and persists the preference.
- Desktop tests: hide policy combinations; manual versus duplicate-login launches;
  Windows command quoting/unsafe paths/length; truthful registration readback.
- Tray monitor tests: loss reveals once, recovery does not hide, and shutdown wakes
  the idle worker and releases its owned probe. A private-D-Bus fixture exercises
  uncached host state and watcher disappearance/replacement; it compiles locally
  but execution is pending because this workspace denies Unix sockets.
- Linux temporary-directory tests: opt-in/idempotent registration and removal,
  space/special-character quoting, repair of owned changed entries, rejection of
  foreign entries and filesystem errors. They never alter the user's startup files.
- Native GUI harness additions: isolated XDG registration/readback/removal,
  start-minimized persistence, duplicate manual/login handoff without config writes.
  These new integration assertions are **pending execution**, not unit-test evidence.
- CI now runs desktop unit tests on both OSes and starts the Linux GUI harness
  in a private session bus. Windows registry delivery and real tray behavior still
  require manual acceptance.
- The actual NSIS cleanup hook passes a compile-only check with warnings as errors.
  Windows smoke assertions cover no implicit startup, a path with spaces, native
  second-launch handoff and graceful shutdown, scoped cleanup, foreign/unrelated
  registry values, and `/UPDATE` preservation. Native Windows execution is pending.

Private-bus regression (disposable session only):

```sh
dbus-run-session -- env VEEK_PRIVATE_DBUS_TEST=1 cargo test --manifest-path app/Cargo.toml --locked private_bus_tray_host_loss_and_recovery -- --ignored
```

## Manual acceptance: Windows 10/11 and Nobara

Use an explicitly opted-in test account/install; keep physical USB separate.
Record OS/desktop, source commit, install path and actual observed result.

1. Start with no VeekPanel login registration. Launch/import/switch profiles;
   verify that none registers startup. Enable from Settings, check the per-user
   entry and readback, disable and verify removal. Check an install path with spaces.
2. Enable startup and start-in-tray, then log out/in. Verify one process, restored
   profiles and functioning background audio. Verify OS startup-manager overrides
   are not mistaken for successful login delivery. Remove registration afterward.
3. With a confirmed visible tray, enable close-to-tray, close the window, verify
   background controls still work, and reopen via both tray Open and app launch.
   There must be one owner, no volume jumps, replayed button presses or config resets.
4. Start minimized without a Linux tray host, or inject tray/config initialization
   failure in a disposable test config. Verify the window becomes visible with an
   error; closing without a recovery route exits. Never corrupt a personal config.
5. Quit from the tray; verify process exit and released config lock. Relaunch and
   check saved intent. Test simultaneous launches and a second `--autostart` process.
6. Check tray-host disappearance/restart, desktop shell restart, sleep/resume,
   repeated audio-service recovery and multi-day idle/load soak. These are open
   acceptance cases; do not restart a user's desktop/audio services unattended.

Current-source Windows installers clean only their exact owned startup command on
normal uninstall. Same-path reinstall and `/UPDATE` replacement preserve opt-in;
normal uninstall-based upgrades remove it. The published 0.1.0 installer has no
new hook. Disable startup in Settings before relocation or Linux removal. Native
installer, real desktop tray-loss recovery and reboot acceptance remain open.


## Nobara isolated native results (2026-10-08)

The current handoff backend passed the private-D-Bus tray-host loss/recovery test
and the native GUI startup checks: explicit XDG entry creation/readback/removal,
start-minimized persistence and duplicate normal/`--autostart` launches returning
to the same owner. All registration lived in disposable config; personal login
state and desktop services were untouched. These results do not establish real
KDE shell recovery, actual logout/login delivery, physical background control or
Windows uninstall cleanup. Those manual acceptance gates above remain open.

## Current Windows CI hardening (2026-10-08)

Current `94ed405` native Windows core/desktop jobs passed, but its installer smoke
did not reach lifecycle cases because an absent startup value terminated the guard.
The repaired smoke now exercises actual installed-app startup enable/readback/remove
against HKCU through WebView2/native IPC. It starts with persisted start-minimized
and `--autostart`, requires hidden only when the native tray is ready, checks quiet
duplicate autostart, then manually relaunches to reveal the original owner. With
a ready tray it also closes/hides and reopens that owner before a graceful shutdown
and lock-release check. Existing reinstall/update/scoped cleanup tests remain.
These cases subsequently passed at d341f80; see VERIFICATION.md for exact results.

This proves native window/ownership behavior on hosted Windows when it passes.
It does not interact with Explorer's tray menu or prove login delivery, shell
restart, consumer Windows, real background audio/USB, sleep/reboot or soak. Failure
to create a tray is checked through the visible fallback if actually observed;
this does not inject or establish every tray/config failure mode.

### Hosted Windows result (2026-10-08)

Source `d341f80` passed installer run `37732328374`, job `113164039343`, including
real installed WebView2/native startup registration and background cases described
above, plus reinstall/update, migration and scoped uninstall. Temporary debug
policy removal passed. See VERIFICATION.md for exact evidence and artifact scope.
Consumer Windows, actual login/Explorer tray, physical background controls and
sleep/reboot/soak remain open. The same source's Linux native GUI job lost its
driver connection after profile switching; isolated Nobara results remain valid
within their scope but do not erase this recurring hosted failure.


The later e76aaac Linux evidence harness passed the full native GUI locally on
Nobara and on hosted Ubuntu (run 37807217907/job 113414524577), including private
startup registration/removal and duplicate launches. This successful run does not
explain or eliminate the intermittent hosted driver failures. Windows production
and installer code remain identical to the verified d341f80 source.
