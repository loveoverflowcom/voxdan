#!/usr/bin/env python3
"""Loopback-only synthetic Ollama HTTP contract double for tests and Studio inspection.

This process performs no AI inference and never contacts a provider. The caller must
select an original synthetic proposal document. Request text is never logged.
"""
import argparse
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import threading

ROOT = Path(__file__).resolve().parents[1]
MAX_REQUEST_BYTES = 96 * 1024


class FixtureRuntime:
    def __init__(self, document, model="fixture-test-double"):
        self.document = document
        self.model = model
        self.requests = 0
        self.probes = 0
        self.shows = 0
        self.cloud_disabled = True
        self.mode = "valid"
        self.lock = threading.Lock()
        self.entered = threading.Event()
        self.release = threading.Event()
        self.release.set()

    def handler(self):
        runtime = self

        class Handler(BaseHTTPRequestHandler):
            def log_message(self, *_args):
                # No request/model/source content appears in stderr or persisted logs.
                pass

            def reply(self, status, value):
                body = json.dumps(value, ensure_ascii=False).encode()
                self.send_response(status)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                try:
                    self.wfile.write(body)
                except (BrokenPipeError, ConnectionResetError):
                    pass  # Expected when the test kills a dispatched Cantos host.

            def do_GET(self):
                if self.path == "/api/status":
                    with runtime.lock:
                        runtime.probes += 1
                    self.reply(200, {"cloud": {"disabled": runtime.cloud_disabled, "source": "env"}})
                    return
                if self.path != "/fixture/status":
                    self.reply(404, {"error": "fixture_route_only"})
                    return
                with runtime.lock:
                    count = runtime.requests
                self.reply(200, {"kind": "synthetic_contract_double", "requests": count,
                                 "status_probes": runtime.probes, "model_probes": runtime.shows})

            def do_POST(self):
                if self.path not in ("/api/chat", "/api/show"):
                    self.reply(404, {"error": "fixture_route_only"})
                    return
                try:
                    length = int(self.headers.get("Content-Length", "0"))
                    if not 0 < length <= MAX_REQUEST_BYTES:
                        self.reply(413, {"error": "request_limit"})
                        return
                    request = json.loads(self.rfile.read(length))
                    if not isinstance(request, dict) or request.get("model") != runtime.model:
                        self.reply(400, {"error": "fixture_contract"})
                        return
                except (ValueError, UnicodeDecodeError):
                    self.reply(400, {"error": "fixture_contract"})
                    return
                if self.path == "/api/show":
                    if request != {"model": runtime.model, "verbose": False}:
                        self.reply(400, {"error": "fixture_contract"})
                        return
                    with runtime.lock:
                        runtime.shows += 1
                    self.reply(200, {
                        "details": {"format": "gguf", "parameter_size": "1.0B"},
                        "model_info": {"general.architecture": "llama"},
                        "modelfile": "FROM /synthetic/blobs/sha256-" + "1" * 64,
                    })
                    return
                if request.get("stream") is not False:
                    self.reply(400, {"error": "fixture_contract"})
                    return
                with runtime.lock:
                    runtime.requests += 1
                    mode = runtime.mode
                runtime.entered.set()
                if not runtime.release.wait(timeout=15):
                    self.reply(503, {"error": "fixture_barrier_timeout"})
                    return
                self.reply(200, {
                    "model": runtime.model,
                    "message": {"role": "assistant", "content": runtime.document},
                    "done": True, "done_reason": "length" if mode == "length" else "stop",
                    "prompt_eval_count": 87, "eval_count": 123,
                })

        return Handler


def create_server(runtime, port=0):
    server = ThreadingHTTPServer(("127.0.0.1", port), runtime.handler())
    server.daemon_threads = True
    return server


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--port", type=int, required=True)
    parser.add_argument("--document", type=Path,
                        default=ROOT / "contracts/fixtures/adaptation/proposal-vi.json")
    args = parser.parse_args()
    if not 1 <= args.port <= 65535:
        parser.error("port must be between 1 and 65535")
    document = args.document.read_text()
    json.loads(document)
    server = create_server(FixtureRuntime(document), args.port)
    print(f"Synthetic contract double (no AI inference): http://127.0.0.1:{args.port}", flush=True)
    try:
        server.serve_forever(poll_interval=0.1)
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()


if __name__ == "__main__":
    main()
