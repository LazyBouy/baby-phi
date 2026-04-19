<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Requirements doc — agent self-service surface -->

# a02 — My Auth Requests

## 2. Page Purpose + Primary Actor

Dual-view surface where an Agent manages Auth Requests:

- **Inbound** — Auth Requests where this Agent holds an approver slot. Approve / deny / reconsider / escalate per the per-state access matrix from [permissions/02 § Per-State Access Matrix](../../concepts/permissions/02-auth-request.md#per-state-access-matrix).
- **Outbound** — Auth Requests this Agent has submitted. View state, see approver responses, cancel in non-terminal states.

**Primary actor:** any Agent. Inbound tab relevant when the agent holds ownership or allocation authority over resources (receives approver slots); outbound tab relevant for any agent that has submitted at least one Auth Request.

## 3. Available When

- Any Agent. Visibility of slots / submissions is filtered per grants.

## 4. UI Sketch

```
┌───────────────────────────────────────────────────────────────────┐
│ My Auth Requests — agent:lead-acme-1                              │
├───────────────────────────────────────────────────────────────────┤
│ [Tab: 📥 Inbound (2)  ●]   [Tab: 📤 Outbound (5)]                │
│                                                                    │
│ Inbound — awaiting your approval                                 │
│ ┌──────────────────────────────────────────────────────────────┐ │
│ │ req-9001  from auditor-x-1  [read,list] on 2 sessions        │ │
│ │ state: In Progress           your slot: Unfilled             │ │
│ │ [View full] [Approve my slot] [Deny my slot] [Escalate]      │ │
│ ├──────────────────────────────────────────────────────────────┤ │
│ │ req-9017  from coder-acme-3  [read] on memory-pool          │ │
│ │ state: Pending               your slot: Unfilled             │ │
│ │ [View full] [Approve my slot] [Deny my slot] [Escalate]      │ │
│ └──────────────────────────────────────────────────────────────┘ │
└───────────────────────────────────────────────────────────────────┘
```

```
Outbound tab
┌──────────────────────────────────────────────────────────────────┐
│ req-8044  (self)    [allocate] on workspace       Approved       │
│ req-8091  (self)    [read] on session:s-9830      Partial        │
│ req-8113  (self)    [transfer] of /workspace/foo  Pending        │
│   [View]  [Cancel]                                                │
└──────────────────────────────────────────────────────────────────┘
```

## 5. Read Requirements

- **R-AGENT-a02-R1:** Inbound view SHALL list every Auth Request where the viewing Agent holds an approver slot (regardless of state). Group by state; unfilled slots surfaced first.
- **R-AGENT-a02-R2:** For each Auth Request in Inbound, the surface SHALL display: requestor, requested scope + resources (list of slots), the Agent's own slot's state, the full state of the request, and justification text.
- **R-AGENT-a02-R3:** Outbound view SHALL list every Auth Request submitted by the viewing Agent, with state, time submitted, approver responses, and any resulting Grant id on approval.
- **R-AGENT-a02-R4:** A detail view per Auth Request SHALL show the per-resource-slot model (see [permissions/02 § Schema](../../concepts/permissions/02-auth-request.md#schema--per-resource-slot-model)) including each slot's approvers and their individual states.

## 6. Write Requirements

- **R-AGENT-a02-W1 (inbound):** The Agent SHALL be able to **approve** an unfilled slot they hold. Slot state transitions to `Approved`; request-level state may transition per aggregation rules.
- **R-AGENT-a02-W2 (inbound):** The Agent SHALL be able to **deny** an unfilled slot they hold. Slot state → `Denied`.
- **R-AGENT-a02-W3 (inbound):** The Agent SHALL be able to **reconsider** (unfill) a slot they already filled, provided the request has not yet reached a closed terminal state (see [permissions/02 § Multi-Approver Dynamics](../../concepts/permissions/02-auth-request.md#multi-approver-dynamics)). Slot returns to `Unfilled`.
- **R-AGENT-a02-W4 (inbound):** The Agent SHALL be able to **escalate** a slot to a higher authority if they cannot decide. See [permissions/02 § Escalation When Routing Fails](../../concepts/permissions/02-auth-request.md#escalation-when-routing-fails).
- **R-AGENT-a02-W5 (outbound):** The Agent SHALL be able to **cancel** their own submission while the request is in Draft / Pending / In Progress. Cancellation terminates the request without producing a Grant.
- **R-AGENT-a02-W6 (outbound):** The Agent SHALL be able to resubmit-as-new a Denied or Expired request with modified parameters (narrower resources, different justification).

## 7. Permission / Visibility Rules

- **Inbound visibility** — derived from the Auth Request's slot list; an Agent sees only the requests where they hold a slot.
- **Approve / deny / reconsider** — requires slot ownership; the per-state access matrix gates which actions are available given the current state.
- **Escalate** — requires `[allocate]` on the resource OR the router-escalation path defined in [permissions/02 § Escalation](../../concepts/permissions/02-auth-request.md#escalation-when-routing-fails).
- **Outbound visibility** — requestor-side only; the submitting Agent sees their own submissions and can cancel in non-terminal states.
- **Cross-org viewing** — Auth Requests involving cross-org resources show truncated metadata; full details require the cross-org grant.

## 8. Event & Notification Requirements

- **R-AGENT-a02-N1:** When a new Inbound request arrives, the Agent is notified (via Channel for Human Agents; via inbox digest for LLM agents).
- **R-AGENT-a02-N2:** Every slot-fill emits audit event `AuthRequestSlotFilled { auth_request_id, approver, state, ... }`.
- **R-AGENT-a02-N3:** Request-level state transitions (Approved / Denied / Partial / Expired / Revoked / Cancelled) emit their own audit events per [permissions/02 § State Machine](../../concepts/permissions/02-auth-request.md#state-machine).

## 9. Backend Actions Triggered

Approve / deny (W1/W2):
- Slot state update on the Auth Request node.
- Request-level state re-derivation per [permissions/02 § Schema — Per-Resource Slot Model](../../concepts/permissions/02-auth-request.md#schema--per-resource-slot-model) aggregation rules.
- On request reaching Approved (all required slots Approved): Grant materialised (see [system/s04-auth-request-state-transitions.md](../system/s04-auth-request-state-transitions.md)).
- On Denied: request closes; no Grant.

Escalate (W4):
- New approver slot appended (original marked `skipped_due_to_timeout` or `escalated`); see [system/s04](../system/s04-auth-request-state-transitions.md).

Cancel (W5):
- Request transitions to Cancelled; no Grant; audit event.

## 10. API Contract Sketch

```
GET  /api/v0/agents/{agent_id}/auth-requests/inbound?state=...
     → 200: { requests: [...] }

GET  /api/v0/agents/{agent_id}/auth-requests/outbound?state=...
     → 200: { requests: [...] }

POST /api/v0/auth-requests/{id}/slots/{my_slot_id}/approve
POST /api/v0/auth-requests/{id}/slots/{my_slot_id}/deny
POST /api/v0/auth-requests/{id}/slots/{my_slot_id}/reconsider
POST /api/v0/auth-requests/{id}/slots/{my_slot_id}/escalate
     → 200: { new_slot_state, request_state, audit_event_id }

POST /api/v0/auth-requests/{id}/cancel          (requestor-only)
     → 200: { cancelled_at, audit_event_id }
```

## 11. Acceptance Scenarios

**Scenario 1 — partial approval produces partial grant.**
*Given* a request where `lead-acme-1` holds slots on two resources — `session:s-9831` and `session:s-9832` — and mirrors [permissions/08 § Step 9](../../concepts/permissions/08-worked-example.md#step-9-ad-hoc-auth-request-and-revocation-cascade), *When* `lead-acme-1` approves the first slot and denies the second, *Then* the request transitions to Partial and a Grant is issued covering only `session:s-9831`.

**Scenario 2 — reconsider before closed.**
*Given* a request is In Progress (some slots filled, some pending), *When* `lead-acme-1` re-opens their previously-Approved slot via Reconsider, *Then* the slot returns to Unfilled, the request remains In Progress, and the audit event `AuthRequestSlotReconsidered` is emitted.

**Scenario 3 — cross-org joint-research approval.**
*Given* [projects/03-joint-research.md](../../projects/03-joint-research.md) produces an Auth Request with both `lead-acme-1` and `lead-beta-1` as slot-holders for a shared session, *When* each approves independently, *Then* the request transitions to Approved and the Grant covers the session per the per-co-owner rule in [permissions/06 § Co-Ownership × Multi-Scope rule 6](../../concepts/permissions/06-multi-scope-consent.md#co-ownership--multi-scope-session-access).

## 12. Cross-References

**Concept files:**
- [concepts/permissions/02 § Auth Request Lifecycle](../../concepts/permissions/02-auth-request.md#auth-request-lifecycle) — the full state machine and per-state access matrix.
- [concepts/permissions/02 § Per-State Access Matrix](../../concepts/permissions/02-auth-request.md#per-state-access-matrix) — gates used here.
- [concepts/permissions/02 § Escalation When Routing Fails](../../concepts/permissions/02-auth-request.md#escalation-when-routing-fails).
- [concepts/permissions/06 § Co-Ownership × Multi-Scope](../../concepts/permissions/06-multi-scope-consent.md#co-ownership--multi-scope-session-access).

**Related agent-self-service pages:**
- [a01-my-inbox-outbox.md](a01-my-inbox-outbox.md) — cross-refs new Auth Request notifications delivered to inbox.

**Related system flows:**
- [system/s04-auth-request-state-transitions.md](../system/s04-auth-request-state-transitions.md) — the state-machine engine this page interacts with.

**Project / org layouts exercised:**
- [projects/03-joint-research.md](../../projects/03-joint-research.md), [concepts/permissions/08-worked-example.md § Step 9](../../concepts/permissions/08-worked-example.md#step-9-ad-hoc-auth-request-and-revocation-cascade).
