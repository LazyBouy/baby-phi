<!-- Last verified: 2026-04-22 by Claude Code -->

# Architecture — Project Creation Wizard (page 10)

**Status: [PLANNED M4/P6]** — fleshed out at P6 close.

6-step web wizard. Shape A (single-org) immediate materialisation via `apply_project_creation` compound tx; Shape B (co-owned) creates a Template E Auth Request with 2 approver slots and materialises only on both-approve. Template A grant fires **automatically** via the domain event bus subscription (new at M4 per ADR-0028) on every `HAS_LEAD` edge write.

See:
- [Requirements admin/10](../../../requirements/admin/10-project-creation-wizard.md).
- [shape-a-vs-shape-b.md](shape-a-vs-shape-b.md).
- [event-bus.md](event-bus.md).
- [M4 plan archive §P6](../../../../plan/build/a634be65-m4-agents-and-projects.md).
