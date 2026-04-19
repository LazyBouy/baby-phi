# Plan: Requirements from the Fresh-Install Admin Journey

> **Legend:**
> - `[PLAN: new]` — part of this fresh plan
> - `[DOCS: ✅ done]` / `[DOCS: ⏳ pending]` / `[DOCS: n/a]`

## Context  `[PLAN: new]` `[DOCS: n/a]`

The concept is ready (~88–92% calibrated). The next step is to derive **requirements** from the concept — commitments the implementation must meet, phrased so that "is this met?" is answerable with an observable check.

We use **admin-UI-first framing, anchored on the fresh-install journey**. An admin who has just installed baby-phi moves through a fixed sequence of pages to go from zero to "an agent can productively run a session." That sequence is also the **bootstrap dependency chain**: you can't configure agents before an org exists; you can't create an org before the platform admin has claimed the System Bootstrap Template adoption. Derived requirements therefore order themselves.

Self-service and system-agent behaviours are **admin-configured**. An admin page sets up what agents are and what reactive handlers do; the runtime contract is captured in a parallel `system/` folder. **Agent self-service surfaces** (inbox/outbox, consent, auth-request tracking, profile) are part of this plan — admins configure what they *are*, agents interact with them through their own pages/tool surfaces.

**Terminology note — NFR:** "Non-Functional Requirement" — quality-attribute requirements (performance, observability, security, cost) as distinct from Functional Requirements (what the system does). Used throughout the plan.

