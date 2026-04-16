<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->

# Ontological Data Model

> Extracted from brainstorm.md Section 2 + Section 3.12, refined 2026-04-09.
> The canonical reference for baby-phi's node types, edge types, value objects, and schema registry.
> See also: [agent.md](agent.md) (extended agent model), [permissions.md](permissions/README.md) (capability model + Consent), [phi-core-mapping.md](phi-core-mapping.md) (full type mapping)

---

## Design Principle: Agent as the Core Node

The ontology radiates outward from **Agent**. Every entity exists because an Agent needs it. No orphan sessions, no floating configs — everything traces back to an Agent node.

This is a graph-first model (think ontology, not relational tables), even if the storage layer is initially flat files or SQLite. The relationships are first-class.

---

## Node Types (29 total)

### Core Identity

| Node | Identity | phi-core Source | Why it exists |
|------|----------|-----------------|---------------|
| **Agent** | `agent_id` | `BasicAgent` | The nucleus — everything radiates from here |
| **AgentProfile** | `profile_id` | `AgentProfile` | Blueprint: who the agent IS |
| **User** | `user_id` | *baby-phi concept* | Who owns/interacts with agents |
| **Identity** | `agent_id` | *baby-phi concept* | Emergent self of an LLM Agent — materialized node with `self_description` (NL), `lived`/`witnessed` (structs), and `embedding` (vector). Updated reactively on session end, memory extraction, skill change, rating. See [agent.md § Identity Node Content](agent.md#identity-node-content--provisional-direction). LLM Agents only; Human Agents have no Identity. |

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
| **Grant** | `grant_id` | *baby-phi concept* | Capability-based access control (5-tuple record held by a principal) |
| **AuthRequest** | `request_id` | *baby-phi concept* | First-class workflow composite (`auth_request_object`) that mediates Grant creation. Carries: `request_id`, `requestor`, `kinds`, `scope`, `state` (enum: Draft/Pending/In Progress/Approved/Denied/Partial/Expired/Revoked/Cancelled), `valid_until`, `submitted_at`, `resource_slots` (list of `{resource, approvers[{approver, state, responded_at}]}`), `routing_override`, `justification`, `audit_class`, retention fields (`active_window_days`, `archived`, `terminal_state_entered_at`). Full lifecycle in [permissions.md → Auth Request Lifecycle](permissions/02-auth-request.md#auth-request-lifecycle). |
| **Template** | `template_name` | *baby-phi concept* | A reusable permission pattern adopted at org level; adoption emits an Auth Request that serves as the provenance for all subsequent Grants the template fires. See [permissions.md → Standard Permission Templates](permissions/07-templates-and-tools.md#standard-permission-templates). |
| **ToolAuthorityManifest** | attached to `ToolDefinition.name` | *baby-phi concept* | Publish-time authority declaration for a tool. Carries `resource`, `actions`, `constraints`, `kind`, `target_kinds`, `delegable`, `approval`. See [permissions.md → Tool Authority Manifest](permissions/04-manifest-and-resolution.md#tool-authority-manifest-tool-requirements). |
| **Consent** | `consent_id` | *baby-phi concept* | Subordinate consent record gating Authority Template grants (see [permissions.md → Consent Policy](permissions/06-multi-scope-consent.md#consent-policy-organizational)) |
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

## Edge Types (54+ total)

### Agent-Centric (first-order)

| From | Edge | To | Cardinality | Meaning |
|------|------|----|-------------|---------|
| Agent | `HAS_PROFILE` | AgentProfile | 1:1 | Blueprint identity |
| Agent | `USES_MODEL` | ModelConfig | N:1 | Current LLM backing |
| Agent | `HAS_TOOL` | ToolDefinition | 1:N | Available tools |
| Agent | `HAS_SKILL` | Skill | 1:N | Loaded skills |
| Agent | `HOLDS_GRANT` | Grant | 1:N | Access control |
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
| Agent | `OWNED_BY` | User | N:1 | Who controls this agent. *(This is a special case of the generic `Resource ──OWNED_BY──▶ Principal` edge in Governance Wiring below: an Agent is itself an ownable resource, and a User is a Principal.)* |
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
| ToolDefinition | `HAS_MANIFEST` | ToolAuthorityManifest | 1:1 | Publish-time authority declaration (checked by the manifest validator and used at runtime for the Permission Check) |
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

> **Terminology.** A **Principal** is any entity that can hold authority: Agent, Project, Organization, or User. A **Resource** is any entity that can be owned: a filesystem object, a session, a memory, an agent (agents are both principals and resources), an external service, etc. These two type unions appear in the edge table below.

**Ownership edges** (Rust-style ownership model — see [permissions.md → Resource Ownership](permissions/01-resource-ontology.md#resource-ownership)):

| From | Edge | To | Cardinality | Meaning |
|------|------|----|-------------|---------|
| Resource | `OWNED_BY` | Principal | N:1 | Current owner. Generic form of the `Agent ──OWNED_BY──▶ User` edge above. |
| Principal | `CREATED` | Resource | 1:N | Creation provenance (the principal who created the resource) |
| Principal | `ALLOCATED_TO` | Principal | N:N | A has allocated a share of authority over some Resource to B. Edge properties: `resource_ref`, `scope` (action-vocabulary list, e.g. `[allocate]` or `[read, write]`), `provenance_auth_request`. Multiple `ALLOCATED_TO` edges with `scope: [allocate]` on the same resource is precisely how [co-ownership](permissions/01-resource-ontology.md#co-ownership-shared-resources) is represented. |

> **Transfer history** is not stored on a dedicated edge. Transfers are Auth Requests with `scope: [transfer]`; on approval, the `OWNED_BY` edge is rewritten. Ownership history for a resource is traversable via `AuthRequest ──REQUESTS_ON──▶ Resource` filtered by `scope: [transfer]` and ordered by `responded_at`.

**Grant + Auth Request edges** (authority-chain model — see [permissions.md → Authority Chain](permissions/04-manifest-and-resolution.md#the-authority-chain)):

| From | Edge | To | Cardinality | Meaning |
|------|------|----|-------------|---------|
| User | `ISSUED_GRANT` | Grant | 1:N | Authorization source (legacy provenance edge; kept for direct human-issued grants) |
| Grant | `DESCENDS_FROM` | AuthRequest | N:1 | Structural provenance — the Auth Request that produced this Grant. Replaces the earlier string-valued `provenance` field with a traversable edge. |
| Grant | `APPLIES_TO` | Agent | N:N | Which subjects the Grant covers |
| Agent | `HOLDS_GRANT` | Grant | 1:N | Agent-specific grants |
| Project | `HOLDS_GRANT` | Grant | 1:N | Project rules |
| Organization | `HOLDS_GRANT` | Grant | 1:N | Org ceiling |
| AuthRequest | `REQUESTS_ON` | Resource | N:N | Which resources this request targets (one per entry in `resource_slots`) |
| AuthRequest | `APPROVED_BY` | Principal | N:N | Per-slot approval edges. Edge properties: `resource_ref`, `state` (Unfilled/Approved/Denied), `responded_at`, `reconsidered_at`. |
| AuthRequest | `SUBMITTED_BY` | Principal | N:1 | Who created the request (the `requestor` field) |
| AuthRequest | `EMITTED_BY` | Template | N:1 | Present only on template-adopted requests; traces back to the template definition |

**Consent edges** (subordinate-consent policy — see [permissions.md → Consent Policy](permissions/06-multi-scope-consent.md#consent-policy-organizational)):

| From | Edge | To | Cardinality | Meaning |
|------|------|----|-------------|---------|
| Agent | `HAS_CONSENT` | Consent | 1:N | Subordinate's consent records (one-time consent policy) |
| Consent | `SCOPED_TO` | Organization | N:1 | Org under whose policy the consent operates |

---

## Value Objects

> See brainstorm.md Section 2.4 for the full value object catalog (50+ types).

Value objects have no independent identity. They are embedded as properties within nodes. Key categories:
- **Message content:** Content, TurnId, ExtensionMessage
- **Metrics:** Usage, CostConfig
- **Enums:** StopReason, TurnTrigger, ContinuationKind, LoopStatus, ApiProtocol, ThinkingLevel, `AuthRequestState` (Draft/Pending/In Progress/Approved/Denied/Partial/Expired/Revoked/Cancelled), `ResourceSlotState` (Unfilled/Approved/Denied), etc.
- **Compaction:** CompactionBlock, CompactedSection, TurnRange
- **Snapshots:** LoopConfigSnapshot, OpenAiCompat
- **Cross-session refs:** SpawnRef, ChildLoopRef, ParallelGroupRecord
- **MCP protocol:** McpToolInfo, ServerCapabilities, JsonRpc types
- **Config sections:** All `*Section` / `*Instance` types from config/schema.rs
- **Auth Request slot value objects** (embedded on AuthRequest nodes via `resource_slots`):
  - `ResourceSlot { resource: ResourceRef, approvers: Vec<ApproverSlot>, state: ResourceSlotState }` — one per resource in an Auth Request. Per-resource state is derived from its approvers' states (see [permissions/02 § Schema](permissions/02-auth-request.md#schema--per-resource-slot-model)).
  - `ApproverSlot { approver: principal_id, state: ApproverSlotState, responded_at: Option<DateTime>, reconsidered_at: Option<DateTime> }` — one per required approver inside a ResourceSlot.
  - `ResourceSlotState` enum: `In Progress | Approved | Denied | Partial | Expired`.
  - `ApproverSlotState` enum: `Unfilled | Approved | Denied`.
- **Ownership transfer (value object, embedded on Auth Requests with `scope: [transfer]`):** `TransferRecord { transfer_id, resource_id, from_principal, to_principal, timestamp, requestor, approver }`. Not an independent node — traversable via the owning Auth Request. Immutability inherits from the Auth Request's post-submission immutability.

---

## Tag Conventions

Tags are first-class on ownable composites. They power selectors in Grants and Auth Requests. Two reserved-namespace conventions:

- **Type-identity tags** — `#kind:{composite_name}` — auto-added at creation; declares which composite type the instance belongs to (e.g., `#kind:session`, `#kind:memory`, `#kind:auth_request`).
- **Instance-identity tags** — `{kind}:{instance_id}` — auto-added at creation; addresses this specific instance (e.g., `session:s-9831`, `memory:m-4581`, `auth_request:req-7102`).

Reserved tag namespaces (runtime-only; rejected at publish/creation time if written manually):

| Namespace | Owner | Purpose |
|-----------|-------|---------|
| `#kind:*` | runtime | Composite type identity |
| `{kind}:*` for any registered composite | runtime | Instance identity (self-tag on the composite instance) |
| `delegated_from:*` | runtime | Lineage tag on sessions (links a sub-session to its parent loop) |
| `derived_from:*` | runtime | Derivation tag (e.g., a Memory extracted from a Session carries `derived_from:session:{source_id}`) |

All other tag namespaces (`agent:*`, `project:*`, `org:*`, `task:*`, `role_at_creation:*`, `#public`, `#sensitive`, etc.) follow the lifecycle rules of their respective composites.

Full specification: [permissions.md → Instance Identity Tags](permissions/01-resource-ontology.md#instance-identity-tags-kindid).

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

### Who May Extend the Schema

Creating a new `SchemaNode` or `SchemaEdge` at runtime is a privileged operation. It requires **both**:

1. The acting agent holds `[allocate]` on `control_plane_object` — typically only platform admins and specifically-designated System Agents. This is the baseline "may touch meta-graph" capability.
2. A **schema-extension Auth Request** (Template E shape — see [permissions/07-templates-and-tools.md § Template E — Explicit Capability](permissions/07-templates-and-tools.md#opt-in-example-templates-c-d-and-e)) has been approved for the specific extension. The Auth Request's `justification` field carries the motivation; its approver is the org's platform admin. The approved request serves as the extension's provenance.

This mirrors template adoption: **one gate event authorises the extension; subsequent nodes or edges of the new type then follow the standard ownership model.** The first node of a newly-defined type is owned per the usual creation rules (most-specific scope wins — project > org > agent).

**Schema removal** follows the same pattern in reverse. Removing a `SchemaNode` or `SchemaEdge` requires a removal Auth Request plus a migration plan for existing instances. Per the v0 additive-only policy documented in [coordination.md § Design Decisions](coordination.md#design-decisions-v0-defaults-revisitable), destructive schema changes are rare and always audited.

**Why two gates rather than one.** `[allocate]` on `control_plane_object` is the *capability* to propose schema changes; the Auth Request is the *specific authorisation* for this particular change. Without the capability gate, any compromised agent with a stale Auth Request could keep extending; without the per-change Auth Request, any platform admin could silently extend the schema. Both gates together make schema extension both auditable and revocable.
