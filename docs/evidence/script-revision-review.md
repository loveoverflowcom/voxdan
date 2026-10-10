# Prompt-2 pre-commit assurance review

## Scope

Single reviewer, separate read-only passes using
[cantos-code-review](../../.agents/skills/cantos-code-review/SKILL.md), its diff-review and
assurance references. English repository report follows AGENTS; the user handoff is Vietnamese.
BASE/HEAD before commit: `ea8e1d98a31301b0d63dfa75d8e56450397b581d` on develop. Scope is the
staged/unstaged/untracked prompt-2 snapshot, not a PR or three-dot remote comparison.

Input fingerprint: SHA-256 `38bce240bdf24f1a87a4917b3a74257c3bc56f0a574f6b5a0197ad7c17b5015d`
over each sorted changed path, NUL, file bytes, NUL (54 files; this report excluded to avoid
self-reference). Git status/diff and untracked inventory were inspected; the tree was clean at
entry. No unrelated change was staged or discarded. AGENTS changes add real runtime commands;
they do not weaken the pre-existing verification requirements. ADRs remain proposed.

Coverage: **complete-for-declared-scope** for the 54 source/contract/doc paths at the module and
claim boundaries below. Cargo.lock was inspected as generated dependency metadata alongside
its concrete manifests, pinned APIs/source and native/WASM builds; transitive source security
auditing was not performed. No binary/media files are in the change. Native mobile, production
workers, Theatre, provider/storage and deployment targets were not reviewed as implementations
because this diff creates none. Existing CI was read; it executes native workspace checks but
does not execute the separate PG/browser gates. No remote CI result is held.

Assessment: **no-actionable-findings** for the declared local development persistence/minimal
consumer boundary after the resolved regressions. This is neither approval nor full #1/010
acceptance; production/UI/performance evidence gaps below remain.

## Assurance ledger

| Claim | Owner / target | Required by | Evidence held → level | Disposition / next action |
| --- | --- | --- | --- | --- |
| Script IR/c1 compatibility and complete export | server script_ir/model projection, existing reader/canonical/validation; contracts | policy:AGENTS.md architecture/correctness; doc:contracts/README.md | Unchanged c1 goldens; 12 existing tests, independent oracle and 7 killed mutations → differentially-tested / mutation-tested | held; no legacy goldens/schema changed |
| No outward dependencies in domain rules | script_ir and revisions; server target | policy:cantos-engineering decoupling | use edges inspected; pure access/save decisions in plain tests → documented (source-reviewed) / example-tested | held; DTO crate has concrete Axum/WASM consumers |
| Raw wire values cannot forge validated Script IR | HTTP DTO → PostgreSQL admission; row decode → canonical reader | policy:cantos-engineering boundary-hardening | Invalid IR/foreign evidence rollback; stored hash tamper fails → integration-tested | held; raw UUID/base strings remain runtime checks, direct privileged SQL is outside the constructor barrier |
| Authenticated actor and current read/write authorization | postgres authenticate/authorize; HTTP middleware | review-profile:requester; policy:AGENTS.md private drafts | Anonymous/spoofed actor header, wrong owner/history/write, reader/editor and revoked/expired membership/session checks → integration-tested | held; operator/deployment identity still separate |
| Complete accepted export and evidence ownership | postgres save; SQL revision/registry/link schema | review-profile:requester; doc:contracts/README.md evidence | Independent complete fixture agrees; metadata edit keeps c1 but changes export/e1; owner evidence and links asserted → integration-tested / differentially-tested | held; registry existence is not legal clearance or source-byte preservation |
| Accepted facts append-only | migration triggers, constrained app role, load admission | policy:AGENTS.md immutability | App DML/DDL SQLSTATE42501, admin update/delete55000, original pinned export unchanged → integration-tested | held within application/normal DML boundary; superuser bypass explicitly excluded |
| Atomic migration/replay and rollback | migrate transaction/checksum ledger | review-profile:requester migration/rollback | Replay with retained data, changed checksum refusal, deliberate DDL collision leaves no ledger/actors then repair succeeds → integration-tested | held; no destructive down migration promised |
| Stale concurrent writes cannot overwrite | pure decide_save + locked head/guarded update | review-profile:requester concurrency | Two real writers wait behind observed PG row-lock barrier; one accepted and one stale, head/count exact → fault-injected / integration-tested | held |
| Retry binds actor/base/complete snapshot | SQL unique key and pure prior-op decision | review-profile:requester retry/idempotency | Same key exact replay even after moved head; changed base/export conflicts; concurrent duplicate response equality → integration-tested | held; no exactly-once external generation claim |
| Crash/reconnect and acknowledged-loss recovery | Store pool, transaction, actual binaries/runner | review-profile:requester restart | DB socket terminated after INSERT/before commit; zero partial row and same-key retry; SIGKILL host + PG restart + real HTTP export/actor/time equality → fault-injected / integration-tested | held; arbitrary cancellation-at-commit scheduling not exhaustively modeled |
| HTTP errors/body/origin and diagnostics | http.rs; schema/fixtures/shared DTO tests | policy:cantos-engineering HTTP boundary | Actual response schema validation, unknown body/numeric path, invalid multi-fault IR and Origin denial → example-tested / integration-tested | held; stable code/path only, no SQL/manuscript values echoed |
| Draft and pending retry survive failure/races | Studio editor.rs | review-profile:requester; policy:cantos-leptos-web | Conflict/rebase, late ticket, edit during save, unknown outcome then expired auth, changed actor/cookie ack and Dirty status → example-tested (production reducer, no fake server) | held for in-page state; reload/close recovery deferred |
| Concrete Studio consumer is live | Studio api/view/WASM build; Axum static host | review-profile:requester minimal live consumer | Real local sign-in/read/edit/Return save, revision2/3 actor/time from PG → interaction-tested | held; no mock or full scene editor claimed |
| Tokens, Vietnamese, keyboard/focus, themes/scaling | token JSON/generator/CSS, locale catalog, native HTML | policy:AGENTS.md UI verification; doc:docs/design/ui-system.md | Independent contrast/drift tests; AX/input and inspected wide/compact light/dark 200% captures → accessibility-checked / screenshot-inspected / cross-theme-inspected / cross-viewport-inspected | held for named matrix; exact CSS instrumentation, VoiceOver and OS IME are gaps for broader UI adoption |
| Read inventory and DBSP deferral are honest | postgres/http/migration, read evidence | policy:cantos-engineering read-performance; review-profile:requester prompt2 only | All affected GET/internal fetches traced with indices/scopes/feeding writes → documented/source-inspected | held for inventory/defer; targets, EXPLAIN and measurement NOT_RUN per explicit boundary |
| Tooling/docs correspond to actual runners | workspace manifests, app/root/contract/architecture/product/UI/queue docs, scripts | policy:cantos-work-item docs update; policy:AGENTS.md verification | Bootstrap links/JSON/skills, 47 Python tests, native/WASM fmt/lint/tests, Trunk, token drift and git diff check → statically-checked / example-tested / compiled | held; mutation workspace copy fixed for real Web member; no remote CI implied |
| Media/publication/listener/native rules | untouched targets | policy:AGENTS.md architecture | No route/provider/media/worker/mobile implementation added | not-applicable: no production approval, synthesis or playback in this slice |

