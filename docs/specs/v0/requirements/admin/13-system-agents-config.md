<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Requirements doc — admin page, Phase 8 of fresh-install journey -->

# 13 — System Agents Configuration

## 2. Page Purpose + Primary Actor

The page where the org admin reviews and tunes the **System Agents** running for the org. The two standard System Agents — `memory-extraction-agent` and `agent-catalog-agent` — are created automatically at org adoption time; this page lets the admin verify they are running, adjust `parallelize`, swap profile references, add new **org-specific System Agents** (e.g., `compliance-audit-agent`, `grading-agent`), or disable one.

**Primary actor:** Human Agent with org-admin authority.

## 3. Position in the Journey

- **Phase:** 8 of 9 (Configure system agents).
- **Depends on:** [06-org-creation-wizard.md](06-org-creation-wizard.md) (which creates the two standard System Agents).
- **Enables:** Phase 9 (first session produces events the system agents react to).

## 4. UI Sketch

```
┌──────────────────────────────────────────────────────────────────┐
│ System Agents — Acme Corporation                  [+ Add Agent] │
├──────────────────────────────────────────────────────────────────┤
│ Standard                                                          │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ memory-extraction-agent                                     │ │
│ │   status: ● running    parallelize: [ 2 ]   trigger: sess.end│ │
│ │   profile_ref: system-memory-extraction  [View profile]     │ │
│ │   last fired: 2m ago    queue depth: 0                      │ │
│ │   [Tune]  [Disable (uncommon)]                              │ │
│ ├─────────────────────────────────────────────────────────────┤ │
│ │ agent-catalog-agent                                          │ │
│ │   status: ● running    parallelize: [ 1 ]   trigger: edge   │ │
│ │   profile_ref: system-agent-catalog       [View profile]    │ │
│ │   last fired: 10s ago   queue depth: 0                      │ │
│ │   [Tune]                                                     │ │
│ └─────────────────────────────────────────────────────────────┘ │
│                                                                    │
│ Org-specific                                                      │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │  (none — [+ Add Agent] to provision)                        │ │
│ └─────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────┘
```

## 5. Read Requirements

- **R-ADMIN-13-R1:** The page SHALL list all System Agents in the org's roster, grouped **Standard** (the two defaults) vs **Org-specific** (added extras).
- **R-ADMIN-13-R2:** Each entry SHALL show status (running / degraded / disabled), `parallelize` value, trigger type, profile_ref, last-fired timestamp, and current queue depth (for async-processed triggers).
- **R-ADMIN-13-R3:** The page SHALL show a timeline of the last 20 fire events across all system agents — helpful for confirming the system is reactive after Phase 9.
- **R-ADMIN-13-R4:** Each row SHALL show the Grants the System Agent holds, in a drawer or modal.

## 6. Write Requirements

