# Implementation plan and gates

The continuation request authorizes audio, mappings/configuration, background
architecture and the backend-connected GUI without waiting for hardware. See
CONTINUATION_REQUEST.md. M1 is **partially validated**: the friend's Mini (reported
1.0, corrected from Original) passed basic discovery and all four knob/button
checks on Windows 11. Lifecycle and Nobara physical acceptance remain pending.
See HARDWARE_VALIDATION.md for the reviewed 2026-10-04 capture and worksheet.

| Milestone | Work | Evidence needed to accept |
| --- | --- | --- |
| Research/setup | Protocol/source/license investigation; Rust stack; Windows/Nobara architecture; risks; repository and CI | Traceable sources, buildable scoped prototype, current AGENTS.md |
| 1 — Hardware | Model/HID allowlist; Original serial adapter; control parser; diagnostic utility; errors and reconnect; mock and physical tests | Basic controls passed on the supplied Windows 11 Mini; still require Nobara/lifecycle checks and precise revision evidence; endpoint ranges captured, reconnect and sleep checks documented. Other advertised models tested individually. |
| 2 — Audio | Native audio contract, Windows Core Audio, Nobara PipeWire; mock backend; endpoint/session discovery and subscriptions | Physical knob → master and app volume on both systems; mic/output mute; external updates; disappearing devices/apps and restarted service recovery. No UI required. |
| 3 — Core | Stable app identity, mappings, groups, versioned config/migrations, profile switching, durable reconnect policy | Automated config/migration/matching/group/profile tests; app relaunch/dual-boot mappings; atomic save/recovery; button/rotation independent; absent targets safe. |
| 4 — UI | Tauri/Svelte desktop, device dashboard, onboarding, drag/drop, live feedback, settings and diagnostics | Every active element wired to real backend; no fake apps/devices; keyboard/accessibility, dark/light/scaling; no I/O blocking UI; performance baselines. |
| 5 — Background | Tray, close-window behavior, opt-in startup, single instance, suspend/resume and long-running recovery | Multi-day soak, repeated USB cycles and audio service restarts; no stuck presses, leaks or root requirement; profiles restored after reboot. |
| 6 — Packaging | Windows installer/portable; Flatpak and AppImage; native RPM/DEB path; permission guidance; update/signing strategy | Clean Windows 10/11 and Nobara install → connect → assign → use without terminal. USB access under actual sandbox, normal launcher, upgrade/uninstall; broader Linux distro smoke tests. |
| 7 — Polish | Animations, guided errors, diagnostics redaction, accessibility, performance and UX refinements | Measured responsiveness/idle resources, user workflow trials, tested privacy filtering, regression/compatibility matrix. |

## Parallel pending physical work: M1

1. Read `AGENTS.md`, `docs/VERIFICATION.md` and `docs/HARDWARE_PROTOCOL.md`.
2. Preserve the reviewed Windows Mini evidence. Obtain the remaining lifecycle/
   Nobara observations; compare enumeration before/after connecting;
   record VID/PID/product/interface/driver with personal identifiers redacted.
   The device belongs to the user's friend. Use the M1 downloadable diagnostic kit
   for his trial when convenient; local access must not be assumed. The released Original serial launcher is inappropriate for this unit;
   use the Mini kit's `TEST-MINI.cmd` / `test-mini` for Mini HID evidence.
3. Run the Mini HID diagnostics without arbitrary writes or firmware modifications.
   Verify report framing, polarity, indices, extrema and startup traffic.
4. Correct only the adapter when evidence differs. Add provenance-tagged fixtures
   and regression tests. Solve positive automatic identity without claiming every
   generic serial bridge is a PCPanel.
5. Validate Windows and Nobara unplug/replug, port renumbering, suspend/resume and
   permission recovery. Update the acceptance matrix honestly.
6. Keep M1 acceptance pending until observed. Continue separately authorized
   software development using explicitly labeled synthetic hardware tests.

## Known M1 limitations to carry forward

Original auto-identification and stock protocol are unresolved; explicit port must
be supplied. Mini has one reviewed physical Windows 11 capture; RGB/Pro and Nobara
HID behavior remain unverified. USB release numbers
are not verified firmware versions. LED formats are investigated only. Discovery
uses a 3-second scan fallback. Silent device stalls/resets, sleep/resume, duplicate
interfaces and queue overload need physical/soak validation. Tests exercise transport
errors and synthetic serial reconnect; they do not certify real hardware behavior.


## Current software continuation

M3 has an initial implementation of durable selectors, mappings/groups, independent
buttons, profiles and atomic versioned configuration. M4 has an actual native
Tauri/Svelte preview using the same background runtime, with an explicit simulated
panel mode and real audio. Basic tray/background work has begun; login startup,
soak/reliability and production acceptance remain pending. See
[CORE_AND_DESKTOP.md](CORE_AND_DESKTOP.md) for exact implemented behavior, commands
and remaining limitations. These changes do not close the hardware/platform gates.


Windows installer work began at the user's explicit request on 2026-10-05, ahead
of further profile work. A per-user NSIS preview packages the verified desktop
foundation; this begins M6 but does not close clean-machine or production gates.
See WINDOWS_INSTALLER.md and VERIFICATION.md for build, smoke-test and download
status. Linux distribution and signing remain pending.


M3 profile continuation (2026-10-07): schema 2 gives every profile independent
groups and preferred devices, with schema 0/1 migration and full duplication.
Private native routing and native GUI checks passed locally; see VERIFICATION.md
for Windows upgrade and release evidence. This is another M3/M4 increment, not
acceptance of unimplemented actions, physical behavior or all original features.
