# Authenticated Script IR revision persistence evidence

## Scope and provenance

Prompt 2 only: one local development Rust/Axum/PostgreSQL save/read slice and a minimal live
Leptos Studio consumer. Base: `develop` at
`ea8e1d98a31301b0d63dfa75d8e56450397b581d` (prompt 1). No reset discarded that commit.
The initial working tree was clean. The user's explicit instruction keeps this change on
`develop` as one local commit, overriding the normal feature-branch/PR workflow.
No push, credential/permission/remote change, PR, issue comment or close was performed.

The prerequisite is the user's verified prompt-1 handoff, not acceptance of proposed ADR 0002
or completion of issue #1. The linked issue was read successfully; its full acceptance
checklist remains broader than this slice. Parent read-only GitHub evidence confirmed remote
develop now contains the prompt-1 SHA. Historical push403 is not evidence of a current missing
repository write permission, and push capability was not retried.

Environment: macOS 15.6 (24G84), Rust 1.87.0, PostgreSQL 17.4 (Homebrew), Trunk 0.21.14,
wasm-bindgen 0.2.100, headed Safari 18.6 (20621.3.11.11.3). Original synthetic two-scene
Vietnamese fixtures and synthetic alice/bob identities only. No production database, private
manuscript, actual session secret, paid provider, object store or generated audio was used.
Raw logs and disposable clusters live under ignored `target/revision-evidence/`.
They are local artifacts, not remote CI output. The task-owned HTTP host and all four running disposable PostgreSQL clusters were stopped after inspection; their ignored data/logs remain available for diagnosis.

## Invariant and owners

**Invariant:** an authenticated authorized save admits a complete validated export as a new
immutable revision atomically, rejects a competing stale write, and reconciles an ambiguous
retry to the original actor/base/export acceptance; reads require current access and verify
the full stored export instead of repairing it.

| Owner / boundary | Claim / failure mode | Evidence level and concrete oracle |
| --- | --- | --- |
| script_ir / revisions pure modules | Validated private values; complete export; closed read/write access table; prior replay before stale check | Existing 12 contract tests + 2 pure revision tests; exact decision variants |
| PostgreSQL transaction | Actor/session/current owner/member checks; complete export, links and locked head commit together | integration-tested with nine real PG tests; no in-memory repository |
| SQL rows/triggers/app role | Accepted bytes/actor/time/evidence links cannot be rewritten by app; corrupted rows fail admission | integration-tested app SQLSTATE42501 and immutable55000; privileged test-only tamper returns corrupt_revision |
| Replay/integrity | Same c1 can have different metadata; exact base/full export binds key; old pinned history unchanged | differential fixture comparison plus same-c1/different-e1/metadata tests; no digest dedup |
| Concurrency/failure | No silent lost update or half-accepted row; duplicate key returns original result | barrier-controlled competing writers; DB connection killed after INSERT/before commit; retry with same key |
| Process recovery | Committed export/actor/time/retry survive host and DB restart | HTTP process SIGKILL, PG fast restart, migration checksum replay, real HTTP read/replay equality |
| HTTP/wire | Typed denial, malformed/unknown fields, Origin and current auth; private diagnostic payload not leaked | JSON Schema validates actual responses, shared DTO/fixture tests; anonymous invalid body401, stale409, invalid422 |
| Studio pure editor | Preserve text, immutable retry snapshot and original actor through unknown outcome/expired auth | example-tested reducer races, conflict/rebase, edit during save, stale reads, expired-session retry, changed-cookie acknowledgement |
| Studio rendered shell | Actual read/edit/save, Vietnamese status, keyboard/focus, themes/zoom | interaction-tested and screenshot-inspected in headed Safari against real local Axum/PG; details below |

Types are a partial barrier: Script IR constructors are private/validated and access/decision
outcomes are enums, but wire strings/integers are raw DTOs. UUID/base refinement and row
integrity are runtime admission, not a claim that all invalid states are unrepresentable.
Direct SQL inserts by a privileged role and superuser DDL can bypass application admission.
The separate app role is required; the HTTP host does not auto-provision it.

## Commands and execution results

Commands run from the root unless a directory is stated. Final result details are recorded
after the aggregate run; ordinary workspace tests leave the nine PG cases ignored and do not
replace the separate integration result.

| Gate | Result / actual scope | Local artifact |
| --- | --- | --- |
| cargo fmt --all -- --check | PASS | fmt.log |
| cargo clippy --workspace --all-targets --locked --offline -- -D warnings | PASS (native targets) | clippy.log |
| cargo clippy -p cantos-studio --target wasm32-unknown-unknown --locked --offline -- -D warnings | PASS (actual view/transport WASM) | wasm-clippy.log |
| cargo test --workspace --locked --offline | PASS; 23 tests, 9 explicitly ignored PG cases | rust-tests.log |
| PATH=/opt/homebrew/opt/postgresql@17/bin:$PATH python3 scripts/test_postgres.py --keep | PASS; 9 real PG tests plus process/PG restart and exact response replay | postgres-run.log, postgres-tests.log, host.log |
| NO_COLOR=true trunk build --locked --offline (apps/web; cached matching helper on PATH) | PASS; debug CSR bundle | trunk-build.log |
| python3 scripts/check_repository.py | PASS | repository-check.log |
| TMPDIR=/private/tmp python3 -m unittest discover -s scripts -p 'test_*.py' | PASS; 47 tests | python-tests.log |
| python3 scripts/reference_script_ir.py | PASS; unchanged independent c1 oracle | reference.log |
| TMPDIR=/private/tmp python3 scripts/mutate_script_ir.py | PASS; 7 bounded validator/encoder mutation checks | mutation.log |
| python3 scripts/studio_tokens.py --check | PASS; source/hash/CSS drift | tokens.log |
| git diff --check | PASS | diff-check.log |
| Native Safari walkthrough | PASS for the bounded observations below | CUA AX states and emitted captures in this task |
| Remote CI, production auth/deployment, exact 320 CSS viewport instrumentation, VoiceOver, OS IME, mobile, audio/provider/storage | NOT_RUN | No such evidence |
| EXPLAIN/latency/throughput or DBSP experiment | NOT_RUN by prompt-2 boundary | Read inventory only |

