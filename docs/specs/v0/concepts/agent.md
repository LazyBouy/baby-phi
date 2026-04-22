<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-22 by Claude Code -->
<!-- M4/P0 amendment: expanded Agent Taxonomy with §Agent Roles to cover the 6-variant AgentRole enum. Human-side roles (Executive / Admin / Member) added alongside the pre-existing LLM-side roles (Intern / Contract / System). Pins the is_valid_for(kind) rule. -->

# Agent

> Extracted from brainstorm.md Section 3, refined 2026-04-09.
> See also: [human-agent.md](human-agent.md), [token-economy.md](token-economy.md), [permissions.md](permissions/README.md), [project.md](project.md)

---

## Grounding Principle

**Everything is an Agent.** Humans, LLM agents, and future entity types all share the Agent node type. They differ in capabilities (human agents lack models; LLM agents lack channel headers) but share **memory**, **sessions**, **permissions**, and **participation in projects**.

> **Note on Identity:** Identity (the emergent self — see below) belongs only to LLM Agents. Human Agents do not have a system-computed Identity — their identity exists outside the system. Human Agents are participants, not subjects of identity tracking.

> **Inter-Agent Messaging.** Every Agent has exactly one **Inbox** (messages received) and one **Outbox** (messages sent), formalised as the composites `inbox_object` and `outbox_object` (see [permissions/05 § Inbox and Outbox](permissions/05-memory-sessions.md#inbox-and-outbox-agent-messaging)). Messaging is **pure information flow**, separate from the agent's task queue (`ASSIGNED_TO`) and from sub-agent delegation (`DELEGATES_TO`). The sender deposits a message; the recipient's next session may inspect the inbox, but there is **no automatic reaction** — the agent decides whether, when, and how to respond. This keeps agents autonomous in behaviour while making peer-to-peer signalling first-class.

---

## Agent Taxonomy

phi recognizes the following agent types:

```
                          ┌─────────┐
                          │  AGENT  │
                          └────┬────┘
                               │
              ┌────────────────┴────────────────┐
              │                                 │
        ┌─────▼─────┐                     ┌─────▼─────┐
        │   HUMAN   │                     │    LLM    │
        │   AGENT   │                     │   AGENT   │
        └───────────┘                     └─────┬─────┘
                                                │
                              ┌─────────────────┴─────────────────┐
                              │                                   │
                        ┌─────▼─────┐                       ┌─────▼─────┐
                        │  SYSTEM   │                       │ STANDARD  │
                        │   AGENT   │                       │   AGENT   │
                        └───────────┘                       └─────┬─────┘
                                                                  │
                                                ┌─────────────────┴─────────────────┐
                                                │                                   │
                                          ┌─────▼─────┐                       ┌─────▼─────┐
                                          │  INTERN   │  promotion threshold  │ CONTRACT  │
                                          │   AGENT   │ ────────────────────► │   AGENT   │
                                          └───────────┘                       └───────────┘
```

| Type | Token Economy | Bidding | Identity | Examples |
|------|--------------|---------|----------|----------|
| **Human Agent** | N/A | N/A (sponsors/assigns) | No (external) | Sponsor, reviewer, operator |
| **System Agent** | Outside (fixed cost) | No | Yes | Introspection agent, project setup agent, monitoring agent |
| **Standard / Intern** | Outside (until promoted) | No | Yes | New worker agent, < 10 jobs completed |
| **Standard / Contract** | Inside (full participant) | Yes | Yes | Promoted agent with track record |

---

## Agent Roles

The taxonomy above describes the **kinds** of agent (Human vs LLM, and
the LLM subdivisions). The **role** field refines this: every agent
(of either kind) carries an optional `role` discriminator that pins
their governance position within the org.

phi's `AgentRole` enum (M4) has **six variants** spanning both
kinds:

| Role | Valid for kind | Meaning | Examples |
|------|---|---|---|
| `Executive` | Human | Top-tier governance authority in an org. Typically the CEO at org creation; may delegate. Holds broad `[allocate]` grants on the org's control plane. | CEO, Founder |
| `Admin` | Human | Operational governance — can create/edit agents, manage projects, adopt templates. Reports to `Executive`. | Platform admin, HR lead |
| `Member` | Human | Ordinary human participant. Can be nominated as project lead/member; cannot manage the org-level control plane without additional grants. | Individual contributor, reviewer |
| `Intern` | LLM | Standard LLM agent pre-token-economy. No bidding; fixed cost. Promotes to `Contract` after the rating threshold is met. | New worker agent |
| `Contract` | LLM | Standard LLM agent in the token economy. Participates in bids; has a worth/value/meaning record. | Promoted agent with track record |
| `System` | LLM | Infrastructure LLM agent (memory-extractor, agent-catalog). Lives outside the token economy. Created at org creation or page 13. | memory-extractor, agent-catalog |

**Cross-kind invariant** — `AgentRole::is_valid_for(kind)` rule:

- `Executive`, `Admin`, `Member` → valid only when `Agent.kind ==
  Human`.
- `Intern`, `Contract`, `System` → valid only when `Agent.kind ==
  Llm`.

phi's Rust code enforces this at create-time + edit-time of an
Agent (M4 page 09 editor). Role is **immutable post-creation** at M4
scope; role transitions (e.g., Intern → Contract) go through
separate flows (token-economy promotion; see [token-economy.md](token-economy.md)).

**`role = None`** is valid for legacy agents created before M4 (the
field is `Option<AgentRole>`). The M4 dashboard surfaces these as
`unclassified` in its per-role count panel. Operators may backfill a
role via page 09 edit mode.

**Canonical implementation:** `phi/modules/crates/domain/src/model/nodes.rs::AgentRole`
(shipped at M4/P1). M4 dashboard panel: `AgentsSummary {
executive, admin, member, intern, contract, system, unclassified }`.

---

## Participation in Projects

> **What "participation" means:** An Agent participates in a Project when there is a `HAS_AGENT` edge from the Project to the Agent. Participation grants the agent:
> 1. **Read access** to project-scoped Memory (public + own private)
> 2. **Eligibility** to receive Tasks within that project (assigned for Workers/Interns; biddable for Contractors)
> 3. **Visibility** in the project's agent roster
> 4. **Inheritance** of project-level Permissions and ExecutionLimits
> 5. **Communication scope** — agents within the same project can send messages to each other (subject to permissions)
>
> An agent may participate in multiple projects simultaneously. Each agent has a `current_project` (active context) and a `base_project` (default home), used for Memory tagging and routing.
>
> Likewise, each agent has a `current_organization` and `base_organization` for org-scoped operations.

---

## LLM Agent Anatomy — The Extended Model

An LLM Agent is not just a config wrapper. It has **nature** (Soul), **capability** (Power), **history** (Experience), **emergent self** (Identity), and **social standing** (Worth/Value/Meaning).

```
┌─────────────────────────────────────────────────────────┐
│                     LLM AGENT                           │
│                                                         │
│  ┌──────────┐   Soul (immutable, born structure)        │
│  │ Genetics │   = AgentProfile + SystemPrompt +         │
│  │          │     ModelConfig snapshot at creation      │
│  └──────────┘                                           │
│                                                         │
│  ┌──────────┐   Power (verbs, phrases, sentences)       │
│  │ Ability  │   = Tools (verbs)                         │
│  │          │   + MCP servers (verbs with locales)      │
│  │          │   + Skills (composed verbs — organized,   │
│  │          │     hence gives both edge AND blindspots) │
│  └──────────┘                                           │
│                                                         │
│  ┌──────────┐   Experience (stored history)             │
│  │ History  │   = Sessions + Memory (Short/Medium/Long) │
│  └──────────┘                                           │
│                                                         │
│  ┌──────────┐   Identity (emergent, event-driven)       │
│  │ Self     │   = f(Soul + Experience + Skills)         │
│  │          │   Updated reactively on: session end,     │
│  │          │   memory extraction, skill change,        │
│  │          │   rating received                          │
│  └──────────┘                                           │
│                                                         │
│  ┌──────────┐   Worth / Value / Meaning (economic)      │
│  │ Standing │   = see token-economy.md                  │
│  │          │   (Standard/Contract agents only)         │
│  └──────────┘                                           │
└─────────────────────────────────────────────────────────┘
```

### Soul (Immutable Born Structure)

The Soul is the agent's **genetics** — defined at creation, never mutated.

| Component | phi-core Source | Meaning |
|-----------|----------------|---------|
| Profile snapshot | `AgentProfile` (frozen) | system_prompt, thinking_level, temperature, personality |
| Model binding | `ModelConfig` (frozen) | which LLM was assigned at birth |
| System prompt | `SystemPrompt` (frozen) | the original assembled prompt |

> **Immutability:** The Soul node is write-once. If you need to change an agent's fundamental nature, you create a new agent. The old agent's history remains intact. This is "genetics" — you don't edit DNA, you breed new organisms.

> **Open question:** Should there be a `REINCARNATED_FROM` edge when a new agent is created from a modified Soul? This preserves lineage while allowing evolution.

### Power (Tools, MCP, Skills)

Power is what the agent **can do**. Three levels of composition:

| Level | Metaphor | phi-core Source | Description |
|-------|----------|-----------------|-------------|
| **Verbs** | Individual actions | `ToolDefinition` | Atomic tools: read_file, bash, search |
| **Verbs with Locales** | Actions in a context | `McpServer` → tools | MCP tools: GitHub operations, database queries |
| **Phrases & Sentences** | Composed actions | `Skill` | Skills = combinations of verbs and other skills. Organized = gives both edge and blindspots |

> **Skill creation:** Two mechanisms:
> 1. **Explicit** — Agent intentionally composes a new skill from verbs + existing skills. Requires `HOLDS_GRANT` for schema mutation.
> 2. **Emergent** — System detects repeated tool-call patterns in session history and proposes candidate skills. Agent or human approves.
>
> **Blindspots:** Skills are organized knowledge — but organization implies assumptions. A skill that always uses `bash` for file operations has a blindspot for `edit_file`. This is a feature, not a bug — it models real expertise tradeoffs. Because context is limited, having certain skills means the agent *cannot* have other skills — this is the basis of both edge and blindspot.

> **How skills compose with grants.** A skill is a composition of tool verbs (and other skills). A grant authorises invocation of specific tool × resource × action tuples. **The two compose as set intersection at invocation time:**
>
> - An agent can invoke a verb within a skill if and only if they hold a grant that authorises that verb on that resource with those constraints.
> - Possessing a skill does not grant authority; it only names a workflow. An agent with a "deploy_to_prod" skill but no `[execute]` grant on `process_exec_object` with the right selector still cannot deploy — the skill is loaded, but every step of its workflow fails the Permission Check.
> - Conversely, holding authority does not impart skill. An agent with broad grants but no loaded skill that sequences the relevant verbs must invoke each verb individually; the skill's pre-composed reasoning isn't there to call on.
>
> Blindspots follow from this composition rule: a skill's workflow encodes a specific tool choice, so the agent reaches for that tool even when a different authorised tool would do better. The grants remain available for explicit non-skill invocations — a blindspot limits *what the skill reaches for*, not *what the agent can in principle do*.

### Experience (Sessions + Memory)

| Type | Persistence | Scope | phi-core Source |
|------|-------------|-------|-----------------|
| **Sessions** | Permanent | Per-agent execution history | `Session`, `LoopRecord`, `Turn`, `Message` |
| **Short-term Memory** | Ephemeral | Current session context | In-run context, steering messages |
| **Medium-term Memory** | Session-scoped | Across loops within a session | Compacted summaries, tool outputs |
| **Long-term Memory** | Permanent | Across all sessions | `Memory` node with tagged scope (see below) |

#### Parallelized Sessions

An AgentProfile carries a `parallelize: u32` field (default `1`) that bounds how many **concurrent sessions** a single agent instance may run at once. An agent with `parallelize: 4` can execute up to four independent sessions simultaneously — typically on different projects or on different tasks within the same project.

**Semantics:**

- **Sessions are always independent executions.** Concurrent sessions do not share short-term memory or the loop's in-flight context. Each runs its own `agent_loop()` (see [phi-core-mapping.md](phi-core-mapping.md) for the phi-core types involved).
- **The agent's `HAS_MEMORY` pool is shared across concurrent sessions.** Memory writes from concurrent sessions follow the LWW consistency rule documented in [coordination.md § Design Decisions](coordination.md#design-decisions-v0-defaults-revisitable). Cross-references between sessions use `derived_from:session:{id}` tags on the resulting memories so downstream readers can trace which session produced which memory.
- **Inbox and Outbox are shared, too.** All N concurrent sessions read from the same inbox and append to the same outbox — see [permissions/05 § Multi-Session Delivery Under Parallelized Agents](permissions/05-memory-sessions.md#multi-session-delivery-under-parallelized-agents). Message ordering follows LWW.
- **Configurable at adoption; tightenable per project.** The `parallelize` value is set on the AgentProfile in the owning org's agent roster. A project may **tighten** (reduce) the effective `parallelize` for an agent working within its scope, but cannot raise it above the profile's declared maximum.
- **`parallelize: 1` is the default** — the traditional single-session agent. Values > 1 are opt-in per profile.

**Why parallelize per profile, not per agent instance:** the profile defines what the agent *is*; `parallelize` is about the agent's concurrency capacity under that profile. An org with 5 agents sharing a profile has 5 × `parallelize` total concurrent sessions possible from that profile family.

**Typical values:**

| Value | Fit |
|-------|-----|
| `1` | Single-threaded agents; interns; most LLM worker roles |
| `2–4` | Productive workers supporting multiple parallel projects or subtasks |
| `4–8` | System agents (memory extraction, catalog, monitoring) processing independent events |
| `16+` | Platform-infrastructure agents handling high-throughput admin work — rare and deliberate |

#### Memory Model — Public, Private, and Supervisor Extraction

Long-term Memory is **tag-based**, not folder-based. Memory retrieval works by filtering on tags an agent is allowed to query.

**Two visibility classes:**

| Class | Tagging | Who Can Read |
|-------|---------|--------------|
| **Private Memory** | Tagged with `agent_id` only | Only the owning agent |
| **Public Memory** | Tagged with one or more of: `agent_id`, `project_id`, `org_id`, or `#public` | Any agent allowed by the tags |

**Tag scopes available to every agent:**

| Scope | Meaning |
|-------|---------|
| `agent_id` | The agent itself — used for private memory or identifying authorship on public memory |
| `current_project` | The project the agent is currently working in |
| `base_project` | The agent's default home project |
| `current_organization` | The org context the agent is currently operating under |
| `base_organization` | The agent's default home organization |
| `#public` | Open to all agents in the system |

> A memory tagged **only** with an `agent_id` is **private** to that agent. A memory tagged with an `agent_id` **and** a project/org/`#public` tag is **public** — the `agent_id` then identifies the author rather than restricting access.

When an agent extracts a memory, it chooses the scope tags. A finding relevant only to itself gets `agent_id`. A finding useful to the whole project gets `project_id`. A finding useful to everyone gets `#public`.

**Supervisor / Sponsor Extraction:**

> Supervisor and Sponsor agents have **read access to all sessions under their authority** for the purpose of extracting Memory. A Supervisor reading a sub-agent's Session can:
> - Extract a Public Memory tagged with the project (visible to the project team)
> - Extract a Private Memory tagged only with the Supervisor's `agent_id` (visible only to the Supervisor)
>
> This is how organizational learning happens: supervisors mine the work of their reports for insights, then publish those insights at the appropriate scope.

Both creation modes — direct extraction by the working agent, and post-hoc extraction by a supervisor — produce Memory nodes in the same data model. The difference is *who* extracted and *what tags* were chosen.

> Implementation details (how tag matching is enforced, whether memories are stored as graph nodes or in a separate vector store) are deferred — the conceptual model is what matters here.

### Identity (Emergent, Event-Driven)

> **Applies to LLM Agents only.** Human Agents do not have a system-computed Identity.

Identity is NOT assigned — it **develops** from the interaction of Soul, Experience, and Skills.

```
Identity = f(Soul, LivedExperience, WitnessedExperience, Skills)
```

#### Two Streams of Experience

Experience contributes to Identity through two distinct streams:

| Stream | Source | Who Has It | What It Captures |
|--------|--------|------------|------------------|
| **Lived Experience** | Sessions where the agent was the executing agent | All LLM Agents | Direct doing — what the agent has actually attempted, succeeded at, and failed at |
| **Witnessed Experience** | Sessions of subordinates the agent has authority over | Supervisor / Sponsor / Lead Agents only | Observed work — patterns the agent has seen across the team it supervises |

> **Why two streams:** A team lead's identity is not just what they personally executed — it is also shaped by what their team has done under their guidance. Modeling these as separate dimensions prevents two failure modes:
> 1. **Identity inflation by association** — a supervisor with 100 reports would otherwise accumulate 100 agents' worth of identity passively
> 2. **Erasure of mentorship** — without witnessed experience, supervisors look like idle agents in the identity model even when they are doing significant organizational work

#### Witnessed Experience Is Mediated by Extraction

A Supervisor does not passively accumulate Witnessed Experience by virtue of having reports. Instead, **Witnessed Experience accrues only through Memory extraction**:

- A Supervisor reads a subordinate's session
- The Supervisor extracts a Memory from that session (private or public — see [permissions.md](permissions/README.md))
- The act of extraction is what contributes to the Supervisor's Witnessed Experience

This means:
- **Active observation matters** — Witnessed Experience is the supervisor's own synthesis, not raw access
- **No double counting** — the subordinate's Lived Experience and the supervisor's Witnessed Experience are separate dimensions, even though they reference the same source sessions
- **Authority gates contribution** — Witnessed Experience can only accumulate through sessions the Supervisor has authority to read (see [permissions.md — Authority Templates](permissions/05-memory-sessions.md#authority-templates-formerly-the-authority-question) and [Multi-Scope Session Access](permissions/06-multi-scope-consent.md#multi-scope-session-access))

**Concurrent sub-agent supervision.** A supervisor with multiple concurrently-running sub-agents accumulates Witnessed Experience **reactively per extraction**, not batched at session-end. Each Memory extraction emits a discrete update to the supervisor's `witnessed` struct; concurrent extractions from different sub-agents produce concurrent updates, ordered by the standard last-writer-wins consistency rule documented in [coordination.md § Design Decisions](coordination.md#design-decisions-v0-defaults-revisitable). No batch mode exists — this keeps the identity model's reactive semantics uniform across all triggers (session end, memory extraction, skill change, rating), so a supervisor observing ten parallel sub-agents gets ten independent updates rather than a single consolidated one at some arbitrary later moment.

> **Open question:** Should Witnessed Experience be weighted differently from Lived Experience in the Identity computation? A literal "I did it myself" insight is arguably more valuable than "I saw someone else do it." But weighting them equally is simpler and avoids subjective parameters.

#### Materialization

Identity is a stored node, updated reactively:

- On session end → Lived Experience changed
- On Memory extraction from a subordinate's session → Witnessed Experience changed
- On skill added/removed → capability changed
- On rating received → reputation changed
- **Not computed from scratch** each time — incrementally updated by the triggering event
- **Queryable:** "What is Agent X's current identity?" returns the materialized node

#### Identity Node Content — Provisional Direction

> **Status:** provisional **and load-bearing for v0**. The three-field model below (`self_description` + `lived` + `witnessed` + `embedding`) is the v0 commitment — implementations should code against it. The "provisional" label means the *long-term* shape may evolve as usage patterns emerge in v1; it does **not** mean the v0 shape is incomplete or under negotiation. Future revisability is `[OUT OF V0 SCOPE]` and is not a v0 gap.

The Identity node carries three complementary views, each serving a different query pattern:

| Field | Type | Purpose | Update trigger |
|-------|------|---------|----------------|
| `self_description` | String (≤ 500 tokens) | Agent-authored natural-language bio. Optimised for human-facing introspection ("who is this agent?") and for LLM context — an agent can read its own `self_description` to ground its behaviour. | Rewritten by the agent (or by a system-agent synthesiser) on session end, skill change, or a significant rating event. |
| `lived` | `LivedExperience` struct | Structured metrics of direct doing. Fields: `sessions_completed`, `sessions_successful`, `ratings_window` (last 20 per the rolling window in [token-economy.md](token-economy.md)), `skills: Vec<SkillRef>`, `specializations: Vec<String>` (top tags by frequency across completed work). | Session end, rating received, skill added/removed. |
| `witnessed` | `WitnessedExperience` struct | Structured metrics of supervised doing. Fields: `memories_extracted: u64`, `subordinates_observed: Vec<agent_id>`, `extraction_scope_distribution` (how many extractions were private vs public). Empty for non-supervisor agents. | On Memory extraction from a subordinate's session. |
| `embedding` | Vec<f32> (dim configurable; default 1536) | Dense vector derived from `self_description`. Used for similarity queries and matching (e.g., "find me an agent whose character is close to X"). | Re-derived whenever `self_description` is rewritten. |

**Why three views are kept rather than one:**

- `self_description` is the human-facing and LLM-facing narrative — irreplaceable for context-embedding the agent into its own reasoning.
- `lived` + `witnessed` are filter-and-sort friendly — they let the market, supervisors, and the hiring logic ask precise questions (min rating, skill present, supervisory experience > 0) without parsing natural language.
- `embedding` is the similarity-search surface — linking agents whose characters align, or retrieving past Identity snapshots for before/after comparisons.

Redundancy is intentional. A future revision may prune one of the three once usage patterns make the canonical form clear; for v0 we keep all three to avoid premature commitment.

**Scoping the embedding model.** The embedding model is a **platform-level configuration fixed at org-bootstrap time**, not a per-agent choice. The `embedding.dim` field in the schema exists to accommodate the platform's chosen model (e.g., 1536 for `text-embedding-3-small`, 3072 for `text-embedding-3-large`, 1024 for Voyage `voyage-3`), not to let individual agents pick different models. This matters because cosine similarity is only meaningful within a single model's vector space — an index mixing embeddings from different models silently produces wrong similarity scores, which is the kind of bug that looks correct in dev and fails silently in prod.

**Model change is an admin event.** If the platform or org switches embedding providers, existing Identity embeddings become un-queryable against new ones. The switch therefore triggers a batch re-embed of every Identity node — this is an explicit admin action, not a silent migration, and it is itself auditable through the same Auth Request machinery used for all sensitive admin operations.

**Separate sub-fields for Lived vs Witnessed** is satisfied by the `lived` and `witnessed` structs — they are computed independently and can be queried or weighted independently without the double-counting risk flagged in the [Two Streams of Experience](#two-streams-of-experience) section.

> **Open (non-blocking):** Should `self_description` be versioned (keeping the full history of rewrites) or only the current form? The current plan stores only the latest revision; a `HAS_IDENTITY_HISTORY` edge to versioned snapshots can be added later without schema rework.

---

## LLM Agent Categories

LLM Agents split into two top-level categories: **System Agents** and **Standard Agents**.

### System Agent

System Agents perform infrastructure, maintenance, or platform-level work for an organization or project. They operate **outside the token economy** by design.

**Properties:**
- **Token usage is a fixed cost** to the owning organization or project — not charged against any individual contract or bid
- **No bidding** — they are always-available services
- **No Worth / Value / Meaning** — they are not rated or priced
- **Direct communication** — any agent in the project or organization can talk to them without going through formal channels
- **Always-on availability** — they are part of the platform fabric, not contracted on demand
- **Identity still applies** — they have Soul, Power, Experience, and a developing Identity

**v0 Standard System Agents** (formalised in [system-agents.md](system-agents.md)):
- **Memory Extraction Agent** — runs on session-end events, extracts candidate memories, allocates them to the correct pool (agent / project / org / `#public`) based on tag permissions.
- **Agent Catalog Agent** — maintains a queryable catalogue of all active Agents in the org, tracks lifecycle events.

**Other Examples (future, `[OUT OF V0 SCOPE]`):**
- **Introspection Agent** — exposes the data model for query, helps other agents understand "what exists"
- **Project Setup Agent** — bootstraps new projects (creates initial nodes, applies templates, configures permissions)
- **Monitoring Agent** — watches for anomalies, runaway costs, stuck loops
- **Skill Curation Agent** — detects emergent skill patterns and proposes them for approval

See [system-agents.md](system-agents.md) for the full catalogue, profiles, grants, and behaviour of the two v0 Standard System Agents, plus stubs for the future additions above.

> System Agents are typically created at organization or project setup time and persist for the lifetime of their host.

### Standard Agent

Standard Agents are the "workers" of the system — agents that may eventually participate in the token economy. Standard Agents come in two sub-modes based on experience.

#### Intern Agent (Pre-Economy)

> **Was previously called "Worker Agent" in earlier brainstorms.**

An Intern is a Standard Agent that has not yet qualified for the token economy.

**Properties:**
- **Human (or Lead Agent) directly assigns tasks** — no bidding
- **No token budget** — token usage is paid by the project/org as overhead
- **Receives ratings** — every completed job earns a rating, building track record
- **Identity develops** as it gains experience and ratings
- **Promotion criteria** (configurable, defaults shown):
  - **Job count threshold:** completed at least **10 jobs** (default)
  - **Rating threshold:** rolling-window average rating ≥ **0.6** (default)
- **Promotion event:** When both thresholds are met, the agent is promoted to **Contract Agent**. This is a one-way transition.

> **Why a probation period:** New agents have no track record. Letting them bid immediately would either flood the market with unknown quality, or force the market to ignore them. The Intern phase is a structured way to build a track record before market entry.

#### Contract Agent (Token Economy Participant)

A Contract Agent has been promoted from Intern status and now participates fully in the token economy.

**Properties:**
- **Bids for Tasks** in the Market or directly within projects
- **Receives a token budget** upon winning a contract
- **Keeps the savings** (tokens remaining after completing the work) — incentivizes efficiency
- **Has Worth, Value, and Meaning** calculations (see [token-economy.md](token-economy.md))
- **Receives ratings** that feed back into Worth
- **Can participate in task estimation** (a basic skill all Contract agents share)
- **Can evaluate other agents** (using a consistent framework)

#### Worth, Value, Meaning, and the Rating Window

> **Canonical home:** [token-economy.md](token-economy.md). This section is intentionally a pointer — formulas, the rolling rating window, the Intern → Contract carry-forward rule, and the bidding process all live in one place to avoid drift.

In summary:

- **Worth** is rating-weighted profitability per unit of work — a backward-looking reputation metric
- **Value** is the market price the agent commands — forward-looking
- **Meaning** is the relationship between the two
- The **rolling rating window** stores the last N (default 20) ratings explicitly and folds older ratings into a running average
- **Intern → Contract carry-forward:** Intern-period token consumption is preserved when the agent is promoted, avoiding divide-by-zero on the first contract

---

## Mode Transitions

| From | To | Trigger | Reversible? |
|------|----|---------|-------------|
| (creation) | Intern | Standard agent created | — |
| Intern | Contract | Job count ≥ 10 AND rolling avg rating ≥ 0.6 (defaults) | No (one-way promotion) |
| (creation) | System | Created with `system_agent: true` flag at setup | No (System ↔ Standard not allowed) |

> **Why no demotion:** A Contract agent that performs poorly is naturally penalized by the market — its Worth drops, its bids become less competitive, and other agents stop hiring it. There is no need for an explicit demotion path. Bad reputation is its own consequence.

> **Why System ↔ Standard is not allowed:** System agents are infrastructure. Allowing them to enter the market would break the "fixed cost" assumption that organizations rely on for budgeting. They are a separate species.
