<!-- Last verified: 2026-04-23 by Claude Code -->

# Operations — Agent Profile Editor (page 09)

**Status: [EXISTS]** — landed at M4/P5.

## Endpoints

| Method | Path | Use |
|---|---|---|
| `POST` | `/api/v0/orgs/:org_id/agents` | Create agent (wizard submit). |
| `PATCH` | `/api/v0/agents/:id/profile` | Update blueprint / display_name / parallelize / execution_limits. |
| `DELETE` | `/api/v0/agents/:id/execution-limits-override` | Revert override row; agent falls back to org snapshot per ADR-0023. |

All gated by `AuthenticatedSession` (cookie `phi_kernel_session`).

### Create wire shape

```json
{
  "display_name": "iris-bot",
  "kind": "llm",
  "role": "intern",
  "blueprint": {
    "system_prompt": "You distill agent memories.",
    "thinking_level": "medium",
    "temperature": 0.2
  },
  "parallelize": 1,
  "initial_execution_limits_override": null
}
```

`blueprint` accepts any `phi_core::AgentProfile` shape (omitted fields use phi-core defaults). `initial_execution_limits_override` is either `null` (inherit; ADR-0023 default) or a `phi_core::ExecutionLimits` object (ADR-0027 opt-in).

### Update patch shape

```json
{
  "display_name": "iris-bot-v2",
  "parallelize": 2,
  "blueprint": { "temperature": 0.1 },
  "execution_limits": { "set": { "max_turns": 25, ... } }
}
```

`execution_limits` is one of `{ "unchanged": null }`, `{ "revert": null }`, or `{ "set": { ...ExecutionLimits... } }`. Omitted = unchanged.

## Error codes

| HTTP | `code` | Meaning | Operator action |
|---|---|---|---|
| 400 | `VALIDATION_FAILED` | Empty `display_name`, etc. | Fix the payload. |
| 400 | `AGENT_IMMUTABLE_FIELD_CHANGED` | Patch tried to change `kind`, `role`, or `owning_org`. | Remove the offending field from the patch. These are immutable post-creation at M4; role transitions land in a later milestone. |
| 400 | `AGENT_ROLE_INVALID_FOR_KIND` | Supplied `role` is incompatible with `kind` per [`AgentRole::is_valid_for`]. | E.g. `executive` is Human-only; `intern` is LLM-only. Choose a compatible pairing. |
| 400 | `PARALLELIZE_CEILING_EXCEEDED` | `parallelize` outside `1..=64`. | M4 caps parallelize at 64; per-org caps land at M5. |
| 400 | `EXECUTION_LIMITS_EXCEED_ORG_CEILING` | Override row would raise some field above the org snapshot's corresponding ceiling. | Override values must tighten, never loosen — per ADR-0027. Lower the values or use `{ "revert": null }` to fall back to inherit. |
| 403 | `SYSTEM_AGENT_READ_ONLY` | Edit attempted on a `System`-role agent. | System agents are platform-managed; edit them via platform-defaults (M5) instead. |
| 404 | `ORG_NOT_FOUND` | `org_id` path segment unknown. | Verify via `GET /api/v0/orgs`. |
| 404 | `AGENT_NOT_FOUND` | `agent_id` path segment unknown. | Verify via `GET /api/v0/orgs/:org_id/agents`. |
| 409 | `AGENT_ID_IN_USE` | Create race: a concurrent request produced the same id. | Retry with a fresh uuid; the repo layer auto-allocates new ids. |
| 409 | `ACTIVE_SESSIONS_BLOCK_MODEL_CHANGE` | **Reserved for M5.** At M4 the `count_active_sessions_for_agent` stub always returns `0`, so this code is unreachable via the handler wire. The enum variant + handler mapping are present for forward-compat. | N/A at M4. |
| 500 | `AUDIT_EMIT_FAILED` | Compound tx committed but the follow-up audit emit failed. | The agent IS persisted (durable). Check server logs for the emit error; the hash chain is consistent up to the last successfully emitted event. |
| 500 | `INTERNAL_ERROR` | Unhandled repository error. | Inspect server logs. |

