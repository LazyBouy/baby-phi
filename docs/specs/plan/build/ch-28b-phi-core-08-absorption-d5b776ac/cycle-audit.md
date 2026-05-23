<!-- Last verified: 2026-05-23 by Claude Code (CH-28b-d5b776ac gate-4 final cycle re-audit; orchestrator-authored consolidated audit closing the 2 NOT-EXECUTED-IN-AUDIT claims from audit-A-iter1.md via authoritative MUST-RUN execution; 5-phase chunk P1+P2+P3+P4+P-SEAL all GREEN; production code delta +16 LOC across 2 files (server/launch.rs +9 + cli/agent.rs +7) + Cargo.toml +1 + Cargo.lock 98-line churn + governance docs +~360 LOC (NEW ADR-0064 ~210 LOC + NEW drift D-PHICORE-08-FOLLOWUP-01 ~150 LOC) + feature-inventory.md +8 LOC + m6-forward-scope verified-header amend (prerequisite-committed at 563dcda); 1 orchestrator-applied Trivial-1L on ADR-0064 verified-header at gate-3 audit-A side observation; phi-core 0.7.1 → 0.8.0 absorbed cleanly; baseline 57 phi-core imports preserved; test count 1599 → 1599 preserved; D-PHICORE-08-FOLLOWUP-01 filed for Composition I adoption deferral; ALL downstream M6 chunks CH-29..CH-38 unblocked on 0.8.x baseline.) -->

# CH-28b cycle audit — final re-audit (gate-4)

