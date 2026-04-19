<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Requirements doc — admin page, Phase 7 of fresh-install journey -->

# 12 — Authority Template Adoption

## 2. Page Purpose + Primary Actor

The page where the org admin reviews the **adoption Auth Requests** produced by the org-creation wizard (or by later customisation) for each Authority Template (A, B, C, D, E). On this page:
- Newly-created (not-yet-approved) adoption Auth Requests appear for review.
- Previously-approved adoptions appear as "active templates" with their fire-count and the grants they have produced to date.
- The admin can revoke an adoption, which cascade-revokes all grants it ever fired.

**Primary actor:** Human Agent holding `[allocate]` on the org's template-adoption registry (typically the org admin / CEO).

## 3. Position in the Journey

- **Phase:** 7 of 9 (Approve authority-template adoptions).
- **Depends on:** [06-org-creation-wizard.md](06-org-creation-wizard.md) (which creates the adoption Auth Requests) and at least one agent or project exists for the templates to fire against (Phases 5–6).
- **Enables:** Template A/B/C/D grants fire automatically as edges are created thereafter; Template E remains available for ad-hoc use.

## 4. UI Sketch

```
┌──────────────────────────────────────────────────────────────────┐
│ Templates — Acme Corporation                                     │
├──────────────────────────────────────────────────────────────────┤
│ Pending adoption approvals (2)                                   │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ Template A (Project Lead Authority)        [Approve][Deny] │ │
│ │ Template B (Direct Delegation Authority)   [Approve][Deny] │ │
│ └─────────────────────────────────────────────────────────────┘ │
│                                                                    │
│ Active templates (0)                                             │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │  (none yet — approve above to activate)                     │ │
│ └─────────────────────────────────────────────────────────────┘ │
│                                                                    │
│ Available-but-not-yet-adopted:                                   │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ ○ Template C (Hierarchical Org Chart)  [Adopt]              │ │
│ │ ○ Template D (Project-Scoped Role)     [Adopt]              │ │
│ │   Template E (Explicit Manual) — always available           │ │
│ └─────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────┘
```

After approving A and B, the top panel empties and the middle panel populates:

```
Active templates (2)
┌─────────────────────────────────────────────────────────────────┐
│ ● Template A  adopted 2026-04-15  fires: 0  grants: 0  [Revoke] │
│ ● Template B  adopted 2026-04-15  fires: 0  grants: 0  [Revoke] │
└─────────────────────────────────────────────────────────────────┘
```

## 5. Read Requirements

