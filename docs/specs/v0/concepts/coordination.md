<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-28 by Claude Code (CH-21 amendment: §"event-driven reactivity / runtime-status telemetry" claim that listener fires advance system-agent runtime-status tiles is now fully honored — both call sites for `record_system_agent_fire` are in production (catalog from CH-22, memory-extractor from CH-21). Drift D6.1 terminally remediated.) -->
<!-- CH-03 amendment (2026-04-28): §"Design Decisions" — Storage backend row corrected from "SQLite" to "SurrealDB (RocksDB-embedded; remote ≥ 2.0 supported)"; new dedicated §"Storage backend" subsection added below the table with configurability framing + 7-criterion conforming-backend contract. Drift D-new-02 terminally remediated. See ADR-0042. -->

# Multi-Agent Coordination

> Extracted from brainstorm.md Sections 5 + 6.
> See also: [agent.md](agent.md), [permissions.md](permissions/README.md) (delegation rules), [organization.md](organization.md) (Market)

---

## Future Scenarios (from phi-core roadmap)

These phi-core future scenarios directly feed into phi's coordination design:

### HITL Resume (Human-in-the-Loop)

Agent is aborted mid-execution, human reviews, then resumes. Requires checkpoint/restore on Agent state. phi-core needs `Agent::checkpoint()` / `Agent::restore(checkpoint)`.

**phi implication:** The data model must support partial sessions — loops that are `Aborted` with a resumption path. The graph edge `CONTINUES_FROM` with `ContinuationKind::Rerun` or `Branch` captures this.

### Checkpoint Restore (Cross-Process)

Serialize agent state to storage, load it in a different process. phi-core needs `AgentSnapshot` type.

**phi implication:** The data layer IS the persistence. If all state is in the graph, checkpoint/restore is just "read the graph" / "write the graph". No separate snapshot mechanism needed.

### Parallel Exploration

Multiple branches from the same checkpoint run concurrently. phi-core supports this via `agent_loop_continue(Branch)` with cloned contexts.

**phi implication:** The `Loop` node naturally supports this — multiple Loops share the same `CONTINUES_FROM` parent, each as a sibling branch. `ParallelGroupRecord` (value object on Loop) tracks which branch was selected. The `PARALLEL_WITH` edge connects siblings.

### Auto Origin/Continue Selection

Agent decides whether to `agent_loop` or `agent_loop_continue` based on context state.

**phi implication:** This is the "agent invocation layer" — phi should provide a high-level `send(agent_id, message)` that inspects the agent's current state in the data model and dispatches correctly.

---

## Coordination Patterns

> Not yet designed — placeholders for future brainstorming.

### Shared Data (Blackboard)

Agents coordinate by reading/writing shared nodes in the graph. No direct messaging — just data. Like a blackboard architecture. The Memory node type enables this.

### Event-Driven

Agents subscribe to events on specific nodes. "When Agent A creates a Message with tool_call X, notify Agent B." Built on top of `AgentEvent` streams and the Event node.

### Explicit Messaging

Agents send messages to each other through a dedicated channel. Could use a shared Session or a new `COMMUNICATES_WITH` edge with a message queue.

### Orchestrator Pattern

A supervisor agent that spawns, monitors, and coordinates worker agents. Maps directly to `DELEGATES_TO` edges. The orchestrator has `HOLDS_GRANT` to spawn and monitor.

---

## Design Decisions (v0 defaults — revisitable)

Each decision below is a working hypothesis for v0, not a locked-in commitment. They give implementers a concrete starting point and will be revisited once usage patterns surface.

