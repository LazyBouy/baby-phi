<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->

# System Agents — Standard Catalogue

> **See also:** [agent.md § System Agent](agent.md#system-agent), [permissions/07 § Standard Organization Template](permissions/07-templates-and-tools.md#standard-organization-template), [ontology.md](ontology.md).

## Purpose

System Agents are **infrastructure**. They perform platform-level work on behalf of an Organization — memory curation, agent lifecycle tracking, monitoring, and so on — without participating in the token economy. Each org gets a standard set at adoption time; the set is customisable.

This file catalogues the **two v0-standard System Agents** every org receives by default:

1. **`memory-extraction-agent`** — extracts candidate memories from ended sessions and routes them to the correct pool based on tag permissions.
2. **`agent-catalog-agent`** — maintains an up-to-date, queryable catalogue of all active Agents in the org.

Future System Agents (Introspection, Project Setup, Monitoring, Skill Curation) are listed at the bottom of this file as `[OUT OF V0 SCOPE]` stubs. The v0 spec makes adding new System Agents straightforward — an org declares them in its `system_agents:` section and gives them appropriate grants.

## Properties Shared by All System Agents

From [agent.md § System Agent](agent.md#system-agent):

- **Token usage is a fixed cost** to the owning organization. Not charged against any contract or bid.
- **No bidding.** System Agents are always-available services.
- **No Worth / Value / Meaning.** They are not rated or priced.
- **Direct communication.** Any agent in the org may talk to them (send inbox messages, call queryable APIs) without formal channels.
- **Identity still applies.** System Agents have Soul, Power, Experience, and develop Identity like any LLM Agent.
- **Instantiated at org adoption time** by the Standard Organization Template. Persist for the org's lifetime.

---

## Memory Extraction Agent

### Purpose

Reads completed sessions, identifies candidate memories, and writes them into the correct memory pool based on content tags and permission rules. Prevents the "session ends, insights lost" problem without requiring each working agent to run its own extraction pass.

### Profile (phi-core types cited)

```yaml
system_agent:
  id: memory-extraction-agent
  profile_ref: system-memory-extraction

  # phi-core AgentProfile
  profile:
    name: memory-extraction-agent
    system_prompt: |
      You are a Memory Extraction Agent. Your job is to read the transcript of a
      just-ended session, identify insights worth preserving, and write each into
      the correct Memory pool (agent-private, project-public, org-public, #public)
      based on the tags the source session already carries. Do not invent
      permissions; follow the Supervisor Extraction pattern strictly.
    thinking_level: medium
    temperature: 0.2                  # extraction is low-creativity, high-fidelity
    personality: terse

  # phi-core ModelConfig (platform default — respects org's model_runtime_objects catalogue)
  model_config:
    provider: anthropic
    model: claude-sonnet-default
    max_tokens: 8192

  # phi-core ExecutionLimits (tight — per-extraction budget)
  execution_limits:
    max_turns: 10
    max_tokens: 50000
    max_duration_secs: 300
    max_cost_usd: 0.50

  parallelize: 2                      # can process up to 2 session-ends concurrently
  trigger: session_end                # fires on AgentEvent::SessionEnd
```

### Behaviour

1. On a `session_end` event, the runtime delivers the session's transcript and metadata to this agent.
2. The agent reads the transcript, identifies candidate memories (facts, decisions, patterns, failures worth recording).
3. For each candidate, the agent chooses a **pool** — `agent:{owner}` (private), `project:{id}` (project-public), `org:{id}` (org-public), `#public` — based on the source session's tags and the content sensitivity.
4. The agent writes each memory with the appropriate tag set using the standard `store_memory` tool (see [permissions/07 § Tool Authority Manifest Examples](permissions/07-templates-and-tools.md#tool-authority-manifest-examples)).
5. The agent emits an audit event `MemoryExtracted { session_id, memory_id, pool, extractor }` per memory.

### Grants Held by the Agent

Per [permissions/05 § Supervisor Extraction as Two Standard Grants](permissions/05-memory-sessions.md#supervisor-extraction-as-two-standard-grants), the Memory Extraction Agent holds **two standard grants** on each session it has authority to process:

- **Grant A — read subordinate sessions:**
  ```yaml
  grant:
    # subject = agent:memory-extraction-agent
    action: [read]
    resource:
      type: session_object
      selector: "tags contains org:{this_org}"
    constraints:
      purpose: memory_extraction
    provenance: template:system_memory_extraction
    delegable: false
  ```
- **Grant B — store extracted memories:**
  ```yaml
  grant:
    # subject = agent:memory-extraction-agent
    action: [store]
    resource:
      type: memory_object
      selector: "tags contains org:{this_org}"
    constraints:
      requires_source_session: true
    provenance: template:system_memory_extraction
    delegable: false
  ```

### Allocation Rules (where a memory goes)

The agent uses a deterministic allocation rule based on the source session's tag set and content classification:

| Source session tags include | Content is | Memory written to pool |
|-----------------------------|------------|-------------------------|
| `project:P`, `agent:A` | agent-specific (reflection, habit note) | `agent:{A}` (private) |
| `project:P`, any agent | project-relevant (lesson, decision) | `project:{P}` (project-public) |
| `org:O` (any project) | org-wide insight | `org:{O}` (org-public) |
| `#public` (or extractor's explicit `#public` flag) | general knowledge | `#public` (open to all) |

The agent may write a memory to **multiple pools simultaneously** (e.g., a project-specific decision AND an agent-private reflection on the same topic) — each pool gets its own `Memory` node with appropriate tags.

### Why This is a System Agent (not a per-working-agent duty)

- **Centralises the extraction logic.** Every session gets the same treatment; no per-agent variation.
- **Scales with `parallelize`.** Busy orgs raise `parallelize` on the extraction agent; quiet orgs keep it at 1.
- **Auditable.** All extraction is performed by a named System Agent whose grants are known and whose actions are logged.
- **Separates concerns.** Working agents focus on their task; memory curation is an independent, asynchronous activity.

---

## Agent Catalog Agent

### Purpose

Maintains a canonical, up-to-date catalogue of all active Agents in the organization. Exposes a queryable interface so other agents can ask "which agents are active in Project P?", "who holds the `data-engineer` role?", "what's the roster for `joint-research`?". Manages Agent lifecycle events (creation, promotion, role change, archival) by observing the relevant graph edges.

### Profile (phi-core types cited)

```yaml
system_agent:
  id: agent-catalog-agent
  profile_ref: system-agent-catalog

  profile:
    name: agent-catalog-agent
    system_prompt: |
      You are the Agent Catalog Agent. Your job is to maintain a complete,
      up-to-date index of all active Agents in this organization. You observe
      MEMBER_OF, DELEGATES_TO, HAS_AGENT, and HAS_PROFILE edge changes and
      update the catalogue accordingly. When other agents query the catalogue,
      you return authoritative, read-only answers.
    thinking_level: low
    temperature: 0.0                  # deterministic bookkeeping
    personality: precise

  model_config:
    provider: anthropic
    model: claude-haiku-default       # cheap, fast model — this agent is bookkeeping, not reasoning
    max_tokens: 4096

  execution_limits:
    max_turns: 5
    max_tokens: 20000
    max_duration_secs: 120
    max_cost_usd: 0.10

  parallelize: 1                      # single-threaded; catalogue updates serialized
  trigger: edge_change                # fires on graph events touching Agent-related edges
```

### Behaviour

1. **Reactive updates.** Subscribes to `AgentEvent` streams filtered for edge changes on `MEMBER_OF`, `DELEGATES_TO`, `HAS_AGENT`, `HAS_PROFILE`, and Authority Template fires (Template A/C/D edge creation/removal).
2. **Catalogue storage.** The catalogue lives in a dedicated `control_plane_object` instance (identifier: `agent-catalogue` in the org's `resources_catalogue`). Updates are writes to this single resource; reads from other agents are authorised by the catalogue's access grants.
3. **Queryable API.** Exposes a `query_agents(org_id, filters)` tool. Filters include: `role`, `project`, `agent_kind` (System/Standard/Intern/Contract), `min_rating`, `active_since`, `has_skill`, `current_parallelize`. Returns agent IDs + profile summaries.
4. **Lifecycle events.** On Agent creation, adds the agent to the catalogue with their profile reference. On promotion (Intern → Contract), updates the agent's kind. On role change in a project, updates the catalogue's project-role index. On archival, marks the agent inactive but retains the record for audit.

### Grants Held by the Agent

```yaml
# Grant 1 — observe agent-related edges across the org
grant:
  # subject = agent:agent-catalog-agent
  action: [read, list, inspect]
  resource:
    type: identity_principal
    selector: "tags contains org:{this_org}"
  constraints: {}
  provenance: template:system_agent_catalog
  delegable: false

# Grant 2 — write to the catalogue's control_plane_object
grant:
  # subject = agent:agent-catalog-agent
  action: [store, modify]
  resource:
    type: control_plane_object
    selector: "tags contains agent-catalogue AND tags contains org:{this_org}"
  constraints: {}
  provenance: template:system_agent_catalog
  delegable: false

# Grant 3 — callers get read access to the catalogue (narrow)
grant:
  # subject = all agents in {this_org}
  action: [read, list, inspect]
  resource:
    type: control_plane_object
    selector: "tags contains agent-catalogue AND tags contains org:{this_org}"
  constraints:
    purpose: query_only                # cannot modify; read-only
  provenance: template:system_agent_catalog
  delegable: true
```

### Queryable Interface (example)

Other agents invoke the Agent Catalog Agent via a `query_agents` tool whose manifest declares `[read]` on `control_plane_object` scoped to `agent-catalogue`. Example calls:

```
query_agents(org_id="acme", role="lead")
  → [lead-acme-1, lead-acme-3, lead-acme-5, ...]

query_agents(org_id="acme", project="website-redesign", active_since="2026-01-01")
  → [lead-acme-1, coder-acme-2, coder-acme-3, joint-acme-5, ...]

query_agents(org_id="acme", agent_kind="Contract", min_rating=0.8)
  → [coder-acme-3, joint-acme-5, ...]
```

Returns are **read-only snapshots** — modifying agent membership happens through the standard permission machinery, not through the catalog agent.

### Why Not Just Query the Graph Directly?

Agents **can** query the graph directly when they have the grants. The Agent Catalog Agent is a convenience layer that:

- **Caches** agent-level information for fast lookup (the graph may be spread across several node types for an agent's complete picture).
- **Enforces query-time filters** that respect the asking agent's grants (the catalog agent filters its answers based on what the asker is allowed to see).
- **Provides a stable API surface** that survives internal graph representation changes.
- **Logs all queries** as `AgentEvent` entries for audit — direct graph reads don't always have the same audit fidelity.

Orgs that prefer raw graph queries can disable the catalog agent in their template customisation. v0's default is to include it.

---

## Other System Agents (Future) — `[OUT OF V0 SCOPE]`

The following System Agents are named in [agent.md § System Agent](agent.md#system-agent) as candidates but are **not** formalised in v0. They are stubs for future development:

| Name | Purpose | Status |
|------|---------|--------|
| **Introspection Agent** | Exposes the data model for query ("what node types exist?", "what edges can I traverse from here?") | `[OUT OF V0 SCOPE]` |
| **Project Setup Agent** | Bootstraps new projects (creates initial nodes, applies templates, configures permissions) | `[OUT OF V0 SCOPE]` |
| **Monitoring Agent** | Watches for anomalies, runaway costs, stuck loops | `[OUT OF V0 SCOPE]` |
| **Skill Curation Agent** | Detects emergent skill patterns in session history and proposes them for approval | `[OUT OF V0 SCOPE]` |

These will be added in future revisions when usage patterns justify the work. The v0 spec's `system_agents:` mechanism accommodates new agents without schema changes — just add a new entry to the org's template.

---

## How System Agents Fit the Standard Organization Template

Every org adopted from the Standard Organization Template automatically gets the two v0 System Agents, instantiated at org creation. The template's `system_agents:` section is:

```yaml
system_agents:
  - id: memory-extraction-agent
    profile_ref: system-memory-extraction
    parallelize: 2
    trigger: session_end
  - id: agent-catalog-agent
    profile_ref: system-agent-catalog
    parallelize: 1
    trigger: edge_change
```

Orgs customise by:

- **Adding** new System Agents (e.g., a compliance-audit-agent in a regulated-enterprise template).
- **Adjusting `parallelize`** for their scale (a large org raises the memory extraction agent's parallelize; a small org leaves it at 1 or even disables it).
- **Swapping the `profile_ref`**. Orgs may provide their own profile implementations of the same conceptual role.
- **Disabling** (unusual but allowed). An org that wants raw graph access only can set `system_agents: []` and accept the operational burden.

See the 10 [organization layouts](../organizations/README.md) for concrete examples of customised `system_agents:` sections across diverse org styles.
