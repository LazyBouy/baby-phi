# baby-phi v0 — Brainstorm

> Living document. Captures ideas and design decisions from brainstorming sessions.
> Not a spec — a seedbed. Ideas here graduate into proper specs when they solidify.

---

## 1. Core Vision

baby-phi is an **agent management system** where many agents operate, perform different activities without stepping on each other's resources, yet can communicate and coordinate as required.

The foundation is a **data model layer** that translates phi-core structs into an interconnected, self-describing data fabric — queryable, introspectable, and extensible at runtime.

---

## 2. Ontological Data Model

### 2.1 Design Principle: Agent as the Core Node

The ontology radiates outward from **Agent**. Every entity exists because an Agent needs it. No orphan sessions, no floating configs — everything traces back to an Agent node.

This is a graph-first model (think ontology, not relational tables), even if the storage layer is initially flat files or SQLite. The relationships are first-class.

### 2.2 Node Types

#### Core Identity

| Node | Identity | Properties | phi-core Source | Why it exists |
|------|----------|------------|-----------------|---------------|
| **Agent** | `agent_id` | name, status, created_at | `BasicAgent` | The nucleus — everything radiates from here |
| **AgentProfile** | `profile_id` | name, description, system_prompt, thinking_level, temperature, max_tokens, config_id, workspace | `AgentProfile` | Blueprint: who the agent IS |
| **User** | `user_id` | name, role, created_at | *baby-phi concept* | Who owns/interacts with agents. External identity. |

#### Execution History

| Node | Identity | Properties | phi-core Source | Why it exists |
|------|----------|------------|-----------------|---------------|
| **Session** | `session_id` | scope, formation, created_at, last_active_at | `Session` | One logical task or conversation |
| **Loop** | `loop_id` | status, continuation_kind, usage, started_at, ended_at, rejection | `LoopRecord` | One `agent_loop()` / `agent_loop_continue()` call |
| **Turn** | `turn_id` (loop_id + index) | triggered_by, usage, started_at, ended_at | `Turn` | One LLM round-trip |
| **Message** | generated id | role, content, timestamp, model, provider, stop_reason | `Message` / `AgentMessage` / `LlmMessage` | Atomic unit of conversation |
| **Event** | `loop_id + sequence` | type, timestamp, payload | `AgentEvent` / `LoopEvent` | Granular execution trace (streaming, tool updates, etc.) |

> **Why Message is a separate node (not embedded in Turn):**
> Messages can exist without Turns — e.g., standalone provider testing where you send
> messages directly to a model without the agent loop. This decoupling also enables
> cross-agent message references and message-level queries independent of execution context.

