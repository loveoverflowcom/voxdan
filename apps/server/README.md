# Server implementation guide

This directory contains the `cantos-server` Cargo package's pure Script IR module and the
`validate-script` CLI, consumed by [the contract corpus](../../contracts/README.md). There is no
Axum host, database, identity provider or production worker yet. One root Cargo workspace keeps
future modules in this package; no domain crate or separate service has been introduced.

Run from the repository root with Rust 1.87:

```sh
cargo run --locked --bin validate-script -- contracts/fixtures/script-ir/0.1.0/accept/two-scenes.json
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

`src/script_ir/wire.rs` owns version dispatch and raw shape decoding. `validation.rs` admits
immutable domain values defined in `model.rs`; `canonical.rs` encodes content and speech without
I/O. The CLI performs only file reads/output after validation. It does not save, authorize or
mark a script production-ready. [Evidence](../../docs/evidence/script-ir-contract.md) separates
these local checks from the remaining application gates.

Continue [#1](https://github.com/loveoverflowcom/cantos/issues/1) with a runnable Axum/PostgreSQL
path, migrations, actor/permission checks, immutable complete revision records, stale-write/retry
handling and a minimal Leptos save/retrieve consumer. Follow the
[architecture](../../docs/architecture/overview.md) and [work queue](../../docs/work-plan/README.md).
Do not generate empty crates for hypothetical future services.

Domain logic stays independent of HTTP, database clients and AI/media providers. Production workers share the domain/application codebase, persist progress in PostgreSQL and write media to object storage. Fake providers belong in tests; never present them as production TTS support.
