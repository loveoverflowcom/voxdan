# Prompt-3 Script revision read measurements

## Scope and provenance

Prompt 3 only, based on local `develop` at
`626416500599b852dc985683fa3a4ac905f9401f`. Entry tree was clean and ahead of origin/develop by
one commit; prompt-1 `ea8e1d98a31301b0d63dfa75d8e56450397b581d` and prompt-2 remain intact.
No fetch/reset/rewrite, push, remote/auth change, PR, issue close, production DB or engine
installation was performed. GitHub issue #1/010 remains broader than this slice.

The implemented deliverable is a bounded synthetic benchmark/oracle harness with regression
tests, a PostgreSQL index experiment confined to disposable DBs, and this decision record.
Production SQL, migrations, Script IR/c1/e1, authorization, HTTP and Studio behavior are unchanged.
This is a legitimate measured no-change result, not an unrun optimization claim.

Schema version: read/write `0.1.0`, no schema/converter change. Original fixture/golden digests
are unchanged; benchmark variants live outside Git. Stored revisions are never rewritten.
SpokenContent contract: unchanged; no actual speech rendering or invalidation run is claimed.

Follow the [protocol](script-revision-benchmark-protocol.md) for exact commands, chosen targets,
sample accounting and cache preparation. The [read inventory](script-revision-reads.md) maps
the real consumers, indexes, access scope and feeding writes. No list, pagination, catalog or
aggregate endpoint is invented. Frequency/cardinality in production remain unknown.

Environment: macOS 15.6 (24G84), Mac mini, Apple M4, 24 GiB RAM, 10 logical CPUs; PostgreSQL 17.4 Homebrew,
Rust/Cargo 1.87.0, Python 3.13.5, existing debug server profile (`debug=0`). Each run creates a
fresh loopback-only trust cluster and restricted app role; each of its three phases clones
the same committed seed DB. Fast restart resets shared buffers before cold-start batches;
OS page cache and other machine activity are uncontrolled. Durability stays enabled.
App pool 8 and existing 5s statement/3s lock/15s HTTP limits remain in force.

Raw results live in ignored `target/read-benchmarks/`; [safe results and plan observations](script-revision-performance-results.json)
retain every repetition. Raw timing values,
CPU/RAM samples, plans, SQL/schema/harness hashes and actual config are retained locally.
No raw CSV/content/audio is committed. Fixtures are the original synthetic Vietnamese
two-scene script, plus generated 512-dialogue variants, not private manuscripts.

## Invariant, owner and authority

**Invariant:** the measured GETs preserve the complete immutable accepted response and current
owner/member/session authorization; a candidate cannot trade those semantics for speed.
PostgreSQL is the sole authority. Oracle comparisons use quiescent committed positions within
each phase, exact accepted actor/time/export/digests, and separate expected denial bodies.
No CDC/view watermark, serving lag, async fallback or atomic cutover exists to claim.

| Claim / failure mode | Owner / boundary | Evidence held |
| --- | --- | --- |
| Complete response and unchanged revision identity | postgres load/decode; HTTP; PG independent SQL projection | integration-tested / differentially-tested against real PG, exact response equality before/after each phase |
| Current access, concealed missing/foreign rows | backend owner/member/session rules | owner/reader/editor allowed; outsider/foreign owner/anonymous/missing history/revocation denied; reader save403 leaves head unchanged |
| Reproducible comparison, no inherited DB | harness lifecycle/fixture/accounting | fresh cluster + seed template clones; unit guards for env isolation, source SQL, skew, percentiles and clone origin |
| Immutable history, stale writes, replay and failure recovery | existing PostgreSQL transaction + domain rules | unchanged production seams; fresh full PG suite result recorded below |
| No schema/digest/domain policy change | migrations/script_ir/contracts | source hashes unchanged; no new domain abstraction/core needed for an external measurement adapter |

## Workloads and observations

