# Boundary hardening for refined types

> **Scope.** Keep validated Cantos values valid when they cross a trust boundary: HTTP JSON,
> Script IR files, SQL rows, object metadata, provider responses, AI adaptation output, fixtures
> and migrations. Use when adding `Deserialize`, row decoding, a fixture, an import path or a
> migration around a type with a constructor invariant.

## Raw at the edge, refined inside

```text
wire / file / row / provider payload / AI output
        ↓  decode (syntax only)
raw DTO with plain fields
        ↓  validate + convert (semantics, all field errors at a human boundary)
domain value
```

Use one type for wire and domain only when the invariant is intrinsic, stable across versions,
decode failure is the right boundary behavior, and no migration needs pre-invariant raw data.
Otherwise keep a raw DTO and convert explicitly.

At a human-facing boundary (Studio import, API request) collect all independent field errors so
the creator can fix them in one pass. Inside the domain, fail fast with the first exact variant.

## Route serde through validation

`#[derive(Deserialize)]` on a refined struct constructs fields without calling its constructor.
Route through `TryFrom`:

```rust
// Illustrative.
#[derive(serde::Deserialize)]
#[serde(try_from = "RawSpokenText")]
pub struct SpokenText(String);

#[derive(serde::Deserialize)]
struct RawSpokenText(String);

impl TryFrom<RawSpokenText> for SpokenText {
    type Error = ScriptIssue;

    fn try_from(raw: RawSpokenText) -> Result<Self, Self::Error> {
        SpokenText::new(raw.0)
    }
}
```

Error messages name the invalid class and expected range, never a secret or a full private
manuscript excerpt.

## Decode rows explicitly

Decode into a raw row type, then convert with the domain constructor. An invalid stored row is
corruption or version skew, not a user error: fail loudly with the row's stable identity, count
it, and never `filter_map` it away unless dropping data is an explicit, counted policy.

```rust
// Illustrative.
struct DialogueRow { id: Uuid, speaker_id: Uuid, text: String, position: i32 }

impl TryFrom<DialogueRow> for Dialogue {
    type Error = StorageCorruption;

    fn try_from(row: DialogueRow) -> Result<Self, Self::Error> {
        Ok(Dialogue::new(
            DialogueId::from_storage(row.id),
            CharacterId::from_storage(row.speaker_id),
            SpokenText::new(row.text).map_err(|issue| StorageCorruption::dialogue(row.id, issue))?,
            ScenePosition::try_from(row.position).map_err(|_| StorageCorruption::position(row.id))?,
        ))
    }
}
```

## Untrusted producers in Cantos

| Producer | Treat as | Hardening |
|---|---|---|
| Imported TXT/Markdown/DOCX | hostile bytes | size/type limits, safe parser, no macro or external entity execution, preserved original |
| AI adaptation output | unvalidated draft | schema + semantic validation, speaker resolution, coverage check against source, creator review |
| TTS provider response | untrusted media and metadata | decode and measure audio, verify duration/format, never trust provider-reported success alone |
| Client API request | untrusted | raw DTO, authorization in the backend, idempotency key validation |
| Listener progress from devices | untrusted ordering | server revision decides; device clocks are context |
| Old fixtures and stored rows | possibly old version | versioned decoders and migration before validation |

## Audit bypasses

Search for: `pub` fields; `Default` on non-empty or non-zero values; `DerefMut`/`AsMut`;
`Deserialize` or row decoding straight into refined fields; property-test generators building
fields directly; widened `pub(crate)` constructors; migrations and old fixtures; macros that
expand to struct literals; `_unchecked` helpers without a documented obligation.

## Migrations and versions

A migration that changes the meaning of stored data needs: fixtures of the **old** shape, the
migration, then validation through the current constructors. A round trip through the new code
never proves that old data still reads. Unknown schema versions and unknown enum variants follow
an explicit compatibility policy (reject with an actionable error, or preserve as opaque data),
never a silent default.

## Minimum tests

- deserialization rejects each invalid class with the exact error;
- round trip preserves canonical value (and canonical bytes where digests depend on them);
- each supported old fixture decodes and validates;
- a corrupt row fails loudly and names the record;
- AI output with an invented speaker, missing source text or narration mislabeled as dialogue
  is reported, not accepted.
