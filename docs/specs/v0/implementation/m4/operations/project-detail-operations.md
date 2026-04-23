<!-- Last verified: 2026-04-23 by Claude Code -->

# Operations — Project Detail (page 11) — M4/P7

**Status**: [EXISTS] since M4/P7.

Operator runbook for `GET /api/v0/projects/:id` + `PATCH
/api/v0/projects/:id/okrs` + the `phi project show` / `phi project
update-okrs` CLI subcommands + the Next.js page at
`/organizations/[org_id]/projects/[id]`.

## Error-code table

Every non-2xx response carries
`{ "code": "<STABLE_CODE>", "message": "..." }`. Stable codes:

| HTTP | Code | Operator action |
|---|---|---|
| 404 | `PROJECT_NOT_FOUND` | The `project_id` has no row. Typo, race with a delete (project deletion ships at M5+), or the id belongs to another tenant. |
| 403 | `PROJECT_ACCESS_DENIED` | Viewer is not a member of any owning org and is not on the roster. Verify the viewer's `Agent.owning_org` and the project's owning-org edges (`phi org list --json` + `phi project show --json`). |
| 403 | `APPROVER_NOT_AUTHORIZED` | The writer path re-uses this code when the viewer has no relation to the project. Treat identically to `PROJECT_ACCESS_DENIED` — the stable code differs only for historical reasons (shared with Shape B approval). |
| 400 | `OKR_VALIDATION_FAILED` | The PATCH body violates an OKR rule. The `message` names the offending `objective_id` / `kr_id` + the specific rule. Common causes: duplicate id, delete on an Objective with dependent KRs, `measurement_type` vs `target_value` shape mismatch. |
| 400 | `VALIDATION_FAILED` | Generic input error (missing required field, malformed UUID, etc.). |
| 500 | `INTERNAL` | A repository call failed mid-operation. See server logs — the `project_id` + span trace id are logged at `error`. |
| 500 | `AUDIT_EMIT_FAILED` | The row was upserted but the audit-chain append failed. The project state is durable; the audit chain is inconsistent. Run the per-org audit chain repair playbook (see `docs/ops/runbook.md §M1 — Audit-chain repair`). Retries are NOT idempotent on the audit path — do not replay the PATCH body. |

## CLI recipes

### Show a project

```bash
phi project show --id <project_id>
# human-readable summary — project header, owning orgs, roster size,
# OKR counts, "recent sessions (none — C-M5-3)"

phi project show --id <project_id> --json
# raw ProjectDetail JSON — pipe into jq for scripting
```

### Patch OKRs in place

OKR patches are JSON arrays. Each entry has the shape
`{ "kind": "objective" | "key_result", "op": "create" | "update" |
"delete", ... }` — see architecture doc §OKR patch contract for the
full grammar.

```bash
# Create an objective + one KR in one round-trip.
phi project update-okrs --id <project_id> --patch-json '[
  {
    "kind": "objective",
    "op":   "create",
    "payload": {
      "objective_id": "q2-recall",
      "name":         "Hit 0.85 recall at 10k tokens",
      "description":  "",
      "status":       "active",
      "owner":        "<lead_agent_id>",
      "key_result_ids": []
    }
  },
  {
    "kind": "key_result",
    "op":   "create",
    "payload": {
      "kr_id":            "q2-recall-1",
      "objective_id":     "q2-recall",
      "name":             "Recall score",
      "description":      "",
      "measurement_type": "percentage",
      "target_value":     { "kind": "percentage", "value": 0.85 },
      "owner":            "<lead_agent_id>",
      "status":           "not_started"
    }
  }
]'
```

The CLI prints `OKR patch applied: N mutation(s)` on success. Each
applied mutation emits one `platform.project.okr_updated` audit event
onto the primary owning-org audit chain; a replay reproduces the
patch in order.

### Update a KR's progress

```bash
phi project update-okrs --id <project_id> --patch-json '[
  {
    "kind": "key_result",
    "op":   "update",
    "payload": {
      "kr_id":            "q2-recall-1",
      "objective_id":     "q2-recall",
      "name":             "Recall score",
      "description":      "",
      "measurement_type": "percentage",
      "target_value":     { "kind": "percentage", "value": 0.85 },
      "current_value":    { "kind": "percentage", "value": 0.71 },
      "owner":            "<lead_agent_id>",
      "status":           "in_progress"
    }
  }
]'
```

### Delete an Objective

Always delete dependent KRs **first** — the server rejects a
delete on an Objective with surviving KRs (`OKR_VALIDATION_FAILED`).

```bash
phi project update-okrs --id <project_id> --patch-json '[
  { "kind": "key_result", "op": "delete", "kr_id": "q2-recall-1" },
  { "kind": "objective", "op": "delete", "objective_id": "q2-recall" }
]'
```

Both mutations apply in a single transaction — the KR delete fires
its audit, then the Objective delete fires its audit; if the
Objective delete fails validation the KR delete is already applied
(the patch is sequential, not transactional at M4 scope).

## Web walkthrough

1. From the org dashboard, click a project tile → lands on
   `/organizations/[org_id]/projects/[id]`.
2. The header shows the project `name`, `status`, `shape`, and the
   `lead` agent (rendered from the roster panel).
3. The OKR panel surfaces each Objective as a collapsible card; each
   card lists its KRs with current/target values + a progress bar.
4. The "Recent sessions" panel is a static placeholder at M4 ("No
   sessions yet — the first session shipped via `phi session launch`
   will appear here at M5"). C-M5-3 in the base plan flips this to
   real rows.

## Incident playbooks

### OKR patch left the project in an inconsistent state

The patch is applied sequentially — each entry mutates the in-memory
OKR vectors, then the full Project row is upserted. If the upsert
fails mid-patch the store row is unchanged (the upsert is one SurrealQL
statement). The mutations that were already serialised but NOT
applied to storage are discarded. The audit chain is not affected.

If the upsert succeeded but the audit emission failed mid-patch
(e.g. `AUDIT_EMIT_FAILED` on the 3rd of 5 entries), the row holds the
full post-image but the audit chain has only the first N-1 events.
Run the M1 audit-chain repair playbook.

### "Recent sessions" renders empty for a project that had sessions at M5

M4 always returns `recent_sessions: []`. At M5, if the field stays
empty for a project that operators know launched sessions, check:

- Migration 0005 applied (`store/migrations/0005_sessions.surql`).
- `Session.project_id` column populated at session-start (M5's
  `apply_session_start` compound tx).
- The `RUNS_IN` edge from Session → Project exists.

See the M5 runbook §session-panel-empty once M5 ships.

## Monitoring

- Metric `phi_project_detail_requests_total{code}` — 200 / 404 / 403 /
  400 / 500 counters.
- Metric `phi_project_okr_patch_applied_total` — one increment per
  successful patch (irrespective of entry count).
- Metric `phi_project_okr_patch_mutation_total{kind, op}` — one
  increment per entry (4 ops × 2 kinds = 6 combinations actually
  used).

Metrics are emitted via the existing Prometheus layer; no changes to
`rust.yml` CI required at M4/P7.

## Cross-references

- [Architecture — Project Detail](../architecture/project-detail.md).
- [Error-code aggregator — `docs/ops/runbook.md §M4`](../../../../../ops/runbook.md).
- [M4 plan archive §P7](../../../../plan/build/a634be65-m4-agents-and-projects.md).
