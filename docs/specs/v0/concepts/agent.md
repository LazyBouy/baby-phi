<!-- Status: CONCEPTUAL -->

# Agent

> Extracted from brainstorm.md Section 3.
> See also: [human-agent.md](human-agent.md), [token-economy.md](token-economy.md), [permissions.md](permissions.md)

---

## Grounding Principle

**Everything is an Agent.** Humans, LLM agents, and future entity types all share the Agent node type. They differ in capabilities (human agents lack models; LLM agents lack channel headers) but share identity, memory, sessions, permissions, and participation in projects.

---

## Agent Anatomy — The Extended Model

An Agent is not just a config wrapper. It has **nature** (Soul), **capability** (Power), **history** (Experience), **reputation** (Worth), and **social position** (Value/Meaning).

```
┌─────────────────────────────────────────────────────────┐
│                        AGENT                            │
│                                                         │
│  ┌──────────┐   Soul (immutable, born structure)        │
│  │ Genetics │   = AgentProfile + SystemPrompt +         │
│  │          │     ModelConfig snapshot at creation       │
│  └──────────┘                                           │
│                                                         │
│  ┌──────────┐   Power (verbs, phrases, sentences)       │
│  │ Ability  │   = Tools (verbs)                         │
│  │          │   + MCP servers (verbs with locales)       │
│  │          │   + Skills (composed verbs — organized,    │
│  │          │     hence gives both edge AND blindspots)  │
│  └──────────┘                                           │
│                                                         │
│  ┌──────────┐   Experience (stored history)              │
│  │ History  │   = Sessions + Memory (Short/Medium/Long)  │
│  └──────────┘                                           │
│                                                         │
│  ┌──────────┐   Identity (emergent, event-driven)        │
│  │ Self     │   = f(Soul + Experience + Skills)          │
│  │          │   Updated reactively on: session end,      │
│  │          │   skill added, rating received             │
│  └──────────┘                                           │
│                                                         │
│  ┌──────────┐   Worth / Value / Meaning (economic)       │
│  │ Standing │   = see token-economy.md                   │
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
> 1. **Explicit** — Agent intentionally composes a new skill from verbs + existing skills. Requires `HAS_PERMISSION(schema_mutation)`.
> 2. **Emergent** — System detects repeated tool-call patterns in session history and proposes candidate skills. Agent or human approves.
>
> **Blindspots:** Skills are organized knowledge — but organization implies assumptions. A skill that always uses `bash` for file operations has a blindspot for `edit_file`. This is a feature, not a bug — it models real expertise tradeoffs. Because context is limited, having certain skills means the agent *cannot* have other skills — this is the basis of both edge and blindspot.

### Experience (Sessions + Memory)

| Type | Persistence | Scope | phi-core Source |
|------|-------------|-------|-----------------|
| **Sessions** | Permanent | Per-agent execution history | `Session`, `LoopRecord`, `Turn`, `Message` |
| **Short-term Memory** | Ephemeral | Current session context | In-run context, steering messages |
| **Medium-term Memory** | Session-scoped | Across loops within a session | Compacted summaries, tool outputs |
| **Long-term Memory** | Permanent | Across all sessions | `Memory` node (user/feedback/project/reference types) |

### Identity (Emergent, Event-Driven)

Identity is NOT assigned — it **develops** from the interaction of Soul, Experience, and Skills.

```
Identity = f(Soul, Experience, Skills)
```

- **Materialization:** Identity is a stored node, updated reactively:
  - On session end -> experience changed
  - On skill added/removed -> capability changed
  - On rating received -> reputation changed
- **Not computed from scratch** each time — incrementally updated by the triggering event
- **Queryable:** "What is Agent X's current identity?" returns the materialized node

> **Open question:** What does the Identity node actually contain? Candidates:
> - A summary embedding (vector representation of the agent's character)
> - A structured profile (strengths, weaknesses, specializations as fields)
> - A natural language self-description (agent writes its own bio)
> - All of the above

---

## Agent Modes

Not all agents participate in the economy. Two modes:

### Contract Agent (Bidder)

- Participates in bidding for Tasks
- Receives a token budget upon winning a contract
- Keeps savings (tokens remaining after completing work)
- Has Worth, Value, and Meaning calculations
- Receives ratings on project completion
- Can participate in task estimation (a basic skill)
- Can evaluate other agents (a consistent framework)

### Worker Agent (Assigned)

- Human directly assigns tasks
- No bidding, no token budget
- No Worth/Value/Meaning economics
- Simpler model — the current default
- Suitable for single-task, single-session use cases

> **Mode is a property of Agent, not a subtype.** An agent can potentially transition from Worker to Contract mode as it gains experience and ratings. The system doesn't prevent it — permissions do.
