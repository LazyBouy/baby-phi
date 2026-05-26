<!-- Last verified: 2026-05-25 by Claude Code (CH-28d gate-4 final cycle re-audit — orchestrator-authored at Phase 4 close per outer CLAUDE.md gate-4 mandatory step; MUST-RUN list authoritatively executed; verdict PASS at all axes; 0 audit-fix-loop iterations; cycle hex `79b7386b`.) -->

# CH-28d cycle audit — phi-core 0.10.0 absorption (cycle `79b7386b`)

**Verdict at gate-4**: ✅ **PASS** — all 7 deliverables shipped + workspace GREEN on phi-core 0.10.x + 4 CI guards GREEN + 1 audit iteration with 0 FAILs + 0 orchestrator-applied patches + 0 implementer/planner re-spawns.

**Cycle metadata**:
- **Hex**: `79b7386b`
- **Chunk-type**: TECHNICAL-PREREQUISITE (phi-core 0.9 → 0.10.x baseline shift)
- **Severity**: LOW · **Effort**: 0.3–0.5 ed (matched actual)
- **Phases**: 4 (P1 + P2 + P3 + P-SEAL)
- **Audit envelope**: SMALL (1 auditor Letter A; per plan §11 sizing override rationale; 3rd consecutive activation for phi-core absorption chunks per CH-28b + CH-28c precedent)
- **Prerequisite**: CH-28c (cycle `40214078`, closed 2026-05-24 commit `8fc2018`) + prerequisite forward-scope commit `af037cf` 2026-05-25 (orchestrator Phase 0 Option A inline-draft)

---

## §1 — Audit pipeline summary

| Auditor | Iter | Verdict | PASS | FAIL | NOT-EXECUTED-IN-AUDIT | Trivial-1L | Trivial-multi | Tactical-FAIL | Architectural-FAIL |
|---|---|---|---|---|---|---|---|---|---|
| Letter A | iter-1 | **PASS** | 13 | 0 | 3 (clippy + tests + CI-guards; orchestrator-owned at gate-4) | 0 | 0 | 0 | 0 |

**Audit log**: [`audit-A-iter1.md`](audit-A-iter1.md) (auditor-authored; verdict PASS; spot-checks at file:line for Claims 3 + 4 + 5 + 8 + 9 + 10 + 14 + 15 + 16).

**3 NOT-EXECUTED-IN-AUDIT claims** (Claims 11 + 12 + a third related to the cargo invocations) closed at gate-4 by orchestrator MUST-RUN execution below (§3).

---

## §2 — Diff review (per `git -C /root/projects/phi/baby-phi diff` + untracked files at chunk close)

### Modified files (5)
| File | LOC Δ | What changed |
|---|---|---|
| `Cargo.toml:17` | +1/-1 | `phi-core = "0.9"` → `phi-core = "0.10"` (F2.a semver-range; 3rd consecutive activation) |
| `Cargo.lock:3357 region` | +2/-2 | phi-core entry version `"0.9.0"` → `"0.10.0"` + checksum update (1 dep updated; minimal transitive churn) |
| `docs/specs/plan/build/_cycle-index.md` | +1/-0 | CH-28d row inserted between CH-28c and `## Legacy cycles` heading (orchestrator-authored at Phase 1.7 archive; Status `in-flight`; Iterations `pending`) |
| `docs/specs/v0/feature-inventory.md` | +8/-0 | NEW §3 Deferred catalogue row for `D-PHICORE-10-FOLLOWUP-01` between `D-PHICORE-09-FOLLOWUP-01` + `D-CH28-FOLLOWUP-01` rows + verified-header prepended with CH-28d-stamped citation |
| `modules/crates/server/src/platform/sessions/launch.rs:577-593` | +17/-0 | 11-line CH-28d carrier-fix comment block citing ADR-0066 §D66.3 + D-PHICORE-10-FOLLOWUP-01 + ADR-0034 §D34.6 + 2 new field-adds (`revert_render_policy: phi_core::types::node_tag::RevertRenderPolicy::default()` + `current_tool: None`); position appended AFTER existing CH-28b `revert_pending: None,` block + BEFORE the closing `};` |

