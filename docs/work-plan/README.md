# MVP work queue

Start with [010 — Import and edit a versioned script](010-import-and-edit-script.md). Each item delivers a usable business capability; implementation details must be confirmed against the code created in earlier items.

The repository starts with documentation and templates. The capabilities below are planned, not implemented. Numeric filename prefixes express the current recommended sequence, may be renumbered, and are not permanent task IDs.

| Order | Outcome | Prerequisite |
| --- | --- | --- |
| [010](010-import-and-edit-script.md) | A creator imports a manuscript, reviews an adaptation, and saves an editable Script IR version. | Project brief and architecture decisions |
| [020](020-cast-and-generate-dialogue.md) | A creator assigns voices and generates dialogue through durable, resumable jobs. | Validated scripts from 010 |
| [030](030-mix-review-and-publish.md) | A creator mixes, reviews, uploads, and publishes a complete episode. | Generated dialogue from 020 |
| [040](040-theatre-web-listening.md) | A listener discovers and streams published episodes in Theatre Web. | Published manifests and media from 030 |
| [050](050-cmp-native-listening.md) | A listener uses native Android/iOS playback, downloads, and synchronized progress. | Publication contracts from 030 and listening contracts from 040 |

## How to use this queue

- Read the project brief and architecture documents before starting an item; the brief is authoritative.
- Create or link a GitHub issue when execution begins. Linked GitHub issues are the source of truth for assignment, current status, discussion, and acceptance; these files preserve scope, reasoning, and sequence. Do not maintain a competing status board here.
- Deliver one reviewable outcome at a time. Internal steps may span several PRs, but each PR must identify its review boundary and avoid implying that the whole item is complete.
- Confirm domain and platform assumptions against the actual code, migrations, provider behavior, and runtime before choosing concrete modules, tables, or routes.
- After a completed item, update this queue to make the next task obvious. Record necessary correctness work as a dependency; put optional expansion in [the backlog](backlog/deferred-expansion.md).

## Shared completion evidence

For every item, provide the user journey demonstrated, relevant contract/migration changes, targeted failure-case checks, and known limitations. A successful mock response alone does not demonstrate a real provider, storage, or native playback integration. Label mocked and live evidence separately.

Keep Rust/Axum and PostgreSQL as a modular backend with Leptos Web and CMP native mobile clients. Avoid introducing independent services without a demonstrated operational need. Apply the shared Material 3 Expressive design requirements to each UI slice.
