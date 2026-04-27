<!-- Last verified: 2026-04-27 by Claude Code -->
<!-- CH-01 amendment (2026-04-27): disable + archive handlers now flip durable agent-row state (`active = false` on disable; `archived_at = Some(now)` on archive). See §"CH-01 amendment" below + ADR-0034. -->
<!-- CH-22 amendment (2026-04-27): AgentCatalogListener body shipped — see §"CH-22 amendment" below. Six production emit sites wired (ADR-0035 §D35.5). -->

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

## CH-01 amendment — durable lifecycle columns (2026-04-27)

Migration [`0007_agent_active_archived.surql`](../../../../../../modules/crates/store/migrations/0007_agent_active_archived.surql) adds two durable columns to the `agent` row:

- `active: bool DEFAULT true` — flipped to `false` by `system_agents/disable.rs`.
- `archived_at: option<string>` (RFC3339 datetime) — set by `system_agents/archive.rs` to `Some(Utc::now())` on archive.

Repo-trait additions (both backends — [`SurrealStore`](../../../../../../modules/crates/store/src/repo_impl.rs) + [`InMemoryRepository`](../../../../../../modules/crates/domain/src/in_memory.rs) implement):

```rust
async fn set_agent_active(&self, agent_id: AgentId, active: bool) -> RepositoryResult<()>;
async fn set_agent_archived_at(&self, agent_id: AgentId, archived_at: Option<DateTime<Utc>>) -> RepositoryResult<()>;
```

Both return `RepositoryError::NotFound` if the agent row is missing — explicit existence check because SurrealDB's bare `UPDATE` on a missing record silently returns empty.

**Ordering rule (ADR-0034 §D34.4):** `disable.rs` and `archive.rs` flip durable state BEFORE emitting their audit event. Durable state is authoritative; audit is replayable. If audit emit fails after a successful durable flip, the persisted state is still correct — operators re-derive the missing audit row by replaying. The reverse order would leave the audit log "ahead" of reality.

Tests: [`store/tests/repo_agent_lifecycle_test.rs`](../../../../../../modules/crates/store/tests/repo_agent_lifecycle_test.rs) (cross-backend contract); [`acceptance_system_agents.rs::archive_with_confirm_succeeds_and_flips_durable_archived_at`](../../../../../../modules/crates/server/tests/acceptance_system_agents.rs).

## CH-22 amendment — AgentCatalogListener body shipped (2026-04-27)

The M5/P3 stub at [`listeners.rs`](../../../../../../modules/crates/domain/src/events/listeners.rs) is replaced by a full body:

1. **Trigger set unchanged** — same 8 `DomainEvent` variants (`AgentCreated`, `AgentArchived`, `HasProfileEdgeChanged`, `HasLeadEdgeCreated`, `ManagesEdgeCreated`, `HasAgentSupervisorEdgeCreated`, `SessionStarted`, `SessionEnded`). `SessionAborted` stays a documented no-op.
2. **Per-fire mutation** — body reads the durable `Agent` row via `repo.get_agent(agent_id)`, computes `catalog_active = agent.active && agent.archived_at.is_none()` (ADR-0034 §D34.5 archive-wins-ties), and upserts the `agent_catalog_entry` row.
3. **D6.1 second call site** — body calls [`record_system_agent_fire`](../../../../../../modules/crates/domain/src/events/listeners.rs) AFTER the catalog upsert, with the catalog system agent's id as target. The runtime-status tile for the catalog system agent advances on every fire. CH-21 (memory-extraction listener) ships the first call site for this drift.
4. **Audit mode (ADR-0035)** — listener gains a `CatalogAuditMode` enum (Silent / Debug). Default is Silent — production audit logs stay lean. Operators flip to Debug via `[listeners.catalog] audit_mode = "debug"` (TOML) or `PHI_LISTENERS__CATALOG__AUDIT_MODE=debug` (env). Debug mode emits one `agent_catalog_refreshed` audit event per fire, audit-class `Silent` (30-day retention).

**Production emit sites wired (ADR-0035 §D35.5).** Without these, the listener body would have been dead code from the production HTTP path's perspective:

| Handler | Emit |
|---|---|
| [`agents/create.rs`](../../../../../../modules/crates/server/src/platform/agents/create.rs) | `AgentCreated` |
| [`agents/update.rs`](../../../../../../modules/crates/server/src/platform/agents/update.rs) | `HasProfileEdgeChanged` (only when profile row mutated) |
| [`system_agents/add.rs`](../../../../../../modules/crates/server/src/platform/system_agents/add.rs) | `AgentCreated` |
| [`system_agents/disable.rs`](../../../../../../modules/crates/server/src/platform/system_agents/disable.rs) | `AgentArchived` (variant doc broadly covers "soft-deleted / disabled") |
| [`system_agents/archive.rs`](../../../../../../modules/crates/server/src/platform/system_agents/archive.rs) | `AgentArchived` |
| [`orgs/create.rs`](../../../../../../modules/crates/server/src/platform/orgs/create.rs) | `AgentCreated` × 3 (CEO + memory-extractor + agent-catalog) |

All 6 emits run AFTER the durable commit + audit emit (ADR-0028 fail-safe order). The 4 HTTP handlers thread `state.event_bus.clone()` into the orchestrators they call.

**Tests:** 15 unit tests in [`listeners.rs::tests`](../../../../../../modules/crates/domain/src/events/listeners.rs) covering all 8 variants + audit-mode flag + ADR-0034 §D34.5 conforming criteria; 2 acceptance scenarios in [`acceptance_system_flows_s03.rs`](../../../../../../modules/crates/server/tests/acceptance_system_flows_s03.rs).

## Cross-references

- [requirements/admin/13-system-agents-config.md](../../../requirements/admin/13-system-agents-config.md).
- [Event bus M5 extensions](./event-bus-m5-extensions.md) — `DomainEvent::SessionEnded` + edge variants drive the listener fires the runtime-status tiles upsert on.
- [ADR-0023](../../m3/decisions/0023-system-agents-inherit-from-org-snapshot.md) — organization defaults pattern drives trigger effective-parallelize resolution.
- [ADR-0034](../../m5_2/decisions/0034-agent-durable-lifecycle.md) — durable lifecycle columns + governance/runtime boundary (CH-01).
- [ADR-0035](../../m5_2/decisions/0035-agent-catalog-listener-audit-mode.md) — audit-mode + production emit-site wiring (CH-22).
- [M5 plan §P6](../../../../plan/build/01710c13-m5-templates-system-agents-sessions.md).
