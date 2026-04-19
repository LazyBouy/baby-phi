<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Requirements doc — admin page, Phase 5 of fresh-install journey -->

# 08 — Agent Roster List

## 2. Page Purpose + Primary Actor

The Agent Roster List page lets a Human Agent with org-admin authority see all Agents in the current organization and manage their lifecycle (add, archive, view detail).

**Primary actor:** Human Agent holding `[allocate]` on the org's `agent-catalogue` control_plane_object (typically the org's CEO; sometimes a separate delegated admin).

**Secondary actors:** Human Agents and LLM Agents who are project leads — they access the page in read-only mode, with visibility filtered to agents in projects they lead (via Template A). Per the terminology in [../README.md](../README.md), "admin" throughout is shorthand for the Human Agent holding the relevant grant.

## 3. Position in the Journey

- **Phase:** 5 of 9 (Build agent roster) — page 1 of 2.
- **Depends on:** Phase 3 (org created), Phase 4 (org dashboard).
- **Enables:** Phase 6 (projects can reference these agents), Phase 7 (Template A fires as leads are created), Phase 9 (first session needs an agent).

## 4. UI Sketch

```
┌─────────────────────────────────────────────────────────────────┐
│ Agent Roster — Acme Corporation                   [+ Add Agent] │
├─────────────────────────────────────────────────────────────────┤
│ Filter:  [Kind: Intern☐ Contract☐ System☐ Human☐]               │
│          [Role: sponsor☐ lead☐ member☐ worker☐]                 │
│ Search:  [                                                 ]     │
│                                                                  │
│ ╔══════════════════╦══════════╦═══════════╦══════════════════╗  │
│ ║ Name             ║ Kind     ║ Model     ║ Rating  [Actions]║  │
│ ╟──────────────────╫──────────╫───────────╫──────────────────╢  │
│ ║ lead-acme-1      ║ Contract ║ sonnet    ║ ★0.82 [View][⋯] ║  │
│ ║ coder-acme-2     ║ Intern   ║ sonnet    ║ ★0.75 [View][⋯] ║  │
│ ║ coder-acme-3     ║ Contract ║ sonnet    ║ ★0.88 [View][⋯] ║  │
│ ║ memory-extraction║ System ⚙ ║ sonnet    ║   —   [View]     ║  │
│ ║ agent-catalog    ║ System ⚙ ║ haiku     ║   —   [View]     ║  │
│ ╚══════════════════╩══════════╩═══════════╩══════════════════╝  │
│                                                                  │
│ Total: 5 agents (2 Contract, 1 Intern, 2 System, 0 Human)       │
│ Active Sessions: 3 / 24 capacity    Catalog-agent status: ●sync │
└─────────────────────────────────────────────────────────────────┘

Empty state: "No agents yet. [+ Add First Agent]"
Error state: inline toast "Failed to load roster — retry"
```

## 5. Read Requirements

- **R-ADMIN-08-R1:** The page SHALL display all Agents where a `MEMBER_OF` edge points to the current organization, showing id, kind (Human/Intern/Contract/System), model, current rating (if any), active-session count, and last-seen timestamp.
- **R-ADMIN-08-R2:** The page SHALL display a summary count by kind at the bottom of the list.
- **R-ADMIN-08-R3:** The page SHALL display the org's compute utilisation: `active_sessions / max_concurrent_sessions` from the org's `compute_resources` catalogue entry.
- **R-ADMIN-08-R4:** The page SHALL support filtering by kind (multi-select) and role (derived from project membership); filtering is client-side.
- **R-ADMIN-08-R5:** The page SHALL support substring search on agent id and `AgentProfile.name`.
- **R-ADMIN-08-R6:** System Agents SHALL be visually distinguished from Standard Agents (icon/badge).
- **R-ADMIN-08-R7:** The page SHALL show a live indicator of `agent-catalog-agent`'s current state (syncing / lag / error) sourced from that agent's health endpoint.

## 6. Write Requirements

