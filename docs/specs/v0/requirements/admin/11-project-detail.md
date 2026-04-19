<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Requirements doc — admin page, Phase 6 of fresh-install journey -->

# 11 — Project Detail

## 2. Page Purpose + Primary Actor

The home for a single Project. Shows OKR progress, task board, sub-project tree, per-project agent roster, resource boundaries, and links to sessions run in this project. Read-accessible to any Agent with membership in the project; write actions gated per-control.

**Primary actor:** the project's `HAS_LEAD` Agent (Human or LLM Contract) — full edit access within the project's scope. **Secondary:** project members (read + task status updates for tasks they own). **Observer:** org admin, sponsors.

## 3. Position in the Journey

- **Phase:** 6 of 9 — page 2 of 2.
- **Depends on:** [10-project-creation-wizard.md](10-project-creation-wizard.md) has produced a Project node.
- **Enables:** Phase 9 (sessions launched here).

## 4. UI Sketch

```
┌───────────────────────────────────────────────────────────────────┐
│ Solo Sprint MVP — minimal-startup                [Edit] [Delete] │
├───────────────────────────────────────────────────────────────────┤
│ Status: ● InProgress (35%)    Duration: Week 2 of 6              │
│ Goal: MVP shipped and validated.                                 │
│                                                                    │
│ ┌─ OKRs ──────────────────────────────────────────────────────┐  │
│ │ OBJ  Ship the MVP                Active     Deadline 05-15 │  │
│ │   KR Core features shipped        2/3  ▮▮▯  05-12         │  │
│ │   KR Production deployment        0/1  ▯▯▯  05-15         │  │
│ │ OBJ  Validate with users         Active     Deadline 05-20 │  │
│ │   KR User interviews completed    3/5  ▮▮▯  05-10         │  │
│ │   KR Paying users committed       0/3  ▯▯▯  05-20         │  │
│ └──────────────────────────────────────────────────────────────┘  │
│                                                                    │
│ ┌─ Tasks ─────────────────────────────────────────────────────┐  │
│ │ [Open] Auth flow         intern-a   linked KR: features   │  │
│ │ [Open] Stripe billing    intern-b   linked KR: features   │  │
│ │ [Done] Core screens      intern-a                         │  │
│ └──────────────────────────────────────────────────────────────┘  │
│                                                                    │
│ Sub-projects (0):   — [+ Add sub-project]                         │
│ Roster (3):  founder (lead), intern-a, intern-b                  │
│ Active sessions (1):  s-9901 by intern-a  [→ Sessions]           │
│ Resource boundaries: [view]                                       │
└───────────────────────────────────────────────────────────────────┘
```

Empty state (freshly created): OKR, Task, Sub-project panels all empty with "[+ Add X]" CTAs.

## 5. Read Requirements

- **R-ADMIN-11-R1:** The page SHALL display the Project's name, description, goal, status, progress, duration-window, token budget used vs allocated.
- **R-ADMIN-11-R2:** The page SHALL display all Objectives with their KRs, current vs target, status, deadline.
- **R-ADMIN-11-R3:** The page SHALL display the task list with status, assigned agent, and linked KR.
- **R-ADMIN-11-R4:** The page SHALL display the sub-project tree (via `HAS_SUBPROJECT` edges) with each sub-project's status and progress.
- **R-ADMIN-11-R5:** The page SHALL display the project's agent roster (via `HAS_AGENT` / `HAS_LEAD` / `HAS_SPONSOR` edges) with each agent's role.
- **R-ADMIN-11-R6:** The page SHALL list active and recent sessions tagged with this project.
- **R-ADMIN-11-R7:** The page SHALL link to the resource boundaries viewer (a sub-page or drawer).

## 6. Write Requirements

