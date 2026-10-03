#!/usr/bin/env python3
"""Linux integration test through real serialport I/O and a synthetic PTY device.

NOT a physical PCPanel validation. Exercises fragmented traffic, unplug/reopen,
fresh decoder state and shutdown. Run after cargo build --workspace.
"""
import os
import pathlib
import pty
import queue
import subprocess
import sys
import tempfile
import threading
import time


def run(binary):
    lines = queue.Queue()
    transcript = []

    def wait_for(text, timeout=8):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            try:
                line = lines.get(timeout=max(0.01, deadline - time.monotonic()))
            except queue.Empty:
                break
            transcript.append(line)
            if text in line:
                return line
        raise AssertionError(f"Missing {text!r}: {''.join(transcript)}")

    with tempfile.TemporaryDirectory(prefix="veek-pty-") as tmp:
        port = pathlib.Path(tmp) / "original"
        master, slave = pty.openpty()
        port.symlink_to(os.ttyname(slave))
        process = subprocess.Popen(
            [str(binary), "watch", "--serial", str(port), "--raw"],
            stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
        )
        def collect():
            for line in process.stdout:
                lines.put(line)
        threading.Thread(target=collect, daemon=True).start()
        try:
            wait_for("OPENED_UNVERIFIED")
            os.write(master, b"v0x")
            time.sleep(0.05)
            os.write(master, b"73\r\nb2 0\r\nb2 1\npong\n")
            wait_for("KNOB_1 = 73")
            wait_for("KNOB_3_PRESS = TRUE")
            wait_for("KNOB_3_PRESS = FALSE")
            wait_for("PROTOCOL_MATCH")
            os.write(master, b"v0x")
            wait_for("RAW [76, 30, 78]")
            os.close(master)
            master = None
            os.close(slave)
            slave = None
            wait_for("DISCONNECTED_OR_OPEN_FAILED")
            port.unlink()
            master, slave = pty.openpty()
            port.symlink_to(os.ttyname(slave))
            wait_for("OPENED_UNVERIFIED")
            os.write(master, b"73\nv1x42\npong\n")
            wait_for("PARSE_ERROR")  # Old partial line did not survive reconnect.
            wait_for("KNOB_2 = 42")
            process.send_signal(__import__("signal").SIGINT)
            process.wait(timeout=3)
            assert process.returncode == 0
            print("PASS: synthetic PTY serial input, push buttons, unplug/reopen, parser reset, Ctrl+C")
        finally:
            if process.poll() is None:
                process.kill()
                process.wait()
            for fd in (master, slave):
                if fd is not None:
                    os.close(fd)


if __name__ == "__main__":
    binary = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else "target/debug/veek-probe").resolve()
    run(binary)
