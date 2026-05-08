<!-- Last verified: 2026-05-08 by Claude Code (CH-15 cycle-audit gate-4 — orchestrator final re-audit GREEN; MUST-RUN clippy + 4 CI guards executed authoritatively; tests 1462/0/2; cycle hex `c3f46f17`) -->

# CH-15 cycle audit — Real permission-check gate at session launch (closes drift D4.1)

**Cycle hex:** `c3f46f17`
**Date:** 2026-05-08
**Author:** Claude Code (orchestrator)
**Plan:** [plan.md](./plan.md)
**Audit logs:** [audit-A-iter1.md](./audit-A-iter1.md), [audit-B-iter1.md](./audit-B-iter1.md), [audit-B-iter2.md](./audit-B-iter2.md)
**Verdict:** GREEN

---

## §1 — Audit pipeline summary

| Stage | Auditor | Iter | Verdict | Notes |
|---|---|---|---|---|
| Sub-agent audit | A — code + phi-core + K8s | 1 | PASS (clean) | 13/13 claims; 2 clarifications: claim 4 URI form is selector-grammar predicate `tags contains "project:{uuid}" AND tags contains #kind:session` (semantically equivalent to ADR §D54.3 "selector form"); claim 10 phi-core baseline = 49 via plan's `grep -rn "use phi_core"` (orchestrator's stricter `grep -rn "use phi_core::"` returns 48 — measurement-method drift, not a substantive Δ). |
| Sub-agent audit | B — concept + docs + ADR | 1 | PARTIAL | 12/13 PASS / 1 FAIL on claim 13 (doc-sync sweep): `m5/architecture/authority-templates.md:89-91` retained stale `§"P5 advisory — D4.1 carry-forward"` text claiming "At M5, the launch chain gates on Step 0 (Catalogue) only; steps 1-6 are advisory" — factually contradicted by CH-15's hard-deny ship. File was outside CH-15 plan §3.C doc-impact map. Same gap class as CH-14 audit B iter 1 (cross-doc stale reference outside plan's enumerated map). |
| Trivial-multi orchestrator-applied patch | orchestrator | n/a | applied | Replaced §"P5 advisory — D4.1 carry-forward" body with §"Permission Check at session launch (CH-15 hard-deny)" — accurate post-CH-15 narrative; cross-refs ADR-0054 + `m5_2/architecture/session-launch-permission-gate.md` + migration 0015. Verified-header line 1 prepended with CH-15 trivial-multi stamp + gate-2-sweep-gap acknowledgement. `check-doc-links.sh` re-run post-patch: PASS. |
| Sub-agent audit | B re-audit | 2 | PASS (clean) | 13/13 claims; prior FAIL resolved + no regressions. |
| Orchestrator final cycle re-audit | self | n/a | PASS (this doc) | MUST-RUN list executed authoritatively; see §3. |

**Iteration accounting:** Audit-fix-loop iteration count for CH-15 = **1**. Per CLAUDE.md trivial-multi protocol, the orchestrator-applied 1-doc patch + Audit B re-spawn at iter 2 is the canonical re-audit step (NOT a Tactical-FAIL re-spawn of the implementer). No gate-2 inline correction was needed (in contrast to CH-14): the implementer's chunk-seal cross-check confirmed ADR-0054 ↔ D4.1 agreement on all 4 claims (per chunk-implementer v5 Row 1 discipline).

---

## §2 — User-locked forks

| Fork | Locked at | Path | Recommendation alignment |
|---|---|---|---|
| F1 — Template A grant-extension strategy | gate-1 plan-approval | F1.A (Vec<Grant> return + migration 0015 backfill) | aligns w/ planner |
| F2 — Manifest builder location | gate-1 plan-approval | F2.A (NEW `domain::permissions::builders` module) | aligns w/ planner |
| F3 — Action enum variant placement | gate-1 plan-approval | F3.A (reuse `[Read, Inspect, List]` on `session_object`; forward-scope re-interpretation per ADR §D54.2 + §D54.8) | aligns w/ planner |
| F4 — Hard-deny flip semantics | gate-1 plan-approval | F4.C (migration 0015 runs BEFORE launch.rs flip) | aligns w/ planner |
| F5 — Engine deny audit-event path | gate-1 plan-approval | F5.B (NEW `platform.session.launch_denied` event in `m5_2/audit/events/session_launch.rs`) | aligns w/ planner |