**Terminology note — "admin":** per [concepts/agent.md § Grounding Principle](../concepts/agent.md#grounding-principle), **everything is an Agent** — humans included. "Admin" is **not a distinct entity kind**; it is a **role** that a Human Agent holds by virtue of the grants they carry. Specifically:

- **Platform admin** = the Human Agent holding `[allocate]` on `system:root` (provisioned by the System Bootstrap Template adoption in Phase 1).
- **Org admin** = a Human Agent holding `[allocate]` on the org's `agent-catalogue` and/or `resources_catalogue` control_plane_object instances. The org's CEO is typically the org admin, but the two can be separated.

Throughout the plan, when a page says "the admin performs X," read it as "the Human Agent holding the relevant admin-level grant performs X." Permission rules on every page resolve to concrete grants on concrete Human Agents — there is no ambient "admin" privilege.

This also means **admin-facing pages and agent-self-service pages are used by the same entities**: a Human Agent acting as org admin uses admin pages for that role **and** agent-self-service pages (inbox, Auth Requests filed on their behalf, etc.) for their participation as an Agent in the system. Rendering differs — a Human Agent's inbox delivers via their `Channel` (Slack/email), an LLM Agent's inbox is a programmatic surface — but the underlying inbox composite is the same.

**Archive location:** new subfolder `baby-phi/docs/specs/plan/requirements/`. Going forward, requirements plans live there; concept plans stay at the existing flat `plan/` level.

## History — where prior plans are recorded  `[PLAN: new]` `[DOCS: n/a]`

- Phase A–D archive: `baby-phi/docs/specs/plan/d95fac8f-ownership-auth-request.md`
- Phase F archive: `baby-phi/docs/specs/plan/54b1b2cb-split-and-gap-closure.md`
- Phase G/H archive: `baby-phi/docs/specs/plan/b30cb86b-push-to-95.md`
- Org/project layouts archive: `baby-phi/docs/specs/plan/b99b0bdd-org-project-layouts.md`
- **This plan will archive to:** `baby-phi/docs/specs/plan/requirements/<random>-fresh-install-admin-journey.md`

## Decisions Captured  `[PLAN: new]` `[DOCS: see Impl column]`

| Topic | Decision | Impl Status |
|-------|----------|-------------|
| **Framing** | Admin-UI-first, anchored on fresh-install journey (bootstrap sequence). | ⏳ |
| **Per-admin-page file template** | 10 sections: purpose+actor, UI sketch, read reqs, write reqs, permission/visibility, events/notifications, backend actions triggered, API contract sketch, acceptance scenarios, cross-references. | ⏳ (R2) |
| **Traceability goal** | Every concept rule maps to ≥1 requirement; every requirement cites the concept section it derives from. Traceability matrix is an incremental artefact, populated as pages are written. | ⏳ (R14) |
| **Non-admin actor coverage** | Admin configures self-service and system flows. **Agent self-service** surfaces (inbox/outbox, consent, auth-request tracking, profile, tasks) are in scope as their own folder. **System (headless) behaviours** get a separate folder with requirements cross-referenced from the admin page that provisions them. | ⏳ |
| **Phases (admin journey)** | 9 journey phases → 14 admin page files + 1 overview. | ⏳ (R3–R12) |
| **Agent self-service pages** | 5 pages covering inbox/outbox, auth-request tracking, consent, work (tasks+sessions), profile+grants. Each page's "actor" is an LLM Agent or Human Agent. UI sketch may be a tool-invocation shape rather than a human UI mockup where appropriate. | ⏳ (R13) |
| **Requirement phrasing** | Use `SHALL` for binding requirements, `SHOULD` for recommended. Each requirement has a short ID: `R-ADMIN-<page>-<n>`, `R-AGENT-<page>-<n>`, `R-SYS-<flow>-<n>`, `R-NFR-<area>-<n>`. | ⏳ |

## Output Folder Structure  `[PLAN: new]` `[DOCS: ⏳ pending]`

```
baby-phi/docs/specs/v0/requirements/
  README.md                                ← framing + folder map + NFR definition + requirement-ID convention
  _template/
    admin-page-template.md                 ← the 10-section template every admin/agent page follows

  admin/
    00-fresh-install-journey-overview.md   ← one-pager mapping the 9 phases to the 14 pages
    01-platform-bootstrap-claim.md         ← Phase 1
    02-platform-model-providers.md         ← Phase 2 (1/4)
    03-platform-mcp-servers.md             ← Phase 2 (2/4)
    04-platform-credentials-vault.md       ← Phase 2 (3/4)
    05-platform-defaults.md                ← Phase 2 (4/4) — retention, audit class, auth-request defaults
    06-org-creation-wizard.md              ← Phase 3
    07-organization-dashboard.md           ← Phase 4
    08-agent-roster-list.md                ← Phase 5 (1/2)
    09-agent-profile-editor.md             ← Phase 5 (2/2) — AgentProfile + ModelConfig + ExecutionLimits + parallelize
    10-project-creation-wizard.md          ← Phase 6 (1/2) — OKRs + resource boundaries
    11-project-detail.md                   ← Phase 6 (2/2) — task board, subprojects, OKR tracking
    12-authority-template-adoption.md      ← Phase 7
    13-system-agents-config.md             ← Phase 8
    14-first-session-launch.md             ← Phase 9 — "hello world" validation

  agent-self-service/
    README.md                              ← index of agent-facing surfaces
    a01-my-inbox-outbox.md                 ← view received/sent AgentMessage on inbox_object / outbox_object
    a02-my-auth-requests.md                ← inbound (to approve as owner/co-owner) + outbound (to track submissions)
    a03-my-consent-records.md              ← acknowledge / decline / revoke Consent nodes
    a04-my-work.md                         ← assigned tasks (ASSIGNED_TO) + current/recent sessions
    a05-my-profile-and-grants.md           ← Identity node view + list of grants I hold

  system/
    README.md                              ← index of reactive/headless behaviours
    s01-bootstrap-template-adoption.md     ← system:genesis → platform admin grant
    s02-session-end-memory-extraction.md   ← memory-extraction-agent trigger
    s03-edge-change-catalog-update.md      ← agent-catalog-agent trigger
    s04-auth-request-state-transitions.md  ← state machine + notifications
    s05-template-adoption-grant-fires.md   ← Authority Templates A–E fire on edge events
    s06-periodic-triggers.md               ← retention archival, secret rotation, monitoring

  cross-cutting/
    README.md                              ← NFR index + traceability summary
    nfr-performance.md                     ← latency, throughput, concurrency targets
    nfr-observability.md                   ← audit events, metrics, logs
    nfr-security.md                        ← the permission model's security properties translated to testable NFRs
    nfr-cost.md                            ← token budget enforcement, cost accounting
    traceability-matrix.md                 ← concept section → requirement IDs
```

## Per-Admin-Page File Template  `[PLAN: new]` `[DOCS: ⏳ (R2)]`

Every file in `admin/` conforms to:

1. **Header stub** (Status: CONCEPTUAL, Last verified, part-of-requirements-spec note).
2. **Page Purpose + Primary Actor** — one paragraph: what the page does; who uses it (platform admin / org CEO / project lead).
3. **Position in the Journey** — which phase (1–9); pages this depends on; pages that depend on this.
4. **UI Sketch** — ASCII mockup of the page, showing the main widgets and their states. Empty state, populated state, error state.
5. **Read Requirements** — what the page DISPLAYS. One requirement per distinct read. Each: `R-ADMIN-<page>-R<n>: The page SHALL display {field} derived from {source concept / graph node}.`
6. **Write Requirements** — what the admin CAN CHANGE. Each: `R-ADMIN-<page>-W<n>: The admin SHALL be able to {action}, which produces {backend effect}.` Includes validation rules and error cases.
7. **Permission / Visibility Rules** — who can see this page; who can invoke each write. Maps directly to concept grants. Cites the grant or Authority Template that authorises.
8. **Event & Notification Requirements** — what the admin is notified about (new Auth Requests needing approval, alerted audit events, pending Consent responses, etc.).
9. **Backend Actions Triggered** — cross-links to `system/` files for reactive behaviours this page's writes trigger (e.g., "creating an agent triggers `s03-edge-change-catalog-update`").
10. **API Contract Sketch** — REST-ish endpoints the page calls. Method, path, request payload shape, response shape, errors. Not finalised — a sketch showing the shape of the eventual surface.
11. **Acceptance Scenarios** — 2–4 concrete scenarios grounded in the 10 org / 5 project layouts, phrased as Given/When/Then. Example: "Given a fresh install and no orgs, When the platform admin completes the org-creation wizard for `minimal-startup` (mirroring `organizations/01-minimal-startup.md`), Then the resulting organization has `consent_policy: implicit`, both system agents in the roster, and a populated `resources_catalogue`."
12. **Cross-References** — concept files implemented by this page; phi-core types used; related admin pages; related system flows.

## Worked Example of the Template — `08-agent-roster-list.md`

A full draft of one admin page, showing every section populated. This doubles as the first draft of the actual file: R8 execution will copy this into `admin/08-agent-roster-list.md` and refine.

---

### 1. Header stub

```
<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Requirements doc — admin page, Phase 5 of fresh-install journey -->
```

### 2. Page Purpose + Primary Actor

The Agent Roster List page lets a Human Agent with org-admin authority see all Agents in the current organization and manage their lifecycle (add, archive, view detail).

**Primary actor:** Human Agent holding `[allocate]` on the org's `agent-catalogue` control_plane_object (typically the org's CEO; sometimes a separate delegated admin).

