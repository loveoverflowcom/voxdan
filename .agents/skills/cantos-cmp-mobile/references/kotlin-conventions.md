# Kotlin and Compose conventions

> **Scope.** Handwritten Kotlin in the Cantos Theatre CMP app: composable APIs, state ownership,
> immutable values, `StateFlow` updates, effects and lifetimes, closed state types, errors,
> coroutines, secrets in values and formatting. Use while writing or reviewing any Kotlin.
> Generated sources (API client, tokens, resources) follow their generator, not these rules.

General style lives in the foundation's
[clean-code](../../cantos-engineering/references/clean-code.md#kotlin-and-compose) and
[immutability](../../cantos-engineering/references/immutability.md#value-semantics-in-kotlin)
references; ownership lives in [cmp-architecture.md](cmp-architecture.md). Each rule below has
one canonical definition here. **MUST** protects a stated contract; **SHOULD** admits a reasoned
local alternative recorded in the PR. Enforcement is **manual** everywhere today: no Kotlin
compiler settings, formatter or linter exist in the repository yet.

Snippets are illustrative and proposed, not existing code.

## KT-API — composable contract

| Field | Contract |
|---|---|
| Level / scope | SHOULD for every layout-emitting composable; MUST preserve caller-visible geometry, focus and semantics when changing an existing one |
| Rule / rationale | Required parameters first, then `modifier: Modifier = Modifier` as the first optional parameter, applied **once** at the root; events are `onX` callbacks carrying values (`onPlay(EpisodeId)`); slots are named for their purpose. Callers own layout and input; a modifier applied twice or to a child silently moves hit areas and semantics. |
| Good / counterexample | `Row(modifier = modifier.fillMaxWidth())` at the root vs. ignoring `modifier`, applying it to an inner `Icon`, or passing the whole controller into a row |
| Oracle / enforcement | Call-site review; `semantics-tested` bounds and touch-target assertions on the production component; Compose Rules is a candidate linter, not installed |
| Exception | A private helper scoped to one parent may follow that parent's layout contract |

```kotlin
@Composable
fun EpisodeRow(
    episode: EpisodeRowState,
    onPlay: (EpisodeId) -> Unit,
    onDownload: (EpisodeId) -> Unit,
    modifier: Modifier = Modifier,
) {
    Row(modifier = modifier.fillMaxWidth()) {
        // title "Một lời hẹn", duration, availability, actions
    }
}
```

## KT-STATE — one mutable authority, projections for the rest

| Field | Contract |
|---|---|
| Level / scope | MUST for application and screen state; SHOULD for reusable components (values in, events out) |
| Rule / rationale | Each fact has one writable owner at its lifetime. Cheap derived facts (`canResume`, `isDownloaded`, mini-player visibility) are pure functions of the owner's state, not separate flows synchronized by an effect, which drift under reordering. Facts that can be true **at the same time** stay separate: a refresh failure must not erase the list being read, and buffering while the listener intends to play is not "paused". |
| Good / counterexample | `LibraryUiState(episodes, refresh, append, offline)` vs. `Loading | Success | Error` replacing the visible list with an error screen on a failed refresh |
| Oracle / enforcement | Reducer tests asserting every concurrent combination the screen can observe; manual review for mirrored flags |
| Exception | Menu, focus, scroll, text-field and pressed state stay in the composable or a plain remembered holder; no reducer for a boolean |

```kotlin
// Counterexample: one flag-shaped enum for facts that coexist.
sealed interface LibraryUiState {
    data object Loading : LibraryUiState
    data class Success(val episodes: List<EpisodeRowState>) : LibraryUiState
    data class Error(val error: LibraryError) : LibraryUiState
}

// Better: independent facts, each with its own closed type.
data class LibraryUiState(
    val episodes: ImmutableList<EpisodeRowState>, // candidate library, see KT-VALUES
    val refresh: RefreshStatus,
    val append: AppendStatus,
    val offline: Boolean,
)
```

## KT-VALUES — immutable values, honestly

| Field | Contract |
|---|---|
| Level / scope | MUST for state exposed to UI, reducer inputs/outputs and persisted local records |
| Rule / rationale | `val` properties, `data class` updated with `copy`, transformations return new values. A read-only `List` is a **view**, not deep immutability: if the producer keeps the `MutableList`, the UI sees mutations Compose never observed. Never pass a mutable collection into UI state. `kotlinx.collections.immutable` (`ImmutableList`, `persistentListOf`) is a candidate when its stability and structural sharing pay off; record the reason when adopting it. |
| Good / counterexample | `state.copy(episodes = episodes + row)` vs. `state.episodes as MutableList` then `add(row)` |
| Oracle / enforcement | Review for `Mutable*` escaping a function; a test that mutates the input after the call and asserts the returned state is unchanged |
| Exception | Local `mutableListOf()` inside a function that returns a snapshot; purity is about effects, not about never writing `var` |

Validated values are proof barriers ([types-as-proofs.md](../../cantos-engineering/references/types-as-proofs.md)).
Prefer a value class with a private constructor and a fallible factory:

```kotlin
@JvmInline
value class PlaybackSpeed private constructor(val percent: Int) {
    companion object {
        private val Allowed = setOf(75, 100, 125, 150, 175, 200) // product choice, proposed

        fun ofPercent(percent: Int): PlaybackSpeed? =
            if (percent in Allowed) PlaybackSpeed(percent) else null
    }
}
```

Integer percent keeps equality and serialization exact; a `Double` factor does not. A
`data class` with a private constructor may still expose a public `copy` that skips validation,
depending on the Kotlin version and `@ConsistentCopyVisibility`; verify the pinned compiler or use
a value class or plain class. Route decoded API values through the same factory and prove it with
an invalid-payload test ([boundary-hardening.md](../../cantos-engineering/references/boundary-hardening.md)).

## KT-UPDATE — retry-safe state transforms

| Field | Contract |
|---|---|
| Level / scope | MUST for every `MutableStateFlow.update { }` lambda |
| Rule / rationale | `update` re-runs the lambda when another writer won the compare-and-set. The lambda is pure and deterministic: no engine call, no persistence, no `trySend`, no clock or ID generation inside. |
| Good / counterexample | `_state.update { it.copy(speed = speed) }` then `engine.setSpeed(speed)` vs. calling the engine and saving a checkpoint inside the lambda — under contention the engine is told twice and two checkpoints are written |
| Oracle / enforcement | Review; a contention test with two concurrent writers asserting effect counts |
| Exception | None inside the lambda. A serialized single writer may assign `.value = next` |

For the playback controller, prefer a **serialized event loop**: UI intents and engine events go
into one channel consumed by one coroutine that applies the pure reducer, publishes the new
state and then interprets the effects in order. That gives one writer and a deterministic effect
order without compare-and-set retries. Atomic state does not make an external effect exactly-once.

## KT-LIFETIME — effects have an owner, keys and cleanup

| Field | Contract |
|---|---|
| Level / scope | MUST for requests, subscriptions, platform listeners and timers that span composition, route, account or process lifetimes |
| Rule / rationale | Name the owner, the restart keys and the cleanup. Every async result carries the identity it was requested for (account scope, request generation, publication) and is **rejected** when that identity is no longer current — a late page for the previous account, a refreshed URL for the previous episode. Cancellation cannot undo an external write; important outcomes are acknowledged durably at their owner, not through a one-shot event. |
| Good / counterexample | `LaunchedEffect(episodeId)` loading details, with the result checked against the current `episodeId`; `DisposableEffect` removing a listener vs. registering a listener in the composable body or relying on a `SharedFlow` emission nobody may be collecting |
| Oracle / enforcement | Controlled late-response, account-switch, double-tap and disposal tests with a test dispatcher; device evidence for OS callbacks |
| Exception | Transient UI requests (focus, snackbar) may use shell effects without persistence |

Use `rememberUpdatedState` when a long-lived effect must call the latest callback without
restarting. Choose lifecycle-aware collection (`collectAsStateWithLifecycle` from the KMP
lifecycle artifacts is a candidate) only after checking the pinned artifact supports every target.

## KT-CLOSED — sealed types and exhaustive `when`

| Field | Contract |
|---|---|
| Level / scope | MUST for closed state, event and error sets |
| Rule / rationale | Model closed sets as `sealed interface` with `data object` / `data class` members, and match them with an exhaustive `when` **without `else`**. Adding `DownloadState.Revoked` must break the build at every renderer, label and accessibility description; `else ->` turns that into a silent default. |
| Good / counterexample | an exhaustive `when (state)` naming `NotDownloaded`, `Transferring`, `Verifying`, `Available` and `Failed` vs. the same `when` ending in `else -> strings.unknown` |
| Oracle / enforcement | Compiler exhaustiveness for sealed subjects (`statically-checked` once the build exists); review for `else` on sealed subjects |
| Exception | Open sets (HTTP status codes, unknown server enum values) need a deliberate fallback; map them to a sealed `Unknown(code)` at the adapter, once |

## KT-ERRORS — expected failures are values

| Field | Contract |
|---|---|
| Level / scope | MUST for adapters, controllers and reducers |
| Rule / rationale | Expected failures (offline, access expired, forbidden, storage full, checksum mismatch) are sealed values the reducer can match. `kotlin.Result` erases the error to `Throwable`, so prefer a sealed outcome; an Either library is a candidate only with an ADR. Exceptions are for programmer errors and are mapped at the platform boundary. No `!!`. Raw exception messages never reach UI or logs: platform errors often embed the request URL. |
| Good / counterexample | `DeliveryFailure.AccessExpired` driving a bounded refresh vs. `catch (e: Exception) { showError(e.message) }` |
| Oracle / enforcement | Tests assert the exact variant, never only "is failure"; review for `!!` and `runCatching` |
| Exception | Test code may use `!!` or `requireNotNull` to fail fast on fixture mistakes |

```kotlin
suspend fun fetchEpisode(id: EpisodeId): CatalogOutcome =
    try {
        CatalogOutcome.Found(client.getEpisode(id.value).toEpisode())
    } catch (e: CancellationException) {
        throw e
    } catch (e: Exception) {
        CatalogOutcome.Failed(e.toCatalogFailure())
    }
```

`runCatching` around a suspend call swallows `CancellationException`, so a cancelled screen keeps
working and may commit a stale result. Rethrow it, as above.

## KT-CONCURRENCY — structured, injected, controllable

| Field | Contract |
|---|---|
| Level / scope | MUST for coroutines in application and adapter code |
| Rule / rationale | Every coroutine belongs to a scope owned by a named lifetime: the process scope created at the composition root (with a `SupervisorJob`), the account scope cancelled on logout or switch, the route scope, the composition. No `GlobalScope`, no `runBlocking` on the main thread. Inject the dispatcher used for blocking work and the clock or `TimeSource`, so tests run on virtual time. Monotonic time drives sleep timers, backoff and lease-like waits; wall-clock time is context only. |
| Good / counterexample | `class DownloadQueue(scope: CoroutineScope, io: CoroutineDispatcher, clock: Clock)` vs. `GlobalScope.launch(Dispatchers.IO) { … System.currentTimeMillis() … }` |
| Oracle / enforcement | `runTest` with `StandardTestDispatcher` and virtual time advancing the sleep timer and retry schedule; review for unscoped launches |
| Exception | The composition root may reference platform dispatchers directly when constructing the graph |

`kotlin.time.Clock` (newer stdlib) and `kotlinx-datetime` are candidates for wall-clock time;
verify which one the pinned Kotlin version provides.

## KT-SECRETS — values that must not print

| Field | Contract |
|---|---|
| Level / scope | MUST for tokens, signed URLs, cookies and credential material |
| Rule / rationale | A `data class` `toString()` prints every field into logs, crash reports and test output. Wrap secrets in a value class whose `toString()` redacts, keep them out of navigation arguments, saved state and analytics, and log identities and reason codes instead (`publication_id`, `AccessExpired`). |
| Good / counterexample | `value class SignedMediaUrl(private val raw: String) { override fun toString() = "SignedMediaUrl(redacted)" }` vs. `Log.d(TAG, "loading $url")` |
| Oracle / enforcement | A sentinel-secret test: run the flow with a fake token and URL, capture logs and saved state, assert the sentinel never appears |
| Exception | None in shipped builds; local debugging uses fake credentials |

## KT-STYLE — readable, mechanically formatted

| Field | Contract |
|---|---|
| Level / scope | SHOULD for touched handwritten Kotlin |
| Rule / rationale | Official Kotlin style, explicit imports (no wildcards), trailing commas, one argument per line once a call wraps, named arguments for booleans and same-typed parameters, modifiers chained one per line. Readability beats line count; no semicolons or clever scope-function chains to make a file look shorter. |
| Good / counterexample | `EpisodeRow(episode = row, onPlay = onPlay, modifier = Modifier.padding(…))` vs. `EpisodeRow(row, {}, {}, Modifier.padding(…), true)` |
| Oracle / enforcement | ktlint, detekt and Compose Rules are **candidates, not installed**; choose one formatter and one small rule set in an ADR, run them on the actual source sets, and never let two engines enforce the same Compose rule |
| Exception | Do not mass-reformat existing code inside a behavior change; report formatter-touched files you did not edit |

## KT-STABILITY — measure before annotating

`@Immutable` and `@Stable` are promises to the Compose compiler, not performance switches.
Annotating a type that wraps a mutable collection makes skipped recomposition show stale data.
Prefer genuinely immutable values; claim a performance improvement only with before/after
measurements on a named device and build. Strong skipping behavior depends on the pinned Compose
compiler; verify it instead of assuming.
