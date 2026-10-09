# VoxDan

**Vọng Đài — Theatre of Voices.** An AI-assisted audio drama platform with a creator Studio and a podcast-like Theatre for listeners.

VoxDan produces performances with a cast, narration, atmosphere, music and sound effects. Published audio is rendered once and streamed from stored assets; listening does not call a TTS provider.

## Repository status

This initial repository contains product specifications, architecture decisions, directory guides and project-management templates. The server, Web apps, production workers and mobile player are **not implemented yet**. No runnable Rust or Gradle project is claimed by this bootstrap.

`develop` is the first and default branch. Create short-lived feature branches from it and target pull requests to `develop`.

## Product and stack

| Surface | Purpose | Technology |
| --- | --- | --- |
| VoxDan Studio | Import, adapt, edit, cast, preview, produce and approve episodes | Leptos Web |
| VoxDan Theatre | Discover and listen to published audio drama | Leptos Web; Kotlin Compose Multiplatform on Android/iOS |
| Backend | Domain rules, APIs, production orchestration and publication | Rust, Axum, PostgreSQL |
| Media delivery | Private production assets and approved episode renditions | S3-compatible object storage and CDN |

Start with one modular backend. Worker execution can run separately for resource isolation while sharing the same codebase and PostgreSQL job model.

## Start here

- [Project brief](docs/product/brief.md) and [business rules](docs/product/business-rules.md).
- [Architecture](docs/architecture/overview.md), [Script IR](docs/architecture/script-ir.md) and [production pipeline](docs/product/production-pipeline.md).
- [UI system](docs/design/ui-system.md) and [native mobile requirements](docs/architecture/mobile.md).
- [Ordered MVP work plan](docs/work-plan/README.md): the next implementation step and its review boundary.
- [GitHub planning guide](docs/work-plan/project-planning.md), [Kanban](https://github.com/users/loveoverflowcom/projects/6/views/1) and [Planning table](https://github.com/users/loveoverflowcom/projects/6/views/2): 17 implementation issues across five milestones.
- [Contributor workflow](CONTRIBUTING.md), [agent instructions](AGENTS.md) and [reusable templates](templates/README.md).

## Directory map

| Path | Responsibility at bootstrap |
| --- | --- |
| `apps/server/` | Guide for the future Axum host and production execution |
| `apps/web/` | Guide for Studio and Theatre Web |
| `apps/mobile/` | Guide for the native CMP listening app |
| `contracts/` | Versioning guidance and an illustrative Script IR fixture |
| `docs/` | Product, architecture, design, decisions and the work queue |
| `templates/` | Reusable feature, decision, production and publication records |
| `.github/` | Issue forms, PR template and repository checks |
| `scripts/` | Dependency-free checks for this documentation bootstrap |

## Validate the bootstrap

```sh
python3 scripts/check_repository.py
git diff --check
```

Application build, database migration and audio QA commands will be added with the corresponding implementation. [.env.example](.env.example) records proposed configuration names only; it does not start services.

## MVP completion

The MVP must demonstrate an authorized script imported into Studio, reviewed adaptation and casting, resumable production, partial regeneration, mixed audio passing QC, approved publication, Theatre Web playback and native Android/iOS listening with background controls, offline playback and synchronized progress. See the [work plan](docs/work-plan/README.md) for individual acceptance gates.

The referenced UI image was not attached to this request. Exact visual matching remains pending receipt of that reference. The lavender/purple, compact Material 3 Expressive direction is recorded in the UI specification.

## Licensing and content

A software license has not been selected. Do not infer redistribution permission from repository visibility. Source works, adaptations, voices, music and sound effects have separate rights records; production or publication approval must be tied to their exact revisions.
