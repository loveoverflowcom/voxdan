# Cast characters and generate dialogue durably

## Why

Audio drama needs consistent character voices and predictable production costs. A provider timeout or worker crash must not lose progress, generate uncontrolled duplicate work, or replace audio for the wrong script version.

## Scope

Add Studio casting and per-dialogue audio preview. Save the selected voice, personality/emotion settings, pronunciation profile, and relevant provider/model settings with the generation request so the resulting audio can be traced to its inputs.

Implement pluggable local/open-source and commercial TTS paths, starting with a verified provider from each category. Use PostgreSQL-backed background job states, bounded retries, recovery after interruption, cancellation, and per-dialogue result caching. Reuse results only when all synthesis-affecting inputs match. Account for production cost and make uncertain provider outcomes visible.

Regenerate only changed dialogue or dialogue affected by casting/pronunciation changes. Record affected scenes for the subsequent mixing stage while preserving successful unchanged clips and previous versions.

## Non-goals

Final episode mastering/publication, a distributed job platform, automatic provider marketplaces, voice cloning, or automated expressive-quality scoring.

## Dependencies

[010 — Import and edit a versioned script](010-import-and-edit-script.md). Choose concrete job, provider, and cache structures after inspecting the persisted Script IR and immediate Studio needs. Verify actual provider terms, supported languages, configuration, and availability before selecting defaults.

## Suggested sequence

1. Establish casting and generation inputs for one validated episode version.
2. Integrate local and commercial provider paths with useful provider errors and cost records.
3. Implement durable execution, restart recovery, and stable result association.
4. Add casting controls, queue progress, clip preview, and partial regeneration to Studio.

## Acceptance criteria

- A character retains the selected voice and pronunciation across dialogue; deliberately changed settings produce new generation inputs.
- Each provider path generates playable audio for a real sample. Demonstrate mocked failure cases separately from live synthesis evidence.
- Repeating a completed identical request reuses audio; changing one dialogue reuses other valid clips and marks the affected scene for remixing.
- Worker restart recovers incomplete jobs. Timeout, rate limit, malformed audio, permanent failure, and cancellation produce bounded, understandable states.
- Concurrent requests cannot incorrectly attach a result to another dialogue or revision. Recovered workers do not overwrite a newer successful result.
- An ambiguous commercial-provider timeout does not automatically trigger unlimited rebilling; retry policy and any provider idempotency limitations are explicit.
- A configured production budget prevents unauthorized new billable work; usage, estimates, and uncertain charges remain distinguishable.
- The creator can fix a failed input/provider choice and resume the remaining work without regenerating successful unaffected clips.

## Review boundary

Demonstrate casting → generation → clip preview → single-line edit → partial regeneration. Verify real provider adapters, persistent states, cost visibility, and crash/retry behavior. Review the narrow boundary needed by generation rather than speculative provider abstractions.

## Risks / unknowns

Provider idempotency, billing, language support, and voice stability differ. Local models impose hardware/storage constraints. Emotion hints may not map directly across providers; expose actual supported controls and preserve their versioned meaning.

## Follow-ups

Defer provider auto-routing, expanded catalogs, batching for throughput, and local model tuning. A provider mismatch or duplicate-result association is a correctness issue required before mixing.