ADR-0054 §"Forks" header populated correctly per CH-13 v2 ADR-body checklist + CH-08 retro Row 1 milestone-prefixed paths.

**F3.A is the architecturally consequential fork**: the forward-scope row's literal text (`session.start` / `session.tool_invoke` / `session.read_memory` as actions) was reinterpreted because concept-doc 03's closed 34-verb vocabulary precludes adding new variants. ADR-0054 §D54.2 + §D54.8 carry the rationale; `Action::CANONICAL.len() == 34` invariant is preserved.

---

## §3 — Orchestrator MUST-RUN list (gate-4)

Sub-agent auditors marked workspace-tests + clippy + the 4 CI guards as PASS-with-caveat (sandbox concerns). Per CLAUDE.md the orchestrator runs them authoritatively here.

| Command | Result |
|---|---|
| `RUSTFLAGS="-Dwarnings" cargo clippy -j 4 --workspace --all-targets` | **PASS** (exit 0; zero warnings) |
| `bash scripts/audit-tmp-cargo-counts.sh` (cargo test --workspace -j 4) | **PASS** (1462 / 0 / 2 ignored — at lower bound of plan §8 band [1462, 1490]) |
| `bash scripts/check-doc-links.sh` | **PASS** ("all markdown under docs/specs/v0/implementation has valid relative links + verification headers. OK.") |
| `bash scripts/check-ops-doc-headers.sh` | **PASS** ("all 33 ops doc(s) carry the 'Last verified' header. OK." — bumped from 32 with the new operations doc) |
| `bash scripts/check-phi-core-reuse.sh` | **PASS** ("no forbidden phi-core redeclarations under modules/crates.") |
| `bash scripts/check-spec-drift.sh` | **PASS** ("29 referenced ids all present in docs/specs/v0/requirements.") |
| `cargo fmt --all -- --check` | **PASS** (0 diffs) |
| `grep -rn "use phi_core" modules/crates/ \| wc -l` (plan's canonical grep) | **49** (baseline preserved; predicted Δ = 0) |
| `bash /root/projects/phi/baby-phi/scripts/audit-tmp-cargo-counts.sh` (CH-14 retro Row 8 refactor) | **executed cleanly** — replaces 4-stage `cargo test \| grep \| sed \| awk` pipeline; 0 PermissionRequest fires expected (validate at retro time) |

All MUST-RUN claims close at GREEN. Caveat tags from sub-agent auditors are dismissed.

---

## §4 — Concept-alignment matrix flips

2 rows flipped letter-for-letter from plan §2 target column per CH-12 F-AUDB-1 rule (per Audit B iter 1 + iter 2 verified):

| Row anchor | Before | After |
|---|---|---|
| `permissions/04-manifest-and-resolution.md` (row 180) | `partially-honored` | `**honored**` |
| `permissions/07-templates-and-tools.md` (row 219) | `partially-honored` | `**honored**` |

---

## §5 — Drift transitions

| Drift | Before | After | Notes |
|---|---|---|---|
| D4.1 (HIGH) | `discovered` | `remediated` | Primary closure — advisory-only Permission Check retired; hard-deny on every Decision::Denied at steps 0–6; new `platform.session.launch_denied` audit event; Template A grants extended with paired `session_object` grant; migration 0015 backfills legacy holders. ADR-0054 ratifies all 8 sub-decisions. |

**No new follow-up drifts filed.** This is the second cycle in 5 to close cleanly without `CH-NN-FOLLOWUP-NN` (CH-08 was the prior).

---

## §6 — ADR-0054 close-out

- File: `m5_2/decisions/0054-session-launch-manifest-and-hard-deny-flip.md`
- Status: **Accepted** (was Proposed at P0).
- Sub-decisions: D54.1 (builder location — F2.A) / D54.2 (Action vocabulary preserved — F3.A re-interpretation) / D54.3 (Template A double-grant shape — F1.A) / D54.4 (migration 0015 backfill — F4.C) / D54.5 (audit-event placement — F5.B) / D54.6 (hard-deny error-mapping — Step N → 403) / D54.7 (preview-launch parity) / D54.8 (forward-scope row literal re-interpretation note).
- Cross-references: ALL 4 categories present (concept docs + closed drift D4.1 + 5+ prior ADRs cited with milestone-prefixed paths per CH-08 retro Row 1 + forward-scope row).
- §"Pre-existing-behaviour preservation note" present per CH-14 retro Row 10: explicitly documents the pre-CH-15 advisory-log behavior at launch.rs:198-244 + the new hard-deny behavior.
- Forks header: "F1.A / F2.A / F3.A / F4.C / F5.B user-locked at plan approval — all align with planner recommendation".

---

## §7 — Cycle metrics

| Metric | Value |
|---|---|
| Phases | 5 (P0–P4) per Medium audit envelope |
| Tests at chunk-open | 1431 / 0 / 2 ignored |
| Tests at chunk-close | **1462 / 0 / 2 ignored** (Δ = +31; lower bound of plan §8 band [1462, 1490]) |
| `cargo clippy --workspace --all-targets` | green (`-Dwarnings`) |
| `cargo fmt --check` | green |
| 4 CI guards | green |
| phi-core import baseline (plan's canonical grep `use phi_core`) | 49 → 49 (Δ = 0) |
| Migration count | 14 → 15 (additive grant backfill on existing Template A grants; CHK8S-D-05 unaggravated) |
| K8s deferred ledger | unchanged (no new CHK8S-D-NN entry; all 7 axes no-impact / compatible) |
| Locked forks | 5/5 user-locked at gate-1; 0 additional gate-2 user-locks |
| Audit iteration count | 1 (Audit B re-spawn at iter 2 IS the canonical re-audit for trivial-multi orchestrator-applied 1-doc patch — separate counter on Audit B's logs only) |
| New follow-up drifts | 0 (cleanest close since CH-08) |
| Cascade-discipline outcome (Artifact A) | `fire_grant_on_lead_assignment` Vec<Grant> ripple: predicted ~12 deliberate edits / threshold 18; actual 29 edits — implementer flagged at chunk-seal; orchestrator gate-2 review verified the cascade is mechanical (Vec<Grant> destructure ripple) and fully contained to predicted 5 files (templates/a.rs + events/listeners.rs + template_a_firing_props.rs + audit/events/m4/templates.rs + new acceptance file); no scope-narrowing. **Accepted as legitimate.** Flagged for retrospector pickup at gate 5 — pause-discipline threshold may need refinement for Vec<Grant>-style typed-multi-value cascades. |
| Cascade-discipline outcome (Artifact B) | `build_session_launch_manifest` callsite cascade: 8 raw greps; 2 production callsites (preview.rs + launch.rs) + 4 new files. Within band. |
| Cascade-discipline outcome (Artifact C) | Decision/FailedStep match cascade: 1 callsite edit. Within band (no new exhaustive-match callsites added). |
| Files modified | 22 baby-phi files (1237+/263-) + 7 new files (builders/mod.rs + builders/session_launch.rs + audit/events/m5_2/session_launch.rs + migration 0015 + ADR-0054 + 2 new user-facing docs + new acceptance test) |

---

## §8 — Surface-level verification

- ✅ `domain::permissions::builders::build_session_launch_manifest(project_id, tools)` ships at `permissions/builders/session_launch.rs`; re-exports at `permissions/builders/mod.rs` + `permissions/mod.rs`.
- ✅ `Action::CANONICAL.len() == 34` invariant **preserved** (F3.A); `Action` enum unchanged.
- ✅ `fire_grant_on_lead_assignment` returns `Vec<Grant>` at `templates/a.rs:128`; listener at `events/listeners.rs` iterates Vec + persists each via `repo.create_grant`.
- ✅ Paired session-object grant: `holder = lead_agent`, `action = [Read, Inspect, List]`, `resource.uri = selector-grammar form `tags contains "project:{uuid}" AND tags contains #kind:session`` (per ADR §D54.3 selector form), tag `kind:session`, `descends_from = adoption_ar.id`.
- ✅ Migration `0015_template_a_session_object_grant.surql` ships; idempotent; `migrations.rs` row count 14 → 15.
- ✅ `launch.rs` Step 3 advisory-log → hard-deny flip: `match preview.decision { Decision::Denied { failed_step, reason } => return Err(SessionError::PermissionCheckFailed { step, reason }) }`.
- ✅ `preview.rs` + `launch.rs` BOTH call `build_session_launch_manifest` (preview-launch parity preserved per ADR §D54.7).
- ✅ New audit event `platform.session.launch_denied` at `audit/events/m5_2/session_launch.rs`; canonical_bytes contributors listed in ADR §D54.5; audit class `Alerted`.
- ✅ User-facing docs synced: `m5_2/architecture/session-launch-permission-gate.md` (NEW ~250 words) + `m5_2/operations/session-launch-permission-gate-operations.md` (NEW ~140 words) + `m5/architecture/session-launch.md` (Step 3 rewrite) + `m5/operations/session-launch-operations.md` (error code + playbook) + `m5/user-guide/first-session-walkthrough.md` (CH-15 amendment) + `m5/user-guide/troubleshooting.md` (CH-15 amendment) + `m5/architecture/authority-templates.md` (gate-2 trivial-multi orchestrator patch).
- ✅ `_cycle-index.md` row added for CH-15.
- ✅ `permissions-audit` skill — to be run in gate 5 retrospective.

---

## §9 — Notable findings flagged for retrospector (gate 5)

1. **Doc-sync sweep gap (Audit B iter 1 FAIL)**: `m5/architecture/authority-templates.md:89-91` carried stale "advisory at M5" narrative because the file was outside the plan's enumerated §3.C doc-impact map. Same gap class as CH-14 audit B iter 1. **Standards update candidate**: extend the gate-2 doc-sync sweep grep set to ALL `m*/architecture/*.md` files (not just plan §3.C-listed ones) when a chunk closes a drift with cross-cutting documentary impact (D4.1's "advisory at M5" wording was scattered across 7 files; only 6 were in the plan's map).

2. **Cascade-discipline pause-threshold calibration (Artifact A)**: Vec<Grant> typed-multi-value return change rippled through 29 deliberate edits vs predicted 12 / threshold 18. Cascade was mechanical + contained to predicted files. **Standards update candidate**: when a cascade is a typed-multi-value return-type change (CascadeResult-style precedent per CH-14), the planner should size the pause-threshold against the test-amendment count, not just the deliberate-edit count — destructure-from-tuple → destructure-from-Vec patterns inflate test-edit counts mechanically.

3. **Phi-core import baseline measurement-method drift**: plan + auditor used `grep -rn "use phi_core"` (returns 49), orchestrator used `grep -rn "use phi_core::"` (returns 48). Both are valid measures but produce different counts. **Standards update candidate**: codify the canonical grep in chunk-planner v8 + chunk-auditor v6 to ensure all 4 lanes converge on one number (per CH-08 retro Row 3's canonical grep-pattern naming rule, applied to phi-core imports).

4. **F3 forward-scope re-interpretation precedent**: this is the first cycle where the forward-scope row's literal text disagreed with concept-doc canonical phrasing AND the plan codified the divergence via a dedicated ADR sub-decision (§D54.2 + §D54.8). **Standards update candidate**: codify the "forward-scope vs concept-doc precedence" rule in per-chunk-planning-template §3 — concept doc wins; forward-scope row's wording becomes a documented re-interpretation in the relevant ADR.

5. **Bash-check matcher-bug-confirmed validation (CH-14 retro Row 6 — pending)**: settings.json was updated at CH-14 close with 5 literal-script-name rules. CH-15 is the first cycle to validate them. Permissions-audit skill at gate 5 will report whether the bash-check cluster fires 0 times (validation passes — keep the rules) or non-zero (validation fails → escalate to upstream Claude Code rule-matcher bug-report per the matcher-bug-confirmed protocol).

---

## §10 — Final verdict

**GREEN.** CH-15 closes cleanly. All 5 user-locked forks honored, drift D4.1 (HIGH; the M5-defining advisory-only retirement) remediated, zero phi-core leverage delta, zero K8s blocker class, audit envelope held at Medium, sub-agent audits clean (Audit A iter 1 / Audit B iter 1→2 after orchestrator-applied trivial-multi 1-doc patch), MUST-RUN list authoritatively GREEN at gate 4. Zero new follow-up drifts.

Proceed to gate 5 (retrospective + standards-update review).

---

*Generated 2026-05-08 by Claude Code at orchestrator gate-4 close.*
