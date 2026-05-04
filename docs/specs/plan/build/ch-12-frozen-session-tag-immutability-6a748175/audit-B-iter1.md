<!-- Last verified: 2026-05-04 by Claude Code (chunk-auditor agent B iter 1) -->

# Audit B — CH-12 frozen session-tag immutability — iter 1

**Auditor:** Audit-B (concept fidelity + docs fidelity + ADR)
**Plan:** [./plan.md](./plan.md)
**Cycle hex:** 6a748175
**Word count:** ~560

## Per-claim PASS/FAIL

| # | Claim | Verdict | File:line evidence | Notes |
|---|-------|---------|---------------------|-------|
| 1 | ADR-0049 file exists; Status `**Status: Accepted**` | PASS | `0049-frozen-session-tag-immutability.md:6` reads `**Status: Accepted**` | One bold line. |
| 2 | D49.1–D49.7 documented (incl. D49.1.a Memory clarif.) | PASS | `0049…:40` D49.1, `:54` D49.1.a, `:58` D49.2, `:62` D49.3, `:76` D49.4, `:85` D49.5, `:89` D49.6, `:104` D49.7 — all 7 + sub-decision present | D49.1.a documented in P1 ADR draft (line 54-56). |
| 3 | All 5 forks documented; F5.B as USER-LOCK override | PASS | `0049…:30-34` F1.A/F2.A/F3.C/F4.A/F5.B; `:34` "(USER OVERRIDE)" + zero-cost rationale; `:128` "Why F5.B over planner-recommended F5.A" | Override rationale captured. |
| 4 | Cross-references concept docs + drift + prior ADRs | PASS | `0049…:165-175` lists ADR-0044, 0036, 0037, 0040, 0041, 0033 + concept docs 05/01/09/04 with line ranges 220–231/516–525/531–541, 254–261/267, 190–196 + D-new-08 + forward-scope row | All required anchors present. |
| 5 | D-new-08.md Status=`remediated` + CH-12 lifecycle entry | PASS | `D-new-08.md:11` `**Status**: remediated`; `:49` lifecycle entry mentions Rule E + `validate_tag_write_on_session` + audit-event builder per F5.B | Lifecycle entry complete. |
| 6 | drifts/README.md row flipped, "Closes at"=CH-12 ✓ | PASS | `drifts/README.md:105` `D-new-08 \| ... \| **remediated** \| CH-12 ✓` | Row flipped. |
| 7 | _concept-audit-matrix.md rows flipped to `honored`; CH-11 paperwork rule | PARTIAL/FAIL | matrix:190 `Session tags frozen at creation` → **honored**; matrix:237 `Reserved namespace write rejection` → **honored** with composite cite; matrix:191 `Session tag vocabulary` row was flipped to **honored** but plan §2 row 1 (`plan.md:145`) explicitly targets `partially-honored` post-chunk (emission-axis still aspirational) | **Mismatch between matrix and plan target status for "Session tag vocabulary".** Verified-header description matches body diff. See findings. |
| 8 | concept doc 05 verified-header bumped; body unchanged; +1 line | PASS | `05-memory-sessions.md:2` CH-12 line mentions `validate_tag_write_on_session` + Rule E + audit-event builder per F5.B; `:3` CH-21 line preserved; `git diff` shows pure +1-line insertion at line 2; `wc -l` = 676 | Body unchanged. |
| 9 | concept doc 01 verified-header bumped; body unchanged; +1 line | PASS | `01-resource-ontology.md:2` CH-12 line; `:3` CH-06 line preserved; `git diff` = pure +1-line; `wc -l` = 405 | Body unchanged. |
| 10 | concept doc 09 verified-header bumped; body unchanged; +1 line | PASS | `09-selector-grammar.md:2` CH-12 line; `:3` CH-05 line preserved; `git diff` = pure +1-line; `wc -l` = 221 | Body unchanged. |
| 11 | concept doc 04 verified-header bumped (mentions Rule E in A/B/C/D set); body unchanged; +1 line | PASS | `04-manifest-and-resolution.md:2` CH-12 line: "extended from 4 rules + 3 warnings (Rule A/B/C/D shipped at CH-05) to 5 rules + 3 warnings — adds **Rule E**…"; `:3` CH-05 line preserved; `git diff` pure +1-line; `wc -l` = 586 | Body unchanged. |
| 12 | Plan archive identical to draft modulo verified-header | PASS | `wc -l` both = 920; `diff /root/.claude/plans/sharded-discovering-stearns.md plan.md` = no output (zero diff) | Files byte-identical. |
| 13 | Cycle index row CH-12-6a748175 status=`ready-for-audit` | PASS | `_cycle-index.md:14` `\| 6a748175 \| CH-12 ... \| ready-for-audit \| _pending_ \|`; `:1` verified-header notes status flip in-flight → ready-for-audit at P3 | Row present. |
| 14 | CH-05/06/09/10/11 ADRs Accepted; prior drifts remediated; prior amendment lines preserved | PASS | ADR-0036/0037/0044/0045/0047/0048 all `**Status: Accepted**` (grep confirmed); README.md lines 100/101/102/104/108/114/128 show D-new-03/04/05/07/11/17/31 still remediated; concept-doc grep returned 2-3 prior-chunk amendment-line refs per file | All invariants intact. |
| 15 | Forward-scope CH-12 row (lines 123–128) preserved + §1.4 header | PASS | `22035b2a…md:121` `### Frozen tags + audit`; `:123-128` row body unchanged | Header + row preserved. |
| 16 | Carry-forward emission-gap retained as `partially-honored` | FAIL | Plan §2 row 1 (`plan.md:145`) + plan claim 16 (`plan.md:780`) require the §"Tag Vocabulary for Sessions" row to remain `partially-honored`; matrix row 191 was flipped to `honored` instead | See findings. |
| 17 | F5.B ADR coverage in §D49.7 | PASS | `0049…:104-128` §D49.7: variant `"tool.frozen_tag_write_rejected"` (line 120), `AuditClass::Alerted` (line 121), no new types/migration (line 128 "audit_events table is schema-stable"); user-override rationale at line 128 "User prioritises operator-visibility into retag-attempts" | Complete. |
| 18 | Architecture docs verified-header bumped; bodies unchanged; +1 each | PASS | `audit-events.md` git diff = pure +1-line at line 1 (CH-12 amendment mentions `tool.frozen_tag_write_rejected` + Alerted + F5.B + no migration); `permission-check-engine.md` git diff = pure +1-line at line 1 (CH-12 mentions Rule E + A/B/C/D/E rule set) | Both bumped, bodies untouched. |