100 scripts: 2,495 revisions, 200 members, 12,475 evidence links.
5,000 scripts: 26,995 revisions, 10,000 members, 134,975 evidence links.
32 owners, half the scripts owned by owner0; script0 has 2,000 revisions and others 5;
10% of scripts carry the 95,248-byte/512-dialogue export, others the 1,976-byte/4-dialogue export.
Same source/rights/asset references have separate owner registry keys and distinct accepted
operation/revision identities; neither c1 nor e1 collapses accepted rows.

Cells below are median-of-three per-run **p50 / p95 / p99 ms; completed measured requests/s**.
Each warm run contains 2,400 measured requests plus 32 warmup operations/worker. Throughput's
denominator includes warmup, so it is conservative. Each cold-start batch contains 60 requests,
no application warmup; it progressively warms pages. Cold p99 here is a per-batch maximum,
not a statistically established disk-cold tail. All repetitions and ranges remain in JSON.

| 100 scripts | Existing PG | Covering index candidate | Index dropped, fresh baseline |
| --- | --- | --- | --- |
| Cold-start member C=1 | 1.52 / 15.23 / 16.44; 299.58 | 1.52 / 15.51 / 16.46; 302.33 | 1.51 / 15.51 / 16.51; 303.76 |
| Warm owner C=1 | 1.25 / 14.99 / 15.31; 379.37 | 1.27 / 15.04 / 15.32; 373.53 | 1.28 / 15.05 / 15.41; 374.91 |
| Warm member C=4 | 1.64 / 15.77 / 17.38; 1049.29 | 1.64 / 15.88 / 17.15; 1069.89 | 1.66 / 15.82 / 17.19; 1042.67 |
| Pinned history C=4 | 1.47 / 2.02 / 2.40; 2439.73 | 1.48 / 2.00 / 2.35; 2445.39 | 1.49 / 2.04 / 2.46; 2434.99 |
| 512-dialogue C=4 | 18.11 / 27.89 / 30.31; 191.52 | 18.70 / 28.64 / 31.10; 186.40 | 18.74 / 28.35 / 31.25; 186.68 |
| 90/10 read/write C=4 | 1.54 / 16.05 / 31.61; 638.47 | 1.53 / 16.04 / 31.62; 646.80 | 1.54 / 16.12 / 31.65; 650.38 |
| 20/80 read/write C=4 | 3.72 / 31.02 / 32.19; 150.44 | 3.71 / 31.05 / 32.18; 150.22 | 3.70 / 31.21 / 32.25; 149.30 |
| Denied C=4 | 0.86 / 1.23 / 1.61; 4168.11 | 0.86 / 1.20 / 1.59; 4155.05 | 0.86 / 1.19 / 1.59; 4159.41 |

| 5,000 scripts | Existing PG | Covering index candidate | Index dropped, fresh baseline |
| --- | --- | --- | --- |
| Cold-start member C=1 | 1.49 / 15.56 / 16.61; 326.99 | 1.44 / 15.44 / 16.04; 332.86 | 1.48 / 15.68 / 16.51; 324.33 |
| Warm owner C=1 | 1.27 / 15.03 / 15.35; 382.87 | 1.27 / 15.04 / 15.33; 382.44 | 1.28 / 15.01 / 15.35; 382.45 |
| Warm member C=4 | 1.64 / 15.83 / 17.25; 1041.51 | 1.63 / 15.75 / 16.91; 1059.53 | 1.63 / 15.84 / 17.13; 1031.58 |
| Pinned history C=4 | 1.47 / 2.01 / 2.40; 2442.09 | 1.50 / 2.08 / 2.48; 2404.52 | 1.47 / 2.01 / 2.39; 2456.75 |
| 512-dialogue C=4 | 18.50 / 28.74 / 31.64; 186.39 | 18.55 / 28.50 / 31.21; 187.84 | 18.66 / 28.58 / 31.41; 185.96 |
| 90/10 read/write C=4 | 1.52 / 16.01 / 31.46; 653.83 | 1.52 / 16.16 / 31.59; 643.60 | 1.53 / 16.09 / 31.57; 653.04 |
| 20/80 read/write C=4 | 3.65 / 31.06 / 32.11; 150.52 | 3.68 / 31.24 / 32.29; 149.19 | 3.66 / 31.16 / 32.14; 149.54 |
| Denied C=4 | 0.86 / 1.20 / 1.64; 4184.73 | 0.85 / 1.14 / 1.49; 4229.95 | 0.86 / 1.21 / 1.64; 4148.37 |

