<!-- Last verified: 2026-04-23 by Claude Code -->

# ADR-0025 — Shape B two-approver project creation flow

**Status: Accepted** — flipped at M4/P6 close after Shape A happy-path + Shape B submit (2-slot AR in `Pending`) + Shape B approve-pending handler (drives `transition_slot` through all 4 outcomes) + 4-outcome acceptance tests landed. Materialisation-after-both-approve is wired at the state-machine tier but does NOT yet reconstruct the full `CreateProjectInput` — that piece deferred to **C-M5-6** in the base build plan (Shape B's second-approve materialisation needs a sidecar table / persistence of the pending payload; at M4 the AR state machine drives correctly through all 4 outcomes, emits the correct audit events, but the `Approved` terminal path returns `project_id: null` in the response).

## Context

Project creation Shape B (R-ADMIN-10-W3) requires approval from BOTH co-owning orgs' admins before the project node materialises. This is the first two-approver governance flow in phi (M1-M3 only had single-approver patterns).

## Decision

1. **Reuse existing `AuthRequest.resource_slots[*].approvers: Vec<ApproverSlot>`** infrastructure (M1). No new state-machine variants.
2. **Two-stage tx shape**:
   - Stage 1 at submit: `apply_project_creation_shape_b` creates the Template E AR with 2 approver slots; deposits `AgentMessage` in each co-owner's inbox. No Project node yet.
   - Stage 2 at both-approve: handler calls `finalize_shape_b_after_approvals` which runs the same Shape A compound tx (Project + HAS_LEAD + HAS_AGENT + HAS_SPONSOR + BELONGS_TO edges + audit + event-bus emit).
3. **4-outcome decision table** pinned at P3 by a 50-case proptest (`shape_b_approval_matrix_props`):
   - Both-approve → project materialises.
   - Mixed approve+deny or deny+approve → AR transitions to `Partial`; no project.
   - Both-deny → AR transitions to `Denied`; no project.
4. **Deny notification**: on any deny, the requestor receives an inbox `AgentMessage` with the denying approver's reason. No project rollback required (project was never created).

## Consequences

**Positive:** zero new state-machine complexity; full use of M1's multi-slot AR primitive; proptest pins the invariant.

**Negative:** callers must know the Stage 1 / Stage 2 distinction. Documented in `m4/architecture/shape-a-vs-shape-b.md` + page 10 ops runbook.

**Neutral:** 3-party Shape C is out-of-scope at M4 (concept docs say "co-owned by two orgs"); if a future use-case emerges, it's a separate ADR not an extension of 0025.

## References

- [M4 plan §D4 / §D8](../../../../plan/build/a634be65-m4-agents-and-projects.md).
- [Requirements admin/10 §W3](../../../requirements/admin/10-project-creation-wizard.md).
- [shape-a-vs-shape-b.md](../architecture/shape-a-vs-shape-b.md).
