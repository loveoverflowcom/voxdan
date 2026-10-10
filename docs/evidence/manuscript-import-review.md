# Manuscript import assurance review

## Scope and assessment

Independent read-only review of issue #2 against base and merge-base
`702bb4c764ee2a7a4f99a2d82c82386f18d8a9ab`, using the tracked diff and new files before commit.
The final implementation snapshot has SHA-256
`d998d97802d65b5d8baa2ae21bba8e89e52a108d69945bd69402c85388921f07`.
The fingerprint covers the 36 changed implementation files under `apps/`, `contracts/`,
`scripts/`, `.github/` and Cargo manifests/lockfile: sorted UTF-8 paths, each followed by NUL
and its binary SHA-256, then SHA-256 of that concatenation. Documentation and captures are
reviewed evidence, outside this implementation fingerprint. The delivery PR pins the commit.

Assessment: **no-actionable-findings**. Coverage: **complete-for-declared-scope**.
This means no sufficiently grounded remaining finding in this bounded local import slice;
it is not a safety certificate, approval or acceptance of all prerequisite #1.

The reviewer inspected the deterministic parser, shared wire/schema and independent fixtures,
HTTP authorization/body/error adapters, PostgreSQL migration/store/immutability and fault tests,
Studio reducer/view/messages/CSS, CI/scripts, affected documentation, final logs and five
captures. Rust server, contracts and Leptos Cantos Studio are covered. No worker, Theatre,
CMP Android/iOS, provider, audio or publication implementation is changed. The reviewer did not
rerun expensive checks or independently operate Safari; fresh executor logs and interaction
observations have that distinct provenance. GitHub delivery/CI remains the executor's gate.

## Assurance ledger

All held evidence below was inspected by the reviewer, **not rerun** in that review. Exact
commands, bounds, output locations and live observations are in
[manuscript-import.md](manuscript-import.md).

| Claim | Owner / target | Required by | Evidence held → level | Disposition / next action |
| --- | --- | --- | --- | --- |
| Original bytes/checksum stay separate from extraction | Parser, source store and original HTTP route / server | issue:#2 original source; policy:AGENTS.md provenance | Independent byte/hash fixtures and download comparison → integration-tested | held |
| TXT, Markdown, DOCX and Script IR preserve inspectable Vietnamese | `imports.rs`, wire fixtures / server and contracts | issue:#2 supported formats | Independent expected blocks, DOCX text and strict Script IR goldens → example-tested; differentially-tested | held |
| Unknown speakers and heuristic cues remain review information | Extraction DTOs and Studio comparison | issue:#2 uncertainty; policy:AGENTS.md Script IR | Literal expected uncertainty and warnings → example-tested | held |
| Hostile uploads are bounded without execution or external fetch | ZIP/XML/encoding parser and HTTP limits | review-profile:requester file hazards | Malformed encoding, archive paths/count/expansion/symlinks, DTD/entity and nested text corpus → example-tested | held; coverage-guided fuzzing not run |
| Only the current authorized actor can import/read originals | Existing session boundary, store and Studio receipt admission | policy:AGENTS.md backend permissions | Wrong/revoked/expired/deactivated actor, forged header, lock-wait recheck and changed-cookie acknowledgement tests → integration-tested; example-tested | held for local identity |
| Exact retries and concurrency preserve one immutable receipt | Actor/operation index, advisory transaction lock, immutable intent | review-profile:requester retry/concurrency | Concurrent duplicate requests and changed metadata/bytes conflicts → integration-tested | held |
| Source registry and receipt commit atomically | PostgreSQL single transaction | review-profile:requester rollback | Insert failure and terminated transaction with recovery → fault-injected (local PostgreSQL) | held |
| Receipts/originals survive application and database restart | PostgreSQL and actual HTTP process | issue:#2 reopen after restart | Kill/restart host and new disposable cluster restart, migration replay, independent download/receipt comparison → integration-tested; fault-injected | held |
| Existing accepted sources/revisions and rights authority remain intact | Migration 0003, editorial boundary and privileges | policy:AGENTS.md immutability; issue:#1 prerequisite | Seed 0002 upgrade comparison, forbidden mutation and no head/review transition → integration-tested | held; no rights eligibility granted |
| Wire values and independent safe field diagnostics match consumers | Shared closed enum/schema, HTTP errors, Studio | issue:#2 recoverable errors | Literal wire cases, rejected cue kind, two independent PG HTTP field issues and retained-file UI recovery → example-tested; integration-tested; interaction-tested | held |
| File selection, comparison, reopen and recovery work with keyboard/scaling | Leptos Cantos Studio | policy:AGENTS.md UI observations | Safari file chooser, Option-Tab/Return, final field recovery, light/dark and narrow 200%/250% captures → interaction-tested; accessibility-checked | held for observed bounds; no VoiceOver/IME claim |
| Affected reads use bounded authoritative point lookups | Source/actor/operation reads / server PostgreSQL | policy:cantos-read-performance | Restricted app-role plans and 1,000-source warm Router benchmark → integration-tested | held; production/high concurrency not measured |

