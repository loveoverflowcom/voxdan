# Independent final review of the Script IR revision sequence

Date: 2026-10-10. This review inspected the implementation and reproduced defects independently
of the earlier author reports. The requester authorized fixes, regression checks and one new
local commit; that authorization overrides the review skill's report-only default. Documents
remain in English under the repository instructions. No push, PR, merge, issue update,
credential change or remote change was performed.

## Scope and assessment

Two-dot endpoint review:

- BASE and merge-base: `ca4c9501b36e11dfbbc508a3394f71ded3211072`.
- Reviewed HEAD: `4f555c8b696de9436e692a293e8d0a4b1cb4fe00`.
- Intervening commits, in order: `7e393361f412b87a34b8121528de78416b721e8c`,
  `ea8e1d98a31301b0d63dfa75d8e56450397b581d`,
  `626416500599b852dc985683fa3a4ac905f9401f`,
  `ff6e3cc5c8f8506c904179b2d54ef1a6ebec1c39`,
  `4f555c8b696de9436e692a293e8d0a4b1cb4fe00`.
- Entry: clean local `develop` at reviewed HEAD. Fixes are on
  `review/final-sequence-fixes`, branched from that `develop`; history was not rewritten.
  Final tests below ran on the source changes accompanying this report, not just reviewed HEAD.
- A read-only `git ls-remote --heads origin develop` at 06:30 UTC returned
  `ea8e1d98a31301b0d63dfa75d8e56450397b581d`. This is a dated remote observation, not an
  assumption about subsequent changes. Local tracking refs were not refreshed.

Assessment at reviewed HEAD: **changes-requested** for seven confirmed defects below.
Assessment after these fixes: **no-actionable-findings** in the declared bounded local
Script IR/revision/Studio scope. This is not production approval or completion of #1/010.

Coverage: **complete-for-declared-scope**, at the module and claim boundaries below. The whole
117-path range was inventoried, including policy, manifests, migrations, fixtures, scripts,
CI and evidence. Core/server/store/HTTP/Studio functions, callers and tests were inspected,
not just patch hunks. Shape/canonical fixtures were checked through the literal corpus,
independent oracle and schema differential tests. The two retained benchmark reports were
audited through checksums, 144 run records and 144 curated plans. Individual raw timing samples
and every generated Cargo.lock entry were not manually read; dependency vulnerability analysis,
remote CI execution and GitHub issue acceptance were not part of this local review. Truncated
large reads were supplemented with focused function/diff reads; no conclusion relies on an
unseen tail of a patch. No binary media, changed symlinks or mobile/worker implementation was
present in this range.

| Target | Paths and claims reviewed | Evidence boundary |
| --- | --- | --- |
| Rust server/core | `script_ir/`, `revisions.rs`, `postgres.rs`, `http.rs`, binaries, migration 0001, tests | Native compilation, literal examples, schema/oracle comparison and real local PostgreSQL |
| Leptos Studio | `api.rs`, `editor.rs`, `view.rs`, messages, HTML/CSS/tokens and generator | Reducer examples, native/WASM compilation and bounded Safari interaction |
| Contracts | Script IR 0.1.0, Studio v1, shared DTOs, safe Vietnamese fixtures | Independent c1 goldens and shared-wire tests; no generated mobile bindings |
| Benchmark and tooling | Benchmark/test/runner/oracle/mutation scripts, workflow, environment guide | Guard tests and historical artifact audit; no new latency benchmark |
| Docs and skills | Read-performance routing, changed engineering/listening/review rules, product/architecture/queue, ADRs 0002/0003 and all sequence evidence reports | Document/source consistency; both ADRs remain proposed for production adoption |
| Theatre, workers, CMP/Android/iOS, publication | Their changed routing/policy links only | Runtime targets absent; no provider, audio, publication or device claim |

## Fixed confirmed defects

Locations identify the responsible code at reviewed HEAD, followed by the current fix seam.
All seven are **confirmed-from-source**: each causal path was checked against the introducing
commit and an explicit example or process double. No speculative finding was turned into a fix.

### [P1] Fractional intensity tokens silently became integers

Class: confirmed defect. Category: compatibility. Introduced in `ea8e1d98`.
Location: `apps/server/src/script_ir/wire.rs:163` at reviewed HEAD; current
`integer_intensity` / `exact_unsigned_integer` at lines 165–202.

The contract admits exact integer-valued JSON numbers and rejects fractions. Binary-float
conversion admitted `300.0000000000000001`, `1000.0000000000000001` and tiny nonzero exponents
such as `1e-9999` after rounding to an integer or zero. A valid-looking export then changed the
submitted numeric value. The regression failed on the first near-integer case before the fix
(`target/final-review/intensity-before.log`).