Earlier failures were diagnostic, not passing evidence: sandbox PG shared-memory allocation
was denied and resolved through approved isolated local execution; an overlong PG Unix socket
path was corrected to /tmp. The first killed-connection test exposed fast pool recycling of a
dead socket; verified recycling fixed it and the real regression passed. Restart stdout pipe
inheritance caused a harness wait and was fixed with an explicit PG log file. An expired-session
after ambiguous-save regression failed before preserving the operation identity. Trunk's
NO_COLOR requires true, and offline helper discovery requires its cache directory on PATH.
The expanded workspace initially broke the mutation runner because its disposable copy omitted apps/web; copying the concrete consumer and scoping its test to cantos-server restored the 12-test baseline and killed all seven mutants. The staged diff check also caught a trailing blank line in the new Web manifest; it was removed before the final staged check. Final passing results supersede those specific failures; no unrelated gate is waived.

## Browser observations

Rendering: Leptos 0.7.8 CSR debug bundle; same-origin host
`python3 scripts/serve_studio.py` at `http://127.0.0.1:8080`, backed by the disposable
PG17.4 restart database. No mock response/data source. The unavailable in-app browser was not
used; native Safari was controlled through CUA accessibility/native input.

- Real sign-in as synthetic alice, keyboard read of the printed UUID, complete canonical
  two-scene Vietnamese export with source/rights/cue evidence visible.
- Edit Vietnamese text and keyboard-save revision 2; final reducer build read it, marked later
  input as unsaved, then keyboard-saved revision 3. The backend returned alice and immutable UTC
  acceptance timestamps; focus remained visible on the initiating action.
- Wide captures: 1619 × 862 native window pixels, light/dark. Compact captures: 643 × 862
  window pixels at 100% and confirmed Safari page-menu 200%; effective page width is approximately
  314 CSS px after the scrollbar, narrower than the 320 target, but not instrumented as an exact
  CSS viewport. Toolbar/reason/status and Vietnamese text reflowed; save labels and focus remained
  visible. Both themes were inspected at compact 200%. Normal zoom/window size were restored.
- Native HTML labels/heading structure and polite status were inspected through AX; option-tab
  traversed controls and Return triggered read/save. Textarea and read-only export wrap long JSON.
  Token-role contrast floors are independently calculated by the Python token tests.
- No timing/motion performance inference is made from images. The slice adds no animation/audio;
  reduced-motion CSS is source-inspected only. No VoiceOver, real Telex/VNI composition, automated
  DOM harness, exact visual reference comparison or mobile renderer was run.

Captured pixels were emitted by CUA in this task and inspected there; no local PNG files are
claimed. A Safari reload was rejected by automatic approval review because it could discard
the open draft. A new tab preserved that tab while checking a fresh build. No rejection was
bypassed and no pending approval remains. Page-return behavior is best-effort: Safari may
recreate the WASM page rather than use bfcache, losing in-memory state; durable draft recovery
is explicitly not implemented.

## Decisions, docs and handoff

[ADR 0003](../decisions/0003-script-revision-persistence.md) is proposed; ADR 0002 remains proposed.
Root/app/contracts/product/architecture/UI/work-queue docs and AGENTS verification commands
now name actual runtime gates. Skills with bootstrap-only repository-reality text remain
follow-ups, not authority to pretend the runtime is absent. CI configuration covers native
workspace checks; database/browser builds still require their named local gates unless a
future CI change adds them. No remote CI execution was requested or observed.

[Pre-commit review](script-revision-review.md) records the declared scope and assurance dispositions.
[Read inventory](script-revision-reads.md) enumerates both GETs and internal authentication,
replay, evidence and migration fetches, their indexes, permissions and feeding writes.
DBSP is deferred because there is no measured hot aggregate/read model. Workload targets,
PostgreSQL plans and performance comparisons belong to the next authorized prompt; that prompt
was not opened/executed. Issue #1 and the 010 journey are not closed or fully accepted here.

**Residual risk:** development sessions/TLS/operator provisioning and legal rights eligibility
remain production blockers; stable editorial ID continuity across documents is not enforced.
Role/actor changes are rechecked on subsequent operations, without retroactive revocation of
an in-flight authorized transaction. Browser reload/tab close loses an in-memory draft and
ambiguous retry identity. Exact viewport instrumentation, screen reader, OS IME and the full
scene editor remain UI evidence gaps. No read-performance, production, CI, media or native
claim follows from these local tests.
