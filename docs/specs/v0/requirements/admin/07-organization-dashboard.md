<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Requirements doc — admin page, Phase 4 of fresh-install journey -->

# 07 — Organization Dashboard

## 2. Page Purpose + Primary Actor

The post-creation home for an organization. Serves as the landing page for the Human Agent with org-admin authority (the CEO from Phase 3) and as the steady-state operations home once the org is populated. Shows the state of the org at a glance: active agents, active projects, pending Auth Requests requiring attention, recent alerted audit events, token-budget utilisation.

On a freshly-created org, most counters are zero — the page presents **call-to-action cards** (Add Agent, Create Project, Adopt Template, Configure System Agents) that link to the corresponding Phase 5–8 pages.

**Primary actor:** Human Agent with org-admin authority (CEO). Also navigable read-only by project leads and any Agent with membership in the org.

## 3. Position in the Journey

- **Phase:** 4 of 9 (Organization home).
- **Depends on:** Phase 3 complete (org exists).
- **Enables:** acts as the entry point into Phases 5–8; steady-state home thereafter.

## 4. UI Sketch

Freshly-created state:

```
┌─────────────────────────────────────────────────────────────────┐
│ Acme Corporation                               agent:alex (CEO) │
├─────────────────────────────────────────────────────────────────┤
│ Welcome — your organization is ready to set up.                 │
│                                                                  │
│ ┌─────────────────────┐  ┌─────────────────────┐                │
│ │ 👥 Add Agent         │  │ 📋 Create Project    │                │
│ │ Build your roster    │  │ Scope the first work │                │
│ │ → Phase 5            │  │ → Phase 6            │                │
│ └─────────────────────┘  └─────────────────────┘                │
│                                                                  │
│ ┌─────────────────────┐  ┌─────────────────────┐                │
│ │ 📜 Templates         │  │ ⚙ System Agents      │                │
│ │ Review & approve     │  │ Review & tune        │                │
│ │ → Phase 7            │  │ → Phase 8            │                │
│ └─────────────────────┘  └─────────────────────┘                │
│                                                                  │
│ Current state:  Agents: 2 (both System)  Projects: 0            │
│                 Templates adopted: 2 (A, B)                     │
│                 Token budget: 0 / 5,000,000 tokens used         │
└─────────────────────────────────────────────────────────────────┘
```

Populated state (post-Phase 9):

```
┌─────────────────────────────────────────────────────────────────┐
│ Acme Corporation                              agent:alex (CEO)  │
├─────────────────────────────────────────────────────────────────┤
│ Summary                                                          │
│   👥 Agents: 6 (1 Human, 2 Contract, 1 Intern, 2 System)        │
│   📋 Projects: 2 active (stream-a, stream-b)                    │
│   📬 Pending Auth Requests: 3 awaiting your approval             │
│   🔔 Alerted events (24h): 1                                    │
│   💰 Token budget: 1.2M / 5M (24%)                              │
│                                                                  │
│ Recent activity                                                 │
│   10:42  AgentCreated: coder-acme-2                             │
│   11:08  OrganizationCreated: stream-a                          │
│   11:33  SessionStarted: s-9901                                 │
│                                                                  │
│ [Add Agent]  [Create Project]  [View All Agents]  [View Audit]  │
└─────────────────────────────────────────────────────────────────┘
```

Error state: per-tile "failed to load" with retry.

## 5. Read Requirements

- **R-ADMIN-07-R1:** The page SHALL display the org's name, vision, mission, and the viewing Human Agent's role in that org.
- **R-ADMIN-07-R2:** The page SHALL display a summary row of Agents grouped by kind (Human / Intern / Contract / System).
- **R-ADMIN-07-R3:** The page SHALL display a summary row of Projects (count of active; breakdown by Shape A vs B).
- **R-ADMIN-07-R4:** The page SHALL display the count of Auth Requests where the viewing Human Agent holds an unfilled approver slot.
- **R-ADMIN-07-R5:** The page SHALL display the count of alerted audit events in the last 24 hours.
- **R-ADMIN-07-R6:** The page SHALL display token-budget utilisation as `used / total` from the org's `token-budget-pool` economic_resource.
- **R-ADMIN-07-R7:** The page SHALL display the most recent 5 audit events (regardless of audit_class), clickable to jump to the steady-state audit log.
- **R-ADMIN-07-R8:** On a freshly-created org (all counters zero or near-zero), the page SHALL surface 4 call-to-action cards linking to Phase 5–8 entry pages.
- **R-ADMIN-07-R9:** The page SHALL show the list of Authority Templates currently adopted by the org.

