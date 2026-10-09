# Staged publication protocol

> **Scope.** Make a Cantos release listener-visible only when every byte it references is
> stored, verified and deliverable: the release lifecycle state machine, the manifest, the
> staged upload → promote → verify → commit → outbox protocol, republish, rollback, retraction,
> reconciliation and the crash points that test it. Use when implementing or reviewing publish,
> republish, rollback or retract flows, release or manifest tables, the outbox events of
> publication, or a reconciliation job.

Owning rules: [pipeline § Upload and publication boundary](../../../../docs/product/production-pipeline.md#upload-and-publication-boundary)
(the five-step order and that storage and database share no transaction),
[architecture § Data and delivery](../../../../docs/architecture/overview.md#data-and-delivery)
(atomic metadata transition, failed replacement keeps the current release playable, cache purge
is not truth) and rules 22–24 in
[business rules § Review and publication](../../../../docs/product/business-rules.md#review-and-publication).
Object-level mechanics (keys, uploads, probes, signed URLs) are in
[storage and delivery](storage-and-delivery.md). Every table, column and function name below is
a proposal.

## 1. No shared transaction, so order is the protocol

PostgreSQL and object storage cannot commit together. Correctness comes from ordering and
idempotency, not from a distributed transaction:

```text
manifest first  → the pure description of exactly which bytes the release needs
bytes next      → staged, promoted and verified against that manifest
pointer last    → one PostgreSQL transaction makes the release active
effects after   → outbox consumers update discovery and caches, tolerating repeats
```

| Moment | What a listener can observe | Why it is safe |
|---|---|---|
| staging or promotion in progress | the prior active release only | no listener-visible record references the new keys |
| verified, commit not yet run | the prior active release only | the pointer has not moved |
| inside the commit transaction | the prior active release only | uncommitted rows are invisible |
| after commit, before outbox dispatch | the new release through the listener API | every asset was verified before commit; only discovery indexes lag |

The last row is why bytes come before the pointer. If a deployment requires a post-commit step
to make objects reachable (for example copying to a public key after commit), the pointer no
longer implies deliverability. Avoid that design; otherwise the readiness facts this skill
exposes to the listener's active-release resolver must include an acknowledged activation, and
the prior release must stay visible until it arrives.

## 2. The release lifecycle is a closed state machine

```text
Draft ──BeginStaging──▶ Staging ──VerificationPassed──▶ Verified ──MarkReady──▶ Ready ──Activate──▶ Active
  │                      ▲   │                            │                       │                 │
  │                      └───┴──VerificationFailed────────┘                       │       Supersede │ Retract
  └──────────Abandon (from Draft, Staging, Verified or Ready)─────────▶ Abandoned │                 ▼
                                                                                  │   Superseded ──Retract──▶ Retracted
```

```rust
// Illustrative, proposed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReleaseState { Draft, Staging, Verified, Ready, Active, Superseded, Retracted, Abandoned }

pub enum ReleaseEvent {
    BeginStaging,
    VerificationPassed(VerificationReportId),
    VerificationFailed(VerificationReportId),
    MarkReady(GatePassed),              // private-field token only `PublishableRelease` yields
    Activate,
    Supersede { by: ReleaseId },
    Retract { reason: RetractionReason },
    Abandon,
}

pub enum Step { To(ReleaseState), Unchanged }

pub fn release_transition(from: ReleaseState, event: &ReleaseEvent) -> Result<Step, IllegalTransition> {
    use ReleaseEvent as E;
    use ReleaseState as S;
    match (from, event) {
        (S::Draft, E::BeginStaging) => Ok(Step::To(S::Staging)),
        (S::Staging, E::VerificationPassed(_)) => Ok(Step::To(S::Verified)),
        (S::Staging | S::Verified, E::VerificationFailed(_)) => Ok(Step::To(S::Staging)),
        (S::Verified, E::MarkReady(_)) => Ok(Step::To(S::Ready)),
        (S::Ready, E::Activate) => Ok(Step::To(S::Active)),
        (S::Active, E::Supersede { .. }) => Ok(Step::To(S::Superseded)),
        (S::Active | S::Superseded, E::Retract { .. }) => Ok(Step::To(S::Retracted)),
        (S::Draft | S::Staging | S::Verified | S::Ready, E::Abandon) => Ok(Step::To(S::Abandoned)),
        (S::Staging, E::BeginStaging)
        | (S::Active, E::Activate)
        | (S::Retracted, E::Retract { .. })
        | (S::Abandoned, E::Abandon) => Ok(Step::Unchanged),
        (from, event) => Err(IllegalTransition { from, event: event.kind() }),
    }
}
```

- **No backward edge.** A superseded or retracted release never becomes active again. Rollback
  is a *new* release that references an earlier manifest (section 6), so history stays linear
  and every activation passes the gate at the time it happens.
- **`MarkReady` carries `GatePassed`**, a token with a private field that only
  `PublishableRelease` hands out, so the state machine cannot reach `Ready` without the gate.
- **Repeats are explicit.** A duplicate `Activate` on `Active` is `Unchanged`, which keeps
  at-least-once delivery harmless; any other unlisted pair is an `IllegalTransition` naming
  both sides.
- **Persistence.** The release row (`id`, `episode_id`, `candidate_id`, `manifest_digest`) is
  written once at `Draft`. State lives in append-only `release_transitions`; a cached
  `current_state` column is updated in the same transaction only through `release_transition`.
  A partial unique index on active releases per episode is a candidate second barrier.
- **Ready without Active** exists for a scheduled publication time. Immediate publication
  records `Verified → Ready → Active` in one transaction.

Invariants for a state-machine property test over generated event sequences per episode: at most
one `Active` release; the active pointer references a release whose state is `Active`; terminal
states (`Retracted`, `Abandoned`) accept only their idempotent repeat; every `Active` release has
a passed verification report for its manifest digest.

## 3. The manifest is assembled first and never changes

A pure `assemble_manifest(&ReleaseCandidate, &ClearedRights) -> ReleaseManifest` builds the
manifest from accepted artifacts. Keys are derived from checksums
([keys](storage-and-delivery.md#2-keys-are-immutable-and-derived-from-content)), and codec,
duration and loudness come from the artifact metadata QC measured on the identical bytes.
Verification then checks that the delivery path serves exactly what the manifest says.

```json
{
  "manifest_schema": "0.1.0-draft",
  "episode_id": "episode-demo-01",
  "title": "Một lời hẹn",
  "script_revision": { "id": "episode-demo-01@3", "digest": "sha256:<hex>" },
  "renditions": [
    {
      "role": "stream",
      "asset_id": "asset-<id>",
      "storage": { "class": "publication", "key": "releases/episode-demo-01/sha256-<hex>.m4a" },
      "sha256": "<hex>",
      "size_bytes": 18734211,
      "codec": "<pending format decision>",
      "duration_ms": 1145230
    }
  ],
  "credits": [{ "role": "Người dẫn chuyện", "credit": "<voice credit required by the grant>" }],
  "provenance": {
    "production_run_id": "run-<id>",
    "qc_result_ids": ["qc-<id>"],
    "approval_ids": ["approval-<id>"],
    "rights_record_ids": ["rights-<id>"]
  }
}
```

Illustrative only; the codec, container and schema are open decisions.

- **The release ID is not inside the manifest.** A release references its manifest by digest,
  which lets rollback reuse an earlier manifest and lets a republish of identical content be
  detected as a no-op.
- **Canonical bytes, one digest.** Follow the canonicalization policy in
  [`cantos-script-ir`](../../cantos-script-ir/references/canonical-digest.md) rather than
  inventing a second one; pin it with a golden-fixture test of the exact bytes and digest.
- **Never inside a manifest:** signed URLs, bucket names or endpoints, credentials, rights
  evidence keys, provider request payloads, private source text, intermediate dialogue keys.
- **The listener projection is not owned here.** Clients receive an allow-listed projection
  owned by [`cantos-listening`](../../cantos-listening/references/listener-api.md#listener-visibility);
  storage keys, provenance, provider identifiers and QC findings never cross it. This skill's
  obligation is the other side: version the manifest schema and flag every new field in review,
  so the allow-list is a decision rather than an accident.

## 4. The protocol, step by step

| Step | Shell action | Durable record | Crash here → recovery | Repeat safety |
|---|---|---|---|---|
| S1 assemble | build candidate and manifest from accepted artifacts | release row in `Draft` with `manifest_digest`; manifest row | nothing external happened | candidate unique by identity digest |
| S2 stage | ensure each rendition exists at a private key with a verified checksum — usually the pipeline's accepted artifact; upload only when absent ([uploads](storage-and-delivery.md#4-uploads-checksums-and-ambiguous-outcomes)) | upload attempt row *before* any request | resume or abort the recorded multipart upload | conditional create; existing key with equal checksum is success |
| S3 promote | server-side copy to the publication-class key once the only remaining gate blockers are asset blockers | promotion attempt row | repeat the copy | same key, same bytes |
| S4 verify | probe every rendition through the intended delivery path ([probe](storage-and-delivery.md#6-verify-through-the-delivery-path)) | immutable verification report; `VerificationPassed` or `VerificationFailed` | rerun the probe | a report per attempt; latest by sequence |
| S5 commit | the transaction in section 5 | transitions, pointer, outbox, idempotency key | before `COMMIT`: nothing changed; after: outbox redelivers | publish idempotency key; expected-pointer check |
| S6 consume | discovery index, catalog cache purge, notifications | processed-event row per consumer | redelivery | dedupe by event ID |

Staging and promotion are durable jobs with leases and fencing tokens; a worker whose lease
expired cannot record an upload or verification result for a newer attempt. The lease, fencing
and outbox-dispatch mechanics are owned by
[durable jobs](../../cantos-production-pipeline/references/durable-jobs.md); this protocol only
names the records and their order.

The S3 precondition ("only asset blockers remain") keeps unapproved bytes out of the
publication class. Under signed delivery it is a belt-and-braces rule; under public delivery it
is what stops an unapproved render from ever sitting at a public key.

## 5. The commit transaction

```text
-- Proposed shape; table names are illustrative.
BEGIN;                                             -- isolation per the rights TOCTOU ADR
SELECT active_release_id, pointer_version
  FROM episode_publication WHERE episode_id = $episode FOR UPDATE;
-- active_release_id ≠ $expected_active          → ROLLBACK, PublicationConflict
-- idempotency key already recorded              → ROLLBACK, return the recorded result
-- re-read rights ledger, QC results, approvals and the latest verification report
-- decide_publication(&facts) in Rust: Err(blockers) → ROLLBACK, return every blocker
INSERT INTO release_transitions …  -- new: verified→ready, ready→active; old: active→superseded
UPDATE episode_publication
   SET active_release_id = $new, pointer_version = pointer_version + 1
 WHERE episode_id = $episode;
INSERT INTO outbox (event_id, kind, release_id, …) VALUES ($event, 'release_activated', $new, …);
INSERT INTO publish_requests (idempotency_key, release_id) VALUES ($key, $new);
COMMIT;
```

- **Re-decide inside.** The facts that authorize the commit are read after the lock, so an edit,
  QC result, approval withdrawal or rights revocation committed earlier is seen
  ([TOCTOU](rights-and-provenance.md#4-two-checks-one-evaluator-the-second-inside-the-commit)).
- **Expected pointer.** The request names the active release the creator saw. Two concurrent
  publishes of different candidates cannot both win; the loser gets a typed conflict, not a
  silent overwrite.
- **Identical content.** When the candidate's manifest digest equals the active release's, the
  decision is a no-op returning the active release.
- **Transactional rejection.** Every `Err` path rolls back before any write is visible; the test
  compares the release, transition, pointer and outbox tables before and after
  ([functional core](../../cantos-engineering/references/functional-core.md)).

## 6. Republish, rollback and retraction

| Operation | Mechanism | Prior release | Required evidence |
|---|---|---|---|
| republish | new candidate → new release through S1–S6 | stays `Active` until the new commit; then `Superseded`, still deliverable within retention | a failure at every step leaves the listener API returning the prior release, whose assets still fetch |
| rollback | a new release whose `manifest_digest` is an earlier one; S2 finds objects present, S4 re-verifies them, the gate re-runs at `now` | the bad release becomes `Superseded` | rollback fails loudly if retention removed an object or a grant expired |
| retract | `Active` or `Superseded` → `Retracted`; pointer cleared in the same transaction; outbox removes discovery entries and purges catalog caches | — | the listener API stops serving it immediately; delivery stops per mode (below) |

Retraction latency depends on the delivery mode recorded in the ADR. Under signed delivery the
backend stops minting URLs at commit and outstanding URLs die within their lifetime; under
public delivery, objects must be deleted or blocked at the origin and purged from the CDN, and
cached copies may outlive the purge. Report the bound you measured. Previously downloaded copies
cannot be remotely erased; the offline revalidation behavior belongs to
[`cantos-listening`](../../cantos-listening/references/downloads-contract.md).

## 7. Outbox consumers

| Consumer | Effect | Idempotency |
|---|---|---|
| discovery index | upsert or remove the episode's catalog entry | keyed by episode and pointer version; an older version never overwrites a newer one |
| catalog cache purge | purge cached catalog responses, never content-addressed audio | purging twice is harmless |
| notifications (future) | tell followers about a new episode | processed-event row in the consumer's own transaction |

A consumer failure never moves the pointer back and never fails the publish request. The pointer
was already the truth when the transaction committed.

## 8. Reconciliation

Metadata and delivery can drift: an object deleted by hand, an expired grant, an abandoned
multipart upload, a stuck outbox row. Detect drift with a pure function over two snapshots and
let the shell act on the result:

```rust
// Illustrative, proposed.
pub fn reconcile(expected: &DeliveryExpectations, observed: &StorageObservations) -> Vec<Discrepancy>
```

| Discrepancy | Proposed action (data, executed by the shell) |
|---|---|
| `ActiveAssetMissing { release, key }` or `ActiveAssetChecksumMismatch` | alert; `RepairFromPrivateCopy` re-copies bytes whose checksum matches the manifest |
| `ActiveAssetNotDeliverable { release, probe }` | alert; re-run the delivery probe |
| `RetractedStillDeliverable { release, key }` | alert; re-run the retraction effects |
| `OrphanPublicationObject { key }` | `ScheduleGc { after: grace }`, only if no release in any retained state references it |
| `StaleStagingUpload { upload_id, age }` | abort the multipart upload |
| `OutboxBacklog { oldest_age }` | alert past a measured threshold |
| `PointerToNonActiveRelease` | page a human: a database invariant is broken |

Reconciliation never moves the pointer and never deletes a referenced object; rollback and
retraction are human decisions through the gate. Test the pure function with seeded snapshots,
then run it against a real store with one object deliberately deleted.

## Fault injection: named crash points

Use the harness in [fault injection](../../cantos-production-pipeline/references/fault-injection-testing.md);
these are the publication points it must cover. After **every** fault, the oracle is the same:
ask the listener API for the episode, fetch every asset of the release it returns through the
delivery path and compare sizes and checksums with that release's manifest.

| Point | Injection | Required outcome |
|---|---|---|
| `stage.after_part` | kill the worker after part *k* of a multipart upload | resumed or restarted upload; final checksum equals the manifest; one asset row |
| `stage.after_complete_before_record` | crash after the store completed the upload, before the row is written | recovery finds the key with an equal checksum and records it without re-uploading |
| `verify.object_missing` | delete one promoted object before the probe | `VerificationFailed`; `AssetUnverified` blocker; the prior release unchanged |
| `commit.before_commit` | abort after the pointer update, before `COMMIT` | nothing changed; a retry commits exactly once |
| `commit.after_commit_before_dispatch` | kill the process immediately after `COMMIT` | the listener API already shows the new release; the outbox event is delivered after restart |
| `consume.after_effect_before_ack` | deliver the same event twice | one index entry; a second purge is harmless |
| `republish.verify_fails` | the replacement's object is missing | the prior release stays active and every one of its assets fetches |
| `publish.duplicate_request` | the same idempotency key twice; two candidates racing on one expected pointer | one activation; the duplicate returns the first result; the racer gets `PublicationConflict` |
| `storage.put_timeout` | the PUT times out after the store committed | the attempt is recorded as ambiguous, then resolved by a checksum `HEAD`; no second object |
| `lease.expired_worker` | a stale staging worker finishes after reassignment | its write is fenced; the newer attempt's record stands |

## Tests

| Test | Level |
|---|---|
| `every_state_event_pair_matches_the_transition_table` (oracle row lookup is an exhaustive `match`, so a new state fails to compile) | `example-tested` |
| `generated_event_sequences_never_produce_two_active_releases` | `property-tested` |
| `manifest_canonical_bytes_match_the_golden_fixture` | `differentially-tested` |
| `listener_projection_contains_no_storage_or_provenance_fields` | `example-tested` |
| `failed_republish_keeps_prior_release_playable` via the listener API and MinIO | `fault-injected` + `integration-tested` |
| `commit_with_stale_expected_pointer_is_rejected_without_writes` on real PostgreSQL | `integration-tested` |
| `reconcile_reports_seeded_missing_object_of_active_release` | `example-tested`, then `integration-tested` |

## Gotchas

- "Toggle `published = true`" is not a protocol: the docs reject a database flag alone when
  assets may not be deliverable.
- A verification report proves the bytes at probe time. Rollback and reconciliation re-probe;
  they never trust a report from last month.
- Do not delete the prior release's objects in the republish transaction or its outbox. Removal
  is a retention decision ([retention](storage-and-delivery.md#8-retention-protects-playback-in-progress)).
- Scheduled publication (`Ready` with a future time) re-runs the gate at activation time; rights
  and approvals can change while it waits.
