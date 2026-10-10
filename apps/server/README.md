# Server implementation guide

The `cantos-server` package now implements the Script IR contract, a local development Axum
host, PostgreSQL revision persistence and a migration CLI. It remains one modular package.
Production identity deployment, production workers and paid providers are not implemented.

## Boundaries

- `script_ir/`: private validated values, version admission, canonical export/content/speech.
  No HTTP, SQL, UI or provider dependency.
- `revisions.rs`: pure access table and append/replay/stale/key-reuse decisions.
- `imports.rs`: bounded deterministic TXT, Markdown, DOCX and Script IR extraction.
  Unknown speakers remain unresolved; prose labels and scene cues require review.
- `adaptation.rs`: closed proposal admission, bounded source context, trusted evidence bindings,
  source coverage findings and complete Script IR semantic validation. Domain decisions do
  not import provider transport; the development host has no enabled generation provider.
- `postgres.rs`: current actor/session/access facts, transactions, locked heads, complete
  immutable bytes, evidence ownership links and integrity checks on every load.
- `http.rs`: thin Axum mapping over the [Studio v1 contract](../../contracts/studio-v1.md).
- `migrations/0001_script_revisions.sql`: atomic schema, deferred head FK, operation uniqueness
  and immutable revision/link triggers. Migration replay verifies its SHA-256 checksum.
- `migrations/0002_editorial_handoff.sql`: preserved sources, immutable editorial reviews and
  their operation receipts, added without rewriting migration 0001 or existing revisions.
- `migrations/0003_manuscript_import.sql`: exact original binary bytes, import metadata and
  immutable extraction outcomes on the existing source records.
- `migrations/0004_ai_adaptation.sql`: immutable run inputs, dispatch intent, observations,
  proposals, cancellation receipts and atomic acceptance links.

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

The runner removes inherited `DATABASE_URL`, `PG*` and `CANTOS_*` configuration, disables psql
startup files, and refuses optimized Python (`-O`) before starting processes. It initializes a
fresh loopback-only cluster in ignored `target/revision-evidence/`, creates a non-superuser app
role, runs the real revision and manuscript PostgreSQL suites, and then kills/restarts HTTP
and PostgreSQL processes. Recovery compares exact TXT/DOCX/failed-input bytes and import receipts
as well as the accepted script. It stops the cluster by default.
The Rust integration tests are deliberately ignored by ordinary `cargo test`; that result
alone is not database evidence. The runner needs permission to allocate PostgreSQL shared memory.

