#!/usr/bin/env python3
"""Serve Studio against ONLY the disposable cluster retained by test_postgres.py."""
import json
import os
from pathlib import Path
from urllib.parse import urlparse

ROOT = Path(__file__).resolve().parents[1]


def main():
    config = json.loads((ROOT / "target/revision-evidence/run.json").read_text())
    url = urlparse(config["app_url"])
    if url.hostname != "127.0.0.1" or url.username != "cantos_app" or not url.path.startswith("/cantos_test_"):
        raise ValueError("expected disposable test database")
    env = dict(os.environ, CANTOS_ENV="development", CANTOS_BIND_ADDRESS="127.0.0.1:8080", DATABASE_URL=config["app_url"])
    os.chdir(ROOT)
    print(f"Test script ID: {config['script']}", flush=True)
    print("Sign in with the synthetic alice test token (64 lowercase a characters).", flush=True)
    binary = str(ROOT / "target/debug/cantos-server")
    os.execve(binary, [binary], env)


if __name__ == "__main__":
    main()
