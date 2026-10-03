# Project: Modern Cross-Platform PCPanel Control Software

Build a production-quality, open-source desktop application that serves as a modern replacement for the discontinued PCPanel software.

The application must support the original physical PCPanel hardware and provide a polished, reliable, plug-and-play experience on both Windows and Linux.

This is intended to be a genuinely usable daily-driver application, not a proof of concept.

## 1. Primary Goal

Create a modern desktop audio-control application that:

- Detects compatible PCPanel hardware automatically.
- Communicates directly with the original PCPanel USB hardware.
- Maps the physical knobs, sliders, buttons, and other controls to system and application audio.
- Supports per-application volume control.
- Supports system audio devices.
- Supports microphones and other input devices.
- Supports mute/unmute and configurable button actions.
- Saves configurations automatically.
- Restores mappings after reboot.
- Handles applications appearing and disappearing dynamically.
- Works reliably after sleep, resume, USB disconnects, hardware reconnects, and audio-device changes.
- Requires as little manual configuration as possible.
- Runs natively on Windows and Linux.

The experience should be:

**Install → connect PCPanel → hardware detected → assign controls → use it.**

No terminal commands should be required for normal operation.

---

# 2. Research the Existing PCPanel Hardware First

Before implementing the application, research the original PCPanel ecosystem and determine how the physical hardware communicates with the computer.

Investigate available public information including:

- Original PCPanel software behavior
- Existing PCPanel repositories or archived source code if publicly available
- Community-created PCPanel replacements
- USB VID/PID information
- HID reports
- Serial protocols
- USB packet structures
- Control identifiers
- Knob/slider value ranges
- Button events
- LED/RGB control
- Device initialization
- Different PCPanel hardware revisions/models

Do not blindly reproduce proprietary source code. Use publicly documented behavior, open-source implementations where their licenses permit it, and clean-room protocol implementation where appropriate.

Document the protocol in the repository.

Create a hardware abstraction layer so that differences between PCPanel models and future compatible hardware do not contaminate the rest of the application.

Ideally the application should be able to identify the connected PCPanel model automatically.

---

# 3. Cross-Platform Architecture

Design the project around platform-independent core logic with small platform-specific audio backends.

Suggested architecture:

PCPanel Hardware
↓
Hardware / HID Layer
↓
PCPanel Device Abstraction
↓
Event + Mapping Engine
↓
Audio Abstraction Layer
↓
Windows Audio Backend / Linux Audio Backend
↓
Desktop UI

Keep the UI, hardware protocol, configuration system, audio backend, and mapping engine separated.

Do not scatter platform-specific checks throughout the application.

---

# 4. Windows Support

Support modern Windows 10 and Windows 11.

Use appropriate native Windows audio APIs, preferably WASAPI/Core Audio APIs.

The application should be capable of controlling:

- Master output volume
- Individual playback devices
- Individual application/session volume
- Microphone/input volume
- Application mute
- Device mute
- Microphone mute

Applications should appear automatically when they create audio sessions.

Mappings should survive an application closing and reopening whenever the application can be identified reliably.

For example, if a knob is mapped to Spotify, closing and reopening Spotify should not require remapping the knob.

---

# 5. Linux Support

Linux support is a first-class requirement, not an afterthought.

Prioritize modern PipeWire systems.

Support PipeWire/WirePlumber natively where practical.

The application should work well on distributions including:

- Fedora
- Nobara
- Ubuntu
- Arch Linux
- Linux Mint
- Similar modern distributions

Support:

- Default output volume
- Individual output devices
- Application audio streams
- Microphones/input devices
- Application mute
- Device mute
- Microphone mute

Applications and audio streams must be detected dynamically.

Account for the fact that Linux applications may expose multiple streams or change node IDs between launches.

Use stable application metadata wherever possible rather than permanently associating a knob with a temporary PipeWire node ID.

---

# 6. Physical Control Mapping

Allow each physical PCPanel control to be independently configured.

A knob/slider should be able to control things such as:

- System master volume
- Specific application volume
- Specific output device
- Specific microphone/input device
- Multiple applications simultaneously
- A user-created audio group

Buttons should support actions such as:

- Mute/unmute assigned target
- Microphone mute
- Output mute
- Toggle application mute
- Switch default output device
- Switch default input device
- Play/pause
- Previous track
- Next track
- Custom keyboard shortcut
- Run a configured command/program
- Switch profile

Where hardware supports both rotating a knob and pressing it, rotation and press must be independently configurable.

---

# 7. Audio Groups

Implement virtual groups inside the application.

For example, a user could create:

**Game**
- Cyberpunk 2077
- Discord

**Media**
- Spotify
- Firefox
- VLC

A single physical control could then change the relative volume of everything assigned to that group.

The application should gracefully handle group members that are not currently running.

---

# 8. Profiles

Support multiple profiles.

Examples:

- Gaming
- Work
- Streaming
- General

Profiles should store:

- Hardware mappings
- Application mappings
- Audio groups
- Button behavior
- LED configuration
- Device preferences

Allow manual profile switching and prepare the architecture for automatic profile switching based on the foreground application.

