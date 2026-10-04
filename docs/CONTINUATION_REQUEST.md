Resume development of VeekPanel from the current repository state.

Repository:
https://github.com/apittman019-svg/VeekPanel

First re-read AGENTS.md, README.md, the docs directory, git history, current source code, and working tree so you understand everything completed so far.

IMPORTANT CHANGE OF PLAN:

I do not currently have the physical PCPanel hardware available for testing.

Do NOT wait for physical hardware validation before continuing development.

Treat the remaining physical-validation portion of Milestone 1 as BLOCKED/PENDING rather than attempting to guess hardware behavior.

Clearly document which hardware behaviors remain unverified, preserve the existing hardware abstraction and `veek-probe` tools, and continue development using mocks/synthetic hardware where appropriate.

Do not claim unverified PCPanel behavior is confirmed.

## Proceed to Milestone 2

Begin implementing the actual audio subsystem.

My primary development environment is Nobara Linux, but the final application must support both Linux and Windows.

Implement the audio system behind a clean cross-platform abstraction so Linux-specific behavior does not leak throughout the rest of the application.

### Linux backend

Prioritize PipeWire/WirePlumber.

Implement real functionality for:

- System/master output volume
- Output device discovery
- Input/microphone device discovery
- Per-application audio stream discovery
- Per-application volume
- Output mute
- Application mute
- Microphone/input mute
- Dynamic detection of applications starting/stopping audio
- Dynamic detection of audio devices connecting/disconnecting
- Default output/input tracking
- Stable application identity where possible

Test this against the REAL PipeWire environment on this Nobara machine.

Do not fake audio devices or application sessions except in explicit tests/mock mode.

Create useful diagnostic tooling so the audio backend can be tested without the PCPanel attached.

For example, I should be able to use a CLI/debug tool to:

- list detected outputs
- list detected inputs
- list active applications/audio streams
- inspect application identity metadata
- read current volume
- set volume
- mute/unmute

Actually run these tools against this machine and verify the backend works.

## Windows architecture

Keep Windows support as a first-class architectural requirement.

Create the abstraction necessary for a future Windows Core Audio/WASAPI implementation, but do not let lack of a Windows test environment block Linux development.

Do not pretend Windows functionality has been tested if it has not.

If implementing the Windows backend can be done cleanly without interfering with Linux development, proceed when appropriate.

## Mapping Engine

Once the Linux audio backend is working, begin implementing the platform-independent mapping engine.

It should support mappings such as:

Physical Control
→ System Volume

Physical Control
→ Application Volume

Physical Control
→ Audio Device

Physical Control
→ Microphone

Physical Control
→ Audio Group

Button
→ Mute/Unmute

Button
→ Microphone Mute

Button
→ Profile/Action

Because the physical PCPanel is unavailable, use the existing mock/synthetic hardware layer to drive these tests.

The mapping engine must not care whether events came from real hardware or the mock implementation.

## Configuration

Implement persistent configuration with schema/version support.

Support:

- Hardware mappings
- Application mappings
- Audio groups
- Profiles
- Device preferences
- Button actions

Add configuration migration infrastructure now so future versions can change the schema safely.

## Background architecture

Begin preparing the application to operate continuously in the background.

It should eventually:

- detect/reconnect PCPanel hardware
- monitor audio sessions
- maintain mappings
- survive audio-service changes
- survive hardware disconnect/reconnect
- run independently of whether the main GUI window is visible

Keep resource usage low and prefer event-driven APIs.

## GUI

Once the Linux audio backend, mapping engine, and configuration system are functional, BEGIN building the actual desktop GUI.

Do not wait for physical PCPanel validation to start the GUI.

Use the original design requirements:

- sleek modern appearance
- Apple-inspired squircle geometry
- rounded cards
- generous spacing
- subtle animations
- dark/light themes
- clean typography
- modern first-party hardware-control-app feel
- visually represented PCPanel controls
- live volume indicators
- drag/drop or similarly intuitive control assignment
- connected/disconnected states
- application/device selection
- profiles
- settings
- diagnostics

However, do NOT create a fake frontend disconnected from the backend.

The GUI should consume the actual audio/mapping/configuration APIs being built.

When physical hardware is unavailable, provide a clearly labeled development/mock PCPanel mode so the complete application flow can be exercised.

## Development strategy

Continue autonomously through everything that does NOT require the physical PCPanel.

Do not repeatedly stop simply because hardware validation remains pending.

Only stop for me when:

- information genuinely cannot be determined without my input,
- an operation requires a decision with significant architectural consequences,
- elevated permissions are required,
- or continuing could risk damaging user data/system configuration.

Otherwise continue implementing, testing, fixing, documenting, and committing sensible milestones.

Run formatting, linting, tests, and builds regularly.

Use the actual Nobara/PipeWire environment for integration testing wherever possible.

Maintain a clear TODO/blocker documenting:

**Physical PCPanel hardware validation still required before claiming full hardware compatibility.**

When the hardware becomes available later, we will return to that validation and correct any protocol differences discovered.

For now, maximize useful development progress without it.