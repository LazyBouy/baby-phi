<!-- Last verified: 2026-04-23 by Claude Code -->

# User guide — Troubleshooting (M4)

**Status**: [EXISTS] since M4/P8.

Every M4 HTTP error carries a JSON body `{ "code": "<STABLE_CODE>",
"message": "..." }`. The CLI surfaces the `code` verbatim via
`phi: rejected (<CODE>): <message>` + maps it to one of the exit
codes pinned in [`cli-reference-m4.md`](cli-reference-m4.md).

## Agent surface (pages 08 + 09)

| Code | HTTP | CLI exit | Operator action |
|---|---|---|---|
| `AGENT_ID_IN_USE` | 409 | EXIT_REJECTED | Race with an in-flight create or operator-supplied id collision. Retry with a fresh UUID (the server mints one when omitted). |
| `AGENT_IMMUTABLE_FIELD_CHANGED` | 400 | EXIT_REJECTED | PATCH body attempted to change `id` / `kind` / `role` / `base_organization`. These fields are immutable post-creation at M4 scope. Create a new agent if the identity needs to change. |
| `AGENT_ROLE_INVALID_FOR_KIND` | 400 | EXIT_REJECTED | Role fails `is_valid_for(kind)`: Executive/Admin/Member require Human, Intern/Contract/System require LLM. Consult [ADR-0024](../decisions/0024-project-and-agent-role-typing.md). |
| `PARALLELIZE_CEILING_EXCEEDED` | 400 | EXIT_REJECTED | `parallelize` outside `[1, org_cap]`. Raise the org ceiling via page 05 (platform defaults) or lower the agent's value. |
| `EXECUTION_LIMITS_EXCEED_ORG_CEILING` | 400 | EXIT_REJECTED | Per-agent override violates ADR-0027's `≤ org snapshot` invariant. Lower the override fields OR raise the org snapshot via page 05. |
| `ACTIVE_SESSIONS_BLOCK_MODEL_CHANGE` | 409 | EXIT_REJECTED | PATCH tried to change `ModelConfig` while the agent has in-flight sessions. **M4 note**: `count_active_sessions_for_agent` stubs to 0 at M4 (C-M5-5), so this code path never fires until M5 flips the real query. |
| `SYSTEM_AGENT_READ_ONLY` | 403 | EXIT_REJECTED | Edit attempted on a `role=system` agent. System agents are platform-ops-managed; use platform-side tooling. |
| `UNAUTHENTICATED` | 401 | EXIT_PRECONDITION_FAILED | Session cookie missing/expired. Re-run `phi bootstrap claim`. |

## Project surface (pages 10 + 11)

| Code | HTTP | CLI exit | Operator action |
|---|---|---|---|
| `PROJECT_ID_IN_USE` | 409 | EXIT_REJECTED | `project_id` already exists. Usually a retry after a successful operation; verify via `phi project show --id <id>` before re-submitting. |
| `SHAPE_B_MISSING_CO_OWNER` | 400 | EXIT_REJECTED | Shape B project requires `co_owner_org_id`. |
| `SHAPE_A_HAS_CO_OWNER` | 400 | EXIT_REJECTED | Shape A is single-org by definition; omit `co_owner_org_id`. |
| `CO_OWNER_INVALID` | 400 | EXIT_REJECTED | Co-owner org equals primary OR co-owner org not found. |
| `LEAD_NOT_FOUND` | 404 | EXIT_REJECTED | `lead_agent_id` has no row. |
| `LEAD_NOT_IN_OWNING_ORG` | 400 | EXIT_REJECTED | Lead agent's `owning_org` isn't one of the project's owning orgs. |
| `MEMBER_INVALID` | 400 | EXIT_REJECTED | Member or sponsor id not found OR not in an owning org. |
| `ORG_NOT_FOUND` | 404 | EXIT_REJECTED | `org_id` has no row. |
| `PENDING_AR_NOT_FOUND` | 404 | EXIT_REJECTED | `ar_id` unknown. |
| `PENDING_AR_NOT_SHAPE_B` | 400 | EXIT_REJECTED | AR isn't a Shape B project-creation AR; the approve-pending endpoint only drives Shape B. |
| `PENDING_AR_ALREADY_TERMINAL` | 409 | EXIT_REJECTED | AR already decided (Approved / Denied / Partial). No further action possible. |
| `APPROVER_NOT_AUTHORIZED` | 403 | EXIT_REJECTED | Caller isn't one of the two approver slots on the AR OR (on page 11) has no relation to the project. |
| `PROJECT_NOT_FOUND` | 404 | EXIT_REJECTED | Project `:id` has no row. |
| `PROJECT_ACCESS_DENIED` | 403 | EXIT_REJECTED | Viewer is not a member of any owning org and not on the roster. |
| `OKR_VALIDATION_FAILED` | 400 | EXIT_REJECTED | Patch violates a shape rule (see §OKR validation rules below). |
| `TRANSITION_ILLEGAL` | 400 | EXIT_REJECTED | Shape B AR state transition rejected by `transition_slot`. Rare — usually double-decision race. |

