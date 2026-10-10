# Prompt-4 bounded pre-commit assurance review

## Scope and assessment

Single reviewer using [cantos-code-review](../../.agents/skills/cantos-code-review/SKILL.md),
its diff-review/assurance references, engineering/read-performance and Script IR owners.
BASE/HEAD before commit: `ff6e3cc5c8f8506c904179b2d54ef1a6ebec1c39` on local `develop`.
Scope: staged/unstaged/untracked prompt-4 documentation snapshot, not a PR, remote comparison
or final review of the entire prompt sequence. Entry tree was clean. No fetch or remote-state
claim is made; local tracking refs do not establish the current origin state.

Input fingerprint: SHA-256
`60e940ca55c3ff9db3c1329f364c0ed8dcd1c314d5636084e19bdd66e3a60023`
over each sorted changed path, NUL, file bytes, NUL (three paths; this report excluded to avoid
self-reference). Paths: `docs/evidence/script-revision-dbsp-decision.md`,
`docs/decisions/0003-script-revision-persistence.md`, `docs/work-plan/README.md`.

Coverage: **complete-for-declared-scope** for those documents and this evidence record.
Current Store/HTTP/migration/manifests and immediate read inventory/contract boundaries were
traced; prompt-2 persistence/review and prompt-3 performance/protocol/review were read.
The retained safe JSON was compared to both raw reports and all curated plan projections,
including hashes, fixture/phase metadata, 144 run records and 144 plans. Individual raw
request/resource observations were not manually re-inspected. No new runtime, dependency,
schema, fixture or skill change exists. Engine, worker, Theatre, CMP/Android/iOS, provider,
deployment and UI runtime passes are outside this diff and make no new acceptance claim.
Type/construction, core extraction and behavioral regression passes are inapplicable to a
documentation-only decision. Policy/evidence, compatibility, security assumptions and
freshness passes were applied against the trusted pre-change rules and BASE.

Assessment: **no-actionable-findings** for the conditional DEFER decision and future protocol.
This is not engine adoption, production approval or full #1/010 acceptance. No in-scope defect
or required evidence gap remains; missing engine prerequisites justify DEFER rather than a
claim that the engine passed. The separate final sequence review remains parent-owned.

## Assurance ledger

| Claim | Owner / target | Required by | Evidence held → level | Disposition / next action |
| --- | --- | --- | --- | --- |
| Conditional experiment is deferred without a real hot consumer | Decision / engineering read-performance | review-profile:requester conditional prompt 4; policy:read-performance fit decision | Actual point-read consumers/source, unchanged source/query hashes, historical PG plans and results → documented/source-inspected | held; reopen only with linked admission evidence |
| Measurements retain their original provenance and limitations | Prompt-3 report/JSON/protocol | policy:engineering evidence vocabulary; policy:read-performance benchmark record | Two raw checksums, 144 result records, 144 curated plans and rederived quoted numbers agree; no benchmark rerun → documented, artifact audit passed | held; local synthetic debug/shared-buffer observations remain bounded |
| Future oracle and serving obligations are explicit, unexecuted | Decision protocol / future owning adapter | policy:read-performance differential/recovery and serving boundary | Same committed barrier, current backend access, canonical edge checklist, numerical budget admission and end-to-end comparator specified → documented | held for protocol; engine tests/CDC/freshness/rebuild/fallback NOT_RUN |
| Revision identity and authorization are not weakened | Unchanged Store/domain/contracts; decision assumptions | policy:AGENTS.md immutability/private drafts; doc:docs/architecture/script-ir.md | c1 metadata exclusion, full export/base/actor/operation comparison and existing READ COMMITTED revocation limit stated; no serving change → documented/source-inspected | held; runtime evidence remains historical prompt 2/3 |
| ADR and recommended order reflect DEFER without execution-status changes | ADR 0003 / 010 queue | review-profile:requester; policy:AGENTS.md working and review | ADR stays proposed; queue links decision and continues importer/editor; no new service/dependency/migration or issue closure → documented | held; no production or GitHub acceptance implied |

## Commands and evidence freshness

Run from repository root at BASE plus the documentation snapshot above; Python 3.13.5 was
confirmed in this session. Logs/helper are local, ignored artifacts under
`target/dbsp-decision-verification/`. The artifact-audit helper starts no DB/engine and is not
a product runner or a new committed test. Python tests use the existing suite and its named
I/O doubles; they do not become PostgreSQL integration evidence.

| Command / check | Execution result | Log / scope |
| --- | --- | --- |
| python3 target/dbsp-decision-verification/evidence-audit.py | passed / PASS | evidence-audit.log; historical hashes/raw projections and quoted medians/storage/WAL |
| TMPDIR=/private/tmp python3 -m unittest discover -s scripts -p 'test_*.py' | passed / PASS; 57 tests | python-tests.log; example-tested existing Python tools |
| python3 scripts/check_repository.py | passed / PASS; 205 text files, 15 skills | repository-check.log; document/link/JSON/skill hygiene only |
| git diff --check | passed / PASS | diff-check.log; tracked whitespace only; new files require staged check |
| git diff --cached --check | passed / PASS | staged-diff-check.log; whitespace including new files |
| Native/WASM fmt/Clippy/build, Rust tests, disposable PG/restart, c1 oracle and mutations | not-run / NOT_RUN | No code/contracts/toolchain changes; documentation-only policy calls for proportionate checks; prompt-3 results were read, not rerun |
| New read/DBSP benchmark, official engine/version/connector runtime verification, CDC, lag/rebuild/fallback | not-run / NOT_RUN | Admission gates unmet; no engine/consumer installed or invented |
| UI/accessibility, mobile, audio/provider, production auth/TLS/rights/capacity and remote CI | not-run / NOT_RUN | Outside this slice; no new acceptance claim |

No new behavioral test or heavy build is warranted by the decision record. Final repository
and whitespace checks cover the completed report before the local commit. No test
assertion or gate was relaxed. Prior prompt-1/2/3 commits remain ancestors; no unrelated work
was removed, no push/PR/issue mutation or credential/remote change was made.

Pre-existing documentation limitation: some skill repository-reality paragraphs still describe
the bootstrap as having no code. Actual manifests/source and owning product/architecture docs
take precedence under AGENTS; this bounded decision does not extend into skill maintenance.

**Residual risk:** source/artifact inspection and document checks are not runtime engine
equivalence or a production security/capacity audit. There is still no eligible measured
maintained consumer, accepted adapter, freshness/recovery budget or engine-boundary evidence.
Production identity/TLS, rights eligibility, full editor/accessibility and native mobile gates
remain outside this review.
