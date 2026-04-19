<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Requirements doc — admin page, Phase 6 of fresh-install journey -->

# 10 — Project Creation Wizard

## 2. Page Purpose + Primary Actor

A 6-step wizard guiding a Human Agent with org-admin authority (or a delegated lead) through creating a Project. Handles Shape A (single-org) and Shape B (co-owned by two orgs) cases. Includes an optional OKR step where the admin declares `objectives` and `key_results` per the schema in [concepts/project.md § Objectives and Key Results](../../concepts/project.md#objectives-and-key-results-okrs).

**Primary actor:** Human Agent holding `[allocate]` on the owning org's `project-registry` control_plane_object (typically the org admin or a delegated lead).

**For Shape B (co-owned):** the wizard must be completed jointly — the initiating owner submits; the co-owner approves via an inbound Auth Request before the Project node materialises. See Section 9.

## 3. Position in the Journey

- **Phase:** 6 of 9 (Create first project) — page 1 of 2.
- **Depends on:** Phase 5 (roster has at least one non-system Agent to assign as lead).
- **Enables:** Phase 7 (Template A grants may fire when lead is assigned), Phase 9 (first session runs inside a project).

## 4. UI Sketch

```
Step 1/6 — Basics
  name [ Solo Sprint MVP        ]   project_id [ solo-project     ]
  description [ multiline ]
  goal (opt.) [ one-sentence goal ]

Step 2/6 — Ownership (Shape selection)
  ● Single-org (Shape A) — [ acme ▾ ]
  ○ Co-owned (Shape B) — primary [ acme ▾ ] + co-owner [ beta ▾ ]
                           (co-owner must approve before project is created)

Step 3/6 — OKRs  (optional; skip if no formal OKR tracking yet)
  Objectives (0..N)  [+ Add Objective]
  Key Results (0..M) [+ Add Key Result]
  Inline editor per objective: name, description, status, owner, deadline, key_result_ids

Step 4/6 — Resource boundaries
  Import from owning org(s)' catalogue:
    filesystem_objects   [ /workspace/solo-project/** ☑ ]
    secrets              [ anthropic-api-key ☑ ]
    model_runtime        [ claude-sonnet-default ☑ ]
    memory scopes        [ per-agent ☑  per-project ☑ ]
    session scopes       [ per-project ☑  per-agent ☑ ]
  (Boundaries must be a SUBSET of the owning org's catalogue.)

Step 5/6 — Initial agent roster
  Lead (required):   [ founder ▾ ]
  Members:           [ intern-a, intern-b ] [+ Add member]
  Role assignment will fire Template A for the lead on submit.

Step 6/6 — Review & submit
  Diff preview of all choices; Shape B shows "Pending co-owner approval after submit."
```

## 5. Read Requirements

- **R-ADMIN-10-R1:** Step 2 SHALL list only orgs where the acting Human Agent holds `[allocate]` on the project-registry.
- **R-ADMIN-10-R2:** Step 4 SHALL list only catalogue entries that belong to the selected owning org(s). For Shape B, entries must appear in **both** owners' catalogues.
- **R-ADMIN-10-R3:** Step 5 SHALL list agents from the owning org(s)' roster, filtered to those eligible for the chosen role (leads must be Contract or Human; Interns may be members but not leads).
- **R-ADMIN-10-R4:** The review step SHALL show a validation summary: any resource boundaries that are not in both co-owners' catalogues (Shape B) block submit with a clear message.

## 6. Write Requirements

- **R-ADMIN-10-W1:** The admin SHALL be able to traverse steps freely; the draft autosaves.
- **R-ADMIN-10-W2 (Shape A):** On submit, the system creates the Project node with `BELONGS_TO` edge to the owning org, populates OKR value objects, assigns the lead via `HAS_LEAD` (fires Template A), adds members via `HAS_AGENT`, sets `resource_boundaries`. One alerted audit event `ProjectCreated` emitted.
- **R-ADMIN-10-W3 (Shape B):** On submit, the system creates a Template E Auth Request targeting both owning orgs' `project-registry`. Both co-owners hold approver slots. The Project node materialises only when both slots are Approved. If one co-owner declines, the request terminates in `Partial`/`Denied` and no Project is created.
- **R-ADMIN-10-W4:** Validation: project_id matches `[a-z][a-z0-9-]*` and is unique; name non-empty; Shape B requires co-owner to be a different org with at least one model_runtime entry overlapping the primary's catalogue; at least one lead assigned.
- **R-ADMIN-10-W5:** OKR validation: each objective's `key_result_ids` must reference KRs actually listed in the same step; KR `target_value` matches its `measurement_type` (count → int, percentage → 0..1, etc.).

## 7. Permission / Visibility Rules

- **Page access** — `[allocate]` on owning-org `project-registry`. Org admin or a delegated lead with this grant.
- **Shape B submit** — both co-owning orgs must have the acting agent's counterpart (or admin) available to approve. The system does not let one org unilaterally create a Shape B project.
- **Lead assignment (step 5)** — may select any Agent from the owning-org(s) roster regardless of current role.

## 8. Event & Notification Requirements

- **R-ADMIN-10-N1:** On Shape A submit, emit `ProjectCreated` (alerted if org audit default is alerted, else logged).
- **R-ADMIN-10-N2:** On Shape B submit, emit `ProjectCreationPending` referencing the pending Auth Request; co-owner inbox receives an `AgentMessage` with the request.
- **R-ADMIN-10-N3:** Lead assignment emits `TemplateAAdoptionFired` audit event per Template A rules.

## 9. Backend Actions Triggered

Shape A submit:
1. Project node created.
2. `BELONGS_TO` edge to owning org.
3. OKR value objects embedded.
4. `HAS_LEAD`, `HAS_AGENT`, `HAS_SPONSOR` edges.
5. Template A fires — grants issued for the lead (see [system/s05-template-adoption-grant-fires.md](../system/s05-template-adoption-grant-fires.md)).
6. Audit events.

Shape B submit:
1. Template E Auth Request materialised with two approver slots (one per co-owner).
2. `AgentMessage` deposited in each co-owner's inbox.
3. On both-approve: steps 1–6 of Shape A above, plus `BELONGS_TO` edges to **both** orgs.
4. On any deny: request closes, nothing is created, requestor is notified.

## 10. API Contract Sketch

```
GET  /api/v0/orgs/{org_id}/project-wizard/bootstrap
     → 200: { eligible_co_owners, available_catalogue, eligible_leads }

POST /api/v0/orgs/{org_id}/project-wizard/draft
     Body: { step, values }
     → 200: { draft_id }

POST /api/v0/orgs/{org_id}/projects
     Body: {
       project_id, name, description, goal?,
       ownership: "shape_a" | "shape_b",
       co_owner_org_id?,
       objectives: [Objective], key_results: [KeyResult],
       resource_boundaries: {...},
       lead_agent_id, member_agent_ids: [...]
     }
     Shape A → 201: { project_id, audit_event_id }
     Shape B → 202: { pending_auth_request_id }  (awaiting co-owner approval)
     → 400 / 409: validation / collision
```

## 11. Acceptance Scenarios

**Scenario 1 — create flat-single-project inside minimal-startup.**
*Given* [organizations/01-minimal-startup.md](../../organizations/01-minimal-startup.md) is ready and the founder is the acting admin, *When* they run the wizard for Shape A, owning org `minimal-startup`, 2 objectives / 4 KRs, lead `founder`, members `intern-a`+`intern-b`, *Then* a Project is created matching [projects/01-flat-single-project.md](../../projects/01-flat-single-project.md); Template A fires grants for `founder`; `ProjectCreated` audit emitted.

**Scenario 2 — create joint-research Shape B.**
*Given* [organizations/06-joint-venture-acme.md](../../organizations/06-joint-venture-acme.md) and [organizations/07-joint-venture-beta.md](../../organizations/07-joint-venture-beta.md) exist with catalogues overlapping (both have `claude-sonnet-default`), *When* the Acme CEO submits a Shape B wizard for `joint-research` with Beta as co-owner, *Then* a Template E Auth Request materialises with two slots; Beta's CEO receives an inbox message; on Beta's approval the Project is created with `BELONGS_TO` edges to both orgs matching [projects/03-joint-research.md](../../projects/03-joint-research.md).

**Scenario 3 — Shape B co-owner declines.**
*Given* the above Shape B submit, *When* the Beta CEO denies the approver slot, *Then* the Auth Request transitions to Denied, no Project is created, and the Acme CEO receives an inbox message with the denial reason.

## 12. Cross-References

**Concept files:**
- [concepts/project.md](../../concepts/project.md) — Project node, Properties, Edges, OKR value objects.
- [concepts/permissions/06 § Multi-Scope Session Access](../../concepts/permissions/06-multi-scope-consent.md#multi-scope-session-access) — Shape A/B/C/D/E.
- [concepts/permissions/05 § Authority Templates](../../concepts/permissions/05-memory-sessions.md#authority-templates-formerly-the-authority-question) — Template A fires on lead assignment.

**Related admin pages:**
- [11-project-detail.md](11-project-detail.md) — the created project's home.
- [12-authority-template-adoption.md](12-authority-template-adoption.md) — for reviewing Template E flows resulting from Shape B.

**Related system flows:**
- [system/s05-template-adoption-grant-fires.md](../system/s05-template-adoption-grant-fires.md), [system/s04-auth-request-state-transitions.md](../system/s04-auth-request-state-transitions.md).

**Project layouts exercised:**
- [projects/01-flat-single-project.md](../../projects/01-flat-single-project.md), [projects/03-joint-research.md](../../projects/03-joint-research.md).
