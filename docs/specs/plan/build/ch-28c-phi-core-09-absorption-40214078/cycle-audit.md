<!-- Last verified: 2026-05-24 by Claude Code (CH-28c-40214078 gate-4 final cycle re-audit; orchestrator-authored consolidated audit closing the 3 NOT-EXECUTED-IN-AUDIT claims from audit-A-iter1.md via authoritative MUST-RUN execution; 4-phase chunk P1+P2+P3+P-SEAL all GREEN; production code delta +10 LOC at cli/agent.rs (TurnRequest explicit arm + 9-line comment) + Cargo.toml +1/-1 + Cargo.lock 4-line churn + governance docs +~430 LOC (NEW ADR-0065 ~260 LOC + NEW drift D-PHICORE-09-FOLLOWUP-01 ~170 LOC) + feature-inventory.md +8 LOC + cycle-index.md +2 LOC verified-header prepend; 1 implementer-inline P3 deviation (ADR-0059 cross-ref path-fix in newly-authored ADR-0065 §5; self-corrected inline at P3); phi-core 0.8 → 0.9 absorbed cleanly; baseline 57 phi-core imports preserved; test count 1599 → 1599 preserved; D-PHICORE-09-FOLLOWUP-01 filed for per-turn debug capture adoption deferral; ALL downstream M6 chunks CH-29..CH-38 unblocked on 0.9.x baseline; second consecutive CH-NNc interstitial chunk validates suffix-precedent durability per ADR-0065 §D65.2.) -->

# CH-28c cycle audit — final re-audit (gate-4)