---

# 9. UI / Visual Design

Create a sleek, modern desktop interface.

The design language should take inspiration from contemporary Apple/macOS interfaces without copying Apple assets.

Use:

- Rounded "squircle"-style cards and controls
- Generous spacing
- Smooth animations
- Soft shadows
- Clean typography
- Minimal visual clutter
- Subtle translucency where appropriate
- Modern iconography
- Clear hierarchy
- Dark and light themes

Avoid the appearance of a traditional utility program with dozens of tiny controls.

The application should feel like a modern first-party hardware control application.

The main dashboard should visually represent the connected PCPanel.

For example:

┌────────────────────────────────────────────┐
│ PCPanel                         Connected ● │
│                                            │
│   ◉          ◉          ◉          ◉       │
│  Game      Discord      Music      System  │
│   72%        45%         30%        85%    │
│                                            │
│              [ Configure ]                 │
└────────────────────────────────────────────┘

Use attractive animated volume indicators that react when a physical control moves.

---

# 10. Hardware Setup Experience

When no device is connected, show an attractive setup screen:

**Connect your PCPanel**

"Plug your PCPanel into a USB port to get started."

When detected, transition automatically to the device dashboard.

Do not require the user to manually select a serial port or USB device unless automatic detection fails.

If Linux permissions or udev rules are necessary, the installer should handle this or provide a guided setup process.

---

# 11. Drag-and-Drop Configuration

Make configuration highly visual.

Ideally the user should be able to:

1. Select a physical knob.
2. See currently available applications/devices.
3. Drag an application onto the knob.

Example:

KNOB 1

Assigned to:

[ Discord ]

Available Audio:

[ Firefox ]
[ Spotify ]
[ Steam ]
[ Game ]
[ System ]
[ Microphone ]

Changing mappings should take effect immediately.

---

# 12. Live Feedback

When the physical hardware changes volume, update the UI immediately.

When another application changes volume, update the UI.

The UI and operating-system state must remain synchronized.

Avoid polling when the underlying platform provides appropriate event/subscription APIs.

The UI should never freeze because hardware or audio enumeration is occurring.

---

# 13. Device Reconnection

USB handling must be robust.

If the PCPanel is unplugged:

- Do not crash.
- Display "Disconnected."
- Preserve the configuration.

When plugged back in:

- Automatically reconnect.
- Restore configuration.
- Resume operation without restarting the application.

Handle suspend/resume and USB reset events as well.

---

# 14. Application Identity

Create a robust application identity system.

Do not rely exclusively on process IDs because they change.

Windows mappings may use information such as:

- Executable path
- Executable name
- Audio-session metadata
- Application identifier

Linux mappings may use PipeWire properties such as:

- application.name
- application.process.binary
- application.id
- media.name
- node metadata

Create fallback matching rules when exact identification is unavailable.

Expose enough information in a diagnostics screen to troubleshoot incorrect matches.

---

# 15. Background Operation

The application should continue controlling audio while the main window is closed.

Provide a system tray icon.

Tray actions should include:

- Open PCPanel
- Current profile
- Switch profile
- Mute microphone
- Device status
- Settings
- Quit

Support launch-at-login/startup as an optional setting.

The background component should consume minimal CPU when idle.

---

# 16. Settings

Include settings for:

### General

- Start on login
- Start minimized
- Minimize to tray
- Check for updates
- Theme
- UI scaling

### Hardware

- Connected device
- Firmware/device information
- Reconnect device
- LED/RGB configuration where supported
- Input calibration if necessary

### Audio

- Preferred output device
- Preferred input device
- Application detection
- Missing-application behavior

### Profiles

- Create
- Rename
- Duplicate
- Delete
- Import
- Export

### Advanced

- Hardware diagnostics
- Raw hardware event viewer
- Audio-session/node inspector
- Log viewer
- Configuration directory
- Reset configuration

---

# 17. Configuration

Use a human-readable configuration format such as JSON or TOML.

Configuration should include a schema/version field so migrations can occur when the application evolves.

Never silently destroy an old configuration because the format changed.

Support importing/exporting configurations.

Store platform-independent mappings wherever possible so users who dual-boot Windows and Linux can potentially reuse profiles.

---

# 18. Error Handling

The program must handle gracefully:

- PCPanel disconnected
- Unsupported PCPanel revision
- Permission errors
- Audio service restarting
- PipeWire restarting
- Windows audio service changes
- Applications closing
- Audio devices disappearing
- Bluetooth devices disconnecting
- Invalid configuration
- Corrupted configuration
- Hardware reconnecting
- Sleep/resume

Errors should be presented in understandable language rather than raw stack traces.

Detailed technical errors should still be available in logs.

---

# 19. Performance

The program should be lightweight.

Targets:

- Near-zero CPU usage while idle
- Fast startup
- Low memory usage
- Immediate hardware response
- No perceptible latency when turning a knob
- No UI blocking during device/audio enumeration

Hardware events should use event-driven processing whenever possible.

---

# 20. Packaging

Produce normal installers/packages.

Windows:

