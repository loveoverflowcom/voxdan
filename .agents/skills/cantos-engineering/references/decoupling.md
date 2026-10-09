# Decoupling and module boundaries

> **Scope.** Decide where code lives and which way dependencies point in the Cantos modular
> monolith: domain, application, production, infrastructure, Web and mobile boundaries; ports and
> adapters; when a crate, service, trait or shared package is justified; provider neutrality and
> the Narrative Forge boundary. Use before creating a module, crate, trait, DTO, adapter or shared
> package, and when reviewing an import edge.

The owning design is [architecture overview](../../../../docs/architecture/overview.md) and
[decision 0001](../../../../docs/decisions/0001-modular-monolith.md). This reference adds the
working method; it does not redefine the boundary table there.

## The direction rule

```text
domain values + pure rules          no I/O, no framework, no provider, no clock
        ↑
application                         permissions, revision transitions, gates, planning
        ↑
adapters                            PostgreSQL, object storage, TTS, adaptation AI, audio tools
        ↑
entrypoints                         Axum handlers, workers, CLI, Leptos, CMP
```

An arrow means "may depend on". Data flows both ways; imports flow one way. Check `use` edges and
manifests, not a data-flow diagram.

| Rule | Why | Good | Counterexample | Oracle |
|---|---|---|---|---|
| Domain and Script IR import no Axum, SQL client, Leptos, Compose, provider SDK or Narrative Forge type | a framework or provider change must not rewrite business rules | `fn plan_regeneration(prev: &RenderManifest, next: &ScriptRevision) -> RegenerationPlan` | a domain function taking `&PgPool` or returning `axum::Response` | manifest/import review; a dependency check once crates exist (proposed) |
| Map foreign types at the edge | provider payloads and DTOs change on someone else's schedule | adapter converts `ProviderVoiceResponse` into `SynthesisOutcome` | domain enum with a `#[serde(rename = "...")]` copied from a vendor API | adapter contract test with a recorded fixture |
| One owner per policy | duplicated rules diverge | progress reconciliation decided once in Rust, clients render the outcome | Kotlin and Leptos each reimplementing "is this approval stale" | review: search for the rule in every renderer |
| Web and CMP share contracts and design semantics, never runtime code | Leptos and Compose evolve independently | generated client types from one OpenAPI/JSON Schema | Kotlin decoding Rust memory layouts or a shared UI runtime | contract fixtures validated by both clients |

## Ports exist only at real boundaries

A trait is a port when there are at least two real implementations with distinct reasons to
exist: the production adapter and a deterministic test fake count. A trait added only so a mock
can count calls is not a port; extract the decision into a pure function instead.

```rust
// Illustrative. A port at a real boundary: production calls a provider, tests use a script.
pub trait SpeechSynthesizer {
    async fn synthesize(&self, request: &SynthesisRequest) -> Result<SynthesisOutcome, SynthesisError>;
}
```

Keep ports narrow and named for the capability the application needs, not for the vendor:
`SpeechSynthesizer`, `ObjectStore`, `AdaptationDrafter`, never `ElevenLabsClient` or
`OpenAiService` at the application level. Provider capabilities (which emotion or pronunciation
controls exist) are data returned by the adapter, so the core can reject unsupported settings
without knowing the vendor.

## When a new package is justified

The monolith starts with as few packages as the first slice needs
([architecture § Initial source layout](../../../../docs/architecture/overview.md#initial-source-layout)).
Create a crate, Gradle module or shared package only when one of these is true and you can name
the consumer:

- a separate build target needs it (the WASM Web client consuming a pure core, a worker binary);
- it must compile without a dependency the rest of the code needs (a pure domain crate without
  `tokio` or a SQL client);
- an independent consumer exists today, not "may exist later".

Never create `common`, `utils`, `shared` or `helpers` buckets: they become dependency magnets
with no reason to change. Split by semantic ownership — `script`, `production`, `publication`,
`listening` — and only as consumers appear.

A separate worker process is an operational boundary, not a new service: it shares the domain,
application and PostgreSQL job model.

## Narrative Forge and other external projects

Reuse concepts, never internal types. Exchange happens through a versioned file or data contract
with a compatibility-tested adapter at the boundary; Cantos builds and runs without that
repository, its database or its service. The adapter owns mapping tables, round-trip fixtures
and the lossy-conversion log ([Script IR § Narrative Forge integration](../../../../docs/architecture/script-ir.md#narrative-forge-integration)).
See [`cantos-script-ir`](../../cantos-script-ir/references/narrative-forge-adapter.md).

## Provider neutrality

TTS, adaptation AI, audio tooling, storage and CDN are adapters. The application speaks in
provider-neutral requests, capabilities and outcomes, and persists the actual provider, model and
request identifiers it used. Switching provider is a configuration and adapter change plus new
fingerprints, never a domain change. See
[`cantos-production-pipeline`](../../cantos-production-pipeline/references/provider-adapters.md).

## Review questions

1. Which module owns this rule, and is that the only place it is decided?
2. Does any import point outward (domain → adapter, application → Axum, core → Leptos)?
3. Is this trait a port with two real implementations, or a mock seam hiding a rule?
4. Does this new package have a consumer today, and a reason to change of its own?
5. Can the behavior be tested without a database, provider or runtime? If not, which decision is
   still trapped in the shell?

## Stop conditions

Do not introduce a generic effect framework, event bus, plugin system, DI container or workflow
engine before a slice demonstrates the need. Thin CRUD and pure type mappings need no extra
layer. Record a decision with [`templates/adr.md`](../../../../templates/adr.md) when a boundary
choice is consequential.