The reader now inspects the raw decimal token with checked integer arithmetic. Integer spellings
`300.0`, `3e2`, `300000e-3` and signed zero still work; fractions and nonzero negative values
fail. The table test uses literal spellings rather than constructing floats. c1/e1 encoding,
schema version and stored canonical integer bytes are unchanged. JSON Schema tools using floats
can lose precision, so the byte reader remains the numeric admission authority.

### [P1] Normalization admitted exports larger than the storage/read limit

Class: confirmed defect. Category: logic. Introduced in `ea8e1d98`, exposed by `62641650` storage.
Location: `apps/server/src/script_ir/mod.rs:14` at reviewed HEAD; current `read_script:14`.

The 2 MiB check only bounded input. NFC expands U+0344 into two combining characters: 110
dialogues of 5,000 such characters pass each normalized 10,000-scalar field bound, yet produce
a complete export above 2 MiB from an input below it. Admission returned content that the same
reader could not reopen; PostgreSQL's byte constraint turned a caller error into availability
failure. The exact `DocumentTooLarge` regression failed before the fix
(`target/final-review/export-before.log`).

Draft admission now bounds the complete normalized export before returning a trusted value.
The canonical reader checks equality against its already bounded input without an additional
encoding pass. The real PostgreSQL/HTTP test observes 422 `invalid_script` with
`DocumentTooLarge` and no provisional script row. No migration or stored bytes were changed.

### [P1] Repeated pronunciation targets multiplied retained validation memory

Class: confirmed defect. Category: security. Introduced in `ea8e1d98`.
Location: `apps/server/src/script_ir/validation.rs:106` at reviewed HEAD; current
`Validator::dialogue:108`.

The validator retained every occurrence span from every override, including rejected overlaps.
A small document with a 10,000-character `a` line and 10,000 identical `a` overrides retains
100 million pairs of `usize` values, about 1.6 GB of span payload on this 64-bit host before
vector capacity overhead. This synchronous admission path is reachable through a development
session; the HTTP future timeout does not preempt it. The allocation bound follows directly
from the source. The maximum pre-fix input was deliberately not executed. A bounded 512-override
pre-fix CLI example rejected as expected; its RSS collection was unavailable in the sandbox,
so no measured memory improvement is claimed.

The fix tracks occupied UTF-8 bytes once per dialogue and streams occurrences, preserving the
ordered diagnostics and union of occupied spans, including invalid overrides. For text within
the scalar bound the occupancy buffer is at most 40,000 bytes; invalid longer text remains
bounded by the document input/normalization limits. The maximum-array regression checks all
9,999 exact overlap diagnostics. This bounds retained span memory; it does not establish a
production CPU budget or fuzzing coverage.

### [P1] Inherited PostgreSQL configuration could escape the disposable runner

Class: confirmed defect. Category: security. Introduced in `62641650`.
Location: `scripts/test_postgres.py:82` at reviewed HEAD; current `isolated_environment:39`
and all subprocess invocations.

Only `DATABASE_URL` was removed; psql still inherited `PGHOSTADDR`, `PGSERVICE`, `PGOPTIONS`
and startup files, while the HTTP host inherited static-directory/origin overrides. libpq
configuration can change where an explicit hostname URL connects. This violated the mandatory
fresh disposable-cluster boundary. No connection to a production or inherited database was
attempted to demonstrate the risk.

The runner now removes `PG*`, `CANTOS_*` and `DATABASE_URL`, supplies that environment to every
subprocess, uses `psql -X`, and supplies its own host origin/static directory. A poisoned synthetic
environment test checks all listed keys are absent and PATH remains available. The final live
run initialized a new cluster and non-superuser app role successfully.

### [P2] Optimized Python could claim recovery without running its oracle

Class: confirmed defect. Category: evidence. Introduced in `62641650`.
Location: `scripts/test_postgres.py:103` at reviewed HEAD, plus its readiness assertions;
current entry guard at line 84.

`python -O` removes assertions, including HTTP calls inside them. The runner could report PASS
without comparing recovery/replay output. It now refuses before parsing arguments or starting
processes. The real optimized-interpreter regression first failed because `--help` exited 0,
then passed with the explicit refusal (`target/final-review/runner-before.log`). The benchmark
runner's existing optimized-Python refusal remains covered by its own tests.

### [P2] A readiness mismatch left the unreturned HTTP process alive

Class: confirmed defect. Category: logic. Introduced in `62641650`.
Location: `scripts/test_postgres.py:54` at reviewed HEAD; current `start_host:60`.

