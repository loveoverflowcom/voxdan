# ADR 0006: Keep AI adaptation proposals separate from accepted script revisions

- Status: proposed; local development implementation under review
- Date: 2026-10-10
- Decision owner: Cantos maintainers
- Related issue: [#3](https://github.com/loveoverflowcom/cantos/issues/3)
- Extends: [ADR 0005](0005-manuscript-import.md)

## Context

The importer preserves exact source bytes, extraction blocks, conversion warnings and creator
rights claims. The existing revision store admits complete validated Script IR exports with
current authorization, optimistic concurrency and operation receipts. AI output is untrusted
editorial content: a successful model response cannot authorize rights, invent reliable speaker
attribution or move a script head.

The initial integration needs one concrete provider boundary and a practical Studio consumer.
Paid hosted inference, production identity, legal adjudication, full scene/character editing,
casting and audio production remain outside this slice. A provider call and a database commit
cannot form one transaction, so a lost response or process crash must leave a visible attempt
whose outcome is unknown rather than silently issue another call.

## Decision

Reuse the modular server, imported source records and immutable revision authority. Persist an
immutable run input before dispatch: source identity/checksum/extractor snapshot, target script
and base revision, current actor's explicit adaptation authorization, provider/model/config,
prompt/adapter versions and an input fingerprint. Import permission claims alone do not enable
dispatch. Missing required source permission evidence or explicit authorization fails closed.
Publication permission remains unknown and is carried into the proposal separately.

The start request also pins the exact provider metadata the creator reviewed. A changed
endpoint, model, prompt version or configuration is rejected before creating a run. An exact
operation replay keeps its original input even after server reconfiguration; a queued worker
cannot substitute a different provider.

Use an opt-in server-side localhost Ollama adapter and a deterministic provider double for
tests. The adapter never accepts a browser-supplied endpoint or credential. Disable redirects,
environment proxies, hosted destinations and automatic transport retries. Bound input/output,
time and concurrency; retain provider-reported usage only when supplied. Unavailable cost data
is unavailable, not a zero charge. The development provider is a concrete adapter rather than
a canned proposal presented as AI. Its deployment selection and live acceptance require an
available, authorized runtime and model; this ADR does not install either.

Loopback alone does not prove local inference: Ollama can forward cloud models, including
locally named aliases, using the daemon's account. Before exposing verified provider metadata,
the adapter must read bounded `/api/status` and `/api/show` responses, require cloud features
disabled, reject remote model fields and identify local GGUF weights. Freeze the verified model
metadata fingerprint with the reviewed configuration and recheck it before sending source text.
Unknown/older daemons without this proof fail closed. This relies on an approved, host-controlled
runtime without concurrent configuration/model replacement; HTTP cannot attest an arbitrary
malicious localhost service. No persistent daemon configuration is changed by this slice.
See [Ollama cloud disablement](https://docs.ollama.com/faq#how-do-i-disable-ollama-cloud-features),
[model details](https://docs.ollama.com/api-reference/show-model-details) and
[upstream status/show handlers](https://github.com/ollama/ollama/blob/main/server/routes.go).

The concrete transport uses pinned `reqwest = 0.12.28` with default features disabled and JSON
enabled. Its dependency on `tower-http ^0.6.8` requires updating the existing pinned
`tower-http 0.6.6` to `0.6.11`; the existing static-file consumer remains in the same crate.
Both package manifests declare Rust 1.64, within the repository's Rust 1.87 toolchain. Official
API references: [reqwest ClientBuilder](https://docs.rs/reqwest/0.12.28/reqwest/struct.ClientBuilder.html),
[tower-http](https://docs.rs/tower-http/0.6.11/tower_http/),
[Ollama chat](https://docs.ollama.com/api/chat) and
[structured outputs](https://docs.ollama.com/capabilities/structured-outputs).
These document adapter behavior; they do not establish an installed model or live integration.

The model returns a closed proposal format with scenes, speaker labels, spoken text, source
block citations, delivery suggestions, typed sound cues and review notes. It cannot supply
authoritative IDs, rights, provenance, asset URLs or approval state. Cantos creates the IDs and
provenance, then admits the complete candidate through the existing Script IR shape and semantic
validator. Unsupported pacing/prosody intent stays in review notes; Script IR 0.1.0 is unchanged.
Uncertain speakers, cited omissions and uncovered blocks require human comparison. Structural
coverage does not prove that meaning was preserved or that attribution is correct.

Persist dispatch intent before network I/O. Keep its terminal observations and valid proposal
private and immutable. Exact command retries reconcile the recorded run. Timeout or abandoned
dispatch is ambiguous, with no automatic repeat; cancellation fences proposal selection but
does not erase an already issued call or its usage. The source and last accepted revision never
change on generation failure.

Studio displays the pinned source and proposal separately, permits bounded JSON correction and
requires an explicit review/accept action. Acceptance uses the existing authorized revision-save
rules and commits its outcome receipt with the revision. A changed head conflicts; an exact
acceptance retry returns the original result. Editorial acceptance grants no production or
publication permission.

## Alternatives

| Option | Benefits | Costs and limitations | Disposition |
| --- | --- | --- | --- |
| Localhost provider plus closed proposal contract | Concrete adapter, bounded synthetic tests, no new credential authority | Live runtime/model availability and output quality still require evidence | Selected for this local slice |
| Hosted commercial adaptation API | Potentially stronger Vietnamese output | Requires explicit destination, credentials and spending authorization | Deferred |
| Model emits authoritative Script IR directly | Smaller conversion layer | Allows fabricated IDs/evidence and couples prompts to trusted metadata | Rejected |
| Automatically save a successful response | Fewer creator actions | Can overwrite competing edits and mistakes inferred content for reviewed content | Rejected |
| New AI service or generalized job platform | Independent deployment | No concrete need beyond this bounded consumer | Deferred |

## Consequences and revisit conditions

Migration 0004 is additive. Earlier migrations and accepted exports stay unchanged. Runtime
privileges permit only the new bounded state transitions and append-only facts; they do not
grant source/revision mutation or credential provisioning. Reads remain indexed, currently
authorized PostgreSQL lookups. No aggregate consumer currently justifies DBSP.

The [evidence record](../evidence/ai-script-adaptation.md) separates deterministic provider
tests, real PostgreSQL evidence, browser observations and live model acceptance. A missing live
gate keeps issue #3 open and its PR in draft. Revisit for production rights review/identity,
hosted inference, multiple workers, larger chapters, model replacement or full editor #4.
No TTS, media asset, publication or mobile behavior is changed.
