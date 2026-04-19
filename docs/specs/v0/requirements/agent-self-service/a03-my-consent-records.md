<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Requirements doc — agent self-service surface -->

# a03 — My Consent Records

## 2. Page Purpose + Primary Actor

An Agent's surface for managing their own Consent records: acknowledge requested consents, decline them, or revoke previously-acknowledged ones (forward-only per [permissions/06 § Consent Revocation Semantics](../../concepts/permissions/06-multi-scope-consent.md#consent-revocation-semantics)).

**Primary actor:** any Agent (usually a subordinate whose session is about to be read by a supervisor). For co-owned projects the Agent may hold consents scoped to multiple orgs (see [permissions/06 § rule 6](../../concepts/permissions/06-multi-scope-consent.md#co-ownership--multi-scope-session-access)).

## 3. Available When

- The Agent's org has `consent_policy: one_time` or `per_session` (the `implicit` policy auto-acknowledges consents without a user action; an agent under `implicit` policy may still see the page but it will be near-empty).

## 4. UI Sketch

```
┌────────────────────────────────────────────────────────────────┐
│ My Consents — agent:joint-acme-5                                │
├────────────────────────────────────────────────────────────────┤
│ Scoped to: acme  (implicit — auto-acknowledged)                │
│ Scoped to: beta-corp (one_time — needs your acknowledgement)   │
│                                                                 │
│ Pending requests (1)                                           │
│ ┌───────────────────────────────────────────────────────────┐ │
│ │ Beta's Template A one-time consent   status: Requested    │ │
│ │ Requested 2026-04-14  by lead-beta-1                      │ │
│ │ Covers actions: [read, inspect] on your sessions          │ │
│ │ [Acknowledge]  [Decline]                                  │ │
│ └───────────────────────────────────────────────────────────┘ │
│                                                                 │
│ Acknowledged (2)                                               │
│ ┌───────────────────────────────────────────────────────────┐ │
│ │ Acme implicit consent        Acknowledged (auto) 2026-04-01│ │
│ │ Beta per-session consent for s-9931  Acknowledged 2026-04-10│ │
│ │   [Revoke]                                                │ │
│ └───────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────┘
```

## 5. Read Requirements

- **R-AGENT-a03-R1:** The surface SHALL list all Consent records scoped to the viewing Agent, grouped by `scope.org` (since each co-owner org's policy is evaluated independently per [permissions/06 § rule 6](../../concepts/permissions/06-multi-scope-consent.md#co-ownership--multi-scope-session-access)).
- **R-AGENT-a03-R2:** Each entry SHALL show: issuing org, policy kind (`implicit` / `one_time` / `per_session`), covered templates + actions, current state, timestamps (requested_at, responded_at, revoked_at if any).
- **R-AGENT-a03-R3:** The surface SHALL display upcoming consent requests (those in `Requested` state the agent has not yet acted on).

## 6. Write Requirements

- **R-AGENT-a03-W1:** The Agent SHALL be able to Acknowledge a `Requested` consent. State transitions to `Acknowledged`; responded_at is set.
- **R-AGENT-a03-W2:** The Agent SHALL be able to Decline a `Requested` consent. State transitions to `Declined`; any supervisor request that depended on this consent returns `Pending` or `Denied` per the concept rules.
- **R-AGENT-a03-W3:** The Agent SHALL be able to Revoke an `Acknowledged` consent (if `revocable: true` on the Consent node). State transitions to `Revoked`; revoked_at is set. Revocation is forward-only: past actions covered by the consent stand in the audit log; future actions require a fresh Acknowledged consent.
- **R-AGENT-a03-W4:** Acknowledge / Decline / Revoke are idempotent — repeating the same action on the same state is a no-op with a soft-success response.

## 7. Permission / Visibility Rules

- **Page access** — the Agent SHALL see only their own Consent records. Other agents have no visibility into this Agent's consents.
- **Acknowledge / Decline / Revoke** — the subordinate Agent only. Org admins cannot override — they may create a new Auth Request with different semantics, but cannot silently fill another Agent's consent.

## 8. Event & Notification Requirements

- **R-AGENT-a03-N1:** New Requested consents trigger a notification via the Agent's Channel (Human) or inbox (LLM).
- **R-AGENT-a03-N2:** Acknowledge / Decline / Revoke emit audit events (`ConsentAcknowledged`, `ConsentDeclined`, `ConsentRevoked`) with the new state and revoked_at/responded_at fields.
- **R-AGENT-a03-N3:** Revocation triggers a cascade notification to any supervisor currently relying on that consent — their next read will return Pending/Denied.

## 9. Backend Actions Triggered

- Ack / Decline / Revoke (W1–W3): state transitions on the Consent node; audit events.
- On Revoke: any in-flight supervisor reads depending on this consent are blocked at the Permission Check's Step 6 (consent gating) on next invocation. Past reads stand.
- Cross-reference: [system/s04-auth-request-state-transitions.md](../system/s04-auth-request-state-transitions.md) for the full revocation cascade wiring.

## 10. API Contract Sketch

```
GET  /api/v0/agents/{agent_id}/consents
     → 200: { pending: [...], acknowledged: [...], declined: [...], revoked: [...] }

POST /api/v0/agents/{agent_id}/consents/{consent_id}/acknowledge
POST /api/v0/agents/{agent_id}/consents/{consent_id}/decline
POST /api/v0/agents/{agent_id}/consents/{consent_id}/revoke
     → 200: { new_state, responded_at | revoked_at, audit_event_id }
     → 403: not the consent's owning agent
```

## 11. Acceptance Scenarios

**Scenario 1 — ack Beta's one-time consent in joint-research.**
*Given* [projects/03-joint-research.md](../../projects/03-joint-research.md) and [organizations/07-joint-venture-beta.md](../../organizations/07-joint-venture-beta.md) (consent_policy: one_time), and `joint-acme-5` is a member of this co-owned project, *When* `lead-beta-1` tries to read `joint-acme-5`'s session for the first time, a Consent request is created in `joint-acme-5`'s queue, *When* `joint-acme-5` acknowledges it on this page, *Then* the state transitions to `Acknowledged`, `lead-beta-1`'s supervisor read proceeds, and `ConsentAcknowledged` is audit-logged.

**Scenario 2 — revoke cascades forward.**
*Given* `joint-acme-5` previously acknowledged Beta's one-time consent, and `lead-beta-1` has read 5 of their sessions with that consent, *When* `joint-acme-5` revokes the consent, *Then* state → Revoked; the past 5 reads remain in the audit log; `lead-beta-1`'s next read returns `Pending(awaiting_consent=(joint-acme-5, beta-corp))` until a fresh consent is acknowledged.

**Scenario 3 — implicit policy needs no action.**
*Given* `joint-acme-5`'s Acme-side consent is `implicit` (per [organizations/06-joint-venture-acme.md](../../organizations/06-joint-venture-acme.md)), *When* the agent views this page, *Then* the Acme-scoped row shows "Acknowledged (auto) <creation_date>" with no actionable buttons (revocation is still available but uncommon for implicit policies).

## 12. Cross-References

**Concept files:**
- [concepts/permissions/06 § Consent Policy (Organizational)](../../concepts/permissions/06-multi-scope-consent.md#consent-policy-organizational).
- [concepts/permissions/06 § Consent Lifecycle](../../concepts/permissions/06-multi-scope-consent.md#consent-lifecycle).
- [concepts/permissions/06 § Consent Revocation Semantics](../../concepts/permissions/06-multi-scope-consent.md#consent-revocation-semantics).
- [concepts/permissions/06 § rule 6 — per-co-owner consent](../../concepts/permissions/06-multi-scope-consent.md#co-ownership--multi-scope-session-access).

**Related agent-self-service pages:**
- [a01-my-inbox-outbox.md](a01-my-inbox-outbox.md) — Consent requests arrive as inbox notifications.

**Related system flows:**
- [system/s04-auth-request-state-transitions.md](../system/s04-auth-request-state-transitions.md) — consent state transitions and cascading.

**Org / project layouts exercised:**
- [organizations/06-joint-venture-acme.md](../../organizations/06-joint-venture-acme.md), [organizations/07-joint-venture-beta.md](../../organizations/07-joint-venture-beta.md), [projects/03-joint-research.md](../../projects/03-joint-research.md).