For repeatable read measurements, run `scripts/benchmark_revision_reads.py` from the root
with the same PostgreSQL binaries on `PATH`. Its [protocol](../../docs/evidence/script-revision-benchmark-protocol.md)
defines safe synthetic scale/skew, actual HTTP serving, full PostgreSQL oracle comparisons,
permission isolation, EXPLAIN plans and write/storage accounting. Candidate indexes exist only
inside disposable comparison databases, not in the production migration. Resource sampling
also needs access to `ps`/`sysctl`; a denied/missing tool is a blocker, not a zero-cost result.

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
GRANT SELECT ON source_records,script_reviews,script_review_operations TO cantos_app;
GRANT INSERT ON script_reviews,script_review_operations TO cantos_app;
GRANT INSERT ON source_records,script_evidence TO cantos_app;
```

Migrations do not create deployment roles or mint credentials. Never give the DDL owner
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
stable entity identity across revisions and the full #1/010 journey remain
separate work. Domain rules never import Narrative Forge or AI provider internals.

## Editorial handoff and operator tools

Use the migration/operator database role for these commands, only against a local development
instance. The HTTP app role above cannot provision actors or credentials. Its evidence/source
INSERT privileges serve authenticated owner-scoped manuscript imports; general evidence
registration and operator source capture remain trusted operator commands.

```sh
cargo run --locked --bin cantos-migrate
cargo run --locked --bin cantos-operator -- create-creator
cargo run --locked --bin cantos-operator -- issue-token ACTOR_ID 1
cargo run --locked --bin cantos-operator -- revoke-tokens ACTOR_ID
cargo run --locked --bin cantos-operator -- register-evidence OWNER_ID rights RIGHTS_ID 'pending creator assertion'
cargo run --locked --bin cantos-operator -- record-source OWNER_ID SOURCE_ID 'preserved source reference' /path/to/source.txt
```

Issuance accepts 1–720 hours, prints the new 64-hex credential once, and stores only its SHA-256.
Keep the credential in a local secret store; use the existing Studio session exchange to sign
in. Rotation means issue a replacement and revoke existing credentials explicitly. Revocation
of all tokens for an actor includes the replacement if issued before `revoke-tokens`.
The source command preserves UTF-8 text bytes (including NFD/CRLF), bounded to 1 MiB, and records
an immutable source registry link plus SHA-256 in one transaction. Register the remaining
rights/generation/asset references explicitly; registration is not rights clearance. Source
correction requires a new ID. Existing registry descriptions are not overwritten.

[Studio API extensions](../../contracts/studio-v1.md#editorial-handoff-extensions) supply
currently authorized history summaries, linked private sources and owner editorial review.
Reviewing an already reviewed revision returns its first review, even after the head changes.
Reviewing an older unreviewed revision conflicts. Reviews bind immutable revision identity and
c1/e1; they do not approve production or publication. Existing Studio controls consume the
original save/read contract; the new handoff operations have no browser controls yet.

The [handoff evidence](../../docs/evidence/editorial-handoff.md) records the migration upgrade,
operator CLI, preservation, permission, corruption and rollback checks. Run the existing
PostgreSQL suite to include these cases; its ignored tests also cover the additive import migration.

## Manuscript intake

The [manuscript v1 contract](../../contracts/manuscript-v1.md) supplies authenticated import,
reopen and exact-original download routes. The source registry, bytes, metadata, checksum and
parse outcome commit together. An identical actor/operation retry returns its first receipt;
changing bytes or metadata under that operation conflicts. A parse failure preserves the source
and its actionable error. No import creates a ready script, changes a revision head or grants
publication permission. Structured scripts pass the existing Script IR validator and retain
their canonical representation for a later explicit revision save.

The bounded local source documents use existing PostgreSQL `source_records`, as recorded in
[ADR 0005](../../docs/decisions/0005-manuscript-import.md). Media/audio remain object storage.
Larger sources, a storage adapter and production identity require their own reviewed work.
DOCX formatting, embedded content and omitted annotations are visible conversion limitations;
tracked changes and suspected legacy Vietnamese encodings require an explicit future choice.
See [import evidence](../../docs/evidence/manuscript-import.md) for executed checks and limitations.

## Caller-owned adaptation tools

The [adaptation contract](../../contracts/adaptation-v1.md) exports immutable context and accepts
caller-created proposals on this same authenticated development host. The user-controlled
Gemini / ChatGPT / Codex host owns generation. Cantos performs no inference, has no provider
transport or credentials, and creates no provider attempt for a c1 context.

Use the [four HTTP/CLI tool schemas](../../contracts/adaptation-tools-v1.json): create context,
reopen exact context, submit proposal and read editorial review. The CLI requires an explicit
numeric loopback host/port and an existing process-local `CANTOS_SESSION_TOKEN`; it neither
provisions credentials nor connects a cloud account. All commands use the backend's current
session, source ownership and script permissions. The tool surface has no acceptance mutation.

A context pins source ID/SHA-256/extractor version, ordered blocks/warnings, optional exact base
export, rights assertion, prompt/contract versions and digest. It requires a parsed owned import
with nonblank rights holder, permission evidence and usage scope. These are recorded claims,
not legal adjudication. Publication permission remains unknown. Source/model strings have no
host or tool privileges. No context operation advances a revision head.

A submission pins the exact context digest and a separate operation ID. Host/provider/model/
config/prompt, supplied usage and cost are caller-declared, unverified evidence; absent values
stay unknown. Closed output admission constructs trusted IDs/provenance and uses the actual
Script IR validator. An invalid domain result stores an immutable receipt while awaiting a new
explicit correction. The first valid proposal is immutable; later distinct submission conflicts.
Exact context/submission retries reconcile their first result, including after acceptance.

Studio opens the saved context/proposal and supports source comparison, findings, bounded JSON
correction and explicit review/acceptance. Acceptance uses the same atomic revision-save/CAS/
idempotency path. Source and prior exports survive invalid submissions, lost replies and failed
commits. Full scene/character editing belongs to #4.

Application bounds are 24 KiB/256 source blocks, 96 KiB prepared text, 256 KiB proposal output,
1000 proposed nodes and 16 KiB/depth8/256nodes caller configuration. Context/submit HTTP bodies
are bounded to 64 KiB/1 MiB. The CLI bounds stdin/request to 1 MiB, response to 8 MiB and wait to
10 seconds, with no proxy, redirect, automatic retry or automatic operation ID. An uncertain
write must be retried with the exact operation and payload. These are pipeline bounds, not an
external model tokenizer/context guarantee. There is no automatic splitting or truncation.

The runtime role needs the existing adaptation permissions plus the append-only submission table:

```sql
GRANT SELECT, INSERT ON adaptation_runs,adaptation_attempts,adaptation_observations,
  adaptation_proposals,adaptation_cancellations,adaptation_acceptances,
  adaptation_submissions TO cantos_app;
GRANT UPDATE(status,problem,dispatch_deadline,updated_at) ON adaptation_runs TO cantos_app;
```

Migrations 0004/0005 retain the existing source/revision authority and add no UPDATE/DELETE rights
for its immutable facts. a1 frozen history keeps its exact decoder/digest; c1 has a separate type
and fingerprint. Retired start/provider endpoints fail unavailable/null, and historical queued/
expired attempts are never redispatched. See [ADR 0006](../../docs/decisions/0006-ai-script-adaptation.md)
and [current evidence](../../docs/evidence/ai-script-adaptation.md). Historical local-adapter
checks are explicitly superseded. Actual CLI/socket/PostgreSQL proof is required; real external
model generation and output quality remain separately NOT_RUN for this data-pipeline slice.
