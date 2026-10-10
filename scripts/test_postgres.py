#!/usr/bin/env python3
"""Run revision integration tests in a newly created disposable PostgreSQL cluster.

Never reads DATABASE_URL or connects to an existing database. --keep retains this
test cluster for local Studio inspection; its explicit stop command is printed.
"""
import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import shutil
import socket
import subprocess
import time
import urllib.error
import urllib.request
import uuid
import zipfile

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


def isolated_environment():
    # libpq hostaddr/service/options can override an explicit local URL. Host configuration
    # must also belong to this run rather than exposing an inherited static directory.
    return {key: value for key, value in os.environ.items()
            if not key.startswith(("PG", "CANTOS_")) and key != "DATABASE_URL"}


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


def import_http(port, token, source=None, request=None, original=False):
    path = "/imports" if source is None else f"/imports/{source}"
    if original:
        path += "/original"
    headers = {"Cookie": f"cantos_session={token}", "Origin": f"http://127.0.0.1:{port}",
               "Content-Type": "application/json"}
    call = urllib.request.Request(f"http://127.0.0.1:{port}/api/v1{path}",
                                  data=None if request is None else json.dumps(request).encode(),
                                  headers=headers, method="GET" if request is None else "POST")
    with urllib.request.urlopen(call, timeout=5) as response:
        assert response.headers["Cache-Control"] == "no-store"
        assert response.headers["X-Content-Type-Options"] == "nosniff"
        return response.read() if original else json.load(response)