An unexpected readiness response raised before returning the local `Popen` handle, leaving the
caller's cleanup unable to stop it. The regression observed no kill/reap on the original path.
The helper now stops and reaps the process on any failure before transferring ownership,
including interruption. The regression uses process/HTTP doubles; it is example evidence, not
a separate live fault injection. The final real runner and browser host were also stopped.

### [P2] Changing Script ID retained the previous target's saved claim

Class: confirmed defect. Category: logic. Introduced in `62641650`.
Location: `apps/web/src/view.rs:122` at reviewed HEAD; current
`Editor::select_script` at `apps/web/src/editor.rs:65` and its view binding.

After a successful save, typing another Script ID reset the base to 0 and removed the stored
export but retained `Saved`. The visible draft had never been saved to that target. The
regression was run against the original transition factored into the reducer and failed
`Saved` versus `Dirty` (`target/final-review/studio-before.log`).

Target selection now preserves text, marks nonempty text dirty, and clears base/remote state.
Same-target selection and blocked selection during an unresolved save preserve their state.
The reducer example and the final Safari read/save/change-target walkthrough both passed.

## Assurance ledger

Requirements are the actual source rules or the requester's review scope, not new invented gates.
An evidence level describes only the stated target and seam.

| Claim | Owner / target | Required by | Evidence held → level | Disposition / next action |
| --- | --- | --- | --- | --- |
| Closed shape, exact numeric and document bounds | Script IR byte reader | doc:contracts/README.md#read-and-write-policy; review-profile:requester | Literal schema/reader tests, exact decimal and NFC expansion regressions → example-tested / differentially-tested | held; production parser/CPU fuzz budget remains separate |
| Validated content cannot be constructed through public serde or mutated in place | Script IR model, private wire/validator | policy:cantos-engineering boundary hardening | Public fields/constructors/serde/row bypass audit, Rust build → typed / compiled / statically-checked | held for inspected Rust API; no compile-fail or formal proof claim |
| Stable canonical c1 and complete export e1 preserve rights/provenance separately | Encoder, export reader, store | doc:contracts/README.md#canonical-bytes-and-digest-c1 | Unchanged independent goldens; metadata-only edits and corrupt-row tests → differentially-tested / integration-tested (local PostgreSQL) | held; no digest deduplication or rights eligibility claim |
| Actor/resource/action checks and revocation apply to head, history and retries | PostgreSQL authorization and HTTP | policy:AGENTS.md#architecture-boundaries; review-profile:requester | Owner/member/reader/foreign/expired/revoked tests and query inspection → integration-tested (local PostgreSQL) / statically-checked | held for development sessions; in-flight authorization semantics stated below |
| Append, evidence links and head update are atomic and immutable | Pure revision rules, SQL transaction/constraints/triggers | doc:contracts/studio-v1.md; policy:AGENTS.md#correctness-rules | Real competing writers, same-operation delivery, stale/key-reuse, rejected evidence, app-role mutation attempts → integration-tested (local PostgreSQL) | held; PostgreSQL owner/superuser remains trusted |
| Lost response, pre-commit death and restart preserve the same accepted identity | Store, runner and actual Axum binary | review-profile:requester; policy:AGENTS.md#verification | Killed connection, killed HTTP process, PG fast restart, migration replay, exact export/actor/time/retry checks → fault-injected / integration-tested (local PostgreSQL + HTTP) | held at these fault points; no paid provider/exactly-once claim |
| HTTP payload/origin errors fail without manuscript leakage or partial writes | Axum adapter, raw shared DTOs | policy:cantos-engineering HTTP boundary | Malformed/oversized/unknown payload and origin examples, exported-byte regression, error mapping inspection → integration-tested / statically-checked | held; production TLS/auth deployment absent |
| Snapshot/ticket/retry identity survives late completions and edits; target status is truthful | Studio reducer | doc:apps/web/README.md#interaction-and-ownership | Eight reducer/message tests → example-tested; shared wire fixture → differentially-tested | held for in-memory page state; reload persistence absent |
| Studio final UI can read/save and display dirty target state | Leptos/Safari | policy:AGENTS.md#verification; cantos-ui-design | Final WASM build, real backend keyboard actions, focused/compact/200% captures inspected → compiled / interaction-tested / screenshot-inspected / cross-viewport-inspected | held for observed matrix; screen reader/IME/device gates not claimed |
| Runner cannot silently skip assertions or inherit DB/host configuration; failed readiness is cleaned up | Python runner | policy:AGENTS.md#verification; review-profile:requester | Three runner regressions, optimized interpreter refusal and clean live cluster run → example-tested / integration-tested | held; process doubles clearly separated |
| Paired benchmark selection, percentiles/cold semantics and overhead match the recorded decision | Benchmark, read inventory and safe results | policy:cantos-engineering read-performance; review-profile:requester | Script/test inspection, 60 Python tests, raw hashes/144 records/144 plans audit → example-tested / statically-checked; original measurements not rerun | held for historical artifacts only; no fresh final-source latency claim |
| DBSP remains deferred without measured repeated join/aggregate and freshness contract | Decision/review/queue | policy:cantos-engineering read-performance | Existing consumer/query inventory and unchanged historical decision evidence → documented / statically-checked | held; engine/CDC admission experiment not-applicable: no eligible consumer |
| CI instructions have limited permissions and name actual local checks | Workflow/manifests/docs/skills | policy:AGENTS.md#verification | Workflow inspection and local aggregate checks → documented / statically-checked | held for local instructions; remote CI not rerun |
| Publication, provider, Theatre and mobile behavior are not completed by this slice | Product/domain owners | policy:AGENTS.md; work-plan 010–050 | Absence of runtime implementations and explicit scope labels → documented / statically-checked | not-applicable: not implemented or changed here; retain later gates |

