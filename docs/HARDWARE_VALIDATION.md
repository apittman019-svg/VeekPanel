# Physical hardware acceptance checklist

**No rows are passed yet.** Software/PTY tests are recorded in `VERIFICATION.md`.
First priority is the user's stock Original, on Windows and Nobara. Do not flash
firmware, install guessed drivers, or send undocumented commands to obtain a pass.

## Record the test environment

- Date, OS edition/build, kernel where relevant, Rust version, Git commit.
- Model label/photo description, stock vs modified firmware (unknown is acceptable).
- USB VID/PID, product/manufacturer strings, interface/usage/report descriptor,
  `bcdDevice` and serial-present/absent. Redact the serial value and personal paths.
- Driver/transport and node/COM path; competing PCPanel apps closed.
- Command used and raw returned lengths/bytes; fixture provenance is physical,
  synthetic, or source-derived. Never change that label to imply stronger evidence.

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
generalize one model's success to the family. Gate audio work on actual physical
input correctness and a separately authorized M2 task.
