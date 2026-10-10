# Server implementation guide

The `cantos-server` package now implements the Script IR contract, a local development Axum
host, PostgreSQL revision persistence and a migration CLI. It remains one modular package.
Production identity deployment, production workers and paid providers are not implemented.

## Boundaries

- `script_ir/`: private validated values, version admission, canonical export/content/speech.
  No HTTP, SQL, UI or provider dependency.
- `revisions.rs`: pure access table and append/replay/stale/key-reuse decisions.
- `postgres.rs`: current actor/session/access facts, transactions, locked heads, complete
  immutable bytes, evidence ownership links and integrity checks on every load.
- `http.rs`: thin Axum mapping over the [Studio v1 contract](../../contracts/studio-v1.md).
- `migrations/0001_script_revisions.sql`: atomic schema, deferred head FK, operation uniqueness
  and immutable revision/link triggers. Migration replay verifies its SHA-256 checksum.

All validated saves are immutable accepted storage revisions. They are not production approvals.
Metadata-only edits can retain c1 while changing e1 and revision identity; no digest deduplication
is performed. SQL rows are re-admitted through the canonical reader, not deserialized directly
into trusted domain values.

## Local verification and serving

Run from the repository root with Rust 1.87 and PostgreSQL binaries on `PATH`:

```sh
cargo run --locked --bin validate-script -- contracts/fixtures/script-ir/0.1.0/accept/two-scenes.json
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
PATH="/opt/homebrew/opt/postgresql@17/bin:$PATH" python3 scripts/test_postgres.py
```

The runner ignores inherited `DATABASE_URL`, initializes a fresh loopback-only cluster in
ignored `target/revision-evidence/`, creates a non-superuser app role, runs nine real PostgreSQL
tests and then kills/restarts HTTP and PostgreSQL processes. It stops the cluster by default.
The Rust integration tests are deliberately ignored by ordinary `cargo test`; that result
alone is not database evidence. The runner needs permission to allocate PostgreSQL shared memory.

For a synthetic live Studio walkthrough, build Web using [its guide](../web/README.md), then:

```sh
PATH="/opt/homebrew/opt/postgresql@17/bin:$PATH" python3 scripts/test_postgres.py --keep
python3 scripts/serve_studio.py
```

Use the printed UUID and synthetic alice token. The runner prints the exact `pg_ctl ... stop`
command for its retained cluster. Stop the HTTP host with Ctrl-C. No production data, real
credential, provider call or object-storage write is needed.

For another isolated development database, run `cargo run --locked --bin cantos-migrate`
with an explicitly exported migration-owner `DATABASE_URL`. Provision actors, high-entropy
64-hex-character session tokens (store only SHA-256), expiry and immutable owner evidence records
through that trusted operator connection. The runtime app role must be a separate non-owner,
non-superuser with only:

```sql
GRANT USAGE ON SCHEMA public TO cantos_app;
GRANT SELECT ON actors,sessions,scripts,script_members,script_evidence,
  script_revisions,revision_evidence TO cantos_app;
GRANT INSERT ON scripts,script_revisions,revision_evidence TO cantos_app;
GRANT UPDATE(head_revision) ON scripts TO cantos_app;
GRANT UPDATE(revoked) ON sessions TO cantos_app;
```

The migration does not create deployment roles or mint credentials. Never give its DDL owner
to the HTTP host. PostgreSQL superusers can bypass constraints/triggers; they are outside the
application threat boundary. The local runner's trust authentication and predictable tokens
are disposable test fixtures, not deployment guidance.

Run `cargo run --locked --bin cantos-server` with `CANTOS_ENV=development`, loopback
`CANTOS_BIND_ADDRESS` and the app-role local `DATABASE_URL`. Default origin is the bind URL
and default static directory is `apps/web/dist`; overrides are `CANTOS_WEB_ORIGIN` and
`CANTOS_WEB_DIST`. No dotenv loader exists. Remote PostgreSQL/TLS/production hosting are
explicitly unsupported by this host.

See [evidence and residual risks](../../docs/evidence/script-revision-persistence.md) and the
[affected read inventory](../../docs/evidence/script-revision-reads.md). Rights eligibility,
stable entity identity across revisions, source import and the full #1/010 journey remain
separate work. Domain rules never import Narrative Forge or AI provider internals.
