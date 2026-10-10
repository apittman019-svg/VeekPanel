# Completion checklist

The 2026-10-09 request is to finish the app, following the original brief. This is
ongoing product work, not authorization to call the preview production-ready.
Continue focused changes on nightly; preserve existing work and do not merge main
or publish a release without superseding authorization. Physical absence does not
block software work. Windows is primary, Nobara is the first Linux target.

## Implemented foundation

- Direct Mini/RGB/Pro HID adapters; experimental explicitly selected Original serial.
- Native Core Audio/PipeWire, durable matching, system/app/device/input volume/mute.
- Independent knob/button mappings, relative groups, per-profile preferences,
  versioned migrations/backups, profile duplication and switching.
- Actual desktop dashboard/settings/diagnostics, pickup feedback, dark/light,
  accessible control selection, redacted report and configuration import/export.
- Startup opt-in, tray/background ownership and recovery; Windows installer with
  native installed-WebView/backend/migration/uninstall validation.
- Nobara-specific AppImage with source/notices companion; optional online Doom.
- Quick audio assignment and drag handlers now save through the existing backend,
  preserving the separate press action. Native gesture acceptance remains distinct
  from the automated DOM drag-event integration.

## Remaining software work, in order

1. Observe current-source Linux GUI validation after removing its unnecessary
   HTTP forwarding driver. Retain any failures; never retry an uncertain mutation.
2. Native Nobara RPM installation/desktop integration, followed by Flatpak. Keep
   the tested AppImage/source companion and actual platform limits documented.
3. Finish missing button actions: default input/output switching, media controls,
   shortcuts and explicitly enabled local programs. Define platform capabilities;
   do not show successful actions that are unavailable. Imported executable actions
   must remain disabled until explicitly trusted locally.
4. Complete device/identity UX, multiple connected panels, safe LED configuration
   only for documented protocols, scaling/update/settings workflows and safe reset.
   Foreground profile selection is a planned extension, not a fabricated feature.
5. Measure native startup, idle CPU/memory and response latency; perform sustained
   private-backend recovery/ownership tests. Complete accessibility/workflow review,
   signing/update distribution decisions and clean-machine package acceptance.

## Acceptance requiring another environment or hardware

- Friend's Mini: knob-to-audio, unplug/replug, restart, sleep/resume and real daily
  use on Windows. Preserve already accepted basic Mini input evidence.
- Physical Mini on Nobara; validate package udev access under the normal user.
- Windows 10/11 consumer unelevated install, audio, tray/login/reboot and Doom.
- Other advertised models/revisions individually; unknown firmware stays unknown.
- Real desktop-shell login/tray, native Wayland, Bluetooth/audio-device changes,
  multi-day soak and broad Linux compatibility.

Neither passing automated tests nor one package build closes these acceptance
items. Exact commits/jobs, artifact hashes and retained failures belong in
VERIFICATION.md. A usable preview and completion of the entire original brief are
different states; report them accurately.
