Resume development of VeekPanel from the CURRENT repository state.

Repository:
https://github.com/apittman019-svg/VeekPanel

Before making changes, re-read:

- AGENTS.md
- README.md
- docs/MILESTONES.md
- docs/CONTINUATION_REQUEST.md
- docs/AUDIO_VALIDATION.md
- docs/ARCHITECTURE.md
- recent git history
- current source tree
- current working tree

Do not restart work that is already complete.

The repository is the source of truth.

## Current direction

Continue to treat remaining physical PCPanel lifecycle/Nobara validation as a parallel pending task.

Do NOT block normal software development on that remaining hardware work.

The Mini's basic physical controls have already been validated on Windows 11. Preserve that evidence and do not redo it unnecessarily.

Continue development through all work that can be completed without having the physical PCPanel locally attached.

## Immediate priority: finish Milestone 2 integration quality

Review the existing Windows Core Audio and PipeWire implementations and the `veek-audio-probe`.

Close any obvious remaining gaps that can be tested on the current development machine.

Ensure the common audio abstraction cleanly supports:

- output discovery
- input discovery
- default input/output
- system/default volume
- device volume
- application/session volume
- output mute
- input/microphone mute
- application mute
- dynamic stream/session creation and removal
- external volume changes
- audio device changes
- backend restart/recovery
- stable target metadata needed by the mapping layer

Do not rewrite working audio code just for stylistic reasons.

Run the relevant tests and real Nobara/PipeWire integration checks where possible.

## Then proceed into Milestone 3

Build the platform-independent core that turns VeekPanel from a diagnostic project into an actual application.

### Mapping engine

Implement a real mapping engine that consumes normalized hardware events and controls normalized audio targets.

Support:

- knob → default output volume
- knob → specific output device
- knob → application volume
- knob → microphone/input level
- knob → audio group
- button → target mute/unmute
- button → microphone mute/unmute
- button → profile switching
- button → configurable action architecture

Hardware events must enter through an abstraction so the mapping engine works identically with:

- real PCPanel devices
- replayed captures
- mock/development devices

Do not couple mapping logic directly to USB/HID code.

### Important knob behavior

Implement sensible pickup/soft-takeover behavior so loading a profile or attaching a target does NOT cause an immediate volume jump simply because the physical knob position differs from the current system volume.

The engine should wait until the physical knob crosses or reaches the current target level before taking control, unless the user chooses another behavior later.

Make this logic deterministic and well tested.

### Application identity

Implement durable application identity.

Do not depend on PID, transient PipeWire node ID, or transient Windows session ID.

Use the strongest available stable metadata and layered fallback matching.

Mappings should survive application restart whenever reasonably possible.

Keep platform-specific identifiers behind the common identity model.

### Audio groups

Implement groups that allow one physical control to operate multiple targets.

Examples:

Gaming:
- game
- Discord

Media:
- Spotify
- browser
- VLC

Handle absent group members safely.

Define and document group-volume semantics instead of allowing accidental clipping or inconsistent relative levels.

### Profiles

Implement persistent profiles.

Profiles should contain:

- hardware control mappings
- application mappings
- audio groups
- button assignments
- preferred input/output targets
- available hardware-specific options

Support:

- create
- rename
- duplicate
- delete
- switch

Architect automatic profile switching for later without overcomplicating the initial implementation.

## Configuration system

Implement a durable versioned configuration format.

Requirements:

- schema/version field
- safe migrations
- atomic writes
- corruption recovery/backups where appropriate
- import/export-ready representation
- platform-independent mappings wherever practical
- deterministic serialization for debugging

Never silently discard an older configuration because the schema changed.

Write automated tests for migrations and recovery behavior.

## Application/service core

Create the long-running application controller that ties together:

hardware
→ normalized events
→ mapping engine
→ audio backend
→ state/configuration

It should also publish state changes for the future UI.

Keep this layer independent from the GUI.

The program should ultimately continue working when the main window is closed.

Do not build tray/startup features yet unless they naturally fit the architecture; those belong primarily to the background milestone.

## Start Milestone 4 once the backend core is genuinely usable

Once the mapping engine, profiles, configuration, and backend application controller are functioning, begin building the actual desktop GUI.

Use the technology stack already chosen by the project unless there is a documented technical reason to change it.

The GUI must connect to the REAL backend.

Do not create a disconnected visual prototype.

## Visual direction

The interface should feel like a modern first-party hardware controller application.

Use:

- Apple-inspired squircle geometry
- large rounded cards
- smooth but restrained animations
- clean typography
- generous spacing
- dark and light themes
- subtle translucency where appropriate
- modern iconography
- minimal visual clutter

Do not imitate macOS literally or copy Apple assets.

### Main dashboard

Create a dashboard visually representing the PCPanel Mini.

Show four prominent knob controls.

Each knob should display:

- assigned target
- current target volume
- mute state
- mapping state
- physical/hardware activity when available

The physical Mini should remain the design reference, but the UI architecture should support other PCPanel models later.

### Assignment workflow

Make control assignment visual and intuitive.

A user should be able to select a knob and assign:

- System Output
- application
- output device
- microphone/input
- audio group

Prefer direct manipulation, drag/drop, or a similarly intuitive modern interaction.

Do not expose raw backend identifiers to normal users.

### Mock/development mode

Because the physical hardware is not always locally available, retain an explicitly labeled mock/development device mode.

This should allow the entire application workflow to be exercised using virtual knobs/buttons while controlling REAL system/application audio.

Mock mode must never masquerade as real connected PCPanel hardware.

## Diagnostics

Retain and build upon the existing excellent diagnostic tooling.

The eventual GUI diagnostics page should expose useful information such as:

- hardware status
- detected model
- audio backend
- outputs
- inputs
- application streams/sessions
- stable app identity fields
- mappings
- recent backend errors

Keep low-level raw hardware tooling available separately for developers.

## Scope discipline

Do NOT:

- spend significant time redoing already validated Windows Mini knob/button parsing
- wait on remaining hardware lifecycle tests
- fake application/audio data outside development mode
- build a beautiful frontend that has no real backend
- mark untested functionality as verified
- collapse platform-specific audio code into the common core
- introduce unnecessary abstractions merely for theoretical future features

Do:

- preserve existing working code
- keep Windows and Linux first-class
- run tests frequently
- use the actual Nobara PipeWire environment for real integration tests
- add mock tests where hardware is unavailable
- update documentation when architecture changes
- make meaningful commits as coherent pieces are completed

## Working style

Work autonomously through this continuation.

Do not stop after merely proposing an implementation plan.

Inspect the existing implementation and begin making concrete progress.

Only stop for me if:

- a decision materially changes the product direction,
- elevated permissions are required,
- an operation risks user data/system configuration,
- or information genuinely cannot be inferred from the repository or tested safely.

Otherwise continue implementing, testing, documenting and committing.

The next major goal is:

**VeekPanel should become a functioning backend-driven desktop application where a mock Mini can control real system/application audio through persisted mappings and profiles, with the real Mini able to drop into the same hardware abstraction later.**

After that foundation works, continue into the polished dashboard/configuration UI.

## Subsequent user clarification (2026-10-05)

“Except I dont care if you directly copy apple for the visuals, gpt sol made the prompt”

The author permits a directly Apple-like visual style; the earlier wording requiring
only inspiration is not a product constraint. VeekPanel retains its own name/assets.
