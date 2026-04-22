<!-- Last verified: 2026-04-22 by Claude Code -->

# Architecture — M4 overview

**Status: [PLANNED M4/P1]** — fleshed out at P1 close when the
foundation types + migration + web primitives are shipped.

System map: admin pages 08 (agent roster list), 09 (agent profile
editor), 10 (project creation wizard Shape A + Shape B), 11 (project
detail). First milestone to ship an in-process domain event bus
(`TemplateAFireListener` subscribes to `HasLeadEdgeCreated`) and a
per-agent `ExecutionLimits` override layer (inherits from M3's
`OrganizationDefaultsSnapshot` by default).

See:
- [M4 plan archive](../../../../plan/build/a634be65-m4-agents-and-projects.md) §Part 4 — per-phase deliverables.
- [shape-a-vs-shape-b.md](shape-a-vs-shape-b.md) — two-approver flow architecture.
- [event-bus.md](event-bus.md) — in-process event bus + subscription contracts.
- [phi-core-reuse-map.md](phi-core-reuse-map.md) — M4 phi-core leverage table.