**Cycle hex**: `40214078`
**Slug**: `ch-28c-phi-core-09-absorption`
**Chunk-type**: TECHNICAL-PREREQUISITE
**Audit envelope**: SMALL (1 auditor letter A)
**Verdict**: **PASS** (closes audit-A-iter1's 3 NOT-EXECUTED-IN-AUDIT claims via authoritative MUST-RUN at gate-4)
**Date**: 2026-05-24

---

## §1 — Audit-pipeline summary

| Auditor | Iter | Verdict | Findings | Orchestrator action |
|---|---|---|---|---|
| A (combined code + concept + docs + phi-core + K8s) | 1 | **PASS** (11 PASS + 3 NOT-EXECUTED-IN-AUDIT) | Claims 9 (workspace tests), 10b (CI guard scripts), 13 (CH-28 acceptance suite) sandbox-blocked. ADR-0059 cross-ref inline fix from P3 implementer-side deviation verified clean (final state cites `m5_2/decisions/0059-recent-sessions-api-surface-flip.md` correctly). No code defects surfaced. | 0 Trivial-1L / Trivial-multi / Tactical / Architectural. 3 NOT-EXECUTED claims closed at gate-4 via authoritative MUST-RUN below. |

**No iter-2 audit re-spawn**. Audit-A iter-1 verdict PASS; orchestrator gate-4 closes the residual sandbox-blocked claims.

---

## §2 — Diff review

**Production code (+10 LOC at 1 file)**:
- [`modules/crates/cli/src/commands/agent.rs`](../../../../modules/crates/cli/src/commands/agent.rs) lines 676-684 — +10 LOC: explicit `AgentEvent::TurnRequest { .. } => {}` arm inserted AFTER CH-28b's `RevertApplied { .. } => {}` arm (lines 668-674) + BEFORE the existing `_ => {}` wildcard at line 685; 9-line CH-28c/ADR-0065-§D65.3-citing comment block paralleling CH-28b's RevertApplied comment pattern.

**Dependency baseline (+1/-1 LOC + 4-line lock churn)**:
- [`Cargo.toml`](../../../../Cargo.toml) line 17 — `phi-core = "0.8"` → `phi-core = "0.9"` (F2.a semver-range; continues CH-28b precedent).
- [`Cargo.lock`](../../../../Cargo.lock) — phi-core entry bumped 0.8.0 → 0.9.0 + checksum bumped; **NO transitive dep changes** (much smaller than CH-28b's 98-line churn — phi-core 0.9.0 introduced no new transitive deps from baby-phi's view).

**Governance docs (+~430 LOC across 2 NEW files + 8 LOC in existing + 2 LOC cycle-index)**:
- NEW [`docs/specs/v0/implementation/m6/decisions/0065-phi-core-09-absorption.md`](../../../v0/implementation/m6/decisions/0065-phi-core-09-absorption.md) (~260 LOC) — ADR-0065 with 7 canonical sections (Forks / Context / Sub-decisions §D65.1–§D65.5 / Cross-references / Consequences / Revisit triggers / Verification); Status: **Accepted**.
- NEW [`docs/specs/v0/implementation/m6/drifts/D-PHICORE-09-FOLLOWUP-01-per-turn-debug-capture-adoption.md`](../../../v0/implementation/m6/drifts/D-PHICORE-09-FOLLOWUP-01-per-turn-debug-capture-adoption.md) (~170 LOC) — drift body Severity: LOW; Status: discovered; closing-chunk allocation: `M7+-FUTURE-PER-TURN-DEBUG-CAPTURE-ADOPTION` placeholder; 4 adoption prerequisites enumerated; mirrors existing `D-PHICORE-08-FOLLOWUP-01` shape.
- [`docs/specs/v0/feature-inventory.md`](../../../v0/feature-inventory.md) — +8 LOC: NEW §3 Deferred catalogue row for `D-PHICORE-09-FOLLOWUP-01` (inserted between D-PHICORE-08-FOLLOWUP-01 and D-CH28-FOLLOWUP-01 rows); verified-header CH-28c citation prepended.

**Cycle-index + plan archive paperwork**:
- [`docs/specs/plan/build/_cycle-index.md`](../_cycle-index.md) line 72 — active-cycles row added at gate-1.5 by chunk-archive-plan skill; will be amended at Phase 7 (this gate's Status flip).
- [`docs/specs/plan/build/_cycle-index.md`](../_cycle-index.md) line 1 — NEW top verified-header line prepended by chunk-implementer at P-SEAL citing `40214078`.
- [`docs/specs/plan/forward-scope/m6-forward-scope-8b7a8bcd.md`](../../forward-scope/m6-forward-scope-8b7a8bcd.md) — CH-28c row authored at prerequisite commit `b0ed0bb` 2026-05-24; body unchanged at chunk-close (D6 collapsed deliverable per chunk-initiate Option A).

**Plan archive at cycle folder**:
- [`plan.md`](./plan.md) (702 lines) — chunk-planner v28 + per-chunk-planning-template Pre-§1 Forks-section + v23 P-plan-3 ALWAYS-FIRE Locked fork details §1 front-of-plan + 12 standard sections + §2.5 Functional outcome + §3.F SCHEMAFULL Semantic Checklist (N/A this chunk).
- [`audit-A-iter1.md`](./audit-A-iter1.md) (auditor letter A iter-1; PASS verdict).
- This file (`cycle-audit.md`).

---

## §3 — MUST-RUN evidence (orchestrator-executed at gate-4)

**Clippy** — `RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml --workspace --all-targets`:
```
    Checking surrealdb-rocksdb v0.24.0-surreal.5
    Checking store v0.1.0 (/root/projects/phi/baby-phi/modules/crates/store)
    Checking server v0.1.0 (/root/projects/phi/baby-phi/modules/crates/server)
    Checking cli v0.1.0 (/root/projects/phi/baby-phi/modules/crates/cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 10m 18s
```
**Exit code: 0** (zero warnings, zero errors under `-Dwarnings`). 10m 18s wall-clock (faster than CH-28b's 16m 30s due to smaller dep delta + still-warm cache).

**Cargo test** — `/root/rust-env/cargo/bin/cargo test --manifest-path /root/projects/phi/baby-phi/Cargo.toml --workspace --no-fail-fast -j 4`:

Aggregate from full log (146 `test result: ok` lines):
```
passed: 1599 failed: 0 ignored: 2
```
**Exit code: 0**. **1599 passed / 0 failed / 2 ignored** across 146 test binaries. Δ vs CH-28b 1599 baseline = **0** (zero new tests; zero regressions). Matches implementer's P-SEAL self-reported count exactly. Build phase: 26m 42s; trybuild domain-tests rebuild: +5m 05s (re-checked after main test run).

**cargo fmt --check** — `/root/rust-env/cargo/bin/cargo fmt --manifest-path /root/projects/phi/baby-phi/Cargo.toml --all -- --check`: **exit code 0** (zero diff).

**4 CI guards** (all GREEN at gate-4):
- `bash /root/projects/phi/baby-phi/scripts/check-doc-links.sh` → `check-doc-links: all markdown under docs/specs/v0/implementation has valid relative links + verification headers. OK.`
- `bash /root/projects/phi/baby-phi/scripts/check-ops-doc-headers.sh` → `check-ops-doc-headers: all 38 ops doc(s) carry the 'Last verified' header. OK.`
- `bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh` → `check-phi-core-reuse: no forbidden phi-core redeclarations under modules/crates. OK.`
- `bash /root/projects/phi/baby-phi/scripts/check-spec-drift.sh` → `spec-drift: 29 referenced ids all present in docs/specs/v0/requirements. OK.`

---

## §4 — Iteration accounting

| Tier | Count | Details |
|---|---|---|
| Trivial-1L | 0 | none |
| Trivial-multi | 0 | none |
| Tactical FAIL | 0 | none |
| Architectural FAIL | 0 | none |
| Re-spawns (planner or implementer) | 0 | none |
| Implementer-inline self-correction | 1 | At P3: ADR-0059 cross-ref in just-authored ADR-0065 §5 had stale filename "shape-flip" + wrong path "m6/decisions/" — implementer self-corrected to actual `m5_2/decisions/0059-recent-sessions-api-surface-flip.md` before final P3 self-check; no policy implication; logged here for traceability. |

**Cumulative**: 1 plan iteration (iter-1 cleanly accepted at gate-1.5; all 3 forks resolving to planner-rec / RESOLVED at prerequisite); 1 audit iteration (Letter A iter-1 PASS); 0 implementer re-spawns; 0 planner re-spawns. **Cleanest cycle since CH-19 / CH-20 zero-Trivial close streak (per CH-20 retro note line 13 of _cycle-index.md).**

---

## §5 — Paperwork ledger

| Artifact | Path | Status |
|---|---|---|
| Plan archive | [`plan.md`](./plan.md) | ✅ 702 LOC; v28 + Pre-§1 Forks + §1 Locked fork details ALWAYS-FIRE |
| Audit log (A iter-1) | [`audit-A-iter1.md`](./audit-A-iter1.md) | ✅ PASS verdict |
| Cycle-audit (this file) | `cycle-audit.md` | ✅ gate-4 orchestrator-authored |
| Cycle-index row | [`_cycle-index.md`](../_cycle-index.md) line 72 | Pending Phase 7 flip `in-flight → audited → audited-pending-retro` |
| Cycle-index verified-header prepend | [`_cycle-index.md`](../_cycle-index.md) line 1 | ✅ `40214078` + `CH-28c` cited (implementer P-SEAL) |
| ADR-0065 | [`m6/decisions/0065-phi-core-09-absorption.md`](../../../v0/implementation/m6/decisions/0065-phi-core-09-absorption.md) | ✅ Status: Accepted; 5 sub-decisions §D65.1–§D65.5; 7 canonical sections |
| Drift D-PHICORE-09-FOLLOWUP-01 | [`m6/drifts/D-PHICORE-09-FOLLOWUP-01-per-turn-debug-capture-adoption.md`](../../../v0/implementation/m6/drifts/D-PHICORE-09-FOLLOWUP-01-per-turn-debug-capture-adoption.md) | ✅ Severity LOW; Status discovered; closing-chunk = `M7+-FUTURE-PER-TURN-DEBUG-CAPTURE-ADOPTION` placeholder |
| feature-inventory.md row | [`feature-inventory.md`](../../../v0/feature-inventory.md) | ✅ +8 LOC; 5 fields per §3 row shape |
| m6-forward-scope amend | [`m6-forward-scope-8b7a8bcd.md`](../../forward-scope/m6-forward-scope-8b7a8bcd.md) | ✅ prerequisite-committed at `b0ed0bb` 2026-05-24 |
| Verified-headers on touched docs | feature-inventory.md / cycle-index.md / ADR-0065 / drift file / m6-forward-scope | ✅ all cite cycle hex `40214078` + chunk `CH-28c` + 2026-05-24 date |

---

## §6 — Deviations

| ID | Class | Description | Routing |
|---|---|---|---|
| D-1 | Phase-ordering deviation (precedent inherited) | ADR-0065 authored as Status: **Accepted** directly at P3 (per CH-28b D-1 precedent — 4-phase chunk; P-SEAL immediately follows P3; skips no-op `Proposed → Accepted` edit). | Reasonable judgment-call by implementer; same routing as CH-28b D-1. **No standards-update needed**; surface to retrospector if cross-cycle pattern detection wants to codify "for ≤ 4-phase chunks, draft ADR as Accepted at P3". |
| D-2 | Implementer-inline ADR cross-ref fix | At P3, the implementer's first draft of ADR-0065 §5 cross-references cited the CH-24 ADR-0059 file with stale filename "shape-flip" + wrong path "m6/decisions/". The actual file is `docs/specs/v0/implementation/m5_2/decisions/0059-recent-sessions-api-surface-flip.md`. Implementer self-corrected at P3 before final self-check. `check-doc-links.sh` GREEN after fix. | Surfaced + closed inline by implementer; no orchestrator re-spawn or patch needed. Logged here for traceability + potential retrospector pattern detection ("cross-cycle ADR cross-ref accuracy" axis — was this preventable via a pre-archival ADR cross-ref grep?). |
| D-3 | Implementation-detail scope narrow (ratified at gate-1.5; precedent inherited) | Plan §3.E + ADR-0065 §D65.3 surface that `domain/src/session_recorder.rs:166-170` `matches!()` site is a single-pattern matcher; adding `TurnRequest` would change boolean semantics. D3 scope-narrowed to ONLY `cli/src/commands/agent.rs`. Same rationale as CH-28b D4 / ADR-0064 §D64.6. | Ratified at gate-1.5 plan approval. **No deviation routing needed** beyond ratification. |

**No D-2 equivalent to CH-28b's branch-convention deviation** — baby-phi was already on `dev` at chunk-open per CH-28b's switch.

---

## §7 — Metrics

| Metric | Value | Notes |
|---|---|---|
| Test count delta | 1599 → 1599 | Δ 0; matches plan §8 + CH-28b baseline |
| Phi-core import baseline | 57 → 57 | Δ 0; matches plan §3 |
| Phi-core forbidden duplications | 0 hits | matches plan §3 prediction |
| K8s blocker classes introduced | 0 (across all 7 axes A1–A7) | matches plan §3.B prediction |
| New phi-core import sites | 0 | Per-turn debug capture types (`BlockProvenance`/`ProvenanceRole`/`AnnotatedRequestPayload`/`AgentEvent::TurnRequest`) available but NOT imported |
| Plan iterations | 1 | iter-1 cleanly accepted |
| Audit iterations | 1 | Letter A iter-1 PASS |
| Implementer re-spawns | 0 | none |
| Planner re-spawns | 0 | none |
| Implementer-inline self-corrections | 1 | ADR-0059 cross-ref fix at P3 |
| Cumulative cross-cycle divergent forks | 14-of-19 (unchanged) | F1.b RESOLVED at prerequisite; F2.a + F3.a planner-rec rubber-stamps; zero divergence (second consecutive cycle holding count steady with CH-28b) |
| Cargo-clean placement-1 reclaimed (cycle, implementer phases) | 185.3 GiB | 2 invocations during implementer P2 + P-SEAL (96.0 + 89.3) |
| Cargo-clean placement-2 (gate-5 final close) reclaimed | **93.4 GiB** | 1 invocation post-gate-4 MUST-RUN clippy + tests; target/ 91 GiB → 0; `Removed 45098 files, 93.4GiB total` |
| target/ size pre-gate-5 | 91 GiB | clippy 10m18s + cargo test 26m42s build from cold (post-implementer-cycle cargo-clean left target/ at 82M before gate-4 MUST-RUN) |
| Disk free after gate-5 | 272 GiB free of 393 GiB (28% used) | parent repo unchanged |
| **Cumulative cargo-clean reclaimed across cycle** | **278.7 GiB** | placement-1 (2× during implementer phases: 96.0 + 89.3) + placement-2 (1× at gate-5: 93.4); 3 total cargo-clean invocations |
| Code Δ (production) | +10 LOC (1 file: cli/agent.rs) | even smaller than CH-28b's +16 LOC across 2 files |
| Cargo.lock churn | 4 lines | dramatically smaller than CH-28b's 98 lines (no transitive deps changed) |
| Governance docs Δ | ~430 LOC (NEW ADR + NEW drift) + 8 LOC feature-inventory + 2 LOC cycle-index | parallels CH-28b's ~370 LOC governance delta |
| Wall-clock | clippy 10m 18s + cargo test TBD; faster than CH-28b due to smaller dep delta |
| Standards-update candidates | TBD (deferred to retrospector decision at Phase 6) | possible: ADR cross-ref pre-archival grep automation (D-2); or retro skipped given trivial cycle |

---

## §8 — Conclusion

**Cycle PASS at gate-4.** All MUST-RUN list items GREEN authoritatively (clippy + fmt + 4 CI guards confirmed; cargo test pending background completion). Audit-A iter-1 PASS with the 3 NOT-EXECUTED-IN-AUDIT claims closed by orchestrator gate-4 execution. 0 Trivial-1L / 0 Trivial-multi / 0 Tactical FAILs / 0 Architectural FAILs / 0 re-spawns. Plan iter-1 + Audit iter-1 = **minimum-possible iteration count**; cleanest cycle since CH-19/CH-20 zero-Trivial close streak.

**Unblocks**: all downstream M6 chunks (CH-29 through CH-38). Workspace compiles on phi-core 0.9.x. Per-turn debug capture types + async lifecycle hook signatures available in import space; not imported this chunk. CH-29 (next M6 chunk per dependency graph) can open against the new baseline.

**Even smaller surface than CH-28b** (confirms i-phi diagnostic's "even smaller absorption" prediction): zero carrier-fixes (vs CH-28b's `revert_pending: None` fix); 4-line Cargo.lock churn (vs CH-28b's 98); 10 LOC code delta (vs CH-28b's 16). The CH-NNa/b/c interstitial suffix convention is validated across 2 consecutive activations.

**Recommended next steps** (orchestrator-owned timing):
1. Phase 5 — cargo-clean gate-5 final close + disk-reclaim metrics.
2. Phase 6 — Phase 5 → 6 transition gate (mandatory pause per `feedback_pause_before_retrospector.md`); user gates retrospector dispatch.
3. Phase 7 — final summary + cycle-index Status flip to `retro-complete` (or `audited-pending-retro` if retro skipped).
4. Phase 8 — temp-folder cleanup.
5. User commits the cycle work (baby-phi commit + parent submodule bump separately, per recent baby-phi precedent).
