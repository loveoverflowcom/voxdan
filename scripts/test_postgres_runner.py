"""Runner safety and fail-closed guards; process/HTTP doubles are not PG evidence."""
import io
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


if __name__ == "__main__":
    unittest.main()
