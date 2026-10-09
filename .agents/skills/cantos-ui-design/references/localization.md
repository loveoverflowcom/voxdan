# Localization

> **Scope.** Make every visible and accessibility string in Studio and Theatre a localized
> resource on both renderers: the locale-set and framework decisions, the difference between UI
> language and content language, semantic keys, parameters instead of concatenation, plural and
> number/date/duration/currency formatting, typed error presentation, layout under text expansion,
> key and placeholder parity, and the checks that make a missing string visible before a user
> sees it. Use whenever a change adds or edits copy, a label, an error, a status name, an
> accessibility name, a date or a number shown to a person.

Product text is not defined by the UI documents; the locale set is **not decided**. The
[brief](../../../../docs/product/brief.md) and the example episode are Vietnamese
(`Ánh đèn cuối sân khấu`, `Người dẫn chuyện`), the inspection scenarios plan `vi-VN` and `en`, and
[UI system § Accessibility](../../../../docs/design/ui-system.md#accessibility-and-motion-acceptance)
requires labels that carry action, item and state. No localization framework, resource file or key
exists. Every framework, path and key below is **proposed** or a **candidate**; discover the real
resources before citing one, and never add a second mechanism beside an existing one.

## 1. Decisions to record before the first string

| Decision | Options to weigh | Constraint |
|---|---|---|
| locale set | `vi-VN` and `en` as the planned pair; others later | the base locale is the structural source of truth; every required locale is complete before a UI change merges |
| default and fallback | default `vi-VN` for Theatre; Studio follows the creator's saved choice, then the browser or OS | a missing key is a visible build or test failure, never a silent English fallback in a view |
| framework, Web | Fluent-style messages, a Leptos i18n crate, ICU4X-based formatting (all candidates) | must support named parameters, CLDR plural rules and locale-aware formatting; verify the version |
| framework, CMP | Compose Multiplatform resources (`values-vi`, plurals), or a shared catalog | must run on Android and iOS; typed access preferred over string lookup |
| one catalog or two | one source catalog with generated Web and Kotlin consumers, or two resource sets with a parity check | the same semantic key must mean the same thing on both renderers |
| storage and switching | per-account preference, applied before first paint | switching never reloads unsaved Studio edits |

Record the outcome with [`templates/adr.md`](../../../../templates/adr.md) or the design issue. A
decision made implicitly in a component is a defect, as for tokens
([`tokens.md`](tokens.md)). Prefer **one source catalog and generated consumers** when the
decision allows it: it is the same idea as the token source and removes a class of drift. If two
resource sets are chosen, add a parity check (§ 8).

## 2. UI language and content language

Two languages are on screen at once, and they are not the same thing.

| | UI language | Content language |
|---|---|---|
| what it is | buttons, labels, errors, statuses, accessibility names | work and episode titles, synopsis, character names, dialogue text, script notes |
| owned by | the localization resources | the creator's script data ([`cantos-script-ir`](../../cantos-script-ir/SKILL.md)) |
| changes with the UI locale | yes | **never** |
| tagged for assistive technology | the page or app locale | its own language tag where it differs, e.g. `vi` inside an `en` label |

- **Never translate or rewrite content to fit the UI locale.** An English UI shows
  `Một lời hẹn` as written. Never store a UI string in content either: a new adaptation's
  narrator display name comes from the **adaptation's language**, not from whichever locale the
  creator's UI happens to be in.
- **Do not normalize content for display** beyond what the Script IR already guarantees (NFC text);
  the display layer renders what it is given
  ([`vietnamese-text.md`](../../cantos-script-ir/references/vietnamese-text.md)).
- **Tag mixed-language runs.** On the Web set `lang` on the content element; on CMP use the
  renderer's locale annotation for the span when available. Without it, a screen reader reads
  Vietnamese with the wrong voice and mispronounces tone marks
  ([`typography.md`](typography.md), [`accessibility.md`](accessibility.md#2-accessible-names-action--item--state)).
- **Sorting and search follow the content language**, with diacritics handled deliberately; this
  is a product decision to record, not a default of the sort function.

## 3. Semantic keys

Keys name a meaning, not the English copy. Group by surface, then by role.

| Good | Counterexample | Why |
|---|---|---|
| `studio.script.save.action` | `Save` | the text can change without breaking references |
| `theatre.player.bookmark.add.name` | `bookmark_button_1` | role and place are readable |
| `publish.blocker.qc_blocking_finding` | `error_17` | one key per domain variant, so the mapping is checkable |
| `common.cancel` reused for every "Cancel" | five `*.cancel` keys with one meaning | reuse the key only when the **meaning** is identical, not merely the English word |

- A key is stable. Rename by adding the new key, migrating references and retiring the old one in
  one change; never repurpose a key for a different meaning.
- One key per domain variant for any state shown to a person: each `BlockReason`, `FailureReason`,
  `PauseCause`, QC finding and download failure has its own message
  ([`component-states.md`](component-states.md#2-availability-is-a-type)). A shared catch-all
  hides new variants.
- Accessibility names are strings: `...name` and `...description` keys sit beside the visible label
  and follow § 4 like any other copy.
- Developer-only text (logs, assertions, telemetry names, serialization fields, CSS classes, test
  IDs) is not localized and must not reach the UI.

## 4. Parameters, plurals and formatting

**No concatenation.** Word order differs between languages, so a sentence is one resource with
named parameters.

```text
studio.publish.action.name = "Xuất bản tập {title}, bản sửa đổi {revision}"
studio.publish.action.name = "Publish episode {title}, revision {revision}"
```

Never `t("delete") + " " + name`, and never a fragment that only makes sense after another one.

**Plurals belong in the resource**, using the locale's CLDR rules. Vietnamese has a single plural
category; English has `one` and `other`. A resource for a count therefore has the categories its
locale needs and the code passes only the number.

```text
theatre.library.episode_count   vi: other = "{count} tập"
                                en: one = "{count} episode", other = "{count} episodes"
```

Never `if count == 1` in a view.

**Formatting uses locale-aware formatters, never hand-built strings.**

| Value | Rule |
|---|---|
| position and duration | a single formatter shared by Web and CMP (`12:05`, `1:02:09`); the accessible form is a spoken duration (`12 phút 5 giây`), not the clock text |
| dates and times | locale formatter; show time zone only when it matters; never an ISO string in the UI |
| numbers and percentages | locale grouping and decimal marks; a download percentage is an integer unless the design says otherwise |
| money | integer minor units plus currency from the domain ([cost and budget](../../cantos-production-pipeline/references/cost-and-budget.md#2-money-is-integers-with-a-currency)); format with the currency's own minor-unit count (VND has none) — never a float, never a hard-coded symbol |
| unavailable values | an explicit "unavailable" message, never `0`, `--` as the only signal, or an empty string |
| file sizes | one shared formatter with a stated unit base |

**Markup in messages** (a link or emphasis inside a sentence) uses a named slot the framework
supports, not HTML fragments concatenated around translated text.

## 5. Errors are typed, then mapped

A person never sees `error.to_string()`, a provider response body, a SQL message, a stack trace or
a raw HTTP status. The flow is:

```text
typed error or reason (domain / API contract)
        ↓  pure, exhaustive presentation function
message key + parameters + recovery action + (optional) support reference
        ↓
localized resource
```

- The mapping is a `match` with **no wildcard arm**, so a new error variant fails to compile or to
  pass the table test until it has copy.
- Every message says what happened in the user's terms, whether their work is safe, and what they
  can do next: retry, edit, wait, contact support. A recoverable failure keeps the user's input
  ([UI system](../../../../docs/design/ui-system.md#studio-text-first-production)).
- An unknown error maps to one explicit "unexpected" key that carries a support reference
  (a correlation or operation ID), never to an English string in the view.
- A server-supplied reason (for example a publication blocker) is a **code plus parameters**; the
  client localizes it. Do not render server prose directly: it will not follow the UI locale.
- Announce important errors through the renderer's live-region mechanism without repeatedly
  interrupting screen-reader users ([`accessibility.md`](accessibility.md)).

## 6. Layout under text expansion

Vietnamese and English differ in length per phrase, and either can be longer. Content names can be
arbitrarily long.

- Do not size controls by one locale's label. Allow wrapping for labels and keep actionable text
  visible at 200% zoom and large font scales ([`typography.md`](typography.md)).
- Test with a **pseudo-locale** (proposed): expand by roughly a third, add stacked diacritics and
  bracket the string so truncation and concatenation are visible at a glance. A pseudo-locale is a
  development aid and never ships as a user choice.
- Use realistic Vietnamese in captures: `Người dẫn chuyện`, `Ngày mai, mình có diễn tiếp không?`,
  `Ánh đèn cuối sân khấu`; plus one long title and one long character name per list.
- Never truncate an actionable label to fit. Truncating content with an accessible full name is a
  deliberate design choice recorded per component.

## 7. What must be localized

All text that can reach a person: page and section titles, navigation, buttons, field labels,
placeholders, helper and validation text, dialog copy, toasts, empty and loading messages, tooltips,
menus, filter and sort names, table headers, status and enum labels, notifications, and every
accessibility name, description and alternative text. When in doubt it is visible — localize it.

## 8. Checks

| Check | How | Level |
|---|---|---|
| no string literal in views | a literal scan over view and composable sources, with a reviewed allow-list (proposed) | `statically-checked` |
| key parity | every key exists in every required locale; no orphan keys | `statically-checked` (proposed) |
| placeholder parity | the same parameter names in every locale for one key | `statically-checked` (proposed) |
| plural coverage | each locale supplies the categories it needs | `example-tested` |
| error and reason mapping | table test: every variant maps to a key that exists | `example-tested` |
| `lang` and locale tags | a DOM or semantics test that mixed-language spans are tagged | `DOM-tested`, `semantics-tested` |
| layout | pseudo-locale and long-name captures, **opened** | `screenshot-inspected` |

None of these exists today; they are the oracles to automate when the first UI slice lands, and a
report states which ones actually ran.

## 9. Rule cards

| Rule | Failure mode | Good | Counterexample | Oracle | Status · exception |
|---|---|---|---|---|---|
| Every visible and accessibility string is a resource | untranslatable UI; English leaking into Vietnamese screens | `studio.publish.action` in every locale | `"Publish"` in a view | literal scan | proposed · logs and developer diagnostics |
| Content is never translated or stored as UI | an English UI rewrites a Vietnamese title | the title rendered as stored, tagged `vi` | translating a work title | review; tag test | proposed · none |
| Keys name meaning and are stable | broken references; one key, two meanings | rename by add, migrate, retire | repurposing `common.ok` | key-parity check | proposed · none |
| Parameters, not concatenation | untranslatable word order | `"Xuất bản tập {title}"` | `t("publish") + title` | review; placeholder parity | proposed · none |
| Plurals in resources | wrong forms in a language with different categories | CLDR categories per locale | `if count == 1` in a view | plural coverage test | proposed · none |
| Errors are typed then mapped exhaustively | raw provider or SQL text shown; a new variant unhandled | `match` with no wildcard arm | `error.to_string()` | mapping table test | proposed · none |
| Unavailable is not zero | a missing estimate reads as free | an explicit "unavailable" message | `0 ₫` for an unknown cost | presentation table test | proposed · none |
| Realistic Vietnamese in every capture | diacritic and wrapping defects hidden by ASCII | captures with stacked diacritics and long names | "Lorem ipsum" | opened captures | manual · none |

## 10. Evidence

| Claim | Oracle | Honest label |
|---|---|---|
| no hard-coded visible string | literal scan with a reviewed allow-list | `statically-checked` |
| locales are complete and consistent | key and placeholder parity | `statically-checked` |
| every typed error or reason has copy | exhaustive mapping table | `example-tested` |
| mixed-language content is tagged | DOM or semantics assertion | `DOM-tested` / `semantics-tested` |
| layout survives expansion and diacritics | pseudo-locale and long-name captures, opened | `screenshot-inspected`, `cross-viewport-inspected` |
| the copy reads naturally in Vietnamese | a named native-speaker review | recorded reviewer; not an automated claim |
