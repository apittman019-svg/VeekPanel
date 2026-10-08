VeekPanel Linux AppImage preview

This is an experimental desktop preview. Use only the package's recorded source,
checksum and validation results to determine which systems have been tested.
Building an AppImage does not establish hardware or distribution compatibility.

Keep the AppImage in a permanent folder. In your file manager, allow it to run as
an executable, then open it as your ordinary desktop user. A working user-session
PipeWire service and desktop session are required. Do not run VeekPanel as root.

Start with hardware disabled. Close other PCPanel controller software before
detecting your Mini and assigning controls. Move through the current target
volume to pick up control. Development panel mode is explicit and controls real
audio; it is not a silent demo.

This portable preview does not install launcher entries or host USB permissions.
If hardware access is denied, ask an administrator to review/install the included
70-veekpanel.rules for known HID models. A rule inside the AppImage does not grant
host access. Do not make all USB devices writable or disable SELinux. Original
serial hardware requires separate, positively identified device-specific access.

Login startup is optional and enabled only in Settings. Leave the AppImage at the
same path while registered. Disable startup before moving or deleting it; repair
registration explicitly after relocation. Configuration stays in the normal
per-user org.veekpanel.desktop config folder. Removing the AppImage retains it.

This preview is not a complete Linux installer. Menu integration, USB setup,
actual login, physical hardware and broader Linux compatibility need separate
validation. Source: https://github.com/apittman019-svg/VeekPanel
