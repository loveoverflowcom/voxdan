# Approvals and publication gates

> **Scope.** Model approvals as immutable records bound to an exact release candidate, compute
> staleness purely, decide publication in one pure gate that returns every blocker, seal the
> result in a witness only that gate can build, enforce permissions in the backend and expose
> blockers to the Studio as typed "disabled with reason" data. Use when adding an approval kind,
> a QC or rights gate, a publish/republish endpoint, a readiness view or anything that changes
> what an approval covers.

Owning rules:
[business rules § Review and publication](../../../../docs/product/business-rules.md#review-and-publication)
(QC scope, approval binding and invalidation, publication preconditions, release completeness),
rule 13 in [§ Rights and access](../../../../docs/product/business-rules.md#rights-and-access)
(roles and recorded actor/scope) and
[pipeline § Audio processing and QC](../../../../docs/product/production-pipeline.md#audio-processing-and-qc)
(QC results identify the artifact checksum and input revision they evaluated). The
[UI system](../../../../docs/design/ui-system.md#components-and-interaction-states) requires
disabled actions to carry a discoverable reason. Names below are proposals.

## 1. What an approval binds

An approval is written once and never updated. A re-approval, a rejection and a withdrawal are
new records.

```rust
// Illustrative, proposed: no `publication::approval` module exists yet.
pub struct Approval {
    id: ApprovalId,
    kind: ApprovalKind,
    actor: ActorId,
    granted_under: PermissionScope,     // the role/permission the actor held, snapshotted
    subject: CandidateIdentity,         // a value copy, not a foreign key to a mutable row
    decision: ApprovalDecision,
    sequence: LedgerSequence,           // server-assigned order; never a client clock
}

pub enum ApprovalKind {
    RightsReview,
    QcAcceptance,
    PublicationApproval,
}

pub enum ApprovalDecision {
    Approved,
    Rejected { reason: ReviewNote },
    Withdrawn { of: ApprovalId },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateIdentity {
    pub episode: EpisodeId,
    pub script: ScriptRevisionRef,              // revision ID + canonical content digest
    pub production_inputs: RunInputDigest,      // casting, voices, pronunciation, sound assets,
                                                // mix profile, approval policy revision
    pub renders: BTreeMap<RenditionRole, ArtifactChecksum>,
    pub qc_results: BTreeSet<QcResultId>,
    pub metadata: MetadataDigest,               // title, description, credits, artwork shown
}
```

- The script digest comes from [`cantos-script-ir`](../../cantos-script-ir/references/canonical-digest.md);
  the run input digest and QC result IDs come from
  [`cantos-production-pipeline`](../../cantos-production-pipeline/SKILL.md). Publication stores
  them as values and never recomputes them differently.
- `production_inputs` is the catch-all that makes a recast, a pronunciation change or a replaced
  sound asset stale even if the bytes happened to match. `renders` catches a replacement render
  from unchanged inputs. Both are needed.
- QC results are keyed to artifact checksums. A scene mix reused unchanged keeps its QC result as
  valid evidence; the episode-level approvals bound to the old candidate still go stale, because
  the candidate changed. Do not over-invalidate the evidence or under-invalidate the approval.
- One authorized creator may hold several roles. Record one approval per kind even when the
  actor is the same person; the gate checks each required kind, never "some approval exists".
- The docs do not say whether the publication-approval subject covers metadata (title,
  description, artwork). The checklist reviews them, so this proposal binds `metadata`; record
  the decision.

## 2. Staleness is a pure comparison

```rust
// Illustrative, proposed.
pub fn approval_is_current(
    approval: &Approval,
    candidate: &ReleaseCandidate,
) -> Result<(), Vec<StaleReason>> {
    let changes = approval.subject.changes_to(&candidate.identity());
    if changes.is_empty() { Ok(()) } else { Err(changes) }
}

impl CandidateIdentity {
    fn changes_to(&self, current: &CandidateIdentity) -> Vec<StaleReason> {
        // No `..`: a new field fails to compile until staleness accounts for it.
        let CandidateIdentity { episode, script, production_inputs, renders, qc_results, metadata } =
            self;

        let mut changes = Vec::new();
        if episode != &current.episode {
            changes.push(StaleReason::DifferentEpisode);
        }
        if script != &current.script {
            changes.push(StaleReason::ScriptRevisionChanged {
                approved: script.clone(),
                current: current.script.clone(),
            });
        }
        if production_inputs != &current.production_inputs {
            changes.push(StaleReason::ProductionInputsChanged);
        }
        changes.extend(render_changes(renders, &current.renders));
        changes.extend(qc_changes(qc_results, &current.qc_results));
        if metadata != &current.metadata {
            changes.push(StaleReason::MetadataChanged);
        }
        changes
    }
}
```

- **Never store `is_stale`.** A stored flag must be flipped by every edit, recast, upload and
  rerender path, and the next path written forgets. Computing from two immutable values has no
  such obligation. The Studio's "stale" badge calls the same function through the API.
- The exhaustive destructure without `..` is the cheapest `statically-checked` guarantee that a
  new bound input cannot be ignored silently.
- Decision order per `(kind, subject)` is the server `sequence`. The latest decision wins; a
  `Withdrawn` or `Rejected` after an `Approved` leaves no current approval of that kind.
- Whether an approval survives its actor later losing the permission is not decided by the docs.
  Until it is, the gate re-checks that `granted_under` is still held and blocks otherwise (fail
  closed). Record the question.

## 3. One pure gate returns every blocker

```rust
// Illustrative, proposed.
pub struct PublicationFacts {
    pub candidate: ReleaseCandidate,
    pub rights_ledger: RightsLedger,
    pub intended_use: IntendedUse,
    pub qc: Vec<QcResult>,
    pub approvals: Vec<Approval>,
    pub policy: ApprovalPolicy,                 // the revision pinned by the frozen inputs
    pub held_scopes: HeldScopes,                // approvers' current permissions
    pub run: RunStatus,                         // the producing run's ID and state
    pub assets: BTreeMap<RenditionRole, AssetObservation>,
    pub active_release: Option<ReleaseId>,
}

pub fn decide_publication(
    facts: &PublicationFacts,
) -> Result<PublicationDecision, Vec<PublicationBlocker>> {
    let required = facts.candidate.required_subjects();
    let rights = evaluate_rights(&required, &facts.rights_ledger, &facts.intended_use);
    let blockers = collect_blockers(facts, &rights);

    match rights {
        Ok(cleared) if blockers.is_empty() => Ok(PublicationDecision {
            release: PublishableRelease::seal(facts, cleared),
            supersedes: facts.active_release,
        }),
        _ => Err(blockers),
    }
}

fn collect_blockers(
    facts: &PublicationFacts,
    rights: &Result<ClearedRights, Vec<RightsBlocker>>,
) -> Vec<PublicationBlocker> {
    let blockers: BTreeSet<PublicationBlocker> = [
        rights_blockers(rights),
        attribution_blockers(rights, &facts.candidate),
        run_blockers(&facts.run),
        qc_blockers(&facts.qc, &facts.candidate),
        approval_blockers(&facts.approvals, &facts.policy, &facts.held_scopes, &facts.candidate),
        asset_blockers(&facts.assets, &facts.candidate),
        provenance_blockers(&facts.candidate),
    ]
    .into_iter()
    .flatten()
    .collect();

    blockers.into_iter().collect()
}
```

| Blocker family | Variants (each carries the IDs needed to act) |
|---|---|
| production run | `RunNotCompleted { run, state }` — a cancelled, failed or unfinished run blocks everything it produced |
| rights | `Rights(RightsBlocker)` from [rights evaluation](rights-and-provenance.md#3-evaluation-is-pure-total-and-returns-every-blocker) |
| QC | `QcMissing { check, artifact }`, `QcBlockingFinding { finding, artifact }`, `QcEvaluatedOtherArtifact { result, evaluated, current }` |
| approvals | `ApprovalMissing { kind }`, `ApprovalStale { approval, reasons }`, `ApprovalRejected { approval }`, `ApprovalWithdrawn { approval }`, `ApproverScopeLost { approval }` |
| assets | `AssetUnverified { role, observed }`, `AssetChecksumMismatch { role, expected, observed }`, `AssetMissingFromManifest { role }` |
| credits | `AttributionMissing { grant }` |
| provenance | `ProvenanceMissing { artifact }` |

- **No short circuit.** Each family is a small function returning its blockers; the gate
  concatenates them. A `?` chain shows the creator one reason per click.
- **Deterministic order.** Collecting into a `BTreeSet` sorts and deduplicates, so readiness
  responses and snapshot tests are stable.
- **Facts are observations.** `assets` carries `AssetObservation::{Verified, Unverified,
  Missing, ChecksumMismatch}` from the shell's delivery probe, so the gate can explain an
  unverified asset instead of the type system hiding it.
- **The gate does not authorize.** Who may publish is a separate check ([section 4](#4-the-witness-and-the-permit)). Keeping them
  apart lets the readiness view show blockers to a reviewer who cannot publish.
- The gate does not require the candidate to be built from the newest script revision; it
  requires the candidate to be internally consistent and approved. The publish request names the
  candidate explicitly, so nothing publishes a revision nobody reviewed.

## 4. The witness and the permit

```rust
// Illustrative, proposed. Fields private; no Default, Clone, Serialize, Deserialize or FromRow.
pub struct PublishableRelease {
    candidate: ReleaseCandidateId,
    manifest: ReleaseManifest,
    gates: GateEvaluation,              // what was evaluated, for the audit row and the report
}

impl PublishableRelease {
    fn seal(facts: &PublicationFacts, rights: ClearedRights) -> Self {
        // private: only `decide_publication` reaches this
    }
}

pub fn authorize_publish(
    actor: &Actor,
    episode: EpisodeId,
    permissions: &PermissionSet,
) -> Result<PublishPermit, Forbidden>;

pub async fn commit_publication(
    tx: &mut Transaction<'_, Postgres>,
    permit: PublishPermit,
    release: PublishableRelease,
    expected_active: Option<ReleaseId>,
) -> Result<ActivatedRelease, CommitError>;
```

- `commit_publication` consumes both witnesses by value, so the signature itself says "gated and
  authorized", and a witness cannot be reused for a second commit.
- A witness is never persisted or reloaded: a stored proof is stale by definition. It is built
  from facts read inside the commit transaction and dies with it.
- Oracle: a compile-fail test (trybuild is a candidate) that constructs `PublishableRelease { .. }`
  or calls `seal` from outside the module. Label: `type-enforced`.
- `PublishPermit` comes from backend authorization at the handler
  ([HTTP boundary](../../cantos-engineering/references/http-api-boundary.md)). A hidden or
  disabled Studio button is presentation, never enforcement.

## 5. Readiness for the Studio: typed reasons, localized by the UI

The readiness endpoint runs the same gate and returns stable codes with parameters, never
English prose. The Studio renders the localized sentence and keeps "Publish episode" disabled
with that reason; the POST runs the gate again.

| Blocker | Stable code (contract) | Illustrative Studio copy (owned by UI localization) |
|---|---|---|
| `ApprovalStale { ScriptRevisionChanged }` | `approval.stale.script_revision` | Bản duyệt đã cũ: lời thoại "Ngày mai, mình có diễn tiếp không?" đã được sửa sau khi duyệt. |
| `Rights(Expired)` for music | `rights.expired` | Quyền sử dụng nhạc nền đã hết hạn. |
| `QcBlockingFinding` | `qc.blocking_finding` | Còn lỗi QC chặn xuất bản ở cảnh "Sau buổi diễn". |
| `AssetUnverified` | `asset.unverified` | Tệp âm thanh chưa được xác minh qua đường phân phối. |

Codes are a versioned API contract: renaming one breaks Leptos and CMP clients. Final copy and
its accessibility belong to [`cantos-ui-design`](../../cantos-ui-design/references/component-states.md)
and [localization](../../cantos-ui-design/references/localization.md).

## 6. Permissions are actions on resources

| Action | Witness or record | Notes |
|---|---|---|
| record or review rights | `RightsReview` row with actor and scope | reviewer scope snapshotted |
| accept QC | `Approval { kind: QcAcceptance }` | binds QC result IDs |
| approve publication | `Approval { kind: PublicationApproval }` | binds the full identity |
| publish, republish, rollback | `PublishPermit` | separate from approving |
| retract | `RetractPermit` | its own action; retraction is not a publish |

Authorization is decided in the application layer from typed actor, resource and action
values, and every refusal is a typed `Forbidden` with no state change.

## Rule cards

| Rule | Why | Good | Counterexample | Oracle | Status · exception |
|---|---|---|---|---|---|
| Approvals bind identity by value | a new render inherits an old approval | `subject: CandidateIdentity` | `approvals(episode_id, approved bool)` | staleness property under Tests | proposed · none |
| Staleness is computed | stored flags drift | `approval_is_current` | `UPDATE approvals SET stale = true` in edit handlers | property + destructure compile check | proposed · a cached projection rebuilt from the function, never authoritative |
| All blockers, sorted | partial explanations | `BTreeSet` of every family | `qc_ok()?; rights_ok()?;` | all-defects row | proposed · none |
| Only the gate seals | bypassed gate | private `seal` | `PublishableRelease::new(..)` public | compile-fail test | proposed · none |
| Re-decide at commit | TOCTOU | gate on in-transaction facts | commit trusting the preview | interleaving test | proposed · none |
| Backend authorizes | client-side enforcement | `PublishPermit` parameter | handler checks `request.can_publish` | direct POST by a non-publisher → `Forbidden`, no rows | proposed · none |

## Tests

| Test | Shape | Level |
|---|---|---|
| one row per blocker variant | `publishable_baseline()` (episode "Một lời hẹn" of "Ánh đèn cuối sân khấu") plus exactly one defect; assert `Err(vec![that_blocker])` | `example-tested` |
| no short circuit | every defect at once; assert the full sorted list | `example-tested` |
| variant coverage | the row builder `match`es a `BlockerKind` with no wildcard and iterates an `ALL` list | `statically-checked` + `example-tested` |
| gate truth table | enumerate every combination of pass/fail over the gate dimensions (e.g. 2^8); oracle: publishable iff all pass, blocker families equal the failing set | `differentially-tested` (domain and bound stated) |
| staleness | `any_change_to_an_approved_input_makes_the_approval_stale`: generate an identity, approve it, apply one generated mutation (script, inputs digest, one render added/removed/changed, QC set, metadata); assert `Err` with the matching `StaleReason` | `property-tested` |
| no false staleness | an identity rebuilt from the same values in a different insertion order is current | `property-tested` |
| transactional rejection | POST publish with blockers: typed 409 with the same codes as readiness; release, pointer and outbox tables byte-identical before and after | `integration-tested` on real PostgreSQL |
| TOCTOU | edit dialogue-02 after the readiness preview, then commit: rejected with `ApprovalStale` | `integration-tested` |
| assertion strength | mutation run over `decide_publication` and `approval_is_current` (cargo-mutants is a candidate) | `mutation-tested` |

## Gotchas

- QC "passed" without the evaluated artifact checksum is not evidence for this candidate; join
  on checksum, not on episode.
- A rejected approval is a fact the creator needs to see; do not hide it behind "missing".
- Sorting blockers by display text breaks when the locale changes; sort by the typed value.
- An approval row that shows only who approved and when must also show *what* was approved —
  the script revision, render and QC results — or reviewers approve the wrong thing.
