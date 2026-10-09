# Typography and Vietnamese text

> **Scope.** Choose or change type roles, fonts, line heights, truncation, numerals or reading
> width on either renderer, and check that Vietnamese renders without clipping, fallback glyphs or
> lost identity. Use for any change to `type.*` tokens, a font, a text container's size, or a
> screen that shows script text, character names, titles, times or costs.

Requirements are owned by the [UI system](../../../../docs/design/ui-system.md#semantic-tokens)
(Type row) and its [accessibility targets](../../../../docs/design/ui-system.md#accessibility-and-motion-acceptance).
Content normalization and tone-mark policy belong to
[`cantos-script-ir`](../../cantos-script-ir/references/vietnamese-text.md): the UI renders
content exactly as stored and never rewrites it.

## 1. Type roles

The UI system names five roles. Map each once to a renderer style; do not add a role in code
— a new role (a display size, say) is a change to the UI system first.

| Role | Used for | Material 3 baseline candidate | Emphasized variant |
|---|---|---|---|
| `type.headline` | screen titles; work detail title; full-player episode title | a headline size | allowed only for the work detail title and the full-player title |
| `type.title` | section titles, dialog titles, episode titles in rows | a title size | no |
| `type.body` | dialogue text, synopses, explanations | a body size | no |
| `type.label` | buttons, chips, tabs, field labels | a label size | no |
| `type.metadata` | duration, revision, cost, counts, timestamps, speaker notes | a small body or label size, tabular figures | no |

Emphasized styles are a hero-moment budget, not a way to make headings louder. Studio has no hero
moment: its editor, status and publish surfaces use baseline styles only.

## 2. What Vietnamese demands of type

Vietnamese is Latin script with vowel-quality marks (â, ê, ô, ă, ơ, ư), the letter đ and five tone
marks (acute, grave, hook above, tilde, dot below). One vowel can carry two marks: `ỗ` is o +
circumflex + tilde, `ậ` is a + circumflex + dot below, `ữ` is ư + tilde, `ẳ` is ă + hook above.

| Consequence | Failure signature | Check |
|---|---|---|
| marks above capitals rise beyond the cap height (`Ỗ`, `Ẫ`, `Ể`, `Ở`) | tops shaved off in the first line of a tight block or a single-line field | inspect the first line of each role with the capital test strings |
| the dot below descends under the baseline (`Ặ`, `Ộ`, `ỵ`) | dots cut off by a fixed-height row or an ellipsis box | inspect the last line and one-line truncated rows |
| stacked marks need designed glyphs and mark positioning | marks collide or float; a fallback font draws some letters ("ransom note") | render with and without the bundled font available; compare glyph shapes inside one word |
| text may arrive composed (NFC) or decomposed (NFD) | identical words render or compare differently | the boundary normalizes ([`vietnamese-text.md`](../../cantos-script-ir/references/vietnamese-text.md)); the UI test renders both forms identically |
| two tone-placement conventions are in use (`khoá` and `khóa`) | the UI "corrects" a creator's spelling | never transform content for display |

## 3. Font coverage check

Font family is an open decision. Families designed with Vietnamese in mind (for example Be
Vietnam Pro) and broad-coverage families (Noto Sans, Roboto Flex) are **candidates** to evaluate,
not choices.

1. **Glyphs.** Every precomposed Vietnamese letter in each used weight: Latin Extended Additional
   (U+1EA0–U+1EF9), plus `Đ đ` (U+0110/U+0111), `Ơ ơ Ư ư` (U+01A0/U+01A1, U+01AF/U+01B0) and the
   Latin-1 and Extended-A letters Vietnamese uses; combining marks U+0300, U+0301, U+0303, U+0309
   and U+0323 with working mark positioning.
2. **Weights and axes.** Regular, the label weight and the emphasized weight; a variable weight
   axis if emphasized styles depend on it.
3. **Features.** Tabular figures (`tnum`) present and effective.
4. **Delivery.** On the Web, a subset must keep the Vietnamese ranges: font services commonly ship
   a separate Vietnamese `unicode-range` subset, and self-hosting only the Latin subset silently
   falls back for every marked vowel. On CMP, bundle the font so Android and iOS draw the same
   family instead of Roboto on one and San Francisco on the other — or record the decision to use
   system fonts.
5. **License.** Web embedding and app bundling permitted; record it with the decision.
6. **Render.** The test strings in § 9 at every role, light and dark, default and largest text
   size; open the images.

## 4. Line height and clipping

| Field | Contract |
|---|---|
| Scope | every text container on both renderers |
| Why / failure | fixed heights and tight line boxes clip stacked marks and grow worse at 200% zoom and large OS font sizes |
| Good | containers size to content (`min-height`, `heightIn(min = …)`); line height chosen per role by measuring the tallest stacked capital and deepest dot below in the chosen font |
| Counterexample | a 40 px row with `overflow: hidden`; a Compose `Text` with `maxLines = 1` inside `Modifier.height(…)` |
| Oracle | inspected captures with § 9 strings at default and largest text size; a proposed static check for fixed heights on text containers |
| Enforcement / exception | manual · icon-only controls with no text |

Renderer notes to verify, not to assume: Android font-padding defaults and `LineHeightStyle`
trimming in Compose change whether marks above the cap height survive; single-line ellipsis and
line clamping on the Web clip to the line box. Avoid uppercase transforms on Vietnamese labels:
capitals with marks are the tallest glyphs and the hardest to read in caps. Mechanics live in
[`cantos-leptos-web`](../../cantos-leptos-web/references/styling.md) and
[`cantos-cmp-mobile`](../../cantos-cmp-mobile/references/cmp-design-system.md).

## 5. Long names and titles

- **Speaker names are identity.** In the script editor, `Người dẫn chuyện` wraps; it is never
  truncated, because a truncated speaker can be mistaken for another character.
- **Titles wrap before they truncate.** Episode rows allow two lines for the title; detail views
  show it in full. Where a single line is unavoidable (mini-player), ellipsize at the end and keep
  the full title in the accessible name — never a scrolling marquee, which is continuous motion.
- **A long title never pushes the primary action off-screen.** Check at 320 CSS px and the
  narrowest phone width with the long strings below.
- **Line breaking.** Vietnamese separates syllables with spaces, so breaking between syllables is
  natural; do not enable `break-all`-style breaking or hyphenation.
- **Language tagging.** Mark content with its language so line breaking, font selection and
  screen-reader pronunciation are correct ([`localization.md`](localization.md#2-ui-language-and-content-language)).

## 6. Tabular figures

Use tabular figures wherever numbers update in place or are compared in columns: elapsed and total
time, durations in episode lists, estimated and actual cost, job counts (`12/40`), download
percentages and revision numbers in comparisons. Proportional digits make a ticking position
change width every second — the time text then jitters and nudges its neighbors, a small moving
target ([`motion.md`](motion.md#3-rules)). The renderer feature is `font-variant-numeric:
tabular-nums` on the Web and the `tnum` font feature in Compose; both depend on the font shipping
the feature (§ 3).

## 7. Reading measure in the script editor

- The dialogue column has a bounded measure. Start from roughly 60–80 characters per line and
  confirm against real Vietnamese scripts in the chosen font; extra width on wide screens goes to
  the inspector pane, not to longer lines.
- Speaker labels align in their own column or above the line at narrow widths; dialogue wraps
  beneath its speaker; nothing scrolls horizontally at 320 CSS px.
- Emotion, pronunciation notes and sound cues use `type.metadata` in their own slots. Cues are
  typed scene events, never text spliced into dialogue
  ([Script IR](../../../../docs/architecture/script-ir.md#purpose)).
- Line and scene identity stay stable as text reflows; selection and caret survive pane changes
  ([UI system § Studio](../../../../docs/design/ui-system.md#studio-text-first-production)).
  Editor mechanics: [`cantos-leptos-web` script editor](../../cantos-leptos-web/references/script-editor.md).

## 8. Text scaling

Sizes are `rem` on the Web and `sp` on CMP so browser zoom and OS font scale apply. At 200% zoom,
320 CSS px reflow and the largest native sizes, layouts stack rather than overflow, truncation
rises (recheck names), and icons may scale less than text. Android applies nonlinear scaling at
large font sizes and iOS Dynamic Type has accessibility sizes; whether the pinned CMP version maps
iOS Dynamic Type to Compose font scale must be verified on a device or simulator.

## 9. Test strings

Use these as fixtures in every typography, layout and localization check. They are original
sample content.

```text
Speaker:            Người dẫn chuyện
Dialogue:           Ngày mai, mình có diễn tiếp không?
                    Có chứ. Tôi sẽ chờ cậu ở đây.
Narration:          Ánh đèn cuối cùng còn sáng trên sân khấu.
Work / episode:     Ánh đèn cuối sân khấu / Một lời hẹn
Stacked, lowercase: những ngưỡng cửa, bữa tiệc lặng lẽ, quỹ đạo, chuyện cũ
Stacked, capitals:  NGƯỜI DẪN CHUYỆN · Ỗ Ẫ Ậ Ặ Ữ Ự Ỵ Ể Ở Đ
Long title:         Tập 12: Những người gác đèn ở bến cảng cũ và lời hẹn chưa kịp nói
Long name:          Bà Nguyễn Thị Phượng, người giữ chìa khoá nhà hát
Numbers:            1:02:45 / 1:15:00 · 00:59 → 01:00 · 12/40 · 1.234.567
Chrome (en / vi):   Generate selected dialogue / Tạo thoại đã chọn
                    Publish episode / Xuất bản tập
```

Also render every Vietnamese string decomposed to NFD with a script and confirm the output is
pixel-identical to the composed form. A check that used only `An` and `Minh` proves nothing
about diacritics.
