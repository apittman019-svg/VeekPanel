VeekPanel — Milestone 1 Mini and Original PCPanel diagnostic kit

This is an experimental hardware test, not the finished audio app or an installer.
Physical Mini 1.0 compatibility remains pending the friend's test results.
No audio control, firmware flashing, LED writes, or automatic uploads are included.
No Rust installation is needed. Use an ordinary Windows account, not Administrator.

MINI 1.0 (the friend's corrected model):
1. Extract the entire Windows ZIP into a folder you can write to.
2. Close ALL other PCPanel apps and double-click TEST-MINI.cmd.
3. Connect the Mini and press any key. The test automatically looks for Mini HID
   0483:a3c4 for up to 30 seconds. No COM port or Device Manager is required.
4. Follow the 90-second prompts: each knob fully both ways, press/release twice.
5. Read captures/mini-.../SUMMARY.txt and fill in that folder's RESULTS.txt.
6. Unplug/replug and run again for a separate reconnect trial when convenient.
7. Review and zip the whole mini capture folder for return. Nothing uploads.

An opened device or startup report is not a pass. The summary counts reports,
not verified physical actions. NO INPUT RECEIVED means no control test succeeded.
Malformed packets are kept too; do not flash firmware or install guessed drivers.
Only the documented HID state request is written; --no-init can suppress it for
an explicit comparison. More than one Mini interface is never chosen arbitrarily.
Reports are bounded to 1 MiB raw data; metadata/events add a bounded representation.

Mini capture files:
  reports.hex: raw HID reads, one report per line, with time/length comments.
  events.txt: decoded events and parser errors with timing.
  metadata.txt: USB IDs/interface, state request setting, counts and stop reason.
  SUMMARY.txt: automatic observations, explicitly unverified.
  RESULTS.txt: the short physical checklist for you to fill in.
USB paths and unique serial values are omitted. Review raw evidence before sharing.
BUILD_COMMIT.txt is copied when launching from the kit directory.
If the console reports an error, note it in RESULTS.txt and keep the capture.
A forced termination leaves status=in_progress. Ctrl+C finalizes an early capture.

Mini manual command (Windows): veek-probe.exe test-mini
Mini manual command (Linux): ./veek-probe test-mini
Offline replay: veek-probe.exe replay --model mini captures/PATH/reports.hex
Replay does not connect hardware and cannot validate physical behavior.

ORIGINAL/MAPLE ONLY (not Mini):
1. Extract the entire Windows ZIP into a folder you can write to.
2. Read this file and close any other official/community PCPanel software.
3. Double-click START-HERE.cmd. It runs the included veek-probe.exe test-original.
4. Follow the before/after connection prompts to find the panel's actual COM port.
   A generic USB serial adapter is not automatically a PCPanel. If no port appears,
   stop and report what Device Manager shows; do not guess a driver or flash firmware.
5. Choose only the panel's observed port. During the 60-second recording, turn all
   four knobs and press/release each one separately. Record physical order/actions.
6. The tool saves received bytes and displays an OFFLINE replay. Parser errors may
   indicate an incomplete first/last line or differences in the stock protocol.
   An empty or successful replay does not establish hardware compatibility.
7. Open the new captures/original-... folder and fill in RESULTS.txt. Review files
   before sending the folder to the project owner. No files upload automatically.

serial.bin = raw received bytes, including data the parser does not recognize.
chunks.tsv = receive timings and hexadecimal chunks (host reads, not USB packets).
metadata.txt = settings, byte counts and stop reason; hardware stays unverified.
BUILD_COMMIT.txt in the kit identifies the source revision; include it in your results.
If capture stops on an error before RESULTS.txt is created, copy the supplied
RESULTS.txt template into that capture folder and record the error.

Capture stops on disconnect; run again for a separate reconnect trial. Ctrl+C can
stop early. Do not concatenate captures across different connections. A forced
termination leaves status=in_progress. Review all repeated status fields: the last
status/stop_reason describes a completed session. A finished capture is not a pass.

These development executables are unsigned. Distribution/security prompts may occur;
this kit does not change Windows security settings. Source and verification:
https://github.com/apittman019-svg/VeekPanel

Manual commands (Command Prompt in the extracted folder):
  veek-probe.exe list
  veek-probe.exe test-original
  veek-probe.exe watch --serial COM3 --raw --duration 60
  veek-probe.exe capture --serial COM3 --output captures\new-test --duration 60
  veek-probe.exe replay --model original captures\new-test\serial.bin
Use the actual COM port. The capture parent directory must already exist.

The Linux kit is a native test utility, not an AppImage/Flatpak installer. Run
./veek-probe test-original from its extracted directory. See repository Linux setup
for device permissions; no rules are installed automatically.
