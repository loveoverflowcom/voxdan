# Native playback

> **Scope.** The Kotlin playback model and its Android and iOS adapters: the media engine port,
> the pure reducer shared by both platforms, interruptions, audio focus and route changes, sleep
> timer, restore after process death, signed-URL refresh, lock-screen and notification controls,
> and logging. Use for any change to the player, the media session or a platform media adapter.

Owners, not restated here:

- **Meaning** of every playback state, user intent, failure reason and completion rule:
  [`playback-semantics.md`](../../cantos-listening/references/playback-semantics.md), including
  the shared JSON vectors that Theatre Web and mobile must both pass.
- **Product behavior** on mobile:
  [mobile.md § Playback behavior](../../../../docs/architecture/mobile.md#playback-behavior)
  and [business rule 26](../../../../docs/product/business-rules.md#listening-and-mobile).
- **Where the session lives:**
  [cmp-architecture.md § One media session owner](cmp-architecture.md#one-media-session-owner).

This reference supplies the technique: how to model those rules in Kotlin, where each platform
fact enters, and what test proves each behavior. All code is illustrative and proposed. Every
platform API below is a **candidate to verify** against current official documentation and the
pinned library version; none has been exercised in this repository.

## The shape

```text
in-app buttons ─┐
OS remote / lock-screen commands ─┼─→ PlayerEvent ─→ one serialized controller loop
engine, focus, route, interruption ┘                         │
                                                             ↓
                        reduce(state, event) → Transition(state, effects)   pure, commonMain
                                                             │
          ┌──────────────────────┬───────────────────────────┼───────────────────────┐
          ↓                      ↓                           ↓                       ↓
     EngineCommand        NowPlaying update           Checkpoint write        RefreshAccess
     (MediaEngine)        (OS session / center)       (local store)           (listener API)
```

OS commands and in-app buttons produce the **same** events, so the lock screen and the full
player cannot diverge. The engine reports facts; it never decides whether to resume.

## Model orthogonal facts, project the named states

mobile.md lists idle, loading, playing, paused, buffering, seeking, ended and failed, with a
separate reason for temporary access or connectivity failures, and requires the UI to tell user
pause from buffering and interruption. Those are **projections**. Store the independent facts
and derive the named state with a pure function whose names come from `playback-semantics.md`:

```kotlin
sealed interface PlayerState {
    data object Idle : PlayerState
    data class Active(
        val session: PlayerSession,
    ) : PlayerState
}

data class PlayerSession(
    val publication: PublicationRef,       // episode + immutable publication revision
    val intent: ListenerIntent,            // what the listener asked for
    val engine: EngineStatus,              // what the engine last reported
    val interruption: Interruption?,       // imposed by the system; orthogonal to intent
    val position: PlaybackPosition,
    val speed: PlaybackSpeed,
    val sleepTimer: SleepTimer?,
    val failure: PlaybackFailure?,
    val accessGeneration: Int,             // rejects stale refreshed URLs
)

sealed interface ListenerIntent {
    data object Play : ListenerIntent
    data class Paused(val cause: PauseCause) : ListenerIntent
}

enum class PauseCause { Listener, SleepTimer, AudioRouteLost, FocusLostPermanently, Restored }
```

A single flat enum of `Playing, Paused, Buffering, …` cannot say "the listener wants to
play, a call is in progress and the network is buffering" — exactly the case where a wrong state
claims successful playback during an interruption.

## The port reports facts

The `MediaEngine` port ([cmp-architecture.md](cmp-architecture.md#platform-adapters-behind-ports))
emits typed events and executes commands. Map every platform callback into one of a closed set:

```kotlin
sealed interface EngineEvent {
    data class Prepared(val duration: MediaDuration) : EngineEvent
    data object Buffering : EngineEvent
    data object Rendering : EngineEvent                    // audio is actually audible
    data class Paused(val source: EnginePauseSource) : EngineEvent
    data class PositionJumped(val position: PlaybackPosition) : EngineEvent
    data object ReachedEnd : EngineEvent
    data class Failed(val reason: EngineFailureReason) : EngineEvent
    data class InterruptionBegan(val kind: InterruptionKind) : EngineEvent
    data class InterruptionEnded(val systemSuggestsResume: Boolean) : EngineEvent
    data object OutputRouteLost : EngineEvent              // headphones removed
}
```

**Exactly one component owns resumption.** Either the engine's built-in focus handling resumes
after a transient loss and the adapter reports it, or built-in handling is off and the reducer
issues `Play`. Both at once produce a double resume or a resume after the listener paused during
the interruption. Record which one in the adapter's doc comment and test it on a device.

## The reducer

```kotlin
// PlayerEvent = listener intents + mapped EngineEvents + timer and access events.
// This sketch assumes the reducer owns resumption (see above).
fun reduce(session: PlayerSession, event: PlayerEvent): Transition = when (event) {
    PlayerEvent.ListenerPaused -> Transition(
        session.copy(intent = ListenerIntent.Paused(PauseCause.Listener)),
        listOf(PlayerEffect.Engine(EngineCommand.Pause), PlayerEffect.Checkpoint),
    )
    is PlayerEvent.InterruptionBegan -> Transition(
        session.copy(interruption = event.kind),   // intent is unchanged
        listOf(PlayerEffect.Engine(EngineCommand.Pause)),
    )
    is PlayerEvent.InterruptionEnded -> {
        val resume = event.systemSuggestsResume && session.intent == ListenerIntent.Play
        Transition(
            session.copy(interruption = null),
            if (resume) listOf(PlayerEffect.Engine(EngineCommand.Play)) else emptyList(),
        )
    }
    PlayerEvent.SleepTimerExpired -> Transition(
        session.copy(intent = ListenerIntent.Paused(PauseCause.SleepTimer), sleepTimer = null),
        listOf(PlayerEffect.Engine(EngineCommand.Pause), PlayerEffect.Checkpoint),
    )
    // … every other event, exhaustively, without else
}
```

The listener pausing during a call changes `intent`, so the end of the call cannot resume. A
sleep-timer expiry does the same. The reducer has no clock, no engine and no I/O; the controller
loop resolves time and interprets effects
([KT-UPDATE](kotlin-conventions.md#kt-update--retry-safe-state-transforms)).

## Behavior → technique → oracle

| Owned behavior (link above) | Technique | Oracle (test name as theorem) |
|---|---|---|
| interruption or focus loss never shows as listener playback | interruption is a separate fact; projection prefers it over `intent` | `focus_loss_never_projects_listener_playing` |
| no resume after a pause made during an interruption | pause changes `intent`; resume requires `intent == Play` | `listener_pause_during_interruption_survives_interruption_end` |
| no resume after an explicit pause or sleep-timer expiry | both set `Paused(cause)`; only a new listener `Play` clears it | `interruption_end_after_sleep_timer_expiry_does_not_resume` |
| headphone removal pauses (platform convention) | `OutputRouteLost` → `Paused(AudioRouteLost)`; reconnect does not resume | `route_loss_pauses_and_reconnect_does_not_resume` |
| cold launch offers resume, never autoplays | `Restored(checkpoint)` → `Paused(Restored)` plus a resume prompt; no engine `Play` | `restore_from_checkpoint_emits_no_play_command` |
| position survives access refresh | `RefreshAccess(generation)` effect; reload at the last known position; stale generation ignored | `stale_access_generation_is_rejected_and_position_kept` |
| bounded retries with distinct final errors | retry budget in state; `AccessExpired`, `Forbidden`, `Offline` stay distinct | `third_refresh_failure_becomes_final_access_error` |
| episode completion is a product rule, not engine "ended" | `ReachedEnd` feeds the completion rule from playback semantics; seeking to the end alone does not complete unless that rule says so | shared vectors from `cantos-listening` |
| next episode only with the explicit autoplay preference | `ReachedEnd` consults the preference value in state | `end_of_episode_without_autoplay_stops_and_offers_next` |
| remote progress from another device never moves the playhead | a remote position is an offer projected into the UI; only a listener choice emits a seek | `remote_progress_update_emits_no_seek` |
| a new release while playing is an offer, never a silent swap | the session pins its `PublicationRef`; a newer release becomes an offer field | `newer_release_during_playback_keeps_pinned_publication` |

Each row is `example-tested` in `commonTest` first, and the reducer runs the shared vector corpus
(`differentially-tested` on each named Kotlin platform). The platform half is `device-tested`
per [cmp-testing.md](cmp-testing.md).

## Sleep timer

Model the timer as a value: `SleepTimer.At(deadline: MonotonicMark)` or `SleepTimer.EndOfEpisode`.
Use a monotonic time source so a wall-clock change cannot shorten or extend it. The controller
schedules `SleepTimerExpired`; the reducer pauses. The OS controls show only a subset of app
actions, so the in-app player and its accessibility text must state the remaining time
([mobile.md § UI and accessibility](../../../../docs/architecture/mobile.md#ui-and-accessibility)).

The product docs do not say whether a timer survives process death or keeps counting while
playback is paused. If `playback-semantics.md` has not settled it, record the product decision
before encoding one ([`cantos-listening`](../../cantos-listening/SKILL.md) treats an unowned
reducer arm as a plan gap). Whatever is decided, restore never resumes audio, and the checkpoint
must carry enough timer data for the chosen rule.

## Restore after process death

mobile.md names what to persist: active publication, position, listener intent, speed and timer
semantics. Write a checkpoint value (immutable, versioned, last-write-wins locally) on pause,
seek, episode change, speed change, background transition and at a bounded interval while
playing — never per second to the server; progress sync goes through the outbox
([downloads-and-local-state.md](downloads-and-local-state.md#progress-outbox)).

- On cold launch, decode the checkpoint, rebuild state as `Paused(Restored)` and show the resume
  offer with the real position ("Tiếp tục từ 12:05?"). Do **not** activate the audio session or
  request focus yet: on iOS, activating a playback session interrupts the listener's other audio
  even if nothing plays.
- A system-offered resumption (for example a media resumption control on Android) is an explicit
  listener `Play` routed through the same event. Verify which callback the pinned Media3 version
  offers for it.
- If the checkpoint names a publication the listener can no longer access, keep the position
  attached to that revision and explain the change, per
  [mobile.md § Durable listening progress](../../../../docs/architecture/mobile.md#durable-listening-progress).

## Signed URLs: identity is not transport

The media item's identity is `PublicationRef` plus the asset checksum; a signed URL is a
short-lived transport credential
([storage and delivery](../../cantos-publication/references/storage-and-delivery.md)).
Request a fresh URL before the server-stated expiry and after a supported authorization failure;
increment `accessGeneration` on each request and ignore responses for an older generation.
Reload at the last confirmed position and keep the listener's intent. Retries are bounded with
backoff; exhaustion becomes a distinct final failure while position and publication remain.

Candidate techniques: on Android, a resolving data source that substitutes a fresh URI per
request without rebuilding the media item; on iOS, replacing the player item and seeking, or a
resource-loader delegate. Verify seek accuracy and caching for each with the chosen codec and
CDN range behavior before adopting one.

## Lock screen, notification and remote commands

Derive what the OS shows from the same state snapshot with a pure projection:

```kotlin
fun nowPlaying(session: PlayerSession, episode: EpisodeSummary): NowPlaying? =
    if (episode.publication != session.publication) {
        null // never show metadata for a different publication than the one playing
    } else {
        NowPlaying(
            title = episode.title,                // "Một lời hẹn"
            work = episode.workTitle,             // "Ánh đèn cuối sân khấu"
            duration = episode.duration,
            elapsed = session.position,
            rate = if (session.isAudible()) session.speed.toOsRate() else OsPlaybackRate.Zero,
            enabledCommands = session.availableCommands(),
        )
    }
```

Publish on discontinuities (play, pause, seek, speed, episode change), not every tick: both
platforms extrapolate elapsed time from the rate. Enable only commands the state supports (no
"next" without a next episode). Artwork, title, duration and position must match the active
publication; the oracle switches episodes and asserts the projected publication equals the
active one, then a device recording confirms the lock screen.

## Android adapter — candidates to verify

| Concern | Candidate | Verify against current docs and the pinned version |
|---|---|---|
| engine | Media3 ExoPlayer | supported codecs and seeking for the chosen delivery format |
| background + session | Media3 `MediaSessionService` with `MediaSession`; UI attaches through a controller | foreground-service type and permission rules on the targeted API levels; behavior when the task is removed (a product decision for the ADR) |
| notification | Media3's media notification provider | runtime notification permission and any media-session exemption on the targeted API levels |
| audio focus | engine-managed focus via audio attributes with usage media | whether speech content type pauses instead of ducking; which reasons the player reports |
| becoming noisy | engine option to pause on becoming noisy | that it reports a distinct reason the adapter can map to `OutputRouteLost` |
| reason mapping | play-when-ready change reasons and playback suppression reasons | exact constants, so focus loss, noisy, remote and listener requests stay distinct |
| errors | playback exception error codes → `EngineFailureReason` | never log the exception message or data spec: it contains the URI |

## iOS adapter — candidates to verify

| Concern | Candidate | Verify |
|---|---|---|
| engine | `AVPlayer` / `AVPlayerItem`; `timeControlStatus` and its waiting reason for buffering | status and end/failure notifications for the pinned iOS range |
| audio session | `AVAudioSession` category `.playback`, mode `.spokenAudio` as a candidate for drama | how the mode changes interruption and mixing with other spoken audio; activate only on play, deactivate notifying others on stop |
| background | the `audio` background mode capability | capability and entitlement setup in the Xcode project |
| interruptions | interruption notification (began/ended, `shouldResume` option) → `InterruptionBegan` / `InterruptionEnded` | the reducer still decides; `shouldResume` is a suggestion |
| route change | route-change notification with old-device-unavailable reason → `OutputRouteLost` | whether the player pauses itself on that route change; report the reason either way |
| Now Playing | `MPNowPlayingInfoCenter`; `MPRemoteCommandCenter` play, pause, toggle, change position, skip intervals, next/previous, rate | enable and disable commands from state; set rate 0 when paused |
| errors | map `NSError` domain and code only | `userInfo` can carry the failing URL; never log it |
| language | Swift adapter implementing a Kotlin interface, or Kotlin/Native AVFoundation bindings | an ADR; observers removed on release, callbacks marshalled onto the controller loop |

Downloaded files must stay readable while the device is locked; see the file-protection note in
[downloads-and-local-state.md](downloads-and-local-state.md#files-and-storage-locations).

## Logging and diagnostics

Log `publication_id`, event names, reason codes and generation numbers. Never log tokens, signed
URLs, query strings, cookies or raw platform error messages; wrap URL values per
[KT-SECRETS](kotlin-conventions.md#kt-secrets--values-that-must-not-print). The oracle is a
sentinel test: play, refresh access and fail with a fake signed URL, then assert the sentinel is
absent from captured logs, checkpoint files and crash breadcrumbs.

## Common mistakes

- Treating the engine's "playing" flag as the listener's intent.
- Letting both built-in focus handling and the reducer resume after a call.
- Creating an engine inside a screen; a second notification appears after back navigation.
- Autoplaying after cold launch because the restored state said `Play`.
- Storing the signed URL in the checkpoint as the item's identity.
- Updating Now Playing every second instead of on discontinuities, or showing the previous
  episode's artwork after a switch.
- Proving any of the above with a desktop preview; only a device run proves OS behavior.
