# Implementation plan and gates

The user authorized M2 after the M1 diagnostic release. M1 physical acceptance
remains pending because the Original belongs to a friend and is unavailable locally.
M3–M7 remain a continuation plan, not authorization. See AUDIO_VALIDATION.md for
implemented M2 diagnostics and the remaining platform/physical gates.

| Milestone | Work | Evidence needed to accept |
| --- | --- | --- |
| Research/setup | Protocol/source/license investigation; Rust stack; Windows/Nobara architecture; risks; repository and CI | Traceable sources, buildable scoped prototype, current AGENTS.md |
| 1 — Hardware | Model/HID allowlist; Original serial adapter; control parser; diagnostic utility; errors and reconnect; mock and physical tests | Stock Original identified and every rotation/press/release matches physical action on Windows and Nobara; endpoint ranges captured, reconnect and sleep checks documented. Other advertised models tested individually. |
| 2 — Audio | Native audio contract, Windows Core Audio, Nobara PipeWire; mock backend; endpoint/session discovery and subscriptions | Physical knob → master and app volume on both systems; mic/output mute; external updates; disappearing devices/apps and restarted service recovery. No UI required. |
| 3 — Core | Stable app identity, mappings, groups, versioned config/migrations, profile switching, durable reconnect policy | Automated config/migration/matching/group/profile tests; app relaunch/dual-boot mappings; atomic save/recovery; button/rotation independent; absent targets safe. |
| 4 — UI | Tauri/Svelte desktop, device dashboard, onboarding, drag/drop, live feedback, settings and diagnostics | Every active element wired to real backend; no fake apps/devices; keyboard/accessibility, dark/light/scaling; no I/O blocking UI; performance baselines. |
| 5 — Background | Tray, close-window behavior, opt-in startup, single instance, suspend/resume and long-running recovery | Multi-day soak, repeated USB cycles and audio service restarts; no stuck presses, leaks or root requirement; profiles restored after reboot. |
| 6 — Packaging | Windows installer/portable; Flatpak and AppImage; native RPM/DEB path; permission guidance; update/signing strategy | Clean Windows 10/11 and Nobara install → connect → assign → use without terminal. USB access under actual sandbox, normal launcher, upgrade/uninstall; broader Linux distro smoke tests. |
| 7 — Polish | Animations, guided errors, diagnostics redaction, accessibility, performance and UX refinements | Measured responsiveness/idle resources, user workflow trials, tested privacy filtering, regression/compatibility matrix. |

## Parallel pending physical work: M1

1. Read `AGENTS.md`, `docs/VERIFICATION.md` and `docs/HARDWARE_PROTOCOL.md`.
2. Obtain access to the stock Original. Compare enumeration before/after connecting;
   record VID/PID/product/interface/driver with personal identifiers redacted.
   The device belongs to the user's friend. Use the M1 downloadable diagnostic kit
   for his trial when convenient; local access must not be assumed. The bounded
   capture command and guided test are ready for evidence collection.
3. Run serial diagnostics on its explicit port, without arbitrary writes or firmware
   modifications. Verify grammar, polarity, indices, extrema and startup traffic.
4. Correct only the adapter when evidence differs. Add provenance-tagged fixtures
   and regression tests. Solve positive automatic identity without claiming every
   generic serial bridge is a PCPanel.
5. Validate Windows and Nobara unplug/replug, port renumbering, suspend/resume and
   permission recovery. Update the acceptance matrix honestly.
6. Keep M1 acceptance pending until observed. M2 software work is now authorized,
   but neither its physical acceptance nor M3 authorization follows automatically.

## Known M1 limitations to carry forward

Original auto-identification and stock protocol are unresolved; explicit port must
be supplied. HID implementations have no physical captures here. USB release numbers
are not verified firmware versions. LED formats are investigated only. Discovery
uses a 3-second scan fallback. Silent device stalls/resets, sleep/resume, duplicate
interfaces and queue overload need physical/soak validation. Tests exercise transport
errors and synthetic serial reconnect; they do not certify real hardware behavior.
