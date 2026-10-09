# Component states

> **Scope.** Define and test the anatomy and state matrix of every Cantos component that shows
> domain state: buttons and toggles, chips, navigation, fields and the script editor, episode
> rows, player controls, job status, QC findings, approval and stale indicators, download state
> and confirmation dialogs. Covers availability as a type with a reason, status shown by icon plus
> text plus tone, the asynchronous states every surface needs, and the pure presentation mappings
> that keep Web and CMP agreeing. Use when adding or changing a component, mapping a domain state
> to what a person sees, or reviewing a UI for dead-end controls and state gaps.

[UI system § Components and interaction states](../../../../docs/design/ui-system.md#components-and-interaction-states)
owns the requirement: document anatomy, variants and states once; cover idle, hover where
relevant, focused, pressed, selected, disabled with reason, loading, error, empty and success;
give every blocked action a discoverable reason. This reference says how to write and test those
matrices. **The meaning of a state belongs to its domain owner** — playback to
[`cantos-listening`](../../cantos-listening/SKILL.md), runs and QC to
[`cantos-production-pipeline`](../../cantos-production-pipeline/SKILL.md), approvals and release
to [`cantos-publication`](../../cantos-publication/SKILL.md), dialogue identity to
[`cantos-script-ir`](../../cantos-script-ir/SKILL.md) — and the UI only maps what they decide
([`UI-CORE`](../SKILL.md#rules)). No component exists yet; every type, key and component name
below is **proposed**.

## 1. The matrix is the contract

Write the matrix before the component. Rows are states; columns are what each state must
guarantee. A component without a row for a state it can reach has an undefined state.

| State | Visual (semantic roles only) | Accessible name / state | Input | Required text |
|---|---|---|---|---|
| idle | base role pair | name = action + item | accepts | — |
| hover (pointer only) | state layer, no layout shift | unchanged | accepts | — |
| focused | `color.focus` indicator, unobscured | unchanged | accepts | — |
| pressed | pressed state layer; no movement under the finger | unchanged | accepts | — |
| selected / current | role pair plus a non-color cue (check, weight, indicator) | selected or `current` exposed | accepts | — |
| busy | stable size; progress indicator; label kept | busy exposed; one polite announcement | blocked or queued, stated | status text |
| unavailable | readable, visibly inert | unavailable exposed | focusable per renderer technique | **reason** beside the control |
| error | `color.error` pair, icon and text | error associated with the control | recover offered | cause and next step |
| empty | explanation plus an action | heading or text, not a blank region | the action | why it is empty |
| success | brief, non-blocking confirmation | polite announcement | accepts | what succeeded |
| stale / offline | marker with source identity | state in the name or description | refresh or retry | what is out of date |

- **One loudness per screen.** A matrix may not give two equally loud primary roles to the same
  surface ([`material3-expressive.md`](material3-expressive.md)).
- **Tokens only.** Cells refer to semantic roles; literals and reference tones are findings
  ([`tokens.md`](tokens.md)).
- **Both renderers.** Web and CMP implement the same matrix; a state present in one and absent in
  the other is a divergence to fix or record.
- **Test it.** A table test enumerates every state of the presentation mapping; a DOM or semantics
  test asserts name, state and reason for each reachable row; captures show the rows that carry
  visual risk ([`visual-review.md`](visual-review.md)).

## 2. Availability is a type

A control that may be blocked receives its availability as a value, decided by the backend or a
pure core. The UI never recomputes the decision.

```rust
// Illustrative and proposed. The reason type is owned by the domain module that decides it.
pub enum Availability<Reason> {
    Available,
    Unavailable(Reason),           // a closed enum with data, e.g. the publication blockers
}

// Pure presentation mapping: exhaustive, no wildcard arm, one message key per variant.
pub fn publish_unavailable_message(reason: &PublishBlocker) -> Message;
```

```kotlin
// Illustrative and proposed (commonMain). `when` over a sealed type is exhaustive.
sealed interface Availability<out R> {
    data object Available : Availability<Nothing>
    data class Unavailable<R>(val reason: R) : Availability<R>
}
```

- **A reason is a code plus parameters** supplied by the server or core, never prose invented in
  the UI ([`localization.md`](localization.md#5-errors-are-typed-then-mapped)).
- **All reasons at once.** The publication gate returns every blocker; show the list, not the first
  one ([`approvals-and-gates.md`](../../cantos-publication/references/approvals-and-gates.md)).
- **The reason is visible and associated.** Text beside the control, programmatically linked, with
  a link to the thing to fix ([`accessibility.md`](accessibility.md#3-disabled-with-a-reason));
  never only a tooltip.
- **Client availability is a hint.** The server re-runs the decision on the request. A hidden or
  disabled button is not an authorization check.
- **Oracle:** a presentation table test with one row per variant (the compile error when a variant
  is added is the first oracle), plus a DOM or semantics test of the association.

## 3. Status is icon, text and tone

Status never relies on color alone.

```rust
// Illustrative and proposed.
pub struct StatusPresentation {
    pub icon: IconId,
    pub label: MessageKey,        // visible text
    pub tone: Tone,               // Neutral | Info | Success | Warning | Error — semantic roles
    pub announce: Announce,       // None | Polite | Assertive
}
```

- Every state a person must tell apart has a distinct **icon and text**; tone reinforces.
- Test the mapping with a table; capture it in grayscale and with a simulated color-vision
  deficiency when the status carries meaning ([`visual-review.md`](visual-review.md)).
- `Assertive` announcements are for blocking failures that need immediate attention; routine
  progress is `Polite` and throttled.

## 4. Component matrices

Domain meaning is linked, not restated. Each table lists what the UI must distinguish.

### Buttons and toggles

Variants: filled (one per screen), tonal, outlined, text, icon button, toggle. A **busy** button
keeps its size and label and exposes busy; it never swaps its label for a spinner alone. A toggle
that keeps its label exposes pressed or selected; a control that swaps its label
(`Phát` ↔ `Tạm dừng`) does not also expose pressed
([`accessibility.md`](accessibility.md#2-accessible-names-action--item--state)). Command labels
name the object: "Generate selected dialogue", "Publish episode" — not "Submit".

### Chips, filters, navigation

Selected state has a non-color cue. Navigation exposes the current location (`aria-current` on the
Web, selected on CMP), and a narrow layout keeps a reliable return path
([UI system](../../../../docs/design/ui-system.md#studio-text-first-production)). Pills suit
filters, speed and compact actions; a data table keeps rectangular geometry.

### Text fields and the script editor

| State | Rule |
|---|---|
| draft | a visible "unsaved" indicator derived from the durable revision, not from a local flag |
| saving | busy, input preserved, no caret reset |
| saved | shown only after the server confirmed the new revision (identify it) |
| conflict | stale base revision: show both versions and choose; never overwrite silently |
| invalid | error text associated with the field, the first invalid field focusable from an error summary |
| IME composing | no validation, normalization or autosave mid-composition |

No card per utterance; keep status outside volatile editor text
([UI system](../../../../docs/design/ui-system.md#components-and-interaction-states),
[`script-editor.md`](../../cantos-leptos-web/references/script-editor.md)).

### Episode row (Theatre)

Combine listening progress, download state and availability into one row state with a
documented precedence; do not stack five badges.

| Fact | Presentation |
|---|---|
| not started | no progress mark |
| in progress | elapsed and total as text, plus a progress indicator |
| played | text or icon, not color alone |
| downloading / downloaded | download state text and icon ([§ download state](#download-state)) |
| unavailable | reason text (withdrawn, region, access expired) |
| failed | cause and a retry |

Long titles wrap; a missing cover has a designed placeholder; an empty library explains how to
start. The accessible name carries episode number, title and the dominant state.

### Player controls and the mini-player

The label and announcement derive from `PlaybackStatus` ([`playback-semantics.md`](../../cantos-listening/references/playback-semantics.md)):

| Status | Primary control label | Announcement |
|---|---|---|
| `Idle` | `Phát` | — |
| `Loading` / `Buffering` | `Tạm dừng` is **not** offered until the engine runs; show busy | polite "Đang tải" |
| `Playing` | `Tạm dừng` | — |
| `Paused(cause)` | `Phát`; the cause is shown for sleep timer, interruption, route lost | polite, once |
| `Seeking` | unchanged; position updates | — |
| `Ended` | `Phát lại` / restart is an explicit action | polite |
| `Failed(reason)` | recoverable reasons offer retry; final ones explain and stop | assertive for blocking |

The label says `Paused` as soon as the listener asks but never `Playing` before the engine
reports it. The mini-player appears only with an active session, never covers the focused control
and keeps the entry to the full player. Seeking is operable without the waveform; elapsed and
total time are text.

### Job status (Studio)

Maps `RunStatus` and its pause or failure data
([`durable-jobs.md`](../../cantos-production-pipeline/references/durable-jobs.md#2-three-state-machines-one-meaning-each)).

| Run status | Presentation |
|---|---|
| `Queued` / `Running` | stage, affected revision, progress; estimate or an explicit "unavailable" |
| `WaitingReview` | "waiting for review", **not** approved |
| `Paused(reason)` | the reason with numbers (budget needed vs available, uncertain attempt) and the resume or resolve action |
| `Failed` | the cause in user terms and the recovery action |
| `Cancelled` / `Completed` | terminal; a replacement run is the next step; `Completed` is not "published" |

Costs show estimate, reservation, usage and settlement as different things; an unknown estimate
reads "unavailable", never zero. Production continues without an open browser, so status is
loaded from the server, never inferred from a closed tab.

### QC finding and approval

| Item | Presentation |
|---|---|
| finding severity | `Blocking` and `Advisory` differ by icon and text; blocking ones are listed with the artifact they concern |
| approval current | who, scope, and the revision it covers |
| approval stale | `Stale { approved, current }`: show both revisions and why; publication stays blocked |
| approval missing | the required approval and who may give it |

A completed run, a passed technical check and an approval are three different things on screen.

### Download state

Map the platform-neutral states ([`downloads-and-local-state.md`](../../cantos-cmp-mobile/references/downloads-and-local-state.md)):
`NotDownloaded`, `Authorizing`, `Queued(waitingFor)`, `Transferring(received/expected)`,
`Verifying`, `Available`, `Failed(reason)`. Each has text and an icon; `Transferring` shows
percent and offers pause or cancel; `Failed` states the cause (storage, network, access expired,
integrity) and the retry. After a replacement or withdrawal, say what happens to the saved copy
([business rule 27](../../../../docs/product/business-rules.md#listening-and-mobile)).

### Confirmation dialogs

Name the affected episode and revision, the consequence and whether it is recoverable. The
destructive action is never the default focus; focus is contained and restored; Escape or the
platform back gesture cancels; the primary action is reachable without scrolling at the smallest
supported size.

## 5. Asynchronous states every surface needs

| State | Rule |
|---|---|
| loading | a skeleton or placeholder matching the final layout, so content arriving does not shift controls |
| refreshing | keep the previous content visible with a busy indicator; never blank it |
| empty | an explanation and an action, never a blank region |
| error | what failed, whether work is safe, retry; input preserved |
| offline | a persistent marker and what still works; queued actions are shown as queued |
| stale | out-of-date content carries its source identity and a refresh |
| expired access | a recoverable state with a clear action, distinct from a generic failure |

A late response for an old request never replaces newer state; the renderer skills own that
mechanism ([`leptos-async.md`](../../cantos-leptos-web/references/leptos-async.md)).

## 6. Stale indicators

After an edit, recast or profile change, show which clips, scenes and renders are out of date
**and why**, with the source revision, and show the scope of a regeneration **before** it starts
([UI system](../../../../docs/design/ui-system.md#studio-text-first-production)). The scope list
comes from the pure regeneration plan
([`fingerprints-and-invalidation.md`](../../cantos-production-pipeline/references/fingerprints-and-invalidation.md#4-plan-regeneration-as-a-pure-function)),
not from a UI-side guess. An audio clip without its source revision is a finding.

## 7. Rule cards

| Rule | Failure mode | Good | Counterexample | Oracle | Status · exception |
|---|---|---|---|---|---|
| Availability is a typed value with a reason | a dead control the creator cannot explain | `Unavailable(reason)` rendered beside the control | a greyed button | variant table test; association test | proposed · none |
| The UI never recomputes a domain decision | Web and CMP diverge; a bypassable gate | render the server's `Availability` | comparing revision numbers in a view | review; mapping test | proposed · none |
| One message key per domain variant | a new variant shows a generic error | exhaustive `match`, no wildcard arm | a catch-all "Something went wrong" | compile; table test | proposed · none |
| Status is icon, text and tone | color-blind users cannot read QC state | `StatusPresentation` | a red dot alone | grayscale and CVD captures | proposed · none |
| A state exists on both renderers | the same fact means two things | one matrix, two implementations | CMP missing `Verifying` | matrix comparison | proposed · none |
| Completed, passed and approved are three states | a finished job reads as approved | separate rows and wording | `status = approved` on a run | presentation table | proposed · none |
| Keep input through errors and refresh | lost edits | error with retry, input intact | clearing a form on failure | driven failure flow | proposed · none |
| Estimates are never settled charges | a surprise cost | "estimate" and "unavailable" labels | `0 ₫` for unknown | presentation table | proposed · none |

## 8. Evidence

| Claim | Oracle | Honest label |
|---|---|---|
| every domain variant maps to a presentation | exhaustive match plus table test | `statically-checked`, `example-tested` |
| name, role and state of each reachable row | DOM test (Web); Compose semantics test naming the renderer (CMP) | `DOM-tested`, `semantics-tested` |
| the reason is associated with the blocked control | DOM or semantics assertion of the relationship | `DOM-tested`, `semantics-tested` |
| the states look right and survive grayscale | per-state captures, opened | `screenshot-inspected` |
| the flow works through a real input path | driven interaction | `interaction-tested` |
| a screen reader announces the states | a named TalkBack, VoiceOver or NVDA walkthrough | `screen-reader-walked` only when it happened |
