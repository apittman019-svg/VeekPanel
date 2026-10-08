"""Transport/evidence regressions without WebKit, PipeWire or physical hardware."""
import json
import pathlib
import tempfile
import unittest
from http.client import RemoteDisconnected
from unittest.mock import Mock

from evidence import Evidence


class EvidenceTests(unittest.TestCase):
    def test_disconnected_mutation_is_never_replayed(self):
        evidence = Evidence()
        send = Mock(side_effect=RemoteDisconnected("driver closed"))
        body = {"script": "save_assignment()", "args": []}
        with self.assertRaises(RemoteDisconnected):
            evidence.request("POST", "/session/test/execute/sync", body, send)
        send.assert_called_once_with()
        body["script"] = "changed later"
        self.assertEqual(evidence.last_error["request"]["body"]["script"], "save_assignment()")
        self.assertIsNone(evidence.last_completed)

    def test_success_preserves_result_and_previous_transport_error(self):
        evidence = Evidence()
        with self.assertRaises(OSError):
            evidence.request("GET", "/status", None, Mock(side_effect=OSError("not ready")))
        value = {"ready": True}
        self.assertIs(evidence.request("GET", "/status", None, lambda: value), value)
        self.assertEqual(evidence.last_completed["path"], "/status")
        self.assertEqual(evidence.last_error["message"], "not ready")

    def test_failed_evidence_survives_screenshot_and_cleanup(self):
        evidence = Evidence()
        process = Mock(pid=42)
        process.poll.return_value = -11
        with tempfile.TemporaryDirectory() as folder:
            path = pathlib.Path(folder) / "result.json"
            try:
                evidence.request("POST", "/execute/sync", {"script": "read_preference()"},
                                 Mock(side_effect=RemoteDisconnected("driver closed")))
            except RemoteDisconnected as error:
                evidence.finish(path, error, {"driver": process, "server": None})
            evidence.request("GET", "/screenshot", None, lambda: "image")
            result = json.loads(path.read_text())
        self.assertEqual(result["status"], "failed")
        self.assertEqual(result["last_request"]["path"], "/execute/sync")
        self.assertEqual(result["processes"]["driver"]["returncode"], -11)
        self.assertIsNone(result["processes"]["server"])
        self.assertIn("RemoteDisconnected", result["failure"]["traceback"])

    def test_protocol_assertions_still_fail(self):
        evidence = Evidence()
        with self.assertRaisesRegex(AssertionError, "invalid session"):
            evidence.request("POST", "/session", None,
                             Mock(side_effect=AssertionError("invalid session")))


if __name__ == "__main__":
    unittest.main()
