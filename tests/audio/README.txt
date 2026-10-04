VeekPanel Milestone 2 experimental audio diagnostic
==================================================
Windows 10/11 x64 is primary. Linux needs native libpipewire-0.3 and a running
user-session PipeWire server; Nobara is the first Linux test target.
These are console developer tools, not the finished app or an installer.
No physical PCPanel behavior is certified. Windows builds are unsigned.

Read-only commands (Windows: veek-audio-probe.exe; Linux: ./veek-audio-probe):
  veek-audio-probe list
  veek-audio-probe watch --duration 30
Output is JSON. IDs identify live endpoints/sessions, not saved app mappings.
Identity metadata may include local executable paths and process IDs; review it
before sharing. Nothing is uploaded automatically.

Commands that CHANGE audio, only on the explicitly selected target:
  veek-audio-probe set --target default-output --volume 25
  veek-audio-probe set --target default-input --mute on
Use an exact target ID from list instead of default-output/default-input for a
particular endpoint or running app. Mute accepts on/off. Volume accepts 0..100.
The JSON receipt reports OS readback; confirmed=false exits with an error.
Defaults mean Windows multimedia role or PipeWire default node metadata.
No default-device switching is implemented.

Temporary Original PCPanel test (example COM3 must be replaced with the actual port):
  veek-audio-probe bind --serial COM3 --target default-output --knob 1 --duration 60
Optionally add --button-target default-input to mute/unmute that microphone with
the same knob's push button. No button action is enabled without that option.
Move the knob through the current OS volume to pick it up. Initial positions and
startup-held button states do not act. Release before pressing. Indices are 1-based
wire indices: physical ordering and the Original protocol remain unverified.
HID models can select an exact allowlisted --hid-path instead of --serial.

Watch reconnects after audio service failure. Bind stops on hardware/audio loss,
missing target or unconfirmed write. Rerun explicitly to rearm. Targets are pinned
for the run, including defaults resolved at startup. No writes are replayed after
a restart. Set/bind leave successfully changed audio state in place on exit.
No profiles, persistent mappings, GUI, tray, startup or full packaging yet.

BUILD_COMMIT.txt identifies the source revision; THIRD_PARTY contains dependency
notices. For current validation evidence and remaining gates, read:
https://github.com/apittman019-svg/VeekPanel/blob/main/docs/AUDIO_VALIDATION.md
