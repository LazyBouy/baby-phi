<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-09 by Claude Code -->

# Ontological Data Model

> Extracted from brainstorm.md Section 2 + Section 3.12, refined 2026-04-09.
> The canonical reference for baby-phi's node types, edge types, value objects, and schema registry.
> See also: [agent.md](agent.md) (extended agent model), [permissions.md](permissions.md) (capability model + Consent), [phi-core-mapping.md](phi-core-mapping.md) (full type mapping)

---

## Design Principle: Agent as the Core Node

The ontology radiates outward from **Agent**. Every entity exists because an Agent needs it. No orphan sessions, no floating configs — everything traces back to an Agent node.

This is a graph-first model (think ontology, not relational tables), even if the storage layer is initially flat files or SQLite. The relationships are first-class.

---

## Node Types (27 total)

### Core Identity

| Node | Identity | phi-core Source | Why it exists |
|------|----------|-----------------|---------------|
| **Agent** | `agent_id` | `BasicAgent` | The nucleus — everything radiates from here |
| **AgentProfile** | `profile_id` | `AgentProfile` | Blueprint: who the agent IS |
| **User** | `user_id` | *baby-phi concept* | Who owns/interacts with agents |

### Execution History

| Node | Identity | phi-core Source | Why it exists |
|------|----------|-----------------|---------------|
| **Session** | `session_id` | `Session` | One logical task or conversation |
| **Loop** | `loop_id` | `LoopRecord` | One agent_loop() call |
| **Turn** | `turn_id` | `Turn` | One LLM round-trip |
| **Message** | generated | `Message` / `AgentMessage` | Atomic unit of conversation (separate node for standalone testing) |
| **Event** | `loop_id + sequence` | `AgentEvent` / `LoopEvent` | Granular execution trace |

### Capability

| Node | Identity | phi-core Source | Why it exists |
|------|----------|-----------------|---------------|
| **ModelConfig** | `id + provider` | `ModelConfig` | Which LLM backs the agent |
| **ToolDefinition** | `name` | `ToolDefinition` | Tool schema sent to LLM |
| **ToolImplementation** | `name` | `BashTool`, etc. | Concrete tool execution logic |
| **Skill** | `name` | `Skill` | Loaded skill with metadata |
| **McpServer** | `name` | `McpClient`, `ServerInfo` | External tool server via MCP |
| **OpenApiSpec** | `spec_id` | `OpenApiConfig` | External API spec |
| **SystemPrompt** | `id` | `SystemPrompt` | Assembled system prompt |
| **EvaluationStrategy** | `name` | `*Evaluation` types | How parallel branches are evaluated |

### Governance

