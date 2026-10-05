# M2 audio implementation and validation

The user authorized the next milestone after the M1 diagnostic release. M2 adds
native audio controls and a temporary hardware-to-audio diagnostic, following the
original outline. M1 physical acceptance remains pending. The subsequent continuation request authorizes independent mapping/configuration
and desktop work; see CORE_AND_DESKTOP.md. There is no desktop GUI, saved mapping or audio mock in the shipped CLI.

## Run

Build with `cargo build --workspace --release --locked`; see platform setup guides.
Use `veek-audio-probe.exe` on Windows or `./veek-audio-probe` on Linux.

```sh
veek-audio-probe list
veek-audio-probe watch --duration 30
veek-audio-probe set --target default-output --volume 25
veek-audio-probe set --target default-input --mute on
```

`list` and `watch` are read-only JSON diagnostics. `set` changes only the selected
target. Substitute an exact ID from `list` for a device or a running app stream.
An app with several sessions has several explicit targets; grouping/matching is M3.
Volume is finite 0–100; mute is `on` or `off`. Receipt `confirmed` compares requested
state with OS readback (volume tolerance 0.5 percentage points). Unconfirmed writes
exit nonzero; they are not retried or reported as success. A successful readback is
not a guarantee of audible output, hardware position or long-term persistence.

Output may contain process IDs, executable paths and application metadata. Review
before sharing. Nothing uploads automatically. Targets are live IDs, not saved
configuration keys. Commands leave successful volume/mute changes in place on exit.

## Temporary control test

Basic Mini input has passed on the friend's Windows 11 unit. Use its exact HID
path from `veek-probe list` with `bind --hid-path PATH`, and choose a disposable
application/session for initial audio trials. The serial examples below are only
for an actual Original/Maple:

```sh
veek-audio-probe bind --serial COM3 --target default-output --knob 1 --duration 60
veek-audio-probe bind --serial COM3 --target TARGET_ID --knob 1 --button-target default-input
```

On Nobara substitute the actual `/dev/serial/by-id/...` path. HID can instead use
an exact `--hid-path` from `veek-probe list`; the path must match a known VID/PID.
Knobs are one-based wire indices, not verified physical order. This diagnostic
binds one knob and its independent button; slider mappings and multi-control
configuration belong to the mapping milestone. The button action is disabled
unless `--button-target` is provided. It toggles that target's currently observed mute.

The first analog sample never changes audio. Move through the current OS volume
(within 2 percentage points or crossing it) to pick up control. An external volume
change rearms pickup. Initial/held presses never toggle: a release must first be
observed, then a press edge. Duplicate pressed reports do not repeatedly toggle.
Targets, including default aliases, are resolved and pinned for the run. A changed
default does not silently move a binding to another device. On audio service loss,
USB loss, missing target or unconfirmed write, the binding exits; explicit rerun
rearms it. This deliberate diagnostic behavior is not the future durable mapping
and reconnect policy of M3/M5. `watch` reconnects read-only with a new generation.

## Backend behavior

- **Windows:** dedicated MTA thread, active render/capture endpoints through
  `IMMDeviceEnumerator`, `IAudioEndpointVolume`, and sessions through
  `IAudioSessionManager2`/`IAudioSessionControl2`/`ISimpleAudioVolume`. Uses endpoint,
  default, volume, session-created and session-state notifications. Retains session
  references and marshals callback-created objects using `AgileReference`; callback
  handoff is bounded at 256, with enumerator reconciliation. Extreme callback bursts
  still require a Windows stress test. All callback registrations are removed before
  COM teardown. No undocumented default-device switching API is used.
- **Linux:** `pipewire` Rust bindings over native libpipewire, registry/node/Props
  subscriptions and default-node metadata. No subprocess mixer backend or PulseAudio
  compatibility dependency. Cubic slider conversion (`native = slider³`), proportional
  channel balance preservation, equal channels when the prior vector is all zero.
  Writes use node Props, not WirePlumber route-policy persistence; physical ALSA,
  Bluetooth hardware routes and restoration after profile changes still need trials.
