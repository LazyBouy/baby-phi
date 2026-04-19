<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Normative template — every admin/ and agent-self-service/ page follows this shape -->

# Admin / Agent-Self-Service Page Template

> The normative 10-section template. Every file in `admin/` and `agent-self-service/` conforms to this. If a section genuinely does not apply, write `N/A — see X` with a pointer to where the concept's rule lives instead of deleting the section.

---

## Section 1 — Header stub

Three HTML comments at the top of every file:

```
<!-- Status: CONCEPTUAL -->
<!-- Last verified: YYYY-MM-DD by Claude Code -->
<!-- Requirements doc — admin page, Phase N of fresh-install journey -->
```

The third line varies per folder:
- `admin/` → `Requirements doc — admin page, Phase N of fresh-install journey`
- `agent-self-service/` → `Requirements doc — agent self-service surface`

---

## Section 2 — Page Purpose + Primary Actor

One paragraph. State what the page does and who uses it. For primary/secondary actors, spell out the **Human Agent role** (platform admin / org admin / project lead) or **LLM Agent role** — remembering that "admin" is a role not an entity kind. Cite the specific grant that gates access.

**Example:**

> The Agent Roster List page lets a Human Agent with org-admin authority see all Agents in the current organization and manage their lifecycle.
>
> **Primary actor:** Human Agent holding `[allocate]` on the org's `agent-catalogue` control_plane_object (typically the org's CEO).
>
> **Secondary actors:** Project-lead Agents (Human or LLM) with read-only visibility filtered by Template A.

---

## Section 3 — Position in the Journey

For admin pages only. Three bullet lines:

- **Phase:** N of 9 (descriptor).
- **Depends on:** pages that must be completed before this one can function.
- **Enables:** pages or capabilities this one unblocks.

Agent-self-service pages may replace this with:

- **Available when:** the prerequisite grants the agent must hold.
- **Used by:** which agent kinds (Human / LLM / System).

---

## Section 4 — UI Sketch

ASCII mockup of the primary view. Include the main widgets, sample populated data from one of the 10 org layouts or 5 project layouts, and (below) brief descriptions of empty-state and error-state variants.

For agent-self-service pages where the actor is an LLM Agent and there is no human UI, replace the ASCII box with a **tool-invocation shape** — a code block showing the tool manifest the agent uses plus an example call/response pair. Label it clearly so readers know which rendering is shown.

---

## Section 5 — Read Requirements

What the page DISPLAYS. One requirement per distinct read. Format:

```
- **R-ADMIN-NN-R1:** The page SHALL display {field} derived from {source concept / graph node}.
- **R-ADMIN-NN-R2:** The page SHALL display …
```

