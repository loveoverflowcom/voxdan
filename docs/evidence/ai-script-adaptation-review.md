# AI script adaptation review

## Scope and assessment

Read-only review of [issue #3](https://github.com/loveoverflowcom/cantos/issues/3), 2026-10-10,
branch `feat/ai-script-adaptation`. BASE, local HEAD and freshly fetched `origin/develop` were
`adfa82ce72a57d513f55c89ba7a58883af652cb2`. This was the working-tree diff against BASE plus
new nonignored files, not a three-dot PR diff.

The reviewed 51-path snapshot SHA-256 is
`e35a62bc4ec98b6df1dab0fd161cef8fc76a3de52ba0060f2d7330dce3c270d2`.
Input: sorted union of `git diff BASE --name-only` and
`git ls-files --others --exclude-standard`, excluding this report. Hash each UTF-8 path, NUL,
decimal byte length, NUL and exact file bytes in sequence. Ignored runtime/build evidence is
outside that fingerprint and cited separately. Production code did not change between the
successful final tests and this snapshot.

**Assessment: changes-requested**, solely for required real-model acceptance **NOT_RUN**.
No actionable confirmed technical defect remains in the declared source scope. Keep the PR
draft and issue #3 open. This report is not approval or a production-readiness certificate.

Coverage: **complete-for-declared-scope** for the changed source/contracts/tests/docs below.
The current issue was independently fetched read-only. The reviewer read executor logs and
observations, did not rerun tests or operate Safari, and performed no Git mutation, database
write, provider call, PR post or approval. This report is the only authorized reviewer write.

| Target | Reviewed scope |
| --- | --- |
| Rust server and worker | `adaptation.rs`, `adaptation/provider.rs`, `diagnostics.rs`, `postgres/adaptations.rs`, migration 0004, shared save/auth/import helpers, host and HTTP routes; trust, dispatch, recovery and acceptance |
| Leptos / Cantos Studio | `apps/web/src/adaptation.rs` and children, adaptation messages, API/import/view wiring and shared CSS; reducer state, comparison, edit, receipt and recovery |
| Contracts | Shared DTOs/wire tests, both adaptation schemas, original synthetic Vietnamese fixtures and contract doc; existing Script IR/revision/source consumers traced |
| Scripts / CI | Fresh-PG runner, fixture HTTP server/tests and workflow selection; independent oracles, process faults and benchmark |
| Docs / config / dependencies | READMEs, environment example, manifest/lockfile changes, architecture/pipeline/queue, proposed ADR 0006 and evidence |
| Theatre / CMP common / Android / iOS | Not applicable: no listener/playback/mobile changes |
| Skills / templates | No changes; owning skills/references read as review criteria |

Generated dependency records were inspected for the concrete reqwest/tower-http consumers,
not audited as an entire dependency supply chain. ADR 0006 remains proposed, not an accepted
authority that can waive issue criteria. Captures were inspected by the executor in tool output;
no persistent screenshot artifact or independent reviewer browser session is asserted.

## Assurance ledger

All executed evidence below is executor-produced and **not rerun** by this reviewer.
See [implementation evidence](ai-script-adaptation.md) for exact observations and limits.

| Claim | Owner / target | Required by | Evidence held → level | Disposition / next action |
| --- | --- | --- | --- | --- |
| Domain remains independent of provider/HTTP/UI; source/revision authority is reused | Pure admission, provider port, thin routes, existing store | policy:AGENTS.md architecture boundaries | Construction/caller trace → source-inspected | held |
| Original bytes/checksum/extraction and prior accepted exports survive failures | Source registry, frozen input, revision store | issue:#3 preserved source; policy:AGENTS.md immutability | Exact bytes/digests/exports, migration and process/DB restart → integration-tested, fault-injected | held |
| Source permission claims and explicit adaptation authorization precede dispatch; publication rights remain pending | Start gate and immutable rights record | issue:#3 rights/publication criterion | Missing/blank claims and false consent cause zero dispatch → integration-tested | held; creator claims are not legal eligibility |
| Creator authorizes exact provider/model/configuration | `expected_provider`, frozen input, UI consent reset | issue:#3 verified provider; review-profile:requester destination limits | Metadata drift variants and queued drift → example-tested, integration-tested | held |
| Local daemon cannot silently select cloud/remote inference | Verified connect, pre-chat probes, host startup | review-profile:requester no unauthorized destination/paid call | Cloud/alias/missing-proof/changed-weight/unverified-constructor/redirect zero-chat cases → actual-adapter example-tested, integration-tested | held within managed, stable daemon assumption |
| Provenance binds source/base/provider/prompt/config/run/generation | Frozen a1 digest and trusted evidence records | issue:#3 input provenance; policy:AGENTS.md | Binding/digest assertions and restarted records → example-tested, integration-tested | held |
| Model output cannot supply authoritative IDs, rights, assets, lifecycle or tools | Private closed DTOs and shell materialization | policy:cantos-script-ir hostile adaptation boundary | Schema/extra-field/forged-binding tests; all construction callers traced → type-enforced at decoded proposal boundary, example-tested | held |
| Candidates and edited acceptance use actual Script IR structural/semantic validation | `admit_output`, `admit_edited_proposal`, existing reader | issue:#3 structured validation | Independent schema and semantic cases through production functions/store → example-tested, integration-tested | held |
| Unknown speakers, emotion/cues, pacing and source coverage stay reviewable proposals | Findings/coverage and source comparison | issue:#3 inference and human omission review | Narrator/unknown-speaker/cue/citation cases; synthetic human review → example-tested, rendered-observed | held for mechanism; real quality remains live gap |
| Context/output/request/time/concurrency are bounded | Preparation, parser, transport, permit | review-profile:requester resource bounds | Oversize/depth/context/timeout and pre-chat rejection → example-tested, integration-tested | held as application policy; tokenizer/template fit unproved |
| Exact retries never silently dispatch the same run twice | Owner/op uniqueness, run lock, one persisted attempt | policy:AGENTS.md duplicate delivery; review-profile:requester | Concurrent replay, changed request, process/restart call-count oracles → integration-tested, fault-injected | held; no external exactly-once claim |
| Timeout/crash/expiry retain ambiguity without automatic redispatch | Attempt/deadline/observation protocol | policy:AGENTS.md crash/lease reasoning | Timeout, expiry, reopened store and actual host death → integration-tested, fault-injected | held |
| Cancel fences selection/acceptance and retains late usage | Cancel receipt, completion CAS, observations | policy:AGENTS.md cancellation | In-flight cancel/replay/late output → integration-tested | held; external computation stop not claimed |
| Acceptance is atomic with common authorized CAS/idempotent save | `accept_adaptation` and `save_in_transaction` | issue:#3 interrupted acceptance; review-profile:requester atomic save | Conflict/concurrent acceptance and commit-failure rollback → integration-tested, fault-injected | held |
| Ownership/revocation/expiry are current after lock waits | Authentication and source/script checks | policy:AGENTS.md permissions; review-profile:requester isolation | Other-owner/anonymous/revoked/deactivated/expired and lock barrier → integration-tested | held |
| Inputs/proposals/attempts/observations/receipts/history remain immutable facts | Migration 0004 and existing revision protections | policy:AGENTS.md immutability | App-role DML/TRUNCATE, owner trigger tests, corrupt-row re-admission → integration-tested | held while DB protections apply; not malicious-admin proof |
| Supplied usage survives rejected parsed output; money stays unavailable if absent | Output/problem, observations, shared DTOs/copy | issue:#3 usage/estimates/unavailable cost | Literal 19/23 and 87/123 counts preserved; null money → example-tested, integration-tested, rendered-observed | held; fixture counts are not billing evidence |
| UI guards actor/ticket/intent/source and retains dirty edits | Pure reducer and browser controller | policy:cantos-leptos-web; review-profile:requester stale/dirty state | 41 Studio tests; invalid JSON/edit/refresh observations → example-tested, rendered-observed | held; physical IME/tab-restart persistence not claimed |
| Accept pins returned revision and can reopen/recover | `RevisionResponse` consumer and receipt state | issue:#3 user journey | Reducer cases; Safari revision 1/reopen/run recovery → example-tested, rendered-observed | held with synthetic provider |
| Vietnamese/English, theme, focus/reflow and contrast remain usable | Messages/view/shared tokens | policy:AGENTS.md UI observations; policy:cantos-ui-design | Native Safari wide/narrow/200% zoom; CSS contrast calculation → rendered-observed | held for stated scope; assistive/mobile limits below |
| Private reads stay indexed/freshly authorized; no derived cache needed | Run/replay/source queries | policy:postgresql-read-performance acceptance | Inventory/invalidation audit, 1000 runs, 30 warm reads and index plans → integration-tested | held; DBSP deferred, no aggregate consumer |
| Existing Script IR/revision/import behavior stays compatible | Existing reader/save/import consumers | policy:AGENTS.md architecture/compatibility | Reference/native/WASM and 12 revision + 9 import PG cases; exact exports → example-tested, integration-tested | held |
| CI/docs/queue separate implemented mechanism from completion | Runner/workflow, evidence, work plan | issue:#3 delivery; policy:AGENTS.md evidence vocabulary | New PG suite selected; docs keep live gate and #4 boundary → source-inspected | held locally; remote CI not claimed |
| Creator reviews a REAL AI adaptation against preserved source | Available real model plus Studio acceptance | issue:#3 outcome/live evidence criterion | No available approved runtime/model; only doubles → NOT_RUN | gap-required; executor/maintainer obtains authorized runtime/model and live journey |

## High-impact boundary audit

The private provider DTOs deny unknown fields and cannot represent trusted evidence or host
capabilities. The application shell supplies trusted bindings; materialization and edited
acceptance call the actual Script IR reader. Edited acceptance compares the immutable
source/generation/rights envelope and existing work/adaptation/episode identities. All HTTP
callers reach these rules through the store. Structural citations do not prove meaning.

An attempt commits before network I/O, with no DB transaction spanning the call. Terminal CAS
and proposal insertion share a transaction; late output remains an immutable observation.
Acceptance invokes the common save helper and commits its receipt atomically with the revision.
Real PostgreSQL oracles inject failures and terminate a synchronized backend; the runner also
kills the actual host and restarts PostgreSQL. Ambiguity remains visible, not retried silently.

Loopback transport alone does not establish local inference: [Ollama can route cloud models
through its daemon](https://ollama.com/blog/cloud-models). Source-free status/show probes must
establish cloud-disabled local GGUF identity before startup and before source dispatch.
The fingerprint sorts independent parameter keys, retains repeated value order and exact
literal/directive content, and binds effective metadata/adapters/weight identity.
[Upstream rendering](https://github.com/ollama/ollama/blob/main/server/images.go) and
[quote grammar](https://github.com/ollama/ollama/blob/main/parser/parser.go) support those
normalization boundaries. Unknown/older metadata and ambiguous multiline PARAMETER rendering
fail closed. A managed, stable daemon is required; HTTP cannot attest a malicious local process.

## Findings

**[P2] Real-model editorial acceptance is missing** — class: required evidence gap · category:
evidence · confidence: confirmed-from-source-and-held-evidence.

Location: [live prerequisite](ai-script-adaptation.md#live-provider-prerequisite) and
[issue #3 outcome/criteria](https://github.com/loveoverflowcom/cantos/issues/3).
No callable approved model runtime or inference listener was available; no runtime/model was
installed. All generation evidence uses original synthetic Vietnamese fixtures and HTTP doubles.
The mechanism is exercised; real adaptation quality and the exact model/template/context
behavior are unproved. Keep draft/open and do not merge as complete.

Smallest next action: an explicitly authorized, available, cloud-disabled localhost Ollama
runtime with an installed supported local GGUF model; adapt a synthetic Vietnamese chapter,
record model fingerprint/prompt/config/input provenance, supplied usage and unavailable cost,
then compare/edit/accept/reopen in Studio and record editorial limits. This report authorizes
no credential setup, model download, paid call, private transfer or persistent runtime change.

Resolved findings, checked against final source and fresh regression logs:

| Finding | Repair and regression held |
| --- | --- |
| Loopback daemon could proxy a cloud model/alias | Verified local proof, frozen model digest, pre-chat recheck and awaited host connect; actual-adapter cloud/alias/changed-weight/unverified-constructor plus PG zero-chat cases |
| Arbitrary parameter ordering and incomplete quote parsing caused false fingerprint changes/rejection | Canonical independent keys, preserved repeated/literal order, both quote forms and `#` closing content; both final fingerprint transport regressions |
| Configuration changed after creator consent | Exact `expected_provider` and UI consent reset; PG metadata/queued drift and Studio tests |
| Rejected parsed envelopes discarded actual counts | `ProviderOutput.problem` retains counts without partial proposal; rejected-envelope transport and PG truncated-envelope tests |
| Narrator lookup by name mishandled renamed/reserved roles | Role-first narrator identity; renamed narrator and reserved-label admission tests |
| Required-nullable speaker/optional note admission disagreed with schema | Require speaker presence, reject null notes; independent schema/admission cases; stricter integer-token rule documented |
| Accept client decoded a revision as a run | Correct `RevisionResponse`, pinned receipt/reopen, dirty-text preservation; reducer and native Safari acceptance |
| Inherited transaction-start clock allowed stale session after lock wait | `clock_timestamp()` and authority rechecks; real lock-wait expiry/deactivation regression |
| Debug diagnostics leaked private values or changed established paths | Shared pure stable-code/path mapper; private-sentinel/path regression |
| English deterministic findings and fingerprint copy confused Vietnamese/evidence state | Localized explanations, inert attributed provider text, identity-versus-success copy; bilingual Studio tests/Safari observations |
| New HTTP helper failed Clippy large-error lint | Return `StatusCode` while retaining safe 400/413 behavior; final Clippy and hostile HTTP cases |

The clock issue was inherited and exercised by the new path, not attributed as a separate new
defect. An initial diagnostic test expected positional paths; its oracle was corrected to the
existing ID-based validator path without changing validator semantics. Initial failures are not
counted as final passes.

**Non-blocking assurance opportunity:** explicit truncation/shift controls or model-aware
tokenizer/template accounting could strengthen admission. No severity or additional gate is
created. Current contract/evidence clearly qualify the byte/framing policy; actual approved-model
behavior belongs to live acceptance. Larger-chapter chunking requires separate reviewed work.
No remaining source suspicion is promoted to a confirmed defect.

## Checks / freshness

All results are executor-produced, read but not rerun by reviewer.

| Check | Result and record |
| --- | --- |
| `cargo fmt --all -- --check` | passed; `target/adaptation-evidence/final-checks/fmt.log` |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | passed; final `clippy.log` |
| `cargo test --workspace --locked` | passed: 116 native tests; final `rust-tests.log`; 39 ignored DB cases ran separately |
| `python3 scripts/reference_script_ir.py` | passed; final `reference.log` |
| `TMPDIR=/private/tmp python3 scripts/mutate_script_ir.py` | passed: 9 mutations killed; final `mutation.log`; bounded set only |
| `cargo clippy -p cantos-studio --target wasm32-unknown-unknown --locked -- -D warnings` | passed; final `wasm-clippy.log` |
| `NO_COLOR=true trunk build --locked` in `apps/web` | passed; final `trunk.log`; these assets used in Safari |
| `python3 scripts/test_postgres.py --keep` | passed: 12 revision + 9 import + 18 adaptation; `final-postgres-run.log` and `target/revision-evidence/` suite logs; new disposable cluster, restricted app role |
| Host death / PostgreSQL restart | passed; PG log and `http-restart.json`; actual adapter with HTTP double, no inference |
| `TMPDIR=/private/tmp python3 -m unittest discover -s scripts -p 'test_*.py'` | passed: 63; `target/adaptation-evidence/final-python-unittests.log` |
| `python3 scripts/check_repository.py` | passed before report: 228 text files / 15 skills, executor-reported; repeat after this sole documentation addition |
| `git diff --check` | passed before report, executor-reported; final staged/report check remains executor-owned |
| Safari 18.6 / macOS 15.6 | passed for recorded synthetic scenarios; native input/accessibility observations and independent read-only HTTP oracle |
| Live provider/editorial journey | not-run; required gap |
| Remote CI / merge / postmerge CI | not-run at review snapshot; no protection/result asserted |

Safari observed source comparison, invalid JSON preserving state, dirty edit surviving refresh,
explicit Option-Tab/Return accept, read-only revision 1/reopen and run recovery after reload.
The independent HTTP oracle distinguished unchanged original source/proposal from the edited
accepted export and recorded exactly one fixture chat. Wide 1622×862 and narrow 660×852 pixel
windows were inspected. Confirmed 200% page zoom in a 640×852 window implies at most 320 CSS
pixels; this is an inference, not an instrumented viewport. Light/dark, vi/en and focus were
observed; CSS token contrast was independently calculated. No persistent PNG is claimed.

The final 1000-run, five-warmup, 30-serial-read benchmark measured p95 2.795 ms against 250 ms,
using primary/owner-operation indexes. This is warm in-process Router + real application-role
PostgreSQL, not sockets/cold/high-concurrency/production. Task-owned HTTP/double processes and
the fresh cluster were stopped; only the task Safari window was closed, per executor record.

## Test focus and residual risk

| Next scenario | Production seam / expected observation | Held layer / missing evidence |
| --- | --- | --- |
| Real Vietnamese chapter adaptation and review | Verified provider → admission → Studio; exact provenance, uncertainty and explicit accepted revision | Fixture mechanisms held; approved real runtime/model and editorial run missing |
| Timeout/crash/cancel followed by replay | Persisted attempt/terminal CAS; ambiguity or cancellation, late facts, no second dispatch | Real PG/process faults held; no external cancellation/exactly-once promise |
| Stale/concurrent/interrupted acceptance | Common save/receipt transaction; conflict or one replayed receipt, no partial head | Real PG fault/conflict evidence held |
| Model/destination drift | Safe preflight and frozen metadata; zero source chat | Actual-adapter + PG doubles held; managed stable daemon assumption |
| Dirty text/actor/reflow/assistive use | Reducer/tickets/source binding; preserve or gate draft, current immutable receipt | Native tests/limited Safari held; physical IME and assistive-device observations missing |

Residual risk: real model quality, faithful meaning, tokenizer/template fit, actual usage and
larger-chapter suitability remain unproved. The handshake trusts an approved stable daemon,
not malicious local processes or concurrent host changes. Rights assertions are not legal
clearance; editorial acceptance grants no publication permission. Production identity/TLS,
cross-document continuity, full Studio editor #4, casting/audio/publication and mobile remain
outside scope. Unsaved local edits are not promised across tab restart; saved runs/receipts
recover. VoiceOver, physical Telex/VNI, reduced-motion preference switching and mobile/device/audio
are **NOT_RUN**. Warm latency/token contrast do not establish production capacity or assistive
announcements.
