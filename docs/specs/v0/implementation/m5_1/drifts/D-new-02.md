<!-- Last verified: 2026-04-24 by Claude Code -->

# D-new-02 — Storage backend is SurrealDB (not SQLite as concept mandates)

## Identification
- **ID**: D-new-02
- **Phase of origin**: concept-audit (M5.1/P2)
- **Discovery source**: `concept-code-audit`
- **Date discovered**: 2026-04-24
- **Status**: `discovered`
- **Bucket**: A — load-bearing scope gap (architectural decision drift)
- **Severity**: HIGH
- **Tags**: `architecture-decision`, `concept-contradiction`, `needs-adr-or-concept-refresh`
- **Blocks**: nothing runtime (SurrealDB works); concept-fidelity drift only
- **Blocked-by**: none

## Concept alignment
- **Concept doc(s)**: [`concepts/coordination.md`](../../../concepts/coordination.md) §"Design Decisions — Storage backend"
- **Concept claim (verbatim)**: *"Storage backend: SQLite (single-file, transactional, migratable). Step up from phi-core's JSON files with minimal operational overhead."*
- **Contradiction**: Baby-phi actually uses **SurrealDB** (RocksDB-embedded). [`modules/crates/store/Cargo.toml`](../../../../../../modules/crates/store/Cargo.toml) declares `surrealdb = { workspace = true }`; migrations are `.surql` files; entire store adapter targets SurrealDB syntax (LET-first RELATE, type::thing, etc.).
- **Classification**: `contradicts-concept`
- **phi-core leverage status**: `N/A — no phi-core overlap`

## Plan vs. reality
- **Plan said** (coordination.md v0 commitment): SQLite as canonical storage.
- **Reality (shipped state at current HEAD)**: SurrealDB embedded (RocksDB backend). Schema migrations 0001–0005 all in SurrealQL. Store adapter uses SurrealDB primitives throughout. This has been the backend since M1 but the concept doc was never refreshed to reflect the actual choice.
- **Root cause**: `concept-doc-not-consulted` OR `architecture decision made without concept-doc update`. Likely the switch was user-approved during M1 planning but no concept-refresh ADR was written.

## Where visible in code
- **File(s)**: [`modules/crates/store/Cargo.toml`](../../../../../../modules/crates/store/Cargo.toml) surrealdb dep; [`modules/crates/store/migrations/*.surql`](../../../../../../modules/crates/store/migrations/) 5 migrations; [`modules/crates/store/src/repo_impl.rs`](../../../../../../modules/crates/store/src/repo_impl.rs) full SurrealDB adapter
- **Test evidence**: All store/acceptance tests run against SurrealDB. The concept-vs-code divergence has no runtime test.
- **Grep for regression**: `grep -n "surrealdb\|sqlite\|rusqlite" modules/crates/store/Cargo.toml` — current state: surrealdb present, sqlite absent.

## Remediation scope (estimate only — updated 2026-04-24 per Q1 decision)
- **Approach (sketch)**: **Decided path — configurability framing.** (a) Refresh `coordination.md` §Storage-backend: replace "SQLite" with "SurrealDB is the currently configured backend" + declare that **storage backend is configurable** (not hardcoded); (b) new ADR documents the decision + records the **criteria a conforming backend must satisfy** to be plug-in-eligible: transactional semantics, compound-tx support, RELATION edge semantics, FLEXIBLE-TYPE-object support for phi-core wraps, migration idempotency, SCHEMAFULL table support, UNIQUE index enforcement. No code change at M5.1 — the actual configurability abstraction (trait-based repo adapter swap) is a separate future chunk if/when a 2nd backend is ever onboarded; M5.1 work is the architectural recognition recorded in concept + ADR.
- **Implementation chunk this belongs to**: CH-03
- **Dependencies on other drifts**: none
- **Estimated effort**: **~1 day** (concept refresh + ADR with configurability criteria — expanded from 0.5 day per Q1 answer scope).
- **Risk to concept alignment if deferred further**: MEDIUM — concept docs are source of truth per the user-stated M5.1 principle; a contradiction here undermines every reader's trust. Must refresh or renegotiate formally.

## Prior documentation locations (pre-M5.1)
- Plan archive lines: (none — not in M5 drift ledger; discovered via concept audit)
- Code comments: none flagging the discrepancy
- ADR references: none

## Lifecycle history
- 2026-04-24 — `discovered` — M5.1/P2 concept-code audit (Agent 1 report)
- 2026-04-24 — `scoped` — M5.1/P3 Q1 decision: assigned to CH-03; scope expanded to include backend-configurability framing + conforming-backend criteria
