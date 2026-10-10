# Host-driven adaptation review

**Outcome: no unresolved blocking code findings in the frozen local snapshot.** The confirmed
workflow is implemented and supported by actual CLI, authenticated HTTP and fresh PostgreSQL
evidence. Final committed-SHA CI, push, merge and postmerge verification remain executor-owned.

This independent read-only review covers the replacement of the earlier PR #20 implementation,
including caller-submitted c1 workflow. The PR base is
`adfa82ce72a57d513f55c89ba7a58883af652cb2`; the previous implementation commit is
`e9ae8a9f1391d40985cbf010cb46a1caee3435b1`. Historical provider reports do not establish current
behavior. Reviewed on 2026-10-10 by the separate adaptation-review agent.

The reviewer inspected source, contracts, migrations, tests, execution logs and paired API
oracles. The reviewer did not execute builds, provider calls, browser interactions or PostgreSQL
tests. Runtime results distinguish inspected evidence from executor-reported observations.

## Confirmed scope

Generation belongs to the user's AI host. Cantos has no local inference runtime, hosted inference
client, provider worker or automatic generation dispatch. Its four terminal tools create/read a
frozen context, submit a proposal and read review. The Python CLI uses the existing authenticated
Axum service; it is not a configured public ChatGPT/Gemini connector or MCP server.

Cantos preserves source, validates submitted output, records explicitly unverified caller
declarations and requires separate human acceptance through the existing revision-save authority.
Actual model generation and editorial quality are **NOT_RUN**, and do not block this revised
integration scope.

## Assurance ledger

| Claim / invariant | Owner and inspected evidence | Disposition |
| --- | --- | --- |
| No Cantos inference or background redispatch | Server construction, store, HTTP compatibility routes; deleted provider transport/dependency | Supported. Retired start returns unavailable; legacy reads never dispatch. |
| Historical a1 records remain readable | FrozenAdaptation, existing DTO serialization, a1 digest label, legacy PG fixture | Supported for reviewed layout. Nested DTOs use current serializers; not a complete archived-byte golden oracle. |
| New context uses existing durable authority | Migration 0005, FrozenContext, adaptation store | Supported. Separate c1 digest/version, same run authority, immutable inputs and owner/run composite references. |
| Context binds preserved source and base revision | Context creation/decoding, pure preparation, PG tests | Supported. Checksum, extraction version, rights request, base revision, prompts and schema are frozen and checked. |
| Rights remain claims with applicable gates | Context authorization, generated rights/provenance binding, API/UI copy | Supported within scope. Legal eligibility/publication permission is not inferred from a caller claim. |
| Model output cannot mint trusted authority | Narrow output schema, pure materialization, Script IR validation | Supported. IDs, rights, lifecycle and provenance are server-bound; manuscript/output remain untrusted data. |
| Unknown speakers and uncertainty remain visible | Core mapping, findings, corrected independent PG oracle | Supported. Explicit unresolved placeholder/finding; fidelity requires human comparison. |
| Caller metadata remains unverified | Metadata validator, DTOs, manifest and Studio labels | Supported. Prompt consistency checked; unknown usage/cost preserved; ProviderReported cost rejected at caller intake. |
| Submission persistence is atomic | Store transaction, immutable receipts, unique operations, real PG tests | Supported. Invalid output records a receipt; first valid proposal atomically closes submission eligibility. |
| Duplicate requests/cancellation are fenced | Exact-operation comparison, locked transitions, cancellation receipts | Supported. Exact retry replays; changed intent conflicts; distinct later proposal cannot replace first valid one. |
| Acceptance preserves concurrency/history | Shared save_in_transaction, acceptance receipt/status transaction, PG tests | Supported. Explicit review, authorization and optimistic revision checks remain mandatory. |
| Actor isolation/revocation | Owner-scoped lookups, post-lock authentication, PG cases | Supported. Expiry uses actual clock, not transaction-start time. |
| CLI confines transport and handles uncertain writes | Python source, six boundary tests, actual subprocess/socket journey | Supported. Numeric loopback, disabled proxies/redirects, bounded JSON/replies, no automatic retry or accept command. |
| Studio protects dirty edits/stale responses | Acceptance-only reducer, browser integration/state tests | Supported. Actor/ticket guards, IME/dirty preservation and exact pending-acceptance reconciliation remain. |
| Restart recovery preserves observations | Fresh-cluster runner, external-tool-restart.json | Supported by actual HTTP/CLI/PG restart evidence; zero generation attempts recorded. |
| Performance claims are bounded | point-lookups.json, indexed query plans | Supported for warm single-concurrency synthetic point reads; no production/cold/high-concurrency claim. |

