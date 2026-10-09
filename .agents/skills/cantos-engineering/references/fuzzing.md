# Fuzzing

> **Scope.** Decide whether a Cantos surface needs coverage-guided fuzzing, then design targets
> for the boundaries that consume bytes or text nobody on the team controls: manuscript import
> (TXT, Markdown, DOCX), AI adaptation output, Narrative Forge exchange files, provider response
> envelopes, audio probes, manifests and cursors on the wire. Covers the gate, target shape,
> corpus and resource bounds, archive and XML hazards, crash minimisation and promotion into the
> ordinary suite, where fuzzing runs, and the report. Use when the interesting failure is a panic,
> a hang or unbounded resource use rather than a wrong answer. Do not use it for pure functions
> over small typed domains, where a property test or an exhaustive loop is a stronger oracle.

Fuzzing answers one question: *given arbitrary input, does this surface panic, hang or consume
unbounded resources?* It does **not** say that correct input is accepted or that output is right.
The label is `fuzz-tested` — **robustness only**
([vocabulary](../SKILL.md#6-never-collapse-evidence-into-the-word-verified)).

Nothing here is implemented: no Cargo workspace, parser, importer or fuzz target exists. Every
path, command and tool is a **proposal** or a **candidate**; discover the real manifests and
existing targets first. Two decisions are not free and need a recorded
[ADR](../../../../templates/adr.md) before any target: `cargo-fuzz` needs a **nightly toolchain**
and a **separate fuzz workspace** excluded from the root one, and the root policy on `unsafe` and
toolchain pinning decides whether that is acceptable.

## 1. Gate: does this surface need a fuzzer?

Both must be true.

1. **The input is untrusted.** It crosses a boundary Cantos does not control: a creator's uploaded
   file, a provider's response, an AI model's output, a peer's exchange file, a network client, a
   device's stored state. "A later phase will make it untrusted" is not yet a reason.
2. **A fuzzer offers a different oracle** than cheaper tools: the failure mode is a crash, a hang
   or an allocation, and the input is *malformed* in ways a typed generator cannot express.

| Cantos surface | Fuzz? | Notes and instead |
|---|---|---|
| DOCX import (a ZIP of XML) | **yes** — the highest-value target | archive and XML hazards in § 4 |
| Markdown and TXT import parsers, line and speaker segmentation | **yes** | seed with Vietnamese, CRLF, huge lines, stray control characters |
| AI adaptation output → raw DTO → validator | **yes**, structure-aware for the validator | the model's text is untrusted; assert invariants on whatever validates ([`cantos-script-ir`](../../cantos-script-ir/references/import-and-adaptation.md)) |
| Narrative Forge exchange file adapter | **yes**, once it parses rather than forwards | |
| provider response envelopes (JSON, error bodies) | **yes** for the classification path | a hostile or truncated body must still map to a typed failure |
| audio probe and decoder wrappers | **yes** | [`audio-mix-and-qc.md`](../../cantos-production-pipeline/references/audio-mix-and-qc.md#8-decoders-are-fuzz-targets) |
| manifest, grant and listener DTO decoding (`TryFrom` boundary) | **yes** for the *validation*, not for `serde` itself | decode → construct → re-encode must be stable |
| opaque cursors, idempotency keys, ID parsers | maybe | a property test against a naive validator also checks acceptance |
| Vietnamese normalization and offset conversion | panics only | correctness is a property and a golden-fixture problem ([`vietnamese-text.md`](../../cantos-script-ir/references/vietnamese-text.md)) |
| fingerprints, canonical encoding, money, backoff | **no** | property, exhaustive loop or bounded model checking |
| reducers over typed command enums | **no** | state-machine property ([`property-and-differential-testing.md`](property-and-differential-testing.md)) |

**Do not fuzz what a property test covers better.** A no-panic result on a function whose real risk
is a wrong answer is a green tick that means nothing.

## 2. Target design

Keep each target thin, deterministic and free of I/O:

```rust
// Illustrative and proposed. `import_markdown` is a placeholder for the real entry point.
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Claim: importing arbitrary bytes never panics and never allocates without bound.
    let Ok(text) = std::str::from_utf8(data) else { return };
    if let Ok(draft) = script_import::import_markdown(text, &ImportLimits::default()) {
        // Stronger than "it did not panic": an accepted draft satisfies its own invariants.
        assert!(draft.speaker_refs_are_consistent());
        assert!(draft.source_ranges_lie_inside(text));
    }
});
```

- **One target per surface.** A target fuzzing three parsers reports coverage nobody can attribute.
- **No filesystem, network, clock or global state.** A nondeterministic target cannot reproduce a
  crash.
- **Assert invariants on accepted values**, never only the absence of a panic: "it parsed and every
  source range lies inside the input" is a real oracle.
- **Debug assertions are on in fuzz builds.** Use them; they turn silent corruption into a crash.
- **Raw bytes for malformed-input targets** (archives, containers); use `arbitrary`-derived
  structure only for a typed API such as a sequence of commands.
- **Say what a clean run means.** If the workspace forbids `unsafe`, panics, hangs and allocation
  blow-ups are the expected findings, not memory-safety bugs; a PR must not read a clean run as a
  memory-safety claim.

## 3. Corpus, dictionary and bounds

- **Seed from artefacts that already exist and are permitted:** the example episode, golden Script
  IR files, original sample manuscripts, small DOCX files **built in code**. Never seed with a
  private manuscript, a provider's audio or real voice samples.
- **Include non-ASCII from the start.** Vietnamese text exercises multi-byte paths; ASCII-only
  corpora never find byte-versus-character offset confusion. Add stacked diacritics, NFC and NFD
  forms of the same word, a BOM, lone combining marks and invalid UTF-8 for byte-level targets.
- **Commit the minimised corpus** (`cargo fuzz cmin`), not the raw one.
- **Dictionaries pay for textual formats:** the real keys and enum values of the Script IR JSON,
  the exchange format and provider envelopes.
- **Set resource bounds explicitly.** "Does not panic" is insufficient if a 200-byte input
  allocates megabytes or spins for seconds:

```bash
# Illustrative and proposed.
cargo fuzz run <target> -- -max_total_time=600 -rss_limit_mb=2048 -max_len=65536 -timeout=10
```

If the surface declares limits (maximum file size, entry count, nesting depth), those caps are what
the fuzzer attacks. Finding the limit nobody wrote is the point.

## 4. Archive and XML hazards (DOCX and any ZIP-based input)

A DOCX is a ZIP of XML parts. The importer must treat it as hostile:

| Hazard | Required behavior |
|---|---|
| decompression bomb | enforce a maximum total uncompressed size and a compression-ratio limit while streaming, not after |
| entry-count and nesting explosion | cap entries, XML depth and element count |
| path traversal in entry names | ignore or reject names with `..`, absolute paths or platform separators; never write entries to disk by name |
| XML entity expansion and external entities | disable DTDs and entity resolution; never fetch a URL or a file from the document |
| embedded objects and macros | never execute or load them; record that they were ignored |
| encoding tricks | handle invalid UTF-8 and BOMs explicitly; report a typed import defect |
| time | a wall-clock budget for the whole import |

These are the properties the fuzz target and the deterministic regression tests assert. Preserve the
original upload and report the defect with source references, as
[import and adaptation](../../cantos-script-ir/references/import-and-adaptation.md) requires.

## 5. What to look for

```text
panic (index, unwrap, slice, arithmetic overflow in debug)
hang or non-termination
pathological allocation from an attacker-chosen length or count
unexpected recursion depth (nested structures, nested JSON, nested XML)
invalid UTF-8 assumptions and byte-versus-char (or grapheme) offset confusion
quadratic or exponential behavior on adversarial input
a validator that accepts a value the constructor rejects (a barrier bypass)
```

## 6. Crash handling is what makes fuzzing durable

1. Minimise (`cargo fuzz tmin`).
2. **Promote the minimised input to the ordinary deterministic suite** as a named regression, not
   only into the corpus. The corpus is not run on every change; the suite is.
3. Fix at the owning boundary (a limit, a typed defect, a constructor), then keep both the
   regression test and the corpus entry.
4. A crash input derived from a private manuscript is sanitized or rebuilt synthetically before it
   is committed.

## 7. Where fuzzing runs

| Where | What | Why |
|---|---|---|
| each change that touches a target | build the targets only | a fuzz target that stops compiling is the commonest way fuzzing dies |
| scheduled or on request | a bounded run per target with the committed corpus | attributable and cheap |
| phase exit | a longer campaign and a corpus refresh | |

The repository has one workflow today; every job above is a proposal. Do not start remote
compute or long campaigns without the user's request
([`local-execution.md`](local-execution.md)), and never leave a scheduled job invoking a target that
does not exist. Prefer a short local run with explicit bounds and record the duration.

## 8. Report

```text
Target:          <name> · surface: <function> · untrusted source: <who controls the input>
Oracle:          no panic | no panic + invariants | differential against <x>
Corpus:          seeded from <n> artefacts, minimised to <m>; non-ASCII: yes | no; built in code: yes | no
Bounds:          -max_len · -rss_limit_mb · -timeout · format limits (size, entries, depth)
Run:             <duration> · <execs/s> · <new coverage> · toolchain and tool versions
Crashes:         <n>; minimised and promoted as <named tests>
Evidence level:  fuzz-tested — robustness only; says nothing about correct output
Not covered:     <inputs, formats and code the target does not reach>
```

## 9. Rule cards

| Rule | Failure mode | Good | Counterexample | Oracle | Status · exception |
|---|---|---|---|---|---|
| Fuzz only untrusted input with a crash-shaped oracle | minutes burned rediscovering nothing | DOCX importer target | fuzzing fingerprint composition | the gate in § 1 | manual · none |
| Assert invariants on accepted values | "it did not panic" certifies nothing | ranges lie inside the input | a target with no assertion | review of the target | manual · none |
| Bound every resource | a tiny input causes a huge allocation | `-rss_limit_mb`, `-timeout`, format caps | an unbounded run | the run command | manual · none |
| Seed with Vietnamese and built-in-code artefacts | offset bugs never exercised; private data committed | NFC and NFD seeds, generated DOCX | a private manuscript in the corpus | corpus review | manual · none |
| Promote minimised crashes into the suite | the fix regresses unseen | named regression test | corpus entry only | the test exists | manual · none |
| Claim robustness only | a clean run read as correctness | `fuzz-tested`, with limits | "the importer is verified" | the report | manual · none |
