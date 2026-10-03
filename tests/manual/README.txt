VeekPanel — Milestone 1 Original PCPanel diagnostic kit

This is an experimental hardware test, not the finished audio app or an installer.
It has not been validated against the friend's physical Original PCPanel.
No audio control, firmware flashing, LED writes, or automatic uploads are included.
No Rust installation is needed. Use an ordinary Windows account, not Administrator.

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
