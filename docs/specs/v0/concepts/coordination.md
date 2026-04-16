<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->

# Multi-Agent Coordination

> Extracted from brainstorm.md Sections 5 + 6.
> See also: [agent.md](agent.md), [permissions.md](permissions/README.md) (delegation rules), [organization.md](organization.md) (Market)

---

## Future Scenarios (from phi-core roadmap)

These phi-core future scenarios directly feed into baby-phi's coordination design:

### HITL Resume (Human-in-the-Loop)

Agent is aborted mid-execution, human reviews, then resumes. Requires checkpoint/restore on Agent state. phi-core needs `Agent::checkpoint()` / `Agent::restore(checkpoint)`.

**baby-phi implication:** The data model must support partial sessions — loops that are `Aborted` with a resumption path. The graph edge `CONTINUES_FROM` with `ContinuationKind::Rerun` or `Branch` captures this.

### Checkpoint Restore (Cross-Process)

Serialize agent state to storage, load it in a different process. phi-core needs `AgentSnapshot` type.

**baby-phi implication:** The data layer IS the persistence. If all state is in the graph, checkpoint/restore is just "read the graph" / "write the graph". No separate snapshot mechanism needed.

### Parallel Exploration

Multiple branches from the same checkpoint run concurrently. phi-core supports this via `agent_loop_continue(Branch)` with cloned contexts.

**baby-phi implication:** The `Loop` node naturally supports this — multiple Loops share the same `CONTINUES_FROM` parent, each as a sibling branch. `ParallelGroupRecord` (value object on Loop) tracks which branch was selected. The `PARALLEL_WITH` edge connects siblings.

### Auto Origin/Continue Selection

Agent decides whether to `agent_loop` or `agent_loop_continue` based on context state.

**baby-phi implication:** This is the "agent invocation layer" — baby-phi should provide a high-level `send(agent_id, message)` that inspects the agent's current state in the data model and dispatches correctly.

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
| **Storage backend** | **SQLite** (single-file, transactional, migratable). | Step up from phi-core's JSON files with minimal operational overhead. A graph DB (Neo4j, Memgraph, DuckDB-with-PGQ) is a v1 conversation once access patterns stabilise. The ontology doesn't require graph-native storage — tag predicates and edge traversal can be expressed on a relational schema with the right indexing. |
| **Query language** | **Custom tag-predicate DSL** for Grant selectors (already specified throughout the permissions spec). **Cypher-inspired subset** (`MATCH` / `WHERE` / `RETURN`) for graph traversal. Agents see a single `query(...)` API surface; the DSL underneath is an implementation detail. | Tag predicates are the primitive the permissions model is built on; using them for selectors keeps one language. Graph traversal is needed for authority-chain walks and supervision queries; Cypher is the least-surprising syntax. SQL was considered and rejected — it is awkward for transitive edge traversal. |
| **Schema versioning** | **Additive-only in v0.** Adding a property to a node type or a new edge type is a non-breaking change and does not require migration. Removing or renaming a property requires a **migration Auth Request** (Template E shape) that emits the migration plan as provenance and touches all affected instances atomically. | Additive-only is the cheapest path that preserves audit continuity. Any destructive change has to route through the Auth Request mechanism for the same reason ownership transfers do: the authority chain stays traceable. |
| **Event sourcing** | **Hybrid.** State-based current-node view for reads; append-only `AgentEvent` stream for audit and replay. | Matches phi-core's existing event pattern. Full event sourcing (deriving all reads from replay) is slower for common queries and adds operational complexity without enough benefit at v0 scale. The audit stream is separately queryable when an incident requires replay. |
| **Consistency model** | **Last-writer-wins per node** with timestamped optimistic concurrency on writes (retry-on-conflict). **Provenance-carrying edges** (`DESCENDS_FROM`, `EMITTED_BY`, `APPROVED_BY`) are **append-only** — you add a new edge rather than rewriting an existing one. | LWW is simple and acceptable for most agent-coordination workloads where conflicts are rare. Provenance edges can never LWW — an audit trail that silently loses edges is broken. The split gives us cheap writes on state while keeping the authority chain tamper-evident. |
| **Memory types** | The four types from Claude Code's memory model: **`user`, `feedback`, `project`, `reference`**. Adopt as the v0 `memory_type` enum on the Memory node. | These four cover the observed categories in practice (agent preferences, process corrections, project-specific facts, external references). New types can be added non-breakingly (additive-only rule above). |
| **MCP lifecycle** | **Lazy connection on first use**, persistent thereafter for the duration of the owning Session. Disconnect on session end. Re-connection policy is the MCP server's concern. | Eager connection wastes resources when most sessions never touch most tools. Persistent-per-session balances startup cost against connection churn. Tying lifetime to the Session gives a clean cleanup trigger. |
| **Provider testing** | **System-session shape** — tests run under a system agent in a session with no `project:` tag, only `agent:system-tester` and `org:{test_org}`. Sidesteps project-scoped permission checks cleanly. | Avoids the temptation to add a "test mode" that bypasses permissions — which would then become a latent exploit vector. The system-session shape is a normal part of the model, well-specified in [permissions/05-memory-sessions.md § Sessions as a Tagged Resource](permissions/05-memory-sessions.md#sessions-as-a-tagged-resource), so provider tests compose with the same machinery as production sessions. |

> **Revisit triggers.** Each default should be revisited when any of: (a) a concrete failure mode appears in production, (b) the v0 scale assumption is broken (e.g., a single SQLite file becomes the bottleneck), or (c) a more principled alternative surfaces. Until then, implementers treat these as load-bearing v0 invariants.
