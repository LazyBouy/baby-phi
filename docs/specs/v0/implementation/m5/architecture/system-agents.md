<!-- Last verified: 2026-04-23 by Claude Code -->

# Page 13 — System Agents Config architecture

**Status**: `[EXISTS]` as of M5/P6. Business logic in
[`server::platform::system_agents`](../../../../../../modules/crates/server/src/platform/system_agents/);
HTTP surface in
[`server::handlers::system_agents`](../../../../../../modules/crates/server/src/handlers/system_agents.rs).
CLI + Web deferred to P7 (drift D6.2, matching D5.1 precedent).

## HTTP surface

Five routes registered in
[`router.rs`](../../../../../../modules/crates/server/src/router.rs):

| Method | Path | Handler | Purpose |
|---|---|---|---|
| GET   | `/api/v0/orgs/:org_id/system-agents` | `system_agents::list` | R-ADMIN-13-R1/R2 rows + `recent_events` feed |
| POST  | `/api/v0/orgs/:org_id/system-agents` | `system_agents::add` | R-ADMIN-13-W2 add org-specific |
| PATCH | `/api/v0/orgs/:org_id/system-agents/:agent_id` | `system_agents::tune` | R-ADMIN-13-W1 tune parallelize |
| POST  | `/api/v0/orgs/:org_id/system-agents/:agent_id/disable` | `system_agents::disable` | R-ADMIN-13-W3 (confirm required) |
| POST  | `/api/v0/orgs/:org_id/system-agents/:agent_id/archive` | `system_agents::archive` | R-ADMIN-13-W4 (org-specific only) |

## Buckets

The list endpoint returns:

```jsonc
{
  "standard":     [{...}],  // canonical memory-extraction + agent-catalog slugs
  "org_specific": [{...}],  // everything else with AgentRole::System
  "recent_events": [{"agent_id": "...", "at": "..."}]  // last 20 fires
}
```

Bucketing rule: a system agent lands in `standard` when its
`AgentProfile.blueprint.config_id` matches one of
[`STANDARD_SYSTEM_AGENT_PROFILES`](../../../../../../modules/crates/server/src/platform/system_agents/mod.rs)
(`"system-memory-extraction"` or `"system-agent-catalog"`). Everything
else lands in `org_specific` — including the two fixture-provisioned
"standard" agents whose profile slugs differ (the fixture pre-dates
the canonical slugs). See drift D6.3.

## SystemAgentRuntimeStatus node

Governance-node table shipped in migration 0005:

- `agent_id: AgentId`
- `owning_org: OrgId`
- `queue_depth: u32`
- `last_fired_at: Option<DateTime<Utc>>`
- `effective_parallelize: u32`
- `last_error: Option<String>`
- `updated_at: DateTime<Utc>`

M5/P6 ships the shared helper
[`domain::events::listeners::record_system_agent_fire`](../../../../../../modules/crates/domain/src/events/listeners.rs)
that upserts this tile on every system-agent listener fire. **Call
sites are not wired at M5/P6** — Template A/C/D listeners target
grants, not system agents; memory-extraction + agent-catalog listener
bodies are stubs until M5/P8. Helper is ready for P8 + M7+ wiring.
See drift **D6.1**.

## Trigger enum

Five governance-plane triggers (NOT `phi_core::AgentEvent`) — a
deliberate Q3 rejection per Part 1.5 so agent-loop telemetry and
governance reactivity stay separate:

| Slug | Source | Notes |
|---|---|---|
| `session_end` | `DomainEvent::SessionEnded` | s02 memory-extraction trigger |
| `edge_change` | Any edge-mutating `DomainEvent` | s03 agent-catalog + s05 Template firings |
| `periodic` | Timer-driven | Deferred to M7/s06 |
| `explicit` | Operator-invoked API | No-op at M5; reserved |
| `custom_event` | Extensibility hook | M7+ |

## C-M5-5 carry-forward (profile binding on system agents)

Adding a new system agent via the POST handler creates an
`AgentProfile` with `model_config_id: None` — system agents that
need an LLM runtime at invocation time bind it via the **profile
update path** (`PATCH /agents/:id/profile`), NOT via the
system-agent add endpoint. This matches C-M5-5 which already owns
the `model_config_id` lifecycle gate.

## phi-core leverage

One new direct import at P6 — matches Part 1.5 prediction:
- `phi_core::agents::profile::AgentProfile` in
  [`add.rs`](../../../../../../modules/crates/server/src/platform/system_agents/add.rs)
  for profile blueprint construction.

Post-P6 workspace total: **26 lines** (P5 close was 25; +1 at P6).

## Cross-references

- [requirements/admin/13-system-agents-config.md](../../../requirements/admin/13-system-agents-config.md).
- [Event bus M5 extensions](./event-bus-m5-extensions.md) — `DomainEvent::SessionEnded` + edge variants drive the listener fires the runtime-status tiles upsert on.
- [ADR-0023](../../m3/decisions/0023-system-agents-inherit-from-org-snapshot.md) — organization defaults pattern drives trigger effective-parallelize resolution.
- [M5 plan §P6](../../../../plan/build/01710c13-m5-templates-system-agents-sessions.md).