Agent-self-service pages use `R-AGENT-aNN-R<n>` instead. System flow files use `R-SYS-sNN-<n>` with no R/W/N prefix (system flows don't have a UI).

Include any **filter, sort, search, pagination** controls as their own R-numbered requirements.

---

## Section 6 — Write Requirements

What the actor CAN CHANGE or INVOKE. Format mirrors reads but uses `W<n>`:

```
- **R-ADMIN-NN-W1:** The admin SHALL be able to {action}, which produces {backend effect}.
- **R-ADMIN-NN-W2:** Validation: the action SHALL be rejected if {condition} with error {code/message}.
```

Include **validation rules**, **error cases**, and **idempotency notes** where relevant.

Pages that are read-only (e.g., dashboards, audit viewers) should still have this section with `N/A — read-only page` as the content.

---

## Section 7 — Permission / Visibility Rules

Every gate on the page, traced back to the specific grant(s) that authorise. The section must make clear that permissions resolve to grants on the Agent — no ambient privilege. Format:

> All rules resolve to grants held by the **Human Agent / LLM Agent** viewing or acting on the page. No ambient privilege.
>
> - **Page access** — requires `{grant}` on `{resource}`. Held by `{role}` via `{default grant / template}`.
> - **Write action X** — requires `{grant}`. Held by `{role}` only.

Cite the concept file that defines the grant or template whenever possible.

---

## Section 8 — Event & Notification Requirements

What the actor is notified about while using the page (or after acting on it). Format mirrors read/write but uses `N<n>`:

```
- **R-ADMIN-NN-N1:** When {event}, the page SHALL display {toast/banner/indicator}.
- **R-ADMIN-NN-N2:** The page SHALL show a live-updating indicator of {state} sourced from {source}.
```

Include **audit events** this page causes to fire — cite the event name and key fields.

---

## Section 9 — Backend Actions Triggered

Reactive behaviours any of the page's writes provoke. Bullet format:

> Creating an Agent (W1) triggers:
> - Agent node created; `MEMBER_OF` edge added to the org.
> - Default Grants issued (per [permissions/05 § Default Grants](...)).
> - `inbox_object` and `outbox_object` composites atomically added to the org's `resources_catalogue`.
> - `agent-catalog-agent` updates its index (see [system/s03-edge-change-catalog-update.md](...)).
> - Audit event `AgentCreated { … }` emitted.

Cross-link to the relevant `system/` file for each headless behaviour. If the write does not trigger anything beyond the node write itself, say so explicitly.

---

## Section 10 — API Contract Sketch

REST-ish endpoint shapes. Not finalised — a sketch showing method, path, request body, response body, and error codes. Example:

```
GET  /api/v0/orgs/{org_id}/agents
     → 200: { agents: [...], summary: {...} }
     → 403: Permission denied

POST /api/v0/orgs/{org_id}/agents
     Body: { profile: AgentProfile, model_config: ModelConfig, execution_limits: ExecutionLimits, kind, parallelize }
     → 201: { agent_id, inbox_id, outbox_id }
     → 400: Validation errors
     → 403: Permission denied
     → 409: Name collision
```

This section serves double duty: (a) it is the implementer's starting point for the HTTP surface, and (b) it pins what the UI must be able to send/receive, which the test harness can generate fixtures against.

---

## Section 11 — Acceptance Scenarios

**2–4 concrete scenarios** grounded in the [10 org layouts](../../organizations/README.md) or [5 project layouts](../../projects/README.md). Format as Given/When/Then, with the layout name cited:

> **Scenario 1 — {layout name} {short description}.**
> *Given* {initial state referencing a specific layout}, *When* {actor does Y}, *Then* {expected observable outcome}.

At least one scenario per page must name a specific layout file. This grounds the requirements in the existing examples and gives the test harness concrete starting states to assert against.

Edge-case scenarios (permission denial, validation failure, race conditions) are welcome — they harden the acceptance surface.

---

## Section 12 — Cross-References

The last section. Must include:

**Concept files:**
- Every concept file whose rules this page implements, with the specific section anchor.

**phi-core types:**
- Every phi-core type the page's payload uses (AgentProfile, ModelConfig, ExecutionLimits, etc.), cross-referenced via [concepts/phi-core-mapping.md](../../concepts/phi-core-mapping.md).

**Related admin / agent-self-service pages:**
- Upstream and downstream pages in the journey.

**Related system flows:**
- Every `system/sNN-*` file whose reactive behaviour is provisioned by this page.

**Org / project layouts exercised in Acceptance Scenarios:**
- Direct links to the layout files cited in Section 11.

---

## How to use this template

1. Copy this file to `admin/NN-page-name.md` or `agent-self-service/aNN-page-name.md`.
2. Replace each section's template text with the page's actual content.
3. Give every requirement a unique ID matching the convention in [../README.md](../README.md#requirement-id-conventions).
4. Grep `grep -rE 'R-(ADMIN|AGENT|SYS|NFR)-' baby-phi/docs/specs/v0/requirements/` before committing to catch accidental ID collisions.
5. Populate the traceability matrix entry at [cross-cutting/traceability-matrix.md](../cross-cutting/traceability-matrix.md) with the concept sections your page covers.
