<!-- Last verified: 2026-05-11 by Claude Code (CH-24 P-DOCS — drift drafted with Status: remediated. Mid-cycle scope-expansion approved at gate-2 user-lock. Closing chunk: CH-24. Drift body cites the dedicated repo method (per ADR-0059 §D59.2 user-divergent F-D59.2.b lock) + `RecentSessionEntry` view-shape struct (per ADR-0059 §D59.3 user-divergent F-D59.3.b lock) + LIMIT 10 query-side cap (per ADR-0059 §D59.1 planner-aligned F-D59.1.a lock). 6-field shape shipped; `started_by_display_name` deferred per ADR-0059 §D59.3-FOLLOWUP. Cycle hex `5778bb77`.) -->

# D-CH24-recent-sessions-api-flip — Page-11 `recent_sessions` panel hardcoded to `Vec::new()` since M4 (API-surface half of C-M5-3 never landed during M5/P4)

## Identification
- **ID**: D-CH24-recent-sessions-api-flip
- **Phase of origin**: `chunk/CH-24` (discovered mid-chunk during P-NEW-TESTS authoring; remediated in-chunk via P-FLIP-RECENT-SESSIONS phase under user gate-2 lock)
- **Discovery source**: `mid-chunk-pause`
- **Date discovered**: 2026-05-11
- **Status**: `remediated`
- **Bucket**: `B — underspecified shape choice` (the M4 placeholder + M5/P4 carryover-close was load-bearing for the API surface but the shape choice for the flip — caller-side slice vs dedicated method, reuse-stub vs richer struct — was underspecified)
- **Severity**: `MEDIUM`
- **Tags**: `api-surface-gap`, `m4-placeholder`, `mid-cycle-scope-expansion`, `concept-silent`, `bucket-b`
- **Blocks**: none
- **Blocked-by**: none

