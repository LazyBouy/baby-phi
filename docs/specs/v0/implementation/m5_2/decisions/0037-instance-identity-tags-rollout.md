<!-- Last verified: 2026-04-28 by Claude Code -->

# ADR-0037 — Instance-identity-tag rollout: 15-type single-migration coverage

**Status: Accepted**

**Date:** 2026-04-28
**Chunk:** CH-06
**Closes:** D-new-11 (MEDIUM) — composite instances don't auto-emit `{kind}:{id}` self-identity tag at creation

---

## Context

[`concepts/permissions/01-resource-ontology.md`](../../../concepts/permissions/01-resource-ontology.md) §"Instance Identity Tags" mandates that every composite instance carries `{kind}:{instance_id}` (e.g. `session:s-9831`, `auth_request:req-7102`, `agent_catalog_entry:<uuid>`) at creation, in addition to the `#kind:{name}` type tag. The `Composite::auto_tags(instance_id)` helper at [`composites.rs:119`](../../../../../../modules/crates/domain/src/model/composites.rs#L119) shipped in M1 with full unit-test coverage but **zero production call sites** — every composite/node creation handler hand-rolled the `#kind:` tag and never emitted the self-identity pair.

Drift D-new-11 was MEDIUM rather than HIGH because the instance-id is derivable from the row's primary key — selectors against instance ids could in principle look at the `id` column directly. But concept-09's grammar `tags contains session:s-9831` matches against the `tags` array, so without emission there is no data for that selector to match.

CH-06 closes this drift in the same chunk as the grammar (D-new-03) because the two halves are physically inseparable: the grammar is the consumer, instance tags are the target data.

## Decision

### D37.1 — Single migration covering all 10 SurrealDB tables

Migration [`0008_instance_identity_tags.surql`](../../../../../../modules/crates/store/migrations/0008_instance_identity_tags.surql) adds `tags ARRAY<string> DEFAULT []` to:

| Table | Type |
|---|---|
| `mcp_server` | `ExternalService` |
| `token_budget_pool` | `TokenBudgetPool` |
| `agent_execution_limits` | `AgentExecutionLimitsOverride` |
| `agent_catalog_entry` | `AgentCatalogEntry` |
| `system_agent_runtime_status` | `SystemAgentRuntimeStatus` |
| `shape_b_pending_projects` | `ShapeBPendingProject` |
| `auth_request` | `AuthRequest` |
| `inbox_object` | `InboxObject` |
| `outbox_object` | `OutboxObject` |
| `session` | `Session` |

`memory.tags` already shipped at migration 0001 — only the production emission path needed wiring (no additional column).

Forward-only migration per ADR-0012 (no down script). `DEFAULT []` ensures pre-migration rows materialise with empty tags; the corresponding struct fields use `#[serde(default)]` so JSON deserialisation tolerates absence.

Alternatives considered:

- **Per-type migrations 0008–0017** — rejected as 10 sequential migrations for the same shape addition is busy work with no incremental safety benefit.
- **No migration; rely on serde-default only** — rejected because SurrealDB's SCHEMAFULL tables would reject INSERTs that name the new field if it's not declared.

### D37.2 — Composite types add `pub tags: Vec<String>` (#[serde(default)])

Every composite + node struct that owns a graph-node ID gains a `pub tags: Vec<String>` field with `#[serde(default)]`. The 11 composite types are:

- `composites_m2`: `ExternalService` (1)
- `composites_m3`: `OrganizationDefaultsSnapshot`, `TokenBudgetPool` (2)
- `composites_m4`: `Objective`, `KeyResult`, `ResourceBoundaries`, `AgentExecutionLimitsOverride` (4)
- `composites_m5`: `SessionDetail`, `ShapeBPendingProject`, `AgentCatalogEntry`, `SystemAgentRuntimeStatus` (4)

Plus 4 nodes:

- `AuthRequest`, `InboxObject`, `OutboxObject`, `Session` (Memory.tags pre-existed).

`SessionDetail` is a query aggregate (not a stored row); its `tags` mirrors `session.tags` so selector consumers see a stable instance-tag set on the aggregate.

### D37.3 — Tag emission wired only at first-class-ID creation paths

Embedded value-objects (`Objective`, `KeyResult`, `ResourceBoundaries`, `OrganizationDefaultsSnapshot`) carry the `tags` field for shape consistency but emission stays empty:

- `Objective.objective_id` and `KeyResult.kr_id` are project-scoped strings (not typed UUIDs); they have no first-class graph-node identity.
- `ResourceBoundaries` has no id at all; it's a configuration value-object on Project.
- `OrganizationDefaultsSnapshot` has no id; it's embedded on Organization.

Production tag emission is wired at the 10 + 1 first-class-ID creation paths:

| Type | Emission site | Kind name |
|---|---|---|
| ExternalService | `server/src/platform/mcp_servers/register.rs` | `external_service` |
| TokenBudgetPool | `composites_m3.rs::TokenBudgetPool::new` | `token_budget_pool` |
| AgentExecutionLimitsOverride | `server/src/platform/agents/{create,update}.rs` | `agent_execution_limits_override` |
| ShapeBPendingProject | `server/src/platform/projects/create.rs` | `shape_b_pending_project` |
| AgentCatalogEntry | `domain/src/events/listeners.rs::AgentCatalogListener` | `agent_catalog_entry` |
| SystemAgentRuntimeStatus | `domain/src/events/listeners.rs::record_system_agent_fire` | `system_agent_runtime_status` |
| AuthRequest | `domain/src/templates/e.rs` + `server/src/{bootstrap/claim,platform/projects/create}.rs` | `auth_request` |
| InboxObject | `server/src/{bootstrap/claim,platform/{orgs,agents,system_agents}/*}.rs` | `inbox` |
| OutboxObject | (same set) | `outbox` |
| Session | `server/src/platform/sessions/launch.rs` + `domain/src/session_recorder.rs` | `session` |
| Memory | (existing M1 emission) | `memory` |

The store-side reader paths (`repo_impl.rs::AuthRequestRow::into_domain`, `repo_impl_m2.rs::McpServerRow::into_domain`) emit the canonical pair when reading rows that arrived with empty `tags` (post-migration old data); subsequent writes preserve them.

### D37.4 — Reserved-namespace write-restriction deferred to CH-07/CH-12

Concept-01 requires that agents and tools cannot create or modify reserved-namespace tags (`#kind:*`, `{kind}:*`, `delegated_from:*`, `derived_from:*`). The publish-time manifest validator that enforces this lands at CH-07 (multi-scope cascade) and CH-12 (frozen-tag enforcement). CH-06 leaves the parser permissive at the read side per concept-09 §"Reserved Namespace Enforcement".

### D37.5 — `auto_tags()` helper signature stays `[String; 2]`

The existing `Composite::auto_tags(&self, id: &str) -> [String; 2]` signature is unchanged; CH-06 adds a free function `auto_tags_for(kind_name: &str, instance_id: &str) -> [String; 2]` for the M3+ struct types that live outside the `Composite` enum (e.g. `AgentCatalogEntry`, `TokenBudgetPool`). Both helpers share the same body — `auto_tags()` delegates to `auto_tags_for(self.kind_name(), instance_id)`.

---

## Consequences

**Positive:**
- Every composite/node creation path emits the canonical `(#kind:<name>, <name>:<id>)` pair.
- Selectors like `tags contains session:s-9831` can match real data.
- `AgentCatalogListener::on_event` upserts preserve existing tags on update; only the first insert emits the canonical pair (idempotent).

**Negative:**
- 14 struct types gained a new field; ~30 struct-literal call sites in tests + production needed updating.
- Embedded value-objects (5 types) carry an unused `tags` field; future cleanup can elide them once an ADR clarifies which embedded types should ever surface as selector targets.

**Mitigations:**
- `#[serde(default)]` keeps wire-format backward compat.
- Acceptance test [`instance_tags_emission.rs`](../../../../../../modules/crates/domain/tests/instance_tags_emission.rs) pins the canonical-pair invariant for every first-class-ID type.

---

## Cross-References

- [`concepts/permissions/01-resource-ontology.md`](../../../concepts/permissions/01-resource-ontology.md) §"Instance Identity Tags"
- [drift D-new-11](../../m5_1/drifts/D-new-11.md)
- ADR-0036 (selector grammar) — sibling chunk decision
- CH-06 plan archive: [`baby-phi/docs/specs/plan/build/acd383e2-ch-06-selector-grammar-peg-and-instance-tags.md`](../../../../plan/build/acd383e2-ch-06-selector-grammar-peg-and-instance-tags.md)
