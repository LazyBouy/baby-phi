# Plan: 10 Org + 5 Project Layouts, plus Inbox/Outbox, Parallel Sessions, and System Agents

> **Legend:**
> - `[PLAN: new]` — part of this fresh plan
> - `[DOCS: ✅ done]` / `[DOCS: ⏳ pending]` / `[DOCS: n/a]`

## Context  `[PLAN: new]` `[DOCS: n/a]`

Two interleaved workstreams:

1. **Concept catalogue:** 10 organization layouts and 5 project layouts, each a full YAML config + narrative, covering a diverse design space. Folders: `v0/organizations/`, `v0/projects/`.
2. **Spec extensions needed to support the catalogue:**
   - **Resource catalogue rule** — every resource (primary fundamental **and** composite, including composites that are constructed by an org's own operations) must be declared in the owning Organization's catalogue before any project or agent can reference it.
   - **OKRs on Project** — minimal `objectives` + `key_results` fields.
   - **Inbox / Outbox composites** — two new composite resource classes that capture agent-to-agent messaging, distinct from task queues. Each agent has its own inbox and outbox; the agent is free to decide how to react to messages.
   - **Parallelized sessions** — an agent (same AgentProfile) can run multiple concurrent sessions on different projects, bounded by a configurable `parallelize` parameter. Sessions are always independent; memories may pool into the agent's shared memory scope with `derived_from` cross-references.
   - **System Agents (standard two):** Memory Extraction Agent (routes memories from sessions to the correct pool based on tag permissions) and Agent Catalog Agent (tracks all active agents per org, queryable, manages lifecycle). Formalised as standard system agents included in the Standard Organization Template; future system agents can be added without changing the model.

The examples leverage phi-core's existing types (`AgentProfile`, `ModelConfig`, `ExecutionLimits`, `ToolDefinition`, `CompactionConfig`, etc.) so that YAML configs feel like they could be loaded by phi-core's config system.

## History — where prior work is recorded  `[PLAN: new]` `[DOCS: n/a]`

- Phase A–D archive: `baby-phi/docs/specs/plan/d95fac8f-ownership-auth-request.md`
- Phase F archive: `baby-phi/docs/specs/plan/54b1b2cb-split-and-gap-closure.md`
- Phase G/H archive: `baby-phi/docs/specs/plan/b30cb86b-push-to-95.md`
- Current spec state: `baby-phi/docs/specs/v0/concepts/`

## Decisions Captured  `[PLAN: new]` `[DOCS: see Impl column]`

| Topic | Decision | Impl Status |
|-------|----------|-------------|
| **Catalogue folders** | `v0/organizations/` and `v0/projects/`, each with its own `README.md` index. | ⏳ |
| **Depth per example** | **Narrative + full YAML config.** Each file: profile → knobs-summary table → narrative → full YAML → cross-references. | ⏳ |
| **Leverage phi-core** | Agent roster entries use phi-core's `AgentProfile` + `ModelConfig` + `ExecutionLimits` fields. Tool references use `ToolDefinition` shape. Cite phi-core-mapping.md. | ⏳ |
| **Resource catalogue rule** | Spec change. Covers **both primary fundamentals and composites** — including composites constructed at runtime by the org (e.g., a newly-registered `external_service_object` for a specific MCP server). Catalogue is the single source of truth for what exists in the org's scope; grant resolution includes a Step 0 catalogue precondition. | ⏳ (O1) |
| **OKR schema** | Minimal addition to Project: `objectives: Vec<Objective>` + `key_results: Vec<KeyResult>`, with name / description / status / measurement fields. Value objects, not nodes. | ⏳ (O2) |
| **Inbox / Outbox** | Two new composites: `inbox_object` (messages received by this agent) and `outbox_object` (messages this agent has sent). Each is `data_object + tag` + `#kind:inbox` / `#kind:outbox`. One inbox and one outbox per Agent. Messages are embedded value objects (`AgentMessage`) on inbox/outbox nodes — not independent nodes, to match phi-core's existing `Message` node (which is LLM-turn content, different concept). Separate from the agent's task queue. | ⏳ (O3) |
| **Parallelized sessions** | Add `parallelize: u32` (default `1`) field to AgentProfile in agent.md. Sessions are always independent executions; concurrent sessions by the same profile may share the agent's `HAS_MEMORY` pool via LWW consistency from coordination.md; cross-references use `derived_from:session:{id}` tags. An agent running at `parallelize: 4` can have up to 4 concurrent sessions at any moment. | ⏳ (O4) |
| **System Agents** | Formalise **two standard System Agents** now: `memory-extraction-agent` and `agent-catalog-agent`. Included in Standard Organization Template's `system_agents:` section. Both have defined responsibilities, tool manifests, and grants. A new file `v0/concepts/system-agents.md` catalogues these and leaves room for future additions (Introspection Agent, Monitoring Agent, etc.). | ⏳ (O5) |
| **Sponsorship hierarchy** | Reuse existing `HAS_SPONSOR` edge (Project → Agent). Task hierarchy uses `HAS_LEAD` + `ASSIGNED_TO`. Both coexist per project. No new edges needed. | ⏳ |
| **Composite resource creation at org level** | When an org registers a new composite instance (e.g., a new MCP server), the registration emits a schema-extension-style Auth Request (Template E) that adds the instance to the org's `resources_catalogue`. This makes catalogue extensions auditable by construction. | ⏳ (O1) |

## Spec Additions (O1–O5)

### O0: Archive this plan  `[PLAN: new]` `[DOCS: ⏳]`

Copy verbatim to `baby-phi/docs/specs/plan/<random>-org-project-layouts.md` (8-hex-char token). First action after approval.

### O1: Resource Catalogue rule (primary + composite)  `[PLAN: new]` `[DOCS: ⏳]`

**Files:**
- `permissions/01-resource-ontology.md`
- `permissions/04-manifest-and-resolution.md`
- `permissions/07-templates-and-tools.md`
- `organization.md`

**Changes:**

1. **`permissions/01-resource-ontology.md`** — new subsection §Resource Catalogue after §Resource Ownership:

   > **Resource Catalogue.** Every Organization maintains a `resources_catalogue` that enumerates the resource instances under its ownership or sub-allocation. A resource — **both primary fundamentals and composites** — can only be referenced by a project, agent, tool manifest, or grant if it is declared in the owning Organization's catalogue. The catalogue covers:
   >
   > - All 9 fundamentals (filesystem, process_exec, network_endpoint, secret/credential, data_object, tag, identity_principal, economic_resource, time/compute_resource).
   > - All 6 composites (memory_object, session_object, external_service_object, model/runtime_object, control_plane_object, auth_request_object) — **plus inbox_object and outbox_object** once O3 lands.
   > - Composite instances that are **constructed by the org's own operations** (e.g., registering a new MCP server creates a new `external_service_object` instance; the registration operation adds it to the catalogue atomically).
   >
   > Catalogue entries are added at org setup (by the platform admin) or later via a schema-extension-style Auth Request (Template E). Adding to the catalogue is itself an auditable event. The catalogue is the **structural pre-condition** for grant resolution, evaluated as Step 0 before the Permission Check (see [04 § Formal Algorithm](04-manifest-and-resolution.md#formal-algorithm-pseudocode)).

2. **`permissions/04-manifest-and-resolution.md` §Formal Algorithm (Pseudocode)** — prepend Step 0:

   ```python
   # --- Step 0: Catalogue precondition ---
   for resource_ref in manifest.resources_reached(call):
       owner = owning_org(resource_ref)
       if not owner.resources_catalogue.contains(resource_ref):
           return Denied(reason="resource not in org catalogue",
                         failed_step=0, detail=resource_ref)
   ```

3. **`permissions/07-templates-and-tools.md` — Standard Organization Template** — add `resources_catalogue:` and `system_agents:` sections to the YAML. Full shape shown:

   ```yaml
   resources_catalogue:
     # === Fundamentals ===
     filesystem_objects:
       - path: /workspace/{project}/**
         default_owner: project
       - path: /home/{agent}/**
         default_owner: agent
     network_endpoints: [...]
     secrets: [...]
     data_objects: [...]
     tags: [...]
     identity_principals: [...]
     economic_resources:
       - id: token-budget-pool
         units: tokens
     compute_resources: [...]
     process_exec_objects: [...]
     # === Composites ===
     memory_objects:
       - scope: per-agent
       - scope: per-project
       - scope: per-org
       - scope: '#public'
     session_objects:
       - scope: per-project
       - scope: per-agent
     external_services: [...]         # MCP servers, OpenAPI endpoints; additions auditable
     model_runtime_objects: [...]     # LLM provider configs
     control_plane_objects: [...]     # tool registry, policy store
     auth_request_objects: [...]      # implicit — catalogue auto-contains all
     inbox_objects: [...]             # auto-included for each agent in roster
     outbox_objects: [...]            # same
   ```

4. **`organization.md` §Organization (Node Type)** — add `resources_catalogue: ResourceCatalogue` property with cross-link to permissions/07.

### O2: OKR fields on Project  `[PLAN: new]` `[DOCS: ⏳]`

**File:** `baby-phi/docs/specs/v0/concepts/project.md`, with ontology.md echo.

1. Add `objectives: Vec<Objective>` and `key_results: Vec<KeyResult>` rows to Project Properties table (both optional).
2. Add §Objectives and Key Results (OKRs) subsection with value-object definitions:

   ```
   Objective {
     objective_id: String
     name: String
     description: String
     status: enum { Draft, Active, Achieved, Missed, Cancelled }
     owner: agent_id
     deadline: Option<DateTime>
     key_result_ids: Vec<String>
   }
   KeyResult {
     kr_id: String
     name: String
     description: String
     measurement_type: enum { Count, Boolean, Percentage, Custom }
     target_value: Value
     current_value: Value
     owner: agent_id
     deadline: Option<DateTime>
     status: enum { NotStarted, InProgress, Achieved, Missed, Cancelled }
   }
   ```

3. `ontology.md` Value Objects — add Objective and KeyResult.

### O3: Inbox / Outbox composites + AgentMessage  `[PLAN: new]` `[DOCS: ⏳]`

**Files:**
- `permissions/01-resource-ontology.md` (composites table)
- `permissions/05-memory-sessions.md` (new section: inbox/outbox + tag vocabulary)
- `ontology.md` (nodes + edges + value objects)
- `agent.md` (messaging as distinct from task queue)

**Changes:**

1. **`permissions/01-resource-ontology.md` Composites table** — add two rows:

   | Class | Explicit fundamentals | Implicit | Notes |
   |-------|-----------------------|----------|-------|
   | `inbox_object` | `data_object + tag` | `#kind:inbox` + one per Agent | Messages received by this agent. One per agent; read by the owner, written by senders via a `send_message` tool. Separate from task queue. |
   | `outbox_object` | `data_object + tag` | `#kind:outbox` + one per Agent | Messages this agent has sent. Append-only from the agent's perspective; read by owner + authorised auditors. |

2. **`permissions/05-memory-sessions.md`** — add §Inbox and Outbox (Agent Messaging) section parallel to memory/session sections:
   - Resource class: `inbox_object`, `outbox_object`
   - Tag vocabulary: `agent:{owner}`, `sender:{agent_id}`, `received_at:{timestamp}`, `#urgent`, `#read` / `#unread`, `thread:{conversation_id}`
   - Standard actions: `read`, `list`, `recall` (on inbox); `send`, `list` (on outbox); `delete` (owner-only, with audit)
   - Default grants: the agent holds `[read, list, recall, delete]` on its own inbox and `[read, list, send]` on its own outbox.
   - Messages as value objects: `AgentMessage { message_id, sender, recipient, subject, body, sent_at, thread_id, priority }`.
   - Rule: agents are **independent on how they react**. Receiving a message does not auto-trigger any behaviour; the agent's next session may or may not inspect the inbox. Messaging is information, not control.
   - Cross-reference: separate from task assignment (ASSIGNED_TO edges) and from session spawning (DELEGATES_TO / SPAWNED_FROM). Messaging is peer-to-peer information flow.

3. **`ontology.md`** — add:
   - Nodes: `InboxObject`, `OutboxObject` (identity = `agent_id`).
   - Edges: `Agent ──HAS_INBOX──▶ InboxObject` (1:1), `Agent ──HAS_OUTBOX──▶ OutboxObject` (1:1), `InboxObject ──CONTAINS_MESSAGE──▶ AgentMessage` (1:N as value object embedding), same for Outbox.
   - Value Object: `AgentMessage` added to catalogue.
   - Increment node counts (29 → 31).

4. **`agent.md` §Grounding Principle or new §Inter-Agent Messaging** — paragraph:
   > Every Agent has exactly one **Inbox** (messages received) and one **Outbox** (messages sent). These are separate from the agent's task queue (ASSIGNED_TO tasks) and from delegation edges (DELEGATES_TO for sub-agents). Messaging is **pure information flow**: the sender deposits a message, the recipient's next session may inspect the inbox, but there is no automatic reaction — the agent decides whether, when, and how to respond. This keeps agents autonomous in behaviour while making peer-to-peer signalling first-class.

### O4: Parallelized sessions  `[PLAN: new]` `[DOCS: ⏳]`

**File:** `baby-phi/docs/specs/v0/concepts/agent.md`, plus cross-references.

**Change:** Add a new subsection §Parallelized Sessions under the agent-anatomy content:

> **Parallelized Sessions.** An agent profile carries a `parallelize: u32` field (default `1`) that bounds how many concurrent sessions a single agent instance may run. An agent with `parallelize: 4` can execute up to four independent sessions at any moment — typically on different projects or different tasks within the same project.
>
> **Semantics:**
> - Sessions are always **independent executions**. Concurrent sessions do not share short-term memory or the loop's in-flight context; each runs its own `agent_loop()`.
> - The agent's `HAS_MEMORY` pool is shared across concurrent sessions. Memory writes from concurrent sessions follow the [coordination.md LWW consistency rule](coordination.md#design-decisions-v0-defaults-revisitable); cross-references use `derived_from:session:{id}` tags so that memories can cite which session produced them.
> - The `parallelize` value is configurable per-profile at org adoption time and may be tightened by a project (a project can cap an agent's concurrent participation within its scope).
> - `parallelize: 1` is the traditional single-session agent and is the default.

Cross-reference additions:
- `ontology.md`: note on `AgentProfile` schema (Profile now carries `parallelize`).
- `permissions/05-memory-sessions.md`: in the Memory tag vocabulary, note that multiple concurrent sessions write to the shared agent-scope memory under LWW.

### O5: System Agents (catalogue file)  `[PLAN: new]` `[DOCS: ⏳]`

**Files:**
- NEW `baby-phi/docs/specs/v0/concepts/system-agents.md`
- `permissions/07-templates-and-tools.md` (system_agents in Standard Org Template)
- `agent.md` (cross-reference from §System Agent)

**Content of `system-agents.md`:**

1. Header stub (Status, Last verified, cross-links).
2. §Purpose — one paragraph: system agents are infrastructure. Every Organization gets at least the two below; custom system agents can be added.
3. §Memory Extraction Agent:
   - Profile: Soul (AgentProfile), Model (platform-default), ExecutionLimits (tight — per-extraction budget).
   - Trigger: fires on session-end events (hook into AgentEvent stream).
   - Behaviour: reads the completed session, identifies candidate memories, allocates each to the correct pool based on content + tag permissions. Pools: `agent:{owner}` (private), `project:{id}` (public-in-project), `org:{id}` (public-in-org), `#public` (everywhere).
   - Grants: `[read]` on `session_object` for sessions it has authority to process; `[store]` on `memory_object` scoped to the allocated pool.
   - Tool manifest: internal `extract_memory` tool (not in the 14-tool catalogue — system-only).
   - Tagging rules: uses Supervisor Extraction as Two Standard Grants pattern from `permissions/05`.
4. §Agent Catalog Agent:
   - Profile: read-only introspection agent.
   - Behaviour: maintains an up-to-date catalogue of all active Agents per org, including Agent Profiles, current `parallelize` state, assigned projects, recent activity summary. Exposes a `query_agents(org_id, filters)` API.
   - Lifecycle hooks: observes `MEMBER_OF`, `DELEGATES_TO`, `HAS_AGENT` edge changes; updates its catalogue reactively.
   - Grants: `[read, list, inspect]` on all `identity_principal` and agent-related edges within the org; `[store]` on a single `control_plane_object` instance that holds the catalogue.
   - Queryable interface — other agents (with appropriate grants) can ask it for agent listings.
5. §Other System Agents (Future) — bullet list of candidates already named in agent.md's System Agent section: Introspection Agent, Project Setup Agent, Monitoring Agent, Skill Curation Agent. These are left as `[OUT OF V0 SCOPE]` stubs.
6. §How System Agents Fit the Standard Organization Template — the two v0 system agents are instantiated at org adoption time by the Standard Organization Template. Each org's agent roster includes them by default; orgs may disable or customise them in their own template (unusual but allowed).

**Change in `permissions/07`:** add a `system_agents:` section to the Standard Organization Template YAML:

```yaml
system_agents:
  - id: memory-extraction-agent
    profile_ref: system-memory-extraction
    parallelize: 2                   # can process up to 2 session-ends at once
    trigger: session_end
  - id: agent-catalog-agent
    profile_ref: system-agent-catalog
    parallelize: 1
    trigger: edge_change
```

**Change in `agent.md`:** update the §System Agent examples list to forward-link to `system-agents.md` for the two formalised agents.

## Catalogue (O6–O10)

### O6: Create `v0/organizations/` + README  `[PLAN: new]` `[DOCS: ⏳]`

README contains:
- Framing paragraph (examples are illustrative, not normative).
- Index table (10 rows).
- Knobs matrix (consent policy, agent mix, hierarchy depth, audit posture, market participation, co-ownership, inbox/outbox use, parallelize settings, system agents included).
- Recommended reading order.

### O7: Write 10 organization layout files  `[PLAN: new]` `[DOCS: ⏳]`

Each file `<NN>-<name>.md`: header stub, profile, knobs table, narrative, full YAML (with phi-core types), cross-refs.

| # | Name | Distinguishing knobs |
|---|------|-----------------------|
| 01 | `minimal-startup` | Implicit consent, 1 sponsor + 2 interns, `parallelize: 1`, both system agents included at minimum config. Simplest possible org. |
| 02 | `mid-product-team` | Flat two-level hierarchy, 3 leads + ~10 workers, one_time consent, `parallelize: 2` on interns. |
| 03 | `consultancy-strict` | per_session consent, Template D only (A/B disabled), bash entirely removed, tight `ExecutionLimits`, `parallelize: 1`. |
| 04 | `regulated-enterprise` | `audit_class: alerted` default, per_session consent, heavy `#sensitive` catalogue, Template C enabled, compliance auditor system agents (beyond the default two). |
| 05 | `research-lab-nested` | Deep nested `HAS_SUBORGANIZATION` (lab → group → team), long-duration projects, `parallelize: 4` on worker profiles, memory-extraction-agent is heavily used. |
| 06 | `joint-venture-acme` | Co-owns `joint-research` project with Beta. Implicit consent on own projects. |
| 07 | `joint-venture-beta` | Co-owns `joint-research` with #06. one_time consent. Exercises per-co-owner consent rule. |
| 08 | `marketplace-gig` | All contract agents, Template E-heavy, market-bid-ready. Heavy use of inbox/outbox for bid negotiation. |
| 09 | `education-org` | System-agent-heavy (grading, review). Learners are interns with `parallelize: 1`. Instructors as sponsors. per_session consent. |
| 10 | `platform-infra` | Cross-cutting custodian of MCP servers + credentials + token economy. Resource catalogue includes shared composites other orgs reference. |

### O8: Create `v0/projects/` + README  `[PLAN: new]` `[DOCS: ⏳]`

Same shape as O6, scoped to the 5 project layouts.

### O9: Write 5 project layout files  `[PLAN: new]` `[DOCS: ⏳]`

| # | Name | Distinguishing knobs |
|---|------|-----------------------|
| 01 | `flat-single-project` | Shape A. 1 lead + 3 workers. Sprint-length OKR (2 obj / 4 KR). No sub-projects. |
| 02 | `deeply-nested-project` | 2 levels of `HAS_SUBPROJECT`. Aggregated OKRs. Each level has its own lead and tasks. |
| 03 | `joint-research` | Shape B, co-owned by `joint-venture-acme` + `joint-venture-beta`. Mixed-org agent roster. Exercises per-co-owner consent. |
| 04 | `market-bid-project` | Tasks posted for bidding by contract agents from `marketplace-gig`. Template E on each assignment. Bidding coordinated via inbox/outbox messaging. |
| 05 | `compliance-audit-project` | Long duration, 4 objectives / 12 KRs. Auditor roles. Alerted audit_class throughout. `delete_after_years: 7` retention on Auth Requests. |

### O10: Cross-link from existing docs  `[PLAN: new]` `[DOCS: ⏳]`

- `concepts/README.md` — add entries for organization layouts + project layouts.
- `concepts/organization.md` — "See [Organization Layouts](../organizations/README.md) for 10 worked examples" note.
- `concepts/project.md` — same for projects.
- `concepts/permissions/08-worked-example.md` — note at top that the single end-to-end example is complemented by the 10-layout catalogue.

### O11: Verification  `[PLAN: new]` `[DOCS: ⏳]`

1. Structural check — both folders present with README + expected files (10 + 5).
2. YAML readability — each config is valid-looking YAML (keys match spec).
3. Cross-reference check — every `](../concepts/...)` link resolves.
4. phi-core-type usage — every `agent_roster` entry cites `AgentProfile` + `ModelConfig` + `ExecutionLimits`.
5. Catalogue consistency — every resource referenced in a project file's `resource_boundaries` exists in the owning org's `resources_catalogue`.
6. Spec-update verification — `resources_catalogue` in permissions/07, Step 0 in permissions/04, `objectives`/`key_results` in project.md, inbox/outbox composites in permissions/01 and ontology.md, `parallelize` field in agent.md, `system-agents.md` exists with the two agents.
7. Messaging-vs-task-queue distinction — agent.md cross-references are consistent.
8. Parallelize semantics — agent.md's LWW cross-reference to coordination.md resolves.

## Critical Files  `[PLAN: new]` `[DOCS: n/a — reference list]`

| File | Edit(s) |
|------|---------|
| `baby-phi/docs/specs/plan/<random>-org-project-layouts.md` (NEW) | O0 |
| `permissions/01-resource-ontology.md` | O1 (Catalogue section), O3 (inbox/outbox composites) |
| `permissions/04-manifest-and-resolution.md` | O1 (Step 0 in pseudocode) |
| `permissions/05-memory-sessions.md` | O3 (inbox/outbox section) |
| `permissions/07-templates-and-tools.md` | O1 (`resources_catalogue`), O5 (`system_agents`) |
| `concepts/organization.md` | O1 (catalogue property), O10 (cross-link) |
| `concepts/project.md` | O2 (OKR fields + value objects), O10 (cross-link) |
| `concepts/agent.md` | O3 (messaging section), O4 (parallelize field), O5 (system-agents cross-link) |
| `concepts/ontology.md` | O2 (Objective + KeyResult value objects), O3 (InboxObject + OutboxObject nodes + edges + AgentMessage value object), O4 (AgentProfile.parallelize note) |
| `concepts/system-agents.md` (NEW) | O5 |
| `concepts/README.md` | O10 |
| `concepts/permissions/08-worked-example.md` | O10 |
| `v0/organizations/` (NEW folder, 11 files) | O6, O7 |
| `v0/projects/` (NEW folder, 6 files) | O8, O9 |

## What Stays Unchanged  `[PLAN: new]` `[DOCS: n/a — scope guard]`

- Phases A–H content. Spec additions are additive — no existing rule is changed.
- The 14 tool manifest examples in `permissions/07` — remain the tool reference catalogue.
- The split `permissions/` folder. O1 and O3 add content to existing files; no new permissions files.
- Market spec remains `[OUT OF V0 SCOPE]`. Org #08 `marketplace-gig` is structured to participate when Market lands, but the Market mechanics are not spec'd here.
- phi-core types are referenced, not redefined.

## Verification Summary  `[PLAN: new]` `[DOCS: ⏳ pending]`

1. Plan archived.
2. Five spec extensions (O1–O5) land cleanly, all cross-file consistent.
3. `v0/organizations/` has README + 10 layout files, each with full YAML using phi-core types.
4. `v0/projects/` has README + 5 layout files.
5. Resource-catalogue rule demonstrated in every org layout (each org's YAML has a populated `resources_catalogue`).
6. Inbox/outbox used in at least 3 layouts (marketplace-gig prominently, education-org, joint-research).
7. Parallelized sessions demonstrated in at least 3 layouts (research-lab, mid-product-team, platform-infra).
8. Both system agents present in every org's `system_agents:` section.
9. OKRs used in at least 4 project layouts (01, 02, 03, 05) with varying cadence.
10. All cross-links resolve.