## Concept alignment
- **Concept doc(s)**: [`concepts/phi-core-mapping.md`](../../../concepts/phi-core-mapping.md) §"Session list query at API surface" (the page-11 panel as a `phi_core::session::model::Session` consumer)
- **Concept claim (close paraphrase with code-comment cite)**: The page-11 `recent_sessions` panel SHOULD query persisted `Session` rows for the project, ordered newest-first, bounded for panel UX. The pre-CH-24 inline doc-comment at [`server/src/platform/projects/detail.rs:33`](../../../../../../modules/crates/server/src/platform/projects/detail.rs) promised: *"deferred to M5 per D11; the `recent_sessions` field in [`ProjectDetail`] is a placeholder `Vec::new()` until M5 wires baby-phi's governance `Session` node."*
- **Contradiction**: The carryover C-M5-3 ("Session persistence") closed during M5/P4 against the persistence half (rows materialise on launch + finalisation, verified by re-running carryover scenarios against real `phi_core::agent_loop` output post-CH-02). The **API-surface half** — wiring the panel to a real query reading those persisted rows — never landed. The placeholder + the inline doc-comment promising the flip survived through CH-23 close (M5/P9).
- **Classification**: `concept-silent-plan-filled-gap` (concept-doc `phi-core-mapping.md` does not name the panel's query path; the M5/P4 plan committed to flip the placeholder but the deliverable was never tracked)
- **phi-core leverage status**: `direct-reuse` (the new query reads `phi_core::session::model::Session` via the existing wrap at `domain/src/model/nodes.rs` per ADR-0029; zero new `use phi_core` lines)

## Plan vs. reality
- **Plan said** (M5 plan archive at [`build/m5-templates-system-agents-sessions-01710c13.md`](../../../../plan/build/m5-templates-system-agents-sessions-01710c13.md) §P4 page-14 carryover-close list, paraphrased — the M5/P4 plan listed C-M5-3 as "Session persistence" and the page-11 panel as a downstream consumer of that persistence, but did NOT itemise the flip of `Vec::new()` placeholder to real query as a distinct deliverable. The inline doc-comment at `detail.rs:33` is the de-facto plan-promise.)
- **Reality (shipped state at pre-CH-24 HEAD)**: `server/src/platform/projects/detail.rs:229` (primary `project_detail` path) and `:654` (test-side fixture) both hardcode `recent_sessions: Vec::new()`. The carryover scenarios C-M5-3 PASS on the persistence half but were never asserted against the API surface — `recent_sessions` had no test exercising it beyond the snapshot test that asserts absence of phi-core fields (not presence of session rows).
- **Root cause**: `cascading-upstream-deferral` — C-M5-3 was scoped as "Session persistence"; the API-surface flip was implicitly a sub-piece but the M5/P4 plan did not enumerate it as a distinct deliverable; reviewer-gap at P4 close didn't surface the un-flipped placeholder; the snapshot test was the closest invariant and it doesn't assert content.

## Where visible in code
- **File(s)**:
  - [`modules/crates/server/src/platform/projects/detail.rs:33`](../../../../../../modules/crates/server/src/platform/projects/detail.rs) — module doc-comment promised the M5 flip (refreshed at CH-24 P-FLIP-RECENT-SESSIONS to cite ADR-0059).
  - [`modules/crates/server/src/platform/projects/detail.rs:84-95`](../../../../../../modules/crates/server/src/platform/projects/detail.rs) — old `RecentSessionStub` struct (DELETED at CH-24 P-FLIP-RECENT-SESSIONS; replaced by `RecentSessionEntry` at the domain composites tier).
  - [`modules/crates/server/src/platform/projects/detail.rs:229`](../../../../../../modules/crates/server/src/platform/projects/detail.rs) — primary call-site (REWIRED at CH-24 to call `repo.list_recent_sessions_for_project(project_id, RECENT_SESSIONS_LIMIT).await`).
  - [`modules/crates/server/src/platform/projects/detail.rs:654, :696`](../../../../../../modules/crates/server/src/platform/projects/detail.rs) — test-side fixtures (left as `Vec::new()` deliberately — these are unit tests on the wire-shape and need an empty panel to assert phi-core-stripping invariants; refreshed comments cite ADR-0059).
  - [`modules/crates/domain/src/model/composites_m5.rs:225`](../../../../../../modules/crates/domain/src/model/composites_m5.rs) — NEW `RecentSessionEntry` struct (baby-phi-defined view-shape; 6 fields shipped; placement at composites tier so the `Repository` trait can name it directly).
  - [`modules/crates/domain/src/repository.rs`](../../../../../../modules/crates/domain/src/repository.rs) — NEW `Repository::list_recent_sessions_for_project` trait method declaration.
  - [`modules/crates/store/src/repo_impl.rs`](../../../../../../modules/crates/store/src/repo_impl.rs) — NEW `SurrealStore::list_recent_sessions_for_project` impl body (SurrealQL `SELECT * FROM session WHERE project_id = $project ORDER BY started_at DESC LIMIT $limit`).
  - [`modules/crates/domain/src/in_memory.rs`](../../../../../../modules/crates/domain/src/in_memory.rs) — NEW `InMemoryRepository::list_recent_sessions_for_project` impl body (Vec filter + sort + take).
  - [`modules/crates/server/tests/acceptance_m5_sessions.rs:224-230`](../../../../../../modules/crates/server/tests/acceptance_m5_sessions.rs) — tripwire FLIPPED from `assert!(recent.is_empty())` to `assert_eq!(recent.len(), 1)` + `assert_eq!(recent[0]["id"]…)`.
- **Test evidence**:
  - `m5_sessions_full_bootstrap_to_first_session_to_extraction_to_catalog` ([`acceptance_m5_sessions.rs`](../../../../../../modules/crates/server/tests/acceptance_m5_sessions.rs)) — PASS post-flip; the tripwire assertion now asserts the launched session appears in the panel with the expected `id` + non-null `started_at` + `status` enum.
  - `wire_shape_strips_phi_core` (snapshot test at `detail.rs::tests`) — PASS (unchanged — asserts absence of phi-core fields; the new `RecentSessionEntry` carries none).
  - `recent_sessions_defaults_empty_for_projects_with_no_launches` (new pure-struct invariant test at `detail.rs::tests`) — PASS.
- **Grep for regression**:
  ```bash
  # Should return 0 at any future HEAD — Vec::new() reappearing at :229 would re-introduce the gap.
  grep -n "recent_sessions: Vec::new()" /root/projects/phi/baby-phi/modules/crates/server/src/platform/projects/detail.rs | grep -v "tests\b"

  # Should return 0 — RecentSessionStub deletion is permanent.
  grep -rn "RecentSessionStub" /root/projects/phi/baby-phi/modules/crates/

  # Should return 1 trait + 2 impls — cascade integrity.
  grep -rn "fn list_recent_sessions_for_project" /root/projects/phi/baby-phi/modules/crates/
  ```

## Remediation scope
- **Approach (shipped at CH-24 P-FLIP-RECENT-SESSIONS)**:
  1. Add `Repository::list_recent_sessions_for_project(project_id, limit) -> Vec<RecentSessionEntry>` trait method.
  2. Implement on both backends (SurrealStore SurrealQL with query-side LIMIT + InMemoryRepository filter+sort+take).
  3. Add `RecentSessionEntry` 6-field view-shape struct at `domain::model::composites_m5` (defer `started_by_display_name` per §D59.3-FOLLOWUP — Agent-table join needed; defer to follow-up M6 chunk).
  4. Delete `RecentSessionStub` (no type-alias retained).
  5. Wire `detail.rs:229` to the new method with `RECENT_SESSIONS_LIMIT = 10` constant.
  6. Refresh `detail.rs:33` module doc-comment to cite ADR-0059 §D59.1–§D59.3.
  7. Flip tripwire assertion in `acceptance_m5_sessions.rs:224-230` from "asserts empty" to "asserts contains session".
- **Implementation chunk**: **CH-24** (this chunk).
- **Dependencies on other drifts**: none.
- **Estimated effort**: ~0.4 engineer-days (actual at chunk-close; expanded from v3's ~0.3 prediction due to user-divergent gate-2.5 locks widening cascade from 2 surfaces to 6-9).
- **Risk if deferred further**: HIGH at the moment of discovery — the milestone tag `v0.1-m5` would have shipped with a permanently-empty API field for a documented panel, surfacing publicly the first time any operator clicked page-11. Now zero risk (remediated in-chunk).

### Follow-up work (queued for future M6 chunk)

The 6-field `RecentSessionEntry` shape shipped is **forward-compatible** with a 7th field `started_by_display_name: String`. Adding it requires either (a) a SurrealQL `FETCH agent_id` clause + struct refit, or (b) a batch helper `Repository::get_agents_for(&[AgentId]) -> Vec<Agent>` paired with caller-side merge. Neither path is in scope for CH-24 without an N+1-read pause; both are clean follow-up candidates. Queued for the same follow-up chunk that closes the TypeScript-side wire-type cleanup at [`modules/web/lib/api/projects.ts:198`](../../../../../../modules/web/lib/api/projects.ts) (`RecentSessionStubWire` → `RecentSessionEntryWire`).

Tracked as part of M5.3 / M6 forward-scope; will land alongside other Agent-join optimisations.

## Prior documentation locations (pre-CH-24)
- Plan archive lines: M5 plan archive §P4 page-14 carryover-close list (paraphrase; no explicit deliverable enumeration).
- Code comments:
  - [`detail.rs:33`](../../../../../../modules/crates/server/src/platform/projects/detail.rs) — module doc-comment promising the M5 flip (now refreshed).
- ADR references: ADR-0059 (this chunk's ratification ADR).
- Other doc pointers: M4 phi-core reuse map at [`m4/architecture/phi-core-reuse-map.md:88`](../../m4/architecture/phi-core-reuse-map.md) (archival reference to the pre-CH-24 `RecentSessionStub` placeholder shape — historical record only; not patched).

## Lifecycle history
- 2026-05-11 — `discovered` — Surfaced during CH-24 P-NEW-TESTS authoring (drafting `m5_sessions_full_bootstrap_to_first_session_to_extraction_to_catalog`). Original test-author intent was a tripwire assertion `recent.is_empty()` codifying the M4 placeholder as load-bearing — pause via AskUserQuestion surfaced to user.
- 2026-05-11 — `classified → scoped → in-chunk-plan` (compressed) — User gate-2 lock at CH-24 mid-cycle: close-in-chunk via new phase **P-FLIP-RECENT-SESSIONS**. New ADR-0059 drafted with 4 sub-decisions §D59.1–§D59.4. Gate-2.5 forks (F-D59.1.bound + F-D59.2.method + F-D59.3.shape) drafted by planner; user-locked 2-of-3 divergent (F-D59.1.a aligned with planner-recommendation; F-D59.2.b + F-D59.3.b diverged toward richer / more-defensive options).
- 2026-05-11 — `remediated` — CH-24 P-FLIP-RECENT-SESSIONS shipped: NEW trait method + 2 impls + NEW struct (6 of 7 fields per orchestrator-approved gate-2 deviation — `started_by_display_name` deferred to follow-up chunk per §D59.3-FOLLOWUP) + 2 wire-consumer call-sites rewired + tripwire flipped + inline doc-comment refreshed. ADR-0059 ratified at P-SEAL (`Proposed → Accepted`).