| Question | v0 default | Why this default |
|----------|-----------|-------------------|
| **Storage backend** | **SurrealDB** (RocksDB-embedded for v0.1; remote SurrealDB ≥ 2.0 server also supported). The architecture is **configurable**, not hardcoded — see the [Storage backend](#storage-backend) subsection below for the full description + 7-criterion conforming-backend contract. | A pragmatic step up from phi-core's JSON files: transactional, schemaful, migratable, with native graph-edge semantics that match the ontology's typed `RELATION` constraints. A dedicated graph-native DB (Neo4j, Memgraph, DuckDB-with-PGQ) remains a v1 conversation once access patterns stabilise; the configurability framing covers any future swap without architectural rework. |
| **Query language** | **Custom tag-predicate DSL** for Grant selectors (already specified throughout the permissions spec). **Cypher-inspired subset** (`MATCH` / `WHERE` / `RETURN`) for graph traversal. Agents see a single `query(...)` API surface; the DSL underneath is an implementation detail. | Tag predicates are the primitive the permissions model is built on; using them for selectors keeps one language. Graph traversal is needed for authority-chain walks and supervision queries; Cypher is the least-surprising syntax. SQL was considered and rejected — it is awkward for transitive edge traversal. |
| **Schema versioning** | **Additive-only in v0.** Adding a property to a node type or a new edge type is a non-breaking change and does not require migration. Removing or renaming a property requires a **migration Auth Request** (Template E shape) that emits the migration plan as provenance and touches all affected instances atomically. | Additive-only is the cheapest path that preserves audit continuity. Any destructive change has to route through the Auth Request mechanism for the same reason ownership transfers do: the authority chain stays traceable. |
| **Event sourcing** | **Hybrid.** State-based current-node view for reads; append-only `AgentEvent` stream for audit and replay. | Matches phi-core's existing event pattern. Full event sourcing (deriving all reads from replay) is slower for common queries and adds operational complexity without enough benefit at v0 scale. The audit stream is separately queryable when an incident requires replay. |
| **Consistency model** | **Last-writer-wins per node** with timestamped optimistic concurrency on writes (retry-on-conflict). **Provenance-carrying edges** (`DESCENDS_FROM`, `EMITTED_BY`, `APPROVED_BY`) are **append-only** — you add a new edge rather than rewriting an existing one. | LWW is simple and acceptable for most agent-coordination workloads where conflicts are rare. Provenance edges can never LWW — an audit trail that silently loses edges is broken. The split gives us cheap writes on state while keeping the authority chain tamper-evident. |
| **Memory types** | The four types from Claude Code's memory model: **`user`, `feedback`, `project`, `reference`**. Adopt as the v0 `memory_type` enum on the Memory node. | These four cover the observed categories in practice (agent preferences, process corrections, project-specific facts, external references). New types can be added non-breakingly (additive-only rule above). |
| **MCP lifecycle** | **Lazy connection on first use**, persistent thereafter for the duration of the owning Session. Disconnect on session end. Re-connection policy is the MCP server's concern. | Eager connection wastes resources when most sessions never touch most tools. Persistent-per-session balances startup cost against connection churn. Tying lifetime to the Session gives a clean cleanup trigger. |
| **Provider testing** | **System-session shape** — tests run under a system agent in a session with no `project:` tag, only `agent:system-tester` and `org:{test_org}`. Sidesteps project-scoped permission checks cleanly. | Avoids the temptation to add a "test mode" that bypasses permissions — which would then become a latent exploit vector. The system-session shape is a normal part of the model, well-specified in [permissions/05-memory-sessions.md § Sessions as a Tagged Resource](permissions/05-memory-sessions.md#sessions-as-a-tagged-resource), so provider tests compose with the same machinery as production sessions. |

> **Revisit triggers.** Each default should be revisited when any of: (a) a concrete failure mode appears in production, (b) the v0 scale assumption is broken (e.g., the embedded RocksDB-backed SurrealDB instance becomes the bottleneck), or (c) a more principled alternative surfaces. Until then, implementers treat these as load-bearing v0 invariants.

---

## Storage backend

baby-phi v0.1 ships with **SurrealDB** as its database. The implementation has used SurrealDB since M1 — embedded mode (RocksDB local file) for `phi-server` running on a single host, and remote mode (SurrealDB ≥ 2.0 server) for the M7b microservices carve-out per [ADR-0033](../implementation/m5_2/decisions/0033-k8s-prep-refactors.md) §D33.2. Migrations 0001–0009 are written in SurrealQL. The store crate at [`modules/crates/store/`](../../../../modules/crates/store/) is the only place SurrealDB-specific syntax appears.

**The architecture is configurable, not hardcoded.** Domain code, server handlers, and CLI binaries never see SurrealDB types directly — they consume an `Arc<dyn domain::Repository>` (an object-safe Rust trait with ~36 async methods covering node CRUD, grants, auth requests, ownership edges, sessions, and audit events). The `Repository` trait is the **swap surface**: a future backend candidate (Postgres, DuckDB-PGQ, etc.) plugs in by providing a parallel impl crate that satisfies the trait. No domain or server code changes are required at the consumer layer.

**Conforming-backend criteria.** Any candidate impl MUST satisfy ALL of the following to be eligible (see [ADR-0042](../implementation/m5_2/decisions/0042-storage-backend-configurable.md) for the formal decision record + rationale):

1. **Transactional semantics.** Atomic `BEGIN/COMMIT` (or equivalent) compound writes. Single-statement-only backends are not eligible.
2. **Compound-transaction support.** Multi-entity atomic payloads — the existing `apply_org_creation`, `apply_project_creation`, and `apply_agent_creation` handlers each write 5–10 nodes + edges in one tx; partial-failure on any of them must roll back the whole.
3. **Typed-endpoint edge semantics.** SurrealDB's `RELATION FROM<src> TO<dst>` constrains each edge to a concrete src/dst node-type pair; the 66-variant `Edge` enum in [`domain::model::edges`](../../../../modules/crates/domain/src/model/edges.rs) assumes this. Equivalent typed-endpoint constraints (foreign-key check on a join table, etc.) satisfy the criterion in another backend.
4. **Schema-free nested-field carrier.** SurrealDB's `FLEXIBLE TYPE object` lets a single column hold the phi-core `Session.inner: phi_core::session::model::Session` and `LoopRecordNode.inner: phi_core::session::model::LoopRecord` wraps without a separate migration each time phi-core's wrapped shape evolves. Equivalent JSONB or untyped-blob columns satisfy this.
5. **Forward-only idempotent migration runner.** Applied-version tracking via a dedicated ledger table (current impl: `_migrations` table with version + slug + applied_at). Migrations apply once, in numeric order; the runner is safe to re-run on every pod startup.
6. **Strict schema declarations.** `SCHEMAFULL` (or equivalent) tables for every load-bearing M1 node — no quietly-missing required field at write time. Schema-on-read backends (vanilla MongoDB, untyped key-value stores) are not eligible.
7. **UNIQUE index enforcement at the schema layer.** Several invariants ride on this: `bootstrap_credentials_digest` (one platform claim per host), `secrets_vault_slug` (no duplicate vault names per org), `identity_agent_id` (one Identity row per LLM agent per CH-16). Enforcement at the application layer is not sufficient because race conditions during compound-tx writes would otherwise produce ghost duplicates.

**Audit chain inheritance.** The BLAKE3 per-org audit hash chain (per [`m1/architecture/audit-events.md`](../implementation/m1/architecture/audit-events.md)) is computed in domain code from canonical bytes, not by the storage layer. Any conforming backend inherits the chain semantics for free; nothing storage-specific needs to be re-implemented.

**v1 trajectory.** A graph-native DB (Neo4j, Memgraph, DuckDB-PGQ) is a v1 conversation once access patterns stabilise. The ontology's tag predicates + typed-edge traversal can be expressed on a relational schema with the right indexing, but a graph-native engine may simplify some queries (authority-chain walks, supervision tree traversal). The configurability framing means that conversation can happen without rewriting the consumer layer.
