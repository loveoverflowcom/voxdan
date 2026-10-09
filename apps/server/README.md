# Server implementation guide

This directory is a placeholder for the future Rust/Axum backend and production execution. No Cargo package exists yet.

Follow the [architecture](../../docs/architecture/overview.md) and start with the next [work item](../../docs/work-plan/README.md). Establish manifests, a documented local development command, PostgreSQL migrations and a permission-tested first import/edit slice together. Do not generate empty crates for hypothetical future services.

Domain logic stays independent of HTTP, database clients and AI/media providers. Production workers share the domain/application codebase, persist progress in PostgreSQL and write media to object storage. Fake providers belong in tests; never present them as production TTS support.
