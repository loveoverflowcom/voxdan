# Material 3 Expressive for Cantos

> **Scope.** Decide hierarchy and expressive intensity for any Cantos Studio or Theatre screen on
> Leptos Web or CMP: intent questions, the per-surface intensity model, the Lavender Stage
> identity, dynamic color, the five levers, M3 Expressive shape/type/motion specifics and the
> candidate components. Use before styling or restructuring a screen and when judging one.

The direction is owned by the [UI system](../../../../docs/design/ui-system.md) and the
[brief § Design direction](../../../../docs/product/brief.md#design-direction). This reference is
the method for applying it. It says what "good" means; it does not check that you achieved it —
[`visual-review.md`](visual-review.md) does, on rendered output.

## 1. An attention system, not a rounded-card style

Material 3 Expressive is the 2025 evolution of Material 3. Its claim is that deliberate emphasis —
through **color, shape, size, motion and containment** — helps people find what matters faster
and feels more alive, *provided familiar interaction patterns stay intact*. The work is choosing
what deserves attention, then spending emphasis there and nowhere else.

What it is not: more rounding, more saturation, gradients, bounce on everything or a dependency on
a Material library. A Studio editor that wraps every line in a pill-shaped card has used every
lever and communicated nothing.

Guidance and component availability evolve. Treat the Material site as the design source and the
pinned renderer stack as the implementation truth; § 7 lists what must be verified.

## 2. Intent before styling

Answer these in the working ledger before touching tokens or layout:

1. **Primary goal** — what is the person trying to do here?
2. **Primary action** — the single most important action.
3. **Primary content** — what must be read, heard or manipulated.
4. **Secondary tools** — navigation, filters, metadata, secondary actions.
5. **Tone** — calm, focused, warm, trustworthy, immersive.
6. **Intensity** — the row of § 3 this surface (or region) belongs to.

```text
Screen:           Theatre full player
Primary goal:     keep listening to "Một lời hẹn" (Ánh đèn cuối sân khấu, tập 1)
Primary action:   play / pause
Primary content:  episode and work title, position as elapsed / total text
Secondary tools:  skip back/forward, seek, speed, sleep timer, bookmark, download, episode context
Tone:             warm, immersive, calm under interruption
Intensity:        Medium-High

Screen:           Studio script editor, scene "Sau buổi diễn"
Primary goal:     read and correct dialogue before production
Primary action:   the explicit save of a script version; "Generate selected dialogue" only once a
                  selection exists and the revision is eligible
Primary content:  speaker + dialogue lines with stable identity
Secondary tools:  emotion, pronunciation notes, sound cues, inspector, compact production status
Tone:             calm, focused, accountable
Intensity:        Low
```

If two actions both want to be primary, the screen has two tasks; split the flow or demote one.

## 3. Expressive intensity mapped to Cantos surfaces

Intensity applies per **region**: a Medium-Low mini-player sits on a Medium library; a
Low-Medium production status strip sits inside a Low editor. The loudest region on screen must
be the primary content or action.

| Surface / region | Intensity | Attention goes to | Allowed expression | Not allowed |
|---|---|---|---|---|
| Studio work navigation | Low | current work, adaptation, episode and its version | tonal current-item indicator; compact rail or tree | large headers; decorative icons per node |
| Studio script editor | Low | speaker and dialogue text | quiet toolbar; tonal selection; clear draft/saved state; focus ring | a card or pill per utterance; shape morph in the text area; animated status inside text |
| Studio inspector: casting, voice preview | Medium | selected character and voice, the preview action | selected-voice container tone; shape change on the preview control; button group for options | waveform as the only selection cue; several equally loud preview buttons |
| Studio timeline / preview | Medium-Low | scene order, selected clip, stale clips | selected-clip emphasis; stale badge with text | motion while dragging; shrinking a desktop timeline onto a phone |
| Studio production status, QC, approval, publish | Low-Medium | what runs, what is blocked, which revision | icon + text status; determinate counts; confirmation naming episode and revision | celebratory motion on publish; color-only states; indefinite spinners for long jobs |
| Theatre discovery and library | Medium | cover art, title, availability | cover art carries color; filter chips; emphasized type for a work title | oversized banners; gradients; nested cards; carousels that hide episodes |
| Theatre work / episode detail | Medium | the decision to start or resume | emphasized title as a hero moment; one filled primary (play or resume) | equal-weight download/share/bookmark buttons beside play |
| Theatre full player | Medium-High | transport and position | large primary play/pause; shape morph on state; size hierarchy; expressive spatial springs | playback state hidden by animation; waveform-only seek; continuous decorative motion |
| Theatre mini-player | Medium-Low | what is playing, quick toggle | compact persistent bar; progress hairline; expand affordance | covering the last list row or a focused control; motion while the list scrolls |
| Settings, downloads management, account | Low | the setting and its consequence | standard lists and switches | expressive shapes on routine rows |
| Empty, offline, error states | Low-Medium | the explanation and the recovery action | one clear action; restrained illustration without text | a blank region; untranslated text inside an illustration |

No MVP surface is **High**. A marketing landing page is not in scope; adding one is a product
decision, not a styling choice.

## 4. Lavender Stage identity

*Lavender Stage* is this skill's working label for the documented direction: a soft lavender/
purple brand on calm, readable surfaces — a stage where voices, not chrome, perform.

| Rule | Why | Counterexample | Oracle |
|---|---|---|---|
| `brand.primary` is source evidence; the action role `color.primary` is the tone of that hue that meets contrast in each theme | one swatch cannot be assumed to meet the text floor against both light and dark surfaces | using the logo swatch directly as button fill in dark theme | contrast record per theme ([`tokens.md`](tokens.md)) |
| reading surfaces are low-chroma neutrals (a faint lavender tint is allowed) | long script reading and listening sessions; cover art needs a neutral frame | a saturated lavender editor background | inspected captures, light and dark |
| one primary action hue per screen | five accented things means no primary action | filled primary on Play, Download and Bookmark at once | orientation test ([`visual-review.md`](visual-review.md)) |
| status is never color alone | QC and download states must survive color-vision deficiency and grayscale | a red dot as the only "blocking" signal | state matrix ([`component-states.md`](component-states.md)) |
| dark theme is designed, not inverted | tonal steps collapse and contrast fails differently in dark | container steps indistinguishable in dark | `cross-theme-inspected` |
| chrome does not compete with cover art | Theatre's color comes from the work | tinting the whole library from each cover automatically | inspected library capture with varied covers and a missing cover |

Exact values are open decisions for the first reviewed design implementation
([UI system § Semantic tokens](../../../../docs/design/ui-system.md#semantic-tokens)). Relationships
are stable even while values are not:

```text
brand.primary ─ derives ─▶ color.primary · onPrimary · primaryContainer · onPrimaryContainer   action
neutral family ─ derives ─▶ color.surface ‹ surfaceContainer steps · onSurface · onSurfaceVariant  content
                            color.outline · outlineVariant · focus                                structure
                            color.error · warning · success + their on-roles                      status
```

Material color tooling (HCT tonal palettes, scheme variants such as tonal-spot or vibrant) is a
**candidate** authoring aid for proposing tones. Its output is an input to a reviewed decision,
never the token source itself.

## 5. Dynamic color never silently replaces the identity

Android can derive a scheme from the wallpaper; Web and iOS cannot. Cantos ships its own scheme.

| Field | Contract |
|---|---|
| Scope | any use of system dynamic color or content-derived color (cover art) on any target |
| Why / failure | a wallpaper scheme removes the Lavender Stage identity and can break status and focus contrast untested; Android-only behavior splits the brand across platforms |
| Good | Cantos scheme by default; any dynamic option is an explicit, recorded product decision, opt-in, with status and focus roles kept Cantos-defined |
| Counterexample | enabling the platform dynamic scheme because the Material theme supports it |
| Oracle | theme-selection test on Android 12+ with dynamic color available: default render uses Cantos roles; inspected capture |
| Enforcement / exception | manual · none without a product decision recorded in an ADR |

## 6. The five levers in Cantos

| Lever | Use it for | Counterexample | Check |
|---|---|---|---|
| **Color** | primary action and selected state on `color.primary`; tonal hierarchy from surface-container steps; status roles paired with icon and text | borders around every group; an accent on metadata | every color traces to a semantic role; light and dark inspected |
| **Shape** | distinguish entity classes and state: pills for filters, speed and compact actions; rounder, larger primary transport; stable rectangles for scripts and tables | everything a pill, including static labels (fake affordance) | shape tokens only; no repeated radius literals |
| **Size** | make the one primary action unmistakable; real type hierarchy; quiet metadata | an oversized "Generate" in the editor; headline and body 2 px apart | primary visibly heavier; targets meet the documented minimums |
| **Motion** | communicate a state change: press, toggle, expansion, navigation, progress | hover lift on non-clickable surfaces; looping badges | [`motion.md`](motion.md) rules; reduced motion honored |
| **Containment** | group what belongs together; open-to-contained rhythm | card in a card in a card; or no grouping at all | tonal steps and spacing tokens, not borders and shadows |

## 7. M3 Expressive specifics and renderer reality

**Shape library and morphing.** Expressive adds a library of named abstract shapes, extra
corner-size steps and **shape morphing** as a state signal: a button's shape changes on press or
selection. In Cantos, morphing belongs on the full player's primary play/pause control, on
selected toggles (bookmark, a chosen speed) and in the loading indicator — never on text
containers, script rows or data tables, which need stable rectangular reading areas. Compose
exposes polygon morphing through `androidx.graphics.shapes` and Material shape APIs; the Web has
no Material library, so simple corner morphs use CSS radius transitions and polygon morphs need
SVG path interpolation with matching vertex counts. Both are renderer work — see
[`cantos-leptos-web`](../../cantos-leptos-web/references/styling.md) and
[`cantos-cmp-mobile`](../../cantos-cmp-mobile/references/cmp-design-system.md).

**Emphasized typography.** Each baseline style gains an emphasized (heavier) counterpart for hero
moments. Cantos reserves emphasis for the full-player episode title and a work's detail title —
never editor body text, every heading or button labels. See [`typography.md`](typography.md).

**Motion physics.** Springs replace fixed easing curves. **Spatial** springs move position, size,
rotation and shape and may overshoot; **effects** springs change color and opacity and never
overshoot. Each comes in fast, default and slow speeds, inside a **standard** (calm, little
bounce) or **expressive** (more overshoot) motion scheme. Cantos Studio uses standard; the
Theatre full player may use expressive spatial springs for transport. The older Material 3
easing tokens (`standard`, `emphasized`, with accelerate/decelerate variants) are a different
vocabulary — pick one mapping per intent and do not mix the two names in the token source. See
[`motion.md`](motion.md).

**Candidate components.** None of these is confirmed for the pinned stack:

| Component | Candidate Cantos use | Avoid |
|---|---|---|
| button group (standard or connected) | Theatre speed presets; Studio Script/Timeline view switch | as primary navigation |
| split button | Studio "Preview line" with a menu for scene preview | putting Publish or a destructive action in the menu |
| docked or floating toolbar | quiet Studio editor command bar; full-player secondary actions | a floating bar covering script text or the focused line |
| loading indicator | short indeterminate waits: opening an episode, fetching a voice sample | long production jobs (use determinate progress and status text) |
| wavy progress indicator | decorative full-player progress | as the only seek control; animating under reduced motion |
| icon button sizes and toggles | bookmark toggle, skip back/forward | an icon without an accessible name |
| flexible navigation bar / rail | Theatre phone navigation; Studio wide rail | hiding the current version or a reliable return path |
| FAB or FAB menu | none in the MVP: Studio commands are labeled and contextual | a FAB for Publish |
| carousel | only with a product decision | hiding episodes that a list would show |

| Field | Contract |
|---|---|
| Scope | before using any Expressive component, shape, typography or motion API on either renderer |
| Why / failure | androidx introduced the Expressive APIs as experimental (`@ExperimentalMaterial3ExpressiveApi`) in alpha releases; the CMP-published Material 3 artifact can lag, keep them experimental or differ on iOS; Leptos has no official Material 3 library, and the separate Material Web components project must be checked for maintenance status and Expressive coverage |
| Good | name the API, check the pinned artifact on Android **and** iOS (or the browser matrix), wrap it in a Cantos component that consumes semantic tokens, record the choice in an ADR |
| Counterexample | pasting `MaterialExpressiveTheme`, `ButtonGroup`, `SplitButtonLayout`, `LoadingIndicator` or `MaterialShapes` from an article that targets a newer androidx alpha |
| Oracle | compiles and renders on every target; inspected capture of the wrapped component |
| Enforcement / exception | manual · if unavailable, build the behavior from stable primitives and tokens; Expressive intent does not require a Material dependency |

## 8. Familiar patterns stay familiar

- Lists stay lists. Peer episode rows have equal weight; script lines are stable rows, never a
  bento grid.
- Buttons look like buttons, fields like fields, sliders like sliders. Primary actions carry text,
  not an ambiguous icon alone.
- Reading order is predictable at every width. Narrow Studio screens show the selected scene
  rather than a shrunken desktop timeline.
- Platform conventions win where they differ: back gestures, safe areas and media controls on
  Android and iOS; links for navigation and buttons for actions on the Web.

## 9. Counterfeit expression

| Looks expressive | Actually | Fix |
|---|---|---|
| gradient hero banner on discovery | competes with cover art; contradicts the brief | remove it; let cover art carry color |
| every group rounded and carded | no hierarchy; card-in-card depth nobody can act on | tonal steps, spacing tokens, fewer containers |
| pill-shaped metadata labels | fake affordance | plain metadata text; pills only for interactive chips |
| hover lift on a static surface | fake clickability | remove; reserve press/hover feedback for real controls |
| bouncing or looping status badges in Studio | distraction during reading and review | effects-only change plus text; no loops |
| shadows to rank importance | the brief asks for restrained shadows | tonal surfaces; elevation only for real overlap |
| oversized buttons in the editor | wrong intensity for a Low surface | standard size; size emphasis is the player's budget |

Judge the result with [`visual-review.md`](visual-review.md); every claim above is about rendered
output, not stylesheets.
