<!-- Last verified: 2026-04-27 by Claude Code -->
<!-- CH-01 + CH-22 amendments (2026-04-27): operator-visible disable/archive durable-state semantics + agent-catalog audit-mode flag. Full P6 walkthrough prose still deferred to M5-tag-close. -->

# System agents config — walkthrough

**Status**: [PARTIAL M5/P6] — full walkthrough prose deferred to M5-tag-close; CH-01 + CH-22 amendments below describe the operator-visible behavior shipped post-M5/P6.

## CH-01 amendment — disable + archive flip durable state (2026-04-27)

When you `POST /api/v0/orgs/:org/system-agents/:id/disable` (with `confirm: true`), the agent's row gets `active = false` written to durable storage **before** the audit event emits. The behavior is now:

- Re-running disable on an already-disabled agent succeeds with `200` and is idempotent. No `409 SYSTEM_AGENT_ALREADY_TERMINAL` is returned for the disabled state — that error is reserved for archive-after-archive.
- If audit emit fails (look for `AUDIT_EMIT_ERROR`), the agent row's `active = false` IS persisted. Replay the audit chain rather than re-issuing the disable.

`POST /system-agents/:id/archive` similarly writes `archived_at = Some(<ISO-8601 timestamp>)` to durable storage. Re-archiving overwrites the timestamp (later wins). Standard system agents (memory-extraction + agent-catalog) hard-fail with `409 STANDARD_SYSTEM_AGENT_NOT_ARCHIVABLE` — disable them instead.

To verify durable state from the CLI:

```bash
phi agent show --id <agent-uuid> --json | jq '{active, archived_at}'
```

(The `phi agent show` JSON output now carries both fields.)

## CH-22 amendment — catalog listener body + audit_mode flag (2026-04-27)

The agent-catalog system agent now does real work on every agent-lifecycle event in your org. Before CH-22, its `system_agent_runtime_status` tile (visible on page 13) showed empty `last_fired_at` because the listener body was a stub. Post-CH-22:

- Creating any agent (Human Member, LLM, custom system agent), updating an agent's profile, disabling an agent, archiving an agent, starting a session, or ending a session all upsert the `agent_catalog_entry` row for the affected agent and bump the catalog system agent's `last_fired_at` tile.
- The catalog row mirrors `agent.active && agent.archived_at.is_none()` — an archived agent's catalog row reads `active = false` even if the durable `agent.active` is still `true` (archive wins ties; ADR-0034 §D34.5).

### Audit-mode flag (operators / developers)

Default behaviour is **silent** — the listener does its work but emits no audit event per fire. To trace the listener end-to-end during dev or acceptance testing, flip to debug mode:

```bash
# Override at boot via env var (preferred for short-lived investigations)
PHI_LISTENERS__CATALOG__AUDIT_MODE=debug phi-server

# Or via config/<profile>.toml
[listeners.catalog]
audit_mode = "debug"
```

Debug mode emits one `agent_catalog_refreshed` audit event per listener fire, audit-class `Silent` (30-day retention). Diff payload includes `triggering_event_id` + `triggering_event_kind` so you can join the listener fire to the originating audit row.

**Production rule:** leave `audit_mode = "silent"` in production. Debug mode would 10–100× audit-log volume on a busy org without governance benefit.

## Cross-references

- [requirements/admin/13-system-agents-config.md](../../../requirements/admin/13-system-agents-config.md).
- [System agents architecture](../architecture/system-agents.md).
- [System agents operations](../operations/system-agents-operations.md).
- [CLI reference M5](cli-reference-m5.md) — `phi system-agent` subcommand surface.