Member-p95 ranges across repetitions (baseline / candidate / return): 100-script
15.716–15.864 / 15.824–15.906 / 15.807–15.920 ms; 5,000-script
15.797–15.840 / 15.732–15.819 / 15.802–15.852 ms. At the larger scale the tiny improvement is
about 0.5%, far below the predeclared 15% threshold; it is not grounds to adopt an extra index.
Each scale has 151,740 measured requests across 72 runs, plus 7,200 warmup operations;
144 runs total retain 303,480 measured requests. Wall intervals total 386.10s and 385.11s,
including warmup; provisioning, cloning/restart/oracle/EXPLAIN/operator bursts are outside them.
The request seed, parameters, operation IDs and write sequence are identical across phases;
new acceptance timestamps are compared to each phase's own committed PostgreSQL rows.

All per-run warm percentiles fit the chosen investigation budgets; no production acceptance
follows. Maximum sampled Axum+PG CPU averages are 3.19 and 3.19 core equivalents. Peak observed
Axum RSS is 23.03/22.69 MiB; summed PG RSS 204.33/312.84 MiB; client process-lifetime peak
46.72/45.80 MiB (100/5,000 scripts). These are separate component maxima, not simultaneous
physical memory totals. Raw arrays retain sample counts and deltas. Final PG data directories
are under 1 GiB; observed free disk remained above the 5 GiB stop budget (about 12 GiB at exit).

Raw completed artifacts:

- `target/read-benchmarks/run-1791610270070198000/report.json` (100 scripts).
- `target/read-benchmarks/run-1791610677100344000/report.json` (5,000 scripts).
- In each directory: `samples-*.json`, `resources-*.json`, nine `plans-*.json`, seed CSVs,
  admitted fixture JSON and owned PostgreSQL/host logs. Both task-owned hosts/clusters stopped.
- Both runs used harness SHA-256 `c12eca436c6c4e4fb454f65d51ca072ae31930ec18bd8048ae7017dee4ecb305`;
  authoritative source, query and schema hashes are in the safe result record. Index definitions
  and candidate DDL are retained verbatim there. Config stores PostgreSQL native units:
  shared_buffers 16,384×8 KiB=128 MiB; work_mem 4,096 KiB=4 MiB; effective_cache_size 524,288×8 KiB=4 GiB.

## Simpler optimization decision

Candidate: `CREATE INDEX bench_member_cover ON script_members(script_id,actor_id) INCLUDE(role)`.
It never enters a migration. Existing composite primary keys already select one row for every
implemented read. At 100 scripts, membership baseline scans 200 rows in about 0.02 ms; the candidate
uses an Index Only Scan but still fetches a heap row after the pre-run revocation regression
clears visibility. This is an access-change condition, not proof of best-case all-visible behavior.
Plans after permission writes separately capture visibility/maintenance effects.

No 15% consistent member-p95 improvement is observed at 100 scripts; median 15.77→15.88 ms
is within run drift, and return baseline is 15.82 ms. Candidate storage adds 32 KiB initially
(member total 96→128 KiB), growing to 64 KiB after the permission burst. Twelve 100-row updates
produce 239,120→479,264→239,120 WAL bytes. Operator p95 includes fresh psql process/connection
cost: 13.30→13.64→13.30 ms; it is not pure SQL time. API save p95 in 90/10 is
32.66→32.47→32.57 ms. Saves do not write membership; direct index write amplification applies
to operator access changes, not revision inserts.