## Resolved findings

Three confirmed P2 CLI boundary defects were addressed before the final snapshot:

- A malformed, oversized or interrupted successful POST response could report generic decode
  failure or escape through HTTPException, obscuring a potentially committed write. The final
  path reports `write_outcome_unknown_retry_exact_operation`, preserves exact reconciliation
  and never automatically resends.
- Numeric overflow such as `1e999` could become infinity and fail later during serialization.
  Nonfinite parsed floats now fail before transport, including response parsing.
- Escaped lone surrogates in JSON values/keys could survive parsing and fail output encoding after
  a committed POST. Strings/keys now undergo UTF-8 validation before return.

Final fixes and regression assertions were inspected. Six focused tests passed; the complete
Python suite subsequently passed 68 tests. Final manifest source grammar, integer ceilings and
invalid-output receipt guidance were reconciled with backend contracts.

## Verification evidence

| Check | Result and evidence basis |
| --- | --- |
| Formatting, native Clippy/workspace tests | PASS; inspected final logs/results, 119 native tests |
| Fresh PostgreSQL | PASS; inspected runner log, 12 revision + 9 import + 19 adaptation = 40 cases |
| Actual terminal tool/authenticated HTTP | PASS; subprocess/socket integration in fresh PG runner |
| Host/PG restart and exact receipts | PASS; inspected restart oracle |
| Reference Script IR and mutations | PASS; inspected results, 9 killed/0 survived |
| WASM Clippy/Trunk | PASS; inspected final results |
| Python | PASS 68; executor-confirmed final log |
| Repository checker | PASS 230 text files/15 skills; executor-confirmed final log |
| git diff --check | PASS; executor-confirmed result |
| Safari interaction/accessibility | PASS for documented limited journey; executor observed runtime |
| Real model generation/editorial quality | NOT_RUN; nonblocking for confirmed scope |
| Final committed-SHA CI/postmerge | NOT_RUN in this review; executor-owned |

The actual tool/restart oracle retained one caller run, two submission receipts, one proposal
and one acceptance, with zero generation attempts. The proposal was repository-authored
synthetic Vietnamese content: transport/persistence proof, not model-quality proof.

The reviewer independently compared Studio's before/after API records: source, context digest,
original proposal and submission were unchanged. The accepted revision changed only
`/episode/acts/0/scenes/0/title`; status moved succeeded → accepted and revision 1 reopened with
the recorded digest.

The executor observed Safari on macOS 15.6 at browser frames 639×852 and 1324×968, confirmed
200% zoom, Option-Tab focus indication and light/dark presentation. The journey covered invalid
edited content, correction, renewed acknowledgement, explicit acceptance and reopening.
Persisted screenshot artifacts are not claimed. VoiceOver, Telex/VNI, mobile and motion remain
NOT_RUN. See [execution evidence](ai-script-adaptation.md) for commands, artifacts and limits.

Warm-read benchmark: 1,000 proposals, concurrency 1, 30 measured reads after five warmups,
p50 3.56 ms/p95 3.73 ms/p99 3.75 ms, within the declared 250 ms p95 target. Small synthetic
payloads and warm local reads limit generalization.

## Residual risks and handoff

Human comparison remains necessary for omissions, unsupported interpretations, speaker attribution,
emotion, prosody and cues. Caller model/configuration/usage/cost declarations are not attested.
Actual legal rights, production identity/TLS, cross-document continuity, audio production and
native mobile acceptance remain unfinished.

Future compatibility work must preserve legacy serialized layout/digest semantics. An archived-byte
golden would strengthen assurance beyond reconstructed legacy fixtures; this is a nonblocking
improvement, not a new gate. Full Studio scene/character authoring remains #4. No inference, paid
generation, credential provisioning, deployment or mobile implementation occurred. Current source
is ready for committed-SHA CI and normal PR completion.