The point-read evidence is copied from the final run to
[manuscript-import-read-results.json](manuscript-import-read-results.json): 30 samples,
concurrency 1, p95 1.037 ms against the declared warm 250 ms target. This excludes network,
cold cache and production load. No aggregate consumer justifies adding DBSP.

## Resolved findings

These defects were confirmed from source, corrected, and checked again. They are historical
findings, with no remaining actionable finding. Each is class **confirmed defect** and
confidence **confirmed-from-source**, supported by the stated regression evidence.

| Finding | Category / location | Trigger and consequence | Resolution and evidence held |
| --- | --- | --- | --- |
| [P1] Valid source IDs could not reopen in Studio | compatibility / `apps/web/src/import.rs` | A saved `src_` ID failed UUID preflight | Match the server's lower-hex source ID grammar; reducer tests |
| [P1] DOCX hyphens silently joined words | logic / `apps/server/src/imports.rs` | `noBreakHyphen` or `softHyphen` disappeared during extraction | Preserve U+2011/U+00AD; independent expected-text case |
| [P2] Selecting another source retained old receipt/error state | logic / Studio reducer | Changed source ID displayed facts from a previous selection | Clear stored receipt/status on selection; state-transition regression |
| [P2] A warning-free page implied the whole extraction was warning-free | logic / Studio comparison | Warnings existed only on another 100-block page | State current-page/global warning scope and other-page possibility; paging regression and ARIA description |
| [P2] Cue kind accepted values outside the closed wire contract | compatibility / shared import DTO | A raw string admitted a value the schema rejected | Shared closed `ImportCueKind`; literal wire/schema rejection cases |
| [P2] Metadata errors lacked independent actionable field details | logic / metadata validation and HTTP adapter | Multiple byte-length/blank fields yielded generic diagnostics | Collect safe JSON-pointer/rule issues; unit, real PG HTTP and final Studio retained-file recovery |
| [P2] Changed-cookie acknowledgements could display another actor's receipt | security / Studio response admission | A receipt completed after browser actor changed | Verify `imported_by`, hide receipt and clear busy state; reducer regression; backend remains authoritative |

## Checks and remaining scope

Fresh final executor evidence inspected: workspace fmt/Clippy and tests (78 non-ignored cases),
Python tests (60), both independent Script IR reference goldens, mutation run (nine killed,
zero survivors/timeouts/unviable), new-cluster PostgreSQL revision/import suites (12 + 9),
actual HTTP/restart checks, WASM Clippy, Trunk build and whitespace check: **PASS**.
The executor runs the final repository/link check after saving this report; its result is
recorded in the main evidence and delivery PR. Ordinary Cargo runs ignore the 21 PG cases;
the separate cluster command ran them. Captures were inspected, not treated as interaction proof.
Some captures precede the final fixes for unrelated view state; final light comparison,
250% warnings and field diagnostics were repeated. The effective narrow-width bound is an
inference from native window width and selected zoom, not an instrumented CSS measurement.

Pre-existing prerequisite #1 remains partial: production identity/TLS, legal rights eligibility,
cross-document identifier continuity and full authoring acceptance have not been delivered.
The review does not close #1. AI adaptation #3 and complete Studio editor #4 remain next work;
no code for either begins in this PR.

Residual risk: bounded DOCX extraction is not visual fidelity; tracked changes and legacy
Vietnamese fonts need an explicit future conversion choice. Coverage-guided fuzzing, VoiceOver,
Telex/VNI composition, mobile, providers/audio/publication, production identity/TLS, rights
adjudication, production load and backup restore are NOT_RUN. Unsaved browser file selection
does not survive page closure; saved immutable receipts reopen by ID and uncertain writes retry
their exact operation. Normal repository merge protections and exact-SHA CI remain delivery gates.