def import_restart_fixtures():
    """Original synthetic bytes; DOCX expected text is independent of the Rust extractor."""
    docx = io.BytesIO()
    with zipfile.ZipFile(docx, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        archive.writestr("[Content_Types].xml", '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>')
        archive.writestr("word/document.xml", '<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Mai: Ngày mai, mình có diễn tiếp không?</w:t></w:r></w:p><w:p><w:r><w:t>Nam: Có, ở Vọng Đài.</w:t></w:r></w:p></w:body></w:document>')
    fixtures = []
    for name, fmt, raw in [("vietnamese.txt", "txt", b"\xef\xbb\xbf" + "Mai: Nga\u0300y mai, mình có diễn tiếp không?\r\n\r\n— Chưa rõ người nói.\r\n".encode()),
                           ("vietnamese.docx", "docx", docx.getvalue()),
                           ("invalid.txt", "txt", b"\xff\x00")]:
        fixtures.append({"metadata": {"operation_id": str(uuid.uuid4()), "file_name": name,
                                      "format": fmt, "reference": "original synthetic recovery fixture",
                                      "rights_holder": None, "permission_evidence": None, "usage_scope": None},
                         "original_bytes": list(raw)})
    return fixtures


def start_host(env, port, log, fixture):
    host_env = dict(env, CANTOS_ENV="development", CANTOS_BIND_ADDRESS=f"127.0.0.1:{port}",
                    CANTOS_WEB_ORIGIN=f"http://127.0.0.1:{port}",
                    CANTOS_WEB_DIST=str(EVIDENCE / "no-static-assets"))
    host = subprocess.Popen([str(ROOT / "target/debug/cantos-server")], cwd=ROOT, env=host_env, stdout=log, stderr=log)
    try:
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline:
            if host.poll() is not None:
                raise RuntimeError("host exited before readiness; inspect host.log")
            try:
                assert http(port, fixture) == fixture["response"]
                return host
            except (OSError, urllib.error.URLError):
                time.sleep(0.05)
        raise RuntimeError("host readiness timed out")
    except BaseException:
        # Until return the caller cannot clean this process up, including assertion/interrupt.
        host.kill()
        host.wait(timeout=5)
        raise


def main():
    if not __debug__:
        raise RuntimeError("PostgreSQL assertions are required; run Python without -O")
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
    env = isolated_environment()
    url = f"postgresql://cantos_test_admin@127.0.0.1:{port}/postgres"
    env["CANTOS_TEST_CLUSTER_URL"] = url
    started = False
    host = None
    try:
        run(["initdb", "-D", str(data), "--username=cantos_test_admin", "--auth=trust", "--no-locale", "--encoding=UTF8"], env=env, output=cluster / "initdb.log")
        run(["pg_ctl", "-D", str(data), "-l", str(cluster / "postgres.log"), "-o", f"-h 127.0.0.1 -p {port} -k /tmp", "-w", "start"], env=env)
        started = True
        run(["psql", "-X", url, "-v", "ON_ERROR_STOP=1", "-c", "CREATE ROLE cantos_app LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE; SELECT version();"], env=env)
        run(["cargo", "test", "-p", "cantos-server", "--test", "revisions_postgres", "--locked", "--offline", "--", "--ignored", "--test-threads=1"], env=env, output=EVIDENCE / "postgres-tests.log")
        run(["cargo", "test", "-p", "cantos-server", "--test", "imports_postgres", "--locked", "--offline", "--", "--ignored", "--test-threads=1"], env=env, output=EVIDENCE / "import-postgres-tests.log")
        fixture = json.loads((EVIDENCE / "restart.json").read_text())
        migration_url = f"postgresql://cantos_test_admin@127.0.0.1:{port}/{fixture['database']}"
        run(["psql", "-X", migration_url, "-v", "ON_ERROR_STOP=1", "-c",
             "GRANT INSERT ON source_records,script_evidence TO cantos_app;"], env=env)
        app_url = f"postgresql://cantos_app@127.0.0.1:{port}/{fixture['database']}"
        env["DATABASE_URL"] = app_url
        with (EVIDENCE / "host.log").open("w") as log:
            http_port = free_port()
            host = start_host(env, http_port, log, fixture)
            assert http(http_port, fixture, "POST") == fixture["response"]
            imports = import_restart_fixtures()
            receipts = [import_http(http_port, fixture["token"], request=item) for item in imports]
            for index, (item, receipt) in enumerate(zip(imports, receipts)):
                raw = bytes(item["original_bytes"])
                assert receipt["sha256"] == hashlib.sha256(raw).hexdigest()
                assert receipt["outcome"]["status"] == ("failed" if index == 2 else "parsed")
                assert import_http(http_port, fixture["token"], source=receipt["id"], original=True) == raw
            assert receipts[0]["original_text"].encode() == bytes(imports[0]["original_bytes"])
            assert [block["text"] for block in receipts[1]["outcome"]["extraction"]["blocks"]] == [
                "Mai: Ngày mai, mình có diễn tiếp không?", "Nam: Có, ở Vọng Đài."]
            # Abrupt HTTP process death after a committed save (lost response replay).
            host.kill()
            host.wait(timeout=5)
            host = None
            run(["pg_ctl", "-D", str(data), "-l", str(cluster / "postgres.log"), "-m", "fast", "-w", "restart"], env=env)
            migration_env = dict(env, DATABASE_URL=f"postgresql://cantos_test_admin@127.0.0.1:{port}/{fixture['database']}")
            run([str(ROOT / "target/debug/cantos-migrate")], env=migration_env)
            host = start_host(env, http_port, log, fixture)
            assert http(http_port, fixture) == fixture["response"]
            assert http(http_port, fixture, "POST") == fixture["response"]
            for item, receipt in zip(imports, receipts):
                assert import_http(http_port, fixture["token"], request=item) == receipt
                assert import_http(http_port, fixture["token"], source=receipt["id"]) == receipt
                assert import_http(http_port, fixture["token"], source=receipt["id"], original=True) == bytes(item["original_bytes"])
            (EVIDENCE / "import-restart.json").write_text(json.dumps({"sources": receipts}, ensure_ascii=False, indent=2) + "\n")
            print("PASS: original TXT/DOCX/failed-input bytes and SHA-256 + HTTP kill + PostgreSQL restart + immutable receipt replay")
            print("PASS: HTTP process kill + PostgreSQL restart + migration replay + exact export/actor/time + same-operation replay")
        (EVIDENCE / "run.json").write_text(json.dumps({"cluster": str(cluster), "port": port, "app_url": app_url, "script": fixture["script"]}, indent=2) + "\n")
    finally:
        if host is not None:
            host.kill()
            host.wait(timeout=5)
        if started and not args.keep:
            run(["pg_ctl", "-D", str(data), "-m", "fast", "-w", "stop"], env=env)
        if args.keep and started:
            print(f"Retained test cluster. Stop with: pg_ctl -D {data} -m fast -w stop")


if __name__ == "__main__":
    main()
