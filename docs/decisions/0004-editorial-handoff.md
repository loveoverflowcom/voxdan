# ADR 0004: Preserve editorial handoffs on the Studio revision backend

- Status: proposed; implemented for local review
- Date: 2026-10-10
- Decision owner: Cantos maintainers
- Related issue or PR: [Part of #1](https://github.com/loveoverflowcom/cantos/issues/1)
- Supersedes / superseded by: N/A; extends [ADR 0003](0003-script-revision-persistence.md)

## Context

The original local script-revisions branch and develop implemented different backends. The
user selected develop's Script IR, revision backend and Leptos Studio, with compatible local
features ported onto that implementation. Two incompatible initial migrations or authentication
systems cannot safely share the same application database.

## Decision

Keep develop's exact Script IR 0.1.0, c1/e1 encodings, PostgreSQL adapter, sessions, immutable
save semantics and Studio wire contract. First save already supplies local creation semantics.
Add migration 0002 without changing the checksum or meaning of migration 0001. The migration
runner checks and applies ordered migrations in its existing transaction/advisory lock.

Add immutable owner-scoped preserved UTF-8 sources, linked to the existing evidence registry.
Original bytes are never normalized; reads check their SHA-256 and require a source link from
the currently authorized script's immutable history. Old registry-only evidence stays readable;
it does not become a preserved manuscript retroactively. Corrections create a new source ID.

Add bounded revision history summaries, with current access checks on every page and canonical
integrity checks on every returned revision. The existing pinned revision route opens content.
Add explicit owner editorial reviews, distinct from each save's storage acceptance. A new review
requires the locked current head; retries of an already reviewed revision return the original
review even after an edit. Actor/operation receipts prevent reusing a key for another revision.
Review and receipt commit together. Neither record grants production or publication approval.

Keep the existing development cookie authentication. A trusted operator CLI provisions actors,
issues expiring credentials and revokes them, registers evidence and preserves source text.
Only the DDL/operator role performs these operations. Credential issuance uses 32 bytes from
the OS through pinned `getrandom = 0.3.4`, already in the lockfile. The
[versioned API](https://docs.rs/getrandom/0.3.4/getrandom/fn.fill.html) reports failures including
partial reads; failure never falls back to predictable entropy. Only the SHA-256 verifier is
stored. The credential is emitted once after successful insertion, only by explicit issuance.

## Alternatives

| Option | Benefits | Costs and limitations | Decision |
| --- | --- | --- | --- |
| Additive handoff on develop | Retains Studio and current hardening; one persistence model | New migration and endpoints | Selected by user |
| Replace develop with the local backend | Keeps the old local API | Regresses Studio, evidence ownership and review fixes | Rejected |
| Run both backends | Preserves both wire surfaces | Competing schema, identity and revision authorities | Rejected |

## Consequences

- Product: preserved sources, history inspection and an explicit review fact complement the
  existing Studio save flow. These new operations have no new browser controls in this slice.
- Compatibility: additive API definitions and migration; no changed existing response fields,
  Script IR fixtures, canonical bytes or digests. Existing revisions are never rewritten.
- Operations: grant the app SELECT on new tables and INSERT only on review tables; operator
  credentials stay out of the HTTP host. No provider, media storage or paid execution is added.
- Recovery: retries reconcile receipt identity before staleness; a failed receipt insertion
  rolls back the review. Immutable triggers reject update/delete/truncate of settled handoffs.
- Deferred: production identity/TLS, legal rights adjudication, durable browser recovery,
  source importer/editor UX and stable entity continuity across separate documents. The local
  branch's frozen provenance policy is not imposed on develop's deliberate metadata revisions.

## Validation and revisit conditions

The [handoff evidence](../evidence/editorial-handoff.md) names contract, real-PostgreSQL upgrade,
retry, corruption, access, crash/rollback and credential checks. Revisit at production deployment,
a browser handoff consumer, measured list-read contention, or a changed review/publication policy.
This ADR does not accept the full issue #1/010 journey or authorize production publication.
