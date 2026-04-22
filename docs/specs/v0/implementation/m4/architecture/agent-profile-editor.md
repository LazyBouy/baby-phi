<!-- Last verified: 2026-04-22 by Claude Code -->

# Architecture — Agent Profile Editor (page 09)

**Status: [PLANNED M4/P5]** — fleshed out at P5 close. **M4's phi-core-heaviest phase.**

Create + edit Agent form binding four phi-core types directly:
- `phi_core::agents::profile::AgentProfile` — name, system_prompt, thinking_level, temperature, personality.
- `phi_core::context::execution::ExecutionLimits` — max_turns, max_total_tokens, max_duration, max_cost. Per-agent override supported at M4 (ADR-0027 layered on top of ADR-0023's inherit-from-snapshot default).
- `phi_core::provider::model::ModelConfig` — dropdown populated from org catalogue.
- `phi_core::types::ThinkingLevel` — 4-variant dropdown (`Off` / `Low` / `Medium` / `High`), default `Medium` per D-M4-1.

Per-agent `ExecutionLimits` override invariant: every field ≤ corresponding field in `Organization.defaults_snapshot.execution_limits`. Enforced at create + edit.

Active-session guard: `ModelConfig.id` change returns 409 `ACTIVE_SESSIONS_BLOCK_MODEL_CHANGE` per D-M4-3.

See:
- [Requirements admin/09](../../../requirements/admin/09-agent-profile-editor.md).
- [ADR-0027](../decisions/0027-per-agent-execution-limits-override.md).
- [phi-core-reuse-map.md §Page 09](phi-core-reuse-map.md).
- [M4 plan archive §P5](../../../../plan/build/a634be65-m4-agents-and-projects.md).
