#!/usr/bin/env python3
"""Run revision integration tests in a newly created disposable PostgreSQL cluster.

Never reads DATABASE_URL or connects to an existing database. --keep retains this
test cluster for local Studio inspection; its explicit stop command is printed.
"""
import argparse
import json
import os
from pathlib import Path
import shutil
import socket
import subprocess
import time
import urllib.error
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "target" / "revision-evidence"


def run(command, env=None, output=None):
    result = subprocess.run(command, cwd=ROOT, env=env, text=True, capture_output=True, timeout=180)
    if output:
        output.write_text(result.stdout + result.stderr)
    if result.returncode:
        print(result.stdout + result.stderr)
        raise RuntimeError(f"FAIL: {command[0]} exited {result.returncode}")
    print(result.stdout.strip())
    return result.stdout


def free_port():
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


def http(port, fixture, method="GET"):
    path = "head" if method == "GET" else "revisions"
    request = urllib.request.Request(
        f"http://127.0.0.1:{port}/api/v1/scripts/{fixture['script']}/{path}",
        data=None if method == "GET" else json.dumps(fixture["request"]).encode(),
        method=method,
        headers={"Cookie": f"cantos_session={fixture['token']}",
                 "Origin": f"http://127.0.0.1:{port}", "Content-Type": "application/json"},
    )
    with urllib.request.urlopen(request, timeout=3) as response:
        assert response.headers["Cache-Control"] == "no-store"
        return json.load(response)


def start_host(env, port, log, fixture):
    host_env = dict(env, CANTOS_ENV="development", CANTOS_BIND_ADDRESS=f"127.0.0.1:{port}")
    host = subprocess.Popen([str(ROOT / "target/debug/cantos-server")], cwd=ROOT, env=host_env, stdout=log, stderr=log)
    deadline = time.monotonic() + 10
    while time.monotonic() < deadline:
        if host.poll() is not None:
            raise RuntimeError("host exited before readiness; inspect host.log")
        try:
            assert http(port, fixture) == fixture["response"]
            return host
        except (OSError, urllib.error.URLError):
            time.sleep(0.05)
    host.kill()
    host.wait(timeout=5)
    raise RuntimeError("host readiness timed out")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--keep", action="store_true")
    args = parser.parse_args()
    for tool in ("initdb", "pg_ctl", "psql", "cargo"):
        if not shutil.which(tool):
            raise RuntimeError(f"unavailable: {tool}")
    EVIDENCE.mkdir(parents=True, exist_ok=True)
    cluster = EVIDENCE / f"cluster-{time.time_ns()}"
    cluster.mkdir()
    data = cluster / "pgdata"
    port = free_port()
    env = dict(os.environ)
    # Ignore any inherited application/production URL.
    env.pop("DATABASE_URL", None)
    url = f"postgresql://cantos_test_admin@127.0.0.1:{port}/postgres"
    env["CANTOS_TEST_CLUSTER_URL"] = url
    started = False
    host = None
    try:
        run(["initdb", "-D", str(data), "--username=cantos_test_admin", "--auth=trust", "--no-locale", "--encoding=UTF8"], output=cluster / "initdb.log")
        run(["pg_ctl", "-D", str(data), "-l", str(cluster / "postgres.log"), "-o", f"-h 127.0.0.1 -p {port} -k /tmp", "-w", "start"])
        started = True
        run(["psql", url, "-v", "ON_ERROR_STOP=1", "-c", "CREATE ROLE cantos_app LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE; SELECT version();"])
        run(["cargo", "test", "-p", "cantos-server", "--test", "revisions_postgres", "--locked", "--offline", "--", "--ignored", "--test-threads=1"], env=env, output=EVIDENCE / "postgres-tests.log")
        fixture = json.loads((EVIDENCE / "restart.json").read_text())
        app_url = f"postgresql://cantos_app@127.0.0.1:{port}/{fixture['database']}"
        env["DATABASE_URL"] = app_url
        with (EVIDENCE / "host.log").open("w") as log:
            http_port = free_port()
            host = start_host(env, http_port, log, fixture)
            assert http(http_port, fixture, "POST") == fixture["response"]
            # Abrupt HTTP process death after a committed save (lost response replay).
            host.kill()
            host.wait(timeout=5)
            host = None
            run(["pg_ctl", "-D", str(data), "-l", str(cluster / "postgres.log"), "-m", "fast", "-w", "restart"])
            migration_env = dict(env, DATABASE_URL=f"postgresql://cantos_test_admin@127.0.0.1:{port}/{fixture['database']}")
            run([str(ROOT / "target/debug/cantos-migrate")], env=migration_env)
            host = start_host(env, http_port, log, fixture)
            assert http(http_port, fixture) == fixture["response"]
            assert http(http_port, fixture, "POST") == fixture["response"]
            print("PASS: HTTP process kill + PostgreSQL restart + migration replay + exact export/actor/time + same-operation replay")
        (EVIDENCE / "run.json").write_text(json.dumps({"cluster": str(cluster), "port": port, "app_url": app_url, "script": fixture["script"]}, indent=2) + "\n")
    finally:
        if host is not None:
            host.kill()
            host.wait(timeout=5)
        if started and not args.keep:
            run(["pg_ctl", "-D", str(data), "-m", "fast", "-w", "stop"])
        if args.keep and started:
            print(f"Retained test cluster. Stop with: pg_ctl -D {data} -m fast -w stop")


if __name__ == "__main__":
    main()