## Three ExecutionLimits paths — verification queries

### Path A — inherit (default, ADR-0023)

Agent has no `agent_execution_limits` row. Effective limits = org snapshot.

```sql
SELECT * FROM agent_execution_limits WHERE owning_agent = agent:<id>;
-- expected: zero rows
```

### Path B — override (opt-in, ADR-0027)

Agent has exactly one row (the UNIQUE index on `owning_agent` is asserted in migration 0004).

```sql
SELECT * FROM agent_execution_limits WHERE owning_agent = agent:<id>;
-- expected: one row; `limits.max_*` fields all ≤ org snapshot
```

### Path C — revert

`DELETE /api/v0/agents/:id/execution-limits-override` is **idempotent**: it succeeds whether or not a row exists. After a successful revert, Path A holds. The ops helper:

```bash
phi agent revert-limits --id <uuid>
```

## Audit-event shape

`platform.agent.created` (Alerted):

```json
{
  "event_type": "platform.agent.created",
  "audit_class": "alerted",
  "actor_agent_id": "...",
  "target_entity_id": "<new agent id>",
  "org_scope": "<org id>",
  "diff": {
    "before": null,
    "after": {
      "agent_id": "...",
      "kind": "llm",
      "role": "intern",
      "display_name": "iris-bot",
      "has_profile": true,
      "profile_parallelize": 1,
      "initial_execution_limits_override": { ... }
    }
  }
}
```

`platform.agent_profile.updated` (Logged by default; Alerted when `model_config_id` or `role` changes — both reserved for M5). Diff carries `AgentProfilePatchDiff` with `(before, after)` tuples per field. No-op patches emit no event.

## Playbook — "revert" button appears to do nothing

Symptoms: operator clicks "Revert to org default" but the displayed limits don't change.

1. **Check that an override row actually existed.** The DELETE endpoint is idempotent, so if no row was set, revert is a no-op. Query `agent_execution_limits WHERE owning_agent = agent:<id>`.
2. **Check the org snapshot values.** If the override row had the SAME values as the org snapshot, the effective limits are identical before and after revert.

## Playbook — override breach on an edit that used to work

Symptoms: a previously-valid override starts failing with `EXECUTION_LIMITS_EXCEED_ORG_CEILING`.

Cause: the org snapshot's ceilings were tightened (platform-defaults PUT that created a newer org, OR a manual snapshot update). Per-agent overrides were set against the older, higher ceiling and now breach the new one.

Fix: either lower the per-agent override, OR revert + re-inherit.

## phi-core leverage notes

The edit handler is the phi-core-heaviest file in M4:

- `phi_core::AgentProfile` transits the `blueprint` field end-to-end (wire → orchestrator → repo tx → SurrealDB `FLEXIBLE TYPE object`).
- `phi_core::ExecutionLimits` transits through `AgentExecutionLimitsOverride.limits` via the same pattern. The `is_bounded_by(ceiling)` invariant is a pure-fn on phi-core's type.
- `phi_core::ThinkingLevel` appears in the audit patch diff.

Compile-time coercion witnesses pin each type identity (`_is_phi_core_agent_profile`, `_is_phi_core_execution_limits`, `_is_phi_core_thinking_level`) — the build fails if a local type ever shadows any of these.

## References

- [Agent profile editor architecture](../architecture/agent-profile-editor.md)
- [ADR-0027 — per-agent ExecutionLimits override](../decisions/0027-per-agent-execution-limits-override.md)
- [ADR-0023 — inherit-from-snapshot](../../m3/decisions/0023-system-agents-inherit-from-org-snapshot.md)
- [Repository method `count_active_sessions_for_agent`](../../../../../../modules/crates/domain/src/repository.rs)
- [M4 plan archive §P5](../../../../plan/build/a634be65-m4-agents-and-projects.md)

[`AgentRole::is_valid_for`]: ../../../../../../modules/crates/domain/src/model/nodes.rs
