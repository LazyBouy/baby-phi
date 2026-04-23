<!-- Last verified: 2026-04-23 by Claude Code -->

# M5/P0 — M4 post-flight delta audit

**Status**: [EXISTS] since M5/P0 (planning phase).

This doc is the **10-item audit** that M5/P0 runs before P1 opens. It
verifies M4 shipped as specified and that every M5 carryover from the
base plan + this plan's Part 1 gap list (G1–G19) is still in the state
M5/P1 expects.

Target at P0 close: **0 stale items**. Any stale item here opens a
P0.5 remediation before P1 opens.

---

## 1. M4 test-count baseline holds

- `cargo test --workspace` last green at M4/P8 close: **805 Rust +
  68 Web = 873 combined** (per M4 README line 52 target + post-P8
  follow-up expansion of `acceptance_m4.rs` to 4 scenarios after the
  cross-org isolation gap fix).
- P0 spot-check: `modules/crates/server/tests/acceptance_m4.rs`
  exists with **4 scenarios** (`full_m4_happy_path_bootstrap_to_dashboard`,
  `project_lead_viewer_role_surfaces_on_dashboard`,
  `cross_org_project_show_denies_foreign_viewer`,
  `dashboard_shape_counters_are_org_scoped`). ✅ aligned.

## 2. C-M5-1 — `Template` UNIQUE(name) still blocking multi-org adoption

- Migration 0001 (`template_name` UNIQUE) still in effect; no later
  migration drops it.
- `domain/src/model/nodes.rs::Template` remains a platform-level row
  (no `adopted_by_org` / `adopted_at` fields added — correct per D1's
  decision against option (c)).
- M5/P1 migration 0005 owns the DROP + redefine. ✅ still deferred.

## 3. C-M5-2 — `uses_model` edge still mis-typed

- `store/migrations/0001_initial.surql:315` (approx) defines
  `uses_model` as `FROM agent TO model_config`.
- M2/P6's `model_runtime` catalogue is the live home; `model_config`
  table is vestigial.
- No M4 migration touches this. ✅ still deferred; M5/P1 migration 0005
  redefines.

## 4. C-M5-3 — Session / Loop / Turn scaffolds still id-only

- `domain/src/model/nodes.rs` carries id-only placeholder structs for
  Session / Loop / Turn marked `[PLANNED M5]`.
- `edges.rs::RunsSession` variant exists but has **zero production
  writers** (a compile-time-only stub at M4).
- ✅ still deferred; M5/P1 replaces with the 3-way wrap (ADR-0029).

## 5. C-M5-4 — no AgentTool resolver in platform tree

- `server/src/platform/sessions/` directory **does not exist** at M4
  close.
- No code imports `phi_core::types::tool::AgentTool` anywhere under
  `modules/crates/`.
- ✅ still deferred; M5/P4 adds the resolver + `GET /sessions/:id/tools`.

## 6. C-M5-5 — `count_active_sessions_for_agent` still a stub

- `modules/crates/domain/src/repository.rs:849–851`:
  ```rust
  async fn count_active_sessions_for_agent(&self, _agent: AgentId) -> RepositoryResult<u32> {
      Ok(0)
  }
  ```
  Default method body returning `Ok(0)`. ✅ confirmed still a stub.
- `agent_profile.model_config_id` field is **absent** from
  `AgentProfile` struct (checked via grep on `nodes.rs`). ✅ confirmed.
- M4's `ACTIVE_SESSIONS_BLOCK_MODEL_CHANGE` error-code variant is
  pre-wired but unreachable until the stub flips.
- M5/P2 flips the stub; M5/P4 flips the `update.rs` change arm.

## 7. C-M5-6 — `_keep_materialise_live` dead-code marker still present

- `modules/crates/server/src/platform/projects/create.rs:754–756`
  still carries the `_keep_materialise_live` function (a
  `NodeId`-keep-alive hack that forces the compiler to retain
  `materialise_project` despite zero callers at M4).
- `approve_pending_shape_b` Approved branch at line 661 + line 669
  returns `project: None` — the both-approve materialisation is not
  wired.
- No `shape_b_pending_projects` table exists in any migration. ✅
  confirmed deferred.
- M5/P1 adds the sidecar table; M5/P4 flips the Approved branch +
  deletes the dead-code marker.

## 8. M4 ADRs 0024 / 0025 / 0027 / 0028 all **Accepted**

- `m4/decisions/0024-project-and-agent-role-typing.md` — Accepted
  (P1 close).
- `m4/decisions/0025-shape-b-two-approver-flow.md` — Accepted
  (P6 close).
- `m4/decisions/0027-per-agent-execution-limits-override.md` —
  Accepted (P5 close).
- `m4/decisions/0028-domain-event-bus.md` — Accepted (P3 close).
- (0026 is not used; ADR numbering is monotonic but not contiguous —
  M2 skipped from 0021→0022, similar precedent.)
- ✅ all four files carry `Status: Accepted` at top.

## 9. phi-core leverage baseline

- `grep -rn '^use phi_core::' modules/crates/ | wc -l` → **14
  lines** at M4/P8 close baseline (~7 unique types:
  `AgentProfile`, `ExecutionLimits`, `ModelConfig`, `ThinkingLevel`,
  `ProjectShape` + `AgentRole` enums, `OrganizationDefaultsSnapshot`
  transitively).
- M5 target: **~24 lines / 10 unique types** by P9 close (adds
  `Session`, `LoopRecord`, `Turn`, `AgentEvent`, `SessionRecorder`,
  `AgentTool`, `agent_loop`).
- `scripts/check-phi-core-reuse.sh` green at M4/P8 close; denylist
  extended at M5/P0 (this phase) with 6 new tokens.
- ✅ baseline pinned; P9 verification target set.

## 10. No unexpected drift since M4/P8 close

- `git log --oneline main..HEAD` (on baby-phi submodule) shows **0
  commits** between M4 milestone tag and M5/P0 open (M4 closed clean
  + the post-P8 cross-org follow-up landed before the tag).
- No open PRs touching `modules/crates/` or `modules/web/`.
- No dangling TODO/FIXME comments added since M4/P8 close (spot-check
  via `git diff` on last 2 working commits).
- ✅ clean boundary; M5 opens on a stable base.

---

## Audit outcome

**10 / 10 items confirmed stable**. 0 stale items. P0 can proceed to
ADR drafting + base-plan amendment + CI grep extension + docs seed
without remediation.

## Cross-references

- [M5 plan archive](../../../../plan/build/01710c13-m5-templates-system-agents-sessions.md) — the full P0–P9 plan this delta opens.
- [M4 README](../../m4/README.md) — phase status + ADRs.
- [Base plan §M5 carryovers](../../../../plan/build/36d0c6c5-build-plan-v01.md) — C-M5-1 through C-M5-6.
- [phi-core reuse map (M4)](../../m4/architecture/phi-core-reuse-map.md) — prior import baseline.