- **R-ADMIN-08-W1:** The admin SHALL be able to create a new Agent via the "+ Add Agent" action, navigating to [09-agent-profile-editor.md](09-agent-profile-editor.md) in "new agent" mode.
- **R-ADMIN-08-W2:** The admin SHALL be able to archive an existing Agent via the row's "⋯" menu. Archival marks the Agent inactive but retains the node; archived agents appear only under an "Archived" filter.
- **R-ADMIN-08-W3:** Archiving SHALL NOT delete the archived agent's `inbox_object` / `outbox_object` composites — those remain readable by authorised auditors until Auth Request retention applies.
- **R-ADMIN-08-W4:** The admin SHALL NOT be able to archive System Agents from this page; the row's "⋯" menu on a System Agent is disabled. The "View" link opens [13-system-agents-config.md](13-system-agents-config.md) instead, where disabling is allowed.

## 7. Permission / Visibility Rules

All rules resolve to grants held by the **Human Agent** (or LLM Agent, in the secondary read-only role) viewing or acting on the page. No ambient "admin" privilege.

- **Page access** — requires `[read, list, inspect]` on `identity_principal` scoped to the current org. Held by the Human Agent with platform-admin or org-admin role (via the org's Default Grants) and by project leads (via Template A, filtered to their project).
- **Add / archive actions** — require `[allocate]` on the org's `agent-catalogue` `control_plane_object` instance. Held by the Human Agent with platform-admin or org-admin role. Project-lead Agents do NOT hold this grant and see the buttons disabled.
- **Role-filter visibility** — a project-lead Agent sees only the intersection of the roster with agents in projects they lead; the filter UI greys out checkboxes that would produce no rows for them.
- **Template A exercised** — project leads' view of *worker sessions* is not this page's concern, but this roster is the entry point to pages that do. See [09-agent-profile-editor.md](09-agent-profile-editor.md).

## 8. Event & Notification Requirements

- **R-ADMIN-08-N1:** When a new Agent is created via W1, the page SHALL display a success toast "Agent `<id>` created. Inbox and outbox auto-created."
- **R-ADMIN-08-N2:** When an Agent is archived via W2, the page SHALL require a confirmation dialog showing (a) the agent's active session count, (b) the count of grants that will cascade-revoke. On confirmation, an audit toast displays the resulting `AgentArchived` event ID.
- **R-ADMIN-08-N3:** The page SHALL show a live-updating indicator of `agent-catalog-agent`'s health (polling or WebSocket). On `error` state the page displays a banner "Agent catalog is stale — recent roster changes may not be reflected."

## 9. Backend Actions Triggered

Creating an Agent (W1) triggers:
- Agent node created; `MEMBER_OF` edge added to the org.
- Default Grants issued (per [permissions/05 § Default Grants Issued to Every Agent](../../concepts/permissions/05-memory-sessions.md#default-grants-issued-to-every-agent)).
- `inbox_object` and `outbox_object` composite instances atomically added to the org's `resources_catalogue` (see [system/s03-edge-change-catalog-update.md](../system/s03-edge-change-catalog-update.md)).
- `agent-catalog-agent` updates its index on the resulting `edge_change` trigger.
- Audit event `AgentCreated { agent_id, kind, created_by, org_id, profile_snapshot }` emitted.

Archiving an Agent (W2) triggers:
- Agent node's `active` set to false; `MEMBER_OF` edge retained for audit.
- All currently-live sessions for this Agent terminated with reason `agent_archived`.
- Grants held by the Agent revoked forward-only.
- Audit event `AgentArchived { agent_id, archived_by, session_count_terminated, grant_count_revoked }` emitted.

## 10. API Contract Sketch

```
GET  /api/v0/orgs/{org_id}/agents
     → 200: { agents: [{id, kind, model, rating, active_sessions, last_seen}], summary: {kind_counts, active_sessions, max_concurrent_sessions} }
     → 403: Permission denied (no [read,list] on identity_principal in this org)

POST /api/v0/orgs/{org_id}/agents
     Body: {
       profile: AgentProfile,
       model_config: ModelConfig,
       execution_limits: ExecutionLimits,
       kind: "Intern" | "Contract" | "System" | "Human",
       parallelize: u32
     }
     → 201: { agent_id, inbox_id, outbox_id, default_grants_issued: [grant_id,...] }
     → 400: Validation errors (invalid profile, unknown model_config.id, parallelize > org cap, ...)
     → 403: Permission denied (no [allocate] on agent-catalogue)
     → 409: Name collision

POST /api/v0/orgs/{org_id}/agents/{agent_id}/archive
     Body: { reason: String }
     → 200: { archived_at, session_count_terminated, grant_count_revoked, audit_event_id }
     → 403: Permission denied
     → 409: Agent is a protected System Agent

GET  /api/v0/orgs/{org_id}/_catalog-agent-status
     → 200: { state: "syncing" | "lag" | "error", last_update: DateTime, queue_depth: u32 }
```

## 11. Acceptance Scenarios

**Scenario 1 — minimal-startup first agent.**
*Given* a freshly-created `minimal-startup` organization (mirroring [organizations/01-minimal-startup.md](../../organizations/01-minimal-startup.md)) with no agents yet, *When* the founder navigates to the agent roster list and clicks "+ Add Agent", *Then* they are taken to the agent profile editor in "new agent" mode; after saving, the new Agent appears in the roster list with `kind: Intern`, its `inbox_object` and `outbox_object` exist in the org's `resources_catalogue`, and the default grants are issued.

**Scenario 2 — mid-product-team project-lead Agent visibility.**
*Given* the `mid-product-team` org (mirroring [organizations/02-mid-product-team.md](../../organizations/02-mid-product-team.md)) with its full 14-agent roster populated, *When* the Contract Agent `lead-stream-a` (a project-lead LLM Agent in this org) invokes the roster-read tool surface, *Then* they see the full roster but the "+ Add Agent" and "⋯ Archive" controls are absent from the response (they lack `[allocate]` on the agent-catalogue control plane); filtering by project `stream-a` shows only the 4 agents in their stream. Note that `lead-stream-a` is an LLM Agent — the same page renders as a programmatic surface for them whereas the CEO Human Agent uses the human-UI form of the same page.

**Scenario 3 — cannot archive a System Agent.**
*Given* any org with the default System Agents in its roster, *When* the org admin attempts to click "⋯ → Archive" on `memory-extraction-agent`, *Then* the "Archive" menu item is disabled with a tooltip "System Agents cannot be archived from this page — use System Agents Configuration (page 13) to disable or reconfigure."

**Scenario 4 — archiving an Intern terminates live sessions.**
*Given* an Intern agent currently running 2 concurrent sessions (`parallelize: 2`), *When* the org admin archives the agent with reason "promotion to Contract", *Then* both sessions are terminated with `reason: agent_archived`; the Agent node's `active` becomes false; the archival audit event records `session_count_terminated: 2` and the count of revoked grants. A follow-up Auth Request for the new Contract-tier profile is the separate flow that creates the successor agent.

## 12. Cross-References

**Concept files:**
- [concepts/agent.md § Agent Taxonomy](../../concepts/agent.md#agent-taxonomy) — the Intern/Contract/System/Human distinction rendered in the kind filter.
- [concepts/agent.md § Parallelized Sessions](../../concepts/agent.md#parallelized-sessions) — the `parallelize` field shown on each row.
- [concepts/organization.md § Organization Edges](../../concepts/organization.md) — `MEMBER_OF` defines roster membership.
- [concepts/permissions/01 § Resource Catalogue](../../concepts/permissions/01-resource-ontology.md#resource-catalogue) — why Agent creation atomically extends the catalogue with the new agent's inbox/outbox.
- [concepts/permissions/05 § Default Grants Issued to Every Agent](../../concepts/permissions/05-memory-sessions.md#default-grants-issued-to-every-agent) — the grants this page's W1 causes to be issued.
- [concepts/system-agents.md § Agent Catalog Agent](../../concepts/system-agents.md#agent-catalog-agent) — the system agent whose sync state is displayed on this page.

**phi-core types used in W1 payload:** `AgentProfile`, `ModelConfig`, `ExecutionLimits` — see [concepts/phi-core-mapping.md](../../concepts/phi-core-mapping.md).

**Related admin pages:**
- [09-agent-profile-editor.md](09-agent-profile-editor.md) — detail editor (create/edit mode).
- [13-system-agents-config.md](13-system-agents-config.md) — where System Agents are managed.

**Related system flows:**
- [system/s03-edge-change-catalog-update.md](../system/s03-edge-change-catalog-update.md) — reactive update on agent creation/archival.

**Org layouts exercised in Acceptance Scenarios:**
- [organizations/01-minimal-startup.md](../../organizations/01-minimal-startup.md) (Scenario 1)
- [organizations/02-mid-product-team.md](../../organizations/02-mid-product-team.md) (Scenario 2)
