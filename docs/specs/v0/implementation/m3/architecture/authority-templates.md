<!-- Last verified: 2026-04-22 by Claude Code -->

# Architecture — Authority Templates A/B/C/D

**Status: [PLANNED M3/P2]** — fleshed out when P2 ships.

M3/P2 adds pure-fn builders for Templates A, B, C, D in
`domain::templates::{a,b,c,d}`, each returning an adoption-shaped
`AuthRequest` in `Approved` state with the CEO as the sole
approver. M3 wires **adoption at org creation** (P4); M5 wires the
**trigger-fire behaviour** that actually issues grants when an edge
event occurs.

Template E already shipped in M2/P3 (`domain::templates::e`).
Template F stays reserved (per plan §D2 — "reserved for M6
break-glass work; M3 does not adopt F at org creation").

## Template semantics (concept refresher)

- **Template A** — project-lead authority. When a `HAS_LEAD` edge
  fires from Project P to Agent X, the template issues a grant
  giving X `[read, inspect, list]` on every session tagged
  `project:P`. M3/P1 pre-wires the `HasLead` edge variant so
  P2's pure-fn builder can name it.
- **Template B** — direct delegation. When Agent A spawns Agent B
  via `DELEGATES_TO` at loop `Ln`, A gets `[read, inspect]` on
  sessions tagged `delegated_from:Ln`.
- **Template C** — hierarchical org chart. When an Agent is
  appointed to a node in the org tree, they get subtree `[read,
  inspect, list]`.
- **Template D** — project-scoped role. `supervisor` role on
  Project P grants `[read, inspect]` on sessions tagged `project:P
  AND role_at_creation:worker`.

## Orchestrator split (D7)

Per plan §D7: pure-fn per-template builders in
`domain::templates::{a,b,c,d}`; the **orchestrator**
(`build_template_suite_adoption`) that composes them with the CEO as
approver lives in the server business logic
(`server::platform::orgs::create`) — it depends on the CEO-grant
existing, which is a business concern, not a pure-fn concern.

See:
- [`../../../plan/build/563945fe-m3-organization-creation.md`](../../../../plan/build/563945fe-m3-organization-creation.md) §G6 / §D7.
- [`../../../concepts/permissions/05-memory-sessions.md`](../../../concepts/permissions/05-memory-sessions.md) — Template A-E semantics.

## phi-core leverage

None — templates are baby-phi's governance primitive; phi-core has
no permissions / auth-request / grant surface to reuse.
