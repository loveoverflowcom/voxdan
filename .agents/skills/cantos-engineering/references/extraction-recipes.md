# Extraction recipes

> **Scope.** Worked before-and-after recipes for moving a durable Cantos decision out of a handler,
> worker loop, adapter, Leptos component or Kotlin screen into a pure core, leaving a thin shell
> that resolves facts and executes effects. Use *while moving code*, after
> [`functional-core.md`](functional-core.md) has told you that a decision belongs in a core. It is
> not a routine review checklist and it does not apply to thin CRUD or to an adapter that only
> maps types.

Cantos has no application code yet. These recipes are for the first implementation and for any
later refactor; when writing new code, write the core first and never create the entangled shape
below. All types, crates and modules are **illustrative and proposed**.

## Contents

1. A handler becomes a decision and a transaction
2. Decide, then commit — rejection that changes nothing
3. A worker loop becomes a decision enum and an executor
4. A UI component stops deciding
5. Safe extraction sequence

## 1. A handler becomes a decision and a transaction

The mixed handler hides the publication rule between I/O calls and reads facts at different
instants.

```rust
// Before: the rule is smeared across awaits; facts are read at different times.
async fn publish(state: &AppState, req: PublishRequest) -> Result<ReleaseId, ApiError> {
    let candidate = state.candidates.get(&req.candidate_id).await?;
    if !state.rights.is_cleared(&candidate).await? {
        return Err(ApiError::RightsNotCleared);
    }
    let qc = state.qc.latest(&candidate).await?;
    if qc.has_blocking_findings() {
        return Err(ApiError::QcBlocking);        // the first failure hides every other blocker
    }
    state.releases.activate(&candidate).await      // not in the transaction that read the facts
}
```

Extract the decision over plain facts; the shell reads **inside** the committing transaction:

```rust
// After. The core is pure and returns every blocker.
pub fn decide_publication(facts: &PublicationFacts) -> Result<PublishableRelease, Vec<PublicationBlocker>>;

// The shell resolves facts once, calls the core once and interprets the result.
async fn publish(state: &AppState, req: PublishRequest) -> Result<ReleaseId, ApiError> {
    let permit = authorize(&req.actor, Permission::Publish)?;        // PublishPermit witness
    let mut tx = state.db.begin().await?;
    let facts = read_publication_facts(&mut tx, &req).await?;        // rights, QC, approvals, assets
    let release = decide_publication(&facts).map_err(ApiError::blocked)?;
    commit_release(&mut tx, release, permit).await?;                 // pointer, transitions, outbox
    tx.commit().await?;
    Ok(release_id)
}
```

- **Move semantic rules into the core; keep transport validation at the transport boundary.** A
  malformed request is a `400` in the handler; "QC has a blocking finding" is a core blocker.
