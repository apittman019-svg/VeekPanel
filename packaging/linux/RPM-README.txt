VeekPanel — Nobara desktop preview

Open the RPM with your package manager, review its installation request, then
launch VeekPanel from the application menu. System installation requires your
administrator password; running the app does not. Replug the PCPanel after
installation so the active desktop user's USB access can be applied.

This build targets Nobara 44 x86_64. It uses the system's WebKit, GTK, PipeWire and
tray libraries. Other distributions have not been validated. The package includes
rules only for known PCPanel Mini, Pro and RGB USB IDs, never all serial adapters.
No startup entry is enabled automatically. Enable login startup in Settings if
wanted. Before uninstalling, disable login startup in Settings and quit VeekPanel.
Uninstall removes the packaged app, launcher, icon and USB rule; it preserves your
settings in ~/.config/org.veekpanel.desktop/. Existing user-authored rules remain.

Mini basic input was tested on Windows; Linux physical controls, consumer Windows
audio, sleep/resume and long-running reliability still need acceptance. This is an
unsigned preview, not a completed production release. Doom is optional and online.

Project: https://github.com/apittman019-svg/VeekPanel
Project and dependency licenses and applicable source are under
/usr/lib/VeekPanel/THIRD_PARTY. System libraries remain managed by Nobara.
