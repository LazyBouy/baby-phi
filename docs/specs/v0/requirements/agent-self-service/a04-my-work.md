<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Requirements doc — agent self-service surface -->

# a04 — My Work

## 2. Page Purpose + Primary Actor

The Agent's surface for their own **tasks + sessions**. Two panels:

- **My tasks** — tasks assigned to this Agent via `ASSIGNED_TO` edges. Owner may update status on tasks they own.
- **My sessions** — currently-running + recently-finished sessions this Agent participated in. Useful for an Agent to see their own `parallelize` utilisation, track past work, and jump back into an active session.

**Primary actor:** any Agent. Most useful for Interns and Contract Agents with active assignments.

## 3. Available When

- Any Agent. Empty state shown if the Agent has no tasks or sessions.

## 4. UI Sketch

```
┌────────────────────────────────────────────────────────────────┐
│ My Work — agent:intern-a                                        │
├────────────────────────────────────────────────────────────────┤
│ Tasks (3)                                                       │
│ ┌──────────────────────────────────────────────────────────────┐│
│ │ task-auth-cli    stream-a    In Progress   [Update status]  ││
│ │   linked KR: kr-features-shipped                             ││
│ │ task-billing     stream-a    Not Started   [Update status]  ││
│ │ task-report      stream-a    Completed (auto)                ││
│ └──────────────────────────────────────────────────────────────┘│
│                                                                  │
│ Sessions                                                        │
│ ┌──────────────────────────────────────────────────────────────┐│
│ │ ● Running   s-9931   stream-a   started 10:42  [Open]       ││
│ │ ● Running   s-9932   stream-a   started 11:00  [Open]       ││
│ │ ○ Ended     s-9929   stream-a   duration 22m                ││
│ │                                                              ││
│ │ Concurrency: 2 / 2 (parallelize cap reached)                ││
│ └──────────────────────────────────────────────────────────────┘│
└────────────────────────────────────────────────────────────────┘
```

## 5. Read Requirements

- **R-AGENT-a04-R1:** The surface SHALL list tasks where the viewing Agent is the `ASSIGNED_TO` target, grouped by project and filtered by status. Each row: task id, project, status, linked KR (if any), deadline (if any), assigner.
- **R-AGENT-a04-R2:** The surface SHALL list sessions where the viewing Agent is the owning Agent (via `RUNS_SESSION` edge), sorted by state (Running first) then by start time descending.
- **R-AGENT-a04-R3:** The surface SHALL display the Agent's current concurrency usage: `active_sessions / parallelize` with a visible cap indicator.
- **R-AGENT-a04-R4:** Each active session row SHALL link to a live session view (steady-state page, out of scope here; shown as `[Open]`).

## 6. Write Requirements

- **R-AGENT-a04-W1:** The Agent SHALL be able to update status on tasks they own: `NotStarted → InProgress → Completed`. Transitions record a timestamp and may trigger KR-current-value updates (via the linked KR).
- **R-AGENT-a04-W2:** The Agent SHALL be able to mark a task **blocked** with a short reason; blocked status is visible to the project lead.
- **R-AGENT-a04-W3:** Validation: status transitions respect the TaskStatus flow from [concepts/project.md § Task Status Flow](../../concepts/project.md#task-status-flow).
- **R-AGENT-a04-W4:** The Agent SHALL NOT be able to reassign a task to another Agent from this surface (reassignment happens on [admin/11-project-detail.md](../admin/11-project-detail.md) by the lead).

## 7. Permission / Visibility Rules

- **Task visibility** — the Agent sees only tasks where they are the assignee; the project lead sees all tasks for the project (different page).
- **Task status update** — requires the viewing Agent to be the assignee.
- **Session visibility** — the Agent's own sessions. Auditors with `[read]` on session_object scoped to the Agent may see them on the audit log / session browser pages (out of scope here).

## 8. Event & Notification Requirements

- **R-AGENT-a04-N1:** Status updates emit `TaskStatusChanged { task_id, agent_id, old_status, new_status, timestamp }` audit events.
- **R-AGENT-a04-N2:** When a task transitions to `Completed`, the linked KR is auto-highlighted for the project lead to consider marking the KR `Achieved` (UI-level cue, not an automatic transition).
- **R-AGENT-a04-N3:** Transitioning a task to `Blocked` inserts a message in the project lead's inbox.

## 9. Backend Actions Triggered

- Task status updates: graph edge property change; audit event.
- No direct system-flow trigger beyond the audit event.

## 10. API Contract Sketch

```
GET  /api/v0/agents/{agent_id}/tasks?status=...
     → 200: { tasks: [...] }

GET  /api/v0/agents/{agent_id}/sessions?state=...
     → 200: { sessions: [...], concurrency: {active, parallelize} }

PATCH /api/v0/tasks/{task_id}
     Body: { status, reason? }
     → 200: { new_status, audit_event_id }
     → 403: Not assignee
     → 400: Invalid transition
```

## 11. Acceptance Scenarios

**Scenario 1 — intern updates a task to Completed.**
*Given* `intern-a` in [projects/01-flat-single-project.md](../../projects/01-flat-single-project.md) owns task `task-workflow`, *When* they transition it to `Completed`, *Then* `TaskStatusChanged` is logged, the linked KR `kr-features-shipped` gets a soft-prompt to its owner (founder), and the task shows as Completed on [admin/11-project-detail.md](../admin/11-project-detail.md).

**Scenario 2 — researcher sees parallelize utilisation.**
*Given* [organizations/05-research-lab-nested.md](../../organizations/05-research-lab-nested.md)'s `researcher-vision-1` runs at `parallelize: 4` and has 3 active sessions, *When* they open this page, *Then* the Sessions panel shows 3 active + N ended; concurrency indicator shows `3 / 4`.

**Scenario 3 — blocked task pushes to lead inbox.**
*Given* an intern's task hits a blocker they cannot resolve, *When* they mark the task `Blocked` with reason "awaiting API key rotation", *Then* a `TaskBlocked` event fires and the project lead receives an inbox message with the reason.

## 12. Cross-References

**Concept files:**
- [concepts/project.md § Task (Node Type)](../../concepts/project.md#task-node-type--optional-decomposition), especially [§ Task Status Flow](../../concepts/project.md#task-status-flow).
- [concepts/agent.md § Parallelized Sessions](../../concepts/agent.md#parallelized-sessions).
- [concepts/ontology.md](../../concepts/ontology.md) — `ASSIGNED_TO`, `RUNS_SESSION` edges.

**Related admin pages:**
- [admin/11-project-detail.md](../admin/11-project-detail.md) — the lead's view of all tasks and sessions.

**Project layouts exercised:**
- [projects/01-flat-single-project.md](../../projects/01-flat-single-project.md), [organizations/05-research-lab-nested.md](../../organizations/05-research-lab-nested.md).
