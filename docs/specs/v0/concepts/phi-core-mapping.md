<!-- Status: CONCEPTUAL -->

# phi-core Type Mapping

> Extracted from brainstorm.md Appendix A + Section 2.5.
> Reference document mapping all phi-core public types to the phi ontology.
> See also: [ontology.md](ontology.md) (the ontological model these types map into)

---

## Classification Guide

Every public phi-core struct/enum falls into one of four categories:

| Classification | Meaning | Persisted? |
|---------------|---------|------------|
| **Node** | First-class entity with identity and relationships | Yes |
| **Value Object** | Embedded property within a node (no independent identity) | Yes (within parent) |
| **Runtime-only** | Ephemeral process state, not domain entities | No |
| **Error type** | Error taxonomy, surfaced in Event payloads | No |

---

## Complete Mapping Table

> For the full table mapping all 158+ phi-core types, see brainstorm.md Appendix A.
> Below is a summary by module.

### types/ (25 types)

| Type | Classification | Maps To |
|------|---------------|---------|
| `Content` | Value Object | Property of Message |
| `Message` | **Node** | **Message** |
| `StopReason` | Value Object (enum) | Property of Message |
| `TurnId` | Value Object | Composite key of Turn |
| `LlmMessage` | Value Object | Message + TurnId wrapper |
| `AgentMessage` | Value Object | Routing envelope |
| `ExtensionMessage` | Value Object | Non-LLM message |
| `Usage` | Value Object | Property of Loop, Turn |
| `CacheConfig` | **Node** | **CachePolicy** |
| `ThinkingLevel` | Value Object (enum) | Property of AgentProfile, Loop |
| `ContinuationKind` | Value Object (enum) | Property of Loop |
| `TurnTrigger` | Value Object (enum) | Property of Turn |
| `AgentEvent` | **Node** | **Event** |
| `AgentContext` | Runtime-only | In-memory accumulator |
| `ToolResult` | Value Object | Event payload |
| Remaining types | Runtime-only or Value Object | See Appendix A |

### session/ (14 types)

| Type | Classification | Maps To |
|------|---------------|---------|
| `Session` | **Node** | **Session** |
| `LoopRecord` | **Node** | **Loop** |
| `Turn` | **Node** | **Turn** |
| `LoopConfigSnapshot` | Value Object | CONFIGURED_WITH target |
| `ChildLoopRef` | Value Object | SPAWNED_CHILD edge data |
| `SpawnRef` | Value Object | SPAWNED_FROM edge data |
| `ParallelGroupRecord` | Value Object | PARALLEL_WITH edge data |
| `SessionRecorder` | Runtime-only | Event materializer |
| Remaining types | Value Object or Runtime-only | See Appendix A |

### provider/ (24 types)

| Type | Classification | Maps To |
|------|---------------|---------|
| `ModelConfig` | **Node** | **ModelConfig** |
| `ToolDefinition` | **Node** | **ToolDefinition** |
| `RetryConfig` | **Node** | **RetryPolicy** |
| `ApiProtocol` | Value Object (enum) | Property of ModelConfig |
| `CostConfig` | Value Object | Property of ModelConfig |
| 7 provider impls | Runtime-only | Stateless HTTP clients |
| `MockProvider` etc. | Runtime-only | Test-only |
| Remaining types | Runtime-only or Value Object | See Appendix A |

### agents/ (8 types)

| Type | Classification | Maps To |
|------|---------------|---------|
| `AgentProfile` | **Node** | **AgentProfile** |
| `SystemPrompt` | **Node** | **SystemPrompt** |
| `PromptBlockDef` | **Node** | **PromptBlock** |
| `BasicAgent` | **Node** (partial) | **Agent** runtime state |
| `SubAgentTool` | Runtime-only | Creates DELEGATES_TO edges |
| Remaining types | Value Object or Runtime-only | See Appendix A |

### config/ (28 types)

| Type | Classification | Maps To |
|------|---------------|---------|
| `AgentConfig` | **Node** | **AgentConfig** |
| All `*Section` types | Value Object | Map to node properties via builder |
| `ConfigRef` | Value Object (enum) | Reference protocol |
| `ConfigFormat` | Value Object (enum) | Property of AgentConfig |
| `ConfigError` | Runtime-only | Error taxonomy |

### context/ (14 types)

| Type | Classification | Maps To |
|------|---------------|---------|
| `CompactionConfig` | **Node** | **CompactionPolicy** |
| `ExecutionLimits` | **Node** | **ExecutionLimits** |
| `Skill` | **Node** | **Skill** |
| `CompactionBlock` | Value Object | Property of Loop |
| Remaining types | Runtime-only | Strategy impls, trackers |

### tools/ (11 types)

| Type | Classification | Maps To |
|------|---------------|---------|
| All tool structs | Runtime-only | ToolImplementation node metadata |
| `ToolRegistry` | Runtime-only | In-memory collection |
| `PrunRecord` | Value Object | Property of Loop/Turn |

### mcp/ (16 types)

| Type | Classification | Maps To |
|------|---------------|---------|
| `McpToolInfo` | Value Object | PROVIDES_TOOL edge data |
| `ServerInfo`, `ServerCapabilities` | Value Object | Property of McpServer |
| `McpClient` | Runtime-only | Connection state machine |
| Transport types | Runtime-only | Process-level |
| JSON-RPC types | Value Object | Wire protocol |

### agent_loop/ (9 types)

| Type | Classification | Maps To |
|------|---------------|---------|
| `*Evaluation` strategies | **Node** (via) | **EvaluationStrategy** |
| `AgentLoopConfig` | Runtime-only | Non-serializable config |
| `ScriptCallback` | Runtime-only | Phase 2 WASM |

### openapi/ (5 types)

| Type | Classification | Maps To |
|------|---------------|---------|
| `OpenApiConfig` | **Node** (partial) | **OpenApiSpec** |
| `OpenApiToolAdapter` | Runtime-only | AgentTool bridge |
| Remaining types | Value Object or Runtime-only | See Appendix A |

---

## Runtime-Only Types (45+ total)

These exist only in memory during agent execution. They are implementation machinery, not domain entities.

**Why excluded:** No identity, no persistence need, no cross-agent relevance.

**Categories:**
- **Execution state:** AgentContext, InRunEntry, ExecutionTracker, ContextTracker, TurnMap
- **Config builders:** AgentLoopConfig, StreamConfig, ContextConfig
- **Registries:** ProviderRegistry, ToolRegistry, SkillSet
- **Session recording:** SessionRecorder, SessionRecorderConfig
- **Provider impls:** 7 concrete providers (Anthropic, OpenAI, Google, etc.) + Mock
- **Streaming:** StreamEvent, SseEvent, DefaultContextTranslation
- **Evaluation:** EvaluationDecision, ParallelLoopOutcome, ParallelLoopResult
- **Tool executors:** BashTool, ReadFileTool, WriteFileTool, EditFileTool, SearchTool, ListFilesTool, PrunTool, SubAgentTool, McpToolAdapter, OpenApiToolAdapter
- **Strategy impls:** DefaultCompaction, DefaultBlockCompaction
- **Transports:** StdioTransport, HttpTransport
- **Callbacks:** ScriptCallback

**Error types** (8 total): ProviderError, ToolError, McpError, OpenApiError, SkillError, SessionError, ConfigError, ScriptCallbackError — surfaced in Event payloads or Loop.rejection, not persisted independently.