- The core takes **facts, not services**: no `AppState`, no connection, no clock
  ([`functional-core.md`](functional-core.md#pass-facts-not-services)).
- The commit function's signature takes the witness types it needs (`PublishableRelease`,
  `PublishPermit`), so a handler cannot skip the gate
  ([`types-as-proofs.md`](types-as-proofs.md)).
- Test the core with a baseline-publishable fact set plus exactly one defect per row, and one row
  with every defect asserting the full sorted list. Add a transaction test that a rejection wrote
  nothing ([`cantos-publication`](../../cantos-publication/SKILL.md)).

## 2. Decide, then commit — rejection that changes nothing

A fallible change must not mutate shared state before it knows it will succeed.

```rust
// Before: mutate first, undo on error. A crash between the two leaves a half-applied change.
fn recast(cast: &mut Casting, character: CharacterId, voice: VoiceId) -> Result<(), CastingError> {
    cast.assign(character, voice);                 // already changed
    if !provider.supports(&voice) {
        cast.unassign(character);                  // hope nothing observed the middle state
        return Err(CastingError::VoiceUnsupported(voice));
    }
    Ok(())
}
```

```rust
// After: validate against facts, then produce a new value.
pub fn recast(cast: &Casting, change: &Recast, voices: &VoiceCapabilities)
    -> Result<Casting, CastingError>
{
    voices.require(&change.voice, &change.required_controls)?;      // may fail; nothing changed yet
    Ok(cast.with_voice(change.character, change.voice.clone()))     // a new value
}
```

Test every rejection class **and assert that the input value is unchanged** (equality with a clone,
or the same canonical bytes) after the failure. In persistence, the same rule is "insert or update
only after the decision succeeded, in one transaction"
([`persistence.md`](persistence.md)). Budget reservation, draft edits and gate decisions follow the
same shape.

## 3. A worker loop becomes a decision enum and an executor

The loop that sleeps, retries and counts is untestable without time and a provider.

```rust
// Before: policy, time and effect are interleaved.
loop {
    match provider.synthesize(&request).await {
        Ok(audio) => return Ok(audio),
        Err(_) if attempts < 3 => { attempts += 1; tokio::time::sleep(Duration::from_secs(2)).await; }
        Err(e) => return Err(e),
    }
}
```

```rust
// After: the core decides; the shell executes and reports back.
pub fn decide_next_step(facts: &RunFacts) -> StepDecision;     // Cancel | Reuse | Reconcile | Fail
                                                               // | WaitUntil | Pause | Dispatch
// shell (illustrative)
let facts = load_run_facts(&db, step, clock.now(), jitter.sample()).await?;
match decide_next_step(&facts) {
    StepDecision::Dispatch(plan) => execute_dispatch(plan).await?,   // writes intent, then sends
    StepDecision::WaitUntil(at)  => reschedule(step, at).await?,
    // … one arm per variant, no wildcard
}
```

- The clock instant and the jitter sample are **inputs** the shell samples once; the core never
  reads time or randomness ([`functional-core.md`](functional-core.md#total-and-deterministic)).
- The attempt counter lives in durable attempt history, so a re-claim cannot reset it.
- Test the decision with an exhaustive table over the fact dimensions and the backoff function over
  attempts × jitter; test the shell with a scripted provider and named fault points
  ([`fault-injection-testing.md`](../../cantos-production-pipeline/references/fault-injection-testing.md)).
- A timeout is not `Err(_)`: the core distinguishes a not-sent failure from an ambiguous one, which
  must be reconciled before any repeat
  ([`provider-adapters.md`](../../cantos-production-pipeline/references/provider-adapters.md#5-reconcile-before-you-repeat)).

## 4. A UI component stops deciding

```rust
// Before (Leptos): the view decides domain staleness from two numbers.
let can_publish = move || approval().revision == script().revision && qc().passed;
```

```rust
// After: the backend returns a typed fact; the view maps it with a pure, exhaustive function.
pub fn publish_presentation(a: &Availability<PublishBlocker>) -> PublishPresentation { /* match, no `_` */ }
view! { <PublishButton presentation=move || publish_presentation(&availability()) /> }
```

```kotlin
// After (Compose): the same mapping over a sealed type.
fun publishPresentation(a: Availability<PublishBlocker>): PublishPresentation = when (a) { /* exhaustive */ }
```

Local element state (a menu open, focus, scroll) stays in the component. The mapping is a pure
function with a table test; the server re-runs the decision on the request. Details and the state
matrix: [`component-states.md`](../../cantos-ui-design/references/component-states.md#2-availability-is-a-type).

## 5. Safe extraction sequence

1. **Freeze observable behavior** with characterization tests if coverage is weak. For a bug,
   write the failing test first.
2. **Identify the plain facts** the rule consumes (values, not services) and the one clock instant,
   ID or sample it needs.
3. **Introduce the domain input, output and error types** without moving behavior; make them
   `Clone` + `PartialEq` where the oracle needs it.
4. **Extract a pure function** and keep the old shell as its only caller.
5. **Add exact semantic and property tests** around the pure function: exact error variants,
   rejection changes nothing, determinism.
6. **Delete mocks** that no longer assert an adapter contract; keep the ones that do.
7. **Move types and modules last,** once behavior is stable, and re-run the dependency checks so
   the domain still imports no framework, SQL client or provider SDK
   ([`decoupling.md`](decoupling.md)).

Do not combine extraction with a public API redesign, a dependency upgrade or unrelated cleanup:
each independent axis makes a regression harder to localize. Report `No core extracted: <reason>`
when you decide against it, as the [required order](../SKILL.md#the-required-order) asks.