At 5,000 scripts the existing PK Index Scan takes 0.023 ms (3 buffer hits), versus candidate
0.036 ms (4 hits, 1 heap fetch). After 60,000 permission updates, baseline takes 0.035 ms
(6 hits); candidate 0.065 ms (15 hits, 11 heap fetches). Initial index storage adds 786,432 bytes
(768 KiB), growing to 2,236,416 bytes (2.13 MiB); member total starts at 1,581,056→2,367,488 bytes.
Permission bursts produce 19,218,024→27,690,664→19,218,024 WAL bytes (+44.1% versus baseline);
operator p50/p95 is 21.58/32.16→27.69/43.90→22.14/32.81 ms. API save p95 in 90/10 is
32.35→32.36→32.31 ms. Save WAL medians including warmup are
2,332,760→2,340,680→2,345,496 bytes; full storage/WAL and both mixes remain in JSON.

Plan observations cover both GET lookup queries and internal auth/replay/evidence/migration
fetches with the app role (migration ledger uses the owner). Auth/head are LockRows over bounded
lookups, historical and replay reads use their existing unique indexes, evidence uses its PK.
The skewed script0/revision1 estimate is 80/370 rows (100/5,000 scale) versus 1 actual row; despite
that estimate, PG retains a single-key index access, without join/sort/spill work. All captured
plan roots have zero temp reads/writes. Shared-buffer-cold plans record actual shared reads
(e.g. large membership 3, revision 4); these are not claims of physical disk reads. EXPLAIN profiling
uses separate psql connections and is not HTTP latency. No new statistics/index is justified by
the misestimate alone. Owner/head covering indexes cannot avoid the required row lock; full BYTEA
exports are not suitable payloads for a speculative covering B-tree. No such DDL was shipped.

**Decision for this candidate: reject; retain the existing PostgreSQL path.** Best-case static
all-visible membership traffic is not claimed: revocation regressions touch a hot membership
page before timing, and the later role burst is explicitly measured. Revisit that distinct
workload if real traffic supplies it. This bounded candidate result does not reject all future
PostgreSQL indexes or establish a universal limit.

Small and large full-export reads cost more than the authorization lookup: large exports require
canonical admission, both hash checks, transport and JSON materialization. Those integrity checks
remain. There is no read-side N+1; duplicate auth is two fresh checks by middleware and operation
transaction. The save-side five evidence fetches/five link inserts are a bounded N+1 in this fixture;
the existing save budgets do not justify weakening/batching ownership checks here. Maximum evidence
fan-out, cached validated exports, prepared-statement reuse, pagination and production tuning have
not been measured. Revisit them for a concrete changed consumer/workload.

## DBSP decision and conditional prompt-4 handoff

