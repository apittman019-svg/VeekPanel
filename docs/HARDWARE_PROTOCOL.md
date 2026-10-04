# PCPanel protocol investigation

Status: public implementation evidence, **not physically verified here**.
See [research provenance](RESEARCH.md) for pinned source revisions and licenses.
The user corrected the friend's device to **Mini 1.0** on 2026-10-04 (previously
reported as Original). It has four pushable knobs. The existing Mini HID adapter
is the appropriate investigation path; the exact unit/revision remains unverified.

## Model matrix

| Model | VID:PID (hex) | Transport | Analog controls | Button controls | Native value |
| --- | --- | --- | --- | --- | --- |
| Original / Maple | Unknown for stock unit | USB serial, experimental | 4 knobs | Same 4 knobs, independent presses | nominal 0–100 |
| RGB | `04d8:eb52` | USB HID | 4 knobs | 4 presses | 0–100 |
| Mini | `0483:a3c4` | USB HID | 4 knobs | 4 presses | 0–255 |
| Pro | `0483:a3c5` | USB HID | 5 knobs + 4 sliders | 5 knob presses | 0–255 |

IDs/counts are corroborated by the Apache-2.0 [Device.java](https://github.com/nvdweem/open-pcpanel/blob/261a1c9fe8d9829ad07a353265464ab47fd669bc/core/src/main/java/dev/niels/pcpanel/core/device/Device.java).
Ranges are described by the community [DescriptorFactory](https://github.com/nvdweem/PCPanel/blob/f349b91c202c8ff9c9cb4809646bbdf7a39b85a5/src/main/java/com/getpcpanel/device/provider/pcpanel/DescriptorFactory.java).
Model names do not establish all firmware revisions. A USB release number is
reported as `bcdDevice`, not relabeled as a known firmware version.

## Original/Maple (experimental adapter)

The [MIT replacement firmware](https://github.com/beanstalk42/pcpanel-firmware/blob/26359f789eb558749f5758e1310a1d37c36b93de/src/main.cpp)
uses 9600 baud. Prototype framing is 8 data bits, no parity, 1 stop bit, no flow
control (Arduino defaults). ASCII lines end in LF, normally preceded by CR:

| Message | Interpretation |
| --- | --- |
| `v0x73\r\n` | Knob index 0 at position 73/100 |
| `b2 0\r\n` | Knob index 2 pressed (active-low pull-up input) |
| `b2 1\r\n` | Knob index 2 released |
| `pong\r\n` | Periodic heartbeat in replacement firmware, roughly 1.5 seconds |

Indexes are 0–3. Physical layout order, polarity, heartbeat behavior, startup
sequence, USB bridge ID and exact stock-firmware range still need a real capture.
The reference does not require host commands; the prototype sends none. No evidence
for Original LEDs or an initialization command was found. Do not invent either.

## HID input (RGB/Mini/Pro)

Each returned report is interpreted as one event. Meaningful leading bytes:

| Offset | Analog event | Button event |
| --- | --- | --- |
| 0 | `01` | `02` |
| 1 | Zero-based control index | Zero-based knob index |
| 2 | Unsigned absolute position | `01` pressed, `00` released |
| 3 onward | Padding/unused | Padding/unused |

Pro indices 0–4 are knobs, 5–8 sliders. Mini/RGB indices 0–3 are knobs.
There is no slider button. Evidence: MIT [Pro SDK and enums](https://github.com/oddbear/PcPanelPro/tree/18d5208/PcPanelPro).
The prototype accepts 3–64 bytes, validates opcode/index/value and ignores trailing
bytes. It rejects unsupported reports and keeps reading; it never indexes a short
buffer. It does not assume one USB byte range for every model.

[HIDAPI](https://libusb.info/hidapi/group__API.html) handles host framing: unnumbered
input has no synthetic leading zero. A numbered input includes its report ID; no
numbered PCPanel variation has been established here. Unknown framing is reported
as a parser error, not guessed by stripping bytes. Linux and Windows physical
captures must confirm behavior with the chosen backend.

## HID initialization and output

A state request uses payload byte `01` and zeros to a 64-byte payload. HIDAPI's
write buffer adds report ID `00` at the start, so the host buffer is 65 bytes:
`00 01 00 ... 00`. Request once per successful open/reopen; reject short writes.
`watch --no-init` permits a diagnostic comparison. Never send this to serial.
This requests current positions; it is not proof a report was caused by movement.
M1 exposes all events and does not take audio action on startup snapshots.

## LED/RGB investigation (not implemented)

The community [OutputInterpreter](https://github.com/nvdweem/PCPanel/blob/f349b91c202c8ff9c9cb4809646bbdf7a39b85a5/src/main/java/com/getpcpanel/device/provider/pcpanel/OutputInterpreter.java)
shows distinct RGB, Mini and Pro lighting packet families. Pro uses prefix `05`,
Mini `06`; the next byte selects sliders (`00`, Pro), labels (`01`, Pro), knobs
(`02`), logo (`03`, Pro), or whole-body animation (`04`). Knob slots carry a mode
and RGB/gradient color fields. RGB uses prefix `02` with a different layout and
per-knob tracking flags. Color/animation fields and brightness encoding are
model-specific. This is an investigation summary, not a validated complete LED spec.
No lighting writes, arbitrary output commands, firmware updates or feature-report
experiments are exposed by this prototype. Expand this section from captures before
adding LED support in a separately authorized milestone.

## Discovery, identity and safety boundaries

- Match both VID and PID for HID. Preserve path, interface and usage diagnostics;
  open by exact path, not the first VID/PID match. Unknown IDs stay unopened.
- No stock Original unique USB ID is verified. `list` enumerates serial candidates
  without opening them. Use `watch --serial PORT` explicitly. A generic bridge ID
  is never a sufficient identity or justification to probe unrelated equipment.
- Serial opening is labeled `OPENED_UNVERIFIED`. Valid control plus heartbeat
  yields `PROTOCOL_MATCH`, still not verified hardware identity. No stock-device
  claim is inferred from a PTY or replacement firmware.
- Each open session has a fresh parser. Lines are bounded to 64 bytes; oversized
  lines discard through LF. Timeouts create no event. EOF/errors terminate the
  session for retry; malformed messages do not terminate it.
- Retry/discovery uses a 3-second interval. HID input is blocking with a 100ms
  cancellation timeout, returning promptly on input. Serial reads also use 100ms.
- Handles are connection-scoped. Future persistent IDs must combine model and
  serial where trustworthy, with explicit missing/duplicate-serial resolution.
  Device paths and temporary node numbers are not durable user identities.

## Evidence still required

Run [HARDWARE_VALIDATION.md](HARDWARE_VALIDATION.md) first on the friend's stock
Mini 1.0 on Windows and Nobara. Confirm endpoints, rotation direction, every press
and release, framing and reconnect. Preserve redacted captures and add independent
fixtures. Then test each claimed HID model. Unknown revisions, suspend/resume and
silent USB resets are not certified by this prototype's automated tests.
