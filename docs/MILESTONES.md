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
panel mode and real audio. Tray/background work now includes opt-in login registration,
start-in-tray and single-instance handoff; actual login delivery, tray lifecycle,
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


The 2026-10-07 M4 feedback increment exposes actual pickup and button readiness
from the mapping engine, including external-volume rearming, keyboard control
selection and an accessible assignment form/live status. This is incremental
assignment accessibility, not full assistive-technology or M4 acceptance.

The 2026-10-07 M5 software increment implements machine-local per-user startup
registration, conservative hide/recovery policy, duplicate-launch handoff and
explicit background shutdown. Unit and build checks passed in the remote Linux
workspace; new native GUI/Windows CI and manual reboot/tray/soak checks remain
pending. No physical acceptance or daily-driver certification is claimed.

The next 2026-10-07 M5/M6 increment adds ongoing Linux tray-host loss/recovery
handling and scoped Windows startup cleanup on uninstall. Desktop unit/build/lint
and hook compilation passed. Private-D-Bus/native Windows/desktop-shell trials
remain pending; these software changes do not close M5/M6 acceptance.

The 2026-10-07 first M7 optimization pass reduces redundant audio reads and unchanged
desktop traffic, shares immutable publications, and wakes follow-up UI requests.
See PERFORMANCE.md for synthetic before/after measurements. Native Windows/Nobara
CPU/memory, startup, physical latency and visual/accessibility acceptance remain open.

The following visual foundation adds layered themes, original SVG icons, tactile
physical-position dials, Pro faders and reduced-motion-aware transitions. Frontend
checks/build passed; rendered/native acceptance remains pending. Local Astra takes
over for visual iteration and native validation; Cloud Sol resumes for design,
architecture and optimization review. See UI_POLISH.md. No M4/M7 gate is closed.


2026-10-08 local handoff: native Nobara visual matrices, keyboard selection,
reduced motion/200% scaling, private PipeWire GUI/runtime integration and isolated
startup/tray tests passed. See VERIFICATION.md and UI_POLISH.md for exact scope.
This advances M4/M5/M7 evidence without closing Windows, physical, reboot/soak or
measured-performance gates. GitHub publishing remains deferred.

2026-10-08 release hardening supersedes that publication deferral only for focused
`nightly` commits and CI. Current `94ed405` Windows core/desktop checks passed and
NSIS built; installer smoke was blocked by an absent-registry-value read. The
expanded installed WebView2/startup/background/migration/uninstall smoke is pending
native execution. See the newest VERIFICATION.md entry. M5/M6 remain unaccepted;
consumer Windows, actual login/tray/audio, physical and soak checks remain open.
Next distribution priority after Windows evidence: AppImage, native Nobara package
acceptance, then Flatpak. Local Astra owns real package build/run/GUI/HID loops.


2026-10-08 Windows release-hardening gate: source d341f80 passed native hosted
installer run 37732328374/job 113164039343, covering bundled WebView2/backend
initialization, startup opt-in/readback/removal, owner handoff/background recovery,
fresh install, reinstall/update, schema migration/backup and scoped uninstall.
This substantially improves M5/M6 evidence without closing consumer Windows,
actual login/tray-shell, physical audio/HID, sleep/reboot or soak acceptance.
The Linux GUI driver connection failed again in run 37732328390/job 113164039793;
the cause is open. Retain original failure/process/log evidence before deciding a
fix. No production/UI/audio changes, main merge or release belong to this pass.
Next distribution order remains AppImage, actual Nobara package testing, Flatpak.
