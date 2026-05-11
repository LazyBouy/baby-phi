<!-- Last verified: 2026-05-11 by Claude Code (CH-24 P-SEAL — ADR Status flipped Proposed → Accepted at chunk-seal. Cycle hex `5778bb77`. Closes drift `D-CH24-recent-sessions-api-flip`. Mid-cycle scope-expansion ratification: F-D59.1 (LIMIT 10 — aligned with planner-recommendation), F-D59.2.b (dedicated repo method — USER-DIVERGENT from planner-recommended reuse path), F-D59.3.b (replace `RecentSessionStub` with `RecentSessionEntry` — USER-DIVERGENT from planner-recommended reuse-stub path; 6-field shape shipped, `started_by_display_name` deferred to follow-up chunk pending Agent-table-join design). Pre-existing-behaviour preservation note per chunk-planner v11 variation (c) Never-shipped-yet. The struct lives in `domain::model::composites_m5` (NOT `server::platform::projects::detail` per the plan's loose wording) — the trait return type cannot reference a server-tier struct; placement is in the canonical view-shape composites tier alongside `SessionDetail` per gate-2 orchestrator-approved deviation. ADR shape follows ADR-0058 (CH-20 convention-confirm-in-place) + ADR-0057 (CH-19 Bucket B ratification) precedents.) -->

# ADR-0059 — Recent-sessions API-surface flip (page-11 panel from M4 placeholder to real query via dedicated repo method + view-shape struct)

**Status: Accepted**

**Date:** 2026-05-11
**Chunk:** CH-24 (mid-cycle scope expansion approved at gate-2 / gate-2.5 user-locks)
**Closes:**
- [`D-CH24-recent-sessions-api-flip`](../../m5_1/drifts/D-CH24-recent-sessions-api-flip.md) (MEDIUM, Bucket B — load-bearing API surface gap). The drift transitions `discovered → remediated` in the same chunk via the four sub-decisions below.

---

## Context

M4's `ProjectDetail.recent_sessions` field at [`server/src/platform/projects/detail.rs`](../../../../../../modules/crates/server/src/platform/projects/detail.rs) was hardcoded to `Vec::new()` at two call-sites (`:229` primary `project_detail` path + `:654` test-side fixture). The inline module doc-comment at `detail.rs:33` promised:

> *"deferred to M5 per D11; the `recent_sessions` field in [`ProjectDetail`] is a placeholder `Vec::new()` until M5 wires baby-phi's governance `Session` node."*

C-M5-3 (the "Session persistence" carryover from M4) closed during M5/P4 against the **persistence half** of the contract — the `Session` / `LoopRecord` / `Turn` rows materialise at session launch + finalisation, verified by re-running the carryover scenarios against real `phi_core::agent_loop` output post-CH-02 (`MockProvider` flip). The **API-surface half** — wiring page-11's `recent_sessions` panel to a real query reading those persisted rows — never landed. The placeholder survived through CH-23 close (M5/P9) and would have shipped with the milestone tag `v0.1-m5` if not surfaced.

CH-24's P-NEW-TESTS authoring surfaced the gap: the new acceptance scenario [`m5_sessions_full_bootstrap_to_first_session_to_extraction_to_catalog`](../../../../../../modules/crates/server/tests/acceptance_m5_sessions.rs) was originally drafted with a **tripwire assertion** at lines 224–230 asserting `recent.is_empty()` — codifying the M4 placeholder as load-bearing rather than as a known gap. At gate-2 the user locked: **close in-chunk**. The original CH-24 v1/v2 plan scope ("verification + new acceptance/e2e + docs rollup + milestone tag") was widened mid-cycle to add a new phase **P-FLIP-RECENT-SESSIONS** wiring the real query + flipping the tripwire to assert `recent.len() == 1`.

This ADR is the **first mid-cycle architectural scope expansion** across all chunks — flagged for retrospective routing as a process-novelty (see chunk-planner v13 candidates in CH-24's plan §13).

### Quality-over-speed restatement

*"Concept docs are source-of-truth; implementation aligns to them. Drift is discovered, documented, and planned-through — never accumulated silently."*

CH-24 application: a milestone-seal cycle is the right moment to surface previously-deferred sub-pieces and close them — the alternative (deferring `recent_sessions` flip to a separate M5.3 cleanup chunk after the milestone tag) leaves a hollow API surface in a tagged release. The user's standing principle dictated close-in-chunk; this ADR ratifies the resulting design choices.

---

## Sub-decisions

The CH-24 plan §5 enumerates 4 sub-decisions §D59.1–§D59.4. Each is captured below verbatim from the plan text + gate-2.5 user-locks.

### §D59.1 — `recent_sessions` cardinality bound: LIMIT 10

The `recent_sessions` panel queries persisted `Session` rows for the project, ordered newest-first by `started_at`, bounded to **LIMIT 10**. The bound is pushed into the query layer (see §D59.2) rather than enforced caller-side — a project with thousands of sessions does NOT materialise the full list in-memory before truncation.

**Locked at gate-2.5: aligned with planner-recommendation F-D59.1.a.**

Rationale:
- Matches typical panel UX (top-10 newest).
- Matches the CLI default `phi project show` page size at [`cli/src/commands/project.rs`](../../../../../../modules/crates/cli/src/commands/project.rs).
- The existing `Repository::list_sessions_in_project` method ([`domain/src/repository.rs:1660`](../../../../../../modules/crates/domain/src/repository.rs)) is documented as *"Ordered newest-first by `started_at`"* — LIMIT 10 is the natural truncation point.
- The live list endpoint at `/api/v0/orgs/:org/projects/:proj/sessions` remains the unbounded surface for operators who need full history.

The constant is wired at [`server/src/platform/projects/detail.rs`](../../../../../../modules/crates/server/src/platform/projects/detail.rs):

```rust
/// Page-11 `recent_sessions` panel cardinality bound per ADR-0059
/// §D59.1. Bumping this constant requires re-running the panel
/// acceptance test (see `acceptance_m5_sessions.rs`).
const RECENT_SESSIONS_LIMIT: u32 = 10;
```

Bumping the constant requires re-running the panel acceptance test (which currently asserts `recent.len() == 1` after a single launch — the assertion is upper-bound-safe).

### §D59.2 — Repo method shape: dedicated `list_recent_sessions_for_project`

Add a new method to the `Repository` trait:

```rust
async fn list_recent_sessions_for_project(
    &self,
    project: ProjectId,
    limit: u32,
) -> RepositoryResult<Vec<RecentSessionEntry>>;
```

Three impl surfaces:
- **`Repository` trait declaration** at [`domain/src/repository.rs`](../../../../../../modules/crates/domain/src/repository.rs).
- **`SurrealStore` impl** at [`store/src/repo_impl.rs`](../../../../../../modules/crates/store/src/repo_impl.rs) — SurrealQL body `SELECT * FROM session WHERE project_id = $project ORDER BY started_at DESC LIMIT $limit`. Cap pushed into the query for scale-resilience; raw `Session` row mapped into the view-shape `RecentSessionEntry` (see §D59.3).
- **`InMemoryRepository` impl** at [`domain/src/in_memory.rs`](../../../../../../modules/crates/domain/src/in_memory.rs) — Vec filter on `project_id` + sort_by descending `started_at` + take(`limit` as usize) + view-shape map. Mirrors the SurrealQL contract for test fixture parity.

**Locked at gate-2.5 (USER-DIVERGENT): F-D59.2.b** — diverges from planner-recommended F-D59.2.a (caller-side `.take(10)` reuse of the existing `list_sessions_in_project` method).

Rationale (user gate-2.5 lock):
- **Query-side cap for scale-resilience** — a project with thousands of sessions doesn't materialise the full list in-memory before truncation.
- **Clearer call-site semantics** — the method name encodes the contract; readers don't have to trace `.take(10).collect()` chains.
- **Explicit DB-tier truncation** matches what a production DBA would expect when reading the query plan.
- **Per chunk-planner v10 wire-mapping-cascade rule**, the cost is bounded — 3 surfaces (trait + SurrealStore impl + InMemoryRepository impl) — and well within the cascade discipline.

Planner-recommendation F-D59.2.a (reuse existing method + caller-side slice) is preserved as historical record in the CH-24 plan §F-D59.2 + §13 retrospective-routing per chunk-planner v9 surfacing-not-suppressing approach. The cross-cycle divergence pattern (CH-15/17/18/20 gate-1 + CH-24 gate-1 + 2-of-3 gate-2.5 = 7-of-9 across cycles) suggests the user's instinct for milestone-seal / load-bearing-surface chunks is consistently to prefer richer / more-defensive paths over amend-don't-add.

### §D59.3 — Repo method return type + struct shape: replace `RecentSessionStub` with `RecentSessionEntry`

Delete the M4 placeholder struct `RecentSessionStub` at `detail.rs:84-95` (3 fields: `session_id: String`, `started_at: DateTime<Utc>`, `summary: String`) and replace with a richer view-shape struct `RecentSessionEntry`. The new struct is **baby-phi-defined** at [`domain::model::composites_m5`](../../../../../../modules/crates/domain/src/model/composites_m5.rs) (NOT a phi-core reuse) so the `Repository` trait can name it directly as its return type — the trait sits in the domain crate; a server-tier struct cannot serve as the return type from the lower-tier trait.

Shipped fields (6 of 7 originally enumerated at gate-2.5):

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecentSessionEntry {
    pub id: SessionId,
    pub project_id: ProjectId,
    pub agent_id: AgentId,
    pub started_at: DateTime<Utc>,
    #[serde(default)]
    pub ended_at: Option<DateTime<Utc>>,
    pub status: String,
}
```

Per-field rationale:
- `id: SessionId` — strongly-typed (replaces the M4 stub's `session_id: String`); maps from `Session.id`.
- `project_id: ProjectId` — denormalised for client-side filtering / multi-project panels; maps from `Session.project_id`.
- `agent_id: AgentId` — explicit launching agent; maps from `Session.agent_id`.
- `started_at: DateTime<Utc>` — preserved from stub; maps from `Session.started_at`.
- `ended_at: Option<DateTime<Utc>>` — `None` while running; maps from `Session.ended_at`. `#[serde(default)]` for backward-compat.
- `status: String` — derived from `Session.governance_state` rendered as a stable enum string (`"running" | "completed" | "aborted" | "failed_launch"` per `SessionGovernanceState::as_str`).

The struct ships with `RecentSessionEntry::from_session(session: &Session) -> Self` (centralised projection helper) so both the in-memory and SurrealDB repository impls produce identical rows for identical inputs.

**Locked at gate-2.5 (USER-DIVERGENT): F-D59.3.b** — diverges from planner-recommended F-D59.3.a (reuse `RecentSessionStub` with real population).

Rationale (user gate-2.5 lock):
- Richer panel UX with explicit named fields (CLI page-11 renderer can format from named fields; future web UI gets typed fields).
- Zero technical-debt-marker — the "Stub" suffix encoded a placeholder semantics that the rename retires.
- Explicit fields beat synthesised `summary` strings for downstream consumers.
- The wire-shape change is contained — the prior state had ZERO real consumers (the panel was always empty), so the rename does NOT break a shipped contract (see §D59.4).

#### §D59.3-FOLLOWUP — `started_by_display_name` deferred to follow-up chunk

The gate-2.5 fork body originally enumerated a 7th field `started_by_display_name: String` derived from the agent or principal that initiated the session (`Session.started_by`). On chunk-implementation open, the implementer surfaced that this field's source-of-truth requires an **Agent-table join** (or a 2nd repository round-trip per row) — neither path is acceptable within CH-24's scope without an N+1-read pause via AskUserQuestion or a batch-fetch redesign.

**Resolution at gate-2 (orchestrator-approved deviation):** ship the 6-field shape NOW; defer `started_by_display_name` to a follow-up chunk that designs the join (either a SurrealQL `FETCH agent_id` clause + struct refit, or a batch helper `Repository::get_agents_for(&[AgentId]) -> Vec<Agent>` paired with caller-side merge). The renderer layer (CLI page-11 + future web UI) can derive display names from `agent_id` on its own via a secondary `GET /api/v0/agents/:id` request until the join lands — same UX, slightly more requests on the wire.

The 6-field shape is **forward-compatible** with the 7th field: adding `started_by_display_name: String` later is an additive field (with `#[serde(default)]`) per the same wrap-pattern convention codified at ADR-0058 §D58.2.

Tracking: filed as a follow-up bullet in the closing drift `D-CH24-recent-sessions-api-flip.md` body; expected to land in an M6 chunk alongside other Agent-join optimisations.

### §D59.4 — Pre-existing-behaviour preservation note (Never-shipped-yet variant)

Per chunk-planner v11 §"Variation (c) Never-shipped-yet" — this ADR ratifies a struct rename + new repo method against pre-existing behaviour that **never shipped with real data**.

> *Pre-existing absence preserved: the `recent_sessions` panel never shipped with real data. The M4 placeholder at `detail.rs:84-95` + `:229,654` (`Vec::new()` hardcode) + the inline doc-comment at `detail.rs:33` (*"deferred to M5 per D11; the recent_sessions field [...] is a placeholder Vec::new() until M5"*) + the non-landing during M5/P4 are the never-shipped pre-existing behaviour. CH-24 ships the first real population. The struct rename from `RecentSessionStub` to `RecentSessionEntry` is a wire-shape change but the prior state had ZERO real consumers (the panel was always empty), so the rename does NOT break a shipped contract — it ratifies the shape at first-real-population time.*

Wire-consumer audit:
- **Server tests** — `wire_shape_strips_phi_core` snapshot test at `detail.rs::tests` re-runs against the new struct; the test asserts absence of phi-core fields, not presence of specific stub fields.
- **CLI page-11 renderer** — not yet wired (M5 ships server + web; CLI page-11 retrofit is M6 scope per `D7.4`).
- **Web page-11 renderer** — the web component at `modules/web/app/(admin)/organizations/[id]/projects/[project_id]/page.tsx` currently bypasses `ProjectDetail.recent_sessions` and calls `listSessionsInProjectApi` separately (the M4 placeholder workaround); the stale TypeScript type `RecentSessionStubWire` at [`modules/web/lib/api/projects.ts:198`](../../../../../../modules/web/lib/api/projects.ts) is unreferenced beyond the `ProjectDetailWire.recent_sessions` field declaration. A small TypeScript-side refit (rename + field-shape update) is queued for the same follow-up chunk that closes `started_by_display_name` per §D59.3-FOLLOWUP — the web renderer will then switch from the separate `listSessionsInProjectApi` call to consuming the `ProjectDetail.recent_sessions` field directly.

---

## Consequences

**Positive:**
- Page-11 `recent_sessions` panel ships with real data at M5 seal — no hollow API surface in the `v0.1-m5` tagged release.
- The new repo method `list_recent_sessions_for_project` is reusable for any future panel/CLI surface that needs top-N session listing (e.g., dashboard org-wide recent activity).
- The `RecentSessionEntry` struct establishes a view-shape pattern at the domain composites tier — future per-page panels can mirror this (e.g., `RecentMemoryEntry`, `RecentDecisionEntry`).
- Query-side LIMIT (vs caller-side slice) is the right shape for scale-resilience; pattern carries forward to M6+ panel implementations.

**Negative:**
- Three new code surfaces (trait + 2 impls) to maintain — but trivially small (~10 LOC each) and exercised by the existing CH-24 acceptance scenario transitively.
- `started_by_display_name` deferral means CLI/web renderers need a secondary Agent-fetch until the follow-up chunk lands — temporary mild UX cost (an extra request per panel row).
- The struct rename `RecentSessionStub` → `RecentSessionEntry` invalidates the stale TypeScript wire-type at `modules/web/lib/api/projects.ts:198`; the web renderer currently bypasses this field so there's no runtime breakage, but the type-side cleanup belongs in the follow-up chunk.

**Neutral:**
- Zero new phi-core imports — the new repo method reads `Session` via the existing `phi_core::session::model::Session` wrap path at `domain/src/model/nodes.rs`; the new struct is baby-phi-defined.
- Zero new migrations — the `session` table already has all needed columns.
- Zero new audit events — `recent_sessions` is a read-side panel; reads are silent.

---

## Cross-references

- **Closes drift:** [`D-CH24-recent-sessions-api-flip`](../../m5_1/drifts/D-CH24-recent-sessions-api-flip.md) (MEDIUM, Bucket B; `discovered → remediated` same-chunk).
- **Closes concept-doc claim:** `phi-core-mapping.md` §"Session list query at API surface" (page-11 panel) — flips status `silent-in-code → honored`.
- **Tripwire flipped:** [`acceptance_m5_sessions.rs:224-230`](../../../../../../modules/crates/server/tests/acceptance_m5_sessions.rs) — `m5_sessions_full_bootstrap_to_first_session_to_extraction_to_catalog` asserts `recent.len() == 1` post-flip.
- **CH-24 plan:** [`build/ch-24-carryover-reverification-m5-seal-5778bb77/plan.md`](../../../../plan/build/ch-24-carryover-reverification-m5-seal-5778bb77/plan.md) §5 (sub-decisions) + §F-D59.1/F-D59.2/F-D59.3 (gate-2.5 fork bodies) + §7 P-FLIP-RECENT-SESSIONS (deliverable enumeration).

### Cross-references to prior ADRs cited as precedent

- **ADR-0029** ([`m5/decisions/0029-session-persistence-and-recorder-wrap.md`](../../m5/decisions/0029-session-persistence-and-recorder-wrap.md)) — Session persistence + SessionRecorder wrap. C-M5-3's persistence half landed against this ADR; ADR-0059's API-surface flip half builds on it.
- **ADR-0034** ([`0034-agent-durable-lifecycle.md`](0034-agent-durable-lifecycle.md)) — CH-01 agent durable lifecycle precedent for `domain::model` field additions with `#[serde(default)]` shielding (mirrored at `RecentSessionEntry.ended_at`).
- **ADR-0058** ([`0058-bucket-c-convention-confirm-in-place.md`](0058-bucket-c-convention-confirm-in-place.md)) — Bucket C confirm-in-place convention. The CH-24 mid-cycle scope-expansion follows the spirit of "confirm-in-place" by ratifying a previously-deferred decision in the current cycle rather than deferring further. §D58.2 wrap-pattern (`#[serde(default)]` shielding for additive fields) carries forward to §D59.3-FOLLOWUP's forward-compat path.

---

## Lifecycle history

- 2026-05-11 — `Proposed` at CH-24 P-DOCS. Drafted alongside drift `D-CH24-recent-sessions-api-flip` flip to `remediated`.
- 2026-05-11 — `Accepted` at CH-24 P-SEAL (this commit). All 4 sub-decisions §D59.1–§D59.4 ratified; drift `D-CH24-recent-sessions-api-flip` terminal at `remediated`; ADR closes the mid-cycle scope expansion approved at gate-2 / gate-2.5 user-locks.
