<!-- Last verified: 2026-04-22 by Claude Code -->

# ADR-0028 — In-process domain event bus + Template A subscription

**Status: Accepted** — flipped at M4/P3 close after `TemplateAFireListener` + `InProcessEventBus` wiring landed in `AppState` + the listener test suite passed green.

## Context

Template A fires on `HAS_LEAD` edge creation. M4/P0 planning asked: should the firing be direct-call only (every write-site invokes the pure-fn) or reactive (a subscription listens for edge-change events)?

User decision (M4/P0): **reactive subscription at M4**, not deferred to M5. Requires phi's first domain-event-bus infra.

## Decision

1. **New module `domain/src/events/`** containing:
   - `trait EventBus: Send + Sync { async fn emit(&self, event: DomainEvent); fn subscribe(&self, handler: Arc<dyn EventHandler>); }`.
   - `enum DomainEvent { HasLeadEdgeCreated { project: ProjectId, lead: AgentId, at: DateTime<Utc> } }` — extensible for future edge-change events.
   - `struct InProcessEventBus` default impl (uses `tokio::sync::broadcast` or equivalent).
   - `trait EventHandler: Send + Sync { async fn handle(&self, event: &DomainEvent) -> Result<(), EventError>; }`.
2. **`AppState.event_bus: Arc<dyn EventBus>`** injected at boot; test harness can use a bus-less or no-op impl where reactive behaviour is out-of-scope.
3. **Fail-safe ordering**: compound-tx handlers emit events AFTER successful commit. If emit fails, commit is already durable; event is logged with `event_id` for manual replay. M7b adds event-retry infra.
4. **First subscriber**: `TemplateAFireListener` listens for `HasLeadEdgeCreated`, calls the pure-fn `fire_grant_on_lead_assignment`, persists the Grant, emits `TemplateAAdoptionFired` audit event.
5. **Orthogonal to `phi_core::AgentEvent`** per [`phi/CLAUDE.md` §Orthogonal surfaces](../../../../../../CLAUDE.md). `DomainEvent` is governance reactive-trigger; `phi_core::AgentEvent` is agent-loop telemetry.
6. **SurrealDB LIVE queries deliberately rejected** as the primary mechanism: they require SurrealDB-specific infra (breaks in-memory impl) and can't express the "emit after commit + log on emit failure" semantics cleanly.

## Consequences

**Positive:** clean separation between write-site (compound tx) and reactive side-effects (subscribers). Future M5+ listeners (memory-extractor, agent-catalog) plug in via the same bus.

**Negative:** events between crash-safe tx commit and process crash are lost. Acceptable at M4 (Template A is idempotent — re-running the grant issue on replay is safe); M7b adds persistent event queue for exactly-once semantics.

**Neutral:** single-process only at M4. Multi-process bus (NATS / Kafka) is M7+ scale work.

## References

- [M4 plan §D7 / §D-M4-4](../../../../plan/build/a634be65-m4-agents-and-projects.md).
- [event-bus.md](../architecture/event-bus.md).
- [template-a-firing.md](../architecture/template-a-firing.md).
- [Requirements system/s05](../../../requirements/system/s05-template-adoption-grant-fires.md).
- [`phi/CLAUDE.md`](../../../../../../CLAUDE.md) §Orthogonal surfaces.
