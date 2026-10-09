# Narrative Forge exchange adapter

> **Scope.** The boundary between Cantos Script IR and Narrative Forge: what may be shared
> (concepts and a versioned exchange format), what may not (code, storage, deployment), how an
> adapter is shaped, the mapping table, round-trip fixtures, lossy-conversion logging, identity
> mapping and the checks that keep Cantos runnable without that project. Use before writing any
> code, dependency, fixture or document that mentions Narrative Forge, and when reviewing a
> change that couples to it.

Product rules: [Script IR § Narrative Forge integration](../../../../docs/architecture/script-ir.md#narrative-forge-integration),
[business rule 4](../../../../docs/product/business-rules.md#content-and-revisions),
[architecture § Interfaces](../../../../docs/architecture/overview.md#interfaces) and
[decision 0001](../../../../docs/decisions/0001-modular-monolith.md). Direct integration is
[deferred](../../../../docs/work-plan/backlog/deferred-expansion.md) until a concrete source or
consumer, fixtures and compatibility ownership exist. Until then only § 1 applies: keep the
boundary clean so the adapter stays possible.

## Contents

1. What applies today
2. Inspect the real contract first
3. Adapter shape
4. The mapping table
5. Round-trip laws and fixtures
6. Identity mapping and re-import
7. Tests, enforcement and report

## 1. What applies today

| Rule | Failure mode | Good | Counterexample | Oracle | Status |
|---|---|---|---|---|---|
| no build dependency on Narrative Forge | Cantos cannot build, test or release without another repository | Script IR types defined in Cantos | a `path =` or `git =` Cargo dependency, a Gradle include or a vendored copy of its types | manifest review; a dependency check once manifests exist | manual; check proposed |
| no shared database or runtime call | releases and outages couple; one project's migration breaks the other | file or data exchange through the adapter | a connection string to its database, or a synchronous call during save or production | config and code review | manual |
| Script IR owns its own model | another project's internal changes rewrite Cantos rules | a concept such as "scene" reused by name and meaning | a `From<nf::Scene>` inside the domain module | dependency direction review ([`decoupling.md`](../../cantos-engineering/references/decoupling.md)) | manual |
| no guessed formats | fixtures and mappings encode assumptions nobody verified | "the Narrative Forge contract has not been inspected" in the PR | a hand-written "Narrative Forge example" fixture | review | manual |

Exception: documentation may name Narrative Forge concepts while stating that its contract is
uninspected.

## 2. Inspect the real contract first

Before writing the adapter:

1. Locate the actual, versioned Narrative Forge exchange contract and record its source, version
   and capture date. If no versioned contract exists, stop: request one rather than reading its
   storage or internal types.
2. List every concept it carries and classify each against Script IR: exact, approximate,
   Cantos-only, Narrative-Forge-only.
3. Record the decision (scope, supported versions, direction, lossy fields) with the
   [ADR template](../../../../templates/adr.md), and promote the backlog item with its own
   acceptance and review boundary.

## 3. Adapter shape

```text
Narrative Forge export file (versioned, untrusted)
   ↓ adapter: version probe → raw DTO for that version (mirrors the external format)
   ↓ pure map → current Script IR raw document + ConversionNotes
   ↓ the ordinary import path: validate, provenance, review, draft save
Cantos draft

Cantos ScriptRevision (pinned, immutable)
   ↓ pure map → Narrative Forge raw DTO for one target version + ConversionNotes
Narrative Forge exchange file
```

- The adapter lives in an infrastructure module (proposed name `narrative_forge`), depends on
  Script IR, and nothing in the domain depends on it.
- Imported content is imported content: it gets a `SourceDocument`, an unknown or pending rights
  reference and creator review exactly like a DOCX ([`import-and-adaptation.md`](import-and-adaptation.md)).
- Export works from a pinned revision, never from a mutable draft, so an exported file names
  `(ScriptRevisionId, ContentDigest)`.
- The adapter supports an explicit list of Narrative Forge versions and rejects others with the
  same actionable error shape as [`schema-versioning.md`](schema-versioning.md#2-version-identifiers-and-dispatch).
- Mapping functions are pure. Reading files, calling a service or touching storage stays in the
  shell around them.

## 4. The mapping table

Keep the table next to the adapter and update it in the same change as the mapping code. Fill it
only from the inspected contract; the rows below are the template, not facts.

| Narrative Forge field (contract vX) | Script IR field | Direction | Fidelity | Note kind when lossy | Fixture |
|---|---|---|---|---|---|
| `<field>` | `episode.acts[].scenes[].dialogues[].text` | both | exact | — | `<fixture>` |
| `<field>` | `delivery.emotion` | import | approximate (label table) | `Approximated` | `<fixture>` |
| `<field>` | — | import | dropped | `Dropped` | `<fixture>` |
| — | `sound_cues[].anchor` | export | dropped | `Dropped` | `<fixture>` |

Rules for the table:

- Every external field appears, including the ones Cantos ignores; an unlisted field in an import
  is a rejection, not a silent drop.
- A `Dropped` or `Approximated` row produces a `ConversionNote` at runtime, shown to the creator
  before the draft is saved.
- Speech-affecting fields (text, speaker, delivery, pronunciation, language) are either exact or
  explicitly reviewed: an approximation there changes audio.

## 5. Round-trip laws and fixtures

| Law | Domain | Oracle |
|---|---|---|
| Cantos → NF → Cantos preserves the content digest | revisions inside the NF-representable subset | property over generated reachable scripts |
| outside that subset, the notes name every loss | all revisions | property: dropped paths ⊆ the table's lossy rows |
| NF → Cantos → NF preserves Narrative Forge's own semantic equality | recorded Narrative Forge exports | recorded fixtures |
| unknown versions and fields are refused | crafted cases | example tests with exact errors |
| conversion is deterministic | any input | the same input twice → identical bytes and notes |

Fixture rules:

- Recorded Narrative Forge fixtures are captured from real exports, with source, contract version
  and capture date beside them. They are never produced by Cantos code: regenerating them from the
  adapter turns an external oracle into a circular round trip.
- Cover the spread: a normal multi-speaker scene with narration, missing optional fields,
  unexpected extra fields, Vietnamese text in NFD, and the version just outside the supported
  set.
- Use permitted content only, such as the original `Ánh đèn cuối sân khấu` sample.

## 6. Identity mapping and re-import

Narrative Forge identifiers are not Cantos identifiers: their stability depends on another
project's policy. Keep them as provenance, `ExternalRef { system: NarrativeForge, id }`, and mint
Cantos IDs in the shell.

For repeated imports of the same work, persist a mapping `(ExternalRef → Cantos ID)` so a
re-import updates the same dialogue identities instead of creating new ones (which would detach
approvals and force re-renders). The mapping is append-only; a conflicting second mapping is an
error the creator resolves. Treat this as a proposal for the integration ADR.

## 7. Tests, enforcement and report

| Claim | Evidence level once implemented |
|---|---|
| Cantos builds and tests without Narrative Forge present | `statically-checked` by a manifest/dependency check (proposed) and a CI job without the other checkout |
| mapping is faithful within the subset | `property-tested` |
| recorded exports convert as expected | `differentially-tested` |
| lossy conversions are visible | `example-tested` on exact notes |

Report additions for an adapter change: Narrative Forge contract version and capture source,
supported versions, mapping-table rows changed, lossy fields added or removed, fixtures added and
whether any speech-affecting mapping is approximate.