Decision: **defer**, proposed / not integrated. Do not execute prompt 4 from this task.
The implemented hot reads are indexed points with a tiny session–actor join and current access
checks; there is no repeated large join/aggregate to maintain. The
[DBSP paper](https://docs.feldera.com/vldb23.pdf) describes incremental computation, but does not
establish benefit for this Cantos workload. No engine benchmark, connector, CDC, checkpoint,
rebuild/lag/fallback capacity or exactly-once claim follows from these runs.

Before a conditional prompt 4: identify an actual repeated join/aggregate and consumers, measure
its improved PG baseline and read/write/fan-out workload, set commit-position/freshness/lag and
current-access requirements, review an adapter/ADR/actual engine version and connectors, and
budget state/storage/CPU/RAM, snapshot/rebuild, outage/fallback and delivery/replay evidence.
If those prerequisites remain absent, retain this deferral. Do not invent a Theatre/catalog
endpoint to force adoption. Continue the recommended 010 work without closing issue #1.

## Verification and residual risk

Final-code gates (source hash above; root unless a directory is named):

| Command | Result / evidence scope | Local log under target/read-performance-verification |
| --- | --- | --- |
| cargo fmt --all -- --check | PASS / statically-checked | fmt.log |
| cargo clippy --workspace --all-targets --locked --offline -- -D warnings | PASS / native statically-checked | clippy.log |
| cargo test --workspace --locked --offline | PASS / 23 Rust tests; 9 PG tests explicitly ignored here | rust-tests.log |
| PATH=/opt/homebrew/opt/postgresql@17/bin:$PATH python3 scripts/test_postgres.py | PASS / 9 real PG tests plus HTTP kill, PG restart and exact replay; integration-tested / fault-injected | postgres-run.log |
| TMPDIR=/private/tmp python3 -m unittest discover -s scripts -p 'test_*.py' | PASS / 57 tests; 10 new harness cases | python-tests.log |
| python3 scripts/reference_script_ir.py | PASS / unchanged independent c1 oracle | reference.log |
| TMPDIR=/private/tmp python3 scripts/mutate_script_ir.py | PASS / 7 killed, 0 survived/timeout/unviable | mutation.log |
| cargo clippy -p cantos-studio --target wasm32-unknown-unknown --locked --offline -- -D warnings | PASS / WASM statically-checked | wasm-clippy.log |
| PATH=/Users/manhblue/Library/Caches/dev.trunkrs.Trunk/wasm-bindgen-0.2.100:$PATH NO_COLOR=true trunk build --locked --offline (apps/web) | PASS / compiled debug CSR bundle | trunk-build.log |
| python3 scripts/studio_tokens.py --check | PASS / source/token drift | tokens.log |
| python3 -m py_compile scripts/benchmark_revision_reads.py scripts/test_benchmark_revision_reads.py | PASS / Python syntax | command output, no diagnostic |
| python3 scripts/check_repository.py | PASS / docs/links/JSON/skill hygiene; rerun after final report | repository-check.log |
| git diff --check; staged diff check before commit | PASS / whitespace; final staged check recorded at commit | diff-check.log |
| Both paired benchmark commands from the protocol | PASS / 303,480 measured HTTP requests; exact real PG oracle/denial assertions before/after every phase | read-benchmark-small-paired.log and read-benchmark-large-paired.log in target/ |
| DBSP runtime, CDC/lag/rebuild/fallback and production capacity | NOT_RUN / no engine or production consumer in scope | No such evidence |

The [pre-commit review](script-revision-performance-review.md) covers the final declared diff.
Two review fixes preceded the final measurements: remove phase-dependent RNG offsets so inputs
are paired, and reject optimized Python before it can suppress oracle assertions. A regression
executes the real orchestrator with explicitly labeled I/O doubles to compare seed inputs;
another executes `python -O` and asserts refusal before any cluster start. These unit checks
are not the real PostgreSQL evidence. Initial smoke failures (HTTP connection cleanup and exact
API-error DTO expectations), the stopped growing-history experiment, and unpaired preliminary
runs are diagnostic history only; they are not retained as final comparison results. All final
checks above passed after those code fixes. No formatter changed unrelated code.

No UI code changes;
Safari/VoiceOver/OS IME/exact 320 CSS-pixel observations, mobile, provider/audio, production auth/TLS,
legal eligibility, durable browser recovery and remote CI remain NOT_RUN in prompt 3.
Prompt 2's named evidence remains historical, not a rerun claim.

**Residual risk:** synthetic local traffic and a debug build do not establish production capacity;
Python client/validation/transport and unisolated machine activity affect tails. Cold batches reset
only PG shared buffers; resource sampling misses short-lived/sub-sample CPU and summed PG RSS
double-counts shared memory. Permission revocation is rechecked on subsequent operations and does
not retroactively revoke an already-authorized READ COMMITTED transaction. No incremental serving
boundary or complete importer/editor/publication/listening capability is established.