The persistence audit traced every load/save path through current session and actor facts,
resource authorization and canonical row admission. Session rows are share-locked through the
transaction, so session revocation waits for an already authorized transaction. Actor activity
and membership are checked from current READ COMMITTED facts; their changes do not retroactively
cancel an in-flight request. Subsequent requests reauthorize. No stronger global revocation
linearization is claimed or introduced. Evidence references resolve against the script owner's
registry and are retained with the immutable revision; that registry does not prove legal rights.

The fixes extract only pure decimal admission and Script ID selection. They add no service,
generic store abstraction, query/index change, engine adapter, provider dependency, migration or
public wire field. Rejected fractional tokens are a correction of the stated contract. Canonical
stored exports, golden bytes and digest classification are unchanged. Draft admission performs
one additional complete export bound check; canonical reads preserve one encoding comparison.

## Executed checks and artifacts

Host tooling: Rust/Cargo 1.87.0, Python 3.13.5, PostgreSQL 17.4 Homebrew, Trunk 0.21.14,
wasm-bindgen 0.2.100 and Safari 18.6 on the Mac mini. Dependencies were already cached; no new
installation was needed. `--offline` supplements, rather than removes, the required Cargo lock
checks. Commands ran from the repository root except Trunk.

| Command / operation | Final result | Local output |
| --- | --- | --- |
| `cargo fmt --all -- --check` | PASS | `target/final-review/fmt.log` |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | PASS | `target/final-review/clippy.log` |
| `cargo test --workspace --locked --offline` | PASS: 27 tests; nine PG tests deliberately ignored in this command | `target/final-review/rust-tests.log` |
| `PATH=/opt/homebrew/opt/postgresql@17/bin:$PATH python3 scripts/test_postgres.py --keep` | PASS: nine real PG tests plus HTTP kill/PG restart/exact replay; new disposable cluster only | `target/final-review/postgres-run.log`, `target/revision-evidence/postgres-tests.log` |
| `TMPDIR=/private/tmp python3 -m unittest discover -s scripts -p 'test_*.py'` | PASS: 60 tests | `target/final-review/python-tests.log` |
| `python3 scripts/reference_script_ir.py` | PASS: both unchanged c1 goldens | `target/final-review/reference.log` |
| `TMPDIR=/private/tmp python3 scripts/mutate_script_ir.py` | PASS: 15-test baseline, nine killed; zero survived/timeouts/unviable | `target/final-review/mutation.log` |
| `cargo clippy -p cantos-studio --target wasm32-unknown-unknown --locked --offline -- -D warnings` | PASS | `target/final-review/wasm-clippy.log` |
| `PATH=/Users/manhblue/Library/Caches/dev.trunkrs.Trunk/wasm-bindgen-0.2.100:$PATH NO_COLOR=true trunk build --locked --offline` in `apps/web` | PASS | `target/final-review/trunk-build.log` |
| `python3 scripts/studio_tokens.py --check` | PASS | `target/final-review/tokens.log` |
| `python3 target/dbsp-decision-verification/evidence-audit.py` after reading the retained audit helper | PASS: two raw checksums, 144 run records, 144 plans and historical source/query hashes | `target/final-review/evidence-audit.log` |
| `python3 scripts/check_repository.py` | PASS: 207 text files, 15 skills, local links/JSON/skill structure | `target/final-review/repository-check.log` |
| `git diff --check` | PASS | `target/final-review/diff-check.log` |
| `git diff --cached --check` | PASS | `target/final-review/staged-diff-check.log` |

