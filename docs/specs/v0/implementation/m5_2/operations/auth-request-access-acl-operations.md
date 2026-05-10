<!-- Last verified: 2026-05-09 by Claude Code (CH-18 P3 — NEW operations doc, cycle hex c77937bc) -->

# AuthRequest per-state ACL — operations runbook

> **Audience:** SREs and operators debugging AR-access denials post-CH-18. Pair this page with [`auth-request-access-acl.md`](../architecture/auth-request-access-acl.md) (design) and [ADR-0056](../decisions/0056-auth-request-per-state-acl-enforcement.md).

---

## §1 — Audit-event dictionary entry — `auth_request.access_denied`

Emitted by every AR-mutation handler whenever `domain::auth_requests::access::check_auth_request_access` returns `Err(AuthRequestAccessError::*)` against a principal not authorised for the (state × intended_op) cell of concept doc 02's per-state matrix. The handler emits BEFORE returning the typed `*Error::AccessDenied(...)` to the caller per ADR-0056 §D56.6.

| Field | Type | Canonical? | Notes |
|---|---|---|---|
| `event_type` | string | yes | Always `"auth_request.access_denied"`. |
| `actor_agent_id` | AgentId | yes | The actor making the AR-touching request (typically the slot approver / requestor / revoker). |
| `target_entity_id` | NodeId | yes | Derived from the target AR id via `NodeId::from_uuid(*target_ar.as_uuid())`. |
| `audit_class` | enum | yes | Always `Alerted` (60s delivery to org alert channel per `nfr-observability.md`; retention ≥ 365 days). |
| `org_scope` | OrgId | yes | The owning org of the target AR. |
| `provenance_auth_request_id` | optional | yes | **Always `Some(target_ar)`** — divergence from `frozen_tag_write_rejected` which is always `None`. AR access denial has AR provenance; frozen-tag rejection does not. |
| `diff.after.auth_request_id` | string (uuid) | yes | The target AR's id. |
| `diff.after.intended_op` | string | yes | snake_case `IntendedOp` label (`approve` / `deny` / `revoke` / `modify` / `read` / `submit` / `cancel` / `reconsider` / `override_approve` / `close_as_denied` / `expire`). |
| `diff.after.error_kind` | string | yes | snake_case `AuthRequestAccessError` variant tag (`not_authorised_for_modify` / `not_authorised_for_read` / `operation_forbidden_in_state` / `requestor_only_operation` / `unfilled_approver_slot_only`). |
| `diff.after.principal_kind` | string | yes | Wire-form classification (`agent` / `user` / `organization` / `project` / `system` / `unspecified`). The two `NotAuthorised*` variants carry the principal kind directly; the three state-shape rejections carry `"unspecified"`. |
| `diff.after.attempted_at` | string (rfc3339) | yes | Wall-clock timestamp of the deny. |
| `diff.before` | null | yes | Always `null` (matches the create-style rejection-event convention). |

### Sample event

```json
{
  "event_type": "auth_request.access_denied",
  "audit_class": "alerted",
  "actor_agent_id": "abc-...",
  "target_entity_id": "5ad1c1f0-...",
  "org_scope": "ghi-...",
  "provenance_auth_request_id": "5ad1c1f0-...",
  "diff": {
    "before": null,
    "after": {
      "auth_request_id": "5ad1c1f0-...",
      "intended_op":     "approve",
      "error_kind":      "not_authorised_for_modify",
      "principal_kind":  "agent",
      "attempted_at":    "2026-05-09T12:00:00Z"
    }
  }
}
```

### Retention + delivery

- **Retention**: 365+ days (Alerted-class default per `nfr-observability.md`).
- **Delivery**: Streamed to the owning org's alert channel within 60 seconds.
- **Storage**: `audit_events` SurrealDB table; `event_type` is FLEXIBLE TYPE per migration `0001_initial.surql` — no migration needed for CH-18.
- **Hash chain**: `canonical_bytes()` excludes `prev_event_hash` (per CH-12 + CH-15 precedent); chain symmetry preserved.

---

## §2 — Expected emission rate

CH-18's emission frequency is **low-volume**:

