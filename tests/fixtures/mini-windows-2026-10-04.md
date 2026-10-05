# Mini Windows physical capture excerpt

This fixture contains 16 **physically captured** input reports, not generated
packets. Full 64-byte frames are unchanged; each line's comment identifies its
1-based position in the private source capture. Selection takes the first report
for each knob's 0/255 endpoint and press/release, retaining source order. Gaps are
intentional. Timing, reliability and simultaneous input cannot be inferred from
this reduced fixture. All 61 padding bytes were zero in every source report.

- Returned through the user on 2026-10-04 by the friend who owns the device.
- Tester reports Windows 11; edition/build not recorded.
- User reports Mini 1.0; tester marked printed model/revision and firmware unknown.
- Device: VID:PID `0483:a3c4`, interface 0, usage page `0xff00`, usage 1,
  `bcdDevice=0x0200` (not a verified firmware version). Serial present; value omitted.
- Binary source: `e703b4cf70eaa03c5168b5f16a4ffc72d6249e8a`,
  published `m1-mini-2026-10-04`; `test-mini`, initialization enabled.
- Source ZIP SHA256: `76e7d84f66eac46b917522d6ab8114dff43c6495157558d24311a2350b05b39f`.
- Source reports.hex SHA256: `efedc3552c4c088c78fc099a247d1b98d8c0892adb7daa1dd7952b77501bca90`.
- Source session: 4,473 reports / 286,272 bytes, 64 bytes each, duration stop,
  90,915 ms total including setup; 4,445 analog and 28 button reports; no parser errors.
- All 4,473 reports replayed and matched the returned events.txt exactly.

The completed physical worksheet confirms knob numbers 1–4 from left to right,
full travel and press/release on each, clockwise increasing values and no observed
missing/stuck/repeated/unexpected events. Raw ranges are 0–255 for every knob;
press/release counts are 2/2, 7/7, 3/3, 2/2 respectively. Button values alternate
1 then 0 and finish released in the supplied session.

The competing-apps field is blank. Reconnect/rerun is marked “Yes” without a
specific result or second supplied capture. Sleep/resume is explicitly NOT TESTED.
No Nobara physical evidence, audio-control test or Windows 10 result was supplied.
The full private capture and worksheet are not committed. This excerpt contains
only protocol bytes, with no USB serial value or personal filesystem paths.