- **R-ADMIN-11-W1:** The lead SHALL be able to edit project basics (name, description, goal, status) inline. Status transitions (`InProgress → OnHold`, etc.) require a reason.
- **R-ADMIN-11-W2:** The lead SHALL be able to add / edit / archive Objectives and Key Results inline. Current value updates on KRs are permitted to members (the KR's own owner field).
- **R-ADMIN-11-W3:** The lead SHALL be able to create, assign, and close Tasks. Members MAY update task status for tasks they own.
- **R-ADMIN-11-W4:** The lead SHALL be able to add members to the project, draft-sending an `AgentMessage` invitation to the target Agent. Acceptance via the target's inbox.
- **R-ADMIN-11-W5:** The lead SHALL be able to create sub-projects, which opens [10-project-creation-wizard.md](10-project-creation-wizard.md) pre-filled with this project as parent and inheriting the shape and owning-org(s).
- **R-ADMIN-11-W6:** Validation: status transitions are constrained per [concepts/project.md § Project Status](../../concepts/project.md#project-status); KR `current_value` must match `measurement_type`.

## 7. Permission / Visibility Rules

- **Page access** — any Agent with membership in the project (via `HAS_AGENT`/`HAS_LEAD`/`HAS_SPONSOR`) OR Template A/C/D grant holders for this project's scope.
- **Edit basics (W1, W3 create, W5)** — lead only. Verified via `HAS_LEAD` on this project.
- **OKR edits (W2)** — lead for add/archive; KR owners may update `current_value` on their own KRs.
- **Member updates (W3 member-owned)** — the owning Agent only.
- **Non-lead viewers** — see the page read-only; CTAs for lead-only actions are hidden.

## 8. Event & Notification Requirements

- **R-ADMIN-11-N1:** Every status transition emits an audit event (`ProjectStatusChanged`, `ObjectiveStatusChanged`, `KeyResultStatusChanged`, `TaskStatusChanged`).
- **R-ADMIN-11-N2:** KR status transitions to `Achieved` / `Missed` push notifications to the KR owner's inbox AND the project lead's inbox.
- **R-ADMIN-11-N3:** When a task is completed, if its `linked_kr` is now satisfied, the page SHALL prompt the lead to mark the KR as `Achieved`.
- **R-ADMIN-11-N4:** When Shape B co-owner projects are displayed, events emit against both owning orgs' audit logs.

## 9. Backend Actions Triggered

W1 / W2 / W3 edits trigger graph node updates + audit events. Sub-project creation (W5) triggers a full [10-project-creation-wizard.md](10-project-creation-wizard.md) flow with the parent pre-set. Member invitation (W4) deposits an `AgentMessage` in the target's inbox — see [permissions/05 § Inbox and Outbox](../../concepts/permissions/05-memory-sessions.md#inbox-and-outbox-agent-messaging).

## 10. API Contract Sketch

```
GET  /api/v0/orgs/{org_id}/projects/{project_id}
     → 200: { project, objectives, key_results, tasks, sub_projects, roster, recent_sessions, resource_boundaries }

PATCH /api/v0/orgs/{org_id}/projects/{project_id}
     Body: { name?, description?, goal?, status?, objectives?, key_results? }
     → 200: { diff, audit_event_ids }

POST /api/v0/orgs/{org_id}/projects/{project_id}/tasks
     Body: { task }
     → 201: { task_id }

PATCH /api/v0/orgs/{org_id}/projects/{project_id}/tasks/{task_id}
     Body: { status?, assigned_to?, linked_kr? }
     → 200: { diff }
```

## 11. Acceptance Scenarios

**Scenario 1 — view flat-single-project mid-sprint.**
*Given* the Project corresponding to [projects/01-flat-single-project.md](../../projects/01-flat-single-project.md) is at week 2 of 6 with its 2 Objectives / 4 KRs populated, *When* the founder visits the detail page, *Then* the OKR panel shows current/target for each KR, the task panel lists the 5 in-progress tasks, and the active-sessions panel shows the 1 running session (intern-a's).

**Scenario 2 — KR update by owner.**
*Given* KR `kr-features-shipped` (current 2, target 3) has owner `founder`, *When* the founder updates current_value to 3 from the inline editor, *Then* the KR's status auto-suggests `Achieved`; confirming sets status → Achieved, emits `KeyResultStatusChanged`, and the linked Objective progress updates.

**Scenario 3 — non-lead cannot modify basics.**
*Given* an LLM Contract Agent `coder-a1` is a project member but not lead, *When* they open the page, *Then* the "Edit basics" button is absent; they can still update task status for `task-auth-cli` (their own) but not for tasks owned by other members.

## 12. Cross-References

**Concept files:**
- [concepts/project.md § Properties](../../concepts/project.md) — fields rendered here.
- [concepts/project.md § Objectives and Key Results](../../concepts/project.md#objectives-and-key-results-okrs) — OKR structure.
- [concepts/permissions/05 § Authority Templates](../../concepts/permissions/05-memory-sessions.md#authority-templates-formerly-the-authority-question) — Template A provides lead visibility.

**Related admin pages:**
- [10-project-creation-wizard.md](10-project-creation-wizard.md) (parent creation flow).

**Related agent-self-service pages:**
- [agent-self-service/a04-my-work.md](../agent-self-service/a04-my-work.md) — where assigned agents see their tasks.

**Project layouts exercised:**
- [projects/01-flat-single-project.md](../../projects/01-flat-single-project.md), [projects/02-deeply-nested-project.md](../../projects/02-deeply-nested-project.md) (for sub-project tree), [projects/05-compliance-audit-project.md](../../projects/05-compliance-audit-project.md) (for rich OKR layouts).
