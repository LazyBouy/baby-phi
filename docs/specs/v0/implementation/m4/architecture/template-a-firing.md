<!-- Last verified: 2026-04-23 by Claude Code -->

# Architecture — Template A firing (s05 integration)

**Status: [PLANNED M4/P2+P3]** — pure-fn builder at P2; event-listener subscription wired at P3.

Template A is the "project lead → auto-grant on `HAS_LEAD` assignment" system flow. At M3 phi shipped the *adoption* piece (each org self-approves Template A at creation). M4 ships the *firing* piece:

1. **Pure-fn grant builder** (`domain/src/templates/a.rs::fire_grant_on_lead_assignment`) at P2. Constructs a `Grant` with `[read, list, inspect]` on `project:<id>` resource, holder = the assigned lead, provenance linked to the org's Template A adoption AR.
2. **Reactive subscription** at P3: `TemplateAFireListener` subscribes to `DomainEvent::HasLeadEdgeCreated` on the bus. When an edge is written (either by Shape A compound tx or Shape B post-both-approve materialisation), the listener invokes the pure-fn, persists the Grant via repo, emits a `TemplateAAdoptionFired` audit event.

**Fail-safe ordering** (per ADR-0028): bus emit happens AFTER the compound-tx commit. If emit fails, commit is already durable; event is logged for manual replay.

See:
- [Requirements system/s05](../../../requirements/system/s05-template-adoption-grant-fires.md).
- [event-bus.md](event-bus.md).
- [ADR-0028](../decisions/0028-domain-event-bus.md).
- [M4 plan archive §D7 / §D-M4-4](../../../../plan/build/a634be65-m4-agents-and-projects.md).
