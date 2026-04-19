<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Requirements doc — admin page, Phase 9 of fresh-install journey -->

# 14 — First Session Launch

## 2. Page Purpose + Primary Actor

The "hello world" validation page. The org admin (or a project lead) picks an Agent and a Project, launches a first session, and verifies that the whole stack works end-to-end:

- The session starts (Permission Check passes at Step 0–6).
- The agent's inbox/outbox/memory are visible.
- On session end, `memory-extraction-agent` fires and deposits at least one extracted memory.
- `agent-catalog-agent` updates to reflect the new "last-seen" timestamp.
- The session lands in the audit feed with expected events.

This page is a **fresh-install journey validator**. Once Phase 9 passes, the admin confidently treats the stack as ready for ongoing operations.

**Primary actor:** org admin, or any Agent holding grants sufficient to launch a session on their chosen Agent+Project.

## 3. Position in the Journey

- **Phase:** 9 of 9 (Launch first session).
- **Depends on:** Phases 3–8 all complete.
- **Enables:** steady-state operations — once this validates, the journey is over and the admin moves to the steady-state pages (Auth Request queue, audit log viewer, etc., which are out of scope for this requirements plan).

## 4. UI Sketch

```
┌──────────────────────────────────────────────────────────────────┐
│ First Session Launch — Acme Corporation                          │
├──────────────────────────────────────────────────────────────────┤
│ Pick an Agent:    [ coder-acme-3 ▾ ]                             │
│ Pick a Project:   [ stream-a      ▾ ]                            │
│ Initial prompt:   [ multiline text                              ]│
│                                                                   │
│                          [Launch Session]                        │
│                                                                   │
│ Preview of Permission Check (will run on launch):                │
│   Step 0 Catalogue    ☑ all resources in org catalogue          │
│   Step 1 Manifest     ☑ tools needed: read_file, bash           │
│   Step 2 Grants       ☑ agent holds {read, write} on workspace  │
│   Step 3 Match        ☑ all reaches covered                      │
│   Step 4 Constraints  ☑ bash sandbox + timeout OK                │
│   Step 6 Consent      ☑ implicit policy (minimal-startup)        │
│                                                                   │
│ Launch will open the session in a new view → [open sessions/s-*]│
└──────────────────────────────────────────────────────────────────┘
```

Post-launch view:

```
┌──────────────────────────────────────────────────────────────────┐
│ Session s-9901 — intern-a @ stream-a           [Terminate]       │
├──────────────────────────────────────────────────────────────────┤
│ State: Running (turn 3 of 40)                                    │
│ Transcript [live streaming]                                      │
│   turn 1  [user]  Initial prompt...                              │
│   turn 2  [agent] Understood. Let me start by reading...         │
│   ...                                                             │
│                                                                   │
│ Events timeline                                                  │
│   10:42:01  SessionStarted { s-9901, intern-a, stream-a }        │
│   10:42:05  ToolInvoked { read_file } → ok                       │
│   ...                                                             │
└──────────────────────────────────────────────────────────────────┘
```

## 5. Read Requirements