> **Why Event is a separate node:**
> Events are the finest-grained execution record. They enable event-driven coordination
> between agents (Agent B subscribes to Agent A's ToolExecutionEnd events), observability
> dashboards, and replay/debugging. Events exist even when sessions aren't recorded.

#### Capability

| Node | Identity | Properties | phi-core Source | Why it exists |
|------|----------|------------|-----------------|---------------|
| **ModelConfig** | `id + provider` | name, api, base_url, reasoning, context_window, max_tokens, cost, headers, compat | `ModelConfig` | Which LLM backs the agent |
| **ToolDefinition** | `name` | description, parameters (JSON schema) | `ToolDefinition` | Tool schema sent to LLM |
| **ToolImplementation** | `name` | impl_type (builtin/mcp/openapi/custom), workspace | `BashTool`, `ReadFileTool`, `WriteFileTool`, `EditFileTool`, `SearchTool`, `ListFilesTool`, `PrunTool`, `SubAgentTool`, `McpToolAdapter`, `OpenApiToolAdapter` | Concrete tool execution logic |
| **Skill** | `name` | description, source, file_path, base_dir | `Skill` | Loaded skill with metadata |
| **McpServer** | `name` | transport (stdio/http), command/url, server_info, capabilities | `McpClient`, `ServerInfo`, `ServerCapabilities` | External tool server via MCP protocol |
| **OpenApiSpec** | `spec_id` | source (file/url/inline), base_url, auth, filter | `OpenApiConfig`, `OpenApiToolAdapter` | External API spec that generates tools |
| **SystemPrompt** | `id` | blocks, strategy_type | `SystemPrompt`, `AgentPromptStrategy`, `CustomPromptStrategy`, `MinimalPromptStrategy` | Assembled system prompt for the agent |
| **EvaluationStrategy** | `name` | type (transparent/pick_first/token_efficient/elaborate/llm_judge), judge_config | `TransparentEvaluation`, `PickFirstEvaluation`, `TokenEfficientEvaluation`, `ElaborateEvaluation`, `LlmJudgeEvaluation` | How parallel branches are evaluated |

#### Governance

| Node | Identity | Properties | phi-core Source | Why it exists |
|------|----------|------------|-----------------|---------------|
| **ExecutionLimits** | generated | max_turns, max_tokens, max_duration_secs, max_cost | `ExecutionLimits` | Constrains what an agent can consume |
| **Permission** | generated | scope, actions, target_pattern, granted_by, expires_at | *baby-phi concept* | Access control for agents |
| **CompactionPolicy** | generated | compact_at_pct, budget_threshold_pct, scope, keep_first_turns, keep_recent_turns, max_summary_tokens, tool_output_max_lines, focus_message | `CompactionConfig`, `CompactionScope` | How context is managed when it grows |
| **RetryPolicy** | generated | max_retries, initial_delay_ms, max_delay_ms, jitter_pct | `RetryConfig` | How provider errors are retried |
| **CachePolicy** | generated | enabled, strategy (auto/disabled/manual), cache_system, cache_tools, cache_messages | `CacheConfig`, `CacheStrategy` | Prompt caching behavior |

#### Memory & Knowledge (baby-phi concepts)

| Node | Identity | Properties | phi-core Source | Why it exists |
|------|----------|------------|-----------------|---------------|
| **Memory** | generated | type (user/feedback/project/reference), content, created_at, updated_at, source_agent_id | *baby-phi concept* | Persistent knowledge across sessions |

#### Configuration (structural)

| Node | Identity | Properties | phi-core Source | Why it exists |
|------|----------|------------|-----------------|---------------|
| **AgentConfig** | `config_name` | format (toml/json/yaml), source_path | `AgentConfig` | Root configuration document |
| **PromptBlock** | `name` | title, content (file/inline), order, token_budget | `PromptBlockDef`, `StrategyBlockSection` | One block within a system prompt strategy |

---

### 2.3 Edge Types (Relationships)

#### Agent-centric (first-order — radiating from Agent)

| From | Edge | To | Cardinality | Properties | Meaning |
|------|------|----|-------------|------------|---------|
| Agent | `HAS_PROFILE` | AgentProfile | 1:1 | — | Blueprint identity |
| Agent | `USES_MODEL` | ModelConfig | N:1 | active: bool, since: DateTime | Current LLM backing |
| Agent | `HAS_TOOL` | ToolDefinition | 1:N | enabled: bool | Available tool schemas |
| Agent | `HAS_SKILL` | Skill | 1:N | — | Loaded skills |
| Agent | `GOVERNED_BY` | ExecutionLimits | 1:1 | — | Execution constraints |
| Agent | `HAS_PERMISSION` | Permission | 1:N | — | What this agent is allowed to do |
| Agent | `USES_COMPACTION` | CompactionPolicy | 1:1 | — | Context management strategy |
| Agent | `USES_RETRY` | RetryPolicy | 1:1 | — | Error retry behavior |
| Agent | `USES_CACHE` | CachePolicy | 1:1 | — | Prompt caching behavior |
| Agent | `USES_EVALUATION` | EvaluationStrategy | 1:1 | — | Parallel branch selection |
| Agent | `HAS_SYSTEM_PROMPT` | SystemPrompt | 1:1 | — | Assembled system prompt |
| Agent | `CONNECTS_TO` | McpServer | 1:N | status: connected/disconnected | External tool server |
| Agent | `CONNECTS_TO` | OpenApiSpec | 1:N | — | External API spec |
| Agent | `RUNS_SESSION` | Session | 1:N | — | Execution history |
| Agent | `DELEGATES_TO` | Agent | N:N | tool_name, tool_call_id, loop_id | Sub-agent spawning |
| Agent | `OWNED_BY` | User | N:1 | role: owner/operator | Who controls this agent |
| Agent | `HAS_MEMORY` | Memory | 1:N | — | Persistent knowledge |
| Agent | `LOADED_FROM` | AgentConfig | N:1 | — | Config that created this agent |

#### Execution chain (second-order — within execution history)

| From | Edge | To | Cardinality | Properties | Meaning |
|------|------|----|-------------|------------|---------|
| Session | `CONTAINS_LOOP` | Loop | 1:N | order: u32 | Ordered loop sequence |
| Loop | `CONTINUES_FROM` | Loop | N:1 | kind: ContinuationKind | Continuation/retry/branch chain |
| Loop | `CONTAINS_TURN` | Turn | 1:N | index: u32 | Turns within a loop |
| Loop | `CONFIGURED_WITH` | LoopConfigSnapshot | 1:1 | — | Config snapshot for this loop |
| Loop | `EMITS` | Event | 1:N | sequence: u64 | Ordered event stream |
| Turn | `PRODUCES` | Message | 1:N | role: input/output/tool_result | Messages in a turn |
| Turn | `EXECUTES_TOOL` | ToolDefinition | 1:N | call_id, args, result, is_error | Tool calls within a turn |

#### Cross-agent (third-order — multi-agent coordination)

| From | Edge | To | Cardinality | Properties | Meaning |
|------|------|----|-------------|------------|---------|
| Session | `SPAWNED_FROM` | Session | N:1 | parent_loop_id, tool_call_id, tool_name | Sub-agent session origin |
| Loop | `SPAWNED_CHILD` | Loop | 1:N | child_session_id, tool_call_id, tool_name | Parent loop → child sub-agent loop |
| Loop | `PARALLEL_WITH` | Loop | N:N | group_id, selected: bool, selected_config_index | Evaluational parallelism siblings |

#### Capability wiring (how tools are provided)

| From | Edge | To | Cardinality | Properties | Meaning |
|------|------|----|-------------|------------|---------|
| ToolDefinition | `IMPLEMENTED_BY` | ToolImplementation | 1:1 | — | Schema → concrete impl |
| McpServer | `PROVIDES_TOOL` | ToolDefinition | 1:N | — | MCP server advertised tools |
| OpenApiSpec | `PROVIDES_TOOL` | ToolDefinition | 1:N | operation_id, method, path | OpenAPI operations as tools |
| SystemPrompt | `CONTAINS_BLOCK` | PromptBlock | 1:N | order: u32 | Ordered prompt blocks |

#### Governance wiring

| From | Edge | To | Cardinality | Properties | Meaning |
|------|------|----|-------------|------------|---------|
| User | `GRANTS_PERMISSION` | Permission | 1:N | granted_at: DateTime | Who created the permission |
| Permission | `APPLIES_TO` | Agent | N:N | — | Which agents this permission governs |

---

### 2.4 Value Objects (embedded as properties, NOT nodes)

These have no independent identity. They are always embedded within a node as properties or nested JSON.

#### Message Content & Structure

| Value Object | Embedded In | phi-core Source | Purpose |
|--------------|-------------|-----------------|---------|
| `Content` | Message | `Content` enum (Text, Image, Thinking, ToolCall) | Atomic content units within a message |
| `TurnId` | Message, Turn, Event | `TurnId` { loop_id, turn_index } | Composite turn identifier |
| `ExtensionMessage` | Message | `ExtensionMessage` { role, kind, data } | Non-LLM messages (UI, debug) |

#### Token & Cost Metrics

| Value Object | Embedded In | phi-core Source | Purpose |
|--------------|-------------|-----------------|---------|
| `Usage` | Loop, Turn, Event | `Usage` { input, output, reasoning, cache_read, cache_write, total_tokens } | Token metrics |
| `CostConfig` | ModelConfig | `CostConfig` { input_per_million, output_per_million, cache_read_per_million, cache_write_per_million } | Token pricing |

#### Enums (property types on nodes)

| Value Object | Used On | phi-core Source | Values |
|--------------|---------|-----------------|--------|
| `StopReason` | Message | `StopReason` | Stop, Length, ToolUse, Error, Aborted, MaxTurns, UserStop, Handoff, GuardRail, ContextCompacted, Paused |
| `TurnTrigger` | Turn | `TurnTrigger` | User, SubAgent, Continuation, Branch |
| `ContinuationKind` | Loop | `ContinuationKind` | Initial, Default, Rerun { tag }, Branch { tag }, Compaction |
| `LoopStatus` | Loop | `LoopStatus` | Pending, Running, Completed, Rejected, Aborted |
| `SessionFormation` | Session | `SessionFormation` | Explicit, FirstLoop, InactivityTimeout |
| `SessionScope` | Session | `SessionScope` | Ephemeral, Persistent |
| `ApiProtocol` | ModelConfig | `ApiProtocol` | AnthropicMessages, OpenAiCompletions, OpenAiResponses, AzureOpenAiResponses, GoogleGenerativeAi, GoogleVertex, BedrockConverseStream |
| `ThinkingLevel` | AgentProfile, Loop | `ThinkingLevel` | Off, Minimal, Low, Medium, High |
| `ToolExecutionStrategy` | Agent | `ToolExecutionStrategy` | Sequential, Parallel, Batched { size } |
| `QueueMode` | Agent | `QueueMode` | OneAtATime, All |
| `CompactionScope` | CompactionPolicy | `CompactionScope` | FixedCount(usize), TokenBudget |
| `FilterResult` | Event | `FilterResult` | Pass, Warn(String), Reject(String) |
| `ConfigFormat` | AgentConfig | `ConfigFormat` | Toml, Json, Yaml |
| `OpenApiAuth` | OpenApiSpec | `OpenApiAuth` | None, ApiKey, OAuth2 |
| `OperationFilter` | OpenApiSpec | `OperationFilter` | All, Include, Exclude |
| `MaxTokensField` | ModelConfig.compat | `MaxTokensField` | MaxTokens, MaxCompletionTokens |
| `ThinkingFormat` | ModelConfig.compat | `ThinkingFormat` | Default, OpenAi, Xai, Qwen |

#### Compaction Overlays

| Value Object | Embedded In | phi-core Source | Purpose |
|--------------|-------------|-----------------|---------|
| `CompactionBlock` | Loop | `CompactionBlock` | Non-destructive compaction overlay { keep_first, keep_recent, keep_compacted } |
| `CompactedSection` | CompactionBlock | `CompactedSection` | Turn range + replacement messages |
| `TurnRange` | CompactionBlock | `TurnRange` | { start_turn, end_turn } inclusive bounds |

#### Execution Snapshots

| Value Object | Embedded In | phi-core Source | Purpose |
|--------------|-------------|-----------------|---------|
| `LoopConfigSnapshot` | Loop | `LoopConfigSnapshot` | Denormalized config snapshot { model, provider, config_id, name, api, base_url, reasoning, context_window, max_tokens, thinking_level, temperature } |
| `OpenAiCompat` | ModelConfig | `OpenAiCompat` | Per-provider quirk flags (15+ boolean/enum fields) |

#### Cross-Session References

| Value Object | Embedded In | phi-core Source | Purpose |
|--------------|-------------|-----------------|---------|
| `SpawnRef` | Session | `SpawnRef` | Inbound link: { parent_session_id, parent_loop_id, tool_call_id, tool_name } |
| `ChildLoopRef` | Loop | `ChildLoopRef` | Outbound link: { tool_call_id, tool_name, child_loop_id, child_session_id } |
| `ParallelGroupRecord` | Loop | `ParallelGroupRecord` | { all_loop_ids, selected_loop_id, selected_config_index, evaluation_usage, is_selected } |

#### Pruning Records

| Value Object | Embedded In | phi-core Source | Purpose |
|--------------|-------------|-----------------|---------|
| `PrunRecord` | Loop or Turn | `PrunRecord` | { timestamp, tokens_removed, summary_text } |
| `PrunVariant` | PrunRecord | `PrunVariant` | Silent, Memo |

#### Event Payloads (variant-specific data within Event node)

| Value Object | Embedded In | phi-core Source | Purpose |
|--------------|-------------|-----------------|---------|
| `StreamDelta` | Event (MessageUpdate) | `StreamDelta` | { Text, Thinking, ToolCallStart, ToolCallDelta } |
| `ToolResult` | Event (ToolExecutionEnd) | `ToolResult` | { content, details, child_loop_id } |

#### MCP Protocol (embedded within McpServer interactions)

| Value Object | Embedded In | phi-core Source | Purpose |
|--------------|-------------|-----------------|---------|
| `McpToolInfo` | McpServer (via PROVIDES_TOOL) | `McpToolInfo` | { name, description, inputSchema } |
| `ServerCapabilities` | McpServer | `ServerCapabilities` | { tools, resources, prompts } |
| `ServerInfo` | McpServer | `ServerInfo` | { name, version } |
| `ClientInfo` | McpServer (handshake) | `ClientInfo` | { name, version } |
| `McpContent` | ToolResult (from MCP) | `McpContent` | Text, Image |
| `JsonRpcRequest` | McpServer (transport) | `JsonRpcRequest` | { id, method, params } |
| `JsonRpcResponse` | McpServer (transport) | `JsonRpcResponse` | { id, result } |
| `JsonRpcError` | McpServer (transport) | `JsonRpcError` | { code, message, data } |

#### Config Sections (structural — map to node properties, not independent nodes)

| Value Object | Maps To | phi-core Source | Purpose |
|--------------|---------|-----------------|---------|
| `AgentSection` | Agent properties | `AgentSection` | Top-level agent identity in config |
| `ProfileSection` | AgentProfile collection | `ProfileSection` | Profile definitions in config |
| `ProfileInstanceSection` | AgentProfile properties | `ProfileInstanceSection` | Individual profile variant |
| `AgentInstanceSection` | Agent properties | `AgentInstanceSection` | Agent instance binding with profile ref |
| `ProviderSection` | ModelConfig properties | `ProviderSection` | Provider config (api_key, base_url, etc.) |
| `ProviderInstance` | ModelConfig properties | `ProviderInstance` | Named provider variant |
| `CostSection` | CostConfig (on ModelConfig) | `CostSection` | Token pricing in config |
| `CompatSection` | OpenAiCompat (on ModelConfig) | `CompatSection` | Compatibility flags in config |
| `SessionSection` | Session properties | `SessionSection` | Session scope in config |
| `ToolsSection` | Agent→ToolDefinition edges | `ToolsSection` | Tool list in config |
| `ToolInstance` | ToolDefinition properties | `ToolInstance` | Named tool instance in config |
| `SkillsSection` | Agent→Skill edges | `SkillsSection` | Skill directories in config |
| `SubAgentsSection` | Agent→Agent DELEGATES_TO | `SubAgentsSection` | Sub-agent definitions in config |
| `SubAgentInstance` | Agent properties (child) | `SubAgentInstance` | Named sub-agent instance |
| `CallbacksSection` | Agent lifecycle hooks | `CallbacksSection` | Callback scripts (Phase 2 WASM) |
| `HooksSection` | Agent lifecycle hooks | `HooksSection` | Hook references (Phase 2 WASM) |
| `CompactionSection` | CompactionPolicy properties | `CompactionSection` | Compaction config in config |
| `CompactionInstanceSection` | CompactionPolicy properties | `CompactionInstanceSection` | Named compaction variant |
| `ExecutionSection` | ExecutionLimits properties | `ExecutionSection` | Execution bounds in config |
| `RetrySection` | RetryPolicy properties | `RetrySection` | Retry config in config |
| `CacheSection` | CachePolicy properties | `CacheSection` | Cache config in config |
| `SystemPromptStrategySection` | SystemPrompt properties | `SystemPromptStrategySection` | Prompt strategy in config |
| `StrategyInstanceSection` | SystemPrompt properties | `StrategyInstanceSection` | Named strategy variant |
| `StrategyBlockSection` | PromptBlock properties | `StrategyBlockSection` | Block definition in config |
| `SystemPromptSection` | SystemPrompt collection | `SystemPromptSection` | Prompt instances in config |
| `ConfigRef` | edge properties | `ConfigRef` | Reference protocol: `{{...}}`, `{{%...%}}`, `{{#...#}}` |

### 2.5 Runtime-Only Types (NOT persisted in the data model)

These exist only in memory during agent execution. They are ephemeral process state, not data model entities.

#### Why exclude: These are implementation machinery, not domain entities. They have no identity, no persistence need, and no cross-agent relevance.

| Type | phi-core Source | Why runtime-only |
|------|-----------------|------------------|
| `AgentContext` | `AgentContext` | In-memory accumulator during loop execution; reconstructed from Session/Loop/Messages |
| `InRunEntry` | `InRunEntry` | Working context entries; ephemeral within a loop |
| `AgentLoopConfig` | `AgentLoopConfig` | Contains non-serializable hooks/closures; rebuilt from Agent + policies each loop |
| `StreamConfig` | `StreamConfig` | Envelope for one provider call; lives for one streaming request |
| `ExecutionTracker` | `ExecutionTracker` | Tracks progress against ExecutionLimits during a loop; not persisted |
| `ContextTracker` | `ContextTracker` | Tracks token counts during execution; ephemeral |
| `TurnMap` | `TurnMap` | Internal index for compaction; rebuilt on demand |
| `ProviderRegistry` | `ProviderRegistry` | Dispatches ApiProtocol → provider impl; process-level singleton |
| `ToolRegistry` | `ToolRegistry` | In-memory collection of loaded tools; rebuilt from config |
| `SkillSet` | `SkillSet` | In-memory collection of loaded skills; rebuilt from config |
| `SessionRecorder` | `SessionRecorder` | Materializes events into Session structure; process-level state machine |
| `SessionRecorderConfig` | `SessionRecorderConfig` | Config for the recorder; operational, not domain |

#### Provider Implementations (process-level, stateless)

| Type | phi-core Source | Why runtime-only |
|------|-----------------|------------------|
| `AnthropicProvider` | `AnthropicProvider` | Stateless HTTP client for Anthropic API |
| `OpenAiCompatProvider` | `OpenAiCompatProvider` | Stateless HTTP client for OpenAI-compat APIs |
| `OpenAiResponsesProvider` | `OpenAiResponsesProvider` | Stateless HTTP client for OpenAI Responses API |
| `AzureOpenAiProvider` | `AzureOpenAiProvider` | Stateless HTTP client for Azure OpenAI |
| `GoogleProvider` | `GoogleProvider` | Stateless HTTP client for Google Gemini |
| `GoogleVertexProvider` | `GoogleVertexProvider` | Stateless HTTP client for Google Vertex AI |
| `BedrockProvider` | `BedrockProvider` | Stateless HTTP client for Amazon Bedrock |
| `MockProvider` | `MockProvider` | Test-only provider |
| `MockResponse` | `MockResponse` | Test-only response type |
| `MockToolCall` | `MockToolCall` | Test-only tool call type |
| `DefaultContextTranslation` | `DefaultContextTranslation` | Stateless translation strategy |
| `SseEvent` | `SseEvent` | SSE parsing helper; wire format, not domain |

#### Streaming Events (wire-level, within Event node payload)

| Type | phi-core Source | Why runtime-only |
|------|-----------------|------------------|
| `StreamEvent` | `StreamEvent` | Provider-level streaming events; mapped to AgentEvent before recording |
| `EvaluationDecision` | `EvaluationDecision` | Transient evaluation result; captured as ParallelGroupRecord |
| `ParallelLoopOutcome` | `ParallelLoopOutcome` | Transient branch result; captured in Loop nodes |
| `ParallelLoopResult` | `ParallelLoopResult` | Transient overall result; captured in Loop nodes |
| `ScriptCallback` | `ScriptCallback` | Phase 2 WASM callback execution; runtime behavior, not data |

#### Tool Implementations (stateless executors)

| Type | phi-core Source | Why runtime-only |
|------|-----------------|------------------|
| `BashTool` | `BashTool` | Executor — captured as ToolImplementation node metadata |
| `ReadFileTool` | `ReadFileTool` | Executor |
| `WriteFileTool` | `WriteFileTool` | Executor |
| `EditFileTool` | `EditFileTool` | Executor |
| `SearchTool` | `SearchTool` | Executor |
| `ListFilesTool` | `ListFilesTool` | Executor |
| `PrunTool` | `PrunTool` | Executor |
| `SubAgentTool` | `SubAgentTool` | Executor — also creates DELEGATES_TO edges |
| `McpToolAdapter` | `McpToolAdapter` | Executor bridge to MCP |
| `OpenApiToolAdapter` | `OpenApiToolAdapter` | Executor bridge to OpenAPI |
| `DefaultCompaction` | `DefaultCompaction` | Strategy impl |
| `DefaultBlockCompaction` | `DefaultBlockCompaction` | Strategy impl |

#### Error Types (not persisted — logged or returned)

| Type | phi-core Source | Why runtime-only |
|------|-----------------|------------------|
| `ProviderError` | `ProviderError` | Error taxonomy for provider calls |
| `ToolError` | `ToolError` | Error taxonomy for tool execution |
| `McpError` | `McpError` | Error taxonomy for MCP protocol |
| `OpenApiError` | `OpenApiError` | Error taxonomy for OpenAPI integration |
| `SkillError` | `SkillError` | Error taxonomy for skill loading |
| `SessionError` | `SessionError` | Error taxonomy for session persistence |
| `ConfigError` | `ConfigError` | Error taxonomy for config parsing |
| `ScriptCallbackError` | `ScriptCallbackError` | Error taxonomy for script callbacks |

> **Design note:** Error types may become relevant if we add an **ErrorLog** node for debugging. For now, errors are transient — they surface in Event payloads (AgentEvent variants carry error info) or in Loop.rejection.

#### Context Types (transient execution state)

| Type | phi-core Source | Why runtime-only |
|------|-----------------|------------------|
| `ToolContext` | `ToolContext` | Per-invocation context for tool execution (cancel token, callbacks) |
| `PrunRequest` | `PrunRequest` | Transient request to prune context |
| `ContextConfig` | `ContextConfig` | Combines model limits + compaction policy; rebuilt from nodes each loop |

---

### 2.6 Schema Registry (Meta-Graph)

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

This enables:
- **Introspection:** Agents can query "what node types exist? what edges connect them?"
- **Runtime extension:** Agents with permission can define NEW node types (see Permissions below)
- **Validation:** Data writes are checked against the schema before persisting
- **Migration:** Schema versioning tracks how node types evolve over time

---

### 2.7 Ontology Summary

```
                                    ┌─────────┐
                      OWNED_BY      │  User    │    GRANTS_PERMISSION
                  ┌─────────────────│         │──────────────────┐
                  │                 └─────────┘                  │
                  ▼                                              ▼
            ┌──────────┐                                  ┌────────────┐
  ┌─────────│  Agent   │──────────────┐                   │ Permission │
  │         │          │              │                   └────────────┘
  │         └──┬───┬───┘              │
  │   HAS_     │   │  USES_          │ DELEGATES_TO
  │  PROFILE   │   │  MODEL          │
  ▼            │   ▼                  ▼
AgentProfile   │  ModelConfig       Agent (child)
               │
    ┌──────────┼──────────┬──────────────────┐
    │          │          │                  │
  HAS_TOOL  RUNS_    CONNECTS_TO     HAS_MEMORY
    │       SESSION      │                  │
    ▼          │    ┌────┴────┐              ▼
ToolDef        │    │         │           Memory
    │          │  McpServer  OpenApiSpec
    │          │    │         │
IMPLEMENTED_BY │  PROVIDES  PROVIDES
    │          │  _TOOL     _TOOL
    ▼          │    │         │
ToolImpl       │    └────┬────┘
               │         │
               ▼         ▼
           Session ── ToolDef
               │
          CONTAINS_LOOP
               │
               ▼
             Loop ──CONFIGURED_WITH──▶ LoopConfigSnapshot
               │
          CONTAINS_TURN
               │
               ▼
             Turn ──EXECUTES_TOOL──▶ ToolDef
               │
           PRODUCES
               │
               ▼
           Message
```

**Node count:** 20 node types
**Edge count:** 27 edge types
**Value object count:** 50+ value types
**Runtime-only count:** 45+ types (not persisted)

---

## 3. Permissions Model (Emerging)

> Triggered by: "Should agents be able to define new node types at runtime? — Yes, in some cases"

### 3.1 The Problem

In a multi-agent system, not all agents should have equal power:
- A coding agent should read/write files but not create new node types
- An orchestrator agent should spawn sub-agents but not access another agent's session data
- A schema-admin agent might define new node types but not execute tools

### 3.2 Permission Dimensions

| Dimension | Controls | Examples |
|-----------|----------|---------|
| **Data access** | Which nodes/edges an agent can read/write | "Agent X can read its own sessions but not Agent Y's" |
| **Schema mutation** | Whether an agent can define new node/edge types | "Only orchestrator can create new node types" |
| **Agent spawning** | Whether an agent can DELEGATE_TO other agents | "Worker agents cannot spawn sub-agents" |
| **Resource limits** | Execution constraints per agent | "Max 10 loops per session, max $0.50 per session" |
| **Tool access** | Which tools an agent can use | "Read-only agents get file_read but not bash" |
| **MCP access** | Which MCP servers an agent can connect to | "Agent X can use the GitHub MCP but not the database MCP" |
| **Memory access** | Which memories an agent can read/write | "Agent X can read shared project memories but only write its own" |

### 3.3 phi-core Extension Points for Permissions

From phi-core's `External — Not Core` section (docs/specs/overview.md):

| phi-core hook | Permission use |
|---------------|---------------|
| `InputFilter` | Gate what messages reach the agent (data access control) |
| `BeforeToolExecutionFn` | Gate which tools an agent can invoke (tool access) |
| `ExecutionLimits` | Resource constraints (already built-in) |
| `BeforeLoopFn` | Gate whether a loop can start (agent spawning control) |

Permissions are a baby-phi concern, not phi-core. phi-core provides the hooks; baby-phi provides the policy engine.

### 3.4 Permission as a Graph Edge

```
User ──GRANTS_PERMISSION──▶ Permission ──APPLIES_TO──▶ Agent
                                │
                          scope: "data_access"
                          actions: ["read", "write"]
                          target_pattern: "Session:own"
                          expires_at: null
```

This means permissions are queryable: "what can Agent X do?" is a graph traversal.

### 3.5 Open Questions

- [ ] Is permission per-agent-instance or per-agent-profile?
- [ ] Can permissions be dynamic (change during a session)?
- [ ] Who grants permissions — a system agent? config files? both?
- [ ] How do permissions compose when Agent A delegates to Agent B?
- [ ] Should there be a "superadmin" agent that bypasses permissions?
- [ ] How do permissions interact with MCP server capabilities?

---

## 4. Future Scenarios (from phi-core roadmap)

These phi-core future scenarios directly feed into baby-phi's design:

### 4.1 HITL Resume (Human-in-the-Loop)

Agent is aborted mid-execution, human reviews, then resumes. Requires checkpoint/restore on Agent state. phi-core needs `Agent::checkpoint()` / `Agent::restore(checkpoint)`.

**baby-phi implication:** The data model must support partial sessions — loops that are `Aborted` with a resumption path. The graph edge `CONTINUES_FROM` with `ContinuationKind::Rerun` or `Branch` captures this.

### 4.2 Checkpoint Restore (Cross-Process)

Serialize agent state to storage, load it in a different process. phi-core needs `AgentSnapshot` type.

**baby-phi implication:** The data layer IS the persistence. If all state is in the graph, checkpoint/restore is just "read the graph" / "write the graph". No separate snapshot mechanism needed.

### 4.3 Parallel Exploration

Multiple branches from the same checkpoint run concurrently. phi-core supports this via `agent_loop_continue(Branch)` with cloned contexts.

**baby-phi implication:** The `Loop` node naturally supports this — multiple Loops share the same `CONTINUES_FROM` parent, each as a sibling branch. `ParallelGroupRecord` (value object on Loop) tracks which branch was selected. The `PARALLEL_WITH` edge connects siblings.

### 4.4 Auto Origin/Continue Selection

Agent decides whether to `agent_loop` or `agent_loop_continue` based on context state.

**baby-phi implication:** This is the "agent invocation layer" — baby-phi should provide a high-level `send(agent_id, message)` that inspects the agent's current state in the data model and dispatches correctly.

---

## 5. Multi-Agent Coordination Patterns (To Explore)

> Not yet designed — placeholders for future brainstorming.

### 5.1 Shared Data (Blackboard)

Agents coordinate by reading/writing shared nodes in the graph. No direct messaging — just data. Like a blackboard architecture. The Memory node type enables this.

### 5.2 Event-Driven

Agents subscribe to events on specific nodes. "When Agent A creates a Message with tool_call X, notify Agent B." Built on top of `AgentEvent` streams and the Event node.

### 5.3 Explicit Messaging

Agents send messages to each other through a dedicated channel. Could use a shared Session or a new `COMMUNICATES_WITH` edge with a message queue.

### 5.4 Orchestrator Pattern

A supervisor agent that spawns, monitors, and coordinates worker agents. Maps directly to `DELEGATES_TO` edges. The orchestrator has `HAS_PERMISSION` to spawn and monitor.

---

## 6. Open Design Questions

- [ ] **Storage backend:** Start with JSON files (like phi-core sessions)? SQLite? In-memory graph?
- [ ] **Query language:** How do agents (and the system) query the graph? Custom DSL? SQL? Cypher-like?
- [ ] **Schema versioning:** When a node type evolves, how are old instances migrated?
- [ ] **Event sourcing:** Should the data model be event-sourced (append-only log of mutations) or state-based (mutable current state)?
- [ ] **Consistency model:** When two agents write to the same node concurrently, who wins?
- [ ] **Memory types:** What categories of memory exist? (user, feedback, project, reference — borrowing from Claude Code's memory model)
- [ ] **MCP lifecycle:** When an McpServer node is created, does the connection happen eagerly or lazily?
- [ ] **Provider testing:** How does standalone Message testing (without Agent/Session) fit the ontology? Is it a "system agent" or a special path?

---

## Appendix A: phi-core Struct → Ontology Mapping (Complete)

Every public phi-core struct/enum mapped to its ontology classification.

| phi-core Type | Ontology Classification | Maps To |
|--------------|------------------------|---------|
| **types/content.rs** | | |
| `Content` | Value Object | Property of Message |
| `Message` | **Node** | **Message** |
| `StopReason` | Value Object (enum) | Property of Message |
| **types/agent_message.rs** | | |
| `TurnId` | Value Object | Composite key of Turn |
| `LlmMessage` | Value Object | Message + TurnId wrapper |
| `AgentMessage` | Value Object | Llm/Extension routing envelope |
| **types/extension.rs** | | |
| `ExtensionMessage` | Value Object | Non-LLM message variant |
| **types/usage.rs** | | |
| `Usage` | Value Object | Property of Loop, Turn |
| `CacheConfig` | **Node** | **CachePolicy** |
| `CacheStrategy` | Value Object (enum) | Property of CachePolicy |
| `ThinkingLevel` | Value Object (enum) | Property of AgentProfile, Loop |
| **types/tool.rs** | | |
| `ToolExecutionStrategy` | Value Object (enum) | Property of Agent |
| `ToolResult` | Value Object | Property of Event (ToolExecutionEnd) |
| `ToolError` | Runtime-only | Error taxonomy |
| `ToolContext` | Runtime-only | Per-invocation execution context |
| **types/event.rs** | | |
| `ContinuationKind` | Value Object (enum) | Property of Loop |
| `TurnTrigger` | Value Object (enum) | Property of Turn |
| `AgentEvent` | **Node** | **Event** |
| `StreamDelta` | Value Object | Event payload for MessageUpdate |
| **types/context.rs** | | |
| `AgentContext` | Runtime-only | In-memory accumulator |
| `InRunEntry` | Runtime-only | Working context entries |
| **types/parallel.rs** | | |
| `FilterResult` | Value Object (enum) | Event payload for InputRejected |
| `EvaluationDecision` | Runtime-only | Transient evaluation output |
| `ParallelLoopOutcome` | Runtime-only | Transient branch result |
| `ParallelLoopResult` | Runtime-only | Transient overall result |
| **session/model.rs** | | |
| `SessionFormation` | Value Object (enum) | Property of Session |
| `LoopStatus` | Value Object (enum) | Property of Loop |
| `LoopConfigSnapshot` | Value Object | CONFIGURED_WITH edge target (denormalized) |
| `ChildLoopRef` | Value Object | SPAWNED_CHILD edge properties |
| `SpawnRef` | Value Object | SPAWNED_FROM edge properties |
| `ParallelGroupRecord` | Value Object | PARALLEL_WITH edge properties |
| `LoopEvent` | Value Object | Event wrapper with sequence |
| `Turn` | **Node** | **Turn** |
| `LoopRecord` | **Node** | **Loop** |
| `SessionScope` | Value Object (enum) | Property of Session |
| `Session` | **Node** | **Session** |
| `SessionError` | Runtime-only | Error taxonomy |
| **session/recorder.rs** | | |
| `SessionRecorderConfig` | Runtime-only | Recorder configuration |
| `SessionRecorder` | Runtime-only | Event → Session materializer |
| **provider/model.rs** | | |
| `ApiProtocol` | Value Object (enum) | Property of ModelConfig |
| `CostConfig` | Value Object | Property of ModelConfig |
| `MaxTokensField` | Value Object (enum) | Property of ModelConfig.compat |
| `ThinkingFormat` | Value Object (enum) | Property of ModelConfig.compat |
| `OpenAiCompat` | Value Object | Property of ModelConfig |
| `ModelConfig` | **Node** | **ModelConfig** |
| **provider/traits.rs** | | |
| `StreamEvent` | Runtime-only | Wire-level streaming event |
| `StreamConfig` | Runtime-only | Per-call provider envelope |
| `ToolDefinition` | **Node** | **ToolDefinition** |
| `ProviderError` | Runtime-only | Error taxonomy |
| **provider/retry.rs** | | |
| `RetryConfig` | **Node** | **RetryPolicy** |
| **provider/registry.rs** | | |
| `ProviderRegistry` | Runtime-only | Process-level singleton |
| **provider/sse.rs** | | |
| `SseEvent` | Runtime-only | SSE wire format |
| **provider/*.rs** (7 providers) | | |
| `AnthropicProvider` ... `BedrockProvider` | Runtime-only | Stateless HTTP clients |
| **provider/mock.rs** | | |
| `MockProvider`, `MockResponse`, `MockToolCall` | Runtime-only | Test-only types |
| **provider/context_translation.rs** | | |
| `DefaultContextTranslation` | Runtime-only | Stateless strategy |
| **agents/basic_agent.rs** | | |
| `BasicAgent` | **Node** (partially) | **Agent** (runtime state wraps graph state) |
| **agents/profile.rs** | | |
| `AgentProfile` | **Node** | **AgentProfile** |
| **agents/agent.rs** | | |
| `QueueMode` | Value Object (enum) | Property of Agent |
| **agents/system_prompt.rs** | | |
| `PromptBlockDef` | **Node** | **PromptBlock** |
| `CustomPromptStrategy` | Value Object | Property of SystemPrompt |
| `AgentPromptStrategy` | Value Object | Property of SystemPrompt |
| `MinimalPromptStrategy` | Value Object | Property of SystemPrompt |
| `SystemPrompt` | **Node** | **SystemPrompt** |
| **agents/sub_agent.rs** | | |
| `SubAgentTool` | Runtime-only | Executor creating DELEGATES_TO edges |
| **context/config.rs** | | |
| `CompactionScope` | Value Object (enum) | Property of CompactionPolicy |
| `CompactionConfig` | **Node** | **CompactionPolicy** |
| `ContextConfig` | Runtime-only | Rebuilt from nodes each loop |
| **context/execution.rs** | | |
| `ExecutionLimits` | **Node** | **ExecutionLimits** |
| `ExecutionTracker` | Runtime-only | In-flight tracking |
| **context/compaction.rs** | | |
| `CompactionBlock` | Value Object | Property of Loop |
| `TurnRange` | Value Object | Property of CompactionBlock |
| `CompactedSection` | Value Object | Property of CompactionBlock |
| `TurnMap` | Runtime-only | Internal index |
| **context/tracker.rs** | | |
| `ContextTracker` | Runtime-only | In-flight tracking |
| **context/token.rs** | | |
| `HeuristicTokenCounter` | Runtime-only | Stateless counter |
| **context/strategy.rs** | | |
| `DefaultCompaction` | Runtime-only | Strategy impl |
| `DefaultBlockCompaction` | Runtime-only | Strategy impl |
| **context/skills.rs** | | |
| `Skill` | **Node** | **Skill** |
| `SkillSet` | Runtime-only | In-memory collection |
| `SkillError` | Runtime-only | Error taxonomy |
| **tools/*.rs** | | |
| `BashTool` ... `PrunTool` | Runtime-only | Executor types → ToolImplementation node metadata |
| `ToolRegistry` | Runtime-only | In-memory collection |
| `PrunRecord` | Value Object | Property of Loop/Turn |
| `PrunVariant` | Value Object (enum) | Property of PrunRecord |
| `PrunRequest` | Runtime-only | Transient request |
| **mcp/types.rs** | | |
| `McpToolInfo` | Value Object | PROVIDES_TOOL edge data |
| `ServerInfo` | Value Object | Property of McpServer |
| `ServerCapabilities` | Value Object | Property of McpServer |
| `ClientInfo` | Value Object | Property of McpServer handshake |
| `InitializeResult` | Value Object | McpServer initialization response |
| `ToolsListResult` | Value Object | McpServer tools response |
| `McpContent` | Value Object | MCP content variant |
| `McpToolCallResult` | Value Object | MCP tool execution result |
| `McpError` | Runtime-only | Error taxonomy |
| `JsonRpcRequest` | Value Object | MCP wire protocol |
| `JsonRpcResponse` | Value Object | MCP wire protocol |
| `JsonRpcError` | Value Object | MCP wire protocol |
| **mcp/client.rs** | | |
| `McpClient` | Runtime-only | Connection state machine → McpServer node |
| **mcp/transport.rs** | | |
| `StdioTransport` | Runtime-only | Process-level transport |
| `HttpTransport` | Runtime-only | HTTP transport |
| **mcp/tool_adapter.rs** | | |
| `McpToolAdapter` | Runtime-only | AgentTool bridge |
| **agent_loop/config.rs** | | |
| `AgentLoopConfig` | Runtime-only | Non-serializable; rebuilt from nodes |
| **agent_loop/evaluation.rs** | | |
| `TransparentEvaluation` ... `LlmJudgeEvaluation` | **Node** (via) | **EvaluationStrategy** |
| **agent_loop/script_callback.rs** | | |
| `ScriptCallback` | Runtime-only | Phase 2 WASM execution |
| `ScriptCallbackError` | Runtime-only | Error taxonomy |
| **openapi/types.rs** | | |
| `OpenApiAuth` | Value Object (enum) | Property of OpenApiSpec |
| `OpenApiConfig` | **Node** (partially) | **OpenApiSpec** |
| `OperationFilter` | Value Object (enum) | Property of OpenApiSpec |
| `OpenApiError` | Runtime-only | Error taxonomy |
| **openapi/adapter.rs** | | |
| `OpenApiToolAdapter` | Runtime-only | AgentTool bridge |
| **config/schema.rs** (26 structs) | | |
| `AgentConfig` | **Node** | **AgentConfig** |
| All `*Section` / `*Instance` types | Value Objects | Map to node properties via config builder |
| **config/builder.rs** | | |
| `ConfigError` | Runtime-only | Error taxonomy |
| **config/parser.rs** | | |
| `ConfigFormat` | Value Object (enum) | Property of AgentConfig |
| **config/reference.rs** | | |
| `ConfigRef` | Value Object (enum) | Config reference protocol |

---

## Changelog

| Date | Topic | Key Decisions |
|------|-------|---------------|
| 2026-04-05 | Data model foundation | Agent-centric ontology; 10 node types, 12 edge types; Message as separate node; schema registry for introspection |
| 2026-04-05 | Permissions (emerging) | 5 permission dimensions identified; phi-core hooks as extension points; baby-phi owns policy |
| 2026-04-05 | Future scenarios | HITL resume, checkpoint restore, parallel exploration, auto-dispatch — all mapped to ontology |
| 2026-04-05 | Comprehensive expansion | 20 node types, 27 edge types, 50+ value objects, 45+ runtime-only types; added MCP, Memory, User, Permission, Event, SystemPrompt, EvaluationStrategy, CompactionPolicy, RetryPolicy, CachePolicy, OpenApiSpec, ToolImplementation, PromptBlock, AgentConfig nodes; full phi-core struct mapping in Appendix A |
