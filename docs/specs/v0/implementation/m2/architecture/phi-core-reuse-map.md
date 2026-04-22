<!-- Last verified: 2026-04-21 by Claude Code -->

# Architecture — phi-core reuse map

**Source of truth:** the phi-core Leverage section of
[`phi/CLAUDE.md`](../../../../../../CLAUDE.md) plus §1.5 of the
archived M2 plan at
[`../../../../plan/build/a6005e06-m2-platform-setup.md`](../../../../plan/build/a6005e06-m2-platform-setup.md).
This page is the durable publication of that map so M3+ authors don't
have to dig into the plan archive.

**Principle:** phi is a consumer of phi-core. Every surface that
overlaps with a phi-core type MUST import it (direct reuse) or wrap it
(embed as a field) — **never** re-implement. New phi structs are
only permitted where phi-core has no counterpart.

Legend: ✅ direct reuse; 🔌 wrap; 🚫 no phi-core counterpart (build
phi-native).

## M2 overlap table

| Surface | phi-core type (absolute path) | M2 use site | Mode |
|---|---|---|---|
| Provider binding | [`phi_core::provider::model::ModelConfig`](../../../../../../../phi-core/src/provider/model.rs) | `ModelRuntime.config` on the phi composite | 🔌 |
| Provider protocol enum | [`phi_core::provider::model::ApiProtocol`](../../../../../../../phi-core/src/provider/model.rs) | `ProviderKind` = type alias (re-export only) | ✅ |
| Provider factory | [`phi_core::provider::registry::ProviderRegistry`](../../../../../../../phi-core/src/provider/registry.rs) | Web UI dropdown + health-probe dispatch at M5 | ✅ |
| Cache / thinking | [`phi_core::types::usage::CacheConfig`](../../../../../../../phi-core/src/types/usage.rs) / `ThinkingLevel` | Fields on `ModelRuntime.config` | ✅ |
| Stream traits | [`phi_core::provider::traits::{StreamProvider, StreamConfig, StreamEvent}`](../../../../../../../phi-core/src/provider/traits.rs) | Session-launch M5+ | ✅ (deferred) |
| MCP client | [`phi_core::mcp::client::McpClient`](../../../../../../../phi-core/src/mcp/client.rs) | Built on demand from `ExternalService`; never stored | ✅ |
| MCP types | [`phi_core::mcp::types::{McpToolInfo, ServerInfo}`](../../../../../../../phi-core/src/mcp/types.rs) | Page 03 "registered MCP servers" table with live tool counts | ✅ |
| MCP tool adapter | [`phi_core::mcp::tool_adapter::McpToolAdapter`](../../../../../../../phi-core/src/mcp/tool_adapter.rs) | Tool invocation M5+ | ✅ (deferred) |
| Execution limits | [`phi_core::context::execution::ExecutionLimits`](../../../../../../../phi-core/src/context/execution.rs) | `PlatformDefaults.execution_limits` | 🔌 |
| Agent blueprint | [`phi_core::agents::profile::AgentProfile`](../../../../../../../phi-core/src/agents/profile.rs) | `PlatformDefaults.default_agent_profile` + phi's `AgentProfile.blueprint` (M1 wrap, P0) | 🔌 |
| Context config | [`phi_core::context::config::ContextConfig`](../../../../../../../phi-core/src/context/config.rs) | `PlatformDefaults.context_config` | 🔌 |
| Retry tuning | [`phi_core::provider::retry::RetryConfig`](../../../../../../../phi-core/src/provider/retry.rs) | `PlatformDefaults.retry_config` (P1 added `Serialize`/`Deserialize` derives upstream for this) | 🔌 |
| Config parser | [`phi_core::config::parser::{parse_config, parse_config_file, parse_config_auto}`](../../../../../../../phi-core/src/config/parser.rs) | Page 05 YAML/TOML import/export | ✅ |
| Event stream | [`phi_core::types::event::AgentEvent`](../../../../../../../phi-core/src/types/event.rs) | Session-launch M5+ | ✅ (deferred) |
| Session / Turn | [`phi_core::session::model::{Session, LoopRecord, Turn, LoopStatus}`](../../../../../../../phi-core/src/session/model.rs) | Persisted via `SessionRecorder` M5+ | ✅ (deferred) |

## What phi-core does NOT provide (and M2 must build)

- **Credentials vault** (page 04) — `SecretCredential` + `secrets_vault`
  storage + `store::crypto` seal/unseal. 🚫
- **Permission Check engine + constraint lattice** — phi's domain
  entirely. 🚫
- **MCP health probe** — thin wrapper around phi-core's `McpClient::list_tools()`
  with timeout + retry; the probe itself is phi-native. 🚫
- **`TenantSet`** (platform-level orgs-allowed discriminator) —
  not a phi-core concept. 🚫
- **`PlatformDefaults`** envelope — composes phi-core types but the
  container is phi-only. 🚫
- **Template E auto-approve** — permission-system pattern not in
  phi-core's scope. 🚫
- **`AgentGovernanceProfile`-style fields on phi's `AgentProfile`**
  (e.g. `parallelize`) — governance not in phi-core's scope. 🚫

## Orthogonal surfaces (NOT phi-core duplicates)

These look similar from the outside but sit at different layers. Do
not conflate them.

| phi type | phi-core type | Why distinct |
|---|---|---|
| [`domain::audit::AuditEvent`](../../../../../../modules/crates/domain/src/audit/mod.rs) | [`phi_core::types::event::AgentEvent`](../../../../../../../phi-core/src/types/event.rs) | Governance write log (hash-chain, retention tier) vs agent-loop telemetry stream — see [M1 audit-events doc](../../m1/architecture/audit-events.md) |
| [`server::session::SessionClaims`](../../../../../../modules/crates/server/src/session.rs) | [`phi_core::session::model::Session`](../../../../../../../phi-core/src/session/model.rs) | HTTP identity cookie vs persisted execution trace — see [M1 server-topology doc](../../m1/architecture/server-topology.md) |
| [`domain::model::ToolDefinition`](../../../../../../modules/crates/domain/src/model/nodes.rs) | [`phi_core::types::tool::AgentTool`](../../../../../../../phi-core/src/types/tool.rs) | Permission-metadata node vs runtime trait — see [M1 graph-model doc](../../m1/architecture/graph-model.md) |
| [`server::config::ServerConfig`](../../../../../../modules/crates/server/src/config.rs) | [`phi_core::config::schema::AgentConfig`](../../../../../../../phi-core/src/config/schema.rs) | HTTP infrastructure TOML vs agent blueprint YAML/TOML/JSON — see [M1 overview doc](../../m1/architecture/overview.md) |

## Enforcement

1. **CI lint**: [`scripts/check-phi-core-reuse.sh`](../../../../../../scripts/check-phi-core-reuse.sh)
   greps every `.rs` under `modules/crates/` for forbidden
   re-declarations (`struct ExecutionLimits`, `struct ModelConfig`,
   `struct McpClient`, `struct AgentProfile`, `struct RetryConfig`,
   `struct ContextConfig`, `struct AgentEvent`, `struct Session`,
   `struct LoopRecord`). Zero hits required.
2. **Reviewer checklist**: reject any PR whose new type's field set
   matches a phi-core type; require the import instead.
3. **Per-phase audit**: the P3 close and P-final re-audits spot-check
   the M2 composites for direct phi-core imports.
4. **`thiserror` version parity**: phi's workspace `thiserror`
   version MUST match phi-core's (currently `"2"`); drift breaks
   `#[from]` conversions at runtime.
