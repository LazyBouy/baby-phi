<!-- Last verified: 2026-05-11 by Claude Code (CH-24 P-DOCS — NEW §"Page-11 `recent_sessions` panel semantics" subsection added per plan §7 P-DOCS deliverable 11 + ADR-0059 §D59.1–§D59.4. Documents cardinality bound (LIMIT 10 query-side, not caller-side), ordering (newest-first by `started_at`), freshness (on-read; no caching at v0), dedicated Repository method `list_recent_sessions_for_project`, view-shape `RecentSessionEntry` (baby-phi-defined at `domain::model::composites_m5`). Cycle hex `5778bb77`.) -->
<!-- Last verified: 2026-04-23 by Claude Code -->

# M5 architecture overview

**Status**: [PLANNED M5/P1] — stub seeded at M5/P0; filled as each
surface lands. See the [plan archive](../../../../plan/build/m5-templates-system-agents-sessions-01710c13.md)
for the full P0–P9 scope.

M5 layers three verticals on the M4 foundation:

- **Session persistence** — governance `Session` / `LoopRecordNode`
  / `TurnNode` wrapping the three `phi_core::session::model::*`
  types per [ADR-0029](../decisions/0029-session-persistence-and-recorder-wrap.md).
- **Authority template adoption** — pages 12 drives approve /
  deny / adopt-inline / revoke-cascade across Templates A–E.
  Migration 0005 flips UNIQUE(name) → UNIQUE(kind) per
  [ADR-0030](../decisions/0030-template-node-uniqueness.md).
- **First session launch** — page 14 wires
  `phi_core::agent_loop` with cancellation + bounded concurrency
  per [ADR-0031](../decisions/0031-session-cancellation-and-concurrency.md).

Plus three reactive supervisors — `memory-extraction-agent` (s02),
`agent-catalog-agent` (s03), and the extended Template-A fire
listener (s05).

## Page-11 `recent_sessions` panel semantics (CH-24 close)

Page 11 (project detail) carries a `recent_sessions` panel showing the project's most recently launched sessions. CH-24 (cycle hex `5778bb77`) closes the API-surface flip per [ADR-0059](../../m5_2/decisions/0059-recent-sessions-api-surface-flip.md) §D59.1–§D59.4 (mid-cycle scope expansion approved at gate-2 user-lock; the M4 placeholder `recent_sessions: Vec::new()` at `detail.rs:229` was never flipped during M5/P4):

- **Cardinality bound — LIMIT 10**, pushed into the query layer (NOT caller-side). The constant `RECENT_SESSIONS_LIMIT: u32 = 10` lives at [`server/src/platform/projects/detail.rs`](../../../../../../modules/crates/server/src/platform/projects/detail.rs); bumping it requires re-running the panel acceptance test. The unbounded surface is the live-list endpoint `GET /api/v0/orgs/:org/projects/:proj/sessions`.
- **Ordering — newest-first by `started_at`**, descending. Mirrors the predecessor [`Repository::list_sessions_in_project`](../../../../../../modules/crates/domain/src/repository.rs) method's documented contract.
- **Freshness — on-read.** Every `GET /api/v0/projects/:id` request issues a fresh query; no in-process caching at v0. At M7b (multi-pod / multi-replica deployment) the panel may layer a Redis cache invalidation pattern (see [`m7b/architecture/deferred-from-ch-k8s-prep.md`](../../m7b/architecture/deferred-from-ch-k8s-prep.md)), but the v0 contract is direct DB read.
- **Dedicated Repository method** — [`Repository::list_recent_sessions_for_project(project_id: ProjectId, limit: u32) -> RepositoryResult<Vec<RecentSessionEntry>>`](../../../../../../modules/crates/domain/src/repository.rs). SurrealStore impl uses `SELECT * FROM session WHERE project_id = $project ORDER BY started_at DESC LIMIT $limit`; InMemoryRepository impl mirrors via Vec filter + sort + take. Per ADR-0059 §D59.2 the cap is pushed into the query for scale-resilience (a project with thousands of sessions does NOT materialise the full list in-memory).
- **View-shape struct — [`RecentSessionEntry`](../../../../../../modules/crates/domain/src/model/composites_m5.rs)** (6 fields: `id`, `project_id`, `agent_id`, `started_at`, `ended_at`, `status`). Baby-phi-defined at the domain composites tier so the `Repository` trait can name it directly — NOT a phi-core reuse (the underlying `Session` IS a phi-core reuse via the existing wrap at `domain/src/model/nodes.rs` per [ADR-0029](../decisions/0029-session-persistence-and-recorder-wrap.md)). The 7th field `started_by_display_name` is deferred to a follow-up M6 chunk per ADR-0059 §D59.3-FOLLOWUP (Agent-table join needed).
- **Cross-org isolation invariant** — Org-A viewer cannot see Org-B's recent sessions (enforced by the project's owning-org boundary at the upstream handler). Pinned by acceptance test `m5_cross_org_isolation_at_session_surface` at [`acceptance_m5_sessions.rs`](../../../../../../modules/crates/server/tests/acceptance_m5_sessions.rs).

For the full operator + troubleshooting flow see [m5/operations/m5-ops-runbook.md §IP-4](../operations/m5-ops-runbook.md) and [m5/user-guide/troubleshooting.md §CH-24 amendment](../user-guide/troubleshooting.md).

## Commitment ledger

See the plan archive §Part 2 for the 27-commitment ledger. As each
phase closes, the corresponding commitment row flips to ✅ in the
[README phase status table](../README.md#phase-status).

## Cross-references

- [m4-postflight-delta.md](m4-postflight-delta.md) — the P0 verification audit.
- [phi-core-reuse-map.md](phi-core-reuse-map.md) — per-page leverage table.
- [session-persistence.md](session-persistence.md) — 3-wrap pattern.
- [session-launch.md](session-launch.md) — page 14 flow.
- [authority-templates.md](authority-templates.md) — page 12 flow.
- [system-agents.md](system-agents.md) — page 13 flow.
- [shape-b-materialisation.md](shape-b-materialisation.md) — C-M5-6 pre/post.
- [event-bus-m5-extensions.md](event-bus-m5-extensions.md) — 8 new `DomainEvent` variants.
- [ADR-0059 — recent-sessions API-surface flip](../../m5_2/decisions/0059-recent-sessions-api-surface-flip.md) — CH-24 panel flip with 4 sub-decisions.
- [m5/operations/m5-ops-runbook.md](../operations/m5-ops-runbook.md) — milestone-aggregate ops runbook.
