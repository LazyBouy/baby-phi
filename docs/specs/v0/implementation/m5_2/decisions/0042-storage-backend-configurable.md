<!-- Last verified: 2026-04-28 by Claude Code -->

# ADR-0042 — Storage backend is configurable; SurrealDB is the v0.1 configured impl; 7-criterion conforming contract

**Status: Accepted**

**Date:** 2026-04-28
**Chunk:** CH-03
**Closes:** D-new-02 (HIGH, terminal closure) — concept-vs-code contradiction where `concepts/coordination.md` claimed "SQLite" while the implementation has used SurrealDB since M1.

---

## Context

The concept doc [`coordination.md`](../../../concepts/coordination.md) §"Design Decisions" has stated since M0 that v0's storage backend is **SQLite**. The implementation has used **SurrealDB** (RocksDB-embedded) since M1. Nine migrations (`0001_initial.surql` through `0009_identity_node.surql`) are SurrealQL; the entire `store/` adapter at [`modules/crates/store/src/repo_impl.rs`](../../../../../../modules/crates/store/src/repo_impl.rs) targets SurrealDB primitives (LET-first RELATE, `type::thing`, FLEXIBLE TYPE object, RELATION edges). The concept doc was never refreshed. **Drift D-new-02** ([`m5_1/drifts/D-new-02.md`](../../m5_1/drifts/D-new-02.md), HIGH, Bucket A) tracked this gap.

CH-03's user-decided scope (Q1, 2026-04-24) expanded the originally-planned 0.5-day rename into a 1-day architectural ratification. The rationale: don't just fix the surface contradiction (SQLite → SurrealDB), also formalize the latent architectural intent — baby-phi was designed with a swappable storage layer (every consumer uses `Arc<dyn domain::Repository>`), but that intent was never written down. Without a written record, future planners would have to re-derive whether the codebase is "locked into SurrealDB" or "configured to use SurrealDB" from inspecting the `store/` crate. This ADR makes the answer explicit and enumerates the contract any alternative backend would need to satisfy.

This ADR pairs with [ADR-0033](0033-k8s-prep-refactors.md) (CH-K8S-PREP). ADR-0033 §D33.2 already framed `SurrealStore::open_remote` as a swappable URI (embedded vs remote SurrealDB server); ADR-0042 generalizes the swap notion further to *any conforming backend*.

---

## Decision

### D42.1 — Storage backend is architecturally configurable

The baby-phi storage layer is configurable at the architecture level. The object-safe Rust trait `domain::Repository` ([`modules/crates/domain/src/repository.rs`](../../../../../../modules/crates/domain/src/repository.rs), ~36 async methods) is the **swap surface**. Domain code, server handlers, and CLI binaries consume only `Arc<dyn Repository>` per the [`baby-phi/CLAUDE.md`](../../../../../../CLAUDE.md) §"phi-core leverage" guidance. No call site outside the `store/` crate references SurrealDB types.

This is a written-down recognition of an architectural property the codebase has had since M1, not a new abstraction added at CH-03.

### D42.2 — v0.1 configured backend is SurrealDB

baby-phi v0.1 ships with SurrealDB as the configured backend:

- **Embedded mode** (default): RocksDB-on-disk via [`SurrealStore::open_embedded`](../../../../../../modules/crates/store/src/lib.rs). Suitable for `phi-server` running on a single host.
- **Remote mode**: SurrealDB ≥ 2.0 server via [`SurrealStore::open_remote`](../../../../../../modules/crates/store/src/lib.rs) per ADR-0033 §D33.2. Suitable for the M7b microservices carve-out where multiple `phi-server` pods share one SurrealDB cluster.

Migrations 0001–0009 are written in SurrealQL and live at [`modules/crates/store/migrations/`](../../../../../../modules/crates/store/migrations/). The store crate is the only place SurrealDB-specific syntax appears.

### D42.3 — Conforming-backend criteria (7 items)

Any candidate alternative backend MUST satisfy ALL seven criteria below to be eligible as a swap target. Each criterion is grounded in a specific code-level dependency the current SurrealDB impl satisfies; relaxing any one would force a change in domain or server code, breaking the trait abstraction.