- Installer executable
- Optional portable build

Linux:

Prioritize:

- Flatpak
- AppImage

Also make native packaging possible for:

- RPM
- DEB

Account for USB permissions and udev configuration during Linux installation.

The installed application should appear normally in the desktop application launcher.

---

# 21. Security

Do not require administrator/root privileges for normal operation.

Use elevated privileges only when genuinely necessary during installation/setup.

Do not expose arbitrary command execution remotely.

Validate configuration data.

Keep the hardware communication layer isolated from untrusted data where practical.

---

# 22. Testing

Create automated tests for:

- Configuration parsing
- Configuration migrations
- Application matching
- Profile switching
- Mapping behavior
- Audio groups
- Hardware event parsing

Create mock implementations of the PCPanel device and audio backend so most of the application can be tested without physical hardware.

Hardware integration tests should be separate.

---

# 23. Diagnostics

Include an advanced diagnostics screen because hardware compatibility problems will inevitably occur.

Display:

- PCPanel model
- USB VID/PID
- Firmware/version information if available
- Connection status
- Raw control events
- Audio backend
- Detected applications
- Detected playback devices
- Detected input devices
- Application identifiers
- Recent errors

Include a "Copy Diagnostic Report" button that removes unnecessary personal information before copying.

---

# 24. Technology Selection

Before coding the entire application, evaluate the best framework for this project.

Strongly consider Rust for the backend/core because this application needs:

- USB/HID communication
- Native OS audio APIs
- Low latency
- Low resource usage
- Cross-platform support
- Safe concurrency

A framework such as Tauri may be appropriate for the desktop UI, provided it does not compromise hardware/audio functionality.

Other approaches are acceptable if they produce a better architecture.

Do NOT select Electron merely because it is convenient. If Electron is proposed, justify its memory footprint and hardware/native integration compared with alternatives.

Prefer a small native backend with a polished web-based or native UI.

---

# 25. Repository Structure

Keep the repository organized approximately like:

/app
/core
/hardware
/audio
/audio/windows
/audio/linux
/config
/profiles
/ui
/tests
/docs
/packaging

The exact structure may differ if the chosen language/framework has stronger conventions.

Include:

README.md
CONTRIBUTING.md
LICENSE
docs/HARDWARE_PROTOCOL.md
docs/ARCHITECTURE.md
docs/LINUX_SETUP.md
docs/WINDOWS_SETUP.md

---

# 26. Development Strategy

Do not attempt to build everything simultaneously.

Use milestones.

### Milestone 1 — Hardware

Detect the PCPanel and reliably read its physical controls.

Create a small hardware test utility showing events such as:

KNOB_1 = 73
KNOB_2 = 42
KNOB_3_PRESS = TRUE

Verify this against real hardware.

### Milestone 2 — Audio

Implement the audio abstraction.

Verify:

PCPanel knob → master volume

Then:

PCPanel knob → application volume

Implement this separately on Windows and Linux.

### Milestone 3 — Core Mapping Engine

Build mappings, application identity, groups, profiles, configuration persistence, and reconnect behavior.

### Milestone 4 — UI

Build the polished graphical application around the working backend.

### Milestone 5 — Background Service

Add tray operation, startup behavior, reconnect handling, and long-running reliability.

### Milestone 6 — Packaging

Produce Windows and Linux releases.

### Milestone 7 — Polish

Improve animations, onboarding, diagnostics, error handling, accessibility, and performance.

Do not prioritize visual polish before hardware and audio control are demonstrably reliable.

---

# 27. Important Engineering Rule

Do not fake functionality.

Every UI element shown as functional must actually be connected to working backend behavior.

Do not populate the interface with placeholder applications or fake audio devices except when explicitly running in development/demo mode.

Do not mark a milestone complete until its underlying functionality has been tested.

When something cannot yet be implemented, clearly mark it as unfinished rather than simulating success.

---

# 28. Compatibility Goal

The end result should feel like what the original PCPanel software could have evolved into if it had continued receiving active development.

Preserve compatibility with existing PCPanel hardware while modernizing everything around it:

- Cross-platform support
- Better reliability
- Better application detection
- Better profiles
- Better configuration
- Better diagnostics
- Modern UI
- Easier installation
- Plug-and-play hardware discovery

A user who already owns PCPanel hardware should be able to install this application, plug in the device, and immediately understand how to use it.

---

# 29. First Task

Before writing the full application:

1. Research the publicly available information about PCPanel hardware and software.
2. Identify known PCPanel hardware models/revisions.
3. Determine the USB/HID/serial communication protocol.
4. Find relevant open-source implementations and record their licenses.
5. Determine the best Windows audio-control API.
6. Determine the best PipeWire integration approach for Linux.
7. Propose the application architecture.
8. Propose the technology stack.
9. Identify major technical risks.
10. Produce a milestone implementation plan.

Then implement a minimal hardware-detection prototype.

Do not begin by generating a mock UI.

The first meaningful success criterion is:

**"I plugged an original PCPanel into the computer, the new application identified it, and moving a physical control generated the correct event."**

After that is verified, proceed to audio control.