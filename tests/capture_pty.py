#!/usr/bin/env python3
"""Synthetic serial capture/guide integration. No physical hardware is validated."""
import os
import pathlib
import pty
import queue
import signal
import subprocess
import sys
import tempfile
import threading


def launch(binary, args, cwd, user_input=None):
    process = subprocess.Popen([str(binary), *args], cwd=cwd, stdin=subprocess.PIPE,
                               stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    lines = queue.Queue()
    def collect():
        for line in process.stdout:
            lines.put(line)
    threading.Thread(target=collect, daemon=True).start()
    if user_input is not None:
        process.stdin.write(user_input)
        process.stdin.flush()
    return process, lines


def opened(lines):
    seen = []
    try:
        while True:
            line = lines.get(timeout=10)
            seen.append(line)
            if "CAPTURE_OPENED_UNVERIFIED" in line:
                return
    except queue.Empty:
        raise AssertionError("Capture did not open: " + "".join(seen))


def run(binary):
    with tempfile.TemporaryDirectory(prefix="veek-capture-pty-") as tmp:
        root = pathlib.Path(tmp)
        master, slave = pty.openpty()
        processes = []
        try:
            port = os.ttyname(slave)
            output = root / "raw"
            process, lines = launch(binary, ["capture", "--serial", port, "--output", str(output), "--duration", "1"], root)
            processes.append(process)
            opened(lines)
            payload = b"v0x73\r\nb2 0\r\n\xff\x00unknown\n"
            os.write(master, payload)
            assert process.wait(timeout=5) == 0
            assert (output / "serial.bin").read_bytes() == payload
            metadata = (output / "metadata.txt").read_text()
            assert "hardware_validation=unverified" in metadata and "stop_reason=duration" in metadata
            assert port not in metadata
            replay = subprocess.run([str(binary), "replay", "--model", "original", str(output / "serial.bin")], capture_output=True, text=True, timeout=5)
            assert replay.returncode != 0 and "KNOB_1 = 73" in replay.stdout

            process, lines = launch(binary, ["test-original"], root, f"\n\n{port}\n\n")
            processes.append(process)
            opened(lines)
            guide_payload = b"v1x42\nb2 0\nb2 1\npong\n"
            os.write(master, guide_payload)
            # Wait for the recorder to persist the supplied bytes before interrupting.
            import time
            deadline = time.monotonic() + 5
            folder = next((root / "captures").iterdir())
            while (folder / "serial.bin").stat().st_size < len(guide_payload) and time.monotonic() < deadline:
                time.sleep(0.02)
            assert (folder / "serial.bin").stat().st_size == len(guide_payload)
            process.send_signal(signal.SIGINT)
            assert process.wait(timeout=5) == 0
            assert (folder / "RESULTS.txt").exists()
            assert "stop_reason=interrupted" in (folder / "metadata.txt").read_text()
            assert "NOT an automatic pass" in (folder / "RESULTS.txt").read_text()

            process, lines = launch(binary, ["capture", "--serial", port, "--output", str(root / "disconnect"), "--duration", "5"], root)
            processes.append(process)
            opened(lines)
            os.close(master)
            master = None
            assert process.wait(timeout=5) != 0
            assert "stop_reason=read_error" in (root / "disconnect" / "metadata.txt").read_text()
            print("PASS: synthetic raw capture, unknown-byte preservation, guided collection, Ctrl+C and disconnect evidence")
        finally:
            for process in processes:
                if process.poll() is None:
                    process.kill()
                    process.wait()
            if master is not None:
                os.close(master)
            os.close(slave)


if __name__ == "__main__":
    run(pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else "target/debug/veek-probe").resolve())
