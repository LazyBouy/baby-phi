<!-- Last verified: 2026-04-23 by Claude Code -->

# Architecture — Agent Profile Editor (page 09)

**Status: [EXISTS]** — landed at M4/P5. **M4's phi-core-heaviest phase.**

Create + edit Agent form binding four phi-core types directly:

- [`phi_core::agents::profile::AgentProfile`] — `system_prompt`, `thinking_level`, `temperature`, `config_id`, `skills`, `workspace`.
- [`phi_core::context::execution::ExecutionLimits`] — `max_turns`, `max_total_tokens`, `max_duration`, `max_cost`. Per-agent override supported at M4 (ADR-0027 opt-in on top of ADR-0023's inherit-from-snapshot default).
- [`phi_core::provider::model::ModelConfig`] — **editing deferred to M5**; the field isn't on baby-phi's `AgentProfile` wrap yet (phi-core's `AgentProfile` has no embedded `model_config`; baby-phi will add a governance extension at M5 for per-agent model binding). The 409 `ACTIVE_SESSIONS_BLOCK_MODEL_CHANGE` code path + [`Repository::count_active_sessions_for_agent`] stub stay wired so M5 flips the check without restructuring.
- [`phi_core::types::ThinkingLevel`] — **5-variant** dropdown (`Off`, `Minimal`, `Low`, `Medium`, `High`). M4 plan D-M4-1 mentioned 4 variants; the actual phi-core enum ships 5, and the dropdown matches the code.

## Surfaces

| Tier | Path | Entry point |
|---|---|---|
| HTTP (create) | `POST /api/v0/orgs/:org_id/agents` | `server/src/handlers/agents.rs::create` |
| HTTP (update) | `PATCH /api/v0/agents/:id/profile` | `server/src/handlers/agents.rs::update` |
| HTTP (revert) | `DELETE /api/v0/agents/:id/execution-limits-override` | `server/src/handlers/agents.rs::revert_execution_limits_override` |
| CLI | `phi agent create --org-id --name --kind --role --system-prompt --parallelize [--override-max-*]` | `cli/src/commands/agent.rs::create_impl` |
| CLI | `phi agent update --id --patch-json` | `cli/src/commands/agent.rs::update_impl` |
| CLI | `phi agent revert-limits --id` | `cli/src/commands/agent.rs::revert_limits_impl` |
| Web | `(admin)/organizations/[id]/agents/new` + `[agent_id]` | SSR; form action via Server Action |

Business logic: `server/src/platform/agents/{create,update,execution_limits}.rs`.

## Flow (create)

1. Handler deserialises `CreateAgentRequest` carrying phi-core `AgentProfile` + optional `ExecutionLimits`.
2. `create_agent` orchestrator validates: `display_name` non-empty, `parallelize ∈ 1..=64`, `role.is_valid_for(kind)`, org exists.
3. If `initial_execution_limits_override` is supplied, `AgentExecutionLimitsOverride::is_bounded_by(org_snapshot)` is checked; breach → 400 `EXECUTION_LIMITS_EXCEED_ORG_CEILING`.
4. `Repository::apply_agent_creation` compound tx (M4/P3 C10): Agent + Inbox + Outbox + optional Profile + optional override row + catalogue seeds, atomic.
5. Emits `platform.agent.created` (Alerted) via the audit emitter.

## Flow (update)

1. Handler deserialises `UpdateAgentProfileRequest`.
2. `update_agent_profile` orchestrator:
   - Rejects immutable-field changes (`kind`, `role`, `owning_org`) → 400 `AGENT_IMMUTABLE_FIELD_CHANGED`.
   - Rejects system-role edits → 403 `SYSTEM_AGENT_READ_ONLY`.
   - Validates `parallelize` range, `blueprint` field-by-field diff.
   - Handles three `ExecutionLimits` paths:
     - `Unchanged` — leave current state alone.
     - `Revert` — idempotent DELETE of override row; falls back to org snapshot.
     - `Set(limits)` — upsert override row after bounds check; 400 on breach.
3. Persists via `Repository::upsert_agent` + `Repository::upsert_agent_profile` (when those fields changed).
4. Emits `platform.agent_profile.updated` (Logged; Alerted on `model_config_id` / `role` change — M5) with structured `AgentProfilePatchDiff` iff any field changed. No-op patches return `audit_event_id: null` and emit no event.

## phi-core leverage (Q1 / Q2 / Q3)

- **Q1 direct imports** (the phi-core-heavy phase):
  - `use phi_core::agents::profile::AgentProfile as PhiCoreAgentProfile;` in `create.rs`, `update.rs`, `handlers/agents.rs`.
  - `use phi_core::context::execution::ExecutionLimits;` in `create.rs`, `update.rs`, `execution_limits.rs`, `handlers/agents.rs`.
  - `use phi_core::types::ThinkingLevel;` in `update.rs` (for compile-time coercion witness).
- **Q2 transitive**: wire shapes `CreateAgentRequest.blueprint`, `CreateAgentRequest.initial_execution_limits_override`, `UpdateAgentProfileRequest.blueprint`, `UpdateAgentProfileRequest.execution_limits.set` all carry phi-core types via serde round-trip. The audit `AgentProfilePatchDiff` captures `(before, after)` tuples of phi-core types.
- **Q3 rejections**: `ContextConfig` + `RetryConfig` stay inherit-from-snapshot (ADR-0023). `ModelConfig` editing is M5 (the field isn't on phi-core's `AgentProfile`; baby-phi adds a governance extension at M5).

Compile-time coercion witnesses (all in `#[allow(dead_code)]` free functions):

- `_is_phi_core_agent_profile(_: &PhiCoreAgentProfile)` — pinned in `create.rs`, `update.rs`.
- `_is_phi_core_execution_limits(_: &ExecutionLimits)` — pinned in `create.rs`, `update.rs`, `execution_limits.rs`.
- `_is_phi_core_thinking_level(_: &ThinkingLevel)` — pinned in `update.rs`.

Positive close-audit greps (all must return at least one match):

```bash
grep -En 'use phi_core::agents::profile::AgentProfile' modules/crates/server/src/platform/agents/
grep -En 'use phi_core::context::execution::ExecutionLimits' modules/crates/server/src/platform/agents/
grep -En 'use phi_core::types::ThinkingLevel' modules/crates/server/src/platform/agents/
```

## Invariants

1. **Override bound** — `AgentExecutionLimitsOverride::is_bounded_by(org_snapshot)` returns `true` for every persisted row. Enforced at create (`create.rs`) + set-override (`execution_limits::apply_override`). Violation → 400 `EXECUTION_LIMITS_EXCEED_ORG_CEILING`.
2. **Inherit default** — `resolve_effective_execution_limits(agent)` returns the override when present, else the org snapshot, else `ExecutionLimits::default()` (with a `tracing::warn!`).
3. **Role+kind validity** — `AgentRole::is_valid_for(kind)` enforced at create. Immutable post-create at M4.
4. **System-agent read-only** — any update path for a `System`-role agent returns 403.

## Deferred to M5

- Per-agent `ModelConfig` binding + the active-session-gated change flow. Error-code variant (`ACTIVE_SESSIONS_BLOCK_MODEL_CHANGE`) + stub counter (`count_active_sessions_for_agent` returning `Ok(0)`) are wired today so M5 flips the check in one spot.
- Richer "show agent" endpoint (returns full profile + override + audit trail). At M4/P5 the edit page reads the agent from the roster list.
- `Alerted` escalation on `role` change (role transitions are a separate M5+ flow).

## References

- [ADR-0027](../decisions/0027-per-agent-execution-limits-override.md) — the opt-in override decision.
- [ADR-0023](../../m3/decisions/0023-system-agents-inherit-from-org-snapshot.md) — the default inherit path.
- [Requirements admin/09](../../../requirements/admin/09-agent-profile-editor.md).
- [phi-core-reuse-map.md §Page 09](phi-core-reuse-map.md).
- [Agent-profile editor ops runbook](../operations/agent-profile-editor-operations.md).
- [M4 plan archive §P5](../../../../plan/build/a634be65-m4-agents-and-projects.md).

[`phi_core::agents::profile::AgentProfile`]: https://docs.rs/phi-core/latest/phi_core/agents/profile/struct.AgentProfile.html
[`phi_core::context::execution::ExecutionLimits`]: https://docs.rs/phi-core/latest/phi_core/context/execution/struct.ExecutionLimits.html
[`phi_core::provider::model::ModelConfig`]: https://docs.rs/phi-core/latest/phi_core/provider/model/struct.ModelConfig.html
[`phi_core::types::ThinkingLevel`]: https://docs.rs/phi-core/latest/phi_core/types/enum.ThinkingLevel.html
[`Repository::count_active_sessions_for_agent`]: ../../../../../../modules/crates/domain/src/repository.rs
