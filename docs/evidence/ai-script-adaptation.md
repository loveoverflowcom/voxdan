# AI script adaptation evidence

## Review boundary and provenance

Issue [#3](https://github.com/loveoverflowcom/cantos/issues/3) is the bounded consumer of the
[manuscript import](manuscript-import.md) and existing immutable revision store. The branch is
`feat/ai-script-adaptation`, based on `develop` at
`adfa82ce72a57d513f55c89ba7a58883af652cb2`. The source remains separate from every AI proposal
and accepted Script IR revision. Full Studio scene/character editing remains issue #4.

[ADR 0006](../decisions/0006-ai-script-adaptation.md) records the proposed production adoption
boundary. All runtime evidence here uses original synthetic Vietnamese content. No private
manuscript, third-party destination, persistent credential, paid generation, audio or deployment
is part of this work.

## Invariant ledger

| Claim / invariant | Owner | Failure mode | Required oracle |
| --- | --- | --- | --- |
| Imported source and prior revisions remain immutable | Source registry and revision store | Generation corrupts the source or accepted history | Exact bytes/checksum/export comparison on real PostgreSQL |
| Rights and destination authorization precede dispatch | Application gate and verified local runtime configuration | Unapproved disclosure or use | Missing rights/authorization plus cloud-enabled/aliased-remote zero-chat cases |
| Provider input and output are bounded untrusted data | Pure adaptation core and transport adapter | Injection, forged metadata or uncontrolled allocation | Closed wire/schema, malformed/oversized/deep output cases |
| Successful candidates pass actual Script IR semantics | Existing Script IR reader | Invalid speakers/cues/provenance appear accepted | Independent candidate assertions and validator errors |
| Retries never invisibly dispatch the same run again | Durable dispatch intent and terminal observations | Duplicate generation after timeout/crash | Concurrent replay, timeout and process death fault points |
| Cancellation fences acceptance while retaining observed facts | Run selection and attempt observations | Late output resurrects a cancelled run or erases usage | In-flight cancellation and late-result comparison |
| Human acceptance is atomic, authorized and tied to a base | Existing save transaction and acceptance receipt | Partial save, stale overwrite or duplicate revision | Real PostgreSQL rollback, conflict, revocation and retry cases |
| Studio shows pinned source, warnings and editable proposal | Leptos reducer/view | Stale response or unresolved information looks final | State regressions and actual keyboard/viewport observations |
| Provider, usage and cost claims match evidence | Attempt provenance and UI mapping | Fixture presented as live AI or unknown cost as free | Separate fixture/live record and exhaustive statuses |

## Live provider prerequisite

Live model acceptance is **NOT_RUN**. During environment discovery, no callable `ollama`,
`llama-server` or `lmstudio` executable was found, localhost `127.0.0.1:11434/api/tags` and
`127.0.0.1:1234/v1/models` refused connections, and Python modules `mlx_lm`, `transformers`,
`torch` and `llama_cpp` were absent. No model/runtime was installed or downloaded.

The host is an Apple M4 Mac mini (`Mac16,10`), macOS 15.6 (24G84), with 24 GiB
physical memory. Free space was about 10 GiB during discovery and 8.03 GiB after the
build/test evidence was retained. These measurements are capacity facts, not an inference
benchmark or proof that a particular model will fit. An approved model download, its cached
weights and runtime/context memory must fit the remaining resources; no model is selected here.

The exact prerequisite is an available, explicitly authorized localhost Ollama runtime/model
with cloud features already disabled and local GGUF model identity verifiable by the adapter,
then a synthetic Vietnamese chapter run through the real adapter and Studio comparison/edit/
accept/reopen journey. Record the exact model identity, prompt/config/input digest, provider
usage if supplied, unavailable cost as unavailable, and editorial limitations. Contract tests
against a localhost HTTP double do not satisfy this criterion. Until that gate is held, keep
the PR in draft, issue #3 open and merge/postmerge verification **NOT_RUN**.

## Execution record

Checks were run on the final production-code scope on 2026-10-10. Rust checks used the
locked workspace; PostgreSQL checks created a new disposable cluster with restricted
`cantos_app` grants, never an inherited or production database. Raw records are ignored under
`target/adaptation-evidence/` and `target/revision-evidence/`.

| Check | Held result | Evidence boundary |
| --- | --- | --- |
| `cargo fmt --all -- --check` | PASS | Final source formatting |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS | Native workspace/all targets |
| `cargo test --workspace --locked` | PASS: 116 native tests | 39 database cases separately executed below; 41 Studio reducer/message tests included |
| `python3 scripts/reference_script_ir.py` | PASS | Independent Script IR reference validator/encoder |
| `TMPDIR=/private/tmp python3 scripts/mutate_script_ir.py` | PASS: 9 mutations killed, none survived | Bounded existing validator/encoder mutation set, not mutation completeness |
| `cargo clippy -p cantos-studio --target wasm32-unknown-unknown --locked -- -D warnings` | PASS | WASM compilation/lint, not browser interaction |
| `NO_COLOR=true trunk build --locked` in `apps/web` | PASS | Final browser assets used for the observations below |
| `python3 scripts/test_postgres.py --keep` | PASS: 12 revision + 9 import + 18 adaptation cases | Fresh cluster, real migrations/grants/transactions and restricted-role reads/writes |
| Actual HTTP host death and database restart | PASS | Synthetic provider double; persisted dispatch intent becomes ambiguous, exact replay produces no second chat, retained source/export survive |
| Studio run/edit/accept/reopen | PASS with synthetic provider | Native Safari interaction described below; no model inference |
| Live model / editorial acceptance | NOT_RUN | Missing approved runtime/model |
| Merge and postmerge CI | NOT_RUN | Draft remains blocked on live acceptance |

The core/transport tests include malformed, oversized and semantically invalid output,
forged provenance, unknown speakers, cloud-enabled and remotely backed model rejection,
model/config drift and canonicalized metadata order. Real PostgreSQL oracles cover atomic
proposal/run/attempt persistence, owner isolation, authorization/session revocation, concurrent
idempotency, timeout, cancellation/late observations, conflict, save rollback and accepted
revision immutability. A transport double supplies synthetic token counts; its unknown cost is
stored/displayed as unknown. No external generation is claimed to be exactly once.

`final-checks/results.json` and command logs record the final Rust/reference/mutation/WASM
passes. `final-postgres-run.log`, `point-lookups.json` and `http-restart.json` record the fresh
database, benchmark and process-fault results. An earlier diagnostic test expected a positional
JSON pointer where the existing validator emits an ID-based path; its independent literal oracle
was corrected without changing the validator. A new HTTP helper's Clippy large-error finding
was resolved by returning `StatusCode`; actual malformed/oversized requests still return 400/413.
Repository/Python/whitespace checks are repeated after the final documentation/review record.

## Studio interaction observations

Native Safari 18.6 (20621.3.11.11.3) on macOS 15.6 ran the built Leptos assets against the real
Axum/PostgreSQL host on loopback. The in-app/browser DOM driver was unavailable; all browser
actions used native accessibility/input. Screenshots were captured and inspected in tool output;
they are not asserted to be saved PNG files or committed screenshots. Exact DOM viewport/touch
dimensions and screen-reader announcements were not instrumented.

| Scenario | Observed result |
| --- | --- |
| Provider absent | Clear local-runtime prerequisite; rights authorization disabled |
| Configured synthetic adapter | Model/destination/fingerprint/prompt/config shown; configuration proof explicitly distinguished from successful generation |
| Open source and request | Source ID, exact checksum, original CRLF text, three ordered extraction blocks and unresolved-speaker warnings displayed; Option-Tab/Return activated the request with visible focus |
| Refresh successful run | Narrator, Mai, unknown-speaker placeholder, three cue kinds, review findings/pacing note and all three coverage records retained; reported fixture tokens 87/123 and unknown cost displayed |
| Invalid edited JSON | Explicit review plus keyboard acceptance returned `$ · InvalidDocument`; saved run/revisions and local text were preserved |
| Edit/reset/refresh | Undo restored JSON and reset the review checkbox. A Vietnamese dialogue edit survived status refresh |
| Accept edited proposal | Explicit review and Option-Tab/Return saved revision 1; read-only receipt showed actor/time/export digest |
| Reopen and page reload recovery | Exact accepted export reopened; entering the stored run ID after reload recovered provenance and the accepted receipt |
| Wide/narrow layout | Inspected 1622×862 and 660×852 pixel window captures at normal zoom; source/proposal comparison and long metadata wrapped without observed overlap |
| Text scaling/reflow | Safari Page Menu confirmed 200% zoom; a 640×852 pixel window implies at most 320 CSS px of layout width. Light/dark checkbox, accept/reopen labels and coverage wrapped without observed horizontal page overflow; this is a dimensional upper-bound inference, not an instrumented 320 px viewport |
| Language/theme/focus | Vietnamese and English surrounding copy, unchanged Vietnamese source/provider notes, light/dark rendering and visible keyboard focus observed; original language/system theme/100% zoom restored |

The accepted test run was `0fd1ffea-b206-4c4a-b711-c715452ebc97`, script
`bbb7559f-84b9-4f4b-bef8-5b1453d6c306`, revision 1, export digest
`sir-e1:sha256:4a4f54c0d121c13e5b38147827c6ff9513a7bc934fd41b01dc2e8fc8a2e2883d`.
An independent read-only HTTP oracle confirmed the source checksum remained
`b242e7ce5c677751d5bcd7384ddb0698b740c30294978e4aa5a1a46b929a24f2`, the stored proposal still
contained “Chúng mình sẽ chờ ở đây.” and only the accepted export contained the test edit
“Chúng mình sẽ chờ ở đây, dưới ngọn đèn.” The double recorded exactly one chat request,
two status probes and two model probes for this UI run. These are synthetic fixtures, not AI
quality evidence. `studio-run-before-edit.json`, `studio-accepted.json` and
`studio-fixture-oracle.json` retain the ignored records.

An independent linear-sRGB calculation from actual CSS tokens found minimum light/dark ratios:
body/field text 13.665/12.897, help text 6.935/8.321, primary labels 7.612/8.426, links
6.458/8.722, borders 4.081/5.544 and focus rings 6.935/10.235. Token contrast passes the
4.5:1 text and 3:1 border/focus thresholds. This does not measure native checkbox/disabled
rendering, touch bounds or assistive announcements. VoiceOver, OS Telex/VNI composition,
reduced-motion preference switching, mobile/device behavior and audio are **NOT_RUN**.
Safari's default Tab policy skips buttons; Option-Tab was used without changing browser/system
preferences. Only the task-created test window was closed; earlier user windows were retained.

## Read-path inventory and optimization decision

| Consumer | Source / query shape | Authority and freshness | Scale / decision |
| --- | --- | --- | --- |
| Provider information | Configured verified adapter metadata; no manuscript read | Current authenticated session; configuration fingerprint | One provider; no cache |
| Start / operation replay | Owner-operation unique lookup, owned source ID, optional authorized script head | Current actor/session; frozen full input; transaction | Bounded point reads; no aggregate |
| Run refresh / recovery | Owner-run ID and indexed proposal/observation/receipt lookups | Source owner and current script access; expiry reconciled under lock | Bounded source/proposal; no list or N+1 |
| Proposal acceptance / exact retry | Locked owner-run, acceptance receipt and existing revision save | Current session, script authorization, immutable evidence, CAS head | Commit defines freshness; no derived cache |

Session expiry/revocation, actor deactivation, script authorization/head changes, source identity
and run state are the invalidation dependencies. They are read from PostgreSQL rather than held
in an application cache. Every request repeats the current authority checks, including after
lock waits. No repeated aggregate consumer or measured pressure currently justifies DBSP or an
incremental view. The held benchmark used 1000 synthetic runs (118-byte source, 2827-byte
export), five warmups and 30 serial in-process Router reads against real PostgreSQL. Measured
p50/p95/p99 were 2.474/2.795/2.863 ms; the 250 ms p95 target passed. Query plans used primary
and owner-operation index scans. Warm serial fixture latency is not cold/socket latency,
concurrent load or production throughput. Keep DBSP/CDC deferred until a concrete read consumer
and measured pressure justify it.

## Residual risk

Live model quality, full authoring, production identity/TLS, legal rights eligibility,
cross-document continuity, casting, audio, publication and mobile remain outside the held
evidence. Automatic source citation coverage cannot establish faithful meaning, complete
omission detection or correct unknown-speaker attribution.
Local runtime verification trusts the managed daemon and stable configuration; it cannot prove
the behavior of a malicious process impersonating localhost. Unknown/older daemon metadata is
blocked rather than treated as evidence of local inference.
The conservative application context guard does not prove rendered-template/tokenizer fit or
the daemon's truncation/shift behavior. Live acceptance must check these with the exact approved
model; explicit runtime truncation controls remain a recommended follow-up.
