<!-- Last verified: 2026-05-07 by Claude Code (orchestrator final cycle re-audit gate 4 — CH-07 cycle hex `cc912d07` — GREEN; chunk ready for retrospective + user commit) -->

# CH-07 cycle audit — Multi-scope cascade + contractor-model membership bound

**Cycle hex:** `cc912d07`
**Date:** 2026-05-07
**Orchestrator:** Claude Code (Opus 4.7, full conversation context)
**Plan:** [./plan.md](./plan.md)
**Sub-agent audits:**
- [audit-A-iter1.md](./audit-A-iter1.md) — code correctness + phi-core leverage. Verdict: **PASS (GREEN)**.
- [audit-B-iter1.md](./audit-B-iter1.md) — docs fidelity + concept alignment. Verdict: **PASS (GREEN)**.

**Final cycle verdict:** **GREEN**. CH-07 ready for retrospective + user commit.

---

## §1 — Sub-agent audit summary

| Audit | Iteration | Verdict | Claims | Sandbox-blocked claims |
|---|---|---|---|---|
| A — code | 1 | PASS (GREEN) | 7 PASS / 0 FAIL of 7 substantive (claims 1–7); claim 8a PASS (test count); claim 8b NOT-EXECUTED (clippy); claim 8c NOT-EXECUTED (4 CI guards) | clippy + 4 CI guards (per chunk-auditor v4 §"Sandbox-blocked invocations") |
| B — docs | 1 | PASS (GREEN) | 7 PASS / 0 FAIL of 7 substantive; claim 8 NOT-EXECUTED (4 CI guards) | 4 CI guards |

Both auditors recommend **proceed** — no Implementer or Planner re-spawn required.

---

## §2 — Orchestrator MUST-RUN list (per CLAUDE.md gate 4)

The 5 commands sub-agents cannot reliably execute. All 5 ran from orchestrator shell at the cycle gate; each as its own granular Bash invocation per CLAUDE.md "Granular Bash discipline."

| MUST-RUN command | Result | Notes |
|---|---|---|
| `RUSTFLAGS="-Dwarnings" cargo clippy -j 4 --workspace --all-targets` | ✅ PASS | "Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.29s" — fully cached compile after P3 last build; no warnings emitted under `-Dwarnings`. |
| `bash scripts/check-doc-links.sh` | ✅ PASS | "all markdown under docs/specs/v0/implementation has valid relative links + verification headers. OK." |
| `bash scripts/check-ops-doc-headers.sh` | ✅ PASS | "all 30 ops doc(s) carry the 'Last verified' header. OK." |
| `bash scripts/check-phi-core-reuse.sh` | ✅ PASS | "no forbidden phi-core redeclarations under modules/crates. OK." |
| `bash scripts/check-spec-drift.sh` | ✅ PASS | "29 referenced ids all present in docs/specs/v0/requirements. OK." |

**All 5 MUST-RUN items green.**

---

## §3 — Orchestrator workspace test re-run

**Command:** `/root/rust-env/cargo/bin/cargo test --manifest-path /root/projects/phi/baby-phi/Cargo.toml -j 4 --workspace -- --test-threads=1`

**Result:** **passed=1399 / failed=0 / ignored=2**.

**Plan §8 expected band:** [1399, 1403] (1379 baseline + 20 minimum / 24 maximum new tests under asymmetric ×1.0–×1.20 buffer).

**Verdict:** ✅ PASS — bottom of band; band hit. Test progression: P0=1379, P1=1389 (+10 cascade tests), P2=1393 (+4 step_2a tests), P3=1399 (+6 acceptance tests), P4=1399 (paperwork-only).

**Per-cycle test count delta:** **+20** new tests against 1379 chunk-open baseline (matches plan §7 deliverable counts: 10 P1 + 4 P2 + 6 P3 = 20).

---

## §4 — Orchestrator spot-checks

Beyond the sub-agents' spot-checks, the orchestrator verified the following independently.

### 4.1 — Cascade body line-by-line trace (plan §3 grep claim Artifact A + concept 06 lines 30–53)

`step_5_scope_resolution` at engine.rs:350 dispatches to `cascade_single_scope` (engine.rs:370) when both `session_org_tags` + `session_project_tags` are empty (back-compat path) and `cascade_multi_scope` (engine.rs:387) otherwise. The multi-scope body at engine.rs:387–565:
- Agent-tier short-circuit (lines 398–411) — implementer's design choice, anchored to concept 04 line 354 + concept 06 line 30 inline doc-comment.
- Project-tier match-count branch (lines 413–486) — maps to concept 06 lines 38–43.
- Org-tier match-count branch (lines 488–565) — maps to concept 06 lines 45–53.

