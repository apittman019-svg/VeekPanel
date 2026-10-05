# Physical hardware acceptance checklist

**Mini basic detection and controls passed on the friend's Windows 11 machine.**
Broader lifecycle/platform acceptance remains pending. Software/PTY tests are
recorded separately in `VERIFICATION.md`.
First priority is the friend's Mini 1.0, corrected by the user on 2026-10-04,
on Windows and Nobara. Do not flash
firmware, install guessed drivers, or send undocumented commands to obtain a pass.

## Record the test environment

- Date, OS edition/build, kernel where relevant, Rust version, Git commit.
- Model label/photo description, stock vs modified firmware (unknown is acceptable).
- USB VID/PID, product/manufacturer strings, interface/usage/report descriptor,
  `bcdDevice` and serial-present/absent. Redact the serial value and personal paths.
- Driver/transport and node/COM path; competing PCPanel apps closed.
- Command used and raw returned lengths/bytes; fixture provenance is physical,
synthetic, or source-derived. Never change that label to imply stronger evidence.

## Mini 1.0: current friend trial

Use the Mini kit's `TEST-MINI.cmd` or `veek-probe test-mini`, with competing
PCPanel software closed. See [README](../README.md#friend-testing--downloadable-diagnostic-kit).
The test automatically detects a single Mini HID interface and preserves raw
reports, timestamps, decoded events/errors and a summary in a new capture folder.
No serial port selection is involved. `START-HERE.cmd` is Original-only.
Default discovery is bounded to 30 seconds and recording to 90 seconds/1 MiB raw.
Use `--no-init` only for an explicit state-request comparison. Disconnect ends the
recording: repeat for separate reconnect and sleep/resume trials. A finished
capture is not a pass; physical observations belong in its RESULTS.txt.

Record the actual VID/PID, model/revision label, all four analog endpoints and
indices, each button's press/release, and behavior after unplug/replug. Then check
sleep/resume, permissions and sustained input on each target OS. Initial state
reports alone do not demonstrate physical movement. The Mini reference range is
0–255; observe it rather than marking it passed from the implementation.

## Reviewed Mini result — Windows 11, 2026-10-04

Source kit: `m1-mini-2026-10-04`, binary commit
`e703b4cf70eaa03c5168b5f16a4ffc72d6249e8a`. The user supplied one physical capture
and the friend's completed worksheet. The user calls the unit Mini 1.0; the
worksheet lists printed model/revision and firmware as unknown. Windows edition/
build and competing-app status were not filled in. Do not generalize to every
firmware/revision or Windows 10.

Observed USB `0483:a3c4`, interface 0, usage page `0xff00`, usage 1,
`bcdDevice=0x0200` (USB release number, not a verified firmware version).
The tool opened HID with state initialization enabled and recorded 4,473 64-byte
reports: 4,445 analog, 28 button, 286,272 raw bytes, no parser errors. Capture
finished on duration at 90,915 ms including setup. All reports replayed successfully
and matched the supplied decoded events exactly. No adapter correction was needed.

| Check | Windows 11, supplied unit | Nobara |
| --- | --- | --- |
| Automatic Mini HID discovery/open with initialization enabled | Passed | Pending |
| All four knobs reach raw 0–255 | Passed: raw capture and physical worksheet | Pending |
| Left-to-right knobs map to 1–4; clockwise increases value | Passed: tester confirmed | Pending |
| Each knob button presses/releases independently | Passed: capture and tester; counts 2/2, 7/7, 3/3, 2/2 | Pending |
| Observed missing/stuck/repeated/unexpected events | None reported in this trial; not a soak test | Pending |
| Timed capture completion | Passed | Pending |
| Unplug/replug and rerun | Tester marked Yes; result unspecified, second capture absent | Pending |
| In-process reconnect, different USB port, permission recovery | Pending | Pending |
| Sleep/resume or USB reset | Explicitly not tested | Pending |
| Sustained simultaneous input, resource use and latency | Not established | Pending |

The [physical regression excerpt](../tests/fixtures/mini-windows-2026-10-04.md)
records provenance, source hashes and limitations. Its 16 unchanged frames are
selected from the private capture; the full ZIP/worksheet are not committed.
The original capture's `hardware_validation=unverified` metadata is preserved as
produced: this reviewed table records acceptance of specific checks separately.
The earlier empty Original serial capture remains inconclusive and is superseded
by this HID evidence for the friend's actual Mini.

## Original-only serial collection (retained for other units)

For an actual Original/Maple, `START-HERE.cmd` runs `veek-probe test-original`,
compares before/after device lists, requires explicit port selection, captures
60 seconds and supplies `RESULTS.txt`. No automatic upload or validation decision
is made. Physical observations and source/binary provenance must accompany captures.

Manual collection: `veek-probe capture --serial ACTUAL_PORT --output captures/new-test
--duration 60`. Create the parent first; the new-test directory must not exist.
`serial.bin` preserves exact received bytes including malformed input. `chunks.tsv`
describes host read boundaries, not necessarily USB packets. Metadata omits port
paths/USB serials and stays unverified. Capture ends on disconnect without retry;
keep each connection in a separate directory. Check stop_reason and the last status
field: byte_limit/read_error/output_error are not complete observation sessions,
and in_progress without a final status means the process did not finalize evidence.

## Original acceptance table

| Check | Windows 10/11 | Nobara |
| --- | --- | --- |
| Before/after connection identifies actual USB device and port | Pending | Pending |
| Establish safe positive automatic identity, or document explicit-port limitation | Pending | Pending |
| Verify stock 9600 8N1 framing and startup traffic | Pending | Pending |
| All four knobs: full travel, endpoints, orientation, stable index | Pending | Pending |
| All four knob buttons: press and release, polarity, independent rotation | Pending | Pending |
| Rapid movement / simultaneous presses, no lost/stuck edges | Pending | Pending |
| Heartbeat presence/absence and idle behavior | Pending | Pending |
| Unplug while idle and moving; reconnect same/different port | Pending | Pending |
| Permission denied / port busy produces understandable recovery | Pending | Pending |
| Suspend/resume and USB reset; controls resume without restart | Pending | Pending |
| Ctrl+C, timed shutdown and repeated reopen release handles | Pending | Pending |
| Idle resources and event latency measured | Pending | Pending |

Start with `list`, then `watch --serial ACTUAL_PORT --raw`. Values must match the
physical actions. An opened COM port, a `pong`, or replayed fixture alone is not a
pass. If evidence disagrees with the reference, preserve the capture and amend
the parser/protocol document with a regression test; do not normalize away unknown
bytes until their meaning is established.

## HID follow-up

Repeat each relevant lifecycle/control check for RGB, Mini and Pro independently.
Capture report lengths with/without state initialization; verify Pro slider indices,
every pushable knob and input ranges. Test more than one identical unit, missing or
duplicate serial numbers, and multiple HID interfaces. Confirm no unrelated devices
are opened. Test `--no-init` without assuming it works on every firmware.

When recording a pass, attach evidence and the tested model/firmware/OS. Do not
generalize one model's success to the family. Physical acceptance remains pending;
the continuation request authorizes independent audio/core/UI development meanwhile.