## OKR validation rules (PATCH /projects/:id/okrs)

All rules surface as `OKR_VALIDATION_FAILED` with a specific
`message`:

1. **`op=create`** — `objective_id` / `kr_id` must not exist.
2. **`op=update`** — `objective_id` / `kr_id` must exist.
3. **`op=delete` on an Objective** — fails if any surviving `KeyResult`
   references that `objective_id`. Delete the dependent KRs first
   (sequentially in the same patch, or in an earlier patch).
4. **`measurement_type` vs value shape** — `count` needs
   `target_value.kind = "integer"`; `boolean` needs `"bool"`;
   `percentage` needs `"percentage"` with `0.0 <= value <= 1.0`;
   `custom` accepts any JSON.
5. **No duplicate id** after the full patch applies (cross-entry
   invariant).

## Cross-cutting codes (inherited)

Inherited from M1/M2/M3 — full table in
[`../../m3/user-guide/troubleshooting.md`](../../m3/user-guide/troubleshooting.md):

- `VALIDATION_FAILED` (400) — generic input error.
- `AUDIT_EMIT_FAILED` (500) — hash-chain write failed; underlying
  data write may have succeeded.
- `UNAUTHENTICATED` (401) — missing/expired cookie.
- `INTERNAL` (500) — unexpected repository/server error; check logs
  for the span trace id + correlated `project_id` / `agent_id`.

## Dashboard quirks (page 07 after M4/P8 retrofit)

M4/P8 expanded `AgentsSummary` from `{ human, llm }` to the 6-variant
`AgentRole` buckets + `unclassified`. Known shapes to expect:

- **Fresh org from `phi org create` wizard**: CEO + 2 system agents,
  all with `role = None` → every one lands in `unclassified`. Operators
  assign roles via page 09 as they add agents.
- **Legacy pre-M4 rows** show up in `unclassified` too. Rollups are
  correct; the bucket exists so operators notice the gap.

The `ProjectsSummary` `shape_a` / `shape_b` counters retrofit from
hardcoded zero to real counts via `count_projects_by_shape_in_org`.
A non-zero `shape_a + shape_b` without matching `active` count means
the dashboard view and the projects table disagree — run the
`acceptance_m4::full_m4_happy_path_bootstrap_to_dashboard` test
locally to reproduce.

## Cross-org isolation invariants

These invariants are pinned by
`acceptance_m4::cross_org_project_show_denies_foreign_viewer` and
`acceptance_m4::dashboard_shape_counters_are_org_scoped`:

- `GET /api/v0/projects/:id` is keyed by `ProjectId` (UUID), not by
  name. Two orgs can have identically-named projects; a viewer from
  Org A still gets `403 PROJECT_ACCESS_DENIED` when requesting Org
  B's project, even if Org A has a project with the same name.
- Dashboard `ProjectsSummary.shape_a` / `.shape_b` counters filter by
  the `BELONGS_TO` edge. A project co-owned by Orgs B + C never
  surfaces on Org A's dashboard regardless of the project's shape.
- `ViewerRole::ProjectLead` resolution (`resolve_viewer_role`)
  intersects `list_projects_led_by_agent(viewer)` with this org's
  project set. An agent who leads projects in other orgs but has no
  led-project in THIS org gets `Member` (or `None` if not in the
  org), not `ProjectLead`.

If any of these invariants ever regress — e.g. a name-based lookup
sneaks into the access gate, or `list_projects_in_org` starts
returning cross-org projects — the acceptance tests fail loudly.
Treat a failure here as a security incident, not a test flake.

## Cross-references

- [Top-level runbook §M4](../../../../../../docs/ops/runbook.md) —
  operator-facing aggregated index.
- [M3 troubleshooting](../../m3/user-guide/troubleshooting.md) —
  M1/M2/M3 inherited codes.
- Per-page ops docs:
  [roster](../operations/agent-roster-operations.md) ·
  [profile editor](../operations/agent-profile-editor-operations.md) ·
  [project creation](../operations/project-creation-operations.md) ·
  [project detail](../operations/project-detail-operations.md).
- [M4 plan archive §P8](../../../../plan/build/a634be65-m4-agents-and-projects.md).