1. **Transactional semantics.** Atomic `BEGIN/COMMIT` (or equivalent) for compound writes. The audit hash chain, grant issuance, and Auth Request state transitions all assume single-statement-or-rollback semantics. Single-statement-only backends (vanilla file blobs, eventually-consistent KV stores) are not eligible.
2. **Compound-transaction support.** Multi-entity atomic payloads. Concrete callers: `apply_org_creation` (writes Organization + CEO Agent + 2 system Agents + 4 inbox/outbox composites in one tx), `apply_project_creation`, `apply_agent_creation` (CH-16: writes Agent + Inbox + Outbox + Identity in one tx). Partial-failure on any sub-write must roll back the whole.
3. **Typed-endpoint edge semantics.** SurrealDB's `DEFINE TABLE … TYPE RELATION FROM <src> TO <dst>` constrains each edge type to a concrete src/dst node-type pair. The 66-variant `Edge` enum in [`domain::model::edges`](../../../../../../modules/crates/domain/src/model/edges.rs) assumes typed endpoints; e.g., `MemberOf { from: AgentId, to: OrgId }` cannot accidentally connect a `ProjectId` to a `UserId`. An alternative backend must enforce equivalent typed-endpoint constraints (foreign-key check on a relation table, schema-validated source/target columns, etc.).
4. **Schema-free nested-field carrier.** SurrealDB's `FLEXIBLE TYPE object` lets a single column hold the phi-core wraps `Session.inner: phi_core::session::model::Session` and `LoopRecordNode.inner: phi_core::session::model::LoopRecord` without a migration each time phi-core's wrapped shape evolves. Equivalent JSONB columns (Postgres) or untyped-blob columns satisfy this. Strictly-typed wrappers that would require a domain-side migration on every phi-core change are not eligible.
5. **Forward-only idempotent migration runner.** Applied-version tracking via a dedicated ledger table (current impl: `_migrations` table with `(version, slug, applied_at)` tuples). Migrations apply once, in numeric order; the runner is safe to re-run on every pod startup. Confirmed at [`modules/crates/store/src/migrations.rs`](../../../../../../modules/crates/store/src/migrations.rs).
6. **Strict schema declarations.** `SCHEMAFULL` (or equivalent) tables for every load-bearing M1 node — Agent, AgentProfile, Grant, AuthRequest, Template, Consent, User, Organization, Channel, Inbox/Outbox, Memory, ToolAuthorityManifest, plus the M5 + M5.2 additions (Session, LoopRecord, Turn, Identity, AgentCatalogEntry, SystemAgentRuntimeStatus). No quietly-missing required field at write time. Schema-on-read backends (vanilla MongoDB, untyped key-value stores) are not eligible.
7. **UNIQUE index enforcement at the schema layer.** Several invariants ride on this: `bootstrap_credentials_digest` (one platform claim per host); `secrets_vault_slug` (no duplicate vault names per org); `identity_agent_id` (one Identity row per LLM agent per CH-16). Application-layer enforcement is not sufficient — race conditions during compound-tx writes would otherwise produce ghost duplicates.

### D42.4 — Audit hash chain is backend-independent

The BLAKE3 per-org audit hash chain (per [`m1/architecture/audit-events.md`](../../m1/architecture/audit-events.md)) is computed in domain code from canonical bytes of the audit event payload, not by the storage layer. Any conforming backend inherits the chain semantics for free; nothing storage-specific needs to be re-implemented. The audit-event table itself is just another `SCHEMAFULL` row with append-only writes — its tamper-evidence comes from the hash chain, not from the storage engine.

### D42.5 — Out of scope for CH-03 (and v0.1)

The configurability *abstraction* exists (D42.1: `Repository` trait); the *secondary implementation* does not. CH-03 is doc-only — no second backend is onboarded. Concretely out of scope:

- Alternative-backend impl crates (Postgres, DuckDB-PGQ, etc.) — future chunks if/when business need surfaces.
- A configuration switch in `config/<profile>.toml` to select backend at runtime — not needed until a second impl exists.
- Performance benchmarks comparing backend candidates — premature optimization without a concrete reason to swap.
- Migration tooling for moving data from SurrealDB to an alternative — only relevant when a real swap is being executed.