**Total: 5 files changed, 29 insertions(+), 3 deletions(-).**

### NEW files (3)
| File | LOC | What |
|---|---|---|
| `docs/specs/v0/implementation/m6/decisions/0066-phi-core-10-absorption.md` | 275 | ADR-0066 (Status: Accepted at P3 per CH-28c D-1 precedent); 5 sub-decisions §D66.1 (F2.a semver-range) + §D66.2 (F1.b interstitial 3rd consecutive) + §D66.3 (D3 2-field-add scope-narrow + ADR-0034 §D34.6 cross-ref) + §D66.4 (F4.a single drift covering 4 axes) + §D66.5 (async-Fn migration no-op consumer-side) |
| `docs/specs/v0/implementation/m6/drifts/D-PHICORE-10-FOLLOWUP-01-new-api-adoption-cluster.md` | 94 | NEW drift (Severity LOW; Status `discovered`; Bucket B; Closing chunk `M7+-FUTURE-ARCHITECTURE-SHIFT-OR-OBSERVABILITY-ADOPTION`); 4 axes A/B/C/D documented |
| `docs/specs/plan/build/ch-28d-phi-core-10-absorption-79b7386b/{plan.md, audit-A-iter1.md, cycle-audit.md}` | 767 + ~190 + this | Cycle-folder artifacts |

