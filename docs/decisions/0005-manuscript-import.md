# ADR 0005: Preserve bounded manuscript imports on the existing source boundary

- Status: proposed; implemented for local review
- Date: 2026-10-10
- Decision owner: Cantos maintainers
- Related issue: [#2](https://github.com/loveoverflowcom/cantos/issues/2)
- Extends: [ADR 0004](0004-editorial-handoff.md)

## Context

The revision backend accepts complete validated Script IR documents. A plain manuscript can
contain unresolved speakers and uncertain scene cues, so it cannot become a strict Script IR
by inventing a narrator or character identity. The existing `source_records` table preserves
UTF-8 sources, but cannot preserve DOCX or malformed input byte-for-byte.

## Decision

Extend `source_records` additively with original bytes, immutable import metadata and a separate
extraction outcome. Preserve existing source rows and accepted revisions. The local slice stores
bounded source documents on this existing PostgreSQL boundary; audio/media storage remains
object storage. Object storage adoption for larger sources requires a later reviewed boundary
and crash/reconciliation protocol, rather than a second source authority in this PR.

The maximum original is 1 MiB. The JSON transport is bounded before decoding; the actual byte
array has a separate limit. Decoded text and total ZIP expansion are capped at 2 MiB, ZIP entry
count at 256, and compression ratio at 100. Extraction is a pure deterministic transformation.
Format selection is recorded. Content signatures reject ZIP/OLE spoofing; TXT versus Markdown
is an explicit creator choice because those encodings do not have distinct magic bytes.

Use pinned `zip = 4.6.1` with default features disabled and only deflate enabled, and
`quick-xml = 0.38.4` without encoding/network extensions. Their downloaded package manifests
declare Rust 1.82 and 1.56 respectively, compatible with the repository's Rust 1.87. ZIP parts
are read in memory with bounds, never extracted into a filesystem. XML DTDs, tracked changes
and suspected legacy Vietnamese font encodings are rejected. External resources and macros
are never executed or fetched. Unsupported DOCX structure produces explicit conversion notes.
Versioned API references: [zip](https://docs.rs/zip/4.6.1/zip/) and
[quick-xml](https://docs.rs/quick-xml/0.38.4/quick_xml/).

An authenticated creator imports only into their own source scope. Metadata, registry entry,
original bytes, checksum and extraction outcome commit together. Actor-scoped transaction
serialization and a unique operation receipt make an identical retry return the original
identity. Changed bytes or metadata under the same key conflict. A parser failure is an
inspectable saved source outcome; a rejected oversized request creates no record. A request
interrupted before commit leaves no partial registry/source, while a lost response is recovered
by replaying the exact operation.

The UI displays preserved text or original binary download beside ordered extracted blocks,
speaker labels, unresolved assignments, scene suggestions and warnings. A validated structured
import retains canonical Script IR for later explicit revision save; import never moves a
script head or records editorial/publication approval. Rights holder, permission evidence and
claimed usage scope are recorded as supplied, with publication permission explicitly unverified.

## Alternatives and consequences

Reusing `source_records` retains one source/evidence authority and existing immutable triggers.
Converting every paragraph to narrator dialogue would erase uncertainty; creating a second
storage system would split provenance. Both alternatives are rejected for this scope.

Migration 0003 is forward-only and leaves migrations 0001/0002 unchanged. Older hosts can read
legacy text rows, but do not understand new binary imports; deploy the migration and current
host together for the local slice. Runtime INSERT grants are required on source/evidence tables;
the runtime receives no source UPDATE/DELETE grants or new operator credential authority.

## Validation and revisit conditions

The [import evidence](../evidence/manuscript-import.md) records exact parser fixtures, byte/hash
round trips, PostgreSQL rollback/retry/access/restart checks, Studio observations and remaining
gates. Revisit these limits after measured representative manuscripts, production identity,
object storage or a new extractor version. This decision grants no publication eligibility.
