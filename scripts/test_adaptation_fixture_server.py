"""Synthetic HTTP double tests; these are not live model or PostgreSQL evidence."""
import io
import json
import unittest

from adaptation_fixture_server import FixtureRuntime, MAX_REQUEST_BYTES


class AdaptationFixtureServerTests(unittest.TestCase):
    def setUp(self):
        self.runtime = FixtureRuntime('{"title":"Đèn Vọng Đài"}')

    def call(self, body, path="/api/chat"):
        handler_class = self.runtime.handler()
        handler = handler_class.__new__(handler_class)
        handler.path = path
        handler.headers = {"Content-Length": str(len(body))}
        handler.rfile = io.BytesIO(body)
        replies = []
        handler.reply = lambda status, value: replies.append((status, value))
        handler.do_POST()
        self.assertEqual(len(replies), 1)
        return replies[0]

    def test_valid_and_truncated_envelopes_keep_exact_fixture_text_and_usage(self):
        for mode, reason in [("valid", "stop"), ("length", "length")]:
            self.runtime.mode = mode
            status, value = self.call(json.dumps({"model": "fixture-test-double", "stream": False}).encode())
            self.assertEqual(status, 200)
            self.assertEqual(value["message"]["content"], '{"title":"Đèn Vọng Đài"}')
            self.assertEqual(value["done_reason"], reason)
            self.assertEqual(value["prompt_eval_count"], 87)
            self.assertEqual(value["eval_count"], 123)
        self.assertEqual(self.runtime.requests, 2)

    def test_malformed_wrong_model_and_oversized_requests_never_dispatch(self):
        for body, expected in [(b"not-json", 400),
                               (b'{"model":"unapproved","stream":false}', 400),
                               (b"x" * (MAX_REQUEST_BYTES + 1), 413)]:
            self.assertEqual(self.call(body)[0], expected)
        self.assertEqual(self.runtime.requests, 0)

    def test_model_probe_has_only_model_configuration_and_never_dispatches(self):
        status, value = self.call(b'{"model":"fixture-test-double","verbose":false}', "/api/show")
        self.assertEqual(status, 200)
        self.assertEqual(value["details"]["format"], "gguf")
        self.assertEqual(value["modelfile"], "FROM /synthetic/blobs/sha256-" + "1" * 64)
        self.assertEqual(self.runtime.requests, 0)
        self.assertEqual(self.runtime.shows, 1)
        self.assertEqual(self.call(b'{"model":"fixture-test-double","verbose":false,"source":"private"}', "/api/show")[0], 400)


if __name__ == "__main__":
    unittest.main()