- **R-ADMIN-14-R1:** The picker SHALL list active Agents from the org's roster (exclude Archived and System Agents from the default list; System Agents cannot be manually session-launched).
- **R-ADMIN-14-R2:** The project picker SHALL list active Projects the selected Agent is a member of OR has a lead grant for.
- **R-ADMIN-14-R3:** Before launch, the page SHALL render a **Permission Check preview** per [permissions/04 § Formal Algorithm (Pseudocode)](../../concepts/permissions/04-manifest-and-resolution.md#formal-algorithm-pseudocode) showing expected Step 0–6 outcomes. If any step would fail, that step shows an error and the Launch button is disabled.
- **R-ADMIN-14-R4:** The live-session view SHALL stream turns, tool invocations, and emitted audit events in near real time.

## 6. Write Requirements

- **R-ADMIN-14-W1:** The admin SHALL be able to launch a session with (agent, project, initial_prompt). Launch creates a Session node, a first Loop, and transitions the agent into `current_project = {project}` for this session's lifetime.
- **R-ADMIN-14-W2:** The launch SHALL respect the agent's `parallelize` cap — if already at the cap, launch is rejected with error `PARALLELIZE_CAP_REACHED`.
- **R-ADMIN-14-W3:** The admin SHALL be able to terminate a running session via the Terminate button. Termination reason is recorded; session transitions to `Aborted` with reason `user_terminated`.
- **R-ADMIN-14-W4:** Validation: initial_prompt is non-empty if required by the agent's profile; the selected agent must have `[read, write, execute]` grants sufficient for its declared tool manifests (the preview verifies this).

## 7. Permission / Visibility Rules

- **Page access** — any Agent with `[read, list]` on the org's session registry plus the grants needed to launch the chosen Agent on the chosen Project. Typically the admin; may also be a project lead.
- **Launch (W1)** — requires the acting agent to hold `[execute]` on the chosen Agent's session-start control plane OR be the agent themselves acting on their own behalf.
- **Terminate (W3)** — requires either the acting agent to be the session's owner agent (self-terminate) or to hold `[allocate]` on the session_object (admin override).

## 8. Event & Notification Requirements

- **R-ADMIN-14-N1:** `SessionStarted { session_id, agent_id, project_id, started_by }` audit event emitted on launch.
- **R-ADMIN-14-N2:** On session end (normal), `SessionEnded { session_id, duration, turn_count, token_cost }` is emitted; the `memory-extraction-agent` subscribes to this event (see [system/s02-session-end-memory-extraction.md](../system/s02-session-end-memory-extraction.md)).
- **R-ADMIN-14-N3:** On termination (W3), `SessionAborted { session_id, reason, terminated_by }` emitted.
- **R-ADMIN-14-N4:** The page SHALL confirm (via a post-launch checklist) that:
  - Session appears in session browser.
  - At least one audit event flowed to the audit log.
  - On session end, ≥1 memory was extracted by `memory-extraction-agent`.
  - `agent-catalog-agent` updated the agent's last-seen timestamp.

## 9. Backend Actions Triggered

Launch (W1):
- Session node created with `#kind:session` auto-tagged and the agent's tags (`project:{proj}`, `org:{owning_org}`, `agent:{agent_id}`).
- First `Loop` record created.
- Permission Check runs on each tool invocation during session execution.
- `SessionStarted` audit event.

Session end (automatic or via W3):
- `SessionEnded` / `SessionAborted` event.
- `memory-extraction-agent` subscribes to `SessionEnded` — fires its extraction loop (see [system/s02-session-end-memory-extraction.md](../system/s02-session-end-memory-extraction.md)).
- `agent-catalog-agent` updates last-seen.

## 10. API Contract Sketch

```
POST /api/v0/orgs/{org_id}/projects/{project_id}/sessions
     Body: { agent_id, initial_prompt }
     → 201: { session_id, loop_id, permission_check_trace: {...} }
     → 400: Validation
     → 403: Permission Check failed at step N
     → 409: PARALLELIZE_CAP_REACHED

GET  /api/v0/sessions/{session_id}
     → 200: { session, current_loop, turns_so_far, live: true }

POST /api/v0/sessions/{session_id}/terminate
     Body: { reason }
     → 200: { terminated_at, audit_event_id }
```

## 11. Acceptance Scenarios

**Scenario 1 — hello world for minimal-startup.**
*Given* [organizations/01-minimal-startup.md](../../organizations/01-minimal-startup.md) and [projects/01-flat-single-project.md](../../projects/01-flat-single-project.md) are ready, *When* the founder launches a session of `intern-a` on the `solo-project` with a simple prompt "read the README and summarise", *Then* the session starts with Permission Check green across Steps 0–6, the agent reads a file and produces a turn, the session ends, `memory-extraction-agent` deposits a memory tagged `project:solo-project`, `agent-catalog-agent` updates last-seen, and all events appear in the audit log.

**Scenario 2 — parallelize-cap-reached.**
*Given* `intern-a` has `parallelize: 2` and is already running 2 sessions, *When* the admin tries to launch a third, *Then* the launch is rejected with `PARALLELIZE_CAP_REACHED`; a hint suggests terminating one of the running sessions (linking to the live-session view).

**Scenario 3 — Permission Check preview blocks launch.**
*Given* the admin selects an Agent whose grants lack network access but the tool manifest requires `network_endpoint`, *When* the page previews the Permission Check, *Then* Step 3 (Match) shows a failure with detail "no grant covers (network_endpoint, read)"; Launch button is disabled; the admin must either pick a different agent or issue a grant first.

## 12. Cross-References

**Concept files:**
- [concepts/agent.md § Experience (Sessions + Memory)](../../concepts/agent.md#experience-sessions--memory).
- [concepts/permissions/04 § Formal Algorithm (Pseudocode)](../../concepts/permissions/04-manifest-and-resolution.md#formal-algorithm-pseudocode) — the 6-step Permission Check the preview runs.
- [concepts/permissions/05 § Sessions as a Tagged Resource](../../concepts/permissions/05-memory-sessions.md#sessions-as-a-tagged-resource).

**phi-core types:** `Session`, `LoopRecord`, `Turn`, `Message`, `AgentEvent` — see [concepts/phi-core-mapping.md](../../concepts/phi-core-mapping.md).

**Related admin pages:**
- [11-project-detail.md](11-project-detail.md) — the project home shows the launched session in its "active sessions" panel.

**Related system flows:**
- [system/s02-session-end-memory-extraction.md](../system/s02-session-end-memory-extraction.md).
- [system/s03-edge-change-catalog-update.md](../system/s03-edge-change-catalog-update.md) — last-seen updates.

**Project layouts exercised:**
- [projects/01-flat-single-project.md](../../projects/01-flat-single-project.md) (Scenarios 1–3 baseline).
