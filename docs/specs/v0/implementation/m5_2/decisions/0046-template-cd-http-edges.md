<!-- Last verified: 2026-04-30 by Claude Code -->

# ADR-0046 — Template C/D HTTP edge handlers + production-grade trigger emission

**Status: Accepted**

**Date:** 2026-04-30
**Chunk:** CH-23
**Closes:**
- (none — gap was undocumented at plan-open; closed in-chunk per locked Q2)

**Cross-chunk dependencies:**
- Builds on [ADR-0028](../../m4/decisions/0028-domain-event-bus.md) (event-bus fail-safe — durable write before bus emit).
- Builds on ADR-0034 (durable agent lifecycle — handlers reject archived agents).
- Builds on ADR-0042 §D42.3 #5 (forward-only idempotent migration runner).
- Inaugural production writer for the existing `TemplateCFireListener` and `TemplateDFireListener` shipped at M5/P3.

---

## Context

The forward-scope row scoped CH-23 as a 0.5-day verification of Template C/D listener wiring. Plan-time exploration found that the listeners are unreachable from any production code path because their trigger events have no emitters: the `MANAGES` and `HAS_AGENT_SUPERVISOR` edges live as concept-doc entities + listener-side trigger types but neither the `Edge` enum, the `Repository` trait, the SurrealDB schema, nor any HTTP / CLI handler writes them. The unit + property tests prove the listener bodies work in isolation; they do not prove anything end-to-end.

CH-23 closes that gap. It ships the missing edges as first-class graph variants, a migration that adds the `manages` + `has_agent_supervisor` SurrealDB tables, two compound-tx repo methods that mirror the `HasLead` precedent, two HTTP handlers behind the existing CEO-session auth, and the originally-scoped `acceptance_system_flows_s05.rs` with four scenarios. The chunk produces real production-grade coverage instead of theatre.

The user-decided forks at plan-review (2026-04-30) locked five open questions:

1. **Scope** — ship HTTP edge handlers as part of CH-23 (not bus-level only, not deferred). Closes the production gap as the test does.
2. **Drift tracking** — N/A. The HTTP gap closes inside CH-23, no new drift entry needed.
3. **Route shape (Q3)** — Per-relationship REST: `POST /api/v0/orgs/:org_id/agents/:agent_id/manager` + `POST /api/v0/projects/:project_id/agents/:agent_id/supervisor`. The relationship is a sub-resource of the agent; conventional REST; no generic edge concept exposed in the public API.
4. **Edge field carrier (Q4)** — Carry `org` / `project` on the edge: `Edge::Manages { id, from, to, org }` + `Edge::HasAgentSupervisor { id, from, to, project }`. Listener gets the scope directly, no `member_of` lookup at event-emit time.
5. **Repository method shape (Q5)** — Two dedicated methods + typed receipts: `Repository::create_manages_edge -> ManagesEdgeReceipt` + `Repository::create_has_agent_supervisor_edge -> HasAgentSupervisorEdgeReceipt`. Mirrors the `HasLead` precedent at `repository.rs:300`.

---

## Decision

### D46.1 — Two new `Edge` variants land at `domain::model::edges::Edge`

```rust
Manages {
    id: EdgeId,
    from: AgentId,
    to: AgentId,
    org: OrgId,
},
HasAgentSupervisor {
    id: EdgeId,
    from: AgentId,
    to: AgentId,
    project: ProjectId,
},
```

Per locked Q4: `org` and `project` are carried on the edge so the listener's event payload fields can be populated without a follow-up `member_of` lookup at event-emit time. `EDGE_KIND_NAMES` const expands from `[&str; 69]` to `[&str; 71]`; `Edge::name()` match arm + the M3 model-counts integration test at `m3_model_counts.rs` track the new count.

### D46.2 — Two new `Repository` trait methods + typed receipts

At `domain::repository`:

```rust
async fn create_manages_edge(
    &self,
    org: OrgId,
    manager: AgentId,
    subordinate: AgentId,
    actor: AgentId,
    at: chrono::DateTime<chrono::Utc>,
) -> RepositoryResult<ManagesEdgeReceipt>;

async fn create_has_agent_supervisor_edge(
    &self,
    project: ProjectId,
    supervisor: AgentId,
    supervisee: AgentId,
    actor: AgentId,
    at: chrono::DateTime<chrono::Utc>,
) -> RepositoryResult<HasAgentSupervisorEdgeReceipt>;
```

