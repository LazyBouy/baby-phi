<!-- Last verified: 2026-05-08 by Claude Code (CH-15 P3 chunk — operator playbook for the session-launch hard-deny gate. Pairs with `architecture/session-launch-permission-gate.md` (design) and ADR-0054.) -->

# Session-launch permission gate operations runbook

> **Audience:** SREs and operators debugging session-launch denials post-CH-15. Pair this page with [`session-launch-permission-gate.md`](../architecture/session-launch-permission-gate.md) (design) and [ADR-0054](../decisions/0054-session-launch-manifest-and-hard-deny-flip.md).

---

## Audit-event dictionary entry — `platform.session.launch_denied`

Emitted by `server::platform::sessions::launch::launch_session` whenever the launch-time Permission Check returns `Decision::Denied { failed_step, reason }` for any step 0..6. The handler emits this BEFORE returning `Err(SessionError::PermissionCheckFailed { step, reason })`.

| Field | Type | Canonical? | Notes |
|---|---|---|---|
| `event_type` | string | yes | Always `"platform.session.launch_denied"`. |
| `actor_agent_id` | AgentId | yes | The actor making the launch request (typically the CEO submitting on behalf of the lead). |
| `target_entity_id` | NodeId | yes | Derived from `session_id` — matches `platform.session.started` convention. |
| `audit_class` | enum | yes | Always `Alerted` (60s delivery to org alert channel per `nfr-observability.md`). |
| `org_scope` | OrgId | yes | The owning org of the launch attempt. |
| `provenance_auth_request_id` | optional | n/a | Always `None` — deny is a runtime decision, not a Template fire. |
| `diff.after.session_id` | string (uuid) | yes | Pre-allocated at Step 3.5 even on the deny path so the audit row carries a stable cross-ref. |
| `diff.after.agent_id` | string (uuid) | yes | The launching agent. |
| `diff.after.project_id` | string (uuid) | yes | The launch target project. |
| `diff.after.org_id` | string (uuid) | yes | The owning org. |
| `diff.after.failed_step` | u8 | yes | 0..6 per `FailedStep::as_metric_label()`. |
| `diff.after.reason_kind` | string | yes | DeniedReason variant tag (snake_case, e.g. `no_grants_held`, `manifest_empty`, `consent_declined`). |
| `diff.after.reason_detail` | object | no (operator data) | Optional rich detail; non-canonical (excluded from `canonical_bytes`). |
| `diff.after.emitted_at` | string (rfc3339) | yes | Wall-clock timestamp of the deny. |
| `diff.before` | null | yes | Always `null` (matches the create-style rejection-event convention). |

Sample event:

```json
{
  "event_type": "platform.session.launch_denied",
  "audit_class": "alerted",
  "actor_agent_id": "...",
  "target_entity_id": "...",
  "org_scope": "...",
  "provenance_auth_request_id": null,
  "diff": {
    "before": null,
    "after": {
      "session_id":   "5ad1c1f0-...",
      "agent_id":     "abc-...",
      "project_id":   "def-...",
      "org_id":       "ghi-...",
      "failed_step":  2,
      "reason_kind":  "no_grants_held",
      "emitted_at":   "2026-05-08T..."
    }
  }
}
```

---

## Per-step deny playbook

Each FailedStep variant maps to a numeric step on the wire + a recovery action.

| Step | FailedStep | Likely DeniedReason | Cause | Fix |
|---|---|---|---|---|
| 0 | `Catalogue` | `CatalogueMiss { resource_uri }` | Target_uri references a resource not declared in owning org's `resources_catalogue`. **Should not fire post-CH-15** — the launch builder uses class-level reach (`target_uri = ""`), so Step 0 catalogue-miss now indicates a deeper schema issue. | Investigate via the audit-event `reason_kind` = `catalogue_miss` + check `resources_catalogue` table. |
| 1 | `Expansion` | `ManifestEmpty` | Manifest has no actions OR no resources. The synthetic launch manifest always carries `[Read, Inspect, List]` on `[session_object]` so this should not fire — indicates a builder bug. | File a regression bug; the engine `is_empty()` predicate should never match the launch manifest. |
| 2 | `Resolution` | `NoGrantsHeld` | Most common deny post-CH-15. The lead has NO grants on `session_object` reaches. | Verify Template A adoption AR is approved. Re-emit `HasLeadEdgeCreated` for the project OR run migration `0015_template_a_session_object_grant.surql` to backfill legacy single-grant holders. |
| 2 | `Ceiling` | `CeilingEmptied` | Org/project ceiling clamped every candidate grant to empty. Surfaces as `STEP_2` (sub-step `2a`) on the wire. | Audit ceiling grants on the agent's owning org + project. Most likely a misconfigured ceiling that excludes `session_object`. |
| 3 | `Match` | `NoMatchingGrant { fundamental, action }` | Lead has SOME grants but none match the (fundamental, action) reach. Common when a grant covers `[DataObject]` only but the manifest needs `[Tag]` too (or vice versa). | Verify the paired Template A grants on the lead. Both project-resource (Tag) AND session-object (DataObject + Tag) grants must be present. |
| 4 | `Constraint` | `ConstraintViolation { constraint, grant_id }` | Winning grant violates a manifest-declared constraint. The synthetic launch manifest declares no constraints, so this should not fire. | Investigate; indicates a manifest builder bug. |
| 5 | `Scope` | `ScopeUnresolvable { fundamental, action }` OR `IntersectionEmpty { ... }` | Multi-scope cascade ([ADR-0051](../decisions/0051-multi-scope-cascade-contractor-model.md)) failed to pick a winner. | Investigate session.tags + agent's project memberships. The launching agent must be a member of at least one of the session's tagged projects. |
| 6 | `Consent` | `ConsentDeclined / ConsentRevoked / ConsentExpired / ConsentTimedOutDeny / NoSessionContext` | Step 6 (Consent) deny — routed through `gate_session_launch_consent` per [ADR-0048](../decisions/0048-per-session-consent-gating.md). | Verify the subordinate's consent state. Per CH-11 / ADR-0048, this path was already hard-denying before CH-15 — no change in behaviour at Step 6. |

