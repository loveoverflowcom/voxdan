"""Runner safety and fail-closed guards; process/HTTP doubles are not PG evidence."""
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import unittest
from unittest.mock import MagicMock, patch

import test_postgres as runner


class PostgresRunnerTests(unittest.TestCase):
    def test_optimized_python_refuses_before_cluster_or_host_start(self):
        result = subprocess.run([sys.executable, "-O", str(Path(runner.__file__)), "--help"],
                                text=True, capture_output=True, timeout=10)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("PostgreSQL assertions are required; run Python without -O", result.stderr)
        self.assertNotIn("PASS:", result.stdout)

    def test_inherited_pg_and_host_configuration_cannot_escape_the_disposable_cluster(self):
        inherited = {"PGHOSTADDR": "192.0.2.1", "PGSERVICE": "production",
                     "PGOPTIONS": "-c search_path=private", "PGPASSWORD": "synthetic",
                     "DATABASE_URL": "postgresql://example.invalid/production",
                     "CANTOS_WEB_ORIGIN": "https://example.invalid",
                     "CANTOS_WEB_DIST": "/tmp/private-test-content",
                     "CANTOS_SESSION_TOKEN": "inherited-token-must-not-be-used",
                     "CANTOS_TEST_CLUSTER_URL": "postgresql://example.invalid/other"}
        with patch.dict(os.environ, inherited):
            environment = runner.isolated_environment()
        for key in inherited:
            self.assertNotIn(key, environment)
        self.assertIn("PATH", environment)

    def test_readiness_mismatch_stops_and_reaps_the_unreturned_host(self):
        host = MagicMock()
        host.poll.return_value = None
        with patch.object(runner.subprocess, "Popen", return_value=host), \
                patch.object(runner, "http", return_value={"wrong": "response"}):
            with self.assertRaises(AssertionError):
                runner.start_host({}, 12345, io.StringIO(), {"response": {"expected": "response"}})
        host.kill.assert_called_once_with()
        host.wait.assert_called_once_with(timeout=5)

    def test_tool_session_is_only_in_child_environment_and_receipt_stays_structured(self):
        token = "a" * 64
        environment = {"PATH": "/synthetic/bin"}
        payload = {"run_id": "synthetic-run"}
        envelope = {"ok": True, "http_status": 200, "result": {"workflow": "caller"}}
        completed = subprocess.CompletedProcess([], 0, json.dumps(envelope), "")
        with patch.object(runner.subprocess, "run", return_value=completed) as execute:
            self.assertEqual(runner.adaptation_tool(environment, 12345, token, "review", payload, 200),
                             envelope)
        command = execute.call_args.args[0]
        options = execute.call_args.kwargs
        self.assertEqual(command, [sys.executable, str(runner.ADAPTATION_TOOL), "--base-url",
                                   "http://127.0.0.1:12345", "review"])
        self.assertNotIn(token, " ".join(command))
        self.assertEqual(options["env"]["CANTOS_SESSION_TOKEN"], token)
        self.assertNotIn("CANTOS_SESSION_TOKEN", environment)
        self.assertEqual(json.loads(options["input"]), payload)
        self.assertEqual(options["timeout"], 15)
        execute.assert_called_once()

    def test_tool_rejects_unstructured_or_inconsistent_receipts_without_printing_secrets(self):
        token = "a" * 64
        good = {"ok": True, "http_status": 200, "result": {"id": "synthetic"}}
        cases = [
            (0, "not JSON", "", "one JSON envelope"),
            (1, json.dumps(good), "", "unexpected status or envelope"),
            (0, json.dumps(dict(good, ok=False)), "", "unexpected status or envelope"),
            (0, json.dumps(dict(good, result=None)), "", "unexpected status or envelope"),
            (0, json.dumps(good), token, "output suppressed"),
            (0, json.dumps(dict(good, result={"secret": token})), "", "output suppressed"),
        ]
        for code, stdout, stderr, message in cases:
            with self.subTest(message=message, code=code):
                completed = subprocess.CompletedProcess([], code, stdout, stderr)
                with patch.object(runner.subprocess, "run", return_value=completed), \
                        self.assertRaisesRegex(RuntimeError, message) as failure:
                    runner.adaptation_tool({}, 12345, token, "context-read", {}, 200)
                self.assertNotIn(token, str(failure.exception))


if __name__ == "__main__":
    unittest.main()