Receipt structs carry `created: bool`, `edge_id: EdgeId`, `audit_event_id: Option<AuditEventId>`. `created=true` means a fresh edge was written + the audit event id is `Some(_)`; `created=false` means the (scope, from, to) triple already had a row and the existing edge id is returned (idempotent re-POST). Both methods follow the compound-tx-receipt pattern from `HasLead` per locked Q5.

### D46.3 — Migration 0011 adds `manages` + `has_agent_supervisor` tables

`modules/crates/store/migrations/0011_manages_supervisor_edges.surql` defines two SCHEMAFULL tables:

- `manages` — fields `edge_id`, `org_id`, `manager`, `subordinate`, `created_at`. UNIQUE index on `(org_id, manager, subordinate)`.
- `has_agent_supervisor` — fields `edge_id`, `project_id`, `supervisor`, `supervisee`, `created_at`. UNIQUE index on `(project_id, supervisor, supervisee)`.

The `edge_id` body field is a redundant carrier of the SurrealDB record id, used so the idempotency probe can read the canonical UUID via `SELECT edge_id FROM manages ...` without parsing a `Thing`. Registered in `EMBEDDED_MIGRATIONS` at version 11, slug `manages_supervisor_edges`. The migrations integration test asserts row count = 11 + the version-11 entry.

### D46.4 — Two HTTP handlers + per-relationship REST routes

Routes registered at the workspace router:

- `POST /api/v0/orgs/:org_id/agents/:agent_id/manager` — body `{manager_agent_id: AgentId}`. Handler at `server/src/handlers/agents.rs::set_manager`; orchestrator at `server/src/platform/agents/manager.rs::set_agent_manager`.
- `POST /api/v0/projects/:project_id/agents/:agent_id/supervisor` — body `{supervisor_agent_id: AgentId}`. Handler at `server/src/handlers/projects.rs::set_supervisor`; orchestrator at `server/src/platform/projects/agent_supervisor.rs::set_agent_supervisor`.

Both behind existing CEO-session auth (`AuthenticatedSession`). Per locked Q3 — per-relationship REST resource shape, not a generic `/edges` endpoint.

### D46.5 — Handler emit pattern mirrors `HasLeadEdgeCreated`

Both orchestrators call the repo compound-tx first (which writes the edge + the `platform.manages.edge.created` / `platform.has_agent_supervisor.edge.created` audit event atomically), THEN emit `DomainEvent::ManagesEdgeCreated` / `DomainEvent::HasAgentSupervisorEdgeCreated` on the in-process bus. Mirrors the `projects/create.rs:417` HasLead post-commit pattern. Per ADR-0028: durable write before bus emit; listener errors do not affect the durable state.

The new audit-event builders ship at `domain::audit::events::m5::edges` — `Logged` class to match the existing `template.c.grant_fired` / `template.d.grant_fired` precedent (routine traffic; `Alerted` would flood the audit chain on every relationship change).

### D46.6 — Validation rules + idempotency

Repo-level validation enforced inside both compound-tx methods:

- Both agents must exist + have `active = true` + `archived_at == None` (CH-01 / ADR-0034 invariant).
- `manager == subordinate` / `supervisor == supervisee` rejects with `RepositoryError::InvalidArgument` (no self-loop).
- Same-scope check: agents must have `owning_org == Some(org)` for Manages; for HasAgentSupervisor, both agents must belong to one of the project's owning orgs (resolved via the `belongs_to` relation).
- Idempotency: a re-POST of the same triple returns `created = false` + the existing edge id. The handler maps `created = true` → 201, `created = false` → 200, and suppresses the bus emit on the `false` branch so Templates C/D don't double-fire.

Handler-level error mapping: `Validation` → 400 `MANAGER_INVALID` / `SUPERVISOR_INVALID`; `AgentInactive` → 409 `AGENT_INACTIVE`; `Repository` → 500.

### D46.7 — Acceptance suite at `acceptance_system_flows_s05.rs`

Five scenarios:

