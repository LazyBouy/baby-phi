<!-- Last verified: 2026-04-22 by Claude Code -->

# Architecture — Project model (M4)

**Status: [PLANNED M4/P1]** — fleshed out at P1 close when `Project` struct ships.

`Project` node with embedded `Objective` + `KeyResult` value objects (per [`concepts/project.md`](../../../concepts/project.md) §OKRs). `ProjectShape { A, B }` discriminator. `ProjectStatus { Planned, InProgress(pct), OnHold(reason), Finished }` state machine. `ResourceBoundaries` subset reference into owning org's catalogue.

See:
- [concepts/project.md](../../../concepts/project.md) — source of truth for the model.
- [ADR-0024](../decisions/0024-project-and-agent-role-typing.md).
- [M4 plan archive §D1](../../../../plan/build/a634be65-m4-agents-and-projects.md).
