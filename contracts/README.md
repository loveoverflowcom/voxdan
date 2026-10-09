# Contracts

This directory owns versioned interchange descriptions and representative fixtures. It currently contains only an [illustrative episode](examples/episode-draft.json), described by the [Script IR proposal](../docs/architecture/script-ir.md).

The draft fixture is original sample content, with rights deliberately `pending_review`; it is not publishable or proof of an implemented API. No AI provider, production voice or media asset is selected.

The first implementation must add an authoritative schema, semantic validation, explicit compatibility/version rules and actual HTTP/OpenAPI contracts where endpoints exist. Validate generated Rust/Kotlin client types against those contracts rather than manually maintaining divergent definitions. Generated files must identify their source and generation command.