- **4 mutation callsites** (`templates/{approve,deny,revoke}.rs` + `projects/create.rs` slot-fill mutation) emit one event per principal-mismatch denial.
- **8 submit-side defence-in-depth callsites** (rows #9–#16 of the wiring map) emit on Err — but all 8 are `requestor == input.actor` by construction, so emission is unreachable under normal callsites; emission would surface only if a future refactor decoupled requestor from input.actor.
- **1 slot-fill READ callsite** (`projects/create.rs:636`) — emits on principal mismatch (the approver_id assertion before the slot-fill mutation).
- **List-side post-filter callsites** (`dashboard.rs:273,293`, `show.rs:63`) — **NO emission per filtered entry** per F3.B.list-filter.a. Silent filter; no audit-event volume.

**Volumetric prediction at v2**: < 1 event per minute per org under normal operation. Spikes indicate either (a) a misconfigured agent attempting actions outside its authorised slot, or (b) a UI bug where dashboard entries are not silently filtered before the user clicks "approve".

### Why list-side reads do NOT emit

Per F3.B.list-filter.a: dashboard rendering would multiply audit-event volume by ~10× per non-admin viewer per render. The matrix cell `Pending × Read by Other Agent → Err` is silently honoured by hiding the AR from the list (matches the existing `agent.archived_at IS NOT NULL` filter UX). The 4 mutation callsites DO emit on Err — those are explicit-action paths where the viewer is asserting an action; silent denial there would lose the audit trail.

---

## §3 — Troubleshooting playbook

### Symptom 1 — Agent X is getting access-denied on AR Y

**Symptom:** Operator reports: "Agent X tried to approve / deny / revoke AR Y and got 4xx `ACCESS_DENIED`."

**Diagnosis steps:**

1. Look up the `auth_request.access_denied` audit event for the AR — filter by `target_entity_id == ar.id` + `actor_agent_id == X.id`. The `error_kind` field tells you which deny class fired:
   - `not_authorised_for_modify` → X is not the requestor, not a slot-approver, not bootstrap. Most common case.
   - `unfilled_approver_slot_only` → X owns a slot but has already filled it; `Reconsider` is the legal op, not `Approve` / `Deny`.
   - `operation_forbidden_in_state` → AR is in a state where the op is structurally illegal (e.g., `Submit` on Approved). Likely a UI bug allowing the action button.
   - `requestor_only_operation` → X is not the AR's requestor; the op is requestor-gated (e.g., Cancel on Pending).
   - `not_authorised_for_read` → list-filter is silent; this fires only on the slot-fill READ assert at `projects/create.rs:636`.

2. Look up the AR's `resource_slots[*].approvers[*].approver` field — verify whether X is in any slot.

3. Look up the AR's `requestor` field — verify whether X is the requestor.

**Fix:**

- If X SHOULD be a slot-approver but isn't, re-run the AR's slot-creation flow (typical pattern: re-fire the Template adoption AR; the listener mints the missing slot grants).
- If X IS a slot-approver but `error_kind == not_authorised_for_modify`, X has already filled the slot — the legal op is `Reconsider` (re-edit own slot), not `Approve` / `Deny`.
- If the user is an admin who SHOULD have read-bypass, check `D-CH18-FOLLOWUP-01` — admin/auditor role-discrimination is deferred to M6+; admin reads on other-agent ARs are denied at v2.

### Symptom 2 — Viewer X sees fewer ARs in the dashboard list than expected

**Symptom:** Operator reports: "I see 3 pending ARs in the dashboard but the DB has 5."

**Diagnosis:**

The dashboard list endpoint (`server::platform::orgs::dashboard::list_active_auth_requests_for_org` + `list_adoption_auth_requests_for_org`) applies a silent post-filter per F3.B.list-filter.a. Non-requestor / non-slot-approver / non-bootstrap viewers see strictly fewer ARs. This is **intentional behaviour** per ADR-0056 §D56.5 + §D56.6 (silent filter — no audit-event per filtered entry).

**Common cause**: viewer is an Admin-role agent who does not own slots on the missing ARs. Per `D-CH18-FOLLOWUP-01`, admin/auditor read-bypass is deferred to M6+; CH-18 ships the four-classifier partition (`Requestor`, `SlotApprover`, `Bootstrap`, `OtherAgent`) and silently denies admin reads on other-agent ARs.

**Fix:**

- If the viewer should see all ARs (compliance-audit use case), this is the `D-CH18-FOLLOWUP-01` deferral — track at M6+ admin-classifier wiring.
- If the viewer is the AR's requestor or a slot-approver but the AR is still hidden, verify the `ar.requestor` matches the viewer's `AgentId` exactly OR the viewer's `AgentId` matches at least one entry in `ar.resource_slots[*].approvers[*].approver`. PrincipalRef equality is strict (per F1.B `PartialEq` derive) — Agent IDs must match by UUID.

### Symptom 3 — `auth_request.access_denied` events spike post-deploy

**Symptom:** Audit-event volume for `auth_request.access_denied` jumps from < 1/min to > 10/min after a deploy.

**Diagnosis:**

CH-18 introduces emission at the 4 mutation callsites + the 1 slot-fill READ assert. A spike indicates either:
- A UI bug allowing the "approve" / "deny" / "revoke" button to render for non-slot-approvers (the silent list-filter should have hidden the AR before the click). Check the dashboard list-filter is wired correctly — `viewer_agent_id` must be plumbed through `dashboard.rs::compute_dashboard_summary` to the `list_active_auth_requests_for_org` post-filter.
- A bot / automation invoking AR mutations with the wrong principal. Check `actor_agent_id` distribution in the audit events.
- The slot-fill READ assert at `projects/create.rs:636` firing on legitimate non-slot-approver flows. This SHOULD NOT happen under typical Shape B project creation; if it fires, the slot-locator may be returning the wrong slot.

**Fix:**

- Verify the dashboard list-filter is wired (see Symptom 2 diagnosis path).
- Audit `actor_agent_id` distribution — bots invoking with wrong principal need to use the correct agent context.
- File a bug if `projects/create.rs:636` fires on legitimate flows.

---

## §4 — Typed-error reference

`AuthRequestAccessError` has 5 DENY-class variants. Each maps to an operator-facing narrative:

| Variant | Wire `error_kind` | Operator narrative | Common cause | Fix |
|---|---|---|---|---|
| `NotAuthorisedForRead` | `not_authorised_for_read` | "Principal kind X is not authorised to read auth request in state Y." | Non-requestor, non-slot-approver, non-bootstrap principal attempting Read. Fires at the slot-fill READ assert; silent at list-side. | If admin SHOULD have read-bypass: tracked in `D-CH18-FOLLOWUP-01` (M6+). Otherwise: principal does not have legitimate access — the request is correctly denied. |
| `NotAuthorisedForModify` | `not_authorised_for_modify` | "Principal kind X is not authorised to perform OP on auth request in state Y." | Most common deny. Non-requestor / non-slot-approver attempting `Approve` / `Deny` / `Revoke` / `Modify`. | Verify principal is in `ar.resource_slots[*].approvers[*]` for slot ops or is `ar.requestor` for owner ops. |
| `OperationForbiddenInState` | `operation_forbidden_in_state` | "Operation OP is forbidden on auth request in state Y." | The cell is structurally empty in the matrix regardless of principal — e.g., `Submit` on `Approved`, `Modify` on `Cancelled`. | Likely UI bug allowing illegal-state action button. Check the AR's state transition graph; the action should not have been offered. |
| `RequestorOnlyOperation` | `requestor_only_operation` | "Operation OP on auth request in state Y is permitted only to the requestor." | Non-requestor attempting requestor-gated op (Submit, Cancel, Modify on Draft). | Verify principal matches `ar.requestor` exactly. PrincipalRef equality is strict (F1.B `PartialEq` derive). |
| `UnfilledApproverSlotOnly` | `unfilled_approver_slot_only` | "Operation OP on auth request in state Y requires the principal to own an unfilled approver slot." | Slot-approver who has already filled their slot attempts `Approve` / `Deny` again. | Legal op for filled-slot approver is `Reconsider` (re-edit own slot until closed-terminal). Use the Reconsider endpoint, not Approve / Deny. |

---

## §5 — Metrics

CH-18 introduces **no new metrics**. The `auth_request.access_denied` audit-event volume is observable via the existing audit-event log + alert-channel delivery; an operator can derive denial rate from event timestamps without a new Prometheus metric.

**Recommended for M7b ops-hardening** (per the M7b "production hardening" milestone scope):

- Per-org `auth_request_access_denied_total` counter, tagged by `error_kind` + `intended_op`. Useful for dashboard visualization of denial-rate trends.
- Per-org `dashboard_silent_filter_skipped_count` counter — track how many ARs are silently filtered per dashboard render. Currently invisible at v2; would surface UX-impacting filter aggressiveness.
- Per-handler `auth_request_access_denied_latency_seconds` histogram — measure the predicate's wall-clock cost in the request path. Currently invisible; would catch a regression where the predicate becomes a bottleneck.

These are not blockers for CH-18; M7b ops-hardening can wire them via the existing `metrics` crate hooks once the M7b telemetry foundation lands.

---

## Cross-references

- Architecture page: [`auth-request-access-acl.md`](../architecture/auth-request-access-acl.md).
- ADR: [ADR-0056](../decisions/0056-auth-request-per-state-acl-enforcement.md).
- Concept doc: [`concepts/permissions/02-auth-request.md`](../../../concepts/permissions/02-auth-request.md) §"Per-State Access Matrix".
- Closed drift: [`D-new-12`](../../m5_1/drifts/D-new-12.md).
- Filed drifts: [`D-CH18-FOLLOWUP-01`](../../m5_1/drifts/D-CH18-FOLLOWUP-01-admin-auditor-role-discrimination.md) + [`D-CH18-FOLLOWUP-02`](../../m5_1/drifts/D-CH18-FOLLOWUP-02-adopt-rs-submit-side-wiring.md).
- Sister operations runbooks:
  - [`session-launch-permission-gate-operations.md`](session-launch-permission-gate-operations.md) — CH-15 / ADR-0054 hard-deny precedent.
  - [`session-live-stream-operations.md`](session-live-stream-operations.md) — CH-17 / ADR-0055 silent-filter precedent.
- Audit-event builder: [`domain/src/audit/events/m5_2/auth_request_access.rs`](../../../../../../modules/crates/domain/src/audit/events/m5_2/auth_request_access.rs).
- Top-level runbook: [`docs/ops/runbook.md`](../../../../../../docs/ops/runbook.md) (operator-facing aggregated index; appended at M5/P9).