## Summary

**OVERALL: TACTICAL FAIL** (1 strict FAIL on claim 16 + 1 PARTIAL on claim 7; same root finding).

### Findings

**Finding F-AUDB-1 (Tactical, single-row matrix overshoot).** The implementer flipped matrix row 191 ("Session tag vocabulary") from `partially-honored` to `honored`, but the plan explicitly required this row to **stay** `partially-honored` post-chunk (plan §2 row 1 line 145; §10 close criteria; plan claim 16 line 780; plan §3.C carry-forward defer). The plan's reasoning: CH-12 enforces immutability on whatever is currently emitted, but the **emission** of the 6 M6+ categories (`agent:`, `project:`, `org:`, `task:`, `role_at_creation:`, `agent_kind:`) remains aspirational and is deferred to a future M6+ "Session structural-tag emission" follow-up (recommended LOW drift `D-CH12-FOLLOWUP-01`).

The matrix's verified-header (line 1) and row-191 evidence cell DO accurately describe what shipped (the const lists 10 prefixes, 6 are forward-defensive). However, the **status column** should read `partially-honored` (matching the plan target), with the evidence cell explaining "immutability honored on the const; emission gap remains".

This is a one-cell paperwork mismatch — strictly a Trivial-multi tier FAIL (more than 1-line; also touches the verified-header description). Recommend orchestrator either (a) re-spawn implementer with this single-cell fix, or (b) accept the matrix as-shipped if the orchestrator agrees the row's evidence-cell phrasing makes the emission-axis carry-forward clear enough that downstream readers won't be misled. Per the plan's locked principle "thoroughness over cycle completion", option (a) is preferred.

**Finding F-AUDB-2 (Verified-header description fidelity).** The matrix verified-header at line 1 says "'Session tag vocabulary' row flipped from `partially-honored` to `honored`". This description is internally consistent with the (incorrect) flip — the CH-11-retro paperwork rule passes for the modified row. So claim 7's CH-11 paperwork-rule subclause is not itself a separate failure; the body and header are consistent. The failure is upstream: header and row are both wrong relative to the plan target.

### Recommendations

1. **Re-spawn implementer (Trivial-multi tier)** with the audit log path. Single fix: revert matrix row 191's status column to `partially-honored` and adjust the row's evidence-cell wording + verified-header description to match the plan's split-axis framing (immutability-axis honored; emission-axis carry-forward).
2. Re-spawn auditor B at iter 2 to confirm the cell + header.
3. All other 17 claims are GREEN (16 PASS + 1 PARTIAL that collapses into the same finding). ADR-0049 is exemplary in coverage.
4. F5.B coverage is fully captured. No architectural concerns.