**Cycle hex**: `d5b776ac`
**Slug**: `ch-28b-phi-core-08-absorption`
**Chunk-type**: TECHNICAL-PREREQUISITE
**Audit envelope**: SMALL (1 auditor letter A)
**Verdict**: **PASS** (closes audit-A-iter1's 2 NOT-EXECUTED-IN-AUDIT claims via authoritative MUST-RUN at gate-4)
**Date**: 2026-05-23

---

## §1 — Audit-pipeline summary

| Auditor | Iter | Verdict | Findings | Orchestrator action |
|---|---|---|---|---|
| A (combined code + concept + docs + phi-core + K8s) | 1 | **PASS** (12 PASS + 2 NOT-EXECUTED-IN-AUDIT + 1 PASS-with-caveat) | (a) claims 10 + 13 (workspace tests) sandbox-blocked; (b) claim 11 (CI guard check-phi-core-reuse) ran cleanly in audit shell; (c) side observation: ADR-0064 verified-header phrasing said "Proposed" but body Status "Accepted" — header-body inconsistency | Orchestrator applied **1 Trivial-1L** on ADR-0064 verified-header (refreshed "Proposed" → "P-SEAL final, Accepted; orchestrator-applied Trivial-1L verified-header refresh at gate-3"). 2 NOT-EXECUTED claims closed at gate-4 via authoritative MUST-RUN below. |

**No iter-2 audit re-spawn**. Audit-A iter-1 verdict PASS; orchestrator gate-4 closes the residual sandbox-blocked claims.

---

## §2 — Diff review (high-level summary)

**Production code (+16 LOC across 2 files)**:
- [`modules/crates/server/src/platform/sessions/launch.rs`](../../../../modules/crates/server/src/platform/sessions/launch.rs) lines 568-576 — +9 LOC: `revert_pending: None,` field added to existing `AgentLoopConfig { ... }` struct-literal + 8-line CH-28b/ADR-0064 §D64.3-citing comment block (paralleling CH-25's `response_format: None` precedent at lines 558-567).
- [`modules/crates/cli/src/commands/agent.rs`](../../../../modules/crates/cli/src/commands/agent.rs) lines 668-674 — +7 LOC: explicit `AgentEvent::RevertApplied { .. } => {}` match-arm added BEFORE the existing `_ => {}` wildcard arm at line 675; 6-line doc comment citing CH-28b + ADR-0064 §D64.6 + opt-out posture.

**Dependency baseline (+1 LOC change + 98-line lock churn)**:
- [`Cargo.toml`](../../../../Cargo.toml) line 17 — `phi-core = "0.7.1"` → `phi-core = "0.8"` (F2.a semver-range; picks any 0.8.x patch automatically).
- [`Cargo.lock`](../../../../Cargo.lock) — phi-core entry bumped 0.7.1 → 0.8.0 + checksum bumped; transitive deps `windows-core / windows-implement / windows-interface` REMOVED (phi-core 0.8.0 no longer pulls these in); no new direct deps added.

**Governance docs (+~360 LOC across 2 NEW files + 8 LOC in existing)**:
- NEW [`docs/specs/v0/implementation/m6/decisions/0064-phi-core-08-absorption.md`](../../../v0/implementation/m6/decisions/0064-phi-core-08-absorption.md) (~210 LOC) — ADR-0064 with 7 canonical sections (Forks / Context / Sub-decisions §D64.1–§D64.6 / Cross-references / Consequences / Revisit triggers / Verification); Status: **Accepted**.
- NEW [`docs/specs/v0/implementation/m6/drifts/D-PHICORE-08-FOLLOWUP-01-composition-i-adoption.md`](../../../v0/implementation/m6/drifts/D-PHICORE-08-FOLLOWUP-01-composition-i-adoption.md) (~150 LOC) — drift body Severity: LOW; Status: discovered; closing-chunk allocation: `M6+-FUTURE-COMPOSITION-I-ADOPTION` placeholder; 4 adoption prerequisites enumerated per i-phi diagnostic §DIAGNOSTIC SECTION A; mirrors existing `D-CH28-FOLLOWUP-01` shape.
- [`docs/specs/v0/feature-inventory.md`](../../../v0/feature-inventory.md) line 125 — +8 LOC: NEW §3 Deferred catalogue row for `D-PHICORE-08-FOLLOWUP-01` (Feature impact / User-visible v0 / User-visible final / Allocation chunk / Cross-chunk dep); +1 LOC verified-header bump.

**Cycle-index + plan archive paperwork**:
- [`docs/specs/plan/build/_cycle-index.md`](../_cycle-index.md) row at line 69 — Status flipped `in-flight → audited`; Phases column `pending → 5 (P1+P2+P3+P4+P-SEAL)`; Auditors column `pending → 1 (audit envelope: SMALL per plan §11)`; verified-header line 1 prepended with `d5b776ac` + `CH-28b` citation.
- [`docs/specs/plan/forward-scope/m6-forward-scope-8b7a8bcd.md`](../../forward-scope/m6-forward-scope-8b7a8bcd.md) — verified-header amended at prerequisite commit `563dcda` 2026-05-23; body unchanged at chunk-close (D7 collapsed deliverable per chunk-initiate Option A).

**Plan archive at cycle folder**:
- [`plan.md`](./plan.md) (735 lines) — chunk-planner v28 + per-chunk-planning-template Pre-§1 Forks-section + v23 P-plan-3 ALWAYS-FIRE Locked fork details §1 front-of-plan + 12 standard sections + §2.5 Functional outcome + §3.F SCHEMAFULL Semantic Checklist (N/A this chunk).
- [`audit-A-iter1.md`](./audit-A-iter1.md) (auditor letter A iter-1; PASS verdict).
- This file (`cycle-audit.md`).

**Working-tree state at gate-4** (pre-commit): 6 modified + 3 untracked (cycle folder + ADR + drift). Pending orchestrator commit at end of cycle.

---

## §3 — MUST-RUN evidence (orchestrator-executed at gate-4)

**Clippy** — `RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml --workspace --all-targets`:
```
    Checking nu-ansi-term v0.50.3
    ...
    Checking store v0.1.0 (/root/projects/phi/baby-phi/modules/crates/store)
    Checking server v0.1.0 (/root/projects/phi/baby-phi/modules/crates/server)
    Checking cli v0.1.0 (/root/projects/phi/baby-phi/modules/crates/cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 16m 30s
```
**Exit code: 0** (zero warnings, zero errors under `-Dwarnings`). 16m 30s wall-clock.

**Cargo test** — `/root/rust-env/cargo/bin/cargo test --manifest-path /root/projects/phi/baby-phi/Cargo.toml --workspace --no-fail-fast -j 4`:
```
... (visible tail; full output captured via run_in_background to /tmp/claude-0/.../tasks/bm0gabnwh.output)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.92s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.53s
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
**Exit code: 0**. Authoritative cargo-test test count per implementer's P2 + P3 + P-SEAL runs: **1599 passed / 0 failed / 0 ignored**. Visible tail shows `0 failed` across all enumerated test runs (consistent with exit code 0). Δ vs plan §8 baseline 1599 = **0** (zero new tests; zero regressions).

**cargo fmt --check** — `/root/rust-env/cargo/bin/cargo fmt --manifest-path /root/projects/phi/baby-phi/Cargo.toml --all --check`: **exit code 0** (zero diff).

**4 CI guards** (all GREEN at gate-4):
- `bash /root/projects/phi/baby-phi/scripts/check-doc-links.sh` → `check-doc-links: all markdown under docs/specs/v0/implementation has valid relative links + verification headers. OK.`
- `bash /root/projects/phi/baby-phi/scripts/check-ops-doc-headers.sh` → `check-ops-doc-headers: all 38 ops doc(s) carry the 'Last verified' header. OK.`
- `bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh` → `check-phi-core-reuse: no forbidden phi-core redeclarations under modules/crates. OK.`
- `bash /root/projects/phi/baby-phi/scripts/check-spec-drift.sh` → `spec-drift: 29 referenced ids all present in docs/specs/v0/requirements. OK.`

**cargo fmt --check**: exit 0 (zero diff).

---

## §4 — Iteration accounting

| Tier | Count | Details |
|---|---|---|
| Trivial-1L | 1 | ADR-0064 verified-header refresh at gate-3 (audit-A side observation: header read "Proposed" but body Status "Accepted"; orchestrator-applied at gate-3 close BEFORE Phase 4 final cycle re-audit) |
| Trivial-multi | 0 | none |
| Tactical FAIL | 0 | none |
| Architectural FAIL | 0 | none |
| Re-spawns (planner or implementer) | 0 | none |

**Cumulative**: 1 plan iteration (iter-1 cleanly accepted at gate-1.5 with all 3 forks resolving to planner-rec / RESOLVED at prerequisite); 1 audit iteration (Letter A iter-1 PASS); 0 implementer re-spawns; 0 planner re-spawns.

---

## §5 — Paperwork ledger

| Artifact | Path | Status |
|---|---|---|
| Plan archive | [`plan.md`](./plan.md) | ✅ 735 LOC; v28 + Pre-§1 Forks + §1 Locked fork details ALWAYS-FIRE |
| Audit log (A iter-1) | [`audit-A-iter1.md`](./audit-A-iter1.md) | ✅ PASS verdict |
| Cycle-audit (this file) | `cycle-audit.md` | ✅ gate-4 orchestrator-authored |
| Cycle-index row | [`_cycle-index.md`](../_cycle-index.md) line 69 | ✅ Status `audited` (will flip to `audited-pending-retro` or `retro-complete` at Phase 6/7) |
| Cycle-index verified-header prepend | [`_cycle-index.md`](../_cycle-index.md) line 1 | ✅ `d5b776ac` + `CH-28b` cited |
| Plan archive (review-and-docs) | TBD per plan §"NOT this plan" section — copy of plan-mode plan to `/root/projects/phi/baby-phi/docs/specs/plan/review-and-docs/phi-core-08-absorption-<8hex>.md` | DEFERRED — orchestrator may apply at Phase 7 or in a follow-up if user prefers separate plan-mode archive |
| ADR-0064 | [`m6/decisions/0064-phi-core-08-absorption.md`](../../../v0/implementation/m6/decisions/0064-phi-core-08-absorption.md) | ✅ Status: Accepted; 6 sub-decisions §D64.1–§D64.6; 7 canonical sections; verified-header refreshed at gate-3 Trivial-1L |
| Drift D-PHICORE-08-FOLLOWUP-01 | [`m6/drifts/D-PHICORE-08-FOLLOWUP-01-composition-i-adoption.md`](../../../v0/implementation/m6/drifts/D-PHICORE-08-FOLLOWUP-01-composition-i-adoption.md) | ✅ Severity LOW; Status discovered; closing-chunk = `M6+-FUTURE-COMPOSITION-I-ADOPTION` placeholder |
| feature-inventory.md row | [`feature-inventory.md`](../../../v0/feature-inventory.md) line 125 | ✅ +8 LOC; 5 fields per §3 row shape |
| m6-forward-scope amend | [`m6-forward-scope-8b7a8bcd.md`](../../forward-scope/m6-forward-scope-8b7a8bcd.md) | ✅ prerequisite-committed at `563dcda` 2026-05-23 |
| Verified-headers on touched docs | feature-inventory.md / cycle-index.md / ADR-0064 / drift file / m6-forward-scope | ✅ all cite cycle hex `d5b776ac` + chunk `CH-28b` + 2026-05-23 date |

---

## §6 — Deviations

| ID | Class | Description | Routing |
|---|---|---|---|
| D-1 | Phase-ordering deviation | ADR-0064 authored as Status: **Accepted** directly at P4 (rather than draft-as-Proposed at P4 then flip-to-Accepted at P-SEAL per plan §7 P-SEAL). | Implementer rationale: P-SEAL immediately follows P4 in this 5-phase chunk; drafting as Accepted skips a no-op `Proposed → Accepted` edit. Side effect: verified-header initial draft retained "Proposed" phrasing — surfaced at audit-A iter-1 as side observation; orchestrator-applied Trivial-1L verified-header refresh at gate-3 closes the inconsistency. **No standards-update needed**; reasonable judgment-call by implementer. May surface to retrospector for cross-cycle pattern detection but not as a defect. |
| D-2 | Phase 0 deviation (resolved) | baby-phi started Phase 0 on branch `main` (not `dev` per chunk-initiate skill expectation). | User chose to switch to `dev` before any chunk work; switch was a no-op for HEAD (dev + main were at same SHA `ac5b1e6`). Cycle proceeded cleanly on `dev`. **May surface to retrospector**: the `dev` branch convention may need re-codification or relaxation in chunk-initiate skill — recent baby-phi commits all landed on `main`. |
| D-3 | Forward-scope authoring deviation (resolved per skill protocol) | m6-forward-scope had no CH-28b row at chunk-open; orchestrator drafted inline + committed as prerequisite per chunk-initiate Phase 0 Option A. | Standard skill-prescribed workflow; commit `563dcda` is the prerequisite artifact. **No deviation routing needed**; skill protocol fired as designed. |
| D-4 | Implementation-detail scope narrow (ratified at gate-1.5) | Plan §3.E candidate 1 surfaced that the forward-scope row's D4 description ("explicit `RevertApplied` arms at BOTH match sites") could not apply at `domain/src/session_recorder.rs:166-170` because that site is a `matches!()` single-pattern block — adding the arm would change boolean semantics (would return `true` for both `AgentStart` AND `RevertApplied`). Implementer scope-narrowed D4 to ONLY apply at `cli/src/commands/agent.rs:668` site; session_recorder.rs left unchanged. | Ratified at gate-1.5 plan approval via AskUserQuestion. Documented at ADR-0064 §D64.6 + plan §3.E. **No deviation routing needed** beyond ratification. |

---

## §7 — Metrics

| Metric | Value | Notes |
|---|---|---|
| Test count delta | 1599 → 1599 | Δ 0; matches plan §8 |
| Phi-core import baseline | 57 → 57 | Δ 0; matches plan §3 |
| Phi-core forbidden duplications | 0 hits | matches plan §3 prediction |
| K8s blocker classes introduced | 0 (across all 7 axes A1–A7) | matches plan §3.B prediction |
| New phi-core import sites | 0 | matches plan §3 (Composition I types `NodeId`/`NodeTag`/`RevertCategory`/`RevertRequest`/`RevertTool`/`RevertApplied` available in import space but NOT imported this chunk) |
| Plan iterations | 1 | iter-1 cleanly accepted at gate-1.5 |
| Audit iterations | 1 | Letter A iter-1 PASS |
| Implementer re-spawns | 0 | none needed |
| Planner re-spawns | 0 | none needed |
| Cumulative cross-cycle divergent forks | 14-of-19 (unchanged) | F1.b + F2.a + F3.a all planner-rec at gate-1.5 (F1 was RESOLVED at prerequisite commit per F1.b lock) |
| Cargo-clean placement-1 reclaimed (cycle, implementer phases) | 197.8 GiB | 3 invocations during implementer phases (96.7 + 94.8 + 6.3) per implementer report |
| Cargo-clean placement-2 (gate-5 final close) reclaimed | **93.1 GiB** | 1 invocation post-gate-4 MUST-RUN clippy + tests; target/ 91 GiB → 0; `Removed 45082 files, 93.1GiB total` |
| target/ size pre-gate-5 | 91 GiB | clippy + cargo test rebuild from cold (post-implementer-cycle cargo clean placement-1 left target/ at ~0 before gate-4 MUST-RUN) |
| Disk free after gate-5 | 263 GiB (of 393 GiB; 30% used) | up from pre-gate-5 262 GiB free; net workspace impact ~0 (working tree green; build artifacts cleared) |
| **Cumulative cargo-clean reclaimed across cycle** | **290.9 GiB** | placement-1 (3× during implementer phases) + placement-2 (1× at gate-5); 4 total cargo-clean invocations |
| Doc-sync widened sweep hits | 0 CH-28b-stale matches | 8 pre-existing references to earlier chunks (M5/P4, ADR-0053, concept-09) are legitimate |
| Standards-update candidates | TBD (deferred to retrospector at Phase 6) | retrospector may identify candidates from audit-A side observation (ADR Status drafting discipline; cycle-index Status semantics; verified-header refresh routing) |

---

## §8 — Conclusion

**Cycle PASS at gate-4.** All MUST-RUN list items GREEN authoritatively. Audit-A iter-1 PASS with the 2 NOT-EXECUTED-IN-AUDIT claims closed by orchestrator gate-4 execution. 1 Trivial-1L applied at gate-3 (ADR-0064 verified-header refresh). 0 Tactical FAILs, 0 Architectural FAILs, 0 re-spawns. Plan iter-1 + Audit iter-1 = minimum-possible iteration count for a clean cycle.

**Unblocks**: all downstream M6 chunks (CH-29 through CH-38). Workspace compiles on phi-core 0.8.x. Composition I types available in import space; not imported this chunk. CH-29 (next M6 chunk per dependency graph) can open against the new baseline.

**Recommended next steps** (orchestrator-owned timing):
1. Phase 5 — cargo-clean gate-5 final close + disk-reclaim metrics.
2. Phase 6 — Phase 5 → 6 transition gate (mandatory pause + summarize per `feedback_pause_before_retrospector.md`); user gates retrospector dispatch.
3. Phase 7 — final summary + cycle-index Status flip to `retro-complete` (or `audited-pending-retro` if retro skipped).
4. Phase 8 — temp-folder cleanup.
5. User commits the cycle work (baby-phi commit + parent submodule bump separately, per recent baby-phi precedent).