Intersection fallback at `cascade_intersection_fallback` (engine.rs:576–644) re-clamps using session-tagged org grants as ceilings (Step-2a-style; concept 06 lines 60–62 endorse Step-3-shape denial).

✅ PASS — implementation matches concept 06 pseudocode line-by-line. Audit A claim 1 verified independently.

### 4.2 — Contractor-bound at step_2a_ceiling (plan §3 grep claim Artifact B)

`step_2a_ceiling` at engine.rs:256–297. Lines 267–281 enforce the membership bound:
```rust
let applicable_ceilings: Vec<&Grant> = if session_org_tags.is_empty() {
    ceiling_grants.iter().collect()  // back-compat: empty → all clamp
} else {
    ceiling_grants
        .iter()
        .filter(|g| match g.holder {
            PrincipalRef::Organization(o) => session_org_tags.contains(&o),
            _ => true,  // non-org ceilings unaffected
        })
        .collect()
};
```

This structurally enforces concept 06 line 162: "an agent's home org (`base_organization`) does not reach into sessions belonging to scopes the agent is not a member of."

The defensive `if applicable_ceilings.is_empty() { return candidates; }` short-circuit (lines 282–287) is a Trivial improvement noted by the implementer at P2 — without it, `candidates.into_iter().filter(|cand| ceilings.iter().any(...))` over an empty `ceilings` vec would silently filter every candidate out (subtle bug). The short-circuit is semantically correct per concept 06 line 162 (a contractor's home-org ceiling that doesn't apply leaves the candidate set unclamped at Step 2a; Step 5's intersection fallback is where the actual outsider clamp happens).

✅ PASS — Audit A claim 2 verified independently.

### 4.3 — phi-core leverage (plan §3 grep claim 5)

```bash
grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/domain/src/permissions/ | wc -l
```

Result: **0 hits**. ✅ PASS — matches plan §3 baseline (0 imports added).

### 4.4 — ResolvedGrant single-canonical-definition (plan §3 grep claim 6)

```bash
grep -rnE '^pub struct ResolvedGrant\b' /root/projects/phi/baby-phi/modules/crates/
```

Result: **1 hit at `modules/crates/domain/src/permissions/expansion.rs:56`**.

**Note**: plan §3 cited `expansion.rs:55` (chunk-open value). Audit A flagged this as a "1-line drift" (PASS-with-note, not FAIL). The drift is not a remediation gap — the structural claim "exactly 1 hit" holds — but it is an inaccuracy in the plan text.

**Trivial courtesy correction applied at gate 4** (orchestrator inline patch, NOT a Trivial-1L FAIL fix per CLAUDE.md split):

- Plan §3 line 167: `expansion.rs:55` → `expansion.rs:56`.
- Plan §6 line 686: `expansion.rs:55` → `expansion.rs:56`.
- Plan §12 line 785: `expansion.rs:55` → `expansion.rs:56`.

Per CLAUDE.md Trivial FAIL split, **this is NOT a Trivial-multi FAIL fix** — Audit A's verdict was PASS-with-note (the claim PASSED). The patch is a quality-of-life correction; the audit verdict stands at GREEN without auditor re-spawn.

✅ PASS — Audit A claim 6 verified independently with line-number drift courtesy-corrected.

### 4.5 — CheckContext construction-site cascade (plan §3 Artifact D)

Predicted: 8–14 sites across ~6 files. Actual (per implementer P1 report + diff verification):
- `engine.rs::Fixture::ctx` — 1 (P1).
- `manifest/mod.rs` — 1 definition (P1).
- `tests/common/mod.rs::OwnedCtx::borrow` — 1 (P1).
- `tests/instance_uri_grant_match_props.rs` — 1 (P1).
- `tests/step_4_constraint_value_match_props.rs` — 1 (P1).
- `server/src/handler_support/permission.rs` — 1 (P1; explicit Display arm).
- `server/src/platform/secrets/reveal.rs` — 4 (P1).
- `server/src/platform/sessions/launch.rs` — 1 (P1; production tag-parse wiring).
- `server/src/platform/sessions/preview.rs` — 1 (P1).
- `server/tests/handler_support_test.rs` — 4 (P1).

**Total: 14 construction sites** updated — within plan §3 Artifact D's predicted band. Pause threshold (21 sites = 1.5× upper) NOT triggered.

Defensive default of empty slices preserves M1 callsite-count test invariants for legacy callers; non-empty slices populate from `parse_session_scope_tags(&session.tags)` at the production launch handler.

✅ PASS — Audit A claim 4 verified independently.

### 4.6 — Concept-doc body unchanged (plan §7 P3 deliverable 5; Audit B claim 1)

```bash
git diff HEAD docs/specs/v0/concepts/permissions/04-manifest-and-resolution.md | wc -l
git diff HEAD docs/specs/v0/concepts/permissions/06-multi-scope-consent.md | wc -l
git diff HEAD docs/specs/v0/concepts/permissions/08-worked-example.md | wc -l
```

Each shows a **+1 line** prepend at line 1 (header-only). Body content byte-identical to chunk-open state.

✅ PASS — Audit B claim 1 verified independently.

### 4.7 — Concept-audit-matrix Status letter-for-letter (CH-12 retro Row 1; Audit B claim 4)

Plan §2 target column for all 5 CH-07-flipped rows: `honored`. Matrix HEAD values for all 5 rows: `**honored**` (markdown bold; lowercase; no trailing punctuation).

| Row | Line | Plan §2 target | Matrix value | Letter-for-letter? |
|---|---|---|---|---|
| 5-tier scope cascade | 179 | honored | `**honored**` | ✅ |
| Unified resolve_scope cascade | 203 | honored | `**honored**` | ✅ |
| Contractor model | 209 | honored | `**honored**` | ✅ |
| Shape A/B/C resolution per worked examples | 227 | honored | `**honored**` | ✅ |
| Contractor scenario | 228 | honored | `**honored**` | ✅ |

Per CH-12 retro Row 1, bold-on-bold + same casing + no punctuation drift requirement: **satisfied**.

The 2 §2 logical claims that do NOT have a separate matrix row (`permissions/04 §"Key Invariants" line 310` + `permissions/06 §"Subject-Side Reach Is Bounded"` + `permissions/08 §"Summary: Who Can Read What"`) are documented as roll-up coverage in the matrix's pre-existing line-1 verified-header. This is internally consistent with the implementer's P3 deviation #1 note and does not constitute drift — auditors confirmed.

✅ PASS — Audit B claim 4 verified independently.

### 4.8 — ADR-0051 Forks header + 4 cross-ref categories (CH-13 retro Row 1; Audit B claim 5)

ADR-0051 at `m5_2/decisions/0051-multi-scope-cascade-contractor-model.md`:
- Status: `**Status: Accepted**` (line 6) ✅.
- Forks header (lines 19–25): all 4 user-locks recorded explicitly with date 2026-05-07: F1.A / F2.A / F3.A / F4.A ✅.
- Cross-references covers 4 categories:
  - (a) Concept docs 04 + 06 + 08 ✅.
  - (b) Closed drifts D-new-06 + D-new-20 ✅.
  - (c) Prior ADRs 0033 / 0036 / 0048 / 0050 ✅.
  - (d) Forward-scope row reference ✅.
- 7 sub-decisions D51.1 through D51.7 all have filled bodies (no remaining "Body TBD" stubs) ✅.

✅ PASS — Audit B claim 5 verified independently.

---

## §5 — Iteration accounting

| Phase | Iterations | Notes |
|---|---|---|
| P0 — Cycle archive + ADR scaffold | 1 | Clean; no re-spawn. |
| P1 — Cascade body + IntersectionEmpty | 1 | Clean; 1389 tests; cargo gates green. |
| P2 — step_2a_ceiling membership bound | 1 | Clean; 1393 tests; cargo gates green. |
| P3 — Acceptance tests + docs + matrix | 1 | Clean; 1399 tests; doc CI guards green. |
| P4 — Chunk-seal paperwork | 1 | Clean; 1399 tests; ADR Accepted; D-new-06 + D-new-20 remediated; D-CH07-FOLLOWUP-01 created. |
| Audit A | 1 | GREEN — 9 PASS / 0 FAIL with 2 sandbox-blocked claims. |
| Audit B | 1 | GREEN — 7 PASS / 0 FAIL with 1 sandbox-blocked claim. |
| Orchestrator final cycle re-audit (gate 4) | 1 | GREEN — all 5 MUST-RUN green; 1 trivial-courtesy line-number patch (`expansion.rs:55→56`, 3 instances in plan.md, NOT a FAIL fix). |

**Total iterations: 1 per audit.** No re-spawns of Implementer or Planner. Trivial-1L count: 0 (the line-number patch is a courtesy correction, not a FAIL fix). Trivial-multi count: 0.

---

## §6 — Forks-resolution audit

| Fork | Lock | Implementation faithful to lock? | Evidence |
|---|---|---|---|
| F1 | F1.A (2-tier + tie-breaker, concept-doc verbatim) | ✅ Yes | `cascade_multi_scope` (engine.rs:387) implements project-tier match-count → org-tier match-count → intersection fallback per concept 06 lines 28–53. ADR-0051 §D51.1 pins. |
| F2 | F2.A (multi-scope read from `session.tags`; no Session shape change) | ✅ Yes | `parse_session_scope_tags` (manifest/mod.rs:229) parses `org:<uuid>` / `project:<uuid>` prefixes from `session.tags` into `Vec<OrgId>` / `Vec<ProjectId>`. `Session` struct unchanged. ADR-0051 §D51.4 pins. **No migration.** |
| F3 | F3.A (Step-2a-style re-clamp + new `DeniedReason::IntersectionEmpty`) | ✅ Yes | `cascade_intersection_fallback` (engine.rs:576) re-runs `ceiling_admits` (engine.rs:299) — the same predicate Step 2a uses. Empty intersection → `DeniedReason::IntersectionEmpty`. ADR-0051 §D51.5 pins. |
| F4 | F4.A (`IntersectionEmpty { fundamental, action, session_scope_count: u8 }`) | ✅ Yes | `DeniedReason::IntersectionEmpty` variant at decision.rs:132 with the 3 required fields. Explicit per-variant arms (no `_ =>` catch-all) at the 2 server-side mappers. ADR-0051 §D51.7 pins. |

✅ All 4 fork locks honored.

---

## §7 — Drift lifecycle audit

| Drift | Pre-CH-07 status | Post-CH-07 status | Lifecycle entry verified |
|---|---|---|---|
| D-new-06 | discovered (HIGH) | **remediated** | `D-new-06.md` lifecycle: "2026-05-07 — remediated — CH-07 cycle cc912d07 — full 2-tier cascade + intersection fallback shipped at engine.rs::step_5_scope_resolution per ADR-0051; 10 unit tests + 6 acceptance tests cover the 5 distinct outcomes". README row updated `**remediated**` + `CH-07 ✓`. |
| D-new-20 | discovered (MEDIUM) | **remediated** | `D-new-20.md` lifecycle: "2026-05-07 — remediated — CH-07 cycle cc912d07 — membership-bounded clamp shipped at engine.rs::step_2a_ceiling per ADR-0051 §D51.6; 4 unit tests + Scenario 7 acceptance test". README row updated `**remediated**` + `CH-07 ✓`. |
| D-CH07-FOLLOWUP-01 | (new) | **discovered** (LOW) | New file at `m5_1/drifts/D-CH07-FOLLOWUP-01.md` ✅. Severity LOW, Bucket B, M6+ deferral target ✅. Body covers lexicographic-min placeholder + concept doc 06 line 58 reference + FIXME markers at engine.rs:510 + engine.rs:582 ✅. 4-segment relative paths (matches D-CH12/13-FOLLOWUP-01 sister patterns) ✅. README index row added ✅. |

✅ All 3 drift lifecycle transitions correct.

---

## §8 — Final cycle verdict

**GREEN.** CH-07 closes drifts D-new-06 (HIGH) + D-new-20 (MEDIUM). ADR-0051 Accepted with all 7 sub-decisions filled and 4 user-locks recorded. Concept docs 04 + 06 + 08 verified-header bumps applied with body unchanged. Architecture + operations docs shipped at `m5_2/`. Concept-audit-matrix 5 row Status flipped letter-for-letter to `**honored**`. Cycle index row flipped to `ready-for-audit` (P4) — orchestrator now flips to `retro-complete` after retrospective writes.

**Workspace state:**
- 1399 / 0 / 2 tests (within plan §8 [1399, 1403] band).
- Clippy + fmt + 4 CI guards all green.
- 0 phi-core leverage delta.
- 0 K8s deferred-ledger entries (chunk is K8s-neutral per plan §3.B).
- 1 new M6+ drift (D-CH07-FOLLOWUP-01, LOW).

**Ready for:**
1. **Retrospective** — chunk-retrospector spawn next.
2. **User commit** — after retrospective + standards-update review.

No re-spawn of Implementer or Planner needed. No further audit iterations needed.

---

## §9 — Cross-references

- [Plan](./plan.md) — 12-section per-chunk plan (cycle hex `cc912d07`).
- [Audit A](./audit-A-iter1.md) — code correctness + phi-core leverage (GREEN).
- [Audit B](./audit-B-iter1.md) — docs fidelity + concept alignment (GREEN).
- [ADR-0051](../../v0/implementation/m5_2/decisions/0051-multi-scope-cascade-contractor-model.md) — Accepted.
- [D-new-06](../../v0/implementation/m5_1/drifts/D-new-06.md), [D-new-20](../../v0/implementation/m5_1/drifts/D-new-20.md) — remediated.
- [D-CH07-FOLLOWUP-01](../../v0/implementation/m5_1/drifts/D-CH07-FOLLOWUP-01.md) — new, discovered, M6+.
- [Forward-scope row](../../forward-scope/22035b2a-remaining-scope-post-m5-p7.md) lines 84–89.
- [Multi-agent pipeline meta-plan](../../agentic-workflow/multi-agent-chunk-pipeline-0853574c.md).
