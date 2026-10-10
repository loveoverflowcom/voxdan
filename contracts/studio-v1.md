# Studio revision API v1

Implemented development boundary: [Axum](../apps/server/src/http.rs) →
[PostgreSQL](../apps/server/src/postgres.rs), consumed by [Leptos](../apps/web/src/api.rs).
The [Schema](schema/studio/v1.schema.json), [raw Rust DTOs](api/src/lib.rs) and
[fixtures](fixtures/studio/v1) are hand-authored. Both consumers test the same fixture source;
backend tests validate live responses with that Schema. No generated OpenAPI/Kotlin API exists.

## Routes

| Method/path under /api/v1 | Request / result |
| --- | --- |
| POST /session | SessionRequest token → SessionResponse actor_id and HttpOnly/SameSite=Strict cookie |
| DELETE /session | Revoke token in PostgreSQL, expire cookie → 204 |
| GET /scripts/{UUID}/head | Current authorized Revision |
| GET /scripts/{UUID}/revisions/{positive revision} | Pinned immutable Revision, current access still required |
| POST /scripts/{UUID}/revisions | SaveRevision → Revision; 200 for both acceptance and exact replay |

All mutations require the configured exact Origin. API responses carry no-store/nosniff.
An existing active, unexpired, unrevoked database session authenticates the actor; request
actor headers are ignored. Tokens are exactly 64 hex characters, stored as SHA-256 hashes.
The cookie lacks Secure only because this host deliberately supports loopback development HTTP.
Production sign-in, HTTPS and Secure cookies require a separate reviewed deployment.

Owners/editors read and write; readers only read. Unknown/inaccessible scripts return 404,
not an existence leak; a known reader's write returns 403. First save at base 0 atomically
creates an owner script with revision 1. Later saves require its locked current head. Each
read, save and retry checks current access. A revoked member/session cannot replay old results
through this API.

SaveRevision contains expected_revision (0..i64::MAX), operation_id (canonical lowercase UUID)
and script_json (complete Script IR JSON string). The server revalidates schema/text/semantics,
canonicalizes a complete export, resolves every source/rights/generation/asset reference to
an immutable registry record owned by the script owner, then appends within one transaction.
Registry existence/ownership does not assert legal publication eligibility or preserved source
bytes. Validated storage acceptance does not mean production approval.

## Retry and integrity

The durable key is (script_id, authenticated actor, operation_id). Reuse with the same expected
base and complete canonical export returns the original revision, actor and UTC microsecond
timestamp even if the head has moved. A changed snapshot or base returns operation_reused.
Prior-operation lookup precedes the stale-head decision, after current authorization. A
different new key with a stale base returns stale_revision/current_revision. There is no
deduplication by c1, e1 or text equality.

An unknown transport/commit outcome must retry the same key/base/export as the original actor.
A caller cannot infer rollback from losing an HTTP response. The Studio keeps that snapshot in
memory, blocks new save/read intents until reconciliation, and preserves separately edited text.
A browser reload loses this in-memory intent; durable client recovery is a future authoring gate.

Revision returns script_id, revision, accepted_by, accepted_at, content_digest, export_digest
and the exact complete canonical script_json. Accepted rows and their evidence links are
append-only. Reads re-admit canonical bytes and verify both digests; corruption returns 500
rather than normalized/repaired content.

```text
ExportDigest = "sir-e1:sha256:" + lowercase_hex(
    SHA256(b"cantos/script-export/e1\n" + complete_canonical_export_bytes)
)
```

e1 includes schema, provenance and rights references omitted by the unchanged c1 content
projection. Revision identity plus its full export/evidence remains the accepted fact.

## Errors and bounds

| HTTP | code | Meaning |
| --- | --- | --- |
| 400 | invalid_request | Invalid DTO, UUID or revision bounds |
| 401 | unauthenticated | Missing, invalid, inactive, expired or revoked session |
| 403 | forbidden | Reader write or invalid mutation Origin |
| 404 | not_found | No row in current actor scope |
| 409 | stale_revision | Head changed; current_revision supplied |
| 409 | operation_reused | Same actor/key bound to another base/export |
| 422 | invalid_script / evidence_unavailable | Invalid IR or missing owner evidence |
| 503 | unavailable | Pool, statement/lock/request deadline or unknown transport/commit result |
| 500 | corrupt_revision | Stored canonical bytes/digest failed admission |

ApiError has code, nullable current_revision and issues [{path, rule}]. IR diagnostics expose
stable codes/paths, not raw manuscript values or SQL errors. Input IR is bounded to 2 MiB bytes;
the HTTP envelope limit is 8 MiB, allowing JSON escaping overhead. Pool size 8, wait/connect
5s, SQL 5s, lock 3s, idle transaction 10s and HTTP deadline 15s bound this development slice.
These are resource guardrails, not measured latency targets or production sizing.

Version 1 admits Script IR 0.1.0 only. Any changed wire semantics needs a compatibility decision
and shared fixture updates; no persisted older version conversion exists. See
[persistence evidence](../docs/evidence/script-revision-persistence.md).
