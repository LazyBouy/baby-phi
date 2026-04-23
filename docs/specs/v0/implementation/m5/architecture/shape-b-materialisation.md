<!-- Last verified: 2026-04-23 by Claude Code -->

# Shape B materialisation-after-both-approve (C-M5-6)

**Status**: [PLANNED M5/P4] — stub seeded at M5/P0; full sequence
diagrams + pre/post compound-tx pseudocode land at P4 when the
sidecar flip ships.

Documents the M5 fix for C-M5-6, the Shape B project-creation gap
that M4/P6 left in place: `approve_pending_shape_b` Approved
branch returns `project: None` because the `CreateProjectInput`
payload captured at submit time has nowhere to live.

## Problem (M4 state)

- M4/P6 shipped the full Shape B machinery — 2-approver AR, slot
  transitions, 4-outcome decision matrix, 50-case proptest + HTTP
  acceptance.
- Both-approve returns `{outcome: "terminal", state: "approved", project_id: null}`.
- `_keep_materialise_live` at [`projects/create.rs:756`](../../../../../../modules/crates/server/src/platform/projects/create.rs#L756)
  is a compiler-keep-alive hack so `materialise_project` stays in
  the binary despite having 0 callers.
- Reason: the submit handler captured `CreateProjectInput` (name,
  goal, leads, members, OKRs, token_budget, resource_boundaries)
  on the wire but persisted none of it — the AR carries only
  approver slots + justification text.

## Fix (M5/P4)

- Migration 0005 adds `shape_b_pending_projects` sidecar table
  (UNIQUE `auth_request_id` + `payload FLEXIBLE TYPE object`).
- Submit compound-tx writes the sidecar row alongside the AR.
- Approved branch reads the sidecar, calls `materialise_project`
  with the reconstructed input, deletes the sidecar inside the
  same tx, emits `platform.project.created`, fires
  `DomainEvent::HasLeadEdgeCreated` (which triggers Template A).
- Denied / Partial branches leave the sidecar in place (no
  project created) — operator-triggered cleanup in M7+.
- `_keep_materialise_live` deleted at P4 close.

## Acceptance tests

- `shape_b_both_approve_materialises_project` — flips from
  `project_id: null` to real id + verifies `has_lead` edge + fires
  Template A grant.
- `shape_b_sidecar_persisted_at_submit` — submit writes sidecar.
- `shape_b_sidecar_deleted_after_materialise` — approve path
  cleans up.

## Cross-references

- [M4 shape-a-vs-shape-b.md §Materialisation-after-approve](../../m4/architecture/shape-a-vs-shape-b.md).
- [Base plan §M5 carryovers §C-M5-6](../../../../plan/build/36d0c6c5-build-plan-v01.md).
- [M5 plan §G6 + §P4](../../../../plan/build/01710c13-m5-templates-system-agents-sessions.md).
