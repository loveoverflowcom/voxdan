"""Independent hostile-boundary cases; actual socket/storage journeys belong to test_postgres."""
import io
import http.client
import json
import unittest
from unittest.mock import patch

import cantos_adaptation_tool as tool


class AdaptationToolTests(unittest.TestCase):
    def test_destination_cannot_change_origin_or_send_a_session_to_another_host(self):
        for value in ["https://127.0.0.1:8080", "http://localhost:8080", "http://10.0.0.1:8080",
                      "http://127.0.0.1", "http://user:password@127.0.0.1:8080",
                      "http://127.0.0.1:8080/another", "http://127.0.0.1:8080/?token=private",
                      "http://127.0.0.1:8080/#another", "http://127.0.0.1:0"]:
            with self.subTest(value=value), self.assertRaises(tool.ToolInputError):
                tool.local_origin(value)
        self.assertEqual(tool.local_origin("http://127.0.0.1:8080/"), "http://127.0.0.1:8080")
        self.assertEqual(tool.local_origin("http://[::1]:8080"), "http://[::1]:8080")

    def test_ambiguous_json_or_unbounded_input_never_reaches_transport(self):
        for raw in [b'{"run_id":"a","run_id":"b"}', b'{"value":NaN}', b'{"value":1e999}',
                    b'{"value":"\\ud800"}', b'{"\\ud800":"value"}', b'[]', b'\xff',
                    b'{broken', b' ' * (tool.MAX_INPUT_BYTES + 1)]:
            with self.subTest(raw=raw[:30]), self.assertRaises(tool.ToolInputError):
                tool.read_input(io.BytesIO(raw))

    def test_tool_arguments_cannot_inject_paths_or_an_acceptance_action(self):
        for args in [{"run_id":"../accept"}, {"run_id":"00000000-0000-4000-8000-000000000001", "accept":True}]:
            with self.assertRaises(tool.ToolInputError):
                tool.route("review", args)
        with self.assertRaises(tool.ToolInputError):
            tool.route("accept", {"run_id":"00000000-0000-4000-8000-000000000001"})

    def test_uncertain_write_reports_exact_retry_without_echoing_source_or_token(self):
        token = "e" * 64
        with patch.object(tool.urllib.request.OpenerDirector, "open", side_effect=OSError("private-token=" + token)):
            result, code = tool.invoke("context", {"operation_id":"synthetic", "source":"PRIVATE_SENTINEL"},
                                       "http://127.0.0.1:8080", token)
        self.assertEqual(code, 2)
        self.assertEqual(result, {"ok":False,"http_status":None,"result":{"code":"write_outcome_unknown_retry_exact_operation"}})
        self.assertNotIn(token, json.dumps(result))
        self.assertNotIn("PRIVATE_SENTINEL", json.dumps(result))

    def test_response_bound_precedes_parsing(self):
        with self.assertRaises(tool.ToolInputError):
            tool.response_json(io.BytesIO(b' ' * (tool.MAX_RESPONSE_BYTES + 1)))

    def test_success_response_loss_retains_write_ambiguity_and_never_retries(self):
        class Response(io.BytesIO):
            status = 201

        for response in [Response(b'{malformed'), Response(b' ' * (tool.MAX_RESPONSE_BYTES + 1)),
                         Response(b'{"usage":1e999}'), Response(b'{"value":"\\ud800"}'),
                         Response(b'{"\\ud800":"value"}')]:
            with self.subTest(response=response), patch.object(tool.urllib.request.OpenerDirector, "open", return_value=response) as opened:
                result, code = tool.invoke("context", {"operation_id":"unchanged-id", "source":"PRIVATE_SENTINEL"},
                                           "http://127.0.0.1:8080", "f" * 64)
            self.assertEqual(code, 2)
            self.assertEqual(result, {"ok":False,"http_status":201,"result":{"code":"write_outcome_unknown_retry_exact_operation"}})
            self.assertEqual(opened.call_count, 1)
            self.assertNotIn("PRIVATE_SENTINEL", json.dumps(result))
            self.assertNotIn("f" * 64, json.dumps(result))
        response = Response()
        with patch.object(response, "read", side_effect=http.client.IncompleteRead(b"PRIVATE_SENTINEL")), patch.object(tool.urllib.request.OpenerDirector, "open", return_value=response) as opened:
            result, code = tool.invoke("context", {"operation_id":"unchanged-id"}, "http://127.0.0.1:8080", "f" * 64)
        self.assertEqual(code, 2)
        self.assertEqual(result["result"]["code"], "write_outcome_unknown_retry_exact_operation")
        self.assertEqual(opened.call_count, 1)
        self.assertNotIn("PRIVATE_SENTINEL", json.dumps(result))


if __name__ == "__main__":
    unittest.main()
