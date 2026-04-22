<!-- Last verified: 2026-04-20 by Claude Code -->

# ADR-0013: Audit events — retention tier by class, per-org hash chain

## Status

Accepted — 2026-04-20 (M1 / P1 seed; full off-site stream + retention
sweep land in M7b).

## Context

Per `docs/specs/v0/requirements/cross-cutting/nfr-observability.md`, every
write-side action in phi emits an audit event. The NFR spec calls for:

- A base event shape (event type, actor, target, timestamp, diff,
  provenance).
- Three retention class tiers: Silent (30 days), Logged (365 days),
  Alerted (365+ days, delivered within 60 s).
- Tamper resistance via a hash chain within an org's scope.
- An off-site append-only stream (deferred to M7b).

We need to decide: does the hash chain start in M1, or do we defer it?
Where does the event log live — one table, one per class, on-disk NDJSON,
or all of the above?

## Decision

1. **One primary table, `audit_events`, one shadow NDJSON file.** Events
   land atomically in both. The DB row supports querying (observability
   dashboards, alerted-event delivery); the NDJSON log lets us recover if
   the DB is corrupted (cheap tamper-evidence baseline before M7b's S3
   object-lock stream).
2. **Class stored as a string ASSERT, not a separate table.** The
   `audit_class` column carries `"silent" | "logged" | "alerted"`, enforced
   by a SurrealDB ASSERT clause. Retention is applied downstream — M7b's
   retention sweep reads the class and decides.
3. **Hash chain starts in M1.** Every event carries a
   `prev_event_hash` — BLAKE3-256 of the previous event within the same
   `org_scope`. The helper `domain::audit::hash_event` computes the digest
   over the event's *canonical bytes*, excluding `prev_event_hash` itself
   (to avoid a circular dependency).
4. **Per-org scope, not global.** The chain is independent per
   organisation. This matches the plan's `{org_scope}` label and means
   tamper-detection can report "org X's chain broke at event N" without
   cross-org false positives.
5. **Emitter as a trait, not a concrete type.** `AuditEmitter` is a trait
   in `domain::audit`. Implementations land in P2 (store-side) and P6
   (server-side wiring). Tests can inject an in-memory emitter that
   records everything for assertions.
6. **Base64 string column, not native bytes.** `prev_event_hash_b64` is
   stored as a base64 string, same rationale as secrets_vault
   (driver-level `Vec<u8>` binding doesn't coerce into SurrealDB `bytes`).

## Consequences

Positive:

- **Chain mechanism live from day one.** M7b's off-site-stream work plugs
  into an existing chain rather than bootstrapping one atop a year of
  un-chained events.
- **Canonical-bytes design is testable.** The hash-exclusion invariant is a
  unit test, not a comment. Future changes that break it fail CI.
- **Class ASSERT catches drift.** A new contributor who types
  `"ALERTED"` instead of `"alerted"` gets a schema error at write time.
- **Single table, single log.** Operational simplicity — no per-class
  partitioning until retention-sweep actually needs it.

Negative:

- **Hash chain adds write cost.** Every emit does a lookup
  (`last_event_hash_for_org`) before the insert. For v0.1 scale (bounded
  by a single org), trivial; at M7b+ scale, we'll benchmark.
- **Shadow NDJSON file is separate state.** If the DB and the file ever
  diverge, reconciliation requires an operator.
- **Base64 is a small size tax.** ~33 % overhead on the hash column vs
  raw bytes. Negligible.

## Alternatives considered

- **Defer the chain to M7b.** Rejected: M7b would have to retrofit hashes
  into months of existing events, or start the chain at a cutover point
  (leaving the pre-cutover events unchained). M1 seeding is cheap.
- **Global chain across all orgs.** Rejected: makes cross-org tamper
  detection noisy and leaks the write rate of one org into another's
  chain. Per-org scope is the right isolation boundary.
- **Separate tables per class.** Rejected: queries would need UNION ALL;
  retention sweeps would mean re-implementing the logic N times.
- **Merkle tree instead of linear chain.** Rejected for v0.1: a linear
  chain is enough for tamper detection at our write volumes; Merkle buys
  query-time proofs we don't need yet.
- **Store hash as native `bytes`.** Rejected because the SurrealDB driver
  bind path doesn't coerce `Vec<u8>` into `bytes`. Workable with
  `sql::Bytes` wrapping, but base64 is portable and debuggable.

## References

- Architecture page: [audit-events.md](../architecture/audit-events.md)
- Implementation: [`modules/crates/domain/src/audit/mod.rs`](../../../../../../modules/crates/domain/src/audit/mod.rs)
- NFR source: `docs/specs/v0/requirements/cross-cutting/nfr-observability.md`
- Plan row: "Audit-log tamper resistance" (M7 + M7b) — M1 seeds the local
  chain; M7b adds the off-site object-locked stream.
