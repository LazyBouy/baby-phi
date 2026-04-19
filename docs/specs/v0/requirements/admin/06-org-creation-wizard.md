<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Requirements doc — admin page, Phase 3 of fresh-install journey -->

# 06 — Organization Creation Wizard

## 2. Page Purpose + Primary Actor

An 8-step wizard that guides a Human Agent through creating an organization. On submit, the wizard creates the Organization node, its resource catalogue, its system agents (the two standard ones), default grants, and the adoption Auth Requests for each enabled Authority Template. The CEO supplied in step 7 becomes the org admin (receives `[allocate]` on the org's control-plane objects).

**Primary actor:** Human Agent holding `[allocate]` on `system:root` (the platform admin, from Phase 1).

## 3. Position in the Journey

- **Phase:** 3 of 9 (Create first organization).
- **Depends on:** Phase 1 complete; Phase 2 partly complete (model providers must exist for agents to use; MCP servers and credentials may be added later).
- **Enables:** Phase 4 (org dashboard) and all subsequent phases.

## 4. UI Sketch

Multi-step; showing the flow:

```
Step 1/8 — Basics
┌─────────────────────────────────────────────────────────────────┐
│ Name             [ Acme Corporation                           ] │
│ Org ID           [ acme                                        ] │
│ Vision (opt.)    [ Ship useful products; partner well.        ] │
│ Mission (opt.)   [ Simple, productive, collaborative.         ] │
└─────────────────────────────────────────────────────────────────┘

Step 2/8 — Template selection
┌─────────────────────────────────────────────────────────────────┐
│ ● Start from Standard Organization Template (recommended)       │
│ ○ Start blank (advanced)                                        │
│ ○ Fork from an existing org  [ minimal-startup ▾ ]             │
└─────────────────────────────────────────────────────────────────┘

Step 3/8 — Consent & audit policy
  consent_policy: [implicit | one_time | per_session] (pre-filled from platform default)
  audit_class_default: [silent | logged | alerted]

Step 4/8 — Authority Templates enabled
  ☑ A (Project Lead)   ☑ B (Delegation)   ☐ C (Org Chart)   ☐ D (Project Role)
  (E is always available)

Step 5/8 — Resource Catalogue
  Import from platform:  ☑ claude-sonnet-default   ☑ anthropic-api-key   ☐ mcp-github
  Additional entries:    [+ Add filesystem path]  [+ Add compute pool]  ...

Step 6/8 — Execution limits + token budget
  max_turns [50]  max_tokens [100000]  max_duration_secs [3600]  max_cost_usd [5.00]
  token_budget_pool initial_allocation: [ 5_000_000 ]

Step 7/8 — First sponsor
  Display name [ Founder A           ]
  Channel      [ Slack ▾  @founder   ]
  Will be invited with org-admin role (receives [allocate] on the org control plane).

Step 8/8 — Review & submit
  (diff preview of all choices; confirm to commit)
```

Empty state: step 1 is the starting point on fresh install.
Error state: per-step validation errors inline; global "Failed to commit — retry" on final submit error.

## 5. Read Requirements

- **R-ADMIN-06-R1:** Each step SHALL show pre-filled defaults sourced from the platform defaults (page 05) where applicable.
- **R-ADMIN-06-R2:** The template-selection step SHALL display a preview of what each template option provides (Standard vs Blank vs Fork).
- **R-ADMIN-06-R3:** The resource-catalogue step SHALL list all platform-level entries (model providers, MCP servers, secrets, compute) available for import, showing `tenants_allowed` eligibility for this new org.
- **R-ADMIN-06-R4:** The review step SHALL show the complete diff of choices vs the Standard Organization Template as a reviewable summary.

## 6. Write Requirements

- **R-ADMIN-06-W1:** The admin SHALL be able to progress forward/backward through steps. Progress is autosaved to a draft; the draft persists across sessions until submit or cancel.
- **R-ADMIN-06-W2:** Validation (inline per step): `org_id` matches `[a-z][a-z0-9-]*`; name non-empty; at least one Authority Template enabled; at least one `model/runtime_object` imported; `token_budget_pool.initial_allocation >= 0`; CEO display-name and channel non-empty.
- **R-ADMIN-06-W3:** On final submit, the system atomically creates:
  1. The Organization node with its properties.
  2. The org's `resources_catalogue` with the imported + added entries.
  3. Two system agents (`memory-extraction-agent` and `agent-catalog-agent`) per [system-agents.md](../../concepts/system-agents.md).
  4. A Human Agent for the CEO with their channel, and a Grant giving them `[allocate]` on the org's `agent-catalogue` and `resources_catalogue` control-plane objects.
  5. Default Grants for the org's system agents.
  6. Adoption Auth Requests for each enabled Authority Template, auto-approved by the CEO (who holds `[allocate]` by virtue of the grant issued in step 4).
- **R-ADMIN-06-W4:** If the final submit fails partway, the system SHALL roll back (no orphaned nodes). Implementation: the entire sequence runs inside a single Auth Request with a compound action; see [system/s04-auth-request-state-transitions.md](../system/s04-auth-request-state-transitions.md) for atomicity semantics.

## 7. Permission / Visibility Rules

- **Page access** — `[allocate]` on `system:root`. Platform admin only. Once one org exists, the wizard may still be used to create additional orgs, same gate.
- **Per-step reads** — inherit page access.
- **Final submit** — requires `[allocate]` on `system:root` (cross-org creation is a platform operation).

## 8. Event & Notification Requirements

- **R-ADMIN-06-N1:** Each step transition is a local UI event, not audited.
- **R-ADMIN-06-N2:** Final submit emits audit event `OrganizationCreated { org_id, name, created_by, ceo_agent_id, enabled_templates, consent_policy, audit_class: alerted }`.
- **R-ADMIN-06-N3:** The CEO Human Agent SHALL receive an inbox message (delivered via their registered channel) with subject "You have been invited as CEO of `<org>`" and a link back to the org dashboard (page 07).
- **R-ADMIN-06-N4:** Audit event emitted per adoption Auth Request auto-created: `AuthorityTemplateAdopted { template_id, org_id, adoption_auth_request_id }` — one per enabled template.

## 9. Backend Actions Triggered

Final submit (W3) triggers, in order:
1. Creation of the `Organization` node (see [concepts/organization.md](../../concepts/organization.md)).
2. Population of the org's `resources_catalogue` (see [permissions/01 § Resource Catalogue](../../concepts/permissions/01-resource-ontology.md#resource-catalogue)).
3. Creation of two system Agents — see [system/s03-edge-change-catalog-update.md](../system/s03-edge-change-catalog-update.md) (`agent-catalog-agent` registers itself immediately).
4. Creation of the CEO Human Agent + their inbox/outbox.
5. Issuance of Default Grants per [permissions/07 § Standard Organization Template](../../concepts/permissions/07-templates-and-tools.md#standard-organization-template).
6. For each enabled authority template: adoption Auth Request creation — see [system/s05-template-adoption-grant-fires.md](../system/s05-template-adoption-grant-fires.md). Auto-approved by the CEO.
7. Alerted `OrganizationCreated` audit event.

## 10. API Contract Sketch

```
GET  /api/v0/platform/org-wizard/defaults
     → 200: { platform_defaults, available_platform_catalogue }

POST /api/v0/platform/org-wizard/draft
     Body: { step, values }
     → 200: { draft_id, step, values }     (autosave)

POST /api/v0/platform/orgs
     Body: {
       org_id, name, vision?, mission?,
       template_source: "standard" | "blank" | "fork:<org_id>",
       consent_policy, audit_class_default,
       authority_templates_enabled: [A, B, ...],
       resources_catalogue: {...},
       execution_limits: {...},
       token_budget: { initial_allocation },
       ceo: { display_name, channel: {...} }
     }
     → 201: {
       org_id,
       ceo_agent_id,
       system_agents: [memory-extraction-agent-id, agent-catalog-agent-id],
       adoption_auth_request_ids: [...],
       audit_event_id
     }
     → 400: Validation errors (field level)
     → 409: org_id collision
```

## 11. Acceptance Scenarios

**Scenario 1 — create minimal-startup.**
*Given* the platform admin has completed Phase 1 and registered at least `claude-sonnet-default` + `anthropic-api-key`, *When* they run the wizard choosing Standard template, `consent_policy: implicit`, templates `[A, B]`, token budget `5_000_000`, CEO "Founder A" on Slack, and submit, *Then* the resulting org matches the shape described in [organizations/01-minimal-startup.md](../../organizations/01-minimal-startup.md): an Organization node named `minimal-startup` with two system agents, a populated `resources_catalogue`, the CEO as org admin, and adoption Auth Requests for A and B in Approved state.

**Scenario 2 — create regulated-enterprise with stricter defaults.**
*Given* the platform admin wants a regulated-enterprise-style org, *When* they run the wizard selecting Standard, `consent_policy: per_session`, `audit_class_default: alerted`, templates `[A, B, C]`, and they add a `compliance-audit-agent` reference from the platform catalogue, *Then* the resulting org mirrors [organizations/04-regulated-enterprise.md](../../organizations/04-regulated-enterprise.md) with 3 active templates and alerted audit default.

**Scenario 3 — partial failure rolls back.**
*Given* the admin is on the final submit step and the system agent profile-ref resolution fails (e.g., `system-memory-extraction` profile not found in the platform's system-agent registry), *When* the submit fires, *Then* no Organization node is created, no Auth Requests are materialised, and the admin sees an error banner. The wizard draft is preserved so the admin can retry after fixing.

## 12. Cross-References

**Concept files:**
- [concepts/organization.md](../../concepts/organization.md) — Organization node + edges.
- [concepts/permissions/07 § Standard Organization Template](../../concepts/permissions/07-templates-and-tools.md#standard-organization-template) — the template this wizard defaults to.
- [concepts/permissions/06 § Three Consent Policies](../../concepts/permissions/06-multi-scope-consent.md#three-consent-policies).
- [concepts/system-agents.md](../../concepts/system-agents.md) — the two system agents auto-created.

**Related admin pages:**
- [05-platform-defaults.md](05-platform-defaults.md) — source of pre-filled defaults.
- [02-platform-model-providers.md](02-platform-model-providers.md), [03-platform-mcp-servers.md](03-platform-mcp-servers.md), [04-platform-credentials-vault.md](04-platform-credentials-vault.md) — source of importable platform catalogue entries.
- [07-organization-dashboard.md](07-organization-dashboard.md) — destination after successful submit.
- [12-authority-template-adoption.md](12-authority-template-adoption.md) — where the newly-created adoption Auth Requests are reviewed (though the wizard auto-approves them, the audit record is reviewable there).

**Related system flows:**
- [system/s03-edge-change-catalog-update.md](../system/s03-edge-change-catalog-update.md) — fires as agents are added.
- [system/s05-template-adoption-grant-fires.md](../system/s05-template-adoption-grant-fires.md) — authority-template adoption flow.

**Org layouts exercised in Acceptance Scenarios:**
- [organizations/01-minimal-startup.md](../../organizations/01-minimal-startup.md), [organizations/04-regulated-enterprise.md](../../organizations/04-regulated-enterprise.md).
