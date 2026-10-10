# Private adaptation v1 contract

The local Cantos Studio adaptation slice consumes an immutable import receipt and an optional
pinned revision. Shared raw DTOs live in [`api/src/adaptations.rs`](api/src/adaptations.rs).
They do not establish rights, editorial approval, publication eligibility or production readiness.
The [HTTP envelope schema](schema/adaptation/v1.schema.json) records admitted command/response
shapes; runtime authorization and state consistency are enforced by the store.
Script IR read/write remains `{0.1.0}`; existing c1/e1 digest schemes are unchanged.

## HTTP operations

All manuscript/proposal operations use the existing authenticated Studio session and same-origin
mutation boundary. Run IDs, script IDs and operation IDs are canonical UUIDs; import source IDs
retain the existing opaque source grammar. Provider readiness contains configuration only.

| Method and path below `/api/v1` | Request | Response |
| --- | --- | --- |
| `GET /adaptations/provider` | — | `AdaptationProviderResponse { provider: metadata or null }` |
| `POST /adaptations` | `StartAdaptationRequest` | HTTP 202, `AdaptationRunResponse` |
| `GET /adaptations/{run}` | — | `AdaptationRunResponse` |
| `POST /adaptations/{run}/cancel` | `CancelAdaptationRequest { operation_id }` | `AdaptationRunResponse` |
| `POST /adaptations/{run}/accept` | `AcceptAdaptationRequest` | Existing `RevisionResponse` |

Start names `operation_id`, `source_id`, `script_id`, `expected_revision`, `expected_provider` and
`rights_authorization`. The last field records explicit permission to process this source on the
displayed local runtime; it is a creator assertion, never legal clearance. `expected_provider`
must exactly match the current provider/model/endpoint/prompt/contract/configuration before a
new run is recorded. Configuration drift cannot silently select another destination. An exact
operation replay returns the originally recorded run; it never repeats generation.

`expected_revision: 0` targets a new script; a nonzero revision must be the currently authorized
owned script head. The immutable run pins source ID/checksum/extractor version, the complete
base revision and its c1/e1 digests, request, generation evidence, pending rights evidence and
provider metadata and a local model fingerprint. Provider configuration includes integer temperature milliunits, seed, context
tokens, predicted token cap and timeout seconds. It contains no credential.

Run states are `queued`, `running`, `succeeded`, `invalid_output`, `failed`, `ambiguous`,
`cancelled` and `accepted`. A queued run can be safely claimed once. The running attempt is
recorded before dispatch; expiry or lost post-dispatch transport becomes ambiguous. A replay or
restart never silently repeats an ambiguous call. Cancellation fences acceptance; it does not
claim the external computation stopped. Actual reported token usage remains available when
parsed generated output is rejected. Missing usage/cost is `null`, never invented as zero.
The optional monetary record admits only provider-reported integer minor units in USD (2 decimal
places) or VND (0). The Ollama adapter reports no monetary amount, so its cost remains `null`.

Acceptance names `operation_id`, `expected_revision`, edited complete `script_json` and
`reviewed_findings: true`. It validates Script IR and its fixed source/attempt/rights bindings,
then uses the existing revision authorization, concurrency and idempotency path in the same
transaction as the acceptance receipt. A stale head is a conflict. Source and prior accepted
revisions are never overwritten. Editorial acceptance is distinct from production/publication
approval; unresolved attribution and other findings remain in the immutable run history.

## Narrow provider output

The actual model output is **not Script IR**. Its independent
[JSON Schema](schema/adaptation/cantos-adaptation-1.schema.json) and original synthetic
[Vietnamese fixture](fixtures/adaptation/proposal-vi.json) define `cantos-adaptation-1`:

- `title`; proposed `characters` with names and personality text;
- ordered `scenes`, each with spoken `lines`, typed `cues` and optional `pacing_note`;
- each line has required nullable `speaker`, spoken `text`, nonempty `source_blocks`, supported
  emotion/intensity delivery and an optional `prosody_note`;
- each cue has kind `ambience`, `music` or `sfx`, a description, same-scene zero-based
  `line_index`, edge `start` or `end`, and source block citations;
- explicit `omitted_blocks` with reasons and free-text `review_notes`.

Unknown fields, model IDs, rights, provenance, assets, approval/status fields and Markdown fences
are rejected. Nullable speaker is required; missing attribution uses an explicitly unresolved
placeholder. Optional notes can be absent but cannot be JSON `null`. This provider contract
additionally requires unsigned literal integer tokens for block/anchor indices and intensity;
`300.0` or `3e2` are refused even though JSON Schema treats them as mathematical integers.
The existing Script IR reader retains its independent numeric compatibility behavior.

