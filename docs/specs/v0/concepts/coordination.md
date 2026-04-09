<!-- Status: CONCEPTUAL -->

# Multi-Agent Coordination

> Extracted from brainstorm.md Sections 5 + 6.
> See also: [agent.md](agent.md), [permissions.md](permissions.md) (delegation rules), [organization.md](organization.md) (Market)

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

A supervisor agent that spawns, monitors, and coordinates worker agents. Maps directly to `DELEGATES_TO` edges. The orchestrator has `HAS_PERMISSION` to spawn and monitor.

---

## Open Design Questions

- [ ] **Storage backend:** Start with JSON files (like phi-core sessions)? SQLite? In-memory graph?
- [ ] **Query language:** How do agents (and the system) query the graph? Custom DSL? SQL? Cypher-like?
- [ ] **Schema versioning:** When a node type evolves, how are old instances migrated?
- [ ] **Event sourcing:** Should the data model be event-sourced (append-only log of mutations) or state-based (mutable current state)?
- [ ] **Consistency model:** When two agents write to the same node concurrently, who wins?
- [ ] **Memory types:** What categories of memory exist? (user, feedback, project, reference — borrowing from Claude Code's memory model)
- [ ] **MCP lifecycle:** When an McpServer node is created, does the connection happen eagerly or lazily?
- [ ] **Provider testing:** How does standalone Message testing (without Agent/Session) fit the ontology? Is it a "system agent" or a special path?