## 6. Write Requirements

- **R-ADMIN-07-W1:** The call-to-action cards SHALL navigate to the corresponding pages (08, 10, 12, 13) on click.
- **R-ADMIN-07-W2:** The page itself has no direct writes — all mutations happen on downstream pages. This section is intentionally sparse.

## 7. Permission / Visibility Rules

- **Page access** — any Agent with `MEMBER_OF` to the org OR holding any Grant `APPLIES_TO` this org can view the page in read-only mode. The viewing Human Agent's role determines which summary tiles are shown (project leads see a filtered view restricted to their projects).
- **Summary tiles with "requires action"** (pending Auth Requests, alerted events) — shown only if the viewer holds the relevant grant:
  - Pending Auth Requests: only if the viewer has unfilled approver slots.
  - Alerted events (24h): requires `[read, list, inspect]` on `control_plane_object:audit_log`. Held by org admin; not by ordinary agents.
- **Call-to-action cards** — visible only if the viewer can actually complete the target page's writes. Project leads see "View Projects" instead of "Create Project".

## 8. Event & Notification Requirements

- **R-ADMIN-07-N1:** The pending-Auth-Requests tile SHALL update live (poll every 30s or WebSocket push).
- **R-ADMIN-07-N2:** Alerted event count SHALL update live.
- **R-ADMIN-07-N3:** A first-visit banner SHALL appear on a freshly-created org: "Welcome to `<org>`. Start with Phase 5 to build your agent roster."

## 9. Backend Actions Triggered

None directly — this is a read-only page. Navigation links transition to downstream pages where writes happen.

## 10. API Contract Sketch

```
GET  /api/v0/orgs/{org_id}/dashboard
     → 200: {
       org: { id, name, vision, mission },
       viewer: { agent_id, role, can_admin_manage: bool },
       agents_summary: { total, by_kind: { Human, Intern, Contract, System } },
       projects_summary: { active, shape_a, shape_b },
       pending_auth_requests_count: u32,
       alerted_events_24h: u32,
       token_budget: { used, total, pool_id },
       recent_events: [{id, kind, actor, timestamp, summary}...5],
       templates_adopted: [A, B, ...]
     }
     → 403: Agent has no relation to this org
```

## 11. Acceptance Scenarios

**Scenario 1 — freshly-created minimal-startup.**
*Given* [organizations/01-minimal-startup.md](../../organizations/01-minimal-startup.md) has just been created via the wizard (Agents: 2 System only, Projects: 0, Templates: [A, B] adopted, token budget: 5M unused), *When* the CEO Human Agent visits this page, *Then* the dashboard shows the 4 CTA cards (Add Agent / Create Project / Templates / System Agents), the agents-summary shows 2 (System), and the welcome banner is displayed.

**Scenario 2 — project lead read-only view.**
*Given* [organizations/02-mid-product-team.md](../../organizations/02-mid-product-team.md) with 14 agents and 3 projects, *When* the project-lead Agent `lead-stream-a` visits this page, *Then* the dashboard shows a filtered view: agents in their project (4), projects they lead (1, `stream-a`), no "pending Auth Requests for you" tile (they don't approve), and "Create Project" is replaced with "View Projects". Alerted-events tile is hidden (no `[read]` on audit log at org level).

## 12. Cross-References

**Concept files:**
- [concepts/organization.md § Organization Edges](../../concepts/organization.md) — the graph relations this dashboard reads.
- [concepts/permissions/02 § Per-State Access Matrix](../../concepts/permissions/02-auth-request.md#per-state-access-matrix) — for the "pending approvals" count.

**Related admin pages:**
- [08-agent-roster-list.md](08-agent-roster-list.md), [10-project-creation-wizard.md](10-project-creation-wizard.md), [12-authority-template-adoption.md](12-authority-template-adoption.md), [13-system-agents-config.md](13-system-agents-config.md) — the 4 CTA targets.

**Org layouts exercised in Acceptance Scenarios:**
- [organizations/01-minimal-startup.md](../../organizations/01-minimal-startup.md), [organizations/02-mid-product-team.md](../../organizations/02-mid-product-team.md).
