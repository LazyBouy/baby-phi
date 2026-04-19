<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Requirements doc — system flow -->

# s04 — Auth Request State Transitions

## Purpose

Implements the Auth Request state machine from [concepts/permissions/02 § State Machine](../../concepts/permissions/02-auth-request.md#state-machine): aggregates per-slot state updates into request-level state, materialises Grants on Approved, cascades revocations, handles timeouts and escalations, and emits corresponding audit events. This is the engine every admin page and agent-self-service page that touches Auth Requests plugs into.

## Trigger

Any of:
- Slot state transitions (Approve / Deny / Reconsider) from agent-self-service pages or admin pages.
- Owner actions (Revoke, Override-Approve, Close-As-Denied) from admin pages.
- `valid_until` timeout (cron-like periodic scan).
- Router unavailability detected by [concepts/permissions/02 § Escalation When Routing Fails](../../concepts/permissions/02-auth-request.md#escalation-when-routing-fails).

## Preconditions

- The Auth Request node exists and is not in a closed terminal state (Approved / Denied / Expired / Revoked / Cancelled).

## Behaviour

- **R-SYS-s04-1:** On slot-fill (Approve / Deny), the flow SHALL update the slot's state, compute the per-resource state per the table in [concepts/permissions/02 § Per-resource state derivation rules](../../concepts/permissions/02-auth-request.md#schema--per-resource-slot-model), and re-derive the request-level state.
- **R-SYS-s04-2:** When the request reaches a terminal state with ≥1 resource Approved, the flow SHALL materialise a Grant covering exactly the Approved-resources subset, per [concepts/permissions/02 § Partial-outcome rule (atomic slots)](../../concepts/permissions/02-auth-request.md#state-machine). The Grant's provenance field structurally points to the Auth Request (via the `DESCENDS_FROM` edge — see [concepts/ontology.md](../../concepts/ontology.md) Governance Wiring).
- **R-SYS-s04-3:** When the request is Revoked (owner action on an Approved / Partial state), the flow SHALL forward-only revoke the Grant and cascade to downstream sub-allocations per the revocation rule. Past actions stand in the audit log; future actions are denied.
- **R-SYS-s04-4:** Timeout scan (runs at least once every 60 seconds): the flow SHALL find Auth Requests whose `valid_until` is passed and transition them to Expired. On Expired with ≥1 Approved resource, a Grant covering those is materialised (per the atomic-slot rule).
- **R-SYS-s04-5:** On router unavailability (detected when the designated router has not responded within the window), the flow SHALL **escalate** per [concepts/permissions/02 § Escalation When Routing Fails](../../concepts/permissions/02-auth-request.md#escalation-when-routing-fails): owner → co-owners → platform admin. The original router's slot is marked `Unfilled` with `skipped_due_to_timeout: true`; a new slot for the escalation target is appended.
- **R-SYS-s04-6:** Every state transition emits an audit event named for the transition (e.g., `AuthRequestSlotFilled`, `AuthRequestApproved`, `AuthRequestPartial`, `AuthRequestExpired`, `AuthRequestRevoked`, `AuthRequestCancelled`, `AuthRequestEscalated`).
- **R-SYS-s04-7:** The flow SHALL be **atomic per transition** — concurrent slot-fills on the same request are serialised; the computed state always reflects all currently-visible slot values.
- **R-SYS-s04-8:** On Grant materialisation, an inbox notification is delivered to the requestor (see [a02-my-auth-requests.md](../agent-self-service/a02-my-auth-requests.md)).

## Side effects

- Grant node writes; `DESCENDS_FROM` edge creation.
- Inbox message deliveries to requestors and newly-added escalation approvers.
- Cascade revocations when a downstream grant's provenance is itself revoked (transitive; walks one level at a time).
- Retention-window timer starts on entering a terminal state (see [s06-periodic-triggers.md](s06-periodic-triggers.md) for archival).

## Failure modes

- **Concurrent slot-fill race** → optimistic concurrency with CAS on the slot's state; second writer receives `409 STALE_SLOT_STATE` and re-reads before retry.
- **Grant materialisation partial failure** → full transaction rollback; request reverts to pre-terminal state; event `AuthRequestGrantMaterialisationFailed` alerted; admin alerted.
- **Revocation cascade loop detection** → the cascade walks at most N levels (N = max_cascade_depth, default 100) to prevent infinite recursion; exceeding triggers `AuthRequestRevocationCascadeTooDeep` alerted event.

## Observability

- Metrics: `baby_phi_auth_request_state_transitions_total{from, to}`, `baby_phi_auth_request_approval_latency_seconds`, `baby_phi_auth_request_timeout_expirations_total`, `baby_phi_grant_materialisations_total`, `baby_phi_revocation_cascades_total`.
- Audit events as listed in R-SYS-s04-6.

## Cross-References

**Concept files:**
- [concepts/permissions/02 § Auth Request Lifecycle](../../concepts/permissions/02-auth-request.md#auth-request-lifecycle) — the full state-machine spec this flow implements.
- [concepts/permissions/02 § Escalation When Routing Fails](../../concepts/permissions/02-auth-request.md#escalation-when-routing-fails).
- [concepts/permissions/04 § The Authority Chain](../../concepts/permissions/04-manifest-and-resolution.md#the-authority-chain) — Grant ↔ Auth Request provenance.

**Admin pages provisioning or consuming this flow:**
- [admin/12-authority-template-adoption.md](../admin/12-authority-template-adoption.md) — template adoptions are Auth Requests.
- [admin/10-project-creation-wizard.md](../admin/10-project-creation-wizard.md) — Shape B submits are Auth Requests.
- Most other admin pages trigger Auth Requests via their W-actions.

**Agent-self-service pages consuming:**
- [a02-my-auth-requests.md](../agent-self-service/a02-my-auth-requests.md) — the inbound/outbound UI sits on top of this flow.
- [a03-my-consent-records.md](../agent-self-service/a03-my-consent-records.md) — consent state transitions reuse this machinery.

**Related flows:**
- [s05-template-adoption-grant-fires.md](s05-template-adoption-grant-fires.md) — template-fired grants traverse this flow at template-adoption time.
- [s06-periodic-triggers.md](s06-periodic-triggers.md) — retention-window archival.
