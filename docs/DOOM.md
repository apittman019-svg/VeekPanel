# Doom tab

User-requested optional Easter egg (2026-10-09): the original Doom shareware
episode, played through Internet Archive's published emulator embed:
https://archive.org/embed/DoomsharewareEpisode
Provider/source: https://archive.org/details/DoomsharewareEpisode

Open Doom in the sidebar and press Play Doom, then click inside the player to
start. Arrow keys move, Ctrl fires, Space opens doors and Esc opens the game menu.
Internet access is required. Stop game or leave the tab to remove the iframe and
end the session. Document-hidden/offline events also remove it. Visibility-event
delivery for native tray hiding still needs a native check; no background CPU
measurement is claimed. The provider may preserve its own game storage; VeekPanel
does not promise save-game persistence between sessions.

No player is loaded until Play is pressed. The game component has no native IPC
calls or config writes and remains accessible when the audio backend is unavailable.
It uses a sandboxed cross-origin frame; no remote capability or parent message
bridge is added. CSP permits only this embed URL in frame-src; existing parent
script/connect policies remain unchanged. No game/engine binaries, new package
dependency, Rust changes or redistributed commercial WAD are included.

Cloud checks: frontend type check (zero errors/warnings), four existing frontend
tests, production build and whitespace checks passed. The embed returned HTTP 200
and the provider publishes this exact iframe. Browser runtime installation failed
with a truncated/non-ZIP download, so gameplay/rendering/input, fullscreen, remote
IPC denial and native WebView2/WebKit compatibility are not verified here.

Native acceptance: open the rebuilt app on Windows and Nobara; verify Doom renders,
start a level and check keyboard/audio; Stop and navigate away/back, confirming no
remaining player; check offline/retry, dark/light, minimum size, 200% scaling and
reduced motion. Check hiding to tray/restore and that native audio controls and
configuration still work normally. Do not infer these results from compilation.
Provider outages/content changes may break this online Easter egg independently
of VeekPanel. Existing installer/AppImage evidence applies to the older source;
this UI/CSP change needs normal current-source build and native validation.

## Local native result (2026-10-09)

Applied the checksum-verified patch to unchanged nightly dfd3b16 in the isolated
codex/doom-tab worktree. Frozen install, frontend check (zero errors/warnings),
all four frontend tests, production frontend build and locked native debug build
passed on Nobara. No production Rust, audio adapter, lockfile or dependency changed.

The actual Tauri/WebKit app loaded the live player, entered E1M1, responded to
native keyboard movement and produced non-silent game audio. Screenshots were
visually reviewed; the private sink captured 1,963,422 frames with 1,726,891
nonzero bytes in the first passing run (/tmp/veek-doom-policy). Stop removed the
iframe. The game image during Doom's screen-melt transition is animation, not an
inferred graphics defect.

The optional VEEK_DOOM_ONLY native harness uses a private PipeWire/Pulse session
and the installed WirePlumber policy-only profile (no hardware monitors), plus
native X11 input/capture. It needs Python Xlib/Pillow; the optional tray fixture
uses existing PyGObject. These are local test prerequisites, not app dependencies.
The remote player requires live internet and is not run by default CI.

Earlier exploratory WebDriver frame/script/screenshot timeouts are retained under
/tmp/veek-doom-explore, /tmp/veek-doom-native-input, /tmp/veek-doom-native-capture
and /tmp/veek-doom-routed. The original volume-only private audio fixture lacked
browser stream routing. Native input/capture plus policy-only private routing
allowed gameplay verification without changing production security or replaying
mutations. This does not establish the historical driver's root cause.

Remote native capabilities remain absent. The pinned Tauri source injects its
IPC initialization/key only into the main frame and guards remote commands;
the iframe remains sandboxed and cross-origin. Direct adversarial remote IPC
execution was not verified by the stalled cross-frame driver. Native Windows
gameplay, fullscreen, 200% scaling, reduced motion and real tray-shell/login
acceptance remain pending. Existing installers/AppImage do not contain this tab.


Lifecycle follow-up passed against the same production source (6bfeea7): tab
exit/reopen, synthetic offline/online events, 850x650 layout, light theme, and
hide/duplicate-launch recovery under a private D-Bus tray-watcher fixture. The
player was removed on hide and configuration was restored exactly afterward.
Evidence: /tmp/veek-doom-lifecycle-final. VEEK_DOOM_LIFECYCLE_ONLY=1 skips the
already-observed gameplay when checking these paths; it does not certify gameplay.
Initial failures were incorrect test assumptions/selectors, corrected in the
harness. Real tray menus/shell/login remain untested. See VERIFICATION.md for CI.

Windows installer CI 37996407012 passed for source 6bfeea7 and supplies a new
installer artifact containing Doom. Older published releases and AppImage remain
unchanged. Windows desktop/core and Ubuntu core CI passed; hosted Linux desktop
GUI validation failed with the recurring profile-edit WebDriver disconnect.
This is recorded separately from the passing local Doom checks.