## Resolved findings and remaining coverage

- [P1] Dead PostgreSQL socket could be lent again after transaction termination — class:
  confirmed defect; category: logic; confidence: confirmed-from-source and reproduced in the
  real killed-connection test. Fast recycling was replaced by verified recycling. The original
  rollback/read/retry scenario then passed on PG17.4.
- [P1] Expired authorization after an ambiguous save discarded retry identity — class:
  confirmed defect; category: logic; confidence: confirmed-from-source with a failing-first
  reducer regression. Pending actor/base/key/export now survives denial; another actor cannot
  retry it and an unexpected accepted actor does not clear it. Regression passed.
- [P2] Editing after save still displayed Saved — class: confirmed defect; category: logic;
  confidence: confirmed-from-source and observed in the live workflow. Dirty transition and its
  regression passed, and the live build visibly reported unsaved text before saving revision3.
- Expanded workspace broke the disposable mutation baseline — class: confirmed defect;
  category: evidence. The runner now copies the Web consumer without dist and scopes execution
  to the actual server Script IR test. Baseline12 plus all seven killed mutations passed.

No remaining confirmed in-scope blocker was found. The full 010 journey, production identity/
rights gates, exact viewport/OS IME/screen-reader coverage, durable browser recovery and measured
read targets are not accepted by this result. Source-reviewed page-return reapplication is
best-effort; the final Safari navigation could recreate WASM state, so it is not claimed as
durable draft recovery or a confirmed bfcache-specific fix.

## Checks and provenance

[Execution evidence](script-revision-persistence.md#commands-and-execution-results) contains
exact commands, local log locations, PASS/FAIL/NOT_RUN and version/fixture provenance. Final
native tests23, Python tests47, real PostgreSQL tests9, Script IR mutations7, native/WASM Clippy,
fmt, oracle, token drift, Trunk and diff checks passed. Ordinary ignored PG tests were separately
enabled against the disposable real cluster; ignored/zero-test binary/doc targets were not
miscounted as integration evidence. No checks were weakened to pass.

Review inputs match the final code checks. The token/theme/auth-retry builds were recompiled;
the final pageshow-only addition was built/Clippy-checked and a fresh native page read the live
export. Detailed theme/zoom/keyboard captures used the same substantive UI before that listener
addition. PG source/query/migration/test assertions were unchanged since their passing run;
explicit Tokio features do not change those assertions. Docs-only evidence corrections were
followed by bootstrap and diff hygiene checks.

Residual risk: this single local reviewer and bounded tests are not a security audit or full
phase acceptance. Development auth/TLS/provisioning, rights eligibility, ID continuity across
documents, browser durable recovery, strict in-flight role revocation, full UI matrix and
performance/remote CI remain unverified; no production, provider, audio or native claim is held.
