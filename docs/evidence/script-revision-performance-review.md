# Prompt-3 pre-commit assurance review

## Scope and assessment

Single reviewer, read-only passes with [cantos-code-review](../../.agents/skills/cantos-code-review/SKILL.md),
its diff-review/assurance references and engineering/read-performance/Script IR owners.
BASE/HEAD before commit: `626416500599b852dc985683fa3a4ac905f9401f` on local develop.
Review the staged/unstaged/untracked prompt-3 snapshot, not a PR/remote three-dot comparison.
Entry tree was clean; prior prompt-1 and prompt-2 commits remain intact. No policy, dependency,
manifest, domain, migration, API or UI implementation was changed.

Input fingerprint: SHA-256 `c3a066ed005a5947d29d7ead5044fd71257bb21561759f802b7142ffdfec95cb` over sorted changed path, NUL, file bytes, NUL
(nine paths; this report excluded to avoid self-reference).

Coverage: **complete-for-declared-scope** for the two new Python files, README/server guide,
read inventory, queue, protocol, performance report and generated safe result record. The full
harness lifecycle/seed/oracle/workload/resource code and tests were read; production SQL/HTTP,
initial migration, contracts and immediate consumers were traced against BASE. All 144 per-run
records, phase metadata, source/SQL hashes and fixture checkpoints were structurally compared
with the complete raw final reports, with finite metrics/sample counts checked. Curated plan
nodes were traced to raw JSON EXPLAIN. Raw individual timing/resource arrays are retained locally;
each of their hundreds of thousands of observations was not manually inspected. No transitive
dependency security audit, UI/device/provider or absent engine review is claimed.

Assessment: **no-actionable-findings** for the final bounded local measurement/no-production-change
scope after the fixes below. This is neither approval nor complete #1/010 acceptance.

## Assurance ledger

| Claim | Owner / target | Required by | Evidence held → level | Disposition / next action |
| --- | --- | --- | --- | --- |
| No existing/production DB is used | benchmark lifecycle/env; isolated cluster/app role | review-profile:requester; policy:AGENTS.md disposable cluster | fresh initdb on loopback, separate nonsuperuser app role, env-isolation unit regression, owned-process cleanup → integration-tested / example-tested | held; generated data remains local |
| Scale/skew and comparable request inputs | generator, seed template clones, main orchestration | policy:read-performance workflow; doc:benchmark protocol | original fixture unchanged; 4/512-dialogue admission; common cloned row counts; real orchestrator seed-input regression against explicit I/O doubles; both final actual runs → example-tested / integration-tested | held; no production forecast |
| Full accepted result and current permission semantics | unchanged Store load/decode and HTTP; independent PG row projection | policy:AGENTS.md immutability/private drafts; review-profile:requester | exact actor/time/export/digest responses at quiescent committed positions; owner/reader/editor, foreign owner/outsider/anonymous/missing history/revocation/reader-write denials → differentially-tested / integration-tested | held; no async serving/cache boundary introduced |
| Assertions/resource failures cannot silently disappear | CLI guard, Resources error propagation | policy:engineering evidence vocabulary | real optimized-Python refusal regression; resource preflight and collector exceptions fail the run; successful final samples retain component CPU/RSS → example-tested / integration-tested | held; sampling limitations documented |
| Performance numbers are attributable and bounded | source extraction, raw reports, safe result projection, docs | policy:read-performance evidence record | final harness/source/query hashes; both scales/72 runs each; cold-start caveat; all repetitions preserved; whole response/transport; CPU/RAM/WAL/storage and plans → integration-tested | held; debug/shared machine, limited tail sample sizes remain |
| Index experiment preserves oracle results and accounts for writes | disposable member_cover DB; authoritative PG | policy:read-performance simpler comparison | existing PK vs INCLUDE(role), before/after access changes and EXPLAIN; API save/permission-write/storage/WAL observations → integration-tested / differentially-tested | held; reject candidate, no migration shipped; all-visible static traffic not measured |
| Retry/concurrency/immutability behavior remains intact | unchanged domain/store/migration/HTTP | policy:AGENTS.md correctness; review-profile:requester final gates | fresh PG9 plus HTTP kill/PG restart/replay; Rust23, oracle, seven killed mutations → integration-tested / fault-injected / mutation-tested | held within existing local boundary |
| DBSP decision and queue do not imply integration or issue completion | performance record and work queue | policy:read-performance fit decision; review-profile:prompt3 only | no hot repeated large join/aggregate, no engine/dependency/CDC/serving path; explicit defer prerequisites → documented/source-inspected | held; engine experiment NOT_RUN, conditional prompt4 not executed |
| Checks correspond to real targets | workspace/server/Studio/scripts | policy:AGENTS.md verification | final fmt, native/WASM Clippy, Rust23, Python57, PG9+restart, oracle, mutations7, Trunk, tokens, repository/diff checks → statically-checked / compiled / example-tested / integration-tested | held; compile does not establish interaction/accessibility |
| Pipeline/publication/Theatre/CMP behavior | unchanged/planned targets | policy:AGENTS.md architecture | no changed consumer, provider, release, audio or native surface | not-applicable to this diff; no capability claim |

## Resolved findings and diagnostic history

- **[P2] Different phase RNG offsets made the promised comparison unpaired** — class: confirmed
  defect; category: evidence; confidence: confirmed-from-source. The initial main loop added
  `10 * phase_count` to repetition, changing warm parameter selection across indexes despite
  identical seed DBs. Fixed to use the same repetition in every clone. The regression executes
  the real main orchestrator with I/O doubles and compares all cold/warm seed inputs over two
  repetitions. Final 100/5,000 runs use the corrected source and replace earlier numbers.
- **[P2] Optimized Python could suppress oracle assertions** — class: confirmed defect;
  category: evidence; confidence: confirmed-from-source. A `-O` invocation could otherwise
  reach PASS with checks removed. The CLI now refuses before cluster start; a real subprocess
  regression checks the exact refusal. Final runs use ordinary assertions.
- Initial smoke TypeError/exact error-body assumptions and the interrupted growing-history
  comparison were corrected before final runs. Raw artifacts from preliminary experiments stay
  diagnostic; none is used as final evidence. No production SQL defect was introduced or fixed.

Fixes were performed under the user's explicit “review, fix in scope, rerun” authorization,
through the engineering task after read-only findings; no second permission was required.

## Checks and remaining scope

[Execution table](script-revision-performance.md#verification-and-residual-risk) gives exact commands,
outputs and PASS/NOT_RUN scope. Final code: Rust23; PG9 + real HTTP/PG restart; Python57 including
10 new harness regressions; fmt/native and WASM Clippy; c1 oracle; mutations7 killed; Trunk/token
checks; repository and staged diff checks. Both final clusters/hosts stopped and no benchmark/test
process or approval remains pending. No push, PR, GitHub issue mutation or remote CI run occurred.

Residual risk: synthetic debug workloads are not production capacity; no disk-cold isolation,
thermal/background control, maximum 2 MiB/export or high evidence-fan-out contention workload,
all-visible static membership benchmark, continuous CPU accounting or physical unique PG RSS is
proved. READ COMMITTED permission changes do not retroactively revoke authorized transactions.
Production auth/TLS/rights eligibility, durable browser recovery, full editor/accessibility,
mobile/audio and DBSP freshness/recovery/fallback remain outside this slice.
