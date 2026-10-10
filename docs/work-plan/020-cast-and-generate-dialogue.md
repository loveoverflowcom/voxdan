# Cast characters and generate dialogue durably

## Why

Audio drama needs consistent character voices and predictable production costs. A provider timeout or worker crash must not lose progress, generate uncontrolled duplicate work, or replace audio for the wrong script version.

## Scope

Add Studio casting and freeze the selected voice, versioned performance/pronunciation settings,
model and supported controls against one accepted script revision. Production authorization
binds that exact input scope, applicable rights records and budget policy; draft edits cannot
change an existing snapshot. A frozen candidate and eligibility for billable work are separate
facts. The [production input contract](../../contracts/production-v1.md) and
[casting evidence](../evidence/casting-production-inputs.md) describe the implemented and
locally verified #5 slice, including real PostgreSQL/HTTP recovery and observed Studio
freeze/approval/stale-input behavior. GitHub owns acceptance and execution status.

The user controls Gemini / ChatGPT / Codex and requests tool calls; Cantos owns the data pipeline.
The former local/open-source generation prerequisite is superseded, not passed. Do not install
Ollama/models or run local inference. Provider adapters and real generation belong to #6 and later
work after explicit provider, rights and spend authorization. #5 supplies only settings, rights,
budget and immutable authorization inputs: no TTS call, audio preview, reservation, charge,
worker or provider credential is implied by its acceptance.

Later slices use PostgreSQL-backed background job states, bounded retries, recovery after
interruption, cancellation and per-dialogue result caching. Reuse results only when all
synthesis-affecting inputs match. Account for production cost and make uncertain provider
outcomes visible. A reference capability fixture is not an available voice or provider.

Regenerate only changed dialogue or dialogue affected by casting/pronunciation changes. Record affected scenes for the subsequent mixing stage while preserving successful unchanged clips and previous versions.

## Non-goals

Final episode mastering/publication, a distributed job platform, automatic provider marketplaces, voice cloning, or automated expressive-quality scoring.

## Dependencies

[010 — Import and edit a versioned script](010-import-and-edit-script.md). Choose concrete job, provider, and cache structures after inspecting the persisted Script IR and immediate Studio needs. Verify actual provider terms, supported languages, configuration, and availability before selecting defaults.

## Suggested sequence

1. Establish casting and generation inputs for one validated episode version.
2. Integrate explicitly authorized provider adapters with useful errors and cost records; select
   a real provider and test budget before any paid call. No local inference path is required.
3. Implement durable execution, restart recovery, and stable result association.
4. Add casting controls, queue progress, clip preview, and partial regeneration to Studio.

## Acceptance criteria

- A character retains the selected voice and pronunciation across dialogue; deliberately changed settings produce new generation inputs.
- Each selected, explicitly authorized provider path generates playable audio for a real sample
  in its owning adapter slice. Demonstrate mocked failure cases separately from live synthesis
  evidence; #5 reference capabilities do not satisfy this later gate.
- Repeating a completed identical request reuses audio; changing one dialogue reuses other valid clips and marks the affected scene for remixing.
- Worker restart recovers incomplete jobs. Timeout, rate limit, malformed audio, permanent failure, and cancellation produce bounded, understandable states.
- Concurrent requests cannot incorrectly attach a result to another dialogue or revision. Recovered workers do not overwrite a newer successful result.
- An ambiguous commercial-provider timeout does not automatically trigger unlimited rebilling; retry policy and any provider idempotency limitations are explicit.
- A configured production budget prevents unauthorized new billable work; usage, estimates, and uncertain charges remain distinguishable.
- The creator can fix a failed input/provider choice and resume the remaining work without regenerating successful unaffected clips.

## Review boundary

For #5, demonstrate accepted revision → versioned casting/settings → rights/budget findings →
immutable frozen candidate → exact owner approval → reopened snapshot and current eligibility.
Verify unsupported controls, every scoped-change invalidation, rights expiry/revocation, actor
isolation, concurrent freeze, exact retry and restart recovery on real PostgreSQL. Rights claims
are recorded assertions, not legal verification; missing/pending claims block when required.
Unknown cost is neither zero nor unlimited. Estimates, reservations and actual charges are
distinct; only a provider receipt can establish actual billing.

The complete 020 boundary later demonstrates casting → generation → clip preview → single-line
edit → partial regeneration. Verify real provider adapters, persistent states, cost visibility,
and crash/retry behavior in those owning slices. Review the narrow boundary needed by generation
rather than speculative provider abstractions. Product approval never authorizes an agent to
make a real payment or provider call.

## Risks / unknowns

Provider idempotency, billing, language support and voice stability differ. Reference settings
provide no evidence of provider availability, voice identity or audio quality. Emotion hints
may not map directly across providers; expose actual supported controls and preserve their
versioned meaning. Real-person voices/cloning require specific recorded consent and rights.

## Follow-ups

Defer provider auto-routing, expanded catalogs and batching for throughput. A provider mismatch
or duplicate-result association is a correctness issue required before mixing. Recommend #6
only after #5's own acceptance and review; this plan does not start it automatically.

## GitHub execution

[#5](https://github.com/loveoverflowcom/cantos/issues/5), [#6](https://github.com/loveoverflowcom/cantos/issues/6), [#7](https://github.com/loveoverflowcom/cantos/issues/7), [#8](https://github.com/loveoverflowcom/cantos/issues/8). See the [issue/dependency map](project-planning.md) and [Kanban](https://github.com/users/loveoverflowcom/projects/6/views/1) for the recommended sequence and live execution state.
