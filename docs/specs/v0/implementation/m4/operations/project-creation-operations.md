<!-- Last verified: 2026-04-23 by Claude Code -->

# Operations — Project Creation (page 10)

**Status: [EXISTS]** — landed at M4/P6.

## Endpoints

| Method | Path | Use |
|---|---|---|
| `POST` | `/api/v0/orgs/:org_id/projects` | Submit a new project (Shape A: immediate, returns 201; Shape B: pending AR, returns 202). |
| `POST` | `/api/v0/projects/_pending/:ar_id/approve` | Drive one approver slot on a Shape B pending AR. Returns 200 (still_pending or terminal). |

Both gated by `AuthenticatedSession` (cookie `phi_kernel_session`).

### Create wire shape

```json
{
  "project_id": "<uuid>",
  "name": "Atlas",
  "description": "...",
  "goal": "ship the prototype",
  "shape": "shape_a" | "shape_b",
  "co_owner_org_id": "<uuid>"                // required iff shape = shape_b
  ,
  "lead_agent_id": "<uuid>",
  "member_agent_ids": ["<uuid>", ...],
  "sponsor_agent_ids": ["<uuid>", ...],
  "token_budget": 1000000,                   // optional per-project cap
  "objectives": [Objective, ...],
  "key_results": [KeyResult, ...],
  "resource_boundaries": ResourceBoundaries? // optional
}
```

OKR shapes match `domain/src/model/composites_m4.rs::{Objective, KeyResult, MeasurementType, OkrValue}`. The server re-validates `measurement_type` vs `target_value`/`current_value` shape and returns `400 OKR_VALIDATION_FAILED` on mismatch.

### Approve wire shape

```json
{
  "approver_id": "<uuid>",
  "approve": true | false
}
```

## Response codes

### Create

| HTTP | `code` | Meaning |
|---|---|---|
| 201 | `{outcome: "materialised", ...}` | Shape A — project created + Template A grant fired via event bus. |
| 202 | `{outcome: "pending", pending_ar_id, approver_ids, audit_event_id}` | Shape B — awaiting both co-owner admins. Next step: `phi project approve-pending`. |
| 400 | `VALIDATION_FAILED` | Empty name, etc. |
| 400 | `OKR_VALIDATION_FAILED` | KR references unknown objective; measurement-type shape mismatch. |
| 400 | `SHAPE_A_HAS_CO_OWNER` | Shape A submit supplied `co_owner_org_id`. |
| 400 | `SHAPE_B_MISSING_CO_OWNER` | Shape B submit omitted `co_owner_org_id`. |
| 400 | `CO_OWNER_INVALID` | Co-owner org doesn't exist, OR equals primary owner, OR has no Human agent to act as approver. |
| 400 | `LEAD_NOT_IN_OWNING_ORG` | Lead's `owning_org` isn't one of the owning orgs for this shape. |
| 400 | `MEMBER_INVALID` | Member or sponsor agent id doesn't exist / doesn't belong. |
| 404 | `ORG_NOT_FOUND` | `org_id` path segment unknown. |
| 404 | `LEAD_NOT_FOUND` | `lead_agent_id` unknown. |
| 409 | `PROJECT_ID_IN_USE` | Someone already created this project id. |
| 500 | `AUDIT_EMIT_FAILED` | Compound tx committed but follow-up audit emit failed. Project IS persisted. |
| 500 | `INTERNAL_ERROR` | Unhandled repository error. |

### Approve

| HTTP | `code` | Meaning |
|---|---|---|
| 200 | `{outcome: "still_pending", ar_id}` | Slot transitioned; other approver still pending. |
| 200 | `{outcome: "terminal", state: "approved" \| "denied" \| "partial", project_id}` | Both approvers decided. `project_id` is null at M4 (see C-M5-6). |
| 400 | `PENDING_AR_NOT_SHAPE_B` | AR id resolves but isn't a Shape B project-creation AR. |
| 400 | `TRANSITION_ILLEGAL` | State machine refused the transition (e.g. slot already in Approved state — no-op). |
| 403 | `APPROVER_NOT_AUTHORIZED` | Caller isn't listed as one of the two slot approvers. |
| 404 | `PENDING_AR_NOT_FOUND` | Unknown AR id. |
| 409 | `PENDING_AR_ALREADY_TERMINAL` | Another approver already drove the AR to a terminal state. |

## The 4-outcome decision matrix (Shape B)

| Slot 1 | Slot 2 | AR terminal | Project materialises? | Audit emitted at terminal |
|---|---|---|---|---|
| Approve | Approve | `approved` | Yes (M5/C-M5-6 — M4 returns `project_id: null`) | none at M4 |
| Approve | Deny | `partial` | No | `platform.project_creation.denied` |
| Deny | Approve | `partial` | No | `platform.project_creation.denied` |
| Deny | Deny | `denied` | No | `platform.project_creation.denied` |

Pinned at three tiers: 50-case proptest at the domain state-machine, acceptance tests against the live HTTP stack, and a pure decision helper `should_materialize_project_from_ar_state`.

## Audit-event shapes

**`platform.project.created` (Alerted, Shape A):**