**Secondary actors:** Human Agents and LLM Agents who are project leads — they access the page in read-only mode, with visibility filtered to agents in projects they lead (via Template A). Per the terminology note at the top of this plan, "admin" throughout is shorthand for the Human Agent holding the relevant grant.

### 3. Position in the Journey

- **Phase:** 5 of 9 (Build agent roster).
- **Depends on:** Phase 3 (org created), Phase 4 (org dashboard).
- **Enables:** Phase 6 (projects can reference these agents), Phase 7 (Template A fires as leads are created), Phase 9 (first session needs an agent).

### 4. UI Sketch

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

### 5. Read Requirements

- **R-ADMIN-08-R1:** The page SHALL display all Agents where a `MEMBER_OF` edge points to the current organization, showing id, kind (Human/Intern/Contract/System), model, current rating (if any), active-session count, and last-seen timestamp.
- **R-ADMIN-08-R2:** The page SHALL display a summary count by kind at the bottom of the list.
- **R-ADMIN-08-R3:** The page SHALL display the org's compute utilisation: `active_sessions / max_concurrent_sessions` from the org's `compute_resources` catalogue entry.
- **R-ADMIN-08-R4:** The page SHALL support filtering by kind (multi-select) and role (derived from project membership); filtering is client-side.
- **R-ADMIN-08-R5:** The page SHALL support substring search on agent id and `AgentProfile.name`.
- **R-ADMIN-08-R6:** System Agents SHALL be visually distinguished from Standard Agents (icon/badge).
- **R-ADMIN-08-R7:** The page SHALL show a live indicator of `agent-catalog-agent`'s current state (syncing / lag / error) sourced from that agent's health endpoint.

