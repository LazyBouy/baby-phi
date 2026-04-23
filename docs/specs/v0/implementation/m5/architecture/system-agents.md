<!-- Last verified: 2026-04-23 by Claude Code -->

# Page 13 — System Agents Config architecture

**Status**: [PLANNED M5/P6] — stub seeded at M5/P0; filled when P6
vertical ships.

Page 13 gives operators the ability to tune / add / disable /
archive system agents per org. Live queue-depth + last-fired-at
surface via a new `SystemAgentRuntimeStatus` node + 5 listeners
upserting on each fire.

## Scope

- `GET   /api/v0/orgs/:org_id/system-agents` — list with per-agent status tile.
- `PATCH /api/v0/orgs/:org_id/system-agents/:agent_id` — tune parallelize, trigger, `profile_ref`.
- `POST  /api/v0/orgs/:org_id/system-agents` — add org-specific.
- `POST  /api/v0/orgs/:org_id/system-agents/:agent_id/disable` — strong-warning dialog for standards.
- `POST  /api/v0/orgs/:org_id/system-agents/:agent_id/archive`.

## SystemAgentRuntimeStatus node

New governance node (table shipped in migration 0005):

- `agent_id: AgentId`
- `owning_org: OrgId`
- `queue_depth: u32`
- `last_fired_at: Option<DateTime<Utc>>`
- `effective_parallelize: u32`
- `last_error: Option<String>`

Upserted by all 5 listeners (Template A/C/D + memory-extraction +
agent-catalog) on every fire via a shared helper in
`domain/src/events/listeners.rs`.

## Trigger enum

Governance-plane (NOT `phi_core::AgentEvent`) — see the Q3
rejection in the [phi-core reuse map](phi-core-reuse-map.md):

- `session_end` — fires on `DomainEvent::SessionEnded` (s02).
- `edge_change` — fires on edge `DomainEvent` variants (s03, s05).
- `periodic` — timer-driven (deferred to M7/s06).
- `explicit` — operator-invoked via API.
- `custom_event` — extensibility hook (M7+).

## Cross-references

- [requirements/admin/13-system-agents-config.md](../../../requirements/admin/13-system-agents-config.md).
- [M5 plan §P6](../../../../plan/build/01710c13-m5-templates-system-agents-sessions.md).
