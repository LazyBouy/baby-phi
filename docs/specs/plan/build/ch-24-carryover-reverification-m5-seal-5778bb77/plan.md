<!-- Last verified: 2026-05-11 by Claude Code (chunk-planner v12; CH-24 — Carryover re-verification + M5 final seal; cycle hex `5778bb77`; v2 re-plan after gate-1 user-lock divergent F1.B (5 per-page files) from planner-recommended F1.A (single file); **v3 re-plan: mid-cycle scope expansion approved at gate-2 (user-locked close-in-chunk for C-M5-3 API-surface flip; recent_sessions placeholder wired to real query)**; **v4 re-plan: gate-2.5 user-lock 2-of-3 DIVERGENT from planner-recommendations (F-D59.2 dedicated repo method + F-D59.3 richer RecentSessionEntry struct chosen over reuse-existing paths)**) -->

# CH-24 — Carryover re-verification + M5 final seal

**Plan file token:** `5778bb77` (generated 2026-05-11 at chunk-open via `openssl rand -hex 4`).
**Plan archive path:** `baby-phi/docs/specs/plan/build/ch-24-carryover-reverification-m5-seal-5778bb77/plan.md`.
**Chunk ID:** CH-24.
**Chunk shape:** **Milestone-seal chunk** — verification + new acceptance/e2e + docs rollup + milestone tag. **v3 update:** scope-expanded mid-cycle to also flip the `recent_sessions` C-M5-3 API-surface placeholder to a real query (1 drift closed, 1 ADR drafted). **v4 update:** gate-2.5 user-lock chose dedicated repo method + richer struct shape (replacement, not reuse).
**Severity:** ⚠ HIGH (the chunk seals M5; failure here surfaces hollow carryover closure or doc-staleness at the wrong moment — at milestone tag).
**Expected effort:** ~2 engineer-days (forward-scope line 210) + ~0.3 engineer-days for the v3 scope-expansion + ~0.1 engineer-days for the v4 dedicated-method cascade = **~2.4 engineer-days**.
**Hard prerequisites:** CH-01 → CH-02 → CH-06 → CH-15 → CH-16 → CH-17 → CH-18 → CH-19 → CH-20 → CH-21 → CH-22 → CH-23. All sealed. Confirmed via `git -C /root/projects/phi/baby-phi log --oneline | grep -E '^[a-f0-9]+ ch-(2[0-3]|1[5-9])' | head`.
**Chunks unblocked at close:** CH-25 (Agent-as-creator-and-owner of Org/Project — philosophy §4.1), CH-26 (Org/Project as Composite — philosophy §4.2). Both M5.3 carve-out chunks list CH-24 as their hard prereq (forward-scope line 244).

---

## Forks for orchestrator

> ⚠ **CROSS-CYCLE DIVERGENCE PATTERN (v4 UPDATE)**: planner-recommendation has diverged from user-lock in **5 of last 7 cycles** at gate-1 (CH-15 cycle hex `c3f46f17` F5.B over F5.A; CH-17 cycle hex `40c4d759` F5.B over F5.A; CH-18 cycle hex `c77937bc` F3.B over F3.A; CH-20 cycle hex `240616a4` F1.B over F1.A; **CH-24 cycle hex `5778bb77` F1.B over F1.A**). Lone non-divergent at gate-1: CH-19 cycle hex `2c520ba7`. **Within CH-24 alone, 3 divergent locks** (F1.B at gate-1 + F-D59.2.b + F-D59.3.b at gate-2.5). **Cumulative cross-cycle divergent forks now 7-of-9** (CH-15 F5.B, CH-17 F5.B, CH-18 F3.B, CH-20 F1.B, CH-24 F1.B, CH-24 F-D59.2, CH-24 F-D59.3 vs aligned: CH-19 + CH-24 F-D59.1 + lone gate-1 alignment in earlier cycles). **User pattern: choose tighter / richer / more-defensive / fuller-scope options consistently.** Treat divergence as the modal outcome, not the exception. CH-20's outcome falsified the "doc-only chunks reliably avoid divergence" hypothesis; CH-24's gate-2.5 outcome further falsifies "amend-don't-add is the safer recommendation" — when the surface is load-bearing AND the milestone is sealing, user prefers richer/fuller scope. Flag for retrospective routing as a process-update candidate: should the chunk-planner v13 invert the "amend-don't-add" default for milestone-seal / load-bearing-surface chunks?

> ⚠ **v3 SCOPE-EXPANSION CALLOUT** (added 2026-05-11): User elected at **gate-2 mid-cycle** to close the C-M5-3 API-surface flip in-chunk after P-NEW-TESTS authoring revealed the `recent_sessions: Vec::new()` hardcode at `server/src/platform/projects/detail.rs:229+654` was inline-doc-promised to flip during M5/P4 but never landed. **This is the first mid-cycle architectural scope expansion across all chunks.** The user's standing principle is *"quality and thoroughness over cycle completion"* — a milestone-seal cycle that surfaces a load-bearing API gap is the right moment to close it, not defer. Three NEW forks (F-D59.1.bound, F-D59.2.method, F-D59.3.shape) are appended below the F1–F5 gate-1 forks; they require **gate-2.5 lock** before P-FLIP-RECENT-SESSIONS opens. **v4 update: gate-2.5 locks captured below — 2-of-3 DIVERGENT from planner-recommendations.** Flagged for retrospective routing as a process-update candidate: *do future ratification / milestone-seal chunks reserve a "scope-expansion lane" for findings surfaced during test authoring?*

This chunk is a milestone-seal verification surface. Five gate-1 forks needed user-lock before implementation opened; all locks captured below (v2 plan reflects gate-1 outcomes). Planner recommendations preserved as historical record per v9 surfacing-not-suppressing approach. **v3 appended 3 gate-2.5 forks** for the C-M5-3 API-surface scope expansion. **v4 captures the gate-2.5 user-locks** with full preservation of historical option bodies.

### F1 — Test-file layout for `acceptance_m5.rs`

Forward-scope line 212 names `acceptance_m5.rs` (single file) as a cross-page e2e suite. M5's existing per-page acceptance files (`acceptance_sessions_m5p4.rs`, `acceptance_authority_templates.rs`, `acceptance_memory_extraction.rs`, `acceptance_system_flows_s05.rs`) are per-feature scoped; `acceptance_m5.rs` is the milestone-level rollup.

