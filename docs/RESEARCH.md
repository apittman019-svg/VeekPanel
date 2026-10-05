# Research and technology decision

Research date: 2026-10-03 UTC (2026-10-02 US Eastern). Facts from public sources,
implementation decisions, and unverified assumptions are distinguished below.
This is an independently written implementation, not a vendor-source port or a
formal clean-room process. No vendor binary was decompiled for this project.

## Hardware ecosystem and provenance

| Primary source | Revision / license observed | Findings / use |
| --- | --- | --- |
| [Official downloads](https://www.getpcpanel.com/download) | Vendor download page; no source license established | Separate Maple v1.1.0 and RGB/Mini/Pro v2.3.3 Windows downloads. Distinguishes legacy hardware; not evidence that every model uses HID. No binaries reused. |
| [beanstalk42/pcpanel-firmware](https://github.com/beanstalk42/pcpanel-firmware/tree/26359f789eb558749f5758e1310a1d37c36b93de) | MIT; [license](https://github.com/beanstalk42/pcpanel-firmware/blob/26359f789eb558749f5758e1310a1d37c36b93de/LICENSE) | Replacement Maple firmware claiming compatibility with vendor v1.1.0. Evidence for experimental serial grammar; not a capture of stock firmware. No firmware copied or flashed. |
| [nvdweem/open-pcpanel](https://github.com/nvdweem/open-pcpanel/tree/261a1c9fe8d9829ad07a353265464ab47fd669bc) | Apache-2.0; [license](https://github.com/nvdweem/open-pcpanel/blob/261a1c9fe8d9829ad07a353265464ab47fd669bc/LICENSE) | Archived predecessor. `Device.java` identifies RGB/Mini/Pro IDs and control counts; independently corroborates model table. Its README says Pro interaction mostly works and Mini lighting differs. |
| [oddbear/PcPanelPro](https://github.com/oddbear/PcPanelPro/tree/18d5208) | MIT; [license](https://github.com/oddbear/PcPanelPro/blob/18d5208/LICENSE) | C# Pro SDK, control indices, press/release values, state request and 64-byte payload. Host-library framing must not be confused with device payload. |
| [nvdweem/PCPanel](https://github.com/nvdweem/PCPanel/tree/f349b91c202c8ff9c9cb4809646bbdf7a39b85a5) | GPL v3 text; [license](https://github.com/nvdweem/PCPanel/blob/f349b91c202c8ff9c9cb4809646bbdf7a39b85a5/LICENSE) | Active community implementation. Describes original-app behavior, model ranges and different lighting formats. Predecessor README mentions decompilation in its lineage. Consulted for protocol facts only; no Java code/assets/native binaries imported. |
| [nikcident/pcpaneld](https://github.com/nikcident/pcpaneld/tree/ffc30c656e6c226d7b329698460c96f2d4eb635c) | MIT; [license](https://github.com/nikcident/pcpaneld/blob/ffc30c656e6c226d7b329698460c96f2d4eb635c/LICENSE) | Existing Rust Linux Pro controller. Useful feasibility comparison; not reused as this cross-platform app's base. |
| [taotien/PCPanel_Linux](https://github.com/taotien/PCPanel_Linux/tree/8b6637a) | No repository license found in inspected snapshot | Python/PulseAudio experiment. No code reuse. |
| [Faxmachinen2/pcpmcontrol](https://github.com/Faxmachinen2/pcpmcontrol/tree/e35c94d) | MIT in inspected snapshot | Mini Python implementation, further ecosystem evidence. No code reused. |

The brief describes the official software as discontinued. Public downloads remain
available and community maintenance exists; this research did not independently
establish the vendor's support status. Treat discontinuation as project motivation,
not a verified lifecycle announcement.

The user is building this for a friend who owns an Original with four pressable knobs. Its connection IDs,
firmware framing and button polarity cannot be established from the absent physical
unit. In particular, do not assert that a CH340/Arduino USB ID uniquely identifies it.
See [protocol evidence and uncertainties](HARDWARE_PROTOCOL.md).

## Stack decision

| Option | Assessment | Decision |
| --- | --- | --- |
| Rust core + Tauri 2 + TypeScript/Svelte | Small native control layer; system webview; independent hardware/audio workers; suitable visual UI. WebKitGTK/WebView2 and Linux packaging require testing. | Selected direction. Rust implemented now; Tauri/Svelte deferred to M4. |
| Rust + egui/Slint | Native rendering and less webview integration; viable fallback. Extra design/accessibility/tray integration work needs evaluation against the final UX. | Reconsider only if Tauri fails measured requirements. |
| C++/Qt | Mature native ecosystem and UI; more memory-safety and build/FFI complexity than this project needs. | Not selected. |
| C#/WinUI or WPF | Excellent Windows integration but does not provide the required Linux desktop path without another UI stack. | Not selected. |
| Electron | Bundled browser and Node runtime are difficult to justify for an idle hardware controller; native adapters still required. | Not selected. |

These are engineering judgments, not measured memory claims. Benchmark idle CPU,
resident memory, launch time and event latency before M4 acceptance. The UI must
never become a dependency of the hardware or audio libraries.

Selected prototype dependencies: Rust standard threads/channels, `hidapi` with
upstream HIDAPI Windows C and Linux hidraw backends, `serialport`, `clap`, `ctrlc`, `thiserror`.
Resolved versions are in `Cargo.lock`; [dependency license inventory](DEPENDENCIES.md) records declared licenses. Avoid an async runtime until there is a need.
Rust 1.99.0 is the installed/tested/pinned toolchain, not a claim of an older MSRV.
The optional `hidapi` 2.6.7 `windows-native` backend was rejected after inspection:
its synchronous `WriteFile` success branch returns zero, which cannot satisfy our
checked initialization-write contract. Use the bundled upstream Windows backend
and compile it with MSVC in Windows CI. Do not weaken short-write validation.
MIT is selected for original VeekPanel work. Dependency licenses remain their own;
release packaging must inventory native and transitive licenses too.

Primary implementation references:
[HIDAPI API](https://libusb.info/hidapi/group__API.html),
[hidapi Rust](https://docs.rs/hidapi/2.6.7/hidapi/),
[serialport Rust](https://docs.rs/serialport/4.10.1/serialport/),
[Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).
Tauri's system webview dependencies are WebView2 on Windows and WebKitGTK on Linux;
they are deliberately absent from M1.

## Windows audio decision (M2, design only)

Use the Rust `windows` crate for Core Audio COM APIs. Use `IMMDeviceEnumerator`
for render/capture endpoints, `IAudioEndpointVolume` for endpoint volume and mute,
and `IAudioSessionManager2`/`ISimpleAudioVolume` for application sessions. The app
controls audio state; it need not create WASAPI render/capture streams.
Register endpoint changes, volume notifications and session events, then reconcile
the initial enumeration without losing new sessions. Session-notification setup
needs an MTA and the documented enumerator initialization sequence. Keep callback
lifetimes and COM objects owned by the backend worker; callbacks enqueue events.

Sources: [session manager](https://learn.microsoft.com/en-us/windows/win32/api/audiopolicy/nn-audiopolicy-iaudiosessionmanager2),
[notification requirements](https://learn.microsoft.com/en-us/windows/win32/api/audiopolicy/nf-audiopolicy-iaudiosessionmanager2-registersessionnotification),
[endpoint volume](https://learn.microsoft.com/en-us/windows/win32/api/endpointvolume/nn-endpointvolume-iaudioendpointvolume).
Capture volume can be coupled across sessions; do not promise independently muted
microphone sessions without testing. Default-endpoint switching is a separate risk:
the listed documented APIs do not provide a general default-endpoint setter. Isolate
any later policy adapter, test Windows versions, and expose unsupported capability
honestly rather than pretending a button works.

## Nobara/Linux audio decision (M2, design only)

Use native `libpipewire` through maintained `pipewire-rs`, with one owning event-loop
thread. Subscribe to registry add/remove and node/device parameter changes; apply
SPA `Props` volume/mute and handle device `Route` properties where appropriate.
Track default-device metadata and cooperate with WirePlumber's policy and restore
behavior. Nodes, listeners and proxies stay on their owner thread. Use bounded
commands/events across the boundary. This is the preferred native route for both
PipeWire-native and Pulse-compatible application streams; do not parse `wpctl`
human output or spawn `pactl` for every knob event.

Sources: [pipewire-rs architecture](https://pipewire.pages.freedesktop.org/pipewire-rs/pipewire/),
[PipeWire API](https://docs.pipewire.org/topics.html),
[properties](https://docs.pipewire.org/page_man_pipewire-props_7.html),
[WirePlumber restore/default settings](https://pipewire.pages.freedesktop.org/wireplumber/daemon/configuration/settings.html).
Validate stereo channel balance, cubic UI-vs-linear gain, hardware routes, metadata
permissions and service restarts in M2. Native control is selected but unimplemented.
If gaps appear, evaluate a private WirePlumber library adapter; a libpulse subscription
backend can be a clearly labeled fallback, not the native implementation by another name.

Windows is the primary product target. Nobara is the first Linux release acceptance
target; Fedora, Ubuntu, Arch and Mint stay in the broader compatibility plan.

## Major risks and mitigation

| Risk | Consequence | Required mitigation / gate |
| --- | --- | --- |
| Original protocol inferred from replacement firmware | Stock unit may differ; no unique USB ID proven | Prioritize stock Original captures and press/release polarity. Explicit port selection until positive identity exists; never probe arbitrary serial devices. |
| HID framing / firmware revision differences | Off-by-one events or ignored initialization | Raw diagnostics, capture report descriptors and returned lengths; no heuristic byte stripping. Per-model adapter tests. |
| USB resets, missing serials, identical units | Wrong reconnection/mapping | Connection paths only for live handles; persistent identity separately; collision handling and reconnect/sleep tests. |
| Audio identities change or collide | Wrong app controlled | Metadata matching with confidence, explicit ambiguity, several sessions per identity; never PID/node-ID-only persistence. |
| PipeWire policy, gain scales and route differences | Volume fights other tools or wrong loudness | Subscribe and reconcile; preserve channels; test on actual Nobara before claiming native support. |
| Default device switching / Wayland foreground discovery | Some planned actions not available everywhere | Capability flags, isolated platform adapters; no fake active controls. |
| Flatpak raw HID/serial + host audio access | Sandbox may prevent core functionality | Packaging spike in M6; evaluate USB portal vs hidraw path access and narrowly scoped host helper. Never assume portal supplies hidraw. |
| Slow diagnostic consumer / stale handle | Lost input or blocked shutdown | Bounded queue, cancellation, worker lifecycle tests; production coalescing must preserve button edges. |
| Firmware LED writes and provenance | Incompatible packets / redistribution problems | Read-only protocol research; M1 only documented HID state request. Review source licenses before any reuse. |

Flatpak primary reference: [USB portal API](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Usb.html).
Host udev ACLs still matter. No installer or permission changes were applied in M1.


## Native app lifecycle follow-up (2026-10-04)

PipeWire client metadata enrichment exposed races when transient clients disappear
between registry announcement and bind, or before local proxy cleanup. The backend
now handles exact missing-global binds and missing retired-proxy destroy replies,
while retaining fatal handling for unrelated native errors. Native protocol Core
Destroy is opcode 7; this is error/lifetime handling, not an ignored write failure.
References: [Core API](https://docs.pipewire.org/core_8h_source.html),
[upstream native protocol](https://github.com/PipeWire/pipewire/blob/master/doc/dox/internals/protocol.dox).
Only API/protocol facts were consulted; no upstream implementation was copied.
