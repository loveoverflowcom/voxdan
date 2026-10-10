# Manuscript import evidence

Review boundary: [issue #2](https://github.com/loveoverflowcom/cantos/issues/2), one authorized
creator imports a Vietnamese chapter, compares preserved source and extraction, and reopens
the immutable receipt after backend/database restart. Base: `702bb4c` on `develop`.
[ADR 0005](../decisions/0005-manuscript-import.md) is proposed and implemented for local review.

## Prerequisite #1

The base has an executable Script IR 0.1.0 validator, canonical c1/e1 policy, shared Studio
DTOs, immutable PostgreSQL revisions, current owner/editor/reader authorization, CAS/replay
semantics, source/history/review operations and a minimal Leptos consumer. The exact base
[push workflow](https://github.com/loveoverflowcom/cantos/actions/runs/38034933046) succeeded.
Code and existing historical evidence were inspected; an unchecked or open issue was not used
as proof of either implementation or acceptance.

The immediate gaps for import were UTF-8-only source storage and the absence of an authorized
browser intake path. This PR extends the shared source boundary rather than replacing revision
authority. Production identity/TLS, legal rights eligibility and complete authoring acceptance
remain outside this boundary. Stable identifiers are validated within a document; continuity
across separately authored documents remains a separate design requirement. No full acceptance
or closure of #1 is claimed.

## Invariant and ownership

Original upload bytes, checksum, provenance and claimed permission evidence remain immutable.
Normalized extraction is separate, and uncertain speaker/scene information remains visible.
A denied, invalid, interrupted or repeated request cannot overwrite accepted source/revision
facts, expose another actor's source or produce a misleading ready script.

`imports.rs` is a deterministic parser with no HTTP, SQL, provider or UI dependency. Raw import
DTOs are shared by two concrete consumers. PostgreSQL resolves current actor facts and commits
source registry, bytes, metadata and parse outcome in one transaction. The HTTP adapter maps
requests/errors; the Leptos component renders those facts and uses an immutable retry intent.
No new service, persistence authority, generic port or domain crate is introduced.

## Claim and oracle ledger

| Claim / failure mode | Cheapest adequate oracle | Evidence target |
| --- | --- | --- |
| Four formats preserve inspectable Vietnamese content | Independent synthetic TXT/Markdown/DOCX and strict Script IR fixtures | example-tested; differentially-tested |
| Unknown speakers and approximate cues do not become settled identities | Expected block kinds, unresolved values and conversion warnings | example-tested |
| Hostile files have bounded, actionable failure outcomes | Encoding, size, ZIP count/expansion/path/symlink, XML/DTD and malformed corpus | example-tested |
| Originals and source claims survive reads/restart | Independent SHA-256 and exact HTTP download before/after process and PG restart | integration-tested; fault-injected |
| Retries/competing submissions never duplicate receipts | Concurrent PG requests, changed bytes/metadata conflict and row counts | integration-tested |
| Current actor scope is authoritative | Anonymous/wrong actor/revoked/expired/deactivated sessions and forged actor header | integration-tested |
| Registry/source commit atomically | INSERT failure and terminated transaction, same-operation recovery | integration-tested; fault-injected |
| Existing accepted sources/revisions remain unchanged | Seed 0002 data, apply 0003, compare bytes/hash/revision identity | integration-tested |
| Persisted source/outcome integrity fails loudly | Original-byte and stored outcome corruption through test owner | integration-tested |
| Studio import/reopen/retry remains usable | Reducer regressions and live Safari file selection, comparison and keyboard flow | example-tested; interaction-tested; accessibility-checked |

## Verification record

Local environment: macOS 15.6 / Apple Silicon, Rust 1.87.0, Trunk 0.21.14, PostgreSQL
17.4 (Homebrew), Safari 18.6. The final implementation commit is pinned in the PR description;
checks run against the same source files before that commit. The final review is recorded in
[manuscript-import-review.md](manuscript-import-review.md).

| Result | Exact command / observation | Scope |
| --- | --- | --- |
| PASS | `python3 scripts/check_repository.py` | Documents, fixtures, links and canonical skills |
| PASS | `TMPDIR=/private/tmp python3 -m unittest discover -s scripts -p 'test_*.py'` | 60 repository-script tests |
| PASS | `git diff --check` | Patch whitespace |
| PASS | `cargo fmt --all -- --check` | Workspace Rust formatting |
| PASS | `cargo clippy --workspace --all-targets --locked -- -D warnings` | Native workspace targets |
| PASS | `cargo test --workspace --locked` | 78 native unit/contract/parser/Studio tests; 21 ignored PG tests are separately run |
| PASS | `python3 scripts/reference_script_ir.py` | Both unchanged independent c1/e1 goldens |
| PASS | `TMPDIR=/private/tmp python3 scripts/mutate_script_ir.py` | Nine validator/encoder mutants killed, none survived or timed out |
| PASS | `PATH="/opt/homebrew/opt/postgresql@17/bin:$PATH" python3 scripts/test_postgres.py --keep` | Fresh disposable PG cluster; revision/import suites, actual HTTP kill and PG restart, migration replay and exact receipts/downloads |
| PASS | `cargo clippy -p cantos-studio --target wasm32-unknown-unknown --locked -- -D warnings` | Final CSR compilation/lint |
| PASS | `NO_COLOR=true trunk build --locked` from `apps/web` | Final browser WASM/assets build |
| PASS | Native Safari user journey below | Live app-role Axum/PG, file chooser and keyboard interaction |
| NOT_RUN | Coverage-guided fuzzing, VoiceOver, Telex/VNI IME, mobile, production identity/TLS, provider/audio/publication runs | Outside this bounded local slice; no evidence claimed |

Ignored local logs and receipts live under `target/import-evidence/` and
`target/revision-evidence/`. `scripts/test_postgres.py` never uses an inherited database URL;
`--keep` only retains its newly created cluster for the live Studio inspection. The retained
host/cluster are stopped after inspection.
Ordinary Cargo tests do not run ignored PostgreSQL tests. All fixtures are original synthetic
text; no copyrighted user source, AI/TTS provider, external resource or media store is used.

## Studio runtime observations

The live host served the actual Trunk build at loopback using the restricted application role,
with synthetic development actor `alice`. No production identity or credential was created.

1. Selected `contracts/fixtures/manuscript/chapter-vi.txt` in Safari's native file chooser,
   supplied Vietnamese source metadata and recorded claimed rights separately. Keyboard
   Option-Tab and Return submitted the import. The receipt showed the original Vietnamese
   source, six ordered blocks, labels requiring review, an unknown dash speaker and ambiguous
   bracket content. The instruction-like fixture sentence remained ordinary source content.
2. Reloaded the page, signed in and reopened the same source ID using the keyboard. Receipt
   identity, metadata, timestamp and hash matched. The authenticated download was compared
   independently with the fixture: exactly 328 bytes, SHA-256
   `0e6e0eb790e1c805645bcbd5334cf44e6fa829e090c2050c3008796ba165abca`.
3. Opened the saved malformed UTF-8 fixture. The UI retained its source ID and original
   download, showed `invalid_utf8` at byte 0, and explained how to convert a copy and retry as
   a new import. It did not display a ready extraction.
4. Imported `contracts/fixtures/script-ir/0.1.0/accept/two-scenes.json` through the same file
   chooser. Eight ordered blocks displayed scene-01, ambience, narration, An/Minh dialogue,
   then scene-02 and its SFX. Full validated Script IR remained available for inspection;
   the revision editor head stayed at 0 and no automatic save/approval occurred.
5. Inspected light/dark captures at a 1619 × 862 native window and a 710 × 862 narrow window.
   Source/extraction columns stacked when narrow. Safari's Page Menu confirmed 200% zoom;
   labels/status text wrapped and the keyboard focus ring on reopen remained visible.
   A second narrow pass at a selected 250% zoom showed wrapped warnings and vertically stacked
   buttons. The native window width divided by zoom gives an upper bound of about 284 CSS px
   before scrollbar/chrome deductions, below the 320 px policy boundary. This is an inference
   from observed window/zoom values, not an instrumented CSS viewport measurement.
6. Reloaded the final build, kept the selected TXT file, and supplied two Vietnamese fields
   above their UTF-8 byte limits. Both `/metadata/reference` and `/metadata/rights_holder`
   appeared with `byte_length` and localized recovery guidance. Corrected both fields and
   submitted successfully through the keyboard without selecting the file again. Final
   light comparison and narrow 250% captures were repeated after the review fixes.

Captured images were inspected, then cropped by 80 pixels at the top and bottom to remove
browser chrome and unrelated tabs. No content was reconstructed or generated.

| Capture | Observation |
| --- | --- |
| [Light source comparison](images/manuscript-import/desktop-light-comparison.png) | Original Vietnamese text and extraction warnings |
| [Dark structured comparison](images/manuscript-import/desktop-dark-script-ir.png) | Scene order, typed ambience and narrator identity |
| [Narrow 200% focus](images/manuscript-import/narrow-200-focus.png) | Wrapping and keyboard focus on reopen |
| [Narrow 250% warnings](images/manuscript-import/narrow-250-warnings.png) | Warning text and buttons below the 320 px effective-width boundary |
| [Metadata field errors](images/manuscript-import/desktop-field-errors.png) | Independent localized field diagnostics and retained file |

Accessible labels and focus were observed through Safari's native accessibility tree and
keyboard. This is not a VoiceOver walkthrough, automated axe audit or mobile result. Browser
paste preserved Vietnamese diacritics; an OS input-method composition test was not run.

## Affected reads and DBSP decision

Import reopen/download reads current session/actor facts and one owner/source primary-key row.
Replay reads the actor/operation unique index. No unbounded list, per-block SQL, repeated
aggregate or join read model is added. PostgreSQL commit is the freshness boundary; reads have
no cross-actor cache. Revocation is checked in the existing transaction boundary.
The original is limited to 1 MiB and extraction to 2 MiB; production frequency is unknown.
The warm in-process HTTP Router benchmark used the application role, real PostgreSQL,
1,000 synthetic 1 KiB sources, five warmups, 30 samples and concurrency 1. The measured p50,
p95 and p99 were 0.863 / 1.037 / 1.060 ms against a declared warm p95 target of 250 ms.
Source/replay plans used the primary-key and actor/operation indexes; the tiny two-actor/session
fixture used sequential scans for authorization. Full plans and timings are retained in
[manuscript-import-read-results.json](manuscript-import-read-results.json), copied from the
final `target/import-evidence/point-lookups.json` run. This excludes network transport, cold-cache,
high-concurrency and production-load claims.
Keep authoritative PostgreSQL point reads. DBSP remains deferred: no measured hot aggregate
consumer justifies maintenance/CDC/serving complexity. A future list consumer or large-source
storage boundary must set its own representative workload and simpler PostgreSQL comparison.

## Compatibility, handoff and residual risk

Script IR read/write remains `{0.1.0}`; c1/e1 and existing goldens are unchanged. Import never
moves a script head, saves an editorial review, invokes AI or grants production/publication
eligibility. Permission evidence is a creator claim requiring later adjudication; unknown
rights survive intake. No cache/audio invalidation or billable call is added.

Migration 0003 adds import columns to `source_records`; old rows remain untouched. Text reads
retain their behavior. Binary imports use their authenticated original route. Runtime roles
need INSERT on source/evidence, with no source UPDATE/DELETE or credential issuance authority.
New host and migration should be used together for local import consumers. Import receipt
integrity uses a versioned r1 digest over typed canonical metadata/outcome plus original SHA;
future DTO fields or extractor versions must preserve the existing r1 decoding/preimage policy.

The next bounded work is [AI adaptation #3](https://github.com/loveoverflowcom/cantos/issues/3):
consume immutable source ID/checksum, extractor version, ordered blocks, warnings and recorded
rights claims. Preserve unresolved labels, source coverage and immutable proposal/attempt
identity; admit a complete validated document only through existing revision save. Full Studio
editing belongs to #4. Neither task begins in this PR.

Residual risk: production identity/TLS and rights eligibility remain unimplemented. DOCX is
bounded extraction rather than visual fidelity; tracked changes and legacy Vietnamese fonts
require a future explicit conversion choice. No full editor, coverage-guided fuzzer, VoiceOver
walkthrough, OS Telex/VNI input, mobile, provider, audio, publication, production load or backup
restore evidence is claimed. Unsaved file selection does not survive page closure; saved receipts
reopen by ID and uncertain submissions recover by exact retry.
