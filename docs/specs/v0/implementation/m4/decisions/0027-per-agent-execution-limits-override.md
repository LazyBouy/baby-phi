<!-- Last verified: 2026-04-23 by Claude Code -->

# ADR-0027 — Per-agent ExecutionLimits override

**Status: Accepted** — flipped at M4/P5 close after `create_agent` + `update_agent_profile` orchestrators + the `execution_limits` resolver landed; 4 unit tests + compile-time coercion witnesses pin the invariants.

## Context

[ADR-0023](../../m3/decisions/0023-system-agents-inherit-from-org-snapshot.md) established the inherit-from-snapshot invariant: every agent reads `ExecutionLimits`/`ContextConfig`/`RetryConfig` from its owning org's `defaults_snapshot`. No per-agent rows. M4/P0 planning asked: should page 09 (agent profile editor) support per-agent overrides?

User decision (M4/P0): **yes for `ExecutionLimits`; no for `ContextConfig` and `RetryConfig`**. Page 09 exposes editable max_turns / max_total_tokens / max_duration / max_cost fields. ADR-0023 stays authoritative for the *default* path (no override row); ADR-0027 layers the opt-in override.

## Decision

1. **New table `agent_execution_limits`** (migration 0004) keyed by `owning_agent: AgentId UNIQUE`. Row carries `phi_core::ExecutionLimits` wrapped in `AgentExecutionLimitsOverride` composite + `created_at`.
2. **Resolution function**: `resolve_effective_execution_limits(agent_id) -> phi_core::ExecutionLimits` — returns the override row if present, else falls back to `Organization.defaults_snapshot.execution_limits`.
3. **Invariant (enforced at repo layer, pinned by 50-case proptest)**: every field on the override ≤ corresponding field on the org snapshot. Operators cannot raise per-agent ceilings above org ceilings; only lower them for cost-control.
4. **Form UX**: page 09 has "Inherit from org" / "Override" radio. Override shows 4 editable fields with org-ceiling hints. "Revert to org default" button DELETEs the override row (idempotent).
5. **Retention of ADR-0023**: system agents (page 13 scope, not M4) still inherit-only. Per-agent `ContextConfig` / `RetryConfig` overrides remain unsupported — if demand surfaces, separate ADR at M5+.

## Consequences

**Positive:** cost-control operators can tighten specific agents without raising org-wide ceilings; phi-core wrap pattern identical to M3's `OrganizationDefaultsSnapshot` (compile-time coercion test pinned).

**Negative:** two sources of truth for per-agent limits (override row OR snapshot) — `resolve_effective_execution_limits` is the single entry point every caller must use. Grep-enforced at review.

**Neutral:** bounds invariant (≤ org snapshot) means changing the org snapshot upward doesn't retroactively raise per-agent values; changing org snapshot downward might violate per-agent override. M4 treats this as an acceptable edge case (operator must manually lower per-agent overrides when tightening org defaults). M5 may add validation on platform-defaults PUT.

## References

- [ADR-0023](../../m3/decisions/0023-system-agents-inherit-from-org-snapshot.md) — the default path this ADR layers on top of.
- [M4 plan §D5 / §D-M4-2](../../../../plan/build/a634be65-m4-agents-and-projects.md).
- [Requirements admin/09 §W3](../../../requirements/admin/09-agent-profile-editor.md).
- [agent-profile-editor.md](../architecture/agent-profile-editor.md).
