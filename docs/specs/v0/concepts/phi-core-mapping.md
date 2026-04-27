<!-- Last verified: 2026-04-27 by Claude Code -->
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
| `AgentProfile` (struct) | **Node** | **AgentProfile** (wrapped at `domain::AgentProfile.blueprint`) |
| `SystemPrompt` | **Node** | **SystemPrompt** |
| `PromptBlockDef` | **Node** | **PromptBlock** |
| `Agent` (trait) | **Runtime-only** | The stateless execution interface (`prompt_messages_with_sender`, `continue_loop_with_sender`, state access). NOT a node; NOT instantiated by baby-phi (see §"Connection point" below). |
| `BasicAgent` (struct) | **Runtime-only** | Concrete in-memory impl of the `Agent` trait for callers wanting a stateful long-lived wrapper. baby-phi does NOT persist `BasicAgent` state — baby-phi is per-request stateless and uses `phi_core::agent_loop()` (free function) directly. baby-phi's `domain::Agent` is the orthogonal **governance** counterpart (identity / role / org membership / lifecycle), not a wrap of `BasicAgent`. |
| `SubAgentTool` | Runtime-only | Creates DELEGATES_TO edges |
| Remaining types | Value Object or Runtime-only | See Appendix A |

#### Connection point — `domain::Agent` ↔ phi-core runtime types (governance vs runtime separation)

baby-phi's [`domain::Agent`](../../../modules/crates/domain/src/model/nodes.rs) is a governance node (identity, role, org membership, lifecycle fields like `active` / `archived_at` — added by CH-01) — it has **no field overlap** with `phi_core::Agent` (the trait, which has no struct fields), `phi_core::BasicAgent` (a stateful runtime impl baby-phi never instantiates), or `phi_core::AgentProfile` (a serialisable execution blueprint with no governance fields). The two layers connect at session-launch time via **ID-only delegation** at [`sessions/provider.rs::build_agent_context`](../../../modules/crates/server/src/platform/sessions/provider.rs):

```
domain::Agent       domain::AgentProfile.blueprint        launched Session/LoopRecord
       │                       │                                     │
       │ id.to_string()        │ system_prompt.clone()               │ id.to_string()
       ▼                       ▼                                     ▼
  phi_core::types::context::AgentContext {
      system_prompt: ...,        ← from AgentProfile.blueprint.system_prompt
      agent_id:    Some(...),    ← from domain::Agent.id  (ID-only flow)
      session_id:  Some(...),    ← from launched Session.id
      loop_id:     ...,          ← from launched LoopRecord.id
      messages:    vec![],
  }
```

phi-core never sees `domain::Agent`. It only sees the inputs baby-phi has assembled from governance state. The four explicit connection points:
- **(a) Permission gating** — `domain::Agent.active` / `archived_at` / `role` checked at session-launch step 1 BEFORE phi-core is invoked. Governance gates the engine.
- **(b) ID propagation** — `domain::Agent.id.to_string()` becomes `AgentContext.agent_id` for traceability inside emitted events.
- **(c) Blueprint flow** — `domain::AgentProfile.blueprint.system_prompt` (the phi-core wrap field) becomes `AgentContext.system_prompt`.
- **(d) Model flow** — `domain::ModelConfig` resolves to `phi_core::provider::model::ModelConfig` for `AgentLoopConfig`.

Why baby-phi doesn't use `phi_core::Agent` (the trait) or `BasicAgent` (the impl): `phi_core::agent_loop()` is a free function and does not require the caller to implement the `Agent` trait. The trait + `BasicAgent` exist for callers who want a stateful long-lived in-memory wrapper (e.g., a CLI chat REPL with state living in process memory). **baby-phi is per-request stateless** — every session is a fresh `agent_loop` invocation; nothing lives in process memory between requests; all state is persisted to SurrealDB via `BabyPhiSessionRecorder`. This boundary is recorded in [ADR-0034](../implementation/m5_2/decisions/0034-agent-durable-lifecycle.md) §D34.6 with a review trigger if a future milestone introduces long-lived in-memory chat agents.

See also: [`baby-phi/CLAUDE.md`](../../../CLAUDE.md) §"Orthogonal surfaces that are NOT phi-core duplicates" — bullet for `domain::Agent` cites this connection point.

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