Initial diagnostic failures were retained: new regressions failed against original behavior;
native Clippy caught a manual range check, which was corrected without suppressing its lint.
The first sandboxed PG run could not bind loopback and did not produce integration evidence;
the authorized rerun actually passed. These failures are not counted as successful checks.
The runner unit tests use doubles for readiness and a synthetic inherited environment; the
integration run uses an actual new PostgreSQL cluster and actual HTTP process.

Raw logs and synthetic runtime records remain in ignored `target/`, not Git. The mutation run
records SHA-256 inputs for validation, canonical encoding, admission and wire code; its extra
two mutants check export-bound removal and ignored numeric tokens. No fixture was regenerated
to accommodate a defect. The retained cluster was used only for the live Studio walkthrough,
then its exact task-owned host and cluster were stopped (`target/final-review/cleanup.log`).

### Final Safari observations

The actual built WASM consumer served by `python3 scripts/serve_studio.py` read synthetic
Vietnamese fixture revision 1, then keyboard activation saved the same text as revision 2 and
displayed the durable backend acknowledgement. Changing Script ID preserved the draft, showed
base 0, removed the prior stored export and displayed “Có văn bản chưa lưu.” No save to the new
target was performed. The attempted text-selection edit did not match the fixture and made no
change; this walkthrough proves an unchanged-text save, not a newly edited-line save.

Captures were inspected in native Safari windows 1619×862 at 100%, 643×862 at 100%, and the
compact window at Safari's confirmed 200% page zoom. Vietnamese diacritics, focused Script ID,
save control and status remained readable without visible clipping. These are native window
dimensions, not instrumented CSS viewport sizes. Keyboard read/save actions, focus indication,
native labels and the source's polite live-status semantics were checked; VoiceOver was not.
Screenshots were inspected through the computer-use tool and were not saved as repository
image artifacts. The original window size and 100% zoom were restored. Chrome's automation
adapter was unavailable; it supplies no browser evidence.

### Benchmark and DEFER interpretation

The audit confirms 303,480 historical measured HTTP requests across the two retained scales.
At 5,000 scripts the median warm member p50/p95/p99 values remain baseline
1.64/15.83/17.25 ms, covering index 1.63/15.75/16.91 ms and returned baseline
1.63/15.84/17.13 ms. The original index comparison adds 768 KiB and approximately 44.1%
membership WAL for approximately 0.5% p95 gain. These are the existing historical runs,
not final-code measurements. Paired seeded workload generation, warm-up inclusion, percentile
calculation, shared-buffer-only cold restart, write/storage/WAL definitions and sanity guards
were checked in the harness and tests. OS/file caches were not cleared. Synthetic debug builds
and local timing do not establish production capacity or a statistically robust speedup.

No benchmark was rerun after the parser/validation fixes. Store queries, migration and benchmark
source stayed unchanged, but complete final-source performance remains unmeasured. The decision
still has no repeated hot aggregate/join consumer or committed-position/freshness contract;
DBSP engine/CDC/fallback/rebuild experiments remain NOT_RUN. The next recommended work remains
the bounded 010 importer/editor, with simpler PostgreSQL evaluation when its reads change.

## Remaining and completion record

No confirmed in-scope defect remains from this review. The work queue links this report without
changing GitHub execution status. Some pre-existing skill references still describe illustrative
unimplemented types; the actual code and proposed ADRs, not those examples, determined coverage.
No unrelated skill rewrite was made.

Required checks for the implemented local slice are reported above; an ignored test, unavailable
adapter or historical run is never substituted for a fresh PASS. Production identity/TLS,
legal-rights eligibility, stable-ID continuity across revisions, durable browser drafts,
screen-reader/IME and exact CSS viewport gates, mobile runtimes, provider/audio/publication
flows, production resource budgets and DBSP engine/CDC remain NOT_RUN or unimplemented. They
are limitations of the declared slice, not grounds to start unrelated integration or close #1.

Invariant: untrusted admission cannot silently change fractional intensity, create an export
outside its storage/read bound or retain repeated-match span memory without a text bound;
accepted revisions remain complete, immutable and actor-scoped. Studio communicates the draft's
actual target state. Owner/boundary: Script IR pure admission, PostgreSQL runner, Studio pure
state transitions and their concrete adapters. Evidence: typed/compiled, literal examples,
independent differential oracle, nine bounded mutations, real local PostgreSQL/HTTP faults and
the named Safari observations, each scoped above. Residual risk: these checks do not prove
production authorization, rights, capacity, adversarial CPU limits or future publication and
native playback behavior.