### 6. Write Requirements

- **R-ADMIN-08-W1:** The admin SHALL be able to create a new Agent via the "+ Add Agent" action, navigating to [09-agent-profile-editor.md](09-agent-profile-editor.md) in "new agent" mode.
- **R-ADMIN-08-W2:** The admin SHALL be able to archive an existing Agent via the row's "⋯" menu. Archival marks the Agent inactive but retains the node; archived agents appear only under an "Archived" filter.
- **R-ADMIN-08-W3:** Archiving SHALL NOT delete the archived agent's `inbox_object` / `outbox_object` composites — those remain readable by authorised auditors until Auth Request retention applies.
- **R-ADMIN-08-W4:** The admin SHALL NOT be able to archive System Agents from this page; the row's "⋯" menu on a System Agent is disabled. The "View" link opens [13-system-agents-config.md](13-system-agents-config.md) instead, where disabling is allowed.

### 7. Permission / Visibility Rules

All rules resolve to grants held by the **Human Agent** viewing or acting on the page. No ambient "admin" privilege — a Human Agent's ability to see or do anything here is determined by the grants on their Agent node.

- **Page access** — requires `[read, list, inspect]` on `identity_principal` scoped to the current org. Held by the Human Agent with platform-admin or org-admin role (via the org's Default Grants) and by project leads (via Template A, filtered to their project).
- **Add / archive actions** — require `[allocate]` on the org's `agent-catalogue` `control_plane_object` instance. Held by the Human Agent with platform-admin or org-admin role. Project-lead Agents do NOT hold this grant and see the buttons disabled.
- **Role-filter visibility** — a project-lead Agent sees only the intersection of the roster with agents in projects they lead; the filter UI greys out checkboxes that would produce no rows for them.
- **Template A exercised** — project leads' view of *worker sessions* is not this page's concern, but this roster is the entry point to pages that do. See [09-agent-profile-editor.md § Permissions](09-agent-profile-editor.md).

### 8. Event & Notification Requirements

- **R-ADMIN-08-N1:** When a new Agent is created via W1, the page SHALL display a success toast "Agent `<id>` created. Inbox and outbox auto-created."
- **R-ADMIN-08-N2:** When an Agent is archived via W2, the page SHALL require a confirmation dialog showing (a) the agent's active session count, (b) the count of grants that will cascade-revoke. On confirmation, an audit toast displays the resulting `AgentArchived` event ID.
- **R-ADMIN-08-N3:** The page SHALL show a live-updating indicator of `agent-catalog-agent`'s health (polling or WebSocket). On `error` state the page displays a banner "Agent catalog is stale — recent roster changes may not be reflected."

### 9. Backend Actions Triggered

Creating an Agent (W1) triggers:
- Agent node created; `MEMBER_OF` edge added to the org.
- Default Grants issued (per [permissions/05 § Default Grants Issued to Every Agent](../../concepts/permissions/05-memory-sessions.md#default-grants-issued-to-every-agent)).
- `inbox_object` and `outbox_object` composite instances atomically added to the org's `resources_catalogue` (see [system/s03-edge-change-catalog-update.md](../system/s03-edge-change-catalog-update.md)).
- `agent-catalog-agent` updates its index on the resulting `edge_change` trigger.
- Audit event `AgentCreated { agent_id, kind, created_by, org_id, profile_snapshot }` emitted.

Archiving an Agent (W2) triggers:
- Agent node's `active` set to false; `MEMBER_OF` edge retained for audit.
- All currently-live sessions for this Agent terminated with reason `agent_archived`.
- Grants held by the Agent revoked forward-only (cascades per [permissions/02 § Retention Policy](../../concepts/permissions/02-auth-request.md#retention-policy) + the forward-only revocation rule).
- Audit event `AgentArchived { agent_id, archived_by, session_count_terminated, grant_count_revoked }` emitted.

### 10. API Contract Sketch

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

### 11. Acceptance Scenarios

**Scenario 1 — minimal-startup first agent.**
*Given* a freshly-created `minimal-startup` organization (mirroring [organizations/01-minimal-startup.md](../../organizations/01-minimal-startup.md)) with no agents yet, *When* the founder navigates to the agent roster list and clicks "+ Add Agent", *Then* they are taken to the agent profile editor in "new agent" mode; after saving, the new Agent appears in the roster list with `kind: Intern`, its `inbox_object` and `outbox_object` exist in the org's `resources_catalogue`, and the default grants are issued.

**Scenario 2 — mid-product-team project-lead Agent visibility.**
*Given* the `mid-product-team` org (mirroring [organizations/02-mid-product-team.md](../../organizations/02-mid-product-team.md)) with its full 14-agent roster populated, *When* the Contract Agent `lead-stream-a` (a project-lead LLM Agent in this org) invokes the roster-read tool surface, *Then* they see the full roster but the "+ Add Agent" and "⋯ Archive" controls are absent from the response (they lack `[allocate]` on the agent-catalogue control plane); filtering by project `stream-a` shows only the 4 agents in their stream. Note that `lead-stream-a` is an LLM Agent — the same page renders as a programmatic surface for them whereas the CEO Human Agent uses the human-UI form of the same page.

**Scenario 3 — cannot archive a System Agent.**
*Given* any org with the default System Agents in its roster, *When* the org admin attempts to click "⋯ → Archive" on `memory-extraction-agent`, *Then* the "Archive" menu item is disabled with a tooltip "System Agents cannot be archived from this page — use System Agents Configuration (page 13) to disable or reconfigure."

**Scenario 4 — archiving an Intern terminates live sessions.**
*Given* an Intern agent currently running 2 concurrent sessions (`parallelize: 2`), *When* the org admin archives the agent with reason "promotion to Contract", *Then* both sessions are terminated with `reason: agent_archived`; the Agent node's `active` becomes false; the archival audit event records `session_count_terminated: 2` and the count of revoked grants. A follow-up Auth Request for the new Contract-tier profile is the separate flow that creates the successor agent.

### 12. Cross-References

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

---

## Edit Plan (R-steps)  `[PLAN: new]` `[DOCS: ⏳ pending]`

### R0: Archive this plan  `[PLAN: new]` `[DOCS: ⏳]`

First action after approval. Create the `baby-phi/docs/specs/plan/requirements/` folder if it doesn't exist. Copy this plan verbatim to `baby-phi/docs/specs/plan/requirements/<random>-fresh-install-admin-journey.md` (8-hex-char token, matching convention).

### R1: Create `v0/requirements/` folder + top-level README  `[PLAN: new]` `[DOCS: ⏳]`

README contains:
- Framing paragraph.
- Folder map.
- v0 vs v1 scope policy.
- Requirement-ID convention.
- Link to the admin-page template.
- Pointer to the traceability matrix.

### R2: Write `_template/admin-page-template.md`  `[PLAN: new]` `[DOCS: ⏳]`

The normative 10-section template. Every admin page is expected to match this structure. Includes a short worked example at the bottom so page authors know what each section looks like populated.

### R3: Admin journey overview (`admin/00-...`)  `[PLAN: new]` `[DOCS: ⏳]`

A one-page summary mapping:

| Phase | Goal | Admin pages | Concept files |
|-------|------|-------------|---------------|
| 1 | Claim platform admin role | 01 | system-agents, permissions/02 § System Bootstrap Template |
| 2 | Platform-level resource setup | 02, 03, 04, 05 | permissions/01 § Resource Catalogue, organization.md |
| 3 | Create first organization | 06 | organization.md, permissions/07 § Standard Org Template |
| 4 | Organization home | 07 | organization.md, all 10 org layouts |
| 5 | Build agent roster | 08, 09 | agent.md, phi-core AgentProfile/ModelConfig/ExecutionLimits |
| 6 | Create first project | 10, 11 | project.md, OKR value objects, permissions/05 |
| 7 | Approve authority template adoptions | 12 | permissions/05 Authority Templates, permissions/07 |
| 8 | Configure system agents | 13 | system-agents.md |
| 9 | Launch first session | 14 | agent.md, permissions/05 Sessions |

### R4: Phase 1 — `01-platform-bootstrap-claim.md`  `[PLAN: new]` `[DOCS: ⏳]`

Covers: admin uses the bootstrap credential from install; System Bootstrap Template's adoption Auth Request is materialised; `[allocate]` on `system:root` is granted to the platform admin; first alerted audit event emitted. Cross-references `system-agents.md § Bootstrap as a Template` and `permissions/02 § System Bootstrap Template`.

### R5: Phase 2 — platform settings (4 pages)  `[PLAN: new]` `[DOCS: ⏳]`

- `02-platform-model-providers.md` — register `model/runtime_object` instances (Anthropic, OpenAI, local).
- `03-platform-mcp-servers.md` — register `external_service_object` instances (MCP servers).
- `04-platform-credentials-vault.md` — store `secret/credential` entries with custodian assignments.
- `05-platform-defaults.md` — default `consent_policy`, `audit_class`, Auth Request retention, execution limit templates.

Each adds entries to the platform-level catalogue; each addition emits an alerted audit event.

### R6: Phase 3 — `06-org-creation-wizard.md`  `[PLAN: new]` `[DOCS: ⏳]`

Multi-step wizard:
1. Basics (name, vision, mission).
2. Template selection (Standard Org Template + customisation deltas).
3. Consent policy + audit class defaults.
4. Authority Templates enabled set.
5. Resource catalogue declaration (with import-from-platform option).
6. Execution limits + initial token budget.
7. First sponsor (human) + contact channels.
8. Review + submit.

On submit: Organization node created; system agents auto-added; default grants issued; adoption Auth Requests for enabled templates created and auto-approved by the org admin.

### R7: Phase 4 — `07-organization-dashboard.md`  `[PLAN: new]` `[DOCS: ⏳]`

The post-creation home. Empty state with CTAs: "Add agent," "Create project," "Adopt template." Also the steady-state home once populated. Displays: active agents count, active projects count, pending Auth Requests count, recent alerted events, token budget usage.

### R8: Phase 5 — agent roster (2 pages)  `[PLAN: new]` `[DOCS: ⏳]`

- `08-agent-roster-list.md` — list view + add/archive controls.
- `09-agent-profile-editor.md` — detail editor using phi-core `AgentProfile` + `ModelConfig` + `ExecutionLimits` + `parallelize` + `kind` (Intern/Contract/System). Creating an agent auto-creates `inbox_object` and `outbox_object`.

### R9: Phase 6 — project (2 pages)  `[PLAN: new]` `[DOCS: ⏳]`

- `10-project-creation-wizard.md` — owning org(s), OKRs (Objectives + Key Results using the value-object schema from `project.md`), resource boundaries (subset of owning org(s)' catalogue), initial lead + members.
- `11-project-detail.md` — task board, sub-project tree, OKR progress view, per-project agent roster, resource boundaries editor.

### R10: Phase 7 — `12-authority-template-adoption.md`  `[PLAN: new]` `[DOCS: ⏳]`

Review the adoption Auth Requests that the org-creation wizard generated. Approve/decline per template. Approved adoptions gain visibility as "active templates"; each subsequent grant they fire cites the adoption Auth Request as provenance.

### R11: Phase 8 — `13-system-agents-config.md`  `[PLAN: new]` `[DOCS: ⏳]`

Confirm `memory-extraction-agent` + `agent-catalog-agent` are running with their default `parallelize` settings. Add org-specific system agents (compliance-audit-agent, grading-agent, etc.) following the pattern in `concepts/system-agents.md`. Trigger selection: `session_end`, `edge_change`, `periodic`, `explicit`, etc.

### R12: Phase 9 — `14-first-session-launch.md`  `[PLAN: new]` `[DOCS: ⏳]`

The "hello world" validation page. Pick an agent + project, start a session, observe it land in the session browser (steady-state page, cross-referenced) and the audit feed. On session end, confirm that `s02-session-end-memory-extraction` fires.

### R13: `agent-self-service/` (5 pages)  `[PLAN: new]` `[DOCS: ⏳]`

Each follows the same 10-section template as admin pages. Primary actor is an Agent (LLM or Human). Where a human-UI sketch doesn't fit (e.g., programmatic LLM-agent inbox reads), the "UI Sketch" section becomes a **tool-invocation shape**: the tool manifest the agent uses + example call/response. This parallels the 14 tool manifest examples in `concepts/permissions/07`.

- **`a01-my-inbox-outbox.md`** — read inbox (`AgentMessage` entries on `inbox_object`), send a new message (appends to target's inbox, records in own outbox), mark messages read/archived. Grants: agent's default `[read, list, recall, delete]` on own inbox and `[read, list, send]` on own outbox.
- **`a02-my-auth-requests.md`** — two views on the same page:
  - **Inbound** (as approver slot-holder): list Auth Requests where the agent holds a slot; approve, deny, reconsider (re-unfill), escalate. Tied to `permissions/02 § Per-State Access Matrix`.
  - **Outbound** (as requestor): list Auth Requests the agent submitted; see state, approver responses, resulting Grant. Cancel in Draft/Pending/In Progress states.
- **`a03-my-consent-records.md`** — view all Consent records scoped to this agent. Acknowledge requested consents, decline them, or revoke previously-acknowledged ones (forward-only semantics per `permissions/06 § Consent Revocation Semantics`).
- **`a04-my-work.md`** — tasks assigned to the agent (`ASSIGNED_TO` edges) and sessions the agent is running or has recently run. Links to each session's session_object. Cross-refs agent's current `parallelize` state (how many concurrent sessions are currently live).
- **`a05-my-profile-and-grants.md`** — view own Identity node (`self_description`, `lived`, `witnessed`, `embedding`) and the list of Grants the agent currently holds. Read-only browser — grants are held, not modified from here.

### R14: `system/` files (6 flows)  `[PLAN: new]` `[DOCS: ⏳]`

Each file: purpose, trigger, preconditions, behaviour (pseudocode where load-bearing), side effects, failure modes, observability (what audit events fire), cross-references to admin pages that provision this flow.

- `s01-bootstrap-template-adoption.md` — provisioned by admin page 01.
- `s02-session-end-memory-extraction.md` — provisioned by admin page 13 (agent config).
- `s03-edge-change-catalog-update.md` — provisioned by admin page 13.
- `s04-auth-request-state-transitions.md` — the full state machine from `permissions/02` expressed as requirements.
- `s05-template-adoption-grant-fires.md` — provisioned by admin page 12.
- `s06-periodic-triggers.md` — retention archival, secret rotation, monitoring heartbeats.

### R15: `cross-cutting/` (5 files)  `[PLAN: new]` `[DOCS: ⏳]`

- `nfr-performance.md` — read p95, write p95, Permission Check latency, concurrency targets.
- `nfr-observability.md` — audit event schema, metric enumeration, log retention.
- `nfr-security.md` — permission-model security properties phrased as testable statements (no ambient resource access, all grants have traceable provenance, etc.).
- `nfr-cost.md` — token budget enforcement at org/project/agent scope, cost accounting event schema.
- `traceability-matrix.md` — one table, rows = concept sections, columns = requirement IDs that cover the section. Populated incrementally as pages are written.

### R16: Cross-link from existing concept docs  `[PLAN: new]` `[DOCS: ⏳]`

- `concepts/README.md` — add a "Requirements" row to the index pointing at `../requirements/README.md`.
- `organizations/README.md` and `projects/README.md` — note that the admin pages that create these shapes live under `../requirements/admin/`.

### R17: Verification  `[PLAN: new]` `[DOCS: ⏳]`

1. **Structural check** — all 15 admin files + 5 agent-self-service pages + 6 system files + 4 NFRs + traceability matrix + section READMEs + template exist.
2. **Template conformance** — every admin and agent-self-service page has all 10 template sections populated (or explicit "N/A — see X" for any section that genuinely doesn't apply).
3. **Requirement ID uniqueness** — `grep -rE 'R-(ADMIN|AGENT|SYS|NFR)-' baby-phi/docs/specs/v0/requirements/` returns no duplicate IDs.
4. **Concept coverage** — for each major section heading across `agent.md`, `organization.md`, `project.md`, `permissions/01`–`permissions/07`, `system-agents.md`, `token-economy.md`, and `coordination.md § Design Decisions`, at least one requirement cites it in its Cross-References section.
5. **Scenario grounding** — every admin page's Acceptance Scenarios reference at least one of the 10 org layouts or 5 project layouts.
6. **Cross-reference resolution** — every `](..)` link in requirement files resolves.

## Critical Files  `[PLAN: new]` `[DOCS: n/a — reference list]`

| File | Edit(s) |
|------|---------|
| `baby-phi/docs/specs/plan/requirements/` (NEW folder) | R0 — archive this plan here |
| `baby-phi/docs/specs/plan/requirements/<random>-fresh-install-admin-journey.md` (NEW) | R0 |
| `baby-phi/docs/specs/v0/requirements/` (NEW folder) | R1–R15 |
| `baby-phi/docs/specs/v0/requirements/README.md` | R1 |
| `baby-phi/docs/specs/v0/requirements/_template/admin-page-template.md` | R2 |
| `baby-phi/docs/specs/v0/requirements/admin/00-..14-` (15 files) | R3–R12 |
| `baby-phi/docs/specs/v0/requirements/agent-self-service/README.md` + a01–a05 | R13 |
| `baby-phi/docs/specs/v0/requirements/system/README.md` + s01–s06 | R14 |
| `baby-phi/docs/specs/v0/requirements/cross-cutting/` (5 files + README) | R15 |
| `baby-phi/docs/specs/v0/concepts/README.md` | R16 (cross-link) |
| `baby-phi/docs/specs/v0/organizations/README.md` | R16 (cross-link) |
| `baby-phi/docs/specs/v0/projects/README.md` | R16 (cross-link) |

## Projected Outcome  `[PLAN: new]` `[DOCS: n/a — projection]`

**Not a confidence %** — requirements coverage is binary per concept section. Target:

- Every concept rule in agent.md, organization.md, project.md, permissions/, system-agents.md, and token-economy.md maps to at least one requirement in admin/, agent-self-service/, or system/.
- Every admin page has between 5 and 20 requirements (smaller pages at the low end, the org-creation wizard at the high end).
- Every agent-self-service page has between 4 and 12 requirements.
- Every system flow file has between 3 and 10 requirements.
- Cross-cutting NFRs total 15–25 requirements.
- Total expected requirements: ~250–350 across the folder, with ID patterns `R-ADMIN-*`, `R-AGENT-*`, `R-SYS-*`, `R-NFR-*`.

**Coverage measurement:** after landing, one Explore agent reads both `concepts/` and `requirements/` and produces a coverage report (concept sections with no requirement reference = gaps). Same protocol as the confidence evals but binary (covered / not covered).

## What Stays Unchanged  `[PLAN: new]` `[DOCS: n/a — scope guard]`

- All concept files. Requirements derive from the concept; the concept is not edited.
- The 10 org + 5 project layout catalogue. Requirements reference the layouts as acceptance scenarios but do not modify them.
- phi-core types. Requirements cite them; phi-core source of truth stays in the crate.
- The existing `plan/` flat folder. New concept plans continue to land there; requirements plans land in `plan/requirements/`.

## Verification Summary  `[PLAN: new]` `[DOCS: ⏳ pending]`

1. Plan archived to `plan/requirements/`.
2. `v0/requirements/` folder structure complete.
3. All 15 admin files (14 pages + overview) + 5 agent-self-service pages + 6 system flow files + 5 cross-cutting files + section READMEs + template written.
4. Every requirement has a unique ID.
5. Coverage report (R17 step 4) shows every major concept section cited by ≥1 requirement.
6. Cross-references resolve.