| Node | Identity | phi-core Source | Why it exists |
|------|----------|-----------------|---------------|
| **ExecutionLimits** | generated | `ExecutionLimits` | Constrains agent resources |
| **Permission** | generated | *baby-phi concept* | Capability-based access control |
| **Consent** | `consent_id` | *baby-phi concept* | Subordinate consent record gating Authority Template grants (see [permissions.md → Consent Policy](permissions.md#consent-policy-organizational)) |
| **CompactionPolicy** | generated | `CompactionConfig` | Context management strategy |
| **RetryPolicy** | generated | `RetryConfig` | Error retry behavior |
| **CachePolicy** | generated | `CacheConfig` | Prompt caching behavior |

### Social Structure (baby-phi extensions)

| Node | Identity | Why it exists |
|------|----------|---------------|
| **Project** | `project_id` | Container for work with goal, agents, governance |
| **Task** | `task_id` | Biddable unit of work (optional decomposition) |
| **Bid** | `bid_id` | Agent's proposal for a Task |
| **Rating** | `rating_id` | Quality assessment of agent work |
| **Organization** | `org_id` | Social structure containing agents and projects |
| **Channel** | `channel_id` | How to reach a Human Agent (Slack, email, web UI) |
| **Memory** | generated | Persistent knowledge across sessions |
| **AgentConfig** | `config_name` | Root configuration document |
| **PromptBlock** | `name` | One block within a system prompt strategy |

---

## Edge Types (44+ total)

### Agent-Centric (first-order)

| From | Edge | To | Cardinality | Meaning |
|------|------|----|-------------|---------|
| Agent | `HAS_PROFILE` | AgentProfile | 1:1 | Blueprint identity |
| Agent | `USES_MODEL` | ModelConfig | N:1 | Current LLM backing |
| Agent | `HAS_TOOL` | ToolDefinition | 1:N | Available tools |
| Agent | `HAS_SKILL` | Skill | 1:N | Loaded skills |
| Agent | `HAS_PERMISSION` | Permission | 1:N | Access control |
| Agent | `GOVERNED_BY` | ExecutionLimits | 1:1 | Constraints |
| Agent | `USES_COMPACTION` | CompactionPolicy | 1:1 | Context management |
| Agent | `USES_RETRY` | RetryPolicy | 1:1 | Error retry |
| Agent | `USES_CACHE` | CachePolicy | 1:1 | Prompt caching |
| Agent | `USES_EVALUATION` | EvaluationStrategy | 1:1 | Branch selection |
| Agent | `HAS_SYSTEM_PROMPT` | SystemPrompt | 1:1 | Assembled prompt |
| Agent | `CONNECTS_TO` | McpServer | 1:N | External tool servers |
| Agent | `CONNECTS_TO` | OpenApiSpec | 1:N | External API specs |
| Agent | `RUNS_SESSION` | Session | 1:N | Execution history |
| Agent | `DELEGATES_TO` | Agent | N:N | Sub-agent spawning |
| Agent | `OWNED_BY` | User | N:1 | Who controls this agent |
| Agent | `HAS_MEMORY` | Memory | 1:N | Persistent knowledge |
| Agent | `HAS_CHANNEL` | Channel | 1:N | Human Agent routing |
| Agent | `LOADED_FROM` | AgentConfig | N:1 | Config origin |
| Agent | `MEMBER_OF` | Organization | N:N | Org membership |

### Execution Chain (second-order)

| From | Edge | To | Cardinality | Meaning |
|------|------|----|-------------|---------|
| Session | `CONTAINS_LOOP` | Loop | 1:N | Ordered loop sequence |
| Loop | `CONTINUES_FROM` | Loop | N:1 | Continuation chain |
| Loop | `CONTAINS_TURN` | Turn | 1:N | Turns within a loop |
| Loop | `CONFIGURED_WITH` | LoopConfigSnapshot | 1:1 | Config snapshot |
| Loop | `EMITS` | Event | 1:N | Event stream |
| Turn | `PRODUCES` | Message | 1:N | Messages in a turn |
| Turn | `EXECUTES_TOOL` | ToolDefinition | 1:N | Tool calls |

### Cross-Agent (third-order)

| From | Edge | To | Cardinality | Meaning |
|------|------|----|-------------|---------|
| Session | `SPAWNED_FROM` | Session | N:1 | Sub-agent origin |
| Loop | `SPAWNED_CHILD` | Loop | 1:N | Parent→child |
| Loop | `PARALLEL_WITH` | Loop | N:N | Parallel siblings |

### Capability Wiring

| From | Edge | To | Cardinality | Meaning |
|------|------|----|-------------|---------|
| ToolDefinition | `IMPLEMENTED_BY` | ToolImplementation | 1:1 | Schema→impl |
| McpServer | `PROVIDES_TOOL` | ToolDefinition | 1:N | MCP tools |
| OpenApiSpec | `PROVIDES_TOOL` | ToolDefinition | 1:N | OpenAPI operations |
| SystemPrompt | `CONTAINS_BLOCK` | PromptBlock | 1:N | Prompt blocks |

### Social Structure

| From | Edge | To | Cardinality | Meaning |
|------|------|----|-------------|---------|
| Organization | `HAS_BOARD` | Agent | 1:N | Sponsors/stakeholders |
| Organization | `HAS_CEO` | Agent | 1:1 | Leader |
| Organization | `HAS_PROJECT` | Project | 1:N | Projects |
| Organization | `HAS_MEMBER` | Agent | 1:N | Members |
| Organization | `HAS_SUBORGANIZATION` | Organization | 1:N | Hierarchy |
| Project | `HAS_SPONSOR` | Agent | 1:N | Funders |
| Project | `HAS_AGENT` | Agent | 1:N | Workers |
| Project | `HAS_TASK` | Task | 1:N | Work items |
| Project | `BELONGS_TO` | Organization | N:N | Ownership |
| Task | `ASSIGNED_TO` | Agent | N:1 | Worker |
| Task | `HAS_BID` | Bid | 1:N | Proposals |
| Bid | `SUBMITTED_BY` | Agent | N:1 | Bidder |
| Rating | `RATES` | Agent | N:1 | Rated agent |
| Rating | `GIVEN_BY` | Agent | N:1 | Rater |

### Governance Wiring

| From | Edge | To | Cardinality | Meaning |
|------|------|----|-------------|---------|
| User | `GRANTS_PERMISSION` | Permission | 1:N | Authorization source |
| Permission | `APPLIES_TO` | Agent | N:N | Permission targets |
| Agent | `HAS_PERMISSION` | Permission | 1:N | Agent-specific grants |
| Project | `HAS_PERMISSION` | Permission | 1:N | Project rules |
| Organization | `HAS_PERMISSION` | Permission | 1:N | Org ceiling |
| Agent | `HAS_CONSENT` | Consent | 1:N | Subordinate's consent records (one-time consent policy) |
| Consent | `SCOPED_TO` | Organization | N:1 | Org under whose policy the consent operates |

---

## Value Objects

> See brainstorm.md Section 2.4 for the full value object catalog (50+ types).

Value objects have no independent identity. They are embedded as properties within nodes. Key categories:
- **Message content:** Content, TurnId, ExtensionMessage
- **Metrics:** Usage, CostConfig
- **Enums:** StopReason, TurnTrigger, ContinuationKind, LoopStatus, ApiProtocol, ThinkingLevel, etc.
- **Compaction:** CompactionBlock, CompactedSection, TurnRange
- **Snapshots:** LoopConfigSnapshot, OpenAiCompat
- **Cross-session refs:** SpawnRef, ChildLoopRef, ParallelGroupRecord
- **MCP protocol:** McpToolInfo, ServerCapabilities, JsonRpc types
- **Config sections:** All `*Section` / `*Instance` types from config/schema.rs

---

## Schema Registry (Meta-Graph)

A self-describing layer: nodes that describe other nodes.

```
SchemaNode
  name: String          -- e.g. "Agent", "Session", "Turn"
  properties: Vec<PropertyDef>
  version: u32
  created_by: agent_id  -- which agent defined this (system or runtime)
  is_system: bool       -- true for built-in nodes, false for agent-created

SchemaEdge
  name: String          -- e.g. "RUNS_SESSION", "DELEGATES_TO"
  from_node: String     -- source node type
  to_node: String       -- target node type
  properties: Vec<PropertyDef>
  cardinality: String   -- "1:1", "1:N", "N:1", "N:N"

PropertyDef
  name: String
  type: String          -- "String", "u64", "DateTime", "Json", "Enum(...)", "Vec<T>"
  required: bool
  default: Option<Value>
  indexed: bool         -- whether this property should be queryable
```

Enables:
- **Introspection:** Agents can query "what node types exist?"
- **Runtime extension:** Agents with permission can define new node types
- **Validation:** Data writes checked against schema
- **Migration:** Schema versioning tracks evolution
