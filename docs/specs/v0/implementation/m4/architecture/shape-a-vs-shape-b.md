<!-- Last verified: 2026-04-23 by Claude Code -->

# Architecture — Shape A vs Shape B project creation

**Status: [PLANNED M4/P1]** — fleshed out at P1 close when `ProjectShape` ships.

**Shape A** = single-org project. Immediate materialisation via the
compound `apply_project_creation` tx. **Shape B** = co-owned by two
orgs. On submit, creates a Template E Auth Request with two approver
slots (one per owning-org admin); project materialises only on
both-approve.

4-outcome decision table pinned by `shape_b_approval_matrix_props`
proptest at P3:

| Approver A | Approver B | Outcome |
|---|---|---|
| Approve | Approve | Project materialises; Template A fires for lead |
| Approve | Deny | AR transitions to `Partial`; no project created |
| Deny | Approve | AR transitions to `Partial`; no project created |
| Deny | Deny | AR transitions to `Denied`; no project created |

See:
- [ADR-0025](../decisions/0025-shape-b-two-approver-flow.md).
- [Requirements admin/10 §W3](../../../requirements/admin/10-project-creation-wizard.md).
- [M4 plan archive §D4 / §D8](../../../../plan/build/a634be65-m4-agents-and-projects.md).