```json
{
  "event_type": "platform.project.created",
  "audit_class": "alerted",
  "diff": {
    "before": null,
    "after": {
      "project_id": "...",
      "name": "Atlas",
      "shape": "shape_a",
      "status": "planned",
      "owning_org": "...",
      "co_owner_orgs": [],
      "lead_agent_id": "...",
      "objectives_count": N,
      "key_results_count": M
    }
  }
}
```

**`platform.project_creation.pending` (Logged, Shape B submit):**

```json
{
  "event_type": "platform.project_creation.pending",
  "audit_class": "logged",
  "diff": {
    "after": {
      "auth_request_id": "...",
      "shape": "shape_b",
      "proposed_name": "Atlas",
      "co_owner_orgs": ["..."],
      "approver_agent_ids": ["...", "..."]
    }
  }
}
```

**`platform.project_creation.denied` (Alerted, Shape B terminal non-Approved):**

```json
{
  "event_type": "platform.project_creation.denied",
  "audit_class": "alerted",
  "diff": {
    "after": {
      "auth_request_id": "...",
      "proposed_name": "Atlas",
      "denying_approvers": [
        {"agent_id": "...", "reason": null}
      ]
    }
  }
}
```

## Playbook — Shape B approval deadlock

Symptoms: a pending AR sits in `Pending` state for days; no slot has been driven.

1. **Identify the non-responding approver.** `GET /api/v0/projects/_pending/:ar_id` (read-only inspection — lands at M5 alongside C-M5-6's sidecar table) or query storage directly:
   ```sql
   SELECT id, requestor, resource_slots, submitted_at FROM auth_request
     WHERE kinds CONTAINS '#shape:b:project_creation' AND state = 'pending';
   ```
2. **Two paths:**
   - **Cancel the pending AR** — the requestor can call `auth_requests::transitions::cancel` (M5 surfaces a CLI for this); the AR moves to `Cancelled`, project is never materialised. Operator resubmits with a different co-owner.
   - **Wait for natural timeout** — the AR's `active_window_days` is 30; beyond that the expiry sweep (M5+) will transition it to `Expired`.

## Playbook — Template A grant didn't fire after a Shape A create

Symptoms: project persisted, `platform.project.created` audit emitted, but no `TemplateAAdoptionFired` audit and no lead grant.

1. **Check the event bus wiring.** In the boot log, `phi-server` emits `tracing::info!` for each `event_bus.subscribe` call — confirm the `TemplateAFireListener` was registered.
2. **Check the listener's internal error log.** Listener errors are logged with `event_id` (matches the `audit_event_id` from the create response) but don't rewind the tx — the project is durable even if the grant didn't land. Query `SELECT * FROM audit_event WHERE event_type = 'template_a.adoption.fired' AND org_scope = <org>` — if absent, the listener errored.
3. **Manual replay.** Re-trigger the listener by calling the pure-fn `fire_grant_on_lead_assignment` with the recovered `(lead, project)` pair via a one-off SurrealQL write. M5 adds a proper replay endpoint.

## CLI equivalents

```bash
# Shape A
phi project create --org-id <uuid> --name Atlas --shape shape_a \
    --lead-agent-id <uuid> --member-ids <uuid1>,<uuid2>

# Shape A with OKRs from a file
phi project create --org-id <uuid> --name Atlas --shape shape_a \
    --lead-agent-id <uuid> --okrs-file ./atlas-okrs.json

# Shape B submit
phi project create --org-id <uuid> --name Atlas --shape shape_b \
    --co-owner-org-id <other-uuid> --lead-agent-id <uuid>

# Shape B approve (each co-owner admin runs this)
phi project approve-pending --ar-id <uuid> --approver-id <uuid>

# Shape B deny
phi project approve-pending --ar-id <uuid> --approver-id <uuid> --deny

# Machine-readable
phi project create ... --json | jq '.outcome'
```

Exit codes: `0` success, `65` (`EXIT_REJECTED`) on 4xx, `68` (`EXIT_TRANSPORT`) on connection failure.

## phi-core leverage notes

This page surfaces **zero phi-core fields** — the create + approve endpoints return governance data only (project ids, audit event ids, AR state strings, approver ids). The Project composite + OKRs + ResourceBoundaries + ProjectShape are all pure phi governance types; phi-core has no planning / project concept. The Template A grant emitted on Shape A materialisation is a `Grant` node (baby-phi governance), not a phi-core type.

Positive close-audit:

```bash
grep -En '^use phi_core::' modules/crates/server/src/platform/projects/create.rs
# → 0 lines by design
```

## References

- [Project creation architecture](../architecture/project-creation.md)
- [Shape A vs Shape B](../architecture/shape-a-vs-shape-b.md)
- [Event bus + Template A subscription](../architecture/event-bus.md)
- [ADR-0025](../decisions/0025-shape-b-two-approver-flow.md)
- [ADR-0028](../decisions/0028-domain-event-bus.md)
- [Base build plan §C-M5-6](../../../../plan/build/36d0c6c5-build-plan-v01.md) (Shape B materialisation-after-approve follow-up)
- [M4 plan archive §P6](../../../../plan/build/a634be65-m4-agents-and-projects.md)
