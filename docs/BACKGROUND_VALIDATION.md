# Background and startup validation

This is an M5 development increment, not completed lifecycle acceptance.

## Automated checks

- Workspace test: old schema-2 config defaults `start_minimized` to false without
  rewriting; first save backs up original bytes and persists the preference.
- Desktop tests: hide policy combinations; manual versus duplicate-login launches;
  Windows command quoting/unsafe paths/length; truthful registration readback.
- Linux temporary-directory tests: opt-in/idempotent registration and removal,
  space/special-character quoting, repair of owned changed entries, rejection of
  foreign entries and filesystem errors. They never alter the user's startup files.
- Native GUI harness additions: isolated XDG registration/readback/removal,
  start-minimized persistence, duplicate manual/login handoff without config writes.
  These new integration assertions are **pending execution**, not unit-test evidence.
- CI now runs desktop unit tests on both OSes and starts the Linux GUI harness
  in a private session bus. Windows registry delivery and real tray behavior still
  require manual acceptance.

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

Installer handling of an existing startup entry on uninstall/relocation is not yet
implemented. Disable startup in Settings before uninstalling/moving this preview.