### High-level summary
- **Production code Δ**: +17 LOC at 1 file (`launch.rs`) — slightly above the +13 LOC prediction due to diff including context lines + blank-line spacing.
- **Cargo.lock churn**: 4 lines (extremely minimal — 1 dep updated to 0.10.0; smallest absorption-cycle churn since CH-28c's 4-line churn).
- **Governance docs Δ**: ~369 LOC NEW (ADR 275 + drift 94) + ~9 LOC at modified docs (feature-inventory row + cycle-index row).
- **Workspace test count**: 1599 → 1599 (Δ 0; zero new tests; zero regressions).
- **phi-core import baseline**: 57 → 57 raw `use phi_core` lines (Δ +0); +1 semantic leverage-site via FQN `phi_core::types::node_tag::RevertRenderPolicy::default()` at `launch.rs:592`.

---

## §3 — MUST-RUN evidence (orchestrator-authoritative, gate-4)

| Command | Outcome | Time | Notes |
|---|---|---|---|
| `RUSTFLAGS="-Dwarnings" cargo clippy -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml --workspace --all-targets` | ✅ exit 0 | 9m 30s | "Finished `dev` profile [unoptimized + debuginfo] target(s) in 9m 30s" — zero warnings, zero errors |
| `cargo test --manifest-path /root/projects/phi/baby-phi/Cargo.toml --workspace --no-fail-fast -j 4 > /tmp/ch28d-cargo-test.log 2>&1` | ✅ exit 0 | (background; ~3m) | **1599 passed / 0 failed / 2 ignored across 146 test binaries**; exact match to plan §6 baseline + CH-28c close |
| `cargo fmt --manifest-path /root/projects/phi/baby-phi/Cargo.toml --all -- --check` | ✅ exit 0 | <1s | Zero diff (no formatting drift) |
| `bash /root/projects/phi/baby-phi/scripts/check-doc-links.sh` | ✅ exit 0 | <2s | "all markdown under docs/specs/v0/implementation has valid relative links + verification headers. OK." |
| `bash /root/projects/phi/baby-phi/scripts/check-ops-doc-headers.sh` | ✅ exit 0 | <2s | "all 38 ops doc(s) carry the 'Last verified' header. OK." |
| `bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh` | ✅ exit 0 | <2s | "no forbidden phi-core redeclarations under modules/crates. OK." |
| `bash /root/projects/phi/baby-phi/scripts/check-spec-drift.sh` | ✅ exit 0 | <2s | "29 referenced ids all present in docs/specs/v0/requirements. OK." |

**All 7 MUST-RUN commands GREEN. The 3 audit Claims (11 + 12 + cargo-test) marked `NOT-EXECUTED-IN-AUDIT` by Letter A iter-1 are now closed.**

---

## §4 — Iteration accounting

- **Plan iterations**: 1 (iter-1 archived directly post-planner-spawn; no iter-2 re-spawn since all 4 F-locks resolved to planner-rec internally per user-direction "resolve forks internally" — gate-1.5 Step A skip-condition fires per outer CLAUDE.md P-orch-8: ALL forks lock at planner-rec AND iter-1 plan §1 carries populated bodies for every fork)
- **Audit iterations**: 1 (Letter A iter-1 PASS at first attempt)
- **Trivial-1L patches at gate-3**: 0
- **Trivial-multi patches at gate-3**: 0
- **Tactical FAILs**: 0
- **Architectural FAILs**: 0
- **Implementer re-spawns**: 0
- **Planner re-spawns**: 0
- **Auditor re-spawns**: 0

**Cleanest cycle since CH-28c** (which was the cleanest since CH-19/CH-20 zero-Trivial close streak). CH-28d preserves the streak: 1 plan iter + 1 audit iter + 0 patches + 0 re-spawns + 0 fix-loop iters across 4 phases.

---

## §5 — Paperwork ledger

| Artifact | Required by plan | Present? | Path |
|---|---|---|---|
| Cycle-folder | yes | ✅ | `docs/specs/plan/build/ch-28d-phi-core-10-absorption-79b7386b/` |
| `plan.md` (chunk-archive-plan output) | yes | ✅ | `…/ch-28d-phi-core-10-absorption-79b7386b/plan.md` (767 LOC; chunk-planner iter-1 v31; 12 sections + §1 Locked fork details inline; immutable per chunk-archive-plan) |
| `audit-A-iter1.md` | yes | ✅ | `…/ch-28d-phi-core-10-absorption-79b7386b/audit-A-iter1.md` (auditor-authored; PASS verdict) |
| `cycle-audit.md` (this file) | yes | ✅ | `…/ch-28d-phi-core-10-absorption-79b7386b/cycle-audit.md` (orchestrator-authored at gate-4 close) |
| Cycle-index row | yes | ✅ | `docs/specs/plan/build/_cycle-index.md` line 74 (Status `in-flight`; Iterations `pending`; orchestrator flips at Phase 7) |
| NEW ADR-0066 | yes | ✅ | `docs/specs/v0/implementation/m6/decisions/0066-phi-core-10-absorption.md` (275 LOC; Status: Accepted at P3 per CH-28c D-1 precedent; 5 sub-decisions) |
| NEW drift D-PHICORE-10-FOLLOWUP-01 | yes | ✅ | `docs/specs/v0/implementation/m6/drifts/D-PHICORE-10-FOLLOWUP-01-new-api-adoption-cluster.md` (94 LOC; 4 axes A/B/C/D) |
| feature-inventory.md §3 row | yes | ✅ | `docs/specs/v0/feature-inventory.md` (NEW row + verified-header prepended) |
| Forward-scope row | yes (already landed prerequisite) | ✅ | `docs/specs/plan/forward-scope/m6-forward-scope-8b7a8bcd.md:82-92` (landed at commit `af037cf` 2026-05-25; verified-header line 1 carries CH-28d citation) |
| launch.rs field-adds | yes | ✅ | `modules/crates/server/src/platform/sessions/launch.rs:577-593` (+17 LOC including comment block + 2 field-add lines) |
| Cargo.toml bump | yes | ✅ | `Cargo.toml:17` (`phi-core = "0.10"`) |
| Cargo.lock regen | yes | ✅ | `Cargo.lock` (phi-core entry version `"0.10.0"`) |
| Verified-headers on all modified docs | yes (per chunk-implementer v17 P-SEAL discipline) | ✅ | feature-inventory.md + NEW ADR + NEW drift all CH-28d-stamped; forward-scope already current from prerequisite commit; plan.md immutable; cycle-index orchestrator-owned |

**All artifacts present. Zero paperwork gaps surfaced.**

---

## §6 — Deviations

| ID | Deviation | Class | Rationale |
|---|---|---|---|
| **D-1** | ADR-0066 drafted with **Status: Accepted directly at P3**, skipping the P-SEAL Proposed→Accepted flip described in plan §7 P3 deliverable 3 + P-SEAL deliverable 2 | Plan §7 wording lag vs CH-28c precedent (CH-28c §D-1 deviation pattern inherited) | Plan §7 inherited the older CH-28b 5-phase shape where Status flipped at P-SEAL; CH-28c collapsed to 4 phases + ADR-0065 went Accepted-at-P3 (cycle-index row + runtime prompt + ADR-0066 verified-header all reflect Accepted-at-P3). No semantic divergence; the close criterion (Accepted at chunk close) is met. Auditor verified at Claim 8 PASS. |
| **D-2** | Inline ADR-0034 path-fix at P3 close: plan §9 reading list + runtime prompt cited ADR-0034 at `m6/decisions/0034-domain-agent-and-phi-core-agent.md`; actual canonical path is `m5_2/decisions/0034-agent-durable-lifecycle.md` (parent-dir + filename both differ) | Plan-narrative path-citation miss (same class as CH-28c D-2 ADR-0059 path-fix at P3) | Implementer applied 2-line patch (1 path in ADR-0066 §"Cross-references" + 1 path in NEW drift §"Cross-references") + re-ran `check-doc-links.sh` GREEN. Auditor verified at Claim 15 PASS. **Standards-update candidate**: pre-archival ADR cross-ref filesystem-grep automation (carried forward from CH-28c retro tracker's D-2 candidate; 2nd consecutive cycle surfacing this class). Tracked for chunk-retrospector consideration at Phase 5→6 transition gate. |

**Class summary**: 2 process-side deviations (both Code-class corrective edits ratified inline); 0 user-DIVERGENT fork-locks (all 4 F-locks F1.b + F2.a + F3.a + F4.a planner-rec rubber-stamps); 0 scope-narrowings; 0 LOC-cap pause triggers; 0 phase-ordering deviations; 0 cascade-cardinality-stale-narrative gaps; 0 audit-prompt-authoring miss; 0 SCHEMAFULL semantic deviations (N/A — no migration this chunk).

**Cross-cycle divergent-fork count**: stays at **14-of-19 (~74%)** — CH-28d 0-of-4 divergent. 3rd consecutive cycle holding the count steady (CH-28b 0/3 + CH-28c 0/3 + CH-28d 0/4 = 0/10 across 3 phi-core absorption cycles).

---

## §7 — Metrics

| Metric | Plan §10 / Predicted | Actual at chunk seal | Variance |
|---|---|---|---|
| Production code Δ | +13 LOC | +17 LOC at `launch.rs` (diff includes context lines) | +4 over prediction; within slack |
| Cargo.lock churn | ≤ 50 lines | 4 lines (1 dep updated) | Well under cap; smallest churn this cycle family |
| Governance docs Δ NEW | ~270 LOC (ADR ~150 + drift ~120) | NEW ADR 275 + NEW drift 94 = 369 LOC | +99 over prediction; ADR denser (carrier-fix complexity) + drift sparser than predicted (4 axes A/B/C/D scoped compactly); within governance-tier slack |
| feature-inventory row Δ | ~10 LOC | +8 LOC | Within prediction |
| Tests | 1599 → 1599 (Δ 0) | 1599 → 1599 (Δ 0) | ✅ exact match |
| phi-core raw `use phi_core` line count | 57 → 57 (Δ 0) | 57 → 57 (Δ 0) | ✅ exact match — FQN-only field-adds preserve baseline per F3.a lock |
| phi-core semantic leverage-sites | +1 via FQN | +1 confirmed at `launch.rs:592` | ✅ exact match |
| Plan iterations | 1 | 1 | ✅ exact match (planner-rec-clean cycle; P-orch-8 skip-condition fires; no iter-2 re-spawn needed) |
| Audit iterations | 1 | 1 | ✅ exact match |
| Audit envelope | SMALL (1 auditor Letter A) | SMALL (1 auditor Letter A) | ✅ exact match |
| Cumulative cross-cycle divergent forks | 14-of-19 (no change predicted) | 14-of-19 unchanged (3rd consecutive cycle steady) | ✅ exact match |
| **Cargo-clean placement-1 cumulative** | n/a (predicted; implementer-side ~190 GiB) | **278.6 GiB across 3 invocations** (P2 95.0 + P-SEAL 89.3 + gate-4 94.3) | Within range |
| Cargo-clean placement-2 (gate-5 final close) | TBD | TBD (Phase 5 next) | — |

**Disk state at gate-4 close**: `df -h /root` shows 270 GiB free / 28% used (post-placement-1 reclaim).

**3rd consecutive activation milestones**:
- CH-NNa/b/c/d interstitial chunk-numbering pattern (CH-28b → CH-28c → CH-28d)
- F2.a semver-range form `phi-core = "0.X"` (0.8 → 0.9 → 0.10)
- F3.a carrier-fix-comment-block pattern at `launch.rs:526` site (CH-25 response_format → CH-28b revert_pending → CH-28d revert_render_policy + current_tool)
- F4.a single-drift-per-absorption-chunk filing pattern (D-PHICORE-08-FOLLOWUP-01 → D-PHICORE-09-FOLLOWUP-01 → D-PHICORE-10-FOLLOWUP-01)
- SMALL audit envelope sizing for phi-core absorption chunks

---

## §8 — Conclusion + next steps

CH-28d **passes at gate-4 with zero remediation required**. baby-phi compiles + tests + lints + formats cleanly on phi-core 0.10.x. ALL downstream M6 chunks (CH-29..CH-38) are unblocked on the new baseline; per-tool-call introspection + Composition I decay-window adoption + hook-script-shim adoption + `CurrentToolExecution` standalone consumption all deferred via the new D-PHICORE-10-FOLLOWUP-01 drift's 4 axes.

**Process observation worth surfacing at Phase 5→6 transition gate**: D-2 (inline ADR-0034 path-fix during P3 `check-doc-links` verification) is the **2nd consecutive cycle** to surface a stale ADR path citation in the chunk-planner output (CH-28c D-2 was the 1st with ADR-0059 path-fix). Both were caught at P3 by `check-doc-links.sh` + repaired inline by the implementer. Candidate standards-update axis: **pre-archival ADR cross-ref filesystem-grep automation** at chunk-planner self-check time (would catch the stale path before plan archives + implementer encounters it). Track for chunk-retrospector consideration if the user proceeds with Phase 6.

**Awaiting**:
1. Phase 5 cargo-clean placement-2 (gate-5 final close; target/ already empty post-gate-4 placement-1, expected 0 bytes reclaimed)
2. Phase 5→6 transition gate (mandatory pause per `feedback_pause_before_retrospector.md`)
3. Phase 7 cycle-index Status flip (`in-flight` → `audited-pending-retro` if user skips Phase 6, OR `retro-complete` if Phase 6 runs)
4. Phase 8 temp-folder cleanup (`/tmp/ch28d-cargo-test.log` + auditor task transcripts)
5. User commit (baby-phi cycle work + parent submodule bump)
