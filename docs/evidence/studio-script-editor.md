# Studio Script Editor evidence

Date: 2026-10-10. Review boundary: [issue #4](https://github.com/loveoverflowcom/cantos/issues/4)
and [010 — Import and edit a versioned script](../work-plan/010-import-and-edit-script.md).
Base: `297f6fdb42568819d48c507ec09713667b3a6659` on `develop`; branch
`feat/studio-script-editor`. Final implementation/PR/merge SHAs are pending executor completion.

This is a **draft execution record**. Native and real PostgreSQL evidence is available below.
The context-owner, selected-option, field-buffer/read-race and transport-deadline corrections
are implemented in the current tree. Final browser acceptance, reviewed screenshots, final-tree
check counts and merge evidence remain executor-owned. A successful build is not browser
evidence; the issue is not claimed complete by this draft.

## Invariant, owner and boundary

A creator can edit a complete Script IR draft through structured controls and explicitly accept
or save a new immutable revision without altering its original source, frozen proposal or prior
accepted revisions. Denied, stale, repeated or interrupted actions preserve local text and the
exact unresolved operation. Navigation, pane changes and late responses must not substitute
another actor's, script's or editor's content.

`authoring.rs` is a pure Studio boundary module. Its read models are projections for rendering;
mutations patch known leaves of the complete JSON value. Stable IDs, provenance, rights, cue
assets and unsupported fields survive unrelated edits. Duplicate JSON keys, unsupported schema
versions and numbers that cannot round-trip without precision loss are rejected before a
structured edit. Unsupported data remains available in the advanced JSON draft; preserving an
unknown field does not make it admissible to the current backend contract.

Losslessness here means preserving the complete decoded JSON value except the requested leaf
edit or explicit entity insertion/removal. JSON whitespace, key order and equivalent numeric
notation can change when serialized; original source-byte identity is a separate backend
invariant. The focused oracle adds unknown nested metadata, edits one Vietnamese line, and
compares the entire document against an independently patched expected value. Separate checks
retain the cue's asset/rights record, every other pronunciation entry, original character/line
IDs and provenance while editing delivery or cue descriptions. A domain read/save can still
canonicalize supported text under the unchanged Script IR rules; save acknowledgements do not
rewrite an active browser text field.

The core supports metadata, characters, acts/scenes, dialogue/narration, delivery/emotion,
pronunciation and typed ambience/music/SFX cues. Narration remains dialogue referencing the
explicit narrator character. New entities receive permanent IDs from the shell; reordering
preserves complete values and references. Referenced characters and cue-anchored dialogue cannot
be deleted without an explicit preceding correction. Bounded structural undo/redo retains 40
immutable snapshots. Local checks protect safe editing structure; server admission remains the
only Script IR validation authority.

`editor.rs` owns actor-bound local drafts, dirty/clean baselines, immutable save intents,
optimistic bases, read tickets and acknowledgement/error transitions. `InputActivity` tracks
each buffered or composing field separately, so resolving one input cannot hide another.
`inspector.rs` owns script/head-scoped history/source reads and exact saved-revision owner-review
intents. Proposal acceptance continues through the existing adaptation reducer and transaction.
Leptos renders and dispatches; `api.rs` owns URLs/transport and shared wire types remain in
`contracts/api`. The server/domain crate does not enter the browser dependency graph.

Each structured editor now owns a scoped Leptos `Provider` for its `AuthoringContext`. Proposal
and accepted-revision editors stay mounted with separate document, selection, history, pending
fields and namespace ownership. Dynamic select choices set `option.selected` from the model
after constructing their options, including an explicit unresolved current value. Opening a
document must therefore show its actual speaker/role/emotion/cue choice without silently
selecting the browser's first option or editing the model.

The shell passes current buffered-input activity into revision/proposal/accepted-revision read
completion. A response arriving before blur or composition commit can update compared remote
facts but cannot replace local text/baseline/base while those bytes are pending. Account/script
read tickets and saved-head inspector contexts fence late responses; review/acceptance
acknowledgement is cleared when text, buffered activity or the compared base changes. Native
state regressions cover these transitions; the exact browser event order remains a separate
walkthrough gate.

The additive authenticated `POST /api/v1/validation` previews the existing bounded reader's
shape and semantic findings. It requires the shared session/Origin checks, creates no database
records, and proves neither target permission nor source/rights eligibility. Save and acceptance
still revalidate current authority, evidence and optimistic base in the existing PostgreSQL
transaction. Diagnostic focus maps both current JSON positions and semantic paths containing
stable act/scene/dialogue/cue IDs; reordering does not change the semantic identity.

`api.rs` applies a nominal 15-second browser `AbortController` deadline to each request through
receipt of headers and decoding of the complete JSON body. Completion or future cleanup clears
the timer and aborts the owned fetch. Expiry/transport/decode failure is `unavailable`; a possible
write keeps its existing immutable actor/operation/base/body intent for explicit reconciliation.
The transport neither retries automatically nor interprets abort as proof of server rollback.
The browser deadline is distinct from the existing server middleware deadline and is not a
production timeout/SLA measurement.

No migration, persistence authority, inference runtime, AI provider client or paid call is added.
The user-controlled Gemini/ChatGPT/Codex host creates proposals through the earlier tools; this
slice opens and edits those proposals. Synthetic proposals below establish integration behavior,
not model quality.

## Claim and oracle ledger

| Claim / failure mode | Owner and cheapest adequate oracle | Evidence held / limit |
| --- | --- | --- |
| Structured edits cannot discard unrelated fields or rewrite IDs | Pure document patches; exact complete-value comparison against committed Vietnamese fixtures | PASS, example-tested; unsupported unknown fields still fail backend admission |
| Reordering and structural undo preserve entity identity and content | ID-based move comparison; delete/undo exact snapshot regression | PASS, example-tested; real focus/IME behavior still requires browser input |
| A pending field cannot disappear when another field commits/cancels, or be overwritten by a completed read | Pure per-field activity registry and buffered revision/proposal read regressions; actual shell wiring inspected | PASS, example-tested for held field-registry result; read guards implemented, final-tree regression and browser event order results executor-owned |
| Validation points to the same entity after reorder | Current contract `cases.json` semantic paths and current-position pointer assertions | PASS, example-tested; final DOM focus result pending |
| Preview validates without persistence or approval | Real authenticated HTTP, hostile/invalid input, Origin/session/bounds checks and unchanged database counts | PASS, integration-tested; preview conveys no target ownership |
| Dirty text, stale responses and changed actors cannot replace the current draft | Editor/inspector/adaptation state transitions and immutable request snapshots | PASS, example-tested; live navigation/reload confirmation pending |
| Stale writes and repeated acceptance/save/review preserve history | Fresh PostgreSQL HTTP journey, exact replay receipts, current-head conflict and row counts | PASS, integration-tested; no automatic merge is claimed |
| Reader/editor/owner permissions remain backend decisions | Existing real authorization suites plus reader save/review rejection in editor journey | PASS, integration-tested; clear browser explanations pending |
| Source, proposal and old accepted revisions remain immutable | Independent exact response/body/hash comparisons before/after acceptance, save and reopen | PASS, integration-tested; original synthetic fixture only |
| Process loss and PostgreSQL restart preserve settled facts | HTTP process kill, new host, PG restart, migration replay and exact-intent replay | PASS, fault-injected and integration-tested; not backup restore/failover evidence |
| The three-pane editor works on desktop/narrow/zoom with Vietnamese text and keyboard input | Actual Safari walkthrough, opened captures, focus/cancel/navigation observations | NOT_RUN as a completed gate; walkthrough in progress |
| Multiple structured editors have isolated context ownership | Scoped `Provider` correction after observed collision; same-scenario live rerun | IMPLEMENTED correction; final interaction result pending executor record |
| Native select state displays the current model without a silent edit | Explicit selected-option projection, including unresolved values; native projection regressions present | IMPLEMENTED correction; final regression/browser speaker/emotion/cue results executor-owned |
| A hung or incomplete HTTP reply releases UI transport while preserving ambiguous write identity | Request deadline ownership through body decoding; existing immutable retry reducers | IMPLEMENTED 15-second deadline; live timeout/body-interruption result not claimed here |

## Acceptance matrix

| Acceptance step | Implemented route/behavior | Core / real backend | Final browser result |
| --- | --- | --- | --- |
| Import an authorized Vietnamese source | Existing immutable manuscript import and private original/source comparison | PASS: existing import suite and fresh editor HTTP journey | NOT_RUN; executor walkthrough pending |
| Open a host-created proposal | Existing owner-scoped adaptation review; source, findings, uncertainty and caller declarations remain visible | PASS: fixture context/proposal read, no inference | NOT_RUN; fixture proposal walkthrough pending |
| Edit scene, character and spoken lines | Structured act/scene/character navigation; narration, dialogue, emotion, pronunciation and cues; optional advanced JSON | PASS: lossless core tests and edited accepted JSON in real HTTP/PG journey; scoped context and selected-option corrections implemented | NOT_RUN as a completed gate; final corrected browser scenario pending |
| Compare source and changes; inspect validation | Preserved source/warnings, complete-value keyed changes, nonpersisted backend preview and field-path mapping | PASS: raw source/hash, validator HTTP tests and diagnostic mapping | NOT_RUN; focus and validation-summary walkthrough pending |
| Explicitly review and accept/save | Current-text proposal acknowledgement; existing explicit acceptance and exact saved-head owner review; CAS save | PASS: accepted revision 1, owner review, later revision 2, stale conflict and exact replays | NOT_RUN; browser permission/review/reset behavior pending |
| Reopen exact accepted and historical versions | Existing exact-revision/current-head/history APIs; read-only comparison never automatically replaces local draft | PASS: exact old/head/source/proposal/history after reopening and restart | NOT_RUN; browser reopen/history/pagination pending |
| Preserve unsaved/pending text during navigation and failure | Actor-bound draft, field buffer registry, native unload/discard guard, exact unresolved mutation retry | PASS: deterministic reducer regressions; backend stale/replay cases | NOT_RUN; back/close/cancel/focus and timeout interactions pending |
| Desktop, narrow, zoom and accessibility | Existing Material 3 Expressive tokens, Vietnamese/English copy, native semantic fields and visible focus | Native/WASM compilation held; no visual claim from build | NOT_RUN; final listed viewports/themes/keyboard observations pending |

## Executed checks and provenance

Local environment: Mac mini M4, macOS 15.6, Rust 1.87.0, Trunk 0.21.14, Safari 18.6 and
Homebrew PostgreSQL 17. The backend tests create a fresh disposable cluster; an inherited or
production database URL is never substituted. The runtime uses the restricted application
role and original synthetic Vietnamese source/proposal content. No real manuscript, model,
external identity deployment, media store or generated audio is involved.

| Result at draft time | Exact command / artifact | Scope and remaining limit |
| --- | --- | --- |
| PASS milestone; final result pending | `cargo clippy --workspace --all-targets --locked -- -D warnings` | Native workspace pass held; executor to pin final-tree result |
| PASS milestone; final counts pending | `cargo test --workspace --locked` | Native workspace pass held; executor to record final test counts; ordinary tests do not run ignored PG cases |
| PASS | `cargo test -p cantos-studio --locked authoring::tests` | 13 authoring tests + 2 matching localization tests; focused core agent run |
| PASS | `rustfmt --edition 2021 --check apps/web/src/authoring.rs` | Focused core formatting; final workspace formatter remains executor-owned |
| PASS | `PATH="/opt/homebrew/opt/postgresql@17/bin:$PATH" python3 scripts/test_postgres.py --keep` | 12 revision + 9 import + 21 adaptation/editor = 42 real PG cases; retained new local cluster supports walkthrough |
| PASS milestone; final result pending | `cargo clippy -p cantos-studio --target wasm32-unknown-unknown --locked -- -D warnings` | WASM pass held in local evidence; executor to pin final-tree rerun |
| PASS milestone; final result pending | `NO_COLOR=true trunk build --locked` from `apps/web` | CSR/assets build pass held in local evidence; executor to pin final-tree rerun; compilation only |
| PASS | Actual HTTP process kill and PostgreSQL restart with migration/intent replay | Inspected `target/studio-editor-evidence/server-restart.json`; settled counts and exact observations retained |
| PENDING | `cargo fmt --all -- --check` | Executor to record final tree result |
| PENDING | `python3 scripts/check_repository.py` | Executor to record final documents/contracts/links result |
| PENDING | `TMPDIR=/private/tmp python3 -m unittest discover -s scripts -p 'test_*.py'` | Executor to record final repository-script result |
| PENDING | `python3 scripts/reference_script_ir.py` | Executor to record unchanged independent golden result |
| PENDING | `TMPDIR=/private/tmp python3 scripts/mutate_script_ir.py` | Executor to record bounded validator/encoder mutation result |
| PENDING | `git diff --check` | Executor to record final patch result |
| NOT_RUN as a completed gate | Completed Safari interaction/accessibility/capture matrix | Corrections implemented; same-scenario final inspection/results pending executor record |
| NOT_RUN | VoiceOver, named OS Telex/VNI composition, device/mobile, provider/audio/publication/deployment | No evidence claimed in this slice |

Raw local logs/receipts are ignored under `target/studio-editor-evidence/` and
`target/revision-evidence/`. The current successful PG suite logs are
`postgres-tests.log`, `import-postgres-tests.log` and `adaptation-postgres-tests.log` in the
latter directory. An earlier focused editor log expected HTTP 201 from an existing route that
returns 200; that test-oracle assertion was corrected and is superseded by the successful
21-case adaptation/editor suite. Earlier failure logs are retained for provenance, not reported
as passing gates. Credentials and database files remain outside Git.

The real backend journey changed the unresolved character to “Minh — người gác cửa”, scene title
to “Bên cánh cửa — đã đối chiếu”, and Mai's line to “Chúng mình sẽ chờ đến bình minh.” with hopeful
delivery at 525 permille. Explicit acceptance created revision 1; owner review referenced its
exact c1/e1 digests. A later save changed the same stable dialogue to “Chúng mình sẽ chờ đến sáng.”
as revision 2. Reader save/review attempts were forbidden, a stale base conflicted, and repeated
accept/save/review intents returned their original receipts without extra records.

The restart oracle records identical before/after counts: one source, one proposal/submission,
one acceptance, two revisions, one review and zero generation attempts. Original source SHA-256
was `b242e7ce5c677751d5bcd7384ddb0698b740c30294978e4aa5a1a46b929a24f2`. Exact source/proposal/accepted
revision/review/history responses and committed-intent replay were compared. This is a real
HTTP/PostgreSQL integration journey with fixture content, not a Leptos interaction result or
provider-generation result.

## Browser observation and review follow-up

The executor is driving the built CSR app in native Safari 18.6 on the Mac mini against the
task-owned loopback host and disposable PostgreSQL cluster. A context-owner collision between
structured editors and incorrectly initialized native choices were observed during this
walkthrough. Scoped `Provider` ownership and explicit `option.selected` initialization are now
implemented. The initial local capture is
`target/studio-editor-evidence/screenshots/context-collision-before.png`; it records the finding,
not a final successful editor screenshot. The final corrected flow remains to be recorded here.

Final runtime entries must name the actual source/proposal/revision journey, inspected browser
viewport or window dimensions, theme, zoom, keyboard/focus/cancel behavior, Vietnamese text and
capture paths. They must separately record DOM assertions, real interaction, accessibility checks,
captured images and opened image inspection. No measurements or successful interactions are
inferred from compilation or from the single failure capture.

Review follow-up at this draft boundary:

- Recheck per-field buffer/composition tracking, including one invalid intensity followed by a
  different field commit, Escape restoration, navigation and source/actor changes.
- Recheck reader/editor/owner explanations and guards while retaining backend authority; preview
  success must not present an acceptance or owner-review permission claim.
- Repeat the affected live proposal/revision scenario after the implemented scoped-provider and
  selected-option corrections, then pin the final WASM/build/native checks to the same tree.
- Exercise the request deadline and body-interruption outcome if included in the final runtime
  scope; otherwise retain the unrun transport-chaos limitation rather than infer it from reducers.
- Finish an independent code review, fix actionable findings, pin the final commit/PR and retain
  required CI and postmerge evidence for the exact tested SHA.

The code corrections are implemented. This list preserves final runtime/evidence gaps until the
executor supplies outcomes; it does not mark the original code findings as still unfixed.

## Read inventory and DBSP decision

| Read / consumer | Existing authority and request shape | Freshness / change in this slice |
| --- | --- | --- |
| Current accepted head | `GET /api/v1/scripts/{script}/head`; current actor/script access and immutable complete revision | Existing point read; explicit editor read ticket and actor/script guard |
| Exact historical version | `GET /api/v1/scripts/{script}/revisions/{revision}`; current actor/script access and fixed immutable revision | Existing point read; read-only comparison with its own ticket |
| Revision history | `GET /api/v1/scripts/{script}/history?after_revision=…&limit=…`; existing bounded ascending revision/review range | Existing paged endpoint; Studio requests 20 rows and exposes next-page action |
| Preserved source text | `GET /api/v1/scripts/{script}/sources/{source}`; current access and linked immutable source/hash | Explicit reads cached only in current actor/script/head inspector state; no request per keystroke |
| Imported/binary source fallback | `GET /api/v1/imports/{id}` and authenticated original route; existing owner-scoped import authority | Existing fallback for supported imported source IDs; access failure remains visible |
| Proposal/source comparison | `GET /api/v1/adaptations/{id}/review`; existing owner/context/proposal/input/source facts | Existing coherent point reads; generation is never dispatched |
| Draft validation preview | `POST /api/v1/validation`; current session plus bounded pure Script IR reader | New nonpersisted validation; no target metadata, history query or serving read model |

No underlying history/source/current/proposal query or database hot read path changes. The UI
reuses bounded pagination and explicit cached source reads rather than introducing per-line or
per-input SQL, an unbounded list or a repeated aggregate. Current authorization remains checked
on each request; changing actor/script/head invalidates visible cached inspector facts. Reads
between pages can observe later commits and are not claimed to form a frozen history snapshot.

**DBSP: DEFER.** No measured hot aggregate consumer or query change justifies a new engine,
CDC boundary, materialized cache or dependency. Existing PostgreSQL point/range reads remain
authoritative; prior bounded benchmarks do not prove this new UI's production throughput.
No new latency/cold-cache/concurrency/write-overhead benchmark or speedup is claimed here.

## Compatibility and residual risk

Script IR reads/writes remain `{0.1.0}`. No Script IR converter, schema/golden/digest scheme change or stored
revision rewrite is introduced. The validation wire addition is additive and creates no durable
record. Editing effective spoken fields changes a later revision's SpokenContent; reorder-only
changes preserve line identity/content. This slice performs no TTS or mix/QC/publication
invalidation execution, so no audio regeneration result is claimed.

Buffered drafts and exact retry intents live in the browser session's memory. Native unload
confirmation is a guard, not durable crash recovery: a confirmed reload/close discards unsaved
text. Structural history is bounded; pronunciation entries have no stable IDs in the current
schema and remain explicitly indexed. Conflicts require comparison and an explicit new base;
there is no automatic collaborative merge.

No supplied design-reference image is available for this request. The existing versioned Studio
tokens and [UI system](../design/ui-system.md) govern the compact Material 3 Expressive direction;
no pixel-faithful reference match is claimed. Native labels, keyboard/focus observation and
opened captures can establish bounded accessibility evidence, but no VoiceOver/screen-reader
walkthrough or named OS Telex/VNI IME run has been supplied. Synthetic composition events and
Vietnamese paste/fixture text, if observed, must not be reported as an OS input-method result.

Production identity/TLS, legal adjudication of source/content rights, real Vietnamese AI quality,
ChatGPT/Gemini web connector registration, casting, TTS, mixing, publication, Theatre, native CMP
playback and mobile runtime remain outside this boundary. No CI expansion, package/toolchain
upgrade, provider configuration, secrets setup or deployment is part of this task. Final browser
acceptance, independent review, exact-SHA required checks, merge verification and task-process
cleanup remain executor-owned until this draft is finalized.
