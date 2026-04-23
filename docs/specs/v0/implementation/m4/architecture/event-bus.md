<!-- Last verified: 2026-04-23 by Claude Code -->

# Architecture — In-process domain event bus

**Status: [PLANNED M4/P1]** — scaffolded at P1; wired at P3.

First reactive infra in phi. `trait EventBus { async fn emit(&self, event: DomainEvent); fn subscribe(&self, handler: Arc<dyn EventHandler>); }` with `InProcessEventBus` as the M4 default impl.

**Design constraints:**
- Works with both in-memory and SurrealDB repo impls (no SurrealDB-LIVE dependency).
- **Fail-safe ordering**: emit happens AFTER a successful compound-tx commit. If emit fails, commit is already durable; the event is logged with `event_id` for manual replay. M7b adds automatic event-retry infra.
- **Orthogonal to `phi_core::types::AgentEvent`** (agent-loop telemetry stream) per [`phi/CLAUDE.md` §Orthogonal surfaces](../../../../../../CLAUDE.md). phi's `DomainEvent` is a governance reactive trigger; phi-core's `AgentEvent` is a runtime telemetry stream.

**Initial subscribers:**
- `TemplateAFireListener` — subscribes to `HasLeadEdgeCreated`, calls `fire_grant_on_lead_assignment`, persists the Grant, emits `TemplateAAdoptionFired` audit event.

**Initial `DomainEvent` variants:**
- `HasLeadEdgeCreated { project, lead, at }` — emitted by the Shape A compound tx after successful commit, and by the Shape B post-both-approve materialisation.

See:
- [ADR-0028](../decisions/0028-domain-event-bus.md).
- [M4 plan archive §D7 / §D-M4-4](../../../../plan/build/a634be65-m4-agents-and-projects.md).
- [`phi/CLAUDE.md`](../../../../../../CLAUDE.md) §Orthogonal surfaces — why `DomainEvent` ≠ `phi_core::AgentEvent`.