- A command carries a process-local connection generation plus an exact live ID.
  PipeWire IDs include server cookie and object serial; Windows sessions use endpoint
  and session-instance IDs. PIDs are metadata only. No stale command queue or stored
  commands are replayed after reconnect. Default aliases mean Windows multimedia
  role or PipeWire `default.audio.sink/source`; default switching is not exposed.
- Each backend owns its native objects on the audio worker. Callbacks update observed
  state or signal reconciliation. `watch` also reconciles on a 200ms bounded wait;
  this diagnostic is not an optimized background service. Blocking native Windows
  COM operations have OS-controlled latency; no hard cancellation guarantee is made.

## Automated evidence

`cargo test --workspace --locked` includes seven shared audio tests plus two CLI
integration tests. They cover range/NaN rejection, channel balance, missing targets,
unsupported controls, ambiguous defaults, stale generations, backend failure,
external observed state, unconfirmed writes, pickup and button startup/duplicates.
The M1 tests remain unchanged.

On Linux, run:

```sh
python3 tests/audio/pipewire_integration.py target/release/veek-audio-probe
```

This launches a separate temporary PipeWire daemon with explicit private runtime
and remote name. It loads no hardware discovery, desktop session manager or Pulse
server. Synthetic sink/source objects and a real `pw-cat` client exercise native
OS volume/mute, defaults, proportional balance, external notifications, app lifecycle,
service restart, stale-ID rejection and shutdown. PTY-generated Original packets
exercise the hardware decoder → pickup/button logic → native audio path. These are
not physical-panel or audible endpoint acceptance. Desktop audio is not mutated.

See [the verification record](VERIFICATION.md) for actual run results and CI links.

## Remaining manual acceptance

Record OS/version, source commit, real target IDs (redacted as needed), before/after
observations, and failures rather than assuming any row passed.

| Trial | Windows 10/11 | Nobara |
| --- | --- | --- |
| Real endpoint/input enumeration | Pending interactive machine | Read-only host enumeration passed |
| Output volume, microphone/input mute, app volume/mute | Pending interactive machine | Host default volume and secondary output/input mute restored; native test app volume/mute passed; audible/route behavior pending |
| External mixer/default changes and app close/reopen | Pending interactive machine | Isolated external/default changes passed; live test app lifecycle passed |
| Audio service restart with no stale writes | Pending | Isolated daemon passed; desktop session restart pending |
| Physical Mini knob → master then app, button → mute | Pending | Pending |
| Suspend/resume, Bluetooth/USB audio disappearance | Pending | Pending |
| Latency, channel behavior, permissions, sustained reliability | Pending | Pending |

M2 is an implemented diagnostic foundation, not an accepted production audio layer.
Do not mark physical acceptance complete until these trials provide evidence.

## Primary API references

Consulted 2026-10-03; dependencies pinned by Cargo.lock. Implementation is original
VeekPanel code, with API usage checked against upstream bindings and these sources:

- [Microsoft Core Audio interfaces](https://learn.microsoft.com/en-us/windows/win32/coreaudio/core-audio-interfaces)
- [Session enumeration and retained references](https://learn.microsoft.com/en-us/windows/win32/api/audiopolicy/nn-audiopolicy-iaudiosessionenumerator)
- [Session-created notification and GetCount ordering](https://learn.microsoft.com/en-us/windows/win32/api/audiopolicy/nf-audiopolicy-iaudiosessionmanager2-registersessionnotification)
- [Endpoint volume controls](https://learn.microsoft.com/en-us/windows/win32/coreaudio/endpoint-volume-controls)
- [Microsoft windows-rs bindings](https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/Media/Audio/index.html), version 0.62.2
- [PipeWire Rust ownership, threading and subscriptions](https://pipewire.pages.freedesktop.org/pipewire-rs/pipewire/), version 0.10.1
- [PipeWire node methods](https://docs.pipewire.org/structpw__node__methods.html)
- [WirePlumber cubic volume settings](https://pipewire.pages.freedesktop.org/wireplumber/daemon/configuration/settings.html)