1. **Template C end-to-end on MANAGES** — POST manager → edge row + `template.c.grant_fired` audit + grant on `agent:<subordinate>`.
2. **Idempotent re-POST does not double-fire Template C** — second POST returns 200, no second grant.
3. **Template D end-to-end on HAS_AGENT_SUPERVISOR** — POST supervisor → edge row + `template.d.grant_fired` audit + grant on `project:<p>/agent:<supervisee>`.
4. **A + C + D simultaneous** — project lead with manager + supervisor relationships triggers all three listeners; all three grants exist + each audit fires exactly once.
5. **Cross-listener subscription order** — two `OrderRecordingListener` instances bracketing a `TemplateCFireListener` subscription confirm the `InProcessEventBus` dispatches in subscription-registration order; an atomic counter records the observed positions.

### D46.8 — No CLI / Web UI surface in CH-23

Per locked Q3 and the M5 plan: CLI org-mutation lives in M6+, web UI for relationship-setting follows. CH-23 stays on the HTTP API only.

### D46.9 — No DELETE endpoints in CH-23

Un-assigning a manager / supervisor is a separate operation. The forward direction closes the production gap that motivates CH-23; revocation will land alongside whichever future chunk owns the wider org-mutation surface.

### D46.10 — Same-scope only

Same-org membership for Manages, same-project membership for HasAgentSupervisor. Tighter cross-scope contractor rules (D-new-20) remain out of scope at v0.

---

## Consequences

**Positive:**
- Templates C and D are now reachable from production code paths.
- The `acceptance_system_flows_s05.rs` suite proves the trigger → listener → grant chain end-to-end against the real SurrealDB backend, not a mock.
- The repo-trait surface gains a clean per-edge compound-tx pattern future chunks can reuse for additional edge types.
- `Edge::Manages` / `Edge::HasAgentSupervisor` carrier fields (`org` / `project`) keep the listener path lookup-free.

**Negative:**
- The `EDGE_KIND_NAMES` constant churn (69 → 71) needs a coordinated update across two test files (`edges.rs` + `m3_model_counts.rs` + `model/mod.rs`).
- A redundant `edge_id` body field in the SurrealDB rows trades one field of storage per row for a probe-side win (no `Thing` parsing).

**Neutral:**
- Migration 0011 is forward-only (per ADR-0012); no down script.
- The `provenance_auth_request_id` field on the new audit events is `None` because edge creation is not adoption-AR-gated (it's an admin-driven mutation, not a Template fire).

---

## Cross-references

- Concept doc: [`permissions/07-templates-and-tools.md`](../../../concepts/permissions/07-templates-and-tools.md) §"Template C / D".
- Concept doc: [`ontology.md`](../../../concepts/ontology.md) §"Edges" — MANAGES + HAS_AGENT_SUPERVISOR are now first-class.
- Production-grade pattern reference: [ADR-0028](../../m4/decisions/0028-domain-event-bus.md) (in-process domain event bus + fail-safe semantics).
- Durable lifecycle invariant: [ADR-0034](0034-agent-durable-lifecycle.md).
- Migration runner conforming criteria: [ADR-0042](0042-storage-backend-configurable.md) §D42.3 #5.
- Action enum dependency: [ADR-0043](0043-typed-action-vocabulary.md) (Template C/D grants use typed `Action::Read` + `Action::Inspect`).
- Listener bodies: `domain::events::listeners::TemplateCFireListener` + `TemplateDFireListener` (M5/P3 — unchanged at this chunk).

---

## Verification

- Workspace tests: `cargo test --workspace -- --test-threads=1` green at **1223 / 0 failed** (1198 baseline + 25 new tests).
- Clippy under `RUSTFLAGS="-Dwarnings"`: clean.
- 4 CI guards green: `check-doc-links.sh`, `check-ops-doc-headers.sh`, `check-phi-core-reuse.sh`, `check-spec-drift.sh`.
- Positive greps:
  - `Edge::Manages \\b` (1) + `Edge::HasAgentSupervisor\\b` (1).
  - `create_manages_edge` (≥ 1) + `create_has_agent_supervisor_edge` (≥ 1) on the trait.
  - `migrations/0011_manages_supervisor_edges.surql` exists.
  - `server/tests/acceptance_system_flows_s05.rs` exists.
  - `EDGE_KIND_NAMES: [&str; 71]`.
- Carry-forward green: CH-04 matrix (28/28), CH-05 manifest validator acceptance (9/9), CH-21 memory extraction acceptance (7/7).
