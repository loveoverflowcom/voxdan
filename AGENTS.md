# Agent instructions — VoxDan

## Scope and source of truth

Read [README.md](README.md), the relevant product/architecture docs and [docs/work-plan/README.md](docs/work-plan/README.md) before implementing a feature. This is a documentation and template bootstrap; inspect actual code before treating a proposed module, schema, command or API as implemented.

The current user's instructions take priority. Preserve VoxDan Studio and VoxDan Theatre terminology, Rust/Axum/PostgreSQL/Leptos/CMP choices and native mobile playback. Keep docs in English unless the task requests otherwise; retain correct Vietnamese names and representative content.

## Architecture boundaries

- Begin with a modular monolith; add a service, crate or generic abstraction only for a concrete consumer and reviewed need.
- Domain rules and versioned Script IR must not depend on HTTP, UI frameworks, a specific AI provider or Narrative Forge internals.
- Use PostgreSQL for metadata, revision history and durable job state. Store media in object storage.
- Do not invoke AI during listener playback. Preserve immutable publication manifests and cached renditions.
- Keep private drafts, voice assets and previews inaccessible to public delivery. Enforce permissions in the backend.
- Mobile uses CMP with platform-native audio adapters; no WebView audio player.

## Correctness rules

For production changes, reason about duplicate delivery, worker crashes, cancellation, provider timeouts, expired job leases, stale revisions, cost reservations and approval invalidation. Never claim exactly-once external TTS generation unless a provider contract proves it; record ambiguous attempts before retrying.

Cache keys must cover all effective generation inputs. A dialogue edit invalidates its speech plus dependent mix/QC/publication artifacts; unchanged dialogue can remain reusable. Any generated or imported content requires recorded provenance and applicable rights.

## Working and review

Branch from `develop` and target PRs to `develop`. Keep changes cohesive and avoid unrelated cleanup. Update the work queue after implementation; GitHub issues/PRs own execution status when linked, and the queue owns recommended order and rationale.

Use [templates](templates/README.md) for a concrete proposal or production record when relevant. Clearly distinguish proposed design, implemented behavior and verified behavior. Never commit secrets, real provider credentials, private manuscripts, downloaded media or generated audio to Git.

## Verification

Run `python3 scripts/check_repository.py` and `git diff --check` for this bootstrap. Once code exists, run the documented checks appropriate to the change. Add meaningful crash/retry and publication-gate coverage when implementing those flows. UI changes require observations on relevant viewports/platforms, realistic Vietnamese text, focus/text scaling and accessibility; an image alone does not verify interaction or motion.

Do not report an app build, mobile test, audio quality result or end-to-end production run unless it actually ran. Record limitations and required follow-ups with evidence.
