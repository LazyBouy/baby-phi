<!-- Last verified: 2026-04-27 by Claude Code -->

# ADR-0035 — AgentCatalogListener audit-mode (silent default + debug opt-in) + production emit-site wiring

**Status: Accepted**
**Decided at:** CH-22 chunk-seal (P3), 2026-04-27
**Chunk plan:** [`build/c5f201bb-ch-22-agent-catalog-listener-body.md`](../../../../plan/build/c5f201bb-ch-22-agent-catalog-listener-body.md)
**Closes drifts (partial):** [D6.1](../../m5_1/drifts/D6.1.md) — second call site only; first call site (memory-extraction listener) ships at CH-21
**Concepts touched:** [`system-agents.md`](../../../concepts/system-agents.md) §Agent Catalog Agent

## Context

CH-22's [`AgentCatalogListener`](../../../../../../modules/crates/domain/src/events/listeners.rs) subscribes to 8 `DomainEvent` variants and fires up to **N session-launches × 2 (start + end)** per agent per day. On a busy org with 100 agents and 50 daily sessions that is roughly 10,000 fires/day. Emitting an audit event on every fire would 10–100× audit-log volume without governance benefit — catalog refresh is observability data, not permission-relevant.

A second concern surfaced during P3 implementation. The plan's acceptance test was written against `POST /api/v0/orgs/:org/agents` (and the archive endpoint) but **none of the production handlers emitted the trigger events the listener subscribes to**:

- [`agents/create.rs`](../../../../../../modules/crates/server/src/platform/agents/create.rs) — no `AgentCreated` emit
- [`agents/update.rs`](../../../../../../modules/crates/server/src/platform/agents/update.rs) — no `HasProfileEdgeChanged` emit
- [`system_agents/add.rs`](../../../../../../modules/crates/server/src/platform/system_agents/add.rs) — no `AgentCreated` emit
- [`system_agents/disable.rs`](../../../../../../modules/crates/server/src/platform/system_agents/disable.rs) — no `AgentArchived` emit
- [`system_agents/archive.rs`](../../../../../../modules/crates/server/src/platform/system_agents/archive.rs) — no `AgentArchived` emit
- [`orgs/create.rs`](../../../../../../modules/crates/server/src/platform/orgs/create.rs) — no `AgentCreated` emit for CEO + 2 system agents

This was structurally identical to drift D6.1 (helper shipped, no call sites) and would have left the listener body as dead code from production's perspective. CH-22 expanded P3 scope to wire all six emit sites — the listener body now fires on every production agent-lifecycle path.

## Decision

### D35.1 — `CatalogAuditMode` enum (silent / debug)

`domain::events::CatalogAuditMode` is a `Copy` enum with two variants:

```rust
pub enum CatalogAuditMode {
    Silent,  // default — no audit emission
    Debug,   // emit `agent_catalog_refreshed` per fire
}
```

Lives in the `domain` crate (alongside the listener) so neither the trait nor the listener depends on the `server` crate. The `server` crate's config block re-exports / re-uses the enum.

### D35.2 — Per-listener config block at `[listeners.catalog]`

`config/default.toml` ships:

```toml
[listeners.catalog]
audit_mode = "silent"
```

Override via `PHI_LISTENERS__CATALOG__AUDIT_MODE=debug` (env-var precedence beats TOML, per the existing `serde_with_prefix("PHI")` machinery in [`server/src/config.rs`](../../../../../../modules/crates/server/src/config.rs)). The structure (`listeners.<name>.audit_mode`) is shaped so future listeners (memory-extraction at CH-21, etc.) slot in with `[listeners.memory_extraction]` etc. without a top-level rename.

### D35.3 — Debug mode emits `agent_catalog_refreshed` audit events with `AuditClass::Silent`

The plan called for `AuditClass::Lite`. That variant does not exist in the codebase ([`audit/mod.rs`](../../../../../../modules/crates/domain/src/audit/mod.rs) defines `Silent / Logged / Alerted` only). Adding a new tier would touch the retention contract, the routing matrix, and `nfr-observability.md` — out of CH-22's scope. The CH-22 implementation uses **`AuditClass::Silent`** (30-day retention, no delivery) instead, which captures the plan's intent: minimal retention for debug-only events. The per-listener `audit_mode` flag is the primary gate; the per-event class keeps the trail in the lowest retention tier so even on an org where the flag is left in `debug` the retention overhead is bounded.

The audit-event builder at [`audit/events/m5/agent_catalog.rs`](../../../../../../modules/crates/domain/src/audit/events/m5/agent_catalog.rs) carries:

- `event_type`: `"agent_catalog_refreshed"`
- `actor_agent_id`: the catalog system agent's id (the actor of the refresh)
- `target_entity_id`: the refreshed agent's id (cast to `NodeId`)
- `org_scope`: the agent's owning org
- `audit_class`: `AuditClass::Silent`
- `provenance_auth_request_id`: `None` (reactive listener; no AR triggers)
- `diff.after`: `{refreshed_agent_id, triggering_event_id, triggering_event_kind}` — joins the listener fire to the originating audit row.

### D35.4 — Conforming criteria for future high-volume listeners

Any listener that fires **≥ 1× per session** SHOULD adopt the silent-default pattern unless governance-significant (e.g., grant-mint listeners always audit). Inverting this — defaulting high-volume listeners to logged audit — is reserved for cases where compliance or operator-triage demands per-fire reconstruction.