- **R-ADMIN-12-R1:** The page SHALL list all adoption Auth Requests for this org, grouped by state: **Pending** (awaiting this admin's approval), **Active** (approved, currently firing grants), **Revoked** (no longer firing).
- **R-ADMIN-12-R2:** Each active template SHALL show: date adopted, fire count to date, count of currently-active grants it produced, and a link to the adoption Auth Request's audit detail.
- **R-ADMIN-12-R3:** The page SHALL list the not-yet-adopted Templates C and D as "Available — [Adopt]" buttons (Template E is not adopted — it is always available on demand).

## 6. Write Requirements

- **R-ADMIN-12-W1:** The admin SHALL be able to Approve a pending adoption Auth Request. Approval fills their slot; if the admin is the only approver (the common case), the request transitions to Approved and the template becomes active.
- **R-ADMIN-12-W2:** The admin SHALL be able to Deny a pending adoption Auth Request. This closes the request without activating the template; the template is neither adopted nor usable until a new Auth Request is submitted.
- **R-ADMIN-12-W3:** The admin SHALL be able to Adopt a not-yet-adopted template by submitting a new adoption Auth Request. The admin is the sole approver (unless they delegate) and auto-approves inline.
- **R-ADMIN-12-W4:** The admin SHALL be able to Revoke an active template. Revocation cascade-revokes all grants the template fired (forward-only). Requires confirmation dialog listing the grant count.
- **R-ADMIN-12-W5:** Validation: Templates C and D require specific graph shape to be useful (C needs an org chart; D needs per-project role assignments). The "Adopt" button SHALL surface a warning if the prerequisite structure is missing but NOT block — orgs may adopt and then populate structure later.

## 7. Permission / Visibility Rules

- **Page access** — any Agent with membership in the org may view (read-only) the list of active templates; the pending/revoked sections are visible only to the org admin.
- **Approve / Deny / Adopt / Revoke (W1–W4)** — `[allocate]` on the template-adoption registry. Org admin only.

## 8. Event & Notification Requirements

- **R-ADMIN-12-N1:** Approve emits audit event `AuthorityTemplateAdopted { template_id, org_id, approved_by, audit_class: alerted }`.
- **R-ADMIN-12-N2:** Deny emits `AuthorityTemplateAdoptionDenied { ... }`.
- **R-ADMIN-12-N3:** Revoke emits `AuthorityTemplateRevoked { template_id, grant_count_revoked, revoked_by, alerted: true }`. Per [system/s04-auth-request-state-transitions.md](../system/s04-auth-request-state-transitions.md) the cascade is forward-only.
- **R-ADMIN-12-N4:** Each downstream grant the template fires is a separate event `AuthorityTemplateGrantFired` (not alerted by default; inherits template's adoption audit class).

## 9. Backend Actions Triggered

Approve (W1):
- Adoption Auth Request state transitions per [system/s04-auth-request-state-transitions.md](../system/s04-auth-request-state-transitions.md).
- Template becomes active: subsequent matching graph events (edge creation) fire grants per [system/s05-template-adoption-grant-fires.md](../system/s05-template-adoption-grant-fires.md).

Revoke (W4):
- Adoption request transitions to Revoked.
- Every grant with provenance pointing to this adoption request is revoked forward-only.

## 10. API Contract Sketch

```
GET  /api/v0/orgs/{org_id}/authority-templates
     → 200: { pending: [...], active: [...], revoked: [...], available: ["C","D","E-always"] }

POST /api/v0/orgs/{org_id}/authority-templates/{template_id}/approve
POST /api/v0/orgs/{org_id}/authority-templates/{template_id}/deny
     → 200: { audit_event_id }

POST /api/v0/orgs/{org_id}/authority-templates/{template_id}/adopt
     → 201: { adoption_auth_request_id, audit_event_id }    (auto-approved by admin)

POST /api/v0/orgs/{org_id}/authority-templates/{template_id}/revoke
     → 200: { grant_count_revoked, audit_event_id }
     → 400: confirmation not provided
```

## 11. Acceptance Scenarios

**Scenario 1 — approve A+B after org creation.**
*Given* [06-org-creation-wizard.md](06-org-creation-wizard.md) just created an org with A and B enabled, producing 2 pending adoption Auth Requests, *When* the org admin approves both on this page, *Then* both transition to Active; subsequent `HAS_LEAD` edges fire Template A grants (see [organizations/01-minimal-startup.md](../../organizations/01-minimal-startup.md) as the resulting shape).

**Scenario 2 — adopt C later for a mature org.**
*Given* [organizations/04-regulated-enterprise.md](../../organizations/04-regulated-enterprise.md)-style org that initially adopted only A and B but grows to need hierarchical supervision, *When* the admin clicks Adopt on Template C, *Then* a new adoption Auth Request is created + auto-approved; from then on, new `MANAGES`/`REPORTS_TO` edges in the org chart fire Template C grants.

**Scenario 3 — revoke A cascades grants.**
*Given* Template A has been active for a week and has fired 3 grants (one per project lead appointed), *When* the admin revokes A, *Then* all 3 grants are forward-only revoked (past session reads stand in the audit log; no future reads); the audit event records `grant_count_revoked: 3`.

## 12. Cross-References

**Concept files:**
- [concepts/permissions/05 § Authority Templates](../../concepts/permissions/05-memory-sessions.md#authority-templates-formerly-the-authority-question).
- [concepts/permissions/07 § Templates Are Pre-Authorized Allocations](../../concepts/permissions/07-templates-and-tools.md#templates-are-pre-authorized-allocations).
- [concepts/permissions/07 § Opt-in Example: Templates C, D, and E](../../concepts/permissions/07-templates-and-tools.md#opt-in-example-templates-c-d-and-e).

**Related admin pages:**
- [06-org-creation-wizard.md](06-org-creation-wizard.md) — source of the initial pending adoptions.

**Related system flows:**
- [system/s04-auth-request-state-transitions.md](../system/s04-auth-request-state-transitions.md), [system/s05-template-adoption-grant-fires.md](../system/s05-template-adoption-grant-fires.md).

**Org layouts exercised in Acceptance Scenarios:**
- [organizations/01-minimal-startup.md](../../organizations/01-minimal-startup.md), [organizations/04-regulated-enterprise.md](../../organizations/04-regulated-enterprise.md).