---

## Common scenarios

### Scenario 1 — Pre-CH-15 deploy: every launch returns 403 STEP_2 NoGrantsHeld

**Symptom:** Post-CH-15 deploy, every existing project-lead's launch returns 403 `PERMISSION_CHECK_FAILED_AT_STEP_2: NoGrantsHeld`.

**Cause:** Migration `0015_template_a_session_object_grant.surql` should have backfilled the paired session-object grant for every existing Template A holder. If it didn't run (deploy ordering issue OR ledger drift), legacy holders are missing the new grant.

**Fix:**

1. Verify migration `0015` ran:
   ```surql
   SELECT version, slug, applied_at FROM _migrations WHERE version = 15;
   ```
   Should return one row.
2. If absent, force-apply via the deploy's migration-runner restart (the runner is idempotent — restart re-applies any missing versions).
3. Verify the paired grants exist:
   ```surql
   SELECT count() FROM grant
     WHERE revoked_at = NONE
       AND string::starts_with(resource_uri, "tags contains project:");
   ```
   Should be ≥ N where N = active Template A leads in your data.

### Scenario 2 — Newly-elected lead's launch returns 403

**Symptom:** A project that promotes a new lead (`HAS_LEAD` edge created) returns 403 on the new lead's launch.

**Cause:** The `TemplateAFireListener` did not run — the `HasLeadEdgeCreated` event wasn't emitted, OR the listener errored silently (per ADR-0028 fail-safe semantics: listener errors are logged + dropped).

**Fix:**

1. Check structured logs for `TemplateAFireListener: create_grant failed` warnings around the lead-promotion timestamp.
2. Manually re-emit the event:
   ```rust
   event_bus.emit(DomainEvent::HasLeadEdgeCreated {
       project, lead, at: Utc::now(), event_id: AuditEventId::new(),
   }).await;
   ```
3. Verify two grants land on the lead via:
   ```surql
   SELECT count() FROM grant
     WHERE holder_id = <lead-uuid>
       AND descends_from = <template-a-adoption-ar>
       AND revoked_at = NONE;
   ```
   Should return 2 (project-resource + session-object).

### Scenario 3 — Audit events spike post-deploy

**Symptom:** `platform.session.launch_denied` audit-event volume spikes after the CH-15 deploy.

**Cause:** Expected. Pre-CH-15 every step-1-to-6 deny was advisory-logged + the launch proceeded; post-CH-15 every deny emits an audit event + 403s. If the spike is sustained beyond an hour, something is wrong (operators should have noticed the 403s in their existing monitoring).

**Fix:**

1. Group the audit events by `failed_step` + `reason_kind`:
   ```surql
   SELECT
       diff.after.failed_step AS step,
       diff.after.reason_kind AS reason,
       count() AS n
   FROM audit_event
   WHERE event_type = 'platform.session.launch_denied'
     AND timestamp > <one-hour-ago>
   GROUP BY step, reason
   ORDER BY n DESC;
   ```
2. The most common bucket should be `(2, no_grants_held)` for legacy-holder catch-up. Other buckets indicate a deeper issue.
3. Cross-reference with the per-step playbook above.

---

## Metrics + dashboards (M7b observability extensions)

At CH-15 (M5.2), the audit event itself is the primary dashboard signal. M7b adds:

- `phi_sessions_launch_denied_total{org_id, failed_step, reason_kind}` counter — derived from the audit event stream.
- `phi_sessions_launch_total{outcome}` counter — `outcome` ∈ `{allowed, denied, pending}`. `denied` is incremented on every 403 from CH-15's hard-deny path.
- Alert rule: `rate(phi_sessions_launch_denied_total[5m]) > 5` per org → page the on-call operator (post-deploy spikes are expected within the first 15 minutes; sustained spikes are a problem).

---

## Cross-references

- [ADR-0054](../decisions/0054-session-launch-manifest-and-hard-deny-flip.md) — design decisions D54.1–D54.8.
- [ADR-0033](../decisions/0033-k8s-prep-refactors.md) §D33.2 — migration runner discipline.
- [ADR-0048](../decisions/0048-per-session-consent-gating.md) — Step 6 consent gating reused by Step 6 deny path.
- [m5_2/architecture/session-launch-permission-gate.md](../architecture/session-launch-permission-gate.md) — design page.
- [m5/operations/session-launch-operations.md](../../m5/operations/session-launch-operations.md) — pre-CH-15 launch operations runbook (error codes, incident playbooks).
- [m5/user-guide/troubleshooting.md](../../m5/user-guide/troubleshooting.md) §"CH-15 amendment" — operator-facing troubleshooting for the hard-deny launch gate.
- [drifts/D4.1.md](../../m5_1/drifts/D4.1.md) — closed at CH-15 chunk-seal.