### D35.5 — Production emit-site wiring (six handlers)

CH-22 P3 wires `event_bus.emit(...)` calls into the following handlers, each AFTER the durable commit + audit emit (per ADR-0028 fail-safe semantics — listener faults never invalidate persisted state):

| Handler | Emit |
|---|---|
| [`agents/create.rs`](../../../../../../modules/crates/server/src/platform/agents/create.rs) | `AgentCreated` (Human / LLM members) |
| [`agents/update.rs`](../../../../../../modules/crates/server/src/platform/agents/update.rs) | `HasProfileEdgeChanged` — only when profile row mutated (display_name-only / limits-only edits skip) |
| [`system_agents/add.rs`](../../../../../../modules/crates/server/src/platform/system_agents/add.rs) | `AgentCreated` (custom system agents) |
| [`system_agents/disable.rs`](../../../../../../modules/crates/server/src/platform/system_agents/disable.rs) | `AgentArchived` (variant doc broadly covers "soft-deleted / disabled") |
| [`system_agents/archive.rs`](../../../../../../modules/crates/server/src/platform/system_agents/archive.rs) | `AgentArchived` |
| [`orgs/create.rs`](../../../../../../modules/crates/server/src/platform/orgs/create.rs) | `AgentCreated` × 3 (CEO + memory-extractor + agent-catalog) |

Each orchestrator gains an `event_bus: Arc<dyn EventBus>` parameter (threaded from `state.event_bus` at the HTTP layer). The harness `Acceptance` struct now exposes the same `event_bus` so acceptance tests can subscribe listeners post-spawn — existing tests are unaffected because the default bus has no subscribers.

### D35.6 — Disable + archive both emit `AgentArchived`

The variant doc on `DomainEvent::AgentArchived` says: *"Emitted when an agent is archived (soft-deleted / disabled). The catalog listener flips the entry's `active` flag."* — explicitly covering both lifecycle paths. The listener computes `catalog_active = agent.active && agent.archived_at.is_none()`; whether disable (flips `active = false`) or archive (sets `archived_at = Some`) triggered the emit, the listener reads the durable agent row and computes the correct catalog state. No new variant is introduced.

## Ratification evidence

| Sub-decision | Code location(s) |
|---|---|
| D35.1 enum | [`listeners.rs::CatalogAuditMode`](../../../../../../modules/crates/domain/src/events/listeners.rs) |
| D35.2 config | [`server/src/config.rs::ListenersConfig + ListenerCatalogConfig`](../../../../../../modules/crates/server/src/config.rs); [`config/default.toml [listeners.catalog]`](../../../../../../config/default.toml) |
| D35.3 audit builder | [`audit/events/m5/agent_catalog.rs::agent_catalog_refreshed`](../../../../../../modules/crates/domain/src/audit/events/m5/agent_catalog.rs) |
| D35.5 emit sites | 6 handler files listed above; HTTP handlers at [`handlers/agents.rs`](../../../../../../modules/crates/server/src/handlers/agents.rs), [`handlers/system_agents.rs`](../../../../../../modules/crates/server/src/handlers/system_agents.rs), [`handlers/orgs.rs`](../../../../../../modules/crates/server/src/handlers/orgs.rs) thread `state.event_bus` |
| Tests | 15 unit tests in [`listeners.rs::tests`](../../../../../../modules/crates/domain/src/events/listeners.rs) (covering all 8 variants + audit_mode + D34.5 conforming) + 2 acceptance scenarios in [`acceptance_system_flows_s03.rs`](../../../../../../modules/crates/server/tests/acceptance_system_flows_s03.rs) |

## Consequences

**Positive.** Audit log stays lean in production; debug-mode opt-in retains full reconstruction power for dev + acceptance. The six emit sites turn the listener from "dead code with subscriptions" into a fully-wired vertical slice. CH-22's catalog row now reflects every agent-lifecycle event from production handlers.

**Negative.** Debug mode's `agent_catalog_refreshed` schema becomes a stable contract — future schema changes need an ADR. Six new emit sites are six new ADR-0028 fail-safe boundaries the operator must understand (a listener fault on any of the six leaves the durable state correct but the catalog stale; M7b retry fabric closes the gap).

**Neutral.** Per-listener config feels like the first instance of a "listener-local config" pattern. The same shape extends to memory-extraction (CH-21) cleanly.

## Alternatives considered

- **Tracing-level gate.** Conflates audit chain with log level — rejected. Audit + tracing are intentionally orthogonal surfaces (`baby-phi/CLAUDE.md` §Orthogonal surfaces).
- **Always-emit (no flag).** Log volume blowup on busy orgs — rejected.
- **Sampling rate (e.g. 1%).** Over-engineered for M5 scale — rejected.
- **New `AuditClass::Lite` tier.** Out of CH-22 scope (touches retention contract + nfr-observability.md). May revisit at M7b if telemetry-tier audit becomes a first-class concept.

## Review trigger

Revisit if (a) M6's catalog query API needs richer audit reconstruction for compliance, (b) M7b production observability requires per-fire audit for catalog refresh, or (c) a new high-volume listener wants a different default than silent — at which point we generalise this into a platform-level convention rather than a per-listener choice.
