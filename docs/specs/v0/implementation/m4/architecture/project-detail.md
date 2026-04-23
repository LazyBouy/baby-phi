<!-- Last verified: 2026-04-23 by Claude Code -->

# Architecture — Project Detail (page 11) — M4/P7

**Status**: [EXISTS] since M4/P7.

**What ships at M4/P7**: a read-only project aggregate (`GET
/api/v0/projects/:id`) plus an in-place OKR editor (`PATCH
/api/v0/projects/:id/okrs`), surfaced via CLI + Web. The panel is the
FK-containment target for the future "Recent sessions" list (C-M5-3 in
the base plan — baby-phi's governance `Session` node persists at M5).

## Wire topology

| Method | Path | Op | Status code on success |
|---|---|---|---|
| `GET` | `/api/v0/projects/:id` | show aggregate | `200 OK` |
| `PATCH` | `/api/v0/projects/:id/okrs` | in-place OKR patch | `200 OK` |

Both routes live under `AuthenticatedSession`; both return a
structured error with stable code strings (see the §Error-code table
in the ops doc) on `400 / 403 / 404 / 500`.

## Wire shape — `ProjectDetail`

```jsonc
{
  "project": {
    "id":          "<ProjectId>",
    "name":        "Atlas",
    "description": "Moonshot memory benchmark",
    "goal":        "0.85 recall at 10k tokens",
    "status":      "planned",
    "shape":       "shape_a",
    "token_budget": 1000000,
    "tokens_spent": 0,
    "objectives":  [ /* Objective value objects */ ],
    "key_results": [ /* KeyResult value objects */ ],
    "resource_boundaries": { /* ResourceBoundaries */ },
    "created_at":  "2026-04-23T10:00:00Z"
  },
  "owning_org_ids": ["<OrgId>"],          // 1 for Shape A, 2 for Shape B
  "lead_agent_id":  "<AgentId | null>",
  "roster": [
    {
      "agent_id":       "<AgentId>",
      "kind":           "human",
      "display_name":   "Ana",
      "role":           "executive",      // 6-variant AgentRole per ADR-0024
      "project_role":   "lead"            // lead | member | sponsor
    }
  ],
  "recent_sessions": []                   // M4 placeholder — C-M5-3 flips this
}
```

## phi-core leverage

The page is **deliberately phi-core-stripped**. Every `RosterMember`
drops the per-agent `blueprint` field (which wraps
`phi_core::agents::profile::AgentProfile` per M3/P1). Drill-down to
the full blueprint is available via page 09 (agent profile editor) —
a different endpoint with a different wire contract.

| Q | Answer |
|---|---|
| **Q1** — direct imports | **0** in [detail.rs](../../../../../../modules/crates/server/src/platform/projects/detail.rs). |
| **Q2** — transitive | 0 at the wire tier. The snapshot test `tests::wire_shape_strips_phi_core` asserts the serialised `ProjectDetail` JSON has no `defaults_snapshot` / `blueprint` / `execution_limits` / `context_config` / `retry_config` keys at any depth. |
| **Q3** — rejections | `phi_core::Session` / `LoopRecord` / `Turn` — deferred to M5 per D11 of the M3 plan and [C-M5-3](../../../../plan/build/36d0c6c5-build-plan-v01.md). `phi_core::Usage` — token budget is governance-level, not per-loop. `phi_core::AgentEvent` — orthogonal governance-log surface per `phi/CLAUDE.md`. |

## OKR patch contract

The body of `PATCH /api/v0/projects/:id/okrs` is a `{ "patches": […]
}` envelope over an array of tagged entries:

```jsonc
{
  "patches": [
    {
      "kind": "objective",
      "op":   "create",
      "payload": { /* Objective */ }
    },
    {
      "kind": "key_result",
      "op":   "update",
      "payload": { /* KeyResult */ }
    },
    {
      "kind": "objective",
      "op":   "delete",
      "objective_id": "obj-3"
    }
  ]
}
```

Validation rules (enforced server-side by `apply_okr_patch`):

1. `op=create` — id must not already exist.
2. `op=update` — id must already exist.
3. `op=delete` on an Objective — fails if any `KeyResult` still
   references that `objective_id` (delete dependent KRs first).
4. Every `KeyResult.target_value` / `current_value` must shape-match
   its `measurement_type` (via `MeasurementType::is_valid_value`) —
   same rule as the creation-time validator in
   `create.rs::validate_okrs`.
5. No duplicate `objective_id` or `kr_id` after the full patch
   applies (cross-entry invariant checked at the end).

On success the response body carries the post-image + one
`audit_event_id` per applied mutation:

```jsonc
{
  "project_id": "<ProjectId>",
  "audit_event_ids": ["<AuditEventId>", "..."],
  "objectives":  [ /* Objective[] */ ],
  "key_results": [ /* KeyResult[] */ ]
}
```

Every successful mutation emits `platform.project.okr_updated`
(Logged) onto the project's primary owning-org audit chain with the
`(before, after)` pair for audit-log replay.

## Access gate

The reader path (`GET /api/v0/projects/:id`) accepts the viewer if
**any** of:

- viewer's `Agent.owning_org` ∈ project's `owning_org_ids`
  (organisation member);
- viewer is the project's `HAS_LEAD` target (the project lead).

The writer path (`PATCH …/okrs`) applies the same access rule; there
is no separate grant check at M4. A production-ready grant-aware gate
(require `manage` on `project:<id>`) lands at M5+ alongside the
session-launch permission-check refresh.

## Failure table

| Code | HTTP | Cause |
|---|---|---|
| `PROJECT_NOT_FOUND` | 404 | `project_id` has no row. |
| `PROJECT_ACCESS_DENIED` | 403 | Viewer is not a member of any owning org and not on the roster. |
| `OKR_VALIDATION_FAILED` | 400 | Patch violates one of the §OKR patch contract rules. |
| `APPROVER_NOT_AUTHORIZED` | 403 | Writer-path access gate (shared code with the Shape B approval path — re-used for "no relation to this project"). |
| `REPOSITORY_ERROR` | 500 | Underlying storage call failed. |
| `AUDIT_EMIT_FAILED` | 500 | The row was upserted but the audit chain append failed. |

## Repo surface

Repository methods used by this page (all already shipped at M4/P2
except `upsert_project`, added at M4/P7):

- `get_project(id)` — project row.
- `get_agent(id)` — viewer row (for `owning_org`).
- `list_all_orgs()` + `list_projects_in_org(org)` — owning-org
  resolution (no dedicated `list_owning_orgs_for_project` at M4
  scope).
- `list_agents_in_org(org)` + `list_projects_led_by_agent(agent)` —
  roster + lead resolution.
- `upsert_project(project)` — **new at M4/P7**. Create-or-replace
  row; used by the OKR editor.

The roster panel at M4 currently surfaces **only the lead**. Member +
sponsor roster read-back requires a dedicated
`list_project_roster(project_id)` method (scoped for a follow-up; the
edges themselves were written at creation time via
`apply_project_creation`'s `HAS_AGENT` / `HAS_SPONSOR` tables, so the
data is ready).

## Tests

- Unit (domain + orchestrator): 6+ tests covering wire-strip,
  OKR patch serde (4 shapes), recent-sessions placeholder.
- Acceptance ([acceptance_projects_detail.rs](../../../../../../modules/crates/server/tests/acceptance_projects_detail.rs)): 7 end-
  to-end scenarios — happy show, 404, 403, OKR patch create, update,
  delete, invalid patch rejection.

## Cross-references

- [Project creation (M4/P6)](project-creation.md) — writer path that
  originates the project row.
- [Agent profile editor (M4/P5)](agent-profile-editor.md) — where the
  blueprint drill-down actually lives; page 11's roster panel only
  surfaces the compact identity row.
- [ADR-0024 — Project + AgentRole typing](../decisions/0024-project-and-agent-role-typing.md).
- [Requirements admin/11](../../../requirements/admin/11-project-detail.md).
- [Base plan §M5 / C-M5-3](../../../../plan/build/36d0c6c5-build-plan-v01.md).
