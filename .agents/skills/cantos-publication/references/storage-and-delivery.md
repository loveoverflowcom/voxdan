# Storage and delivery

> **Scope.** Keep Cantos media bytes in S3-compatible storage with private and publication
> classes separated by policy, immutable keys derived from content, asset rows that become ready
> only after checksum verification, idempotent uploads with ambiguous outcomes resolved, a URL
> signer behind the listener's authorization, delivery verified through the real CDN path
> (byte ranges, CORS, caching), cache purge as an optimization and retention that protects
> playback in progress. Use when adding a bucket, key layout, upload, copy, probe, signer, CDN
> setting, purge, retention or garbage-collection rule.

Owning rules: [architecture § Data and delivery](../../../../docs/architecture/overview.md#data-and-delivery)
(private versus approved assets, publishing never makes a production bucket public, readiness
after verification, purge is not truth),
[pipeline § Upload and publication boundary](../../../../docs/product/production-pipeline.md#upload-and-publication-boundary),
[mobile § Playback behavior](../../../../docs/architecture/mobile.md#playback-behavior) (short-lived
signed URLs, seek support) and the [030 risks](../../../../docs/work-plan/030-mix-review-and-publish.md#risks--unknowns)
(byte ranges, signed URL lifetimes, old versions for in-progress and offline playback). Vendor,
delivery mode and retention are open decisions; [`.env.example`](../../../../.env.example) lists
proposed configuration names only. Everything below is a proposal.

## 1. Two storage classes, separated by policy

| Class | Holds | Readers | Policy |
|---|---|---|---|
| private | source manuscripts, permission evidence, voice samples, raw provider output, dialogue renders, scene mixes, masters, staging copies | backend and worker credentials only | block all public access; no anonymous principal; no CDN origin access |
| publication | approved delivery renditions under immutable keys | the CDN origin identity; listeners only through the CDN | no listing; public read only if the delivery ADR chooses public mode, and never on the private class |

- Prefer **two buckets** (`S3_PRIVATE_BUCKET`, `S3_PUBLICATION_BUCKET` in the proposed config)
  over prefixes in one bucket: bucket policies and CDN origins attach to buckets, so the boundary
  is one setting instead of a prefix condition in every policy.
- **Promotion is a server-side copy** from the private class to the publication class. The
  application has no operation that changes an ACL or a bucket policy; such changes are
  deployment configuration under review, never a code path.
- Model the classes as distinct key types so the direction is `type-enforced`:

```rust
// Illustrative, proposed.
pub struct PrivateKey(KeyPath);
pub struct PublicationKey(KeyPath);

pub trait ObjectStore {
    async fn put_if_absent(&self, key: &PrivateKey, body: ByteSource, expected: &Sha256)
        -> Result<PutOutcome, StoreError>;
    async fn head(&self, key: &ObjectRef<'_>) -> Result<Option<ObjectFacts>, StoreError>;
    async fn promote(&self, from: &PrivateKey, to: &PublicationKey)
        -> Result<PutOutcome, StoreError>;          // copy-if-absent; never private → private
}
```

| Delivery mode (ADR) | Listener fetch | Exposure before commit | Retraction | Notes |
|---|---|---|---|---|
| signed | backend mints a short-lived CDN URL or cookie after listening's authorization | none: no URL without the backend | stop minting; outstanding URLs live until expiry | CDN cache key must exclude signature parameters (verify the vendor setting) |
| public | unsigned CDN URL of an immutable key | object reachable by key once promoted | delete at origin and purge; cached copies may linger | promotion waits until only asset blockers remain |

The architecture allows the mode to follow "the episode's policy"; record the mode in the
release's facts so the gate, signer and retraction all read the same value.

## 2. Keys are immutable and derived from content

```text
Proposed layout (illustrative):
private/sources/{work_id}/{sha256}.{ext}                    imported originals
private/evidence/{rights_record_id}/{sha256}.{ext}          permission evidence
private/voices/{creator_scope}/{voice_profile_id}/{sha256}.{ext}
private/provider-raw/{creator_scope}/{attempt_id}/{sha256}.{ext}   where provider terms allow
private/artifacts/{creator_scope}/{kind}/{sha256}.{ext}     dialogue, scene mix, master, rendition
publication/releases/{episode_id}/{sha256}.{ext}            approved renditions
```

| Rule | Why | Good | Counterexample | Oracle | Status · exception |
|---|---|---|---|---|---|
| Never overwrite a key | in-progress streams, CDN caches and offline checksums assume bytes never change | conditional create (`If-None-Match: *` where the store supports it; verify on MinIO and the vendor) | `PUT episode-01.m4a` on republish | second write to an existing key with different bytes is refused | proposed · none |
| Derive keys from content or immutable IDs | renames, Unicode normalization and PII must not touch storage | `sha256-<hex>.m4a` | `Ánh đèn cuối sân khấu/Một lời hẹn.m4a` | key builder property: same bytes → same key, any title → same key | proposed · none |
| Scope private keys by creator | cross-creator deduplication is deferred and leaks deletion across accounts | `{creator_scope}` segment | global `artifacts/{sha256}` | two-creator example | proposed · promotion target keys are per episode |
| Keys are internal | API identifiers must survive storage migration | listener sees asset IDs | key in the listener projection | projection field test (listening) | proposed · none |

When the pipeline already holds the accepted delivery rendition at a private key with a verified
checksum, that object *is* the staging copy; do not upload it again. A download filename, if
offered, is set at promotion as `Content-Disposition: attachment;
filename*=UTF-8''M%E1%BB%99t%20l%E1%BB%9Di%20h%E1%BA%B9n.m4a` (RFC 6266 encoding), never derived
into the key.

## 3. An asset row is ready only after verification

```rust
// Illustrative, proposed.
pub enum StoredObject {
    Pending { key: PrivateKey, expected: ExpectedObject },
    Uploading { key: PrivateKey, expected: ExpectedObject, upload: MultipartUploadId },
    Uploaded { key: PrivateKey, expected: ExpectedObject },   // the store said OK; unproven
    Stored { key: PrivateKey, verified: VerifiedObject },     // size and checksum confirmed
    Failed { key: PrivateKey, reason: StoreFailure },
}

pub fn verify_stored(
    expected: &ExpectedObject,
    observed: &ObjectFacts,
) -> Result<VerifiedObject, ObjectMismatch>;
```

- `VerifiedObject` has a private constructor; only `verify_stored` builds it, and the manifest
  and the gate accept only `Stored`.
- **An ETag is not a checksum.** A multipart ETag is derived from the parts, and some stores
  compute composite rather than full-object checksums for multipart uploads. Verify the
  full-object SHA-256 by read-back, or by a full-object checksum type the chosen store documents;
  confirm the behavior on MinIO and the vendor before relying on it.
- `Uploaded → Stored` is a separate step so a crash between "the store accepted it" and "we
  proved it" recovers by re-verifying, never by assuming.

## 4. Uploads, checksums and ambiguous outcomes

- **Intent before effect.** Commit the upload attempt row (key, expected checksum and size)
  before the first byte; record the multipart upload ID as soon as the store returns it.
- **Resume or abort.** On restart, list the recorded upload's parts and continue, or abort it and
  start over. A store lifecycle rule that aborts stale multipart uploads (a candidate) plus the
  reconciliation finding `StaleStagingUpload` keeps orphans from accumulating cost.
- **Integrity on the wire.** Send a per-part checksum header so the store rejects a corrupted
  body, then still verify the whole object (section 3).
- **Existing key.** Conditional create refusing because the key exists is not an error: read
  back, and equal checksum means success. A different checksum under a content-derived key is a
  corruption alarm, never an overwrite.
- **Ambiguous outcomes.** A timeout after sending may mean the object exists. Record the attempt
  as ambiguous, resolve it with `head` and verification, then retry only if absent. Content
  keys and conditional create make the retry safe; the record still matters, per the
  [AGENTS.md correctness rules](../../../../AGENTS.md#correctness-rules).
- **Workers are fenced.** Upload and promotion jobs use the lease and fencing tokens in
  [durable jobs](../../cantos-production-pipeline/references/durable-jobs.md); a stale worker
  cannot move a row to `Stored`.
- **Promotion sets headers explicitly.** Set `Content-Type`, `Cache-Control` and
  `Content-Disposition` on the publication copy instead of trusting the source metadata, and let
  the delivery probe confirm them.

## 5. Signed URLs are secrets with a lifetime

The `SignedUrl` value type and the rule that one is minted only after the backend authorizes the
listener are specified once, in [listener API § Authorization](../../cantos-listening/references/listener-api.md#authorization).
This section owns the signer behind it:

- A `UrlSigner` port in the shell, implemented by the CDN signing or store presigning adapter.
  Signing keys and storage credentials come from the deployment secret store; they never reach
  the Leptos/WASM bundle, the CMP app, a fact struct or an outbox payload.
- **Lifetime** is an ADR value. Streaming clients refresh before expiry
  ([mobile § Playback behavior](../../../../docs/architecture/mobile.md#playback-behavior)), so
  prefer short lifetimes and refresh over long ones; a download needs only enough time for one
  resumable segment.
- **Never logged anywhere it can be:** the backend's HTTP client spans, storage SDK error
  messages that echo the URL, reverse-proxy and CDN access logs (configure them to drop the query
  string or the signature parameter). The log-capture test asserts no signature parameter
  appears in captured lines.
- **Never an identity:** not a database column, manifest field, download-manifest key, analytics
  property or cache key. Identity is the asset ID and checksum.
- The delivery probe (section 6) uses a URL minted by the same signer and fetched through the
  same host listeners use; a probe with a privileged origin credential proves the origin, not
  delivery.

## 6. Verify through the delivery path

Run every check through the CDN host listeners use, after the object is `Stored` and promoted.
Probing earlier can make the CDN cache a `404` or `403` for that key; keep negative-cache
lifetimes short and never probe before upload completion.

| Check | Request | Pass condition |
|---|---|---|
| full object | `GET` | `200`; body SHA-256 and length equal the manifest |
| type | same response | `Content-Type` matches the manifest format |
| no re-encoding | `GET` with `Accept-Encoding: gzip, br` | no `Content-Encoding` on audio; compression breaks ranges and checksums |
| ranges advertised | `HEAD` | `Accept-Ranges: bytes`; `Content-Length` equals the size |
| first range | `Range: bytes=0-1023` | `206`; `Content-Range: bytes 0-1023/<size>`; bytes equal the local slice |
| middle range | `Range: bytes=<m>-<m+1023>` | `206`; bytes equal the slice |
| suffix range | `Range: bytes=-1024` | `206`; the last 1024 bytes |
| unsatisfiable range | `Range: bytes=<size>-` | `416` with `Content-Range: bytes */<size>` |
| CORS, if Theatre Web needs it | preflight with `Origin` and `Access-Control-Request-Headers: range` | the Theatre origin is allowed, `Range` is allowed, `Content-Range` and `Accept-Ranges` are exposed ([media interop](../../cantos-leptos-web/references/dom-and-media-interop.md)) |
| caching | response headers | long-lived `immutable` caching for content-addressed keys |
| access control (signed mode) | no signature; an expired signature | `403` |
| private class closed | anonymous `GET` of the source private key | `403` |

The probe returns an immutable `VerificationReport` with one entry per check; the gate's
`AssetObservation` is derived from it. A pure `judge_probe(&ExpectedObject, &ProbeResponses)
-> Vec<ProbeFailure>` keeps the pass conditions testable with recorded responses.

## 7. CDN caching and purge are optimizations

- Content-addressed audio is never mutated, so it never needs a purge.
- Mutable responses (catalog listings, if the CDN caches them) carry a short lifetime, which
  bounds staleness; the outbox purge only shortens it. The listener API reads the active pointer
  from PostgreSQL, which is the truth.
- A purge failure never fails or reverts a publish. Test: a `CachePurger` that always fails;
  publish commits, the origin listener API returns the new release, and the catalog is stale for
  at most its lifetime.
- In public mode, retraction purges the object URLs too; measure and report the observed lag
  instead of promising immediacy.

## 8. Retention protects playback in progress

Superseded objects stay needed: a session pinned to the prior release keeps fetching ranges, an
offline client may resume a download, and rollback re-verifies an earlier manifest. Listening's
active-release resolver reads the retention facts defined here for its prior-release exception
([listener visibility](../../cantos-listening/references/listener-api.md#listener-visibility)).

```rust
// Illustrative, proposed.
pub fn retention_decision(
    release: &ReleaseRetentionFacts,    // state, superseded_at, outstanding grants and sessions
    policy: &RetentionPolicy,
    now: Timestamp,
) -> RetentionDecision                  // Retain { until } | EligibleForDeletion | Required(Deletion)
```

- Never delete an object referenced by a release in `Staging`, `Verified`, `Ready` or `Active`,
  or by a `Superseded` release inside its window.
- Deletion is two-phase: record eligibility, wait a grace period, re-check references, delete.
  Reconciliation asserts that no referenced key is missing.
- Versioning or object lock on the publication bucket is a candidate safety net against
  accidental deletion; weigh its storage cost in the ADR.
- The retention window, and whether a rights expiry forces deletion, are product decisions the
  docs leave open; 030 requires them before republishing ships.

## Rule cards

| Rule | Why | Good | Counterexample | Oracle | Status · exception |
|---|---|---|---|---|---|
| Separate classes by policy | drafts and voice samples must stay private | two buckets; promotion copy | public-read on the production bucket "for verification" | anonymous `GET` on a real store → `403` | proposed · none |
| Ready only after verification | the store's `200` is not proof | `Uploaded → Stored` via `verify_stored` | `ready = true` after `PutObject` | crash between upload and record recovers by verification | proposed · none |
| Verify via the delivery path | origin success hides CDN, range and CORS failures | probe table through the CDN host | `HEAD` on the internal endpoint | recorded-response tests for `judge_probe`; live probe | proposed · until a CDN exists, report the gap |
| Signer is server-side and silent | leaked or persisted URLs | `UrlSigner` adapter, redacted logs | signature in an access log | log-capture test | proposed · none |
| Purge never decides | a purge failure must not change truth | outbox purge, idempotent | publish waits for the purge result | failing purger test | proposed · none |
| Retain before delete | in-progress and offline playback break | `retention_decision` + two-phase GC | delete prior objects at republish | reconciliation after GC | proposed · a rights-forced deletion, once decided |

## Tests

| Test | Level |
|---|---|
| `key_for_same_bytes_is_stable_and_independent_of_title` | `property-tested` |
| `second_put_with_different_bytes_is_refused` against MinIO | `integration-tested` |
| `multipart_etag_is_never_accepted_as_checksum` | `example-tested` |
| `ambiguous_put_resolves_by_head_without_second_object` | `fault-injected` |
| `judge_probe_rejects_gzip_encoded_audio_and_wrong_content_range` (recorded responses) | `example-tested` |
| `anonymous_get_on_private_class_is_forbidden` against MinIO | `integration-tested` |
| `signed_flow_logs_contain_no_signature_parameter` | `example-tested` |
| `purge_failure_does_not_fail_publication` | `example-tested` |
| `superseded_release_objects_survive_until_window_ends` | `example-tested` |

A local MinIO (a candidate) is a real S3-compatible store: tests against it are
`integration-tested` for store semantics — conditional writes, multipart, checksums, policies,
presigning. It proves nothing about CDN signing, caching, negative caching, CORS at the edge or
purge. A local reverse proxy in front of it tests that proxy, not the CDN; label it as such.
When code relies on a store behavior (conditional create, a checksum type), test it on the
chosen vendor too before claiming it.

## Gotchas

- Test audio is synthetic and generated at test time (a short tone or silence of a known
  duration); never commit downloaded media or generated episodes to Git.
- Audio served with `Content-Encoding: gzip` breaks `Range` semantics in some players and makes
  byte checksums meaningless; disable compression for audio types at the CDN.
- A CDN that includes query strings in the cache key gives every signed URL its own cache entry,
  multiplying origin load; signature parameters belong outside the cache key.
- Copying an object can drop or change metadata; set headers on the destination explicitly.
- Clock skew between signer and CDN shortens effective lifetimes; measure it before choosing a
  very short lifetime.