Cantos mints random identities independently of text and position, preserves existing work,
adaptation, episode and matched character identity, attaches trusted import/generated provenance,
then runs the existing complete Script IR structural and semantic validator. The reserved model
speaker label `Người dẫn chuyện` resolves by narrator **role**, including a renamed existing
narrator. A suggested character cannot remove that narrator or convert it into an ordinary role.

Every cited block must exist in the pinned extraction. Omission claims cannot conflict with
represented blocks. Coverage records show cited dialogue IDs, omissions and uncovered blocks;
they are model claims requiring source comparison, rather than proof of semantic coverage.
Source warnings, unknown/new speaker labels, changed text, possible narration confusion,
markup-like spoken text and unsupported performance controls remain explicit review findings.
Pacing/prosody notes do not become invented Script IR controls. No full editor, name-map
renaming system or cross-document identity reconciliation is claimed here.

## Local transport and resource admission

The real adapter uses Ollama `POST /api/chat` with a schema `format`, `stream: false`, an explicitly
configured model, bounded options and `keep_alive: 0`. Only numeric loopback HTTP URLs with an
explicit port are admitted. DNS destinations, URL credentials, other paths, queries, fragments,
proxies, redirects and automatic retries are refused. No model pull, credential configuration,
host tool access or hosted-provider fallback is implemented.

Loopback transport alone does not establish local inference: Ollama can forward cloud models.
The real adapter connects through source-free, bounded `GET /api/status` and
`POST /api/show { model, verbose: false }` probes. It requires `cloud.disabled: true`, rejects
remote-model/remote-host metadata including aliases, and requires local GGUF metadata with an
absolute `FROM` path naming a SHA-256 weight blob. A tagged `ollama-s1:sha256` fingerprint covers
the safe model metadata and weight identity, including effective template/defaults.
The fingerprint normalizes rendered parameter groups by key because Ollama renders its Go
options map in arbitrary key order. Repeated values within a key retain their order. All other
Modelfile directives and quoted multiline content retain their order and exact text; both
Ollama-generated single and triple double-quote wrappers are recognized. Ambiguous multiline
parameter rendering is rejected. Changed defaults, template/system/message text, adapters or
weight identities invalidate the fingerprint.

The verified provider freezes that fingerprint into displayed/authorized metadata. Before
each source-bearing call, it repeats the probes and refuses changed weights/configuration.
An unverified configuration-only provider cannot dispatch. Unknown/older runtimes without
this attestation are blocked; Cantos changes no persistent runtime configuration.
Prerequisites are an approved, managed, cloud-disabled local runtime and stable model
configuration; this handshake does not cryptographically attest a malicious localhost process.

Source blocks and instruction-like model/source text are inert JSON/text data. They are never
executed, interpreted as host instructions or rendered as HTML. The prompt is versioned
`cantos-radio-adapt-1`. At most 256 blocks and 24 KiB of source text enter request preparation;
the actual request must also satisfy a conservative context check: system/prompt UTF-8 bytes +
256 framing reserve + predicted output tokens must fit configured context tokens. This deliberately
assumes at most one token per byte and rejects longer chapters instead of silently truncating.
This is application admission, not proof of the runtime's rendered-template/tokenizer budget.
Ollama's truncation/shift defaults are not established by the HTTP double; the approved model's
actual behavior must be checked during live acceptance. Reviewed chunking/tokenizer integration
and explicit runtime truncation policy are future work.

Request bodies are capped at 96 KiB, model output at 256 KiB, and the escaped Ollama envelope at
6 × 256 KiB + 4096 bytes while streaming. Output admits at most 64 scenes, 128 suggested
characters, 1000 total nodes, 1024 aggregate citations and 2000 findings. Text fields have
independent UTF-8 byte bounds in addition to the schema's Unicode length bounds. The provider
enforces context 1024–16384, prediction 128–8192 and timeout 1–180 seconds; the store limits
dispatch concurrency separately.

## Evidence boundary

[`apps/server/tests/adaptation.rs`](../apps/server/tests/adaptation.rs) supplies deterministic
admission, independent schema and evidence-binding cases. The
[transport suite](../apps/server/tests/adaptation_provider.rs) operates synthetic loopback HTTP
stubs for structured requests, redirects, context bounds, actual count preservation and ambiguous
timeout. These are fixture and transport proofs, **not live-model acceptance**. Source fixtures
are original synthetic Vietnamese text. A live local-model run still requires an approved,
available Ollama runtime/model and separately recorded source/semantic/editor acceptance.