### D42.6 — Pairs with ADR-0033 (CH-K8S-PREP)

ADR-0033 §D33.2 introduced `SurrealStore::open_remote` as the second constructor (alongside the original `open_embedded`), framing remote SurrealDB ≥ 2.0 as a swappable URI for the M7b microservices carve-out. ADR-0042 generalizes that framing: just as embedded vs remote is configurable, so is SurrealDB itself vs any conforming alternative. Both ADRs use the same trait-shape-with-conforming-criteria pattern (ADR-0033 §D33.1 establishes it for `SessionRegistry`; §D33.2 for `SurrealStore`; ADR-0042 for the `Repository` trait at large).

---

## Conforming criteria

(See D42.3 above. Restated here as a quick-reference checklist for any future backend-onboarding chunk.)

- [ ] Transactional semantics (atomic compound writes).
- [ ] Compound-transaction support (multi-entity atomic payloads matching `apply_*_creation` shapes).
- [ ] Typed-endpoint edge semantics (typed `RELATION FROM<src> TO<dst>` or equivalent FK constraint).
- [ ] Schema-free nested-field carrier (FLEXIBLE TYPE object / JSONB / equivalent) for phi-core wraps.
- [ ] Forward-only idempotent migration runner with applied-version ledger.
- [ ] Strict schema declarations for every load-bearing node table.
- [ ] UNIQUE index enforcement at the schema layer for invariant fields.

---

## Alternatives considered

- **Rename only (no ADR; 0.5-day chunk).** Rejected per Q1 decision 2026-04-24 — leaves the architectural intent unwritten; future planners would have to re-derive the configurability question by reading the `store/` crate.
- **Add a config switch + second backend impl now.** Rejected as scope creep — no concrete need for a second backend exists at v0.1; building one now would be premature optimization without a real driver.
- **Frame SurrealDB as locked-in (no configurability commitment).** Rejected — would contradict the existing `Repository` trait's design, which already abstracts over storage. Better to write down the property the codebase already has than pretend it doesn't.
- **Move the criteria into a concept doc only (no ADR).** Rejected — concept docs describe what the system *should* be; ADRs record decisions about implementation paths. The choice to commit to configurability is a decision; it belongs in the ADR archive.

---

## Out of scope

See D42.5. Tracked successors (none required at v0.1 — items below are speculative, only relevant if a real driver appears):

- *Future:* second-backend impl crate (Postgres, DuckDB-PGQ) — would open as a new chunk with this ADR's 7-criterion checklist as the acceptance gate.
- *Future:* runtime backend selection via `config/<profile>.toml` — not needed until the second impl exists.

---

## References

- Concept doc: [`coordination.md`](../../../concepts/coordination.md) §"Storage backend" — refreshed at this chunk.
- Drift: [`D-new-02.md`](../../m5_1/drifts/D-new-02.md) — terminally closed at CH-03 seal.
- Plan archive: [`build/4a52a093-ch-03-storage-backend-configurability.md`](../../../../plan/build/4a52a093-ch-03-storage-backend-configurability.md).
- Architecture doc: [`m1/architecture/storage-and-repository.md`](../../m1/architecture/storage-and-repository.md) — the Repository trait surface.
- Paired ADR: [ADR-0033](0033-k8s-prep-refactors.md) (CH-K8S-PREP) — embedded-vs-remote SurrealDB swap framing this ADR generalizes.
- Code:
  - [`modules/crates/domain/src/repository.rs`](../../../../../../modules/crates/domain/src/repository.rs) — Repository trait (the swap surface).
  - [`modules/crates/store/src/lib.rs`](../../../../../../modules/crates/store/src/lib.rs) — SurrealStore constructors.
  - [`modules/crates/store/src/repo_impl.rs`](../../../../../../modules/crates/store/src/repo_impl.rs) — the SurrealDB impl of Repository.
  - [`modules/crates/store/migrations/`](../../../../../../modules/crates/store/migrations/) — 9 SurrealQL migrations.
