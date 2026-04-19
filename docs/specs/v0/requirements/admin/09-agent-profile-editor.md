<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Requirements doc — admin page, Phase 5 of fresh-install journey -->

# 09 — Agent Profile Editor

## 2. Page Purpose + Primary Actor

Detail editor for a single Agent. Supports **create** (reached via "+ Add Agent" from [08-agent-roster-list.md](08-agent-roster-list.md)) and **edit** (reached via a row's "View" link). The editor binds the phi-core types `AgentProfile` + `ModelConfig` + `ExecutionLimits` with baby-phi's `parallelize`, `kind`, and project-role fields. For **System Agents**, the editor is read-only here — full configuration lives in [13-system-agents-config.md](13-system-agents-config.md).

**Primary actor:** Human Agent holding `[allocate]` on the org's `agent-catalogue`.

**Secondary actors:** the Agent being edited (when they view their own profile) — read-only access for `self_description` and grants list (but not the AgentProfile write form); full self-view lives in [agent-self-service/a05-my-profile-and-grants.md](../agent-self-service/a05-my-profile-and-grants.md).

## 3. Position in the Journey

- **Phase:** 5 of 9 — page 2 of 2.
- **Depends on:** [08-agent-roster-list.md](08-agent-roster-list.md); org's resources catalogue populated with at least one `model/runtime_object`.
- **Enables:** Phase 6 (projects assign these agents); Phase 9 (first session uses an agent created here).

## 4. UI Sketch

Create mode:

```
┌─────────────────────────────────────────────────────────────────┐
│ New Agent — Acme Corporation                              [Save]│
├─────────────────────────────────────────────────────────────────┤
│ Identity                                                         │
│   id          [ coder-acme-4                                   ] │
│   kind        ● Intern  ○ Contract  ○ Human  (System read-only) │
│                                                                  │
│ AgentProfile (phi-core)                                          │
│   name              [ coder-acme-4                            ] │
│   system_prompt     [ multiline editor                        ] │
│   thinking_level    ○ low  ● medium  ○ high                    │
│   temperature       [ 0.3                                      ] │
│   personality       [ curious                                 ] │
│                                                                  │
│ ModelConfig (phi-core)                                           │
│   id ▾  [ claude-sonnet-default                               ] │
│   max_tokens [ 4096 ]                                            │
│                                                                  │
│ ExecutionLimits (phi-core)                                       │
│   max_turns [ 40 ]  max_tokens [ 80000 ]                         │
│   max_duration_secs [ 2400 ]  max_cost_usd [ 2.00 ]              │
│                                                                  │
│ baby-phi fields                                                  │
│   parallelize  [ 2 ]  (org cap: 4)                               │
│   project_role   { class-cohort-1: learner }   [+ Add role]     │
│   base_organization   acme (inherited from this org)            │
│                                                                  │
│ Preview: Default Grants that will be issued on save             │
│   [read, list, recall, delete] on inbox_object (own)             │
│   [read, list, inspect, send] on outbox_object (own)             │
│   [read, recall] on memory_object tagged agent:{new_id}          │
│   ...                                                            │
└─────────────────────────────────────────────────────────────────┘
```

Edit mode: same form pre-populated; the id field is read-only; kind cannot be changed (promotion Intern→Contract is a separate flow).

Error state: per-field validation errors inline.

## 5. Read Requirements

- **R-ADMIN-09-R1:** In edit mode, the page SHALL load the Agent's full current state: `AgentProfile`, `ModelConfig`, `ExecutionLimits`, `parallelize`, `kind`, `base_organization`, `current_project`, active project_role map, and the list of Grants this Agent currently holds.
- **R-ADMIN-09-R2:** The `ModelConfig.id` field SHALL be a dropdown populated from the org's `model_runtime_objects` catalogue entries.
- **R-ADMIN-09-R3:** The `parallelize` field SHALL display the org-level cap as an inline hint; values exceeding the cap are rejected on save.
- **R-ADMIN-09-R4:** Below the form, the page SHALL preview the Default Grants that will be issued (create mode) or are currently held (edit mode). For edit mode, each listed grant is clickable to a grant-detail view.
- **R-ADMIN-09-R5:** For an edit-mode System Agent, the form fields SHALL be disabled with a banner "System Agents are managed on page 13."

## 6. Write Requirements

- **R-ADMIN-09-W1 (create):** On save in create mode, the system creates the Agent node with the supplied fields, issues the default grants (see Section 9), and atomically adds `inbox_object` + `outbox_object` to the org's `resources_catalogue`.
- **R-ADMIN-09-W2 (edit):** On save in edit mode, the system updates the Agent's `AgentProfile` / `ModelConfig` / `ExecutionLimits` / `parallelize` / `project_role` with the new values. The Agent's `kind`, `id`, `base_organization`, and existing grants are immutable here — changes to those go through separate flows (promotion, transfer, Auth Request).
- **R-ADMIN-09-W3 (validation):**
  - `id` matches `[a-z][a-z0-9-]*` and is unique within the org.
  - `AgentProfile` fields meet phi-core type constraints (thinking_level enum, temperature in [0, 2], etc.).
  - `ModelConfig.id` exists in the org's catalogue.
  - `ExecutionLimits` values positive and within org ceilings.
  - `parallelize >= 1` and `<= org cap`.
- **R-ADMIN-09-W4:** Edit mode SHALL preserve the Agent's active sessions; changes to `ExecutionLimits` apply to **new** sessions only (active sessions keep the limits in force at their start).
- **R-ADMIN-09-W5:** The admin SHALL NOT be able to reduce `parallelize` below the Agent's current live-session count; if they need to, they must first terminate sessions from the agent's detail (via 13 or directly via session browser — a steady-state page).

## 7. Permission / Visibility Rules

- **Page access in create mode** — `[allocate]` on the org's `agent-catalogue` control_plane_object. Org admin.
- **Page access in edit mode (read the form)** — `[read, inspect]` on the specific Agent's `identity_principal`. Org admin, the Agent themselves (for their own profile view), and project leads for agents in their project (read-only).
- **Save in edit mode** — `[allocate]` on the `agent-catalogue` (org admin). An Agent may NOT edit their own profile through this page — self-update of `self_description` happens on their own identity-browser page, not here.
- **System Agent fields** — fully read-only on this page regardless of grants; writes happen on page 13.

## 8. Event & Notification Requirements

- **R-ADMIN-09-N1:** On create save, emit audit event `AgentCreated { agent_id, kind, profile_snapshot, created_by, org_id }`.
- **R-ADMIN-09-N2:** On edit save, emit audit event `AgentProfileUpdated { agent_id, diff: {field: {old, new}}, updated_by }`.
- **R-ADMIN-09-N3:** When the page is open in edit mode and the Agent begins a new session in another tab/surface, the page SHALL show an inline banner "This agent just started a new session — some edits may not apply until the session ends."

## 9. Backend Actions Triggered

On create (W1):
- Agent node created.
- `MEMBER_OF` edge to the org.
- `inbox_object` + `outbox_object` composite instances created (one per agent, per [permissions/01 § Resource Catalogue](../../concepts/permissions/01-resource-ontology.md#resource-catalogue)).
- Default Grants issued per [permissions/05 § Default Grants Issued to Every Agent](../../concepts/permissions/05-memory-sessions.md#default-grants-issued-to-every-agent) plus the inbox/outbox grants defined in [permissions/05 § Inbox and Outbox](../../concepts/permissions/05-memory-sessions.md#inbox-and-outbox-agent-messaging).
- `agent-catalog-agent` reactive update (see [system/s03-edge-change-catalog-update.md](../system/s03-edge-change-catalog-update.md)).
- `AgentCreated` audit event.

On edit (W2):
- Agent node property update.
- `AgentProfileUpdated` audit event with full diff.
- No grant changes — profile edits do not affect grants held by the agent.

## 10. API Contract Sketch

```
GET  /api/v0/orgs/{org_id}/agents/{agent_id}
     → 200: { agent: {profile, model_config, execution_limits, parallelize, kind, base_organization, project_role}, grants: [...] }
     → 403: Permission denied
     → 404: Not found

POST /api/v0/orgs/{org_id}/agents                 (create — same as admin/08 W1 contract)
     → as specified in [08-agent-roster-list.md § 10](08-agent-roster-list.md#10-api-contract-sketch)

PATCH /api/v0/orgs/{org_id}/agents/{agent_id}
     Body: { profile?, model_config?, execution_limits?, parallelize?, project_role? }
     → 200: { updated_at, diff, audit_event_id }
     → 400: Validation errors
     → 403: Permission denied
     → 409: parallelize-below-live-sessions conflict
```

## 11. Acceptance Scenarios

**Scenario 1 — create an Intern in mid-product-team.**
*Given* [organizations/02-mid-product-team.md](../../organizations/02-mid-product-team.md) has `claude-sonnet-default` in its catalogue and `parallelize` cap 4, *When* the CEO creates a new Intern `coder-c3` with `parallelize: 2`, max_turns 40, max_cost_usd 2.00, *Then* the Agent is created, default grants are issued (including inbox/outbox), the `AgentCreated` audit event records the profile snapshot, and the new row appears in page 08's roster list.

**Scenario 2 — edit parallelize with conflict.**
*Given* Agent `intern-a1` is currently running 2 concurrent sessions (at `parallelize: 2`), *When* the org admin tries to lower `parallelize` to 1, *Then* the save is rejected with `409 parallelize-below-live-sessions` and a message telling the admin to terminate one session first.

**Scenario 3 — System Agent read-only.**
*Given* the page is open in edit mode on `memory-extraction-agent`, *When* any Human Agent views the page, *Then* all fields are disabled and a banner directs them to [13-system-agents-config.md](13-system-agents-config.md).

## 12. Cross-References

**Concept files:**
- [concepts/agent.md § LLM Agent Anatomy](../../concepts/agent.md#llm-agent-anatomy--the-extended-model) — Soul, Power, Experience, Identity.
- [concepts/agent.md § Parallelized Sessions](../../concepts/agent.md#parallelized-sessions) — semantics of the `parallelize` field.
- [concepts/permissions/05 § Default Grants Issued to Every Agent](../../concepts/permissions/05-memory-sessions.md#default-grants-issued-to-every-agent).
- [concepts/permissions/05 § Inbox and Outbox](../../concepts/permissions/05-memory-sessions.md#inbox-and-outbox-agent-messaging).
- [concepts/phi-core-mapping.md](../../concepts/phi-core-mapping.md) — AgentProfile, ModelConfig, ExecutionLimits, CompactionConfig origins.

**phi-core types:** `AgentProfile`, `ModelConfig`, `ExecutionLimits`, `CompactionConfig` (if exposed in v0).

**Related admin pages:**
- [08-agent-roster-list.md](08-agent-roster-list.md) — parent list view.
- [13-system-agents-config.md](13-system-agents-config.md) — where System Agents are actually configured.

**Related agent-self-service pages:**
- [agent-self-service/a05-my-profile-and-grants.md](../agent-self-service/a05-my-profile-and-grants.md) — where an Agent views their own profile read-only.

**Related system flows:**
- [system/s03-edge-change-catalog-update.md](../system/s03-edge-change-catalog-update.md).

**Org layouts exercised in Acceptance Scenarios:**
- [organizations/02-mid-product-team.md](../../organizations/02-mid-product-team.md).