- **F1.A (planner-recommended)** — Single file `server/tests/acceptance_m5.rs` with 2 scenarios: (a) `m5_full_bootstrap_to_first_session_to_extraction_to_catalog` (the cross-page golden path — bootstrap → org → agent → project → adopt Template A+B → add org-specific system agent → launch session → tail events → session ends → assert memory extracted audit + catalog updated + UsesModel edge + RUNS_SESSION edge + page-11 recent-sessions visible), (b) `m5_cross_org_isolation_at_session_surface` (cross-org viewer can NOT see Org-A's recent sessions on Org-B's page-11 — mirrors M4's cross-org follow-up). Rationale: matches forward-scope literal text; matches M4's `acceptance_m4.rs` precedent (4 scenarios in one file); single discoverable entry-point for the milestone-rollup reviewer.
- **F1.B** — Split per-page: `acceptance_m5_orgs.rs`, `acceptance_m5_projects.rs`, `acceptance_m5_agents.rs`, `acceptance_m5_sessions.rs`, `acceptance_m5_memory.rs`. Rationale: parallels CH-20 F1.B's 5-file convention split; finer-grained CI failure isolation; per-page reviewer focus. Drawback: forward-scope row says singular `acceptance_m5.rs`; duplicates coverage of existing per-feature files; inflates file count without inflating coverage.
- **F1.C** — Hybrid: single `acceptance_m5.rs` golden path + 1 supplementary file `acceptance_m5_cross_org.rs` for the isolation scenario. Drawback: cross-org isolation is one scenario, doesn't merit its own file; M4 kept all 4 in `acceptance_m4.rs`.

**Locked at gate-1 (USER-DIVERGENT): F1.B** — 5 per-page test files. Diverges from planner-recommended F1.A. Scenarios redistributed across the 5 files per §7 P-NEW-TESTS deliverable 1 below.

### F2 — `e2e_first_session.rs` subprocess mechanics

(Unchanged from v2 — locked F2.A pure-subprocess. See v2 plan body, content preserved.)

- **F2.A (planner-recommended, LOCKED at gate-1)** — `assert_cmd::Command::cargo_bin("phi")` faithful end-to-end.
- **F2.B** — Library-direct.
- **F2.C** — Hybrid.

### F3 — phi-core reuse map refresh scope

(Unchanged from v2 — locked F3.A medium refresh. See v2 plan body, content preserved.)

- **F3.A (LOCKED)** — Medium refresh: update existing table + actual columns.
- **F3.B** — Light refresh.
- **F3.C** — Full rewrite.

### F4 — Audit envelope

(Unchanged from v2 — locked F4.B 3-agent envelope. See v2 plan body, content preserved.)

- **F4.A** — 2-agent.
- **F4.B (LOCKED, planner-recommended)** — 3-agent (Rust correctness / Docs fidelity / Vertical-slice integrity).
- **F4.C** — 2-agent with tightened threshold.

### F5 — M5 troubleshooting + ops runbook placement

(Unchanged from v2 — locked F5.A amend-don't-add. See v2 plan body, content preserved.)

- **F5.A (LOCKED, planner-recommended)** — amend `m5/user-guide/troubleshooting.md` end-to-end; NEW `m5/operations/m5-ops-runbook.md`.
- **F5.B** — NEW `m5_2/user-guide/troubleshooting.md` parallel page.
- **F5.C** — identical to F5.A.

---

### Forward-scope drift-count pre-flight verification (v12 rule)

Per chunk-planner v12 Row 1: for ratification chunks, verify the forward-scope row's drift-count parenthetical. **CH-24 is NOT a ratification chunk** (forward-scope line 211: *"Drifts closed: none new"*). **v3 update**: this no longer holds — v3 scope-expansion closes 1 new drift (`D-CH24-recent-sessions-api-flip`). The forward-scope row line 211 text *"Drifts closed: none new"* is now superseded by the gate-2 user-lock; flag for retrospective routing as a process-amendment candidate (does the forward-scope row need a META amendment at chunk-seal to reflect the scope expansion?). Recommendation: **YES** — append a one-line note to forward-scope line 211 at P-SEAL noting that CH-24 closed 1 new drift due to in-cycle scope expansion. Track as P-SEAL deliverable 7 (NEW).

---

### F-D59.1.bound — `recent_sessions` cardinality bound (v3 NEW; gate-2.5)

The `recent_sessions` panel on page-11 displays a bounded list of recent sessions for a project. Cardinality choice:

- **F-D59.1.a (planner-recommended)** — **LIMIT 10**. Rationale: matches typical panel UX (top-10 newest); matches CLI default `phi project show` page size in the existing `cli/src/commands/project.rs` surface; the existing `list_sessions_in_project` Repository method (`domain/src/repository.rs:1660`) is documented as *"Ordered newest-first by `started_at`"* — LIMIT 10 is the natural truncation. Drawback: if a project has > 10 active sessions, page-11 shows only 10 — but the live list endpoint at `/api/v0/orgs/:org/projects/:proj/sessions` is the unbounded surface (acceptance_m5_sessions.rs already asserts the live list returns the launched session at line 200-201).
- **F-D59.1.b** — **LIMIT 5**. Rationale: tighter; matches CH-24's quality-over-speed principle (less data per response). Drawback: too small for projects with simultaneous experiments.
- **F-D59.1.c** — **LIMIT 20**. Rationale: roomier panel. Drawback: contradicts typical web-panel UX patterns.

**Locked at gate-2.5: LIMIT 10** (aligned with planner-recommendation F-D59.1.a).

### F-D59.2.method — Repo method shape for the query (v3 NEW; gate-2.5; **v4 USER-LOCK DIVERGENT**)

The existing `list_sessions_in_project(&self, project: ProjectId) -> RepositoryResult<Vec<Session>>` Repository method (`domain/src/repository.rs:1660`, `domain/src/in_memory.rs:2384`, `store/src/repo_impl.rs:3321`) already returns sessions newest-first by `started_at`. The page-11 panel needs a top-10 slice. Two methods:

- **F-D59.2.a (planner-recommended)** — **Reuse existing `list_sessions_in_project` + slice in caller**. Implementation: `detail.rs:project_detail` calls `repo.list_sessions_in_project(project_id).await?`, takes `.into_iter().take(10).collect()`, maps each `Session` to a `RecentSessionStub`. Rationale: chunk-planner v10 amend-don't-add precedence; existing method satisfies the contract end-to-end; ZERO new Repository trait additions; ZERO new SurrealQL queries; ZERO new in-memory test fixtures. Performance: if a project has thousands of sessions this loads the full list — acceptable for M5 (typical project has < 100 sessions); M6 perf-tuning can introduce a `list_recent_sessions_for_project(project, limit)` if profiling surfaces a hotspot. Drawback: slight load-all-then-slice overhead — but only on page-11 reads, which are infrequent.
- **F-D59.2.b** — **Add new `list_recent_sessions_for_project(project: ProjectId, limit: u32) -> RepositoryResult<Vec<Session>>`**. Implementation: extend `Repository` trait + both impls (`in_memory.rs` + `repo_impl.rs`); SurrealQL adds `LIMIT $limit` clause. Rationale: precise wire contract; performance-correct for high-cardinality projects; explicit naming. Drawback: chunk-planner v10 favors amend-don't-add; doubles the cascade (Repository trait + 2 impl bodies + 1 SurrealQL); the existing list method is wire-stable + tested at acceptance level; new method needs new tests.

**Locked at gate-2.5 (USER-DIVERGENT): F-D59.2.b — add new `list_recent_sessions_for_project(project_id: ProjectId, limit: u32) -> RepositoryResult<Vec<RecentSessionEntry>>`**. Diverges from planner-recommended F-D59.2.a (caller-side `.take(10)` reuse). User rationale: query-side cap for scale-resilience (project with thousands of sessions doesn't load full list); clearer call-site semantics (the method name encodes the contract); explicit DB-tier truncation matches what a production DBA would expect; per chunk-planner v10's wire-mapping-cascade rule the cost is bounded (3 surfaces — trait + SurrealStore impl + InMemoryRepository impl). Planner-recommendation preserved above per v9 surfacing-not-suppressing approach. Note: the return type is `Vec<RecentSessionEntry>` (not `Vec<Session>`) — the new method's contract returns the view-shape directly per F-D59.3.b lock.

### F-D59.3.shape — `RecentSessionStub` / `RecentSessionEntry` field set (v3 NEW; gate-2.5; **v4 USER-LOCK DIVERGENT**)

The existing `RecentSessionStub` struct at `detail.rs:84-95` has 3 fields (`session_id: String`, `started_at: DateTime<Utc>`, `summary: String`). The user-request mentions an alternative `RecentSessionEntry { id: SessionId, project_id: ProjectId, agent_id: AgentId, started_at: DateTime<Utc>, ended_at: Option<DateTime<Utc>>, status: String, started_by_display_name: String }` shape. Three options:

- **F-D59.3.a (planner-recommended)** — **Reuse `RecentSessionStub` as-is with real population**. Implementation: keep the 3-field shape; map `Session.id.to_string()` → `session_id`, `Session.started_at` → `started_at`, derive `summary` from `Session` fields (e.g., `format!("Session by {agent_id}, status={state}")` or similar). Rationale: zero wire-shape change; all existing JSON consumers (CLI page-11 renderer, web page-11 renderer) keep working without touch; existing snapshot tests (`wire_shape_strips_phi_core` at `detail.rs:tests`) keep passing; chunk-planner v10 amend-don't-add. The `summary` field is the load-bearing display field — render it from the Session row meaningfully (e.g., `"by {started_by} • {governance_state}"`).
- **F-D59.3.b** — **Replace with `RecentSessionEntry`** (7-field shape: id, project_id, agent_id, started_at, ended_at, status, started_by_display_name). Rationale: richer panel; explicit named fields; aligns with what a future page-11 web UI would render. Drawback: wire-shape break; ALL consumers must update; `Display::summary` becomes the renderer's job, not the API's; requires CLI/web renderer changes (currently out-of-scope for CH-24).
- **F-D59.3.c** — **Extend `RecentSessionStub` with 2 optional fields** (`agent_id: Option<AgentId>`, `governance_state: Option<String>`) while keeping the existing 3 fields. Rationale: progressive enhancement; preserves wire compat; explicit data for future renderers. Drawback: noisy struct; the `summary` field becomes redundant with `governance_state`.

**Locked at gate-2.5 (USER-DIVERGENT): F-D59.3.b — replace `RecentSessionStub` with `RecentSessionEntry { id: SessionId, project_id: ProjectId, agent_id: AgentId, started_at: DateTime<Utc>, ended_at: Option<DateTime<Utc>>, status: String, started_by_display_name: String }`**. Old `RecentSessionStub` DELETED (no type-alias retained — clean rename mandated per REPLACE semantics). Diverges from planner-recommended F-D59.3.a (reuse stub with real population). User rationale: richer panel UX with explicit semantics; zero technical-debt-marker (the "Stub" suffix encoded a placeholder semantics that the v4 rename retires); aligns with a future page-11 web UI; explicit fields beat synthesised `summary` string for downstream consumers. Planner-recommendation preserved above per v9 surfacing-not-suppressing approach.

Field-shape rationale (planner-default for v4 implementer-guidance):
- `id: SessionId` — strongly-typed (replaces `session_id: String`); maps from `Session.id`.
- `project_id: ProjectId` — denormalised for client-side filtering / multi-project panels; maps from `Session.project_id`.
- `agent_id: AgentId` — explicit agent that ran the session; maps from `Session.agent_id`.
- `started_at: DateTime<Utc>` — preserved from stub; maps from `Session.started_at`.
- `ended_at: Option<DateTime<Utc>>` — NEW; nullable for active sessions; maps from `Session.ended_at`.
- `status: String` — derived from `Session.governance_state` rendered as a stable string (e.g., `"running" | "ended" | "errored"`).
- `started_by_display_name: String` — derived from the agent or principal that initiated the session; maps from `Session.started_by` (subject-of-record). If the source-of-truth lookup is not directly on the `Session` row (e.g., requires an Agent join), implementer to confirm at chunk-implementation open + AskUserQuestion if a 2nd repo round-trip is required.

Implementer note: if a field's source-of-truth doesn't exist at repo surface (e.g., `started_by_display_name` requires a join), pause via AskUserQuestion before adding an N+1 read pattern. Acceptable fallback: use `started_by: PrincipalRef` raw value and let the renderer resolve display names, deferring the join to M6.

---

## §1 — Context & principle

### Why this chunk

M5 began with 6 inherited carryovers from M4 (C-M5-1 through C-M5-6). C-M5-1 was closed in M5/P1. C-M5-2 through C-M5-6 were closed across M5/P4–P7 — but **all of them landed against MockProvider-stubbed transcripts** (D4.2's primary cause). CH-02 (M5.2 chunk, closed 2026-04-27) flipped the synthetic transcript feeder to real `phi_core::agent_loop` output. The carryover acceptance scenarios that asserted "this assertion holds against the stub" must now be re-verified against real-loop output, to confirm they're not hollow. CH-24 also ships the milestone-rollup acceptance (5 per-page files per F1.B lock), the e2e subprocess fixture (`e2e_first_session.rs`), the ops runbook + troubleshooting docs that the milestone tag depends on, refreshes the phi-core reuse map with actual close-of-M5 numbers, and stages the milestone tag `v0.1-m5` for user execution.

**v3 update — mid-cycle scope expansion (C-M5-3 API-surface flip):** P-NEW-TESTS authoring surfaced that `recent_sessions: Vec::new()` is hardcoded at `server/src/platform/projects/detail.rs:229+654` despite the inline doc-comment at `detail.rs:33` promising *"deferred to M5 per D11; the `recent_sessions` field in [`ProjectDetail`] is a placeholder `Vec::new()` until M5 wires baby-phi's governance `Session` node"*. C-M5-3 carryover ITSELF passes (Session/LoopRecord/Turn persistence works — verified P-VERIFY 5/5 PASS); only the API-surface flip never landed during M5/P4. The user-lock at gate-2: **close in-chunk** rather than defer to M5.3 cleanup. A new phase **P-FLIP-RECENT-SESSIONS** wires the panel to a real query and flips the tripwire assertion in `acceptance_m5_sessions.rs::m5_sessions_full_bootstrap_to_first_session_to_extraction_to_catalog` (line 224-230) from "asserts empty" to "asserts contains session". A new drift `D-CH24-recent-sessions-api-flip` documents the discovery + remediation; a new ADR-0059 ratifies the API-surface flip with 4 sub-decisions.

**v4 update — gate-2.5 user-lock 2-of-3 DIVERGENT:** Of the 3 gate-2.5 forks, only F-D59.1 (LIMIT 10) aligned with the planner-recommendation. F-D59.2 and F-D59.3 both diverged toward more-defensive / richer options: a dedicated `list_recent_sessions_for_project` repo method with query-side cap (F-D59.2.b) and a richer `RecentSessionEntry` struct replacing the `RecentSessionStub` placeholder (F-D59.3.b). Cascade footprint accordingly widens from v3's 2 callsite edits to **6-8 touched surfaces** (3 repo-method surfaces + 1 NEW struct + 2 wire-consumer call-sites + 0-2 snapshot updates). See §3 phi-core leverage map cascade-grep section for the per-surface enumeration.

**No drift IDs closed in this chunk (v2 baseline)** → **v3 update: 1 drift closed (`D-CH24-recent-sessions-api-flip`, discovered AND remediated in-chunk)**. Forward-scope line 211 wording (*"Drifts closed: none new"*) is amended at chunk-seal via P-SEAL deliverable 7.

### Quality-over-speed restatement

*"Concept docs are source-of-truth; implementation aligns to them. Drift is discovered, documented, and planned-through — never accumulated silently."*

CH-24 application: a milestone seal is where silently-accumulated drift surfaces. The acceptance re-verification phase MUST treat any failed assertion not as a test bug but as a previously-hidden drift; if a carryover assertion holding against MockProvider stubs no longer holds against real-loop output, that is a HIGH-severity discovery, paused via AskUserQuestion, with a new drift filed before CH-24 proceeds.

**v3 SCOPE-EXPANSION CALLOUT** (added 2026-05-11): User-locked override of the "no shipped-code change" principle for CH-24 to ship the M5 seal complete. The principle was that milestone-seal chunks verify + document; v3's scope expansion ships **fresh production code** (`detail.rs:229+654` query rewires + struct population) AND flips a tripwire assertion in the new acceptance suite. **This is a NEW pattern**: a mid-cycle architectural scope expansion driven by a P-NEW-TESTS authoring finding. Flag for retrospective routing as a process-update candidate. Open question for retrospective: *do future ratification / milestone-seal chunks reserve a "scope-expansion lane" for findings surfaced during test authoring, or do they default to defer-to-next-chunk + filing a drift?* See §13 (NEW process-novelty section) for the full retrospective routing prompt.

**v4 PATTERN-NOTE** (added 2026-05-11): The 2-of-3 gate-2.5 divergent locks reinforce a 7-of-9 cumulative pattern across cycles. User prefers richer/fuller/tighter/more-defensive options consistently. For mid-cycle scope expansions on load-bearing surfaces, the "amend-don't-add" default that the chunk-planner v10 favors is NOT the user's instinct. The retrospective routing in §13 (item 5, NEW) should weigh whether to invert this default for milestone-seal / API-surface chunks.

### Forward-scope reference

[`forward-scope/remaining-scope-post-m5-p7-22035b2a.md`](../../forward-scope/remaining-scope-post-m5-p7-22035b2a.md) §M5.2 inherited chunks, line 210–212; cross-ref M5 archive plan §P9 line 798–852.

---

## §2 — Concept alignment walk

CH-24 verifies + documents — and **v3 scope-expansion adds 1 row** for the API-surface flip. **v4 update: row 7 wording refined to reference the new dedicated query method.**

| Concept doc | § anchor | Claim (close paraphrase) | Status at chunk-open | Target status at chunk-close |
|---|---|---|---|---|
| [`phi-core-mapping.md`](../../../v0/concepts/phi-core-mapping.md) | §"Connection point" (lines 64–98) | baby-phi launches phi_core::agent_loop at session-launch time; `UsesModel` edge resolves runtime; `RUNS_SESSION` edge ties to project | honored (CH-02 close; CH-15 permission gate; CH-21 listener body) | honored (re-verified against real-loop output) |
| [`permissions/05-memory-sessions.md`](../../../v0/concepts/permissions/05-memory-sessions.md) | §"session lifecycle" | `Session` → `LoopRecord` → `Turn` persisted via `BabyPhiSessionRecorder`; transcripts available to memory-extraction listener | honored (CH-21) | honored (re-verified) |
| [`agent.md`](../../../v0/concepts/agent.md) | §"turn execution" | An agent's turn executes via phi_core::agent_loop with the AgentProfile-bound StreamProvider | honored (CH-02 close + MockProvider default) | honored (re-verified) |
| [`system-agents.md`](../../../v0/concepts/system-agents.md) | §"Memory Extraction Agent" + §"Agent Catalog Agent" | Memory extraction reads real transcripts (not stubs); Agent catalog updates on 8 DomainEvent variants | honored (CH-21 + CH-22) | honored (re-verified) |
| [`permissions/07-templates-and-tools.md`](../../../v0/concepts/permissions/07-templates-and-tools.md) | §"Template A/B/C/D" | All 4 templates fire end-to-end on their trigger edges | honored (CH-23 + carryover M4) | honored (re-verified across 5 per-page files) |
| [`permissions/README.md`](../../../v0/concepts/permissions/README.md) | §"Entry invariants" | Permission Check engine entry preconditions hold at every session-launch | honored (CH-15) | honored (re-verified) |
| **v3 NEW / v4 refined: [`phi-core-mapping.md`](../../../v0/concepts/phi-core-mapping.md)** | §"Session list query at API surface" (page-11 panel) | The page-11 `recent_sessions` panel queries persisted Session rows for the project via a dedicated `list_recent_sessions_for_project` Repository method, ordered newest-first, bounded to 10 at the query layer; returns the view-shape `RecentSessionEntry` directly | **silent-in-code** (M4 placeholder `Vec::new()` at `detail.rs:229+654`; inline-doc-promised M5 flip never landed) | **honored** (CH-24 P-FLIP-RECENT-SESSIONS wires real query + new method + new struct) |

**Permissions subtree hook**: cited per the §2 rule.
**phi-core-mapping hook**: cited per the §2 rule (CH-24's surface overlaps with phi-core types at the session-launch verification path AND at the v3 list-query surface).

CH-24 introduces **zero new concept claims** (v2 baseline) → **v3 update: one re-affirmed claim** at row 7 — the API-surface flip closes a `silent-in-code` row to `honored`. **v4 update: row 7 specifies the dedicated method + view-shape per gate-2.5 locks.** No concept-doc body text changes; only the implementation surface flips from placeholder to real (with richer shape).

---

## §3 — phi-core leverage map

CH-24 is a verification + docs + (v3) API-surface flip + (v4) dedicated-repo-method cascade chunk. **Expected import-count delta: 0 net new phi-core imports** (v4 update: the new method `list_recent_sessions_for_project` is domain-tier; it reads `Session` via existing `phi_core::session::model::Session` paths already imported at `domain/src/model/nodes.rs:1`; the new struct `RecentSessionEntry` is baby-phi-defined at `server::platform::projects::detail` — NOT a phi-core reuse; no new `use phi_core` lines emerge). Baseline at chunk-open: **57 lines of `use phi_core` across 27 unique files**.

| phi-core type | Current handling in baby-phi | Classification | Action in CH-24 |
|---|---|---|---|
| `phi_core::agent_loop` | direct-imported at `server/src/platform/sessions/launch.rs:90` | direct-reuse | **measure only** — no change |
| `phi_core::agent_loop_continue` | direct-imported at `server/src/platform/sessions/launch.rs:99` | direct-reuse | **measure only** — no change |
| `phi_core::session::model::{Session, LoopRecord, Turn, LoopStatus}` | wrapped at `domain/src/model/nodes.rs` (3-way wrap per ADR-0029) + direct-imported at `launch.rs:91` | wrap + direct-reuse | **measure only** — no change (v4: new repo method reads `Session` via existing wrap; no new import line) |
| `phi_core::types::event::{AgentEvent, ContinuationKind, TurnTrigger}` | direct-imported at `launch.rs:92` + `events.rs` | direct-reuse | **measure only** — no change |
| `phi_core::types::tool::AgentTool` | direct-imported at `sessions/tools.rs:27` | direct-reuse | **measure only** — no change |
| `phi_core::session::recorder::SessionRecorder` | wrapped at `domain/src/session_recorder.rs::BabyPhiSessionRecorder` | wrap | **measure only** — no change |
| `phi_core::provider::mock::MockProvider` | direct-imported at `sessions/provider.rs:26` | direct-reuse | **measure only** — no change |
| `phi_core::provider::traits::StreamProvider` | direct-imported at `sessions/provider.rs:27` | direct-reuse | **measure only** — no change |

### Canonical grep + baseline

```bash
grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l
# expect: 57 (CH-23 close + CH-20 close; carries forward into CH-24 unchanged; v3 PRESERVES this; v4 PRESERVES this)
```

### Positive close-audit greps (v4 update)

```bash
# v4 NEW — list_recent_sessions_for_project trait method declaration
grep -n "list_recent_sessions_for_project" /root/projects/phi/baby-phi/modules/crates/domain/src/repository.rs
# expect: 1 trait method signature
grep -n "list_recent_sessions_for_project" /root/projects/phi/baby-phi/modules/crates/store/src/repo_impl.rs
# expect: 1 impl method body
grep -n "list_recent_sessions_for_project" /root/projects/phi/baby-phi/modules/crates/domain/src/in_memory.rs
# expect: 1 impl method body (test fixture path)

# v4 NEW — RecentSessionEntry struct
grep -rn "RecentSessionEntry" /root/projects/phi/baby-phi/modules/crates/server/
# expect: ≥ 3 hits — struct definition at detail.rs + 2 call-sites at detail.rs:229,654

# v4 NEW — old RecentSessionStub fully replaced
grep -rn "RecentSessionStub" /root/projects/phi/baby-phi/modules/crates/ /root/projects/phi/baby-phi/docs/
# expect: 0 hits in source (fully replaced); historical doc references (e.g., m4 phi-core-reuse-map.md) may remain as archival mentions — context-check at audit

# v3-baseline preserved — recent_sessions panel call-sites no longer empty
grep -n "recent_sessions" /root/projects/phi/baby-phi/modules/crates/server/src/platform/projects/detail.rs
# expect at chunk close: ≥ 2 hits at lines 229 + 654 reading 'list_recent_sessions_for_project' or similar real-query expression (NOT 'Vec::new()')
grep -c "Vec::new()" /root/projects/phi/baby-phi/modules/crates/server/src/platform/projects/detail.rs
# expect at chunk-open: 2 hits at lines 229 + 654 → expect at close: 0 hits at those lines (other Vec::new() elsewhere in file may remain — context-check)
```

### Forbidden-duplication greps

(Unchanged from v2 — zero re-definitions of phi-core types. v3 + v4 do NOT introduce any new struct that could duplicate a phi-core type. `RecentSessionEntry` is a view-shape struct local to baby-phi; no phi-core counterpart.)

### Cascade-artifact discipline (v4 RE-RUN)

**v4 cascade — dedicated repo method + new struct**: 6-8 touched surfaces total. Per chunk-planner v10 wire-mapping cascade rule, the new trait method requires enumerative addition at every impl site (no `_` catch-all dilutes the cascade for trait methods).

```bash
git -C /root/projects/phi/baby-phi grep -nE 'list_sessions_in_project' modules/crates/
# baseline: 3 hits — 1 trait + 2 impls (per v3 verification: domain/src/repository.rs:1660, domain/src/in_memory.rs:2384, store/src/repo_impl.rs:3321)
# v4 prediction for list_recent_sessions_for_project at close: 3 hits (mirror cascade) + caller-site reads in detail.rs
```

**Per-file cascade breakdown for v4** (per chunk-planner v3 cascade discipline):

| File | Surface | Action |
|---|---|---|
| `domain/src/repository.rs` | `Repository` trait method declaration | ADD 1 new method signature: `async fn list_recent_sessions_for_project(&self, project: ProjectId, limit: u32) -> RepositoryResult<Vec<RecentSessionEntry>>;` |
| `store/src/repo_impl.rs` | `SurrealStore` impl method body | ADD 1 new impl body: SurrealQL `SELECT * FROM session WHERE project_id = $project ORDER BY started_at DESC LIMIT $limit` (cap pushed into query for scale-resilience) + view-shape mapping to `RecentSessionEntry` |
| `domain/src/in_memory.rs` | `InMemoryRepository` impl method body | ADD 1 new impl body: Vec filter + sort_by(`started_at` DESC) + take(limit) + view-shape mapping to `RecentSessionEntry` |
| `server/src/platform/projects/detail.rs` (struct def) | NEW struct `RecentSessionEntry` + DELETE old `RecentSessionStub` | REPLACE: old struct removed; new struct added with 7 fields per F-D59.3.b lock |
| `server/src/platform/projects/detail.rs:229` | Call-site 1 (primary `project_detail` path) | REPLACE `recent_sessions: Vec::new()` with `recent_sessions: repo.list_recent_sessions_for_project(project_id, 10).await.map_err(...)?` |
| `server/src/platform/projects/detail.rs:654` | Call-site 2 (secondary path — branch context TBD at chunk-implementation open) | Either REPLACE identically OR deliberately preserve empty in error-path branch with contextual comment |
| `server/src/platform/projects/detail.rs:33` | Module doc-comment | REMOVE stale "deferred to M5 per D11" wording; ADD ADR-0059 cross-reference |
| `acceptance_m5_sessions.rs:224-230` | Tripwire assertion | FLIP from `assert!(recent.is_empty())` to `assert_eq!(recent.len(), 1)` + `assert_eq!(recent[0]["id"]...)` |
| Snapshot tests (TBD) | grep `RecentSessionStub` at chunk-open → expect 0-2 snapshot test updates if any consumer pins the old struct name | 1-line rename per snapshot if any |

**Total cascade footprint**: 3 repo-method surfaces + 1 NEW struct + 2 wire-consumer call-sites + 1 module doc-comment + 1 tripwire flip + 0-2 snapshot updates = **6-9 touched surfaces** (vs v3's 2-3). Pause discipline: if actual cascade > 13 sites (1.5× upper-bound prediction), pause via AskUserQuestion.

**Type-derive pre-check (chunk-planner v7)**: `SessionId`, `ProjectId`, `AgentId` are existing typed wrappers. `Session` already has the needed accessor methods (`.id`, `.project_id`, `.agent_id`, `.started_at`, `.ended_at`, `.governance_state`, `.started_by`) per the wrap at `domain/src/model/nodes.rs`. The new struct `RecentSessionEntry` will need `#[derive(Debug, Clone, Serialize, Deserialize)]` (Serialize for HTTP response; Deserialize optional for round-trip-test fixtures; Clone for any caller-side cloning). Implementer to confirm at chunk-implementation open that all 7 source-of-truth fields are accessible without an N+1 read (specifically `started_by_display_name` may require an Agent-table join — see F-D59.3.b implementer note).

### Cross-cycle phi-core baseline anchor

CH-18 closed at 57; CH-19 closed at 57; CH-20 closed at 57; CH-21/22/23 closed at 57; **CH-24 v4 expected to close at 57 (Δ +0)** — v4 scope expansion uses already-imported `phi_core::session::model::Session` via existing `domain/src/model/nodes.rs` wrap path; new repo method is domain-tier; new struct is baby-phi-defined; no new `use phi_core` line emerges.

---

## §3.B — K8s microservice readiness check

CH-24 introduces no new code surface (v2 baseline). **v3 update**: the API-surface flip wires `recent_sessions` to a real Repository read — the read is per-request, stateless, no in-process state — K8s posture preserved. **v4 update**: same posture; the new dedicated repo method `list_recent_sessions_for_project` is per-request stateless against the SurrealDB session table; query-side LIMIT 10 is more scale-resilient than the v3 caller-side slice (no in-memory full-list materialisation).

| Axis | What to check | This chunk's surface | New blocker? | Action |
|---|---|---|---|---|
| **A1** | New in-process state | None — verification + tests + (v3/v4) one per-request DB read via new repo method | No | — |
| **A2** | New IPC channel | None | No | — |
| **A3** | New pod-local resource | `e2e_first_session.rs` subprocess fixture (test-only) | No (test-only) | Document in test docstring |
| **A4** | Migration runner / first-apply race | No new migration (v4 reuses existing `session` table schema; new repo method is a read, no DDL) | No | — |
| **A5** | Trait-shape requirement | **v4: NEW trait method** `Repository::list_recent_sessions_for_project` — additive method on existing trait; binary-compat preserved for callers; in-process trait extension only | No (additive trait method, not a trait change) | — |
| **A6** | Cross-pod state sharing | No new state (v4 read is per-request, stateless; query-side cap means no per-pod cache) | No | — |
| **A7** | Audit hash-chain symmetry | (v4) The flip + new method does NOT emit a new audit event — read is silent | No | — |

**Conclusion paragraph.** **K8s-neutral.** All 7 axes evaluate `no impact` (A3 has a test-only annotation, not a production blocker; A5 has a "new trait method" annotation but it is purely additive). v4 scope expansion does NOT introduce any K8s blocker class. No new `CHK8S-D-XX` ledger entry.

### Open CHK8S-D-* ledger status at M5 seal

(Unchanged from v2 — 10 entries open + targeted to M7b.)

---

## §3.C — User-facing documentation impact map

CH-24's docs deliverables are the milestone-rollup tier. **v3 update**: scope expansion added 4 new doc deliverables. **v4 update**: 1 additional doc touched — `m5/architecture/phi-core-reuse-map.md` (already in scope per F3.A) gains a note that `RecentSessionEntry` is baby-phi-defined (not a phi-core reuse).

| Tier | File pattern | This chunk touches? | Action |
|---|---|---|---|
| **Architecture** | [`m5/architecture/phi-core-reuse-map.md`](../../../v0/implementation/m5/architecture/phi-core-reuse-map.md) | YES — refresh with actuals per F3 lock + **v4 note: `RecentSessionEntry` is baby-phi-defined (not a phi-core reuse)** | (a) update in-chunk (P-DOCS) |
| **Architecture** | [`m5/README.md`](../../../v0/implementation/m5/README.md) | YES — Phase status table; ADR status | (a) update in-chunk (P-SEAL) |
| **Architecture** | [`m5/architecture/overview.md`](../../../v0/implementation/m5/architecture/overview.md) OR new `m5_2/architecture/recent-sessions-panel.md` (decide at chunk-implementation open based on existing structure) | YES (v3/v4) — body amendment: document `recent_sessions` panel semantics (cardinality bound 10, ordering newest-first, freshness on-read, dedicated method, view-shape struct) | (a) update in-chunk (P-DOCS) — small additive section OR new short page |
| **Operations** | `m5/operations/m5-ops-runbook.md` (**NEW** per F5 lock A) | YES — NEW per-page ops runbook | (a) create in-chunk (P-DOCS) |
| **Operations** | **v3 NEW row in `m5/operations/m5-ops-runbook.md`** | YES — add row for `recent_sessions` panel: incident scenario, query path (cite `list_recent_sessions_for_project`), troubleshooting pointer | (a) add row in-chunk (P-DOCS) |
| **Operations** | [`m5/operations/session-launch-operations.md`](../../../v0/implementation/m5/operations/session-launch-operations.md), [`m5/operations/system-agents-operations.md`](../../../v0/implementation/m5/operations/system-agents-operations.md), [`m5/operations/authority-templates-operations.md`](../../../v0/implementation/m5/operations/authority-templates-operations.md), [`m5/operations/system-flows-s02-s03-operations.md`](../../../v0/implementation/m5/operations/system-flows-s02-s03-operations.md) | verified-header re-stamp only | (a) verified-header bump in-chunk (P-DOCS) |
| **User-guide** | [`m5/user-guide/troubleshooting.md`](../../../v0/implementation/m5/user-guide/troubleshooting.md) | YES per F5.A lock — fill out from placeholder | (a) update in-chunk (P-DOCS) end-to-end |
| **User-guide** | **v3 NEW row in `m5/user-guide/troubleshooting.md`** | YES — add row for "page-11 shows empty recent_sessions" → resolution pointer | (a) add row in-chunk (P-DOCS) |
| **User-guide** | [`m5/user-guide/first-session-walkthrough.md`](../../../v0/implementation/m5/user-guide/first-session-walkthrough.md), [`m5/user-guide/authority-templates-walkthrough.md`](../../../v0/implementation/m5/user-guide/authority-templates-walkthrough.md), [`m5/user-guide/system-agents-walkthrough.md`](../../../v0/implementation/m5/user-guide/system-agents-walkthrough.md), [`m5/user-guide/cli-reference-m5.md`](../../../v0/implementation/m5/user-guide/cli-reference-m5.md) | verified-header re-stamp only | (a) verified-header bump in-chunk (P-DOCS) |
| **User-guide** | [`m5_2/user-guide/identity-overview.md`](../../../v0/implementation/m5_2/user-guide/identity-overview.md), [`m5_2/user-guide/memory-extraction-overview.md`](../../../v0/implementation/m5_2/user-guide/memory-extraction-overview.md), [`m5_2/user-guide/selector-syntax-guide.md`](../../../v0/implementation/m5_2/user-guide/selector-syntax-guide.md) | verified-header check only — likely no change | (b) defer; successor: none |
| **Drifts (v3 NEW)** | `m5_1/drifts/D-CH24-recent-sessions-api-flip.md` | YES — NEW drift file | (a) create in-chunk (P-DOCS) |
| **Drifts index (v3 NEW)** | `m5_1/drifts/_concept-audit-matrix.md` | YES — add row for "page-11 recent_sessions panel" mapping the drift to API-surface flip; status `silent-in-code` → `honored` | (a) add row in-chunk (P-DOCS) |
| **Drifts index (v3 NEW)** | `m5_1/drifts/README.md` | YES — add index entry for new drift | (a) add row in-chunk (P-DOCS) |
| **ADRs (v3 NEW / v4 sub-decisions rewritten)** | `m5_2/decisions/0059-recent-sessions-api-surface-flip.md` | YES — NEW ADR with v4 sub-decision wording | (a) create in-chunk (P-FLIP-RECENT-SESSIONS commits draft `Proposed`; P-SEAL flips to `Accepted`) |
| **Source code doc-comment (v3 NEW)** | `server/src/platform/projects/detail.rs:33` | YES — module doc-comment refresh (remove stale "deferred to M5 per D11" line; replace with current reality per ADR-0059) | (a) update in-chunk (P-FLIP-RECENT-SESSIONS) |
| **Forward-scope (v3 NEW)** | `docs/specs/plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md:211` | YES — one-line amendment noting CH-24 closed 1 new drift via in-cycle scope expansion | (a) amend in-chunk (P-SEAL deliverable 7) |

**Doc-sync sweep at gate-2 (chunk-planner v6 + CH-15 retro widening):** v2 baseline was "ZERO expected". **v3 update**: the API-surface flip removes the stale "M4 placeholder" / "deferred to M5 per D11" / "C-M5-3 flips this to real rows" wording from `detail.rs:33` + the existing test docstring at `acceptance_m5_sessions.rs:204-209` ("Page-11 carryover gap (M4 placeholder still active)" + "C-M5-3 inline note never executed"). **v4 update**: additionally sweep for `RecentSessionStub` references repo-wide; archival doc mentions (e.g., `m4/architecture/phi-core-reuse-map.md:88`) may remain as historical record but ALL active source-code references MUST migrate to `RecentSessionEntry`. Apply at P-FLIP-RECENT-SESSIONS or P-DOCS as appropriate.

---

## §3.D — Forward-scope-vs-concept-doc precedence

Per chunk-planner v9 mandatory pre-flight check (CH-17 retro Row 1):

**Forward-scope row literal terms (line 210–212):**
- `"C-M5-2"` / `"C-M5-3"` / `"C-M5-4"` / `"C-M5-5"` / `"C-M5-6"` — carryover IDs; canonical home at `build-plan-v01-36d0c6c5.md:272–290` + `m5/architecture/m4-postflight-delta.md:29–88`. All 5 IDs verified extant. **v3 note**: C-M5-3 was verified PASS for the persistence half (Session/LoopRecord/Turn writes work), but the API-surface flip half never landed. The carryover ID itself is unchanged; the scope expansion closes a sub-piece of C-M5-3 that the M5/P4 plan promised but didn't ship.
- `"acceptance_m5.rs"` / `"e2e_first_session.rs"` — F1.B fragmentation re-interpretation documented.
- `"Drifts closed: none new"` — **v3 supersedes**: forward-scope wording amended at P-SEAL deliverable 7 to acknowledge CH-24 closed 1 new drift (`D-CH24-recent-sessions-api-flip`) via in-cycle scope expansion.

**Verdict:** zero closed-set concept-doc contradictions. v3 forward-scope amendment is a documented scope-amendment, not a closed-set break. v4 does not change any closed-set claim — the new repo method is a trait extension (no fundamental-kind / action-vocabulary / closed-set break).

---

## §4 — Drifts closed

| Drift ID | File | Severity | Bucket | Transition | Notes |
|---|---|---|---|---|---|
| **D-CH24-recent-sessions-api-flip** (v3 NEW; v4 wording preserved) | `m5_1/drifts/D-CH24-recent-sessions-api-flip.md` (NEW) | MEDIUM | Bucket B (load-bearing API surface gap) | `discovered → remediated` (discovered AND closed in this chunk) | Pre-existing M4 placeholder; C-M5-3 inline doc-comment at `detail.rs:33` promised the flip during M5/P4 but the flip never landed; surfaced during CH-24 P-NEW-TESTS authoring; user-locked at gate-2 mid-cycle to close in-chunk via P-FLIP-RECENT-SESSIONS. Ratified by ADR-0059 (§D59.1–§D59.4). **v4 update**: drift body to cite the dedicated method + RecentSessionEntry per gate-2.5 user-locks. |

**v2 baseline was "(none) — Drifts closed: none new (forward-scope line 211)"**. v3 amends the forward-scope row at P-SEAL deliverable 7 to acknowledge the in-cycle scope expansion. v4 preserves this — still 1 drift closed.

**Mid-flight discovery rule:** if re-verification (P-VERIFY) surfaces a hollow carryover, that is a NEW drift — pause via AskUserQuestion, file the drift, surface to user. **v3 actualisation**: P-VERIFY itself was clean (5/5 PASS); P-NEW-TESTS authoring surfaced the API-surface gap; rule fired, user-lock'd close-in-chunk.

---

## §5 — ADRs drafted

**v2 baseline: ZERO new ADRs.** **v3 update: ONE new ADR — `ADR-0059`**, reserving the conditionally-planned slot from v2. **v4 update: ADR-0059 sub-decisions §D59.2 + §D59.3 REWRITTEN per gate-2.5 user-locks.**

### ADR-0059 — `recent-sessions-api-surface-flip`

- **File**: `docs/specs/v0/implementation/m5_2/decisions/0059-recent-sessions-api-surface-flip.md` (NEW).
- **Status**: `Proposed` at planner draft → `Accepted` at chunk-seal (P-SEAL).
- **Sub-decisions**:

#### §D59.1 — `recent_sessions` cardinality bound (UNCHANGED from v3)

The `recent_sessions` panel queries persisted Session rows for the project, ordered newest-first by `started_at`, bounded to **LIMIT 10**. Per F-D59.1.a user-lock at gate-2.5 — typical panel UX; live list endpoint at `/api/v0/orgs/:org/projects/:proj/sessions` is the unbounded surface. **v4 update**: the LIMIT is pushed into the query layer (not caller-side) per §D59.2 below; the bound is `u32` typed for API-cleanliness.

#### §D59.2 — Repo method shape (REWRITTEN v4 per gate-2.5 user-lock F-D59.2.b)

Add new `Repository::list_recent_sessions_for_project(&self, project_id: ProjectId, limit: u32) -> RepositoryResult<Vec<RecentSessionEntry>>` to the `Repository` trait (`domain/src/repository.rs`). Implementation:

- **SurrealStore impl** (`store/src/repo_impl.rs`): SurrealQL `SELECT * FROM session WHERE project_id = $project ORDER BY started_at DESC LIMIT $limit` — cap pushed into the query for scale-resilience; result rows mapped from raw Session into the view-shape `RecentSessionEntry`.
- **InMemoryRepository impl** (`domain/src/in_memory.rs`): Vec filter on `project_id` + sort_by descending `started_at` + take(`limit` as usize) + map to `RecentSessionEntry`.

Rationale: user-locked at gate-2.5 over the caller-side-take alternative (F-D59.2.a) for (a) query-side cap means projects with thousands of sessions don't materialise the full list in-memory; (b) clearer call-site semantics — the method name encodes the contract; (c) explicit DB-tier truncation matches production-DBA expectations; (d) per chunk-planner v10 wire-mapping-cascade rule the cost is bounded (3 surfaces — trait + SurrealStore impl + InMemoryRepository impl). The cascade adds ~0.1 engineer-days vs the reuse path.

Planner-recommendation (F-D59.2.a, reuse + slice) preserved as historical record per v9 surfacing-not-suppressing approach.

#### §D59.3 — Repo method return type + struct shape (REWRITTEN v4 per gate-2.5 user-lock F-D59.3.b)

Replace `RecentSessionStub` placeholder struct at `server::platform::projects::detail` with `RecentSessionEntry { id: SessionId, project_id: ProjectId, agent_id: AgentId, started_at: DateTime<Utc>, ended_at: Option<DateTime<Utc>>, status: String, started_by_display_name: String }`. Old `RecentSessionStub` struct DELETED (no type-alias retained — clean rename mandated per REPLACE semantics; the "Stub" suffix encoded placeholder semantics that the rename retires).

Rationale: user-locked at gate-2.5 over the reuse-stub path (F-D59.3.a) for (a) richer panel UX with explicit named fields; (b) zero technical-debt-marker; (c) explicit semantics beat synthesised `summary` strings for downstream consumers (CLI page-11 renderer can format from named fields; future web UI gets typed fields); (d) the wire-shape change is contained — current consumers are mock/snapshot tests only (CLI and web do not yet render this panel meaningfully — confirmed via the v3 baseline that says `Vec::new()` was hardcoded since M4).

Field rationale documented at F-D59.3.b lock body above. Implementer note (carried forward to P-FLIP-RECENT-SESSIONS): if `started_by_display_name` requires an Agent-table join, pause via AskUserQuestion before adding an N+1 read pattern; acceptable fallback is to leave `started_by` as `PrincipalRef` raw and defer display-name resolution to the renderer.

Planner-recommendation (F-D59.3.a, reuse stub with real population) preserved as historical record per v9 surfacing-not-suppressing approach.

#### §D59.4 — Pre-existing-behaviour preservation note (per chunk-planner v11 variation (c) Never-shipped-yet; UNCHANGED from v3)

*"Pre-existing absence preserved: the `recent_sessions` panel never shipped with real data. The M4 placeholder at `detail.rs:84-95` + `:229,654` (`Vec::new()` hardcode) + the inline doc-comment at `detail.rs:33` (*"deferred to M5 per D11; the recent_sessions field [...] is a placeholder Vec::new() until M5"*) + the non-landing during M5/P4 are the never-shipped pre-existing behaviour. CH-24 ships the first real population. v4 update: the struct rename from `RecentSessionStub` to `RecentSessionEntry` is a wire-shape change but the prior state had ZERO real consumers (the panel was always empty), so the rename does NOT break a shipped contract — it ratifies the shape at first-real-population time."*

### Cross-references to prior ADRs cited as precedent

- **ADR-0029** (`m5/decisions/0029-session-persistence-and-recorder-wrap.md`) — Session persistence + SessionRecorder wrap (C-M5-3's persistence half landed against this).
- **ADR-0034** (`m5_2/decisions/0034-c-m5-3-baby-phi-session-node.md` if extant, else cite ADR-0029) — C-M5-3 baby-phi `Session` node design.
- **ADR-0058** (`m5_2/decisions/0058-bucket-c-convention-confirm-in-place.md`) — Bucket C confirm-in-place convention; the v3 scope-expansion follows the spirit of "confirm-in-place" by ratifying a previously-deferred decision in the current cycle rather than deferring further.

---

## §6 — Prior-chunk regression re-verification

(Unchanged from v2 — see plan history. v3 added one row; v4 updates the row to reference the new repo method.)

**v3+v4 NEW row** at the bottom:
| Upstream chunk | Invariant this chunk relies on | Re-verification command |
|---|---|---|
| **Carryover (v3/v4)** | C-M5-3 sub-piece: `Session` table is queryable by `project_id` ordered newest-first by `started_at` (the foundation the new `list_recent_sessions_for_project` method builds on) | `grep -n "list_sessions_in_project" /root/projects/phi/baby-phi/modules/crates/domain/src/repository.rs` (expect 1 hit at line 1660 — the prior method that proves the indexed query path works) + `grep -n "list_sessions_in_project" /root/projects/phi/baby-phi/modules/crates/store/src/repo_impl.rs` (expect 1 hit at line 3321) |

---

## §7 — Phases within the chunk

CH-24 is a **6-phase chunk** (v2 baseline was 5 phases): P0 (preflight) + P-VERIFY (carryover re-run) + P-NEW-TESTS (5 per-page acceptance files + e2e) + **P-FLIP-RECENT-SESSIONS (v3 NEW; v4 REWRITTEN deliverables)** + P-DOCS (runbook, troubleshooting, reuse-map, drift file, ADR-0059, concept-audit-matrix, drift README, detail.rs:33 doc-comment refresh) + P-SEAL (CI extensions, paperwork, milestone-tag staging, forward-scope amendment).

### Phase P0 — Preflight verification (unchanged from v2)

(Status: ✓ DONE at v2 close.)

### Phase P-VERIFY — Carryover re-verification against real-loop output (unchanged from v2)

(Status: ✓ DONE at v2 close. 5/5 PASS — all carryovers PASS against real-loop output. C-M5-3's persistence half PASSES; the API-surface half (recent_sessions) was identified during P-NEW-TESTS as a separate sub-piece — addressed by P-FLIP-RECENT-SESSIONS.)

### Phase P-NEW-TESTS — 5 per-page acceptance files + `e2e_first_session.rs` (unchanged from v2; status: ✓ DONE at v2 close)

(Status: ✓ DONE at v2 close. 8/8 NEW tests pass; workspace 1537/0/2 at v2 close.)

**v3 carry-forward**: the tripwire assertion at `acceptance_m5_sessions.rs:224-230` asserting `recent.is_empty()` is **scheduled for flip** at P-FLIP-RECENT-SESSIONS (deliverable 6 below).

### Phase P-FLIP-RECENT-SESSIONS (v3 NEW; v4 REWRITTEN deliverables per gate-2.5 user-locks) — Wire `recent_sessions` to real query

**Goal.** Close the C-M5-3 API-surface flip per the gate-2.5 user-locks: (1) add a dedicated `Repository::list_recent_sessions_for_project` trait method with query-side LIMIT (F-D59.2.b); (2) replace `RecentSessionStub` with `RecentSessionEntry` (F-D59.3.b); (3) wire both call-sites at `detail.rs:229+654` to the new method + struct; (4) flip the tripwire assertion at `acceptance_m5_sessions.rs:224-230` from "asserts empty" to "asserts contains session_id + status"; (5) refresh inline doc-comments.

**Deliverables.**

1. **NEW trait method** `Repository::list_recent_sessions_for_project(&self, project_id: ProjectId, limit: u32) -> RepositoryResult<Vec<RecentSessionEntry>>` at `modules/crates/domain/src/repository.rs` (verify trait file path at chunk-implementation open). Per §D59.2 / F-D59.2.b user-lock. Include rustdoc citing ADR-0059 §D59.1–§D59.3.

2. **NEW struct** `RecentSessionEntry` at `modules/crates/server/src/platform/projects/detail.rs` (consumer module) with 7 fields per F-D59.3.b lock:
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct RecentSessionEntry {
       pub id: SessionId,
       pub project_id: ProjectId,
       pub agent_id: AgentId,
       pub started_at: DateTime<Utc>,
       pub ended_at: Option<DateTime<Utc>>,
       pub status: String,
       pub started_by_display_name: String,
   }
   ```
   DELETE old `RecentSessionStub` struct at `detail.rs:84-95` (no type-alias retained — clean rename). Update `ProjectDetail.recent_sessions: Vec<RecentSessionStub>` field at `detail.rs:110` to `Vec<RecentSessionEntry>`.

3. **NEW SurrealStore impl** in `modules/crates/store/src/repo_impl.rs`: implement `list_recent_sessions_for_project` body using SurrealQL `SELECT * FROM session WHERE project_id = $project ORDER BY started_at DESC LIMIT $limit`. Bind `$project` and `$limit` parameters. Map result rows into `RecentSessionEntry` view-shape. If `started_by_display_name` requires a 2nd round-trip (Agent join), pause via AskUserQuestion before shipping.

4. **NEW InMemoryRepository impl** at `modules/crates/domain/src/in_memory.rs`: implement `list_recent_sessions_for_project` body using `self.sessions.values().filter(|s| s.project_id == project_id).sorted_by(|a, b| b.started_at.cmp(&a.started_at)).take(limit as usize).map(into_recent_session_entry).collect()` (or equivalent — the in-memory impl must mirror the SurrealQL contract: filter by project, sort newest-first, truncate, view-shape map).

5. **WIRE** `recent_sessions` at TWO call-sites in `modules/crates/server/src/platform/projects/detail.rs`:
   - `:229` (primary path in `project_detail`):
     ```rust
     // Before (M4 placeholder):
     recent_sessions: Vec::new(),
     // After (v4 dedicated method + new struct):
     recent_sessions: repo
         .list_recent_sessions_for_project(project_id, 10)
         .await
         .map_err(|e| ProjectError::Repository(e.to_string()))?,
     ```
   - `:654` (secondary path — TBD on implementer-read: this is the second `Vec::new()` site, possibly in a different control-flow branch e.g. AccessDenied / NotFound fallthrough). Read context at chunk-implementation open and wire identically OR confirm it should stay empty in that branch with a contextual comment citing the branch's semantics.

6. **FLIP TRIPWIRE** at `modules/crates/server/tests/acceptance_m5_sessions.rs:224-230`:
   - Replace `assert!(recent.is_empty(), "M4 placeholder ...")` with assertions on the JSON / typed response: `assert_eq!(recent.len(), 1, "Project has exactly 1 session after launch")` AND `assert_eq!(recent[0]["id"].as_str().unwrap(), session_id.to_string())` AND check `recent[0]["started_at"]` is present (non-null DateTime) AND check `recent[0]["status"]` is one of the expected enum strings (e.g., `"running"` if test waits ≤ session-end OR `"ended"` if test waits past extraction).
   - Also patch the surrounding test docstring (lines 204-209 + 232-237) — remove the "M4 placeholder still active" / "C-M5-3 inline note never executed" / "flip the recent_sessions assertion when the placeholder closes" wording; replace with the now-current "real query wired at CH-24 P-FLIP-RECENT-SESSIONS via list_recent_sessions_for_project + RecentSessionEntry" narrative.

7. **REFRESH** `detail.rs:33` module doc-comment — remove the stale `"deferred to M5 per D11; the recent_sessions field in [\`ProjectDetail\`] is a placeholder Vec::new() until M5 wires baby-phi's governance Session node"` line; replace with the current reality e.g. *"`Session` / `LoopRecord` / `Turn` — wrapped at `domain/src/model/nodes.rs` per ADR-0029; the `recent_sessions` field is populated from `Repository::list_recent_sessions_for_project` per ADR-0059 §D59.1–§D59.3."*. Also refresh the new `RecentSessionEntry` struct docstring at `detail.rs:84` (formerly `RecentSessionStub`'s docstring location; document the view-shape contract per ADR-0059 §D59.3).

8. **SNAPSHOT TEST SWEEP** (per chunk-planner v10 cascade discipline): grep `RecentSessionStub` repo-wide; any snapshot test consuming the old struct name gets a 1-line rename. Expected: 0-2 snapshot updates. If > 2 are found, pause via AskUserQuestion.

9. **(OPTIONAL)** Add 1 supplementary acceptance test in `acceptance_m5_sessions.rs` named `m5_sessions_recent_sessions_returns_at_most_10_when_more_present` exercising the LIMIT 10 cardinality bound. Launch 12 sessions sequentially in a single project; assert `recent.len() == 10`; assert `recent[0]["id"]` is the newest. Implementer decides at chunk-implementation open whether to ship (recommend YES if the launch helper supports rapid loop; NO if it requires test-runtime > 10s).

10. **Run tests post-implementation** (cargo-clean discipline per chunk-implementer v8 + CH-18 CLAUDE.md update):
    ```bash
    /root/rust-env/cargo/bin/cargo test --manifest-path /root/projects/phi/baby-phi/Cargo.toml --test acceptance_m5_sessions -j 4 -- --test-threads 1
    /root/rust-env/cargo/bin/cargo clean --manifest-path /root/projects/phi/baby-phi/Cargo.toml
    ```

**Tests.** 0 new tests (baseline) OR +1 supplementary test (deliverable 9 optional). Net delta: 0 or +1 to band. Cascade-grep adds 0 net tests — the new repo method is exercised by the existing scenario transitively via `detail.rs` call-sites.

**Concept-alignment check.** §2 row 1 (phi-core-mapping) re-verified at the API surface; row 7 (NEW v3, refined v4 — "Session list query at API surface via dedicated method + view-shape") transitions `silent-in-code → honored`.

**phi-core leverage check.** Δ +0 imports. The new query call returns the view-shape `RecentSessionEntry` (baby-phi-defined, not a phi-core type); the underlying `Session` is reached via the existing `phi_core::session::model::Session` import path already in scope at `domain/src/model/nodes.rs`; no new `use phi_core` line at any of the 3 new method-impl sites or the consumer module.

**Cascade-grep at chunk-open** (per chunk-planner v3 + v10):
```bash
git -C /root/projects/phi/baby-phi grep -nE 'recent_sessions:' modules/crates/
# expect: 4-5 hits — 2 production callsites (detail.rs:229,654) + 1-2 test fixture refs + 0 fixture cascade
git -C /root/projects/phi/baby-phi grep -nE 'list_sessions_in_project' modules/crates/
# baseline: 3 hits — for the predecessor method's cascade shape (1 trait + 2 impls). New method mirrors at 3 hits.
git -C /root/projects/phi/baby-phi grep -nE 'RecentSessionStub' modules/crates/
# at chunk-open: expect 2 hits in detail.rs + 0 in tests; at chunk-close: expect 0 hits source-side
```
If cascade > 13 sites (1.5× the v4 upper-bound prediction of 9) OR any cardinality category exceeds 1.5× prediction, pause via AskUserQuestion.

**Pause discipline.** If reading `:654` reveals an incompatible control-flow context (e.g., the second `Vec::new()` is in an error-path branch where `repo.list_recent_sessions_for_project` cannot run), pause via AskUserQuestion before wiring. The implementation may need to keep `:654` empty + only wire `:229` — that's acceptable but must be deliberate. If `started_by_display_name` requires an Agent-table join (N+1 read), pause via AskUserQuestion before shipping; acceptable fallback is to use raw `PrincipalRef` and defer display-name to renderer.

**Confidence target.** **≥ 97%** (scope-expansion phase per gate-2.5 user-locks; 9 deliverables; well-bounded but wider than v3's 5-deliverable shape).

### Phase P-DOCS — Ops runbook + troubleshooting + phi-core reuse map refresh + drift file + ADR-0059 + concept-audit-matrix + drift README + forward-scope amendment

**Goal.** v2 baseline goal preserved + v3 additions + **v4 updates**: drift file body cites the dedicated method + RecentSessionEntry; ADR-0059 §D59.2 + §D59.3 wording matches the gate-2.5 user-locks; phi-core-reuse-map gains a note that `RecentSessionEntry` is baby-phi-defined.

**Deliverables.** (v2 deliverables 1–5 preserved + v3/v4 additions below)

1. **NEW** `docs/specs/v0/implementation/m5/operations/m5-ops-runbook.md` (per F5.A lock + v2 plan).
2. **AMEND** `docs/specs/v0/implementation/m5/user-guide/troubleshooting.md` per F5.A lock + v3: add row for "page-11 shows empty recent_sessions" → resolution pointer.
3. **AMEND** `docs/specs/v0/implementation/m5/architecture/phi-core-reuse-map.md` per F3.A lock + **v4 note**: add a row or column entry noting that `RecentSessionEntry` (defined at `server::platform::projects::detail`) is baby-phi-defined, NOT a phi-core reuse. The underlying `Session` IS a phi-core reuse (wrapped at `domain/src/model/nodes.rs` per ADR-0029).
4. **AMEND** `docs/specs/v0/implementation/m7b/architecture/deferred-from-ch-k8s-prep.md` cross-reference.
5. **VERIFIED-HEADER BUMPS** per §3.C.
6. **(v3 NEW; v4 wording refined)** Create `docs/specs/v0/implementation/m5_1/drifts/D-CH24-recent-sessions-api-flip.md`:
   - Severity: MEDIUM.
   - Bucket: B (load-bearing API surface gap).
   - Status: `remediated` at planner draft (will Update at P-SEAL with seal-date).
   - **Closing chunk**: CH-24.
   - Body: Document the discovery context (P-NEW-TESTS authoring surfaced the M4 placeholder at `detail.rs:229+654`; inline doc-comment at `detail.rs:33` promised flip during M5/P4 but flip never landed); document the user gate-2 lock to close in-chunk; document the gate-2.5 user-lock for the dedicated method (F-D59.2.b) + RecentSessionEntry (F-D59.3.b); cross-reference ADR-0059 sub-decisions §D59.1–§D59.4; document the tripwire assertion flip at `acceptance_m5_sessions.rs:224-230`.
7. **(v3 NEW; v4 sub-decisions REWRITTEN)** Create `docs/specs/v0/implementation/m5_2/decisions/0059-recent-sessions-api-surface-flip.md`:
   - Status: `Proposed` at P-FLIP-RECENT-SESSIONS commit → `Accepted` at P-SEAL.
   - Sub-decisions §D59.1 (UNCHANGED), §D59.2 (REWRITTEN per v4), §D59.3 (REWRITTEN per v4), §D59.4 (UNCHANGED) per §5 above.
   - Cross-references ADR-0029, ADR-0034 (if extant) or fallback ADR-0029, ADR-0058.
8. **(v3 NEW)** Add row in `docs/specs/v0/implementation/m5_1/drifts/_concept-audit-matrix.md`:
   - Row entry: "page-11 recent_sessions panel" → drift `D-CH24-recent-sessions-api-flip` → status `silent-in-code` → `honored` (CH-24 close).
9. **(v3 NEW)** Add row in `docs/specs/v0/implementation/m5_1/drifts/README.md` index for the new drift.
10. **(v3 NEW; v4 wording refined)** Verify the `detail.rs:33` module doc-comment refresh from P-FLIP-RECENT-SESSIONS is honored (cross-reference to ADR-0059 §D59.1–§D59.3).
11. **(v3 NEW; v4 wording refined)** Body amendment at `m5/architecture/overview.md` OR new short page `m5_2/architecture/recent-sessions-panel.md` (decide at chunk-implementation open based on existing structure) — additive paragraph documenting `recent_sessions` panel semantics: cardinality bound 10 (query-side, not caller-side); ordering newest-first by `started_at`; freshness = on-read; dedicated Repository method `list_recent_sessions_for_project`; view-shape `RecentSessionEntry`; cross-reference ADR-0059.

**Tests.** No new tests in P-DOCS. CI guards verify.

**Confidence target.** **≥ 97%**.

### Phase P-SEAL — CI extensions + paperwork + milestone-tag staging + forward-scope amendment + ADR-0059 Accept-flip

**Goal.** v2 baseline + v3 additions + v4 unchanged: amend forward-scope row 211; flip ADR-0059 status `Proposed → Accepted`; close out drift status with seal-date.

**Deliverables.** (v2 deliverables 1–6 preserved + v3 additions; v4 unchanged from v3)

1. **AMEND** `.github/workflows/rust.yml` per F1.B (v2 baseline).
2. **AMEND** `docs/specs/v0/implementation/m5/README.md` per §3.C.
3. **INSERT row in `_cycle-index.md`** for CH-24 cycle.
4. **MIGRATE M5.1 drift catalogue to terminal status** — including the new `D-CH24-recent-sessions-api-flip` at `remediated`.
5. **STAGE milestone tag** `v0.1-m5` for user execution.
6. **WORKSPACE HEALTH CONFIRMATION**.
7. **(v3 NEW)** AMEND `docs/specs/plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md:211` — append a one-line note to the "Drifts closed: none new" cell acknowledging CH-24 closed 1 new drift (`D-CH24-recent-sessions-api-flip`) via in-cycle scope expansion. Cite ADR-0059. Note: this is the first time a CH-NN amends its own forward-scope row at chunk-seal due to mid-cycle scope expansion (CH-20's META amendment was for an off-by-N count; CH-24's is for a scope-expansion).
8. **(v3 NEW)** ADR-0059 status flip from `Proposed` → `Accepted` (1-line sed-style edit).

**Tests.** No new tests; final workspace health gate.

**Confidence target.** **≥ 99%**.

---

## §8 — Tests summary

### Expected total test count at chunk close

**Pre-CH-24 baseline**: 1529 (per CH-20 close + CH-21/22/23 deltas) — v2 P0 measurement.

**v2 NEW tests at v2 close**: 8 NEW (5 per-page acceptance scenarios across 5 files + 1 e2e + reduce 2 = 8 — actual measured: 1537/0/2 at v2 close).

**v3/v4 NEW tests at P-FLIP-RECENT-SESSIONS**: 0 (baseline) OR +1 (if deliverable 9 optional supplementary test ships). The dedicated repo method (`list_recent_sessions_for_project`) is exercised by the existing scenario transitively — no new dedicated unit test required.

**v4 band** (unchanged from v3):
- **Lower bound**: 1537 (no supplementary).
- **Upper bound**: 1539 if supplementary ships AND ±1 buffer; or 1540 if supplementary ships + count drift.

**Tripwire flip note** (NOT a test-count delta): `acceptance_m5_sessions.rs::m5_sessions_full_bootstrap_to_first_session_to_extraction_to_catalog` keeps its single `#[test]` function count (1); the assertion shape changes from `assert!(recent.is_empty())` to `assert_eq!(recent[0]["id"], session_id)`. **Same test, different assertion** — does not move the test count band.

**Final v4 target band**: **[1537, 1539]** (lower=1537 if optional supplementary deferred; upper=1539 if supplementary ships + 1-test buffer; 1540 if both ship + count drift). Outside this band → AskUserQuestion.

### MUST-SHIP vs MAY-COVER

**MUST-SHIP** (v2 baseline preserved) + v3/v4 adds:
- `acceptance_m5_sessions.rs::m5_sessions_full_bootstrap_to_first_session_to_extraction_to_catalog` tripwire FLIPPED (same test, assertion shape moved).

**MAY-COVER (v3 NEW)**:
- `acceptance_m5_sessions.rs::m5_sessions_recent_sessions_returns_at_most_10_when_more_present` (P-FLIP-RECENT-SESSIONS deliverable 9; ships only if launch helper supports rapid loop).

### Layer breakdown

- **Unit**: 0 new
- **Integration**: 0 or +1 new (v3 P-FLIP-RECENT-SESSIONS optional supplementary)
- **Property**: 0
- **Acceptance / E2E**: 0 or +1 (supplementary) — counted under integration

---

## §9 — Pre-chunk gate

(Unchanged from v2 — see plan history. v3 added 4 reading-list items; v4 refines reading-list to point at the trait surface that the new method extends + the consumer module.)

### v3+v4 reading-list items

- `docs/specs/v0/implementation/m5_2/decisions/0029-session-persistence-and-recorder-wrap.md` — ADR-0029 cross-reference for ADR-0059 lineage.
- `modules/crates/domain/src/repository.rs:1646–1670` — `list_sessions_in_project` Repository method docstring + signature (the predecessor method whose cascade shape ADR-0059 mirrors).
- `modules/crates/domain/src/repository.rs` module-level docstring (Repository trait contract block; verify trait extension boundary).
- `modules/crates/server/src/platform/projects/detail.rs:33+84-110+229+654` — the inline doc-comment + `RecentSessionStub` + 2 placeholder call-sites + `ProjectDetail.recent_sessions` field.
- `modules/crates/server/tests/acceptance_m5_sessions.rs:204–238` — the tripwire assertion + surrounding docstring to flip at P-FLIP-RECENT-SESSIONS.
- `modules/crates/domain/src/in_memory.rs:2384` — `list_sessions_in_project` in-memory impl as a template for the new `list_recent_sessions_for_project` impl shape.
- `modules/crates/store/src/repo_impl.rs:3321` — `list_sessions_in_project` SurrealStore impl as a SurrealQL template for the new method.

---

## §10 — Close criteria

### 4 aspects (each pass/fail)

- **Code aspect** — All phase deliverables shipped; v2 baseline preserved + v3 additions + v4 widens: NEW trait method `list_recent_sessions_for_project` extant at `domain/src/repository.rs`; NEW SurrealStore impl extant at `store/src/repo_impl.rs`; NEW InMemoryRepository impl extant at `domain/src/in_memory.rs`; NEW struct `RecentSessionEntry` extant at `detail.rs`; old `RecentSessionStub` DELETED (0 source hits); `detail.rs:229+654` both call the new method with LIMIT 10; tripwire FLIPPED at `acceptance_m5_sessions.rs:224-230`; supplementary test optional; `cargo test --workspace -j 4` passes at expected band [1537, 1539]; clippy + fmt green.
- **Docs aspect** — Governance + user-facing tiers per v2; v3 adds + v4 widens: drift file `D-CH24-recent-sessions-api-flip.md` extant with `Status: remediated` + body cites dedicated method + RecentSessionEntry; ADR-0059 extant with `Status: Accepted` at seal + §D59.2 + §D59.3 wording matches gate-2.5 user-locks; concept-audit-matrix row added; drift README index entry added; `detail.rs:33` module doc-comment refreshed; `m5/architecture/overview.md` (or `m5_2/architecture/recent-sessions-panel.md`) additive paragraph shipped; `m5/architecture/phi-core-reuse-map.md` notes `RecentSessionEntry` is baby-phi-defined; forward-scope row 211 amended.
- **phi-core leverage aspect** — §3 import-count delta = 0 (matches v4 prediction exactly); all forbidden-duplication greps return 0.
- **Concept alignment aspect** — §2 rows 1–6 all `honored` (v2 baseline); v3 row 7 transitions `silent-in-code → honored` (v4 refined to specify dedicated method + view-shape).

### 2 confidence % (named numerator/denominator)

- **Implementation confidence %** = `(claims-verified-honored) / (total-claims-in-scope)`. v2 numerator/denominator was **19/19**. v3 widened to **23/23**. **v4 widens to 25/25**:
  - v2's 19 claims preserved (5 carryover re-verifications + 7 per-page scenarios + 1 e2e + 5 prior-chunk invariants + 1 phi_core baseline).
  - v3's 4 claims preserved (20-23): recent_sessions wired at :229 + :654; tripwire flipped; ADR-0059 + drift accepted/remediated.
  - **v4 NEW claims (+2)**: (24) new trait method `list_recent_sessions_for_project` extant across 3 surfaces (trait + 2 impls); (25) new struct `RecentSessionEntry` extant + old `RecentSessionStub` deleted (0 source hits).
  - **Target ≥ 99% (24/25 minimum tolerable)**.
- **Documentation confidence %** = `(doc-pages-cross-checked) / (doc-pages-touched)`. v2 was **13/13**. v3 widened to **18/18**. **v4 widens to 20/20**:
  - v2's 13 preserved.
  - v3's 5 preserved (14-18): D-CH24 drift, ADR-0059, concept-audit-matrix row, drift README index, overview.md additive paragraph / detail.rs:33 module doc.
  - **v4 NEW docs (+2)**: (19) `m5/architecture/phi-core-reuse-map.md` `RecentSessionEntry` baby-phi-defined note; (20) `RecentSessionEntry` struct docstring at `detail.rs:84`.
  - **Target 20/20 = 100%**.

### Composite

**Composite = min(impl%, doc%, code, phi-core, concept-alignment).** Target: **≥99%**. Composite below 99% blocks close.

### Explicit close-target discipline

(Unchanged — close report states all 5 measures with named numerators/denominators.)

### P4 chunk-seal paperwork checklist (chunk-planner v3 + v5)

(Unchanged from v2 + v3 update: `_concept-audit-matrix.md` row touches: **1 expected** for the "page-11 recent_sessions panel" row.)

---

## §11 — Post-chunk independent audit plan

### Agent count

Per F4 lock: **3 auditors**.

### Audit agent prompts drafted here (v4 updates)

#### Audit-A — Rust correctness (≤ 600 words; v4 widens to ~25 claims)

(v2 claims 1–5 preserved + v3 claim 6 + v4 widens claim 6.)

> **Claim 6 (v3 NEW; v4 REWRITTEN/widened)**: P-FLIP-RECENT-SESSIONS deliverables shipped per gate-2.5 user-locks: (a) NEW trait method `Repository::list_recent_sessions_for_project(project_id: ProjectId, limit: u32) -> RepositoryResult<Vec<RecentSessionEntry>>` extant at `modules/crates/domain/src/repository.rs` (verify by grep + reading the trait def); (b) NEW SurrealStore impl body extant at `modules/crates/store/src/repo_impl.rs` (verify by grep + reading the impl block — confirm SurrealQL `LIMIT $limit` clause is present, i.e. cap is query-side not caller-side); (c) NEW InMemoryRepository impl body extant at `modules/crates/domain/src/in_memory.rs` (verify by grep + reading the impl block — confirm filter + sort + take(limit) chain mirrors SurrealQL contract); (d) NEW struct `RecentSessionEntry` extant at `server/src/platform/projects/detail.rs` with 7 fields per F-D59.3.b lock (id: SessionId, project_id: ProjectId, agent_id: AgentId, started_at: DateTime<Utc>, ended_at: Option<DateTime<Utc>>, status: String, started_by_display_name: String); (e) old `RecentSessionStub` DELETED — repo-wide source grep returns 0 hits; (f) `detail.rs:229` calls `repo.list_recent_sessions_for_project(project_id, 10).await` with the LIMIT 10 constant inlined per §D59.1; (g) `detail.rs:654` either similarly wired OR deliberately preserved-empty with contextual comment (verify by reading the surrounding control-flow); (h) the tripwire assertion at `acceptance_m5_sessions.rs:224-230` now asserts `recent.len() ≥ 1` AND `recent[0]["id"]` contains the launched session_id (verify by reading the assertion); (i) ADR-0059 extant at `m5_2/decisions/0059-recent-sessions-api-surface-flip.md` with `Status: Accepted` and 4 sub-decisions §D59.1–§D59.4; (j) drift `D-CH24-recent-sessions-api-flip.md` extant at `m5_1/drifts/` with `Status: remediated`. Pass criterion: all 10 sub-claims (a–j) PASS.

Pass criterion: 25/25 claims PASS (19/25 minimum tolerable).

#### Audit-B — Docs fidelity (≤ 600 words; v4 widens to 9 claims)

(v2 claims 1–7 preserved + v3 claim 8 + v4 widens claim 8 + adds claim 9.)

> **Claim 8 (v3 NEW; v4 REWRITTEN/widened)**: ADR-0059 + drift file fidelity per gate-2.5 user-locks. Verify: (a) `m5_2/decisions/0059-recent-sessions-api-surface-flip.md` exists with status `Accepted` + 4 sub-decisions §D59.1–§D59.4 + cross-references to ADR-0029 and ADR-0058; (b) §D59.2 sub-decision body specifies `list_recent_sessions_for_project(project_id: ProjectId, limit: u32) -> RepositoryResult<Vec<RecentSessionEntry>>` (matches shipped code at `domain/src/repository.rs`); (c) §D59.3 sub-decision body specifies the 7-field `RecentSessionEntry` shape (matches shipped code at `detail.rs`); (d) `m5_1/drifts/D-CH24-recent-sessions-api-flip.md` exists with status `remediated` + Closing chunk: CH-24 + body cross-references ADR-0059 sub-decisions + cites the dedicated method + RecentSessionEntry; (e) `m5_1/drifts/_concept-audit-matrix.md` has a new row for "page-11 recent_sessions panel" mapping the drift; (f) `m5_1/drifts/README.md` index has an entry for the new drift; (g) `server/src/platform/projects/detail.rs:33` module doc-comment refreshed — no longer reads "deferred to M5 per D11"; reads current reality per ADR-0059 §D59.1–§D59.3; (h) forward-scope row line 211 amended with one-line note citing CH-24 + ADR-0059; (i) `m5/architecture/overview.md` (or `m5_2/architecture/recent-sessions-panel.md`) additive paragraph documents `recent_sessions` panel semantics with cardinality bound + dedicated method + view-shape; (j) `m5/architecture/phi-core-reuse-map.md` notes `RecentSessionEntry` is baby-phi-defined (not a phi-core reuse). Pass criterion: all 10 sub-claims (a–j) PASS.

> **Claim 9 (v4 NEW)**: Stale-narrative sweep — repo-wide grep for `RecentSessionStub` returns 0 hits in source code (`modules/crates/`); archival doc references (e.g., `m4/architecture/phi-core-reuse-map.md`) may remain as historical mentions but the audit log must enumerate them. Pass criterion: 0 source hits + audit log enumerates any doc-archival hits.

Pass criterion: 9/9 claims PASS.

#### Audit-C — Vertical-slice integrity (≤ 600 words; v4 widens claim 8)

(v2 claims 1–7 preserved + v3 claim 8 + v4 widens claim 8 wording.)

> **Claim 8 (v3 NEW; v4 REWRITTEN)**: Vertical-slice integrity of recent_sessions panel from API → page-11 web UI surface, post-v4 dedicated method + RecentSessionEntry. Verify: (a) API surface: `GET /api/v0/projects/:id` returns `recent_sessions` array populated with `RecentSessionEntry` rows (NOT `RecentSessionStub`) after a session launch (verified by `acceptance_m5_sessions.rs:m5_sessions_full_bootstrap_to_first_session_to_extraction_to_catalog` post-flip); the JSON shape matches the 7-field RecentSessionEntry contract; (b) the new repo method `list_recent_sessions_for_project` is exercised on the API path (verify by reading `detail.rs:229` call-site); (c) CLI surface: `phi project show :id` displays recent_sessions if the renderer code path exists — IF the CLI does not yet consume this panel, document as `out-of-scope for CH-24 / defer to M6 CLI surface chunk`; (d) Web surface: page-11 web component consumes the API field — IF the M5 web does not yet render this panel, document as `out-of-scope for CH-24 / defer to M6 web surface chunk`; (e) the LIMIT 10 cardinality bound is enforced at the query layer (verify by reading the SurrealQL `LIMIT $limit` clause OR by running the optional supplementary test from P-FLIP-RECENT-SESSIONS deliverable 9 if shipped). Pass criterion: API surface (a) + (b) + (e) PASS; CLI + Web (c, d) either PASS or explicitly OUT-OF-SCOPE.

Pass criterion: 8/8 claims PASS.

---

## §12 — Verification section (end-to-end recipe)

(v2 recipe preserved verbatim — see plan history. v3 appended new steps; v4 widens those steps.)

```bash
# v4 NEW — list_recent_sessions_for_project trait method + impls
grep -n "list_recent_sessions_for_project" /root/projects/phi/baby-phi/modules/crates/domain/src/repository.rs
# expect: 1 trait method declaration

grep -n "list_recent_sessions_for_project" /root/projects/phi/baby-phi/modules/crates/store/src/repo_impl.rs
# expect: 1 impl method body (verify "LIMIT $limit" appears in the SurrealQL body)

grep -n "list_recent_sessions_for_project" /root/projects/phi/baby-phi/modules/crates/domain/src/in_memory.rs
# expect: 1 impl method body (test fixture path)

# v4 NEW — RecentSessionEntry struct + RecentSessionStub deletion
grep -rn "RecentSessionEntry" /root/projects/phi/baby-phi/modules/crates/server/
# expect: ≥ 3 hits — struct definition + 2 call-sites at detail.rs:229+654

grep -rn "RecentSessionStub" /root/projects/phi/baby-phi/modules/crates/ /root/projects/phi/baby-phi/docs/
# expect: 0 hits in source (fully replaced); archival doc mentions (e.g., m4/architecture/phi-core-reuse-map.md) may remain — context-check

# v3-baseline preserved — recent_sessions API flip verification
grep -n "recent_sessions" /root/projects/phi/baby-phi/modules/crates/server/src/platform/projects/detail.rs
# expect: ≥ 2 hits at lines 229 + 654; both reading 'list_recent_sessions_for_project' (NOT 'Vec::new()')

grep -c "Vec::new()" /root/projects/phi/baby-phi/modules/crates/server/src/platform/projects/detail.rs
# expect: 0 in lines 229+654 specifically (other Vec::new() uses in tests:679 may remain — context-check)

ls /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_2/decisions/0059-recent-sessions-api-surface-flip.md
# expect: file exists

ls /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_1/drifts/D-CH24-recent-sessions-api-flip.md
# expect: file exists

grep -c "Status: Accepted" /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_2/decisions/0059*.md
# expect: 1 at chunk seal (Status flipped Proposed → Accepted at P-SEAL deliverable 8)

grep -c "Status: remediated" /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_1/drifts/D-CH24-recent-sessions-api-flip.md
# expect: 1

grep -n "recent_sessions" /root/projects/phi/baby-phi/modules/crates/server/tests/acceptance_m5_sessions.rs
# expect: tripwire FLIPPED — no longer reads "M4 placeholder still active"; reads "real query wired at CH-24 P-FLIP-RECENT-SESSIONS via list_recent_sessions_for_project + RecentSessionEntry"

grep -n "recent_sessions" /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_1/drifts/_concept-audit-matrix.md
# expect: ≥ 1 hit (new row for "page-11 recent_sessions panel")

grep -n "deferred to M5 per D11" /root/projects/phi/baby-phi/modules/crates/server/src/platform/projects/detail.rs
# expect: 0 hits (stale wording removed at P-FLIP-RECENT-SESSIONS deliverable 7)

# v4 NEW — phi-core-reuse-map note for RecentSessionEntry
grep -n "RecentSessionEntry" /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5/architecture/phi-core-reuse-map.md
# expect: ≥ 1 hit (the baby-phi-defined note)
```

---

## §13 — Process novelty + retrospective routing (v3 NEW; v4 EXTENDED)

CH-24 is the **first cycle to scope-expand mid-flight via a P-NEW-TESTS authoring finding** (v3 finding) **AND the first cycle to have a 2-of-3 gate-2.5 divergent user-lock** (v4 finding). The expansion was user-locked at gate-2, not gate-1; the planner did NOT surface this fork at plan-draft because the API-surface placeholder was not within the v1/v2 plan's verification scope (the carryover ID C-M5-3 passed P-VERIFY as expected against persistence semantics; the API-surface flip half is a separate sub-piece that the M5/P4 plan committed but didn't ship). At gate-2.5, the planner-recommendations for F-D59.2 (reuse `list_sessions_in_project`) and F-D59.3 (reuse `RecentSessionStub`) BOTH followed the chunk-planner v10 "amend-don't-add" default; both were overridden by user-locks toward richer / more-defensive options. The pattern is now consistent with the 5-of-7 gate-1 divergence pattern — totalling 7-of-9 across cycles.

### Process-update candidates for retrospective routing

1. **Scope-expansion lane for milestone-seal / ratification chunks** (v3 ORIGINAL) — should the planner reserve a section in the plan template for "P-NEW-TESTS surfacing pattern" where tripwire-assertion writing is given explicit space to discover M-deferred sub-pieces? OR should the default be "tripwire assertion → file drift → defer to next chunk"? The CH-24 user-lock chose close-in-chunk; future chunks may benefit from a documented procedure for the choice.

2. **Forward-scope row drift-count amendment at P-SEAL** (v3 ORIGINAL) — CH-20 amended its forward-scope row at META; CH-24 amends its row for scope-expansion. The pattern of "chunks amend their own forward-scope row at chunk-seal" is now 2-cycle precedent; codify as a chunk-planner v13 rule? Specifically: when a chunk's actual delivered scope differs from the forward-scope row literal text (in EITHER direction — more or less), the planner MUST plan a P-SEAL deliverable for the amendment.

3. **Gate-2.5 fork-lock cadence** (v3 ORIGINAL; v4 EXTENDED) — CH-24 introduces 3 forks (F-D59.1.bound, F-D59.2.method, F-D59.3.shape) for gate-2.5 lock. This is a new gate position. Should the orchestrator gate model add a gate-2.5 step (between gate-2 implementation review and gate-3 audit review)? Currently the gates are 1-5; gate-2.5 is implicit in scope-expansion cycles. **v4 extension**: should the chunk-planner v12 divergence-pattern callout pre-flight on within-cycle divergence-prediction as well? Given the CH-24 data point (2-of-3 gate-2.5 divergent locks within a single cycle), the divergence pattern is not just cross-cycle but also within-cycle for any cycle that introduces forks at multiple gates. Codify in chunk-planner v13 / orchestrator gate sequence.

4. **Mid-cycle architectural scope expansion vs Architectural FAIL** (v3 ORIGINAL) — the audit-fix loop has 4 FAIL tiers (Trivial-1L, Trivial-multi, Tactical, Architectural). Mid-cycle scope expansion is technically a 5th tier — user-initiated scope addition that is not a failure but a deliberate widening. Should the orchestrator's audit-fix loop nomenclature add "Mid-cycle Scope Widening" as an explicit category? CH-24's retrospective should weigh in.

5. **Invert "amend-don't-add" default for milestone-seal / load-bearing-surface chunks?** (v4 NEW) — chunk-planner v10's "amend-don't-add" default biased the F-D59.2 and F-D59.3 recommendations toward reuse-existing paths; both were overridden by user-locks toward richer / more-defensive options. The pattern (7-of-9 cross-cycle divergent forks; 2-of-3 within-cycle gate-2.5 divergent locks) suggests the user's instinct for milestone-seal / API-surface chunks is consistently the opposite of amend-don't-add. Should chunk-planner v13 invert the default for this chunk class (milestone-seal AND load-bearing-surface AND first-real-population AND user-facing-API)? Retrospective should propose a decision rule: under what conjunction of chunk attributes does the default flip?

6. **v3+ gate-2.5 fork bank as planner-default** (v4 NEW) — should the per-chunk-planning-template add a "gate-2.5 fork bank" section by default, allowing planners to draft forks for surfaces that may be discovered mid-cycle? CH-24 demonstrated that mid-cycle scope expansion can be productive; a fork bank would let the planner anticipate likely expansion vectors and pre-draft the option-tree. The trade-off is plan length inflation — a chunk-planner v13 design decision.

### Cross-references

- v2 plan locks: F1.B (5 per-page test files), F2.A (subprocess), F3.A (medium reuse-map refresh), F4.B (3 auditors), F5.A (amend-don't-add troubleshooting + new runbook).
- v3 forks at gate-2.5 (planner-recommendations): F-D59.1.a (LIMIT 10), F-D59.2.a (reuse `list_sessions_in_project`), F-D59.3.a (reuse `RecentSessionStub`).
- **v4 forks at gate-2.5 (user-locks)**: F-D59.1.a (LIMIT 10 — aligned with planner), **F-D59.2.b (dedicated method `list_recent_sessions_for_project` — DIVERGENT)**, **F-D59.3.b (replace with `RecentSessionEntry` — DIVERGENT)**.
- ADR-0059 (NEW); D-CH24-recent-sessions-api-flip (NEW).
- Forward-scope row 211 amendment at P-SEAL deliverable 7.

---

## Cross-references

- **Concept docs**: see §2 table.
- **Drifts (v3 NEW)**: `D-CH24-recent-sessions-api-flip` at `m5_1/drifts/D-CH24-recent-sessions-api-flip.md`.
- **ADRs (v3 NEW; v4 sub-decisions REWRITTEN)**: `ADR-0059 — recent-sessions-api-surface-flip` at `m5_2/decisions/0059-recent-sessions-api-surface-flip.md`.
- **Prior ADRs cited as precedent**:
  - ADR-0029 (Session persistence + SessionRecorder wrap) — `m5/decisions/0029-session-persistence-and-recorder-wrap.md`
  - ADR-0030 (Template-node uniqueness) — `m5/decisions/0030-template-node-uniqueness.md`
  - ADR-0031 (Session cancellation + concurrency bounds) — `m5/decisions/0031-session-cancellation-and-concurrency.md`
  - ADR-0033 (CH-K8S-PREP K8s readiness) — `m7b/decisions/0033-k8s-readiness.md`
  - ADR-0058 (Bucket C confirm-in-place) — `m5_2/decisions/0058-bucket-c-convention-confirm-in-place.md` (v3 precedent for ratifying previously-deferred decisions in-cycle).
- **Forward-scope row**: `docs/specs/plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md` line 210–212 (v3 amendment at P-SEAL deliverable 7).
- **M5 archive plan §P9**: `docs/specs/plan/build/m5-templates-system-agents-sessions-01710c13.md` line 798–852.
