"""Failure evidence for the private native GUI harness; never retries requests."""
import copy
import json
import os
import platform
import time
import traceback


class Evidence:
    def __init__(self):
        self.last_request = None
        self.last_completed = None
        self.last_error = None

    def request(self, method, path, body, send):
        request = {"method": method, "path": path, "body": copy.deepcopy(body)}
        self.last_request = request
        started = time.monotonic()
        try:
            value = send()
        except Exception as error:
            self.last_error = {
                "request": request,
                "type": type(error).__name__,
                "message": str(error),
            }
            raise
        self.last_completed = {**request, "elapsed_seconds": time.monotonic() - started}
        return value

    def finish(self, path, error=None, processes=None):
        # Snapshot before screenshot/session cleanup can replace the failed command.
        result = {
            "status": "failed" if error is not None else "passed",
            "commit": os.environ.get("GITHUB_SHA"),
            "run": os.environ.get("GITHUB_RUN_ID"),
            "attempt": os.environ.get("GITHUB_RUN_ATTEMPT"),
            "platform": platform.platform(),
            "python": platform.python_version(),
            "last_request": self.last_request,
            "last_completed": self.last_completed,
            "last_error": self.last_error,
            "processes": {
                name: None if process is None else {
                    "pid": process.pid, "returncode": process.poll()
                }
                for name, process in (processes or {}).items()
            },
            "failure": None if error is None else {
                "type": type(error).__name__,
                "message": str(error),
                "traceback": "".join(traceback.format_exception(error)),
            },
        }
        path.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