- **R-ADMIN-13-W1 (tune existing):** The admin SHALL be able to adjust `parallelize` on an existing System Agent inline (subject to org cap and the agent's profile declaration).
- **R-ADMIN-13-W2 (add org-specific):** The admin SHALL be able to add a new System Agent via a form supplying: id, profile_ref (must exist in the platform's system-agent-profile registry), parallelize, trigger (from {session_end, edge_change, periodic, explicit, custom_event}), and any org-specific behaviour fields. On save, the Agent is created with `kind: System` and the grants defined in its profile-ref template.
- **R-ADMIN-13-W3 (disable standard):** The admin SHALL be able to disable either of the two standard System Agents (uncommon; shown with a strong-warning dialog because disabling `memory-extraction-agent` means session memories are no longer auto-extracted; disabling `agent-catalog-agent` means the roster may become stale).
- **R-ADMIN-13-W4 (archive org-specific):** The admin SHALL be able to archive an org-specific System Agent; no cascade to in-flight triggers.

## 7. Permission / Visibility Rules

- **Page access** — `[read, list]` on `control_plane_object:agent-catalogue` scoped to System Agents. Org admin and any Agent with a broader `[read, list]` on the catalogue (project leads usually don't).
- **Write actions (W1–W4)** — `[allocate]` on `control_plane_object:agent-catalogue`. Org admin only.

## 8. Event & Notification Requirements

- **R-ADMIN-13-N1:** On any write, emit audit event. Standard System Agent changes are alerted (`MemoryExtractionAgentReconfigured`, `AgentCatalogAgentReconfigured`); org-specific adds are logged (`SystemAgentAdded`), and disables are alerted (`StandardSystemAgentDisabled`).
- **R-ADMIN-13-N2:** When a disable is attempted on `memory-extraction-agent`, the dialog SHALL warn explicitly: "Session memories will not be auto-extracted. Past extractions remain. Re-enable at any time."
- **R-ADMIN-13-N3:** Live status indicators SHALL update on a poll interval (≤5s).

## 9. Backend Actions Triggered

- W1 (tune) — patches the System Agent's `AgentProfile.parallelize`; currently-running triggers finish at the old value; new triggers pick up the new value.
- W2 (add) — creates a new Agent node with `kind: System`, issues the grants per the profile-ref template, starts the trigger subscriber, emits audit event.
- W3 (disable) — pauses the agent's trigger subscriber. The Agent node remains but is marked `active: false`.
- W4 (archive) — graph archival, trigger subscriber removed.

All changes flow through the same `[allocate]` + Template-E-style auth-request flow as other agent-catalogue changes.

## 10. API Contract Sketch

```
GET  /api/v0/orgs/{org_id}/system-agents
     → 200: { standard: [...], org_specific: [...], recent_events: [...] }

PATCH /api/v0/orgs/{org_id}/system-agents/{agent_id}
     Body: { parallelize?, profile_ref? }
     → 200: { updated_at, audit_event_id }

POST  /api/v0/orgs/{org_id}/system-agents
     Body: { id, profile_ref, parallelize, trigger, behaviour_fields? }
     → 201: { agent_id, grants_issued: [...] }

POST  /api/v0/orgs/{org_id}/system-agents/{agent_id}/disable
     Body: { confirm: true }
     → 200: { disabled_at, audit_event_id }
```

## 11. Acceptance Scenarios

**Scenario 1 — raise memory-extraction parallelize for a busy research lab.**
*Given* [organizations/05-research-lab-nested.md](../../organizations/05-research-lab-nested.md) has `memory-extraction-agent` at the default `parallelize: 2`, *When* the lab director raises it to 4 via W1, *Then* the agent's subscriber dispatches up to 4 concurrent session-end extractions, the audit event records the diff, and the timeline shows the next fire running with 4-wide concurrency.

**Scenario 2 — add compliance-audit-agent.**
*Given* [organizations/04-regulated-enterprise.md](../../organizations/04-regulated-enterprise.md) wants the extra compliance agent from the layout spec, *When* the org admin submits a new System Agent with profile_ref `compliance-audit-meridian`, parallelize 4, trigger `session_end`, *Then* a new Agent is created, grants issued, and subsequent session-end events fan out to both memory-extraction and compliance-audit.

**Scenario 3 — disable standard carries strong warning.**
*Given* an admin attempts to disable `memory-extraction-agent`, *When* the dialog appears, *Then* the text explicitly warns about the loss of automatic memory extraction; confirmation requires typing the agent's id into a disable-confirm field.

## 12. Cross-References

**Concept files:**
- [concepts/system-agents.md](../../concepts/system-agents.md) — the authoritative definitions of the two standard System Agents + the stubs for future ones.
- [concepts/agent.md § System Agent](../../concepts/agent.md#system-agent) — the System-Agent kind.
- [concepts/agent.md § Parallelized Sessions](../../concepts/agent.md#parallelized-sessions) — the field tuned in W1.

**Related admin pages:**
- [08-agent-roster-list.md](08-agent-roster-list.md) — System Agents appear here too but are read-only; edits happen on this page.
- [09-agent-profile-editor.md](09-agent-profile-editor.md) — Standard Agents' detail editor; System Agents are read-only there.

**Related system flows:**
- [system/s02-session-end-memory-extraction.md](../system/s02-session-end-memory-extraction.md).
- [system/s03-edge-change-catalog-update.md](../system/s03-edge-change-catalog-update.md).
- [system/s06-periodic-triggers.md](../system/s06-periodic-triggers.md).

**Org layouts exercised in Acceptance Scenarios:**
- [organizations/04-regulated-enterprise.md](../../organizations/04-regulated-enterprise.md), [organizations/05-research-lab-nested.md](../../organizations/05-research-lab-nested.md).
