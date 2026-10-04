# Physical hardware acceptance checklist

**No rows are passed yet.** Software/PTY tests are recorded in `VERIFICATION.md`.
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

All Mini physical checks remain pending reviewed evidence. The user reports the
hardware issue is understood, but has not yet supplied Mini HID events or specific
acceptance results. The earlier returned Original serial capture has zero bytes,
was interrupted after approximately 25 seconds, and has an unfilled worksheet.
It is inconclusive and cannot establish either a Mini failure or success.

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
