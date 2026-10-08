# Visual foundation and local validation — 2026-10-07

The user now wants Cloud 6.1 Sol for architecture, optimization, coordinated review
and deciding the next task; local Astra with Full Access for GUI/animation tuning,
native execution, hardware/platform interaction and Git operations in the home repo.
The user explicitly prioritizes an awesome appearance and accepts some rendering
cost. Keep the plain VeekPanel brand and the current backend architecture.

This pass restores the exact UTF-8 source at optimization head
`ffc6c43f58c50ed9a1ceba54db36a22c5e671d2d` from the published base and saved patch,
then builds a new visual foundation. It replaces the earlier lost, uncommitted UI
attempt. The restored scratch repository has synthetic history, not the original
Git DAG; transfer the visual patch onto the original history locally.

Changes:
- Original SVG icons, translucent sidebar, ambient gradients, layered surfaces,
  tactile physical-position dials, and distinct Pro faders.
- Dark/light/system themes, active profile treatments, per-page entrance animation,
  selection/hover/press motion and reduced-motion support.
- Separate observed physical position and actual target volume, including unknown
  input states; visible pickup/readiness warnings and independent button actions.
- Derived mixer filtering/counts and constrained long names/select widths.
- Existing native commands, snapshot polling/merge, persistence, hardware ranges
  and accessibility/navigation hooks remain connected. No dependency changes.

Checks in the cloud source workspace: frozen-lockfile frontend installation,
`pnpm --dir ui check` (zero errors/warnings), production build, four existing
snapshot tests and `git diff --check` passed. CSS: 18.44 kB / 4.88 kB gzip;
JS: 78.19 kB / 28.94 kB gzip. These are bundle sizes, not native performance results.

Rendered/native GUI verification has not run for this source. No current screenshot,
visual acceptance, Windows/Nobara build, hardware test or animation tuning is
claimed. The cloud browser download failed; do not interpret successful compilation
as proof of layout quality. This is the deliberate handoff boundary.

Astra next: import the saved backend history once, apply the visual patch, run the
native app, inspect Mini/Pro dark/light at 1200×850 and 850×650, exercise long names,
scaling, keyboard/reduced motion, then tune visuals from actual captures. Run the
existing private native GUI/PipeWire regressions; preserve personal services and
startup state. Report native/platform limits separately. GitHub writes/releases
remain deferred. Physical/reboot/soak acceptance remains open.

## Local native validation — 2026-10-08

Imported the verified development bundle on `codex/native-visual-polish`, preserving
its original history through `ffc6c43`, then applied the visual patch once. The
original profile working tree and its two unfinished installer-documentation edits
were left intact. No GitHub writes or releases were performed.

Actual Nobara 44 Tauri/WebKit rendering exposed native select overflow with long
profile names. The profile picker now clips native overflow with room for the
focus outline; its full name is available as a tooltip. Tray failure now shows a
short, actionable banner; technical details remain in Diagnostics. No Rust/backend
behavior changed in this visual increment.

The private native visual matrix passed 30 captures at each of 100% and 200%
scaling: Mini/Pro, light/dark/system, 1200×850 and 850×650 logical pixels, unknown
and 100% physical positions, missing targets, long profile/application names,
secondary pages, disconnected hardware and audio offline. Every capture had no
horizontal document overflow. Native reduced-motion preference was confirmed at
200%, with animations disabled and zero-duration transitions. Pro has five knobs,
four faders and no fader button assignment. Captures distinguish actual private
PipeWire target volume from simulated physical position. System theme resolved to
dark on this test desktop; a live OS theme change was not exercised.

The full native functional run passed real private PipeWire writes/readback,
pickup feedback, simulated Mini mic mute, keyboard ArrowRight/Home/End focus and
selection, profile duplication and isolation, cleared edit drafts, private login
registration/removal, start-minimized persistence and duplicate-launch handoff.
WebKit's unsupported driver click/resize operations use a narrow X11 fallback on
the harness-owned Xvfb display. No personal audio/startup services were changed.

Reproduce after building the UI and native app (run these sequentially; the driver
uses fixed ports 4454/4455):

```sh
VEEK_VISUAL_MATRIX=1 dbus-run-session -- python3 tests/gui/native_smoke.py /absolute/path/to/veekpanel /tmp/veek-native-final
GDK_SCALE=2 VEEK_GUI_REDUCED_MOTION=1 VEEK_VISUAL_ONLY=1 dbus-run-session -- python3 tests/gui/native_smoke.py /absolute/path/to/veekpanel /tmp/veek-native-hidpi
```

The harness requires WebKitWebDriver, tauri-driver, Xvfb, PipeWire utilities and
libX11/libXtst. Artifact directories contain screenshots and `visual-metrics.json`.
Private fixtures are test-only. Native screenshots establish layout evidence, not
physical USB behavior, measured frame pacing, subjective animation acceptance or
Windows WebView2 compatibility. M4/M7, physical/reboot/soak gates remain open.
