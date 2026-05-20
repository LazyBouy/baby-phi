<!-- Last verified: 2026-05-20 by Claude Code (post-CH-28 close; user-driven plan-mode session for chunk-decomposition + fork-framing + deferred-feature-visibility process improvements; rewritten from prior M6-forward-scoping content) -->

# Chunk decomposition + fork framing + deferred-feature visibility improvements

## Context

CH-28 (cycle `0412eb06`, just closed 2026-05-20) closed with workspace GREEN at 1599/0 but required **5 plan iterations + 2 Architectural-FAIL re-spawns** — the highest iteration count in baby-phi history. Across the cycle, the user observed three structural gaps in the multi-agent chunk pipeline that compounded the iteration cost:

**Observation 1 — Chunk size**: CH-28 bundled a cardinality redesign + 2 NEW migrations + 4 NEW Repository methods + composite-write infrastructure + edge rename + 7 acceptance tests into a single chunk (9 phases, 1599 tests, 26 files changed). The chunk SHOULD have been split into 2-3 functionally-cohesive blocks. The current pipeline (chunk-planner v25 + phase-planner v1) does not tag chunks as **functionally-comprehensive** vs **pure-technical-prerequisite**; this distinction is essential to keep chunk sizes user-decision-tractable.

**Observation 2 — Fork framing**: CH-28's 3 user-locked DIVERGENT forks (F1.c hybrid Blueprint table + F2.b USES_PROFILE rename + F3.b split migrations) were originally presented as engineering tradeoffs ("maximum auditability", "wire-format-explicit", "operator inspection window"). The user-facing impact (what an end user perceives if the option lands) was missing. The chunk-planner v23 fork-template DOES require `(a) user-impact summary + (b) pros/cons` (chunk-initiate SKILL.md lines 118-123) but the rule is being interpreted as **architectural-impact**, not user-perceived behavior. The CH-28 iter-1 fork bodies illustrate the gap: F1.a pros = *"per-agent governance lives WITH the per-agent identity"* (architectural, not user-perceived).

**Observation 3 — Deferred-feature visibility**: D-CH28-FOLLOWUP-01 (listener template-tier fan-out → M6-DEFERRED-04 / CH-36) described WHAT was deferred (architectural: 5-step traversal) but did NOT name the user-visible capability or the product impact while deferred ("agents sharing a template see independent profile changes instead of synchronized changes"). The forward-scope §3 M*-DEFERRED-NN markers similarly lack product-feature naming. The user reads these and perceives "essential features being dropped" because they cannot trace the deferred work back to a product capability + its v0 vs final state.

**Intended outcome**: A 7-deliverable update to the multi-agent chunk pipeline that (a) tags chunks FUNCTIONAL vs TECHNICAL-PREREQUISITE, (b) enforces user-facing fork framing with explicit "Defers (if chosen)" routing, (c) ships a single canonical "feature inventory" doc that lets the user verify product-trajectory coverage at any planning gate.

**User-locked decisions (this session, 2026-05-20)**:
- Feature-inventory location: `docs/specs/v0/feature-inventory.md` (top of v0).
- Authoring: orchestrator hand-authored (single-session; pulls from concept-docs + build-plan + forward-scopes + M*-DEFERRED markers).
- Retroactive re-frame of m6-forward-scope: in this plan as Phase 6 (validates new shape end-to-end before CH-29 opens).
- Fork framing strictness: **mandatory** at chunk-planner v26 self-check (mirrors v23 P13 ALWAYS-FIRE pattern; planner self-greps own draft + retries until `**User-visible:**` + `**Product trajectory:**` substrings present, OR fork header carries `TECHNICAL FORK` label).

**Plan archive target (post-approval)**: this plan archives verbatim to `/root/projects/phi/baby-phi/docs/specs/plan/review-and-docs/chunk-decomposition-and-fork-framing-<8hex>.md` (slug-first, hex-suffix per `feedback_plan_archive_naming.md`).

---

## Project-agnostic reuse discipline (user-reinforced 2026-05-20)

**Binding constraint**: agents at `/root/projects/phi/.claude/agents/` + skills at `/root/projects/phi/.claude/skills/` are shared across baby-phi + i-phi (and future projects). Any rule changes to these agent/skill files MUST be project-agnostic — no baby-phi-specific path literals in rule TEXT, no baby-phi-specific concept-doc references hard-coded into agent prompts, no rules that only make sense in baby-phi's tree.

**Per-deliverable project-agnostic audit**:

| Deliverable | Target | Project-agnostic? | Discipline applied |
|---|---|---|---|
| **D1** — per-chunk-planning-template.md update | `baby-phi/docs/specs/v0/implementation/m5_1/process/per-chunk-planning-template.md` | **Project-specific (correct)** | Lives in baby-phi's tree (per-project template path); chunk-planner looks it up via PROJECT_ROOT-aware path resolution. i-phi may want an equivalent template update — file as separate i-phi-side task post-this-plan if i-phi has its own template. |
| **D2** — chunk-planner.md v25 → v26 | `.claude/agents/chunk-planner.md` | **Must be project-agnostic** | NEW P-plan-1-v26 rule body uses generic language ("the chunk plan", "the forks section") — NOT "baby-phi's §3". CH-28 / ADR-0063 may be cited as example-evidence in a rationale paragraph, but rule TEXT (the imperative MUST/MUST NOT directives) is project-agnostic. Self-check grep targets generic substrings (`**User-visible:**`, `**Product trajectory:**`, `TECHNICAL FORK`) — no project-specific path literals. |
| **D3** — phase-planner.md v1 → v2 | `.claude/agents/phase-planner.md` | **Must be project-agnostic** | FUNCTIONAL / TECHNICAL-PREREQUISITE tagging applies to any project's milestone decomposition. Decomposition heuristics (≥30% supporting-infra effort triggers split; max-2-consecutive-TECHNICAL-PREREQUISITE chain cap) are generic. CH-28 cited only as evidence-of-pattern in rationale, not embedded in rule text. |
| **D4** — forward-scope row shape | (documented in phase-planner.md output schema) | **Project-agnostic schema** | The 2 NEW columns (`Chunk-type` + `User-visible delivery`) applied uniformly to any project's forward-scope output by phase-planner v2. |
| **D5** — chunk-initiate skill 4-line template | `.claude/skills/chunk-initiate/SKILL.md` | **Must be project-agnostic** | The 4-line `description` template (`User-visible:` + `Product trajectory:` + `Cycle scope:` + `Defers (if chosen):`) is generic; works for baby-phi forks AND i-phi forks. The skill already takes a `project=baby-phi \| i-phi` input parameter; the new template applies uniformly across both project-modes. |
| **D6** — feature-inventory.md (NEW) | `baby-phi/docs/specs/v0/feature-inventory.md` | **Project-specific (correct)** | Feature inventories are per-project product-trajectory trackers. **Follow-up note**: i-phi will need its own equivalent at `i-phi/docs/v0/feature-inventory.md` (or equivalent path) authored at a future i-phi planning session — out-of-scope HERE. |
| **D7** — m6-forward-scope retroactive re-frame | `baby-phi/docs/specs/plan/forward-scope/m6-forward-scope-8b7a8bcd.md` | **Project-specific (correct)** | Re-frames a single baby-phi forward-scope file. i-phi forward-scopes (if any) re-frame under separate planning when phase-planner v2 next emits them. |

**Self-verification grep at deliverable-close** (added as a Phase-N self-check step for D2 + D3 + D5):

```bash
# Generic discipline grep — should return ZERO baby-phi-specific path literals in RULE TEXT
grep -nE "baby-phi/docs/|baby-phi/modules/|baby-phi-specific|m5_1/process/|m6/decisions/" \
  /root/projects/phi/.claude/agents/chunk-planner.md \
  /root/projects/phi/.claude/agents/phase-planner.md \
  /root/projects/phi/.claude/skills/chunk-initiate/SKILL.md
```

Grep hits in **rationale paragraphs** that cite CH-28 / ADR-0063 / cycle hex as example-evidence are acceptable. Hits in **rule TEXT** (imperative MUST/MUST NOT directives) MUST be patched BEFORE deliverable-close.

---

## Approach

7 sequential deliverables. Each gated on user approval before advancing.

### Deliverable 1 — Update per-chunk-planning-template with functional outcome + product-impact framing

**Target**: `/root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_1/process/per-chunk-planning-template.md`

Add 3 structural changes:

1. **NEW §2.5 — Functional outcome (mandatory)**: a 1-paragraph statement naming the user-visible capability the chunk ships. Format:
   - For FUNCTIONAL chunks: `**User-visible delivery**: <one-paragraph description of what the end user can do post-chunk-close that they could not do before>`
   - For TECHNICAL-PREREQUISITE chunks: `**User-visible delivery**: NONE this chunk. **Unblocks**: <CH-NN+M> which ships <user-visible feature>. **Why this prerequisite**: <one-sentence explanation a non-technical user can understand>`

2. **§3 Forks for orchestrator (REFINE)**: Each fork option's row MUST include a **User-visible** prefix line at the top of pros AND a **Product trajectory** line in cons. Format:
   ```
   | Option | User-visible (what the user perceives) | Pros | Cons + Product trajectory | Status |
   ```
   Forks where ALL options are zero-user-visible-delta (purely engineering choice) MUST be labeled `TECHNICAL FORK (no user-visible delta — pick on engineering merit)` in the fork header — releases the planner from the user-facing framing requirement for that fork.

3. **§4 Drifts closed + Deferred functionality (EXTEND)**: For each drift / follow-up / M*-DEFERRED-NN marker the chunk files, add a mandatory `Product impact during deferral` column. Format:
   ```
   | Drift ID | Title | User-visible feature deferred | Product impact during deferral | Allocation chunk | Cross-chunk dep |
   ```

### Deliverable 2 — Update chunk-planner.md v25 → v26 with strict user-facing fork framing

**Target**: `/root/projects/phi/.claude/agents/chunk-planner.md` (currently v25 at line 7)

Add NEW P-plan-1-v26 rule near current line 749 (before `## Constraints`):

- Each fork's option `pros` cell MUST lead with `**User-visible:** <one-sentence behavior the end user perceives if this option ships>`.
- Each fork's option `cons` cell MUST end with `**Product trajectory:** <how this option compares for long-term product goals — what capabilities are easier/harder to build downstream>`.
- Forks where ALL options share zero user-visible delta MUST be labeled at the fork header: `**TECHNICAL FORK** (no user-visible delta — pick on engineering merit only)`. This releases the user-facing framing requirement for that fork.
- Mandatory self-check at draft-end (mirror v23 P13 pattern): grep own draft for each fork header; if NOT labeled `TECHNICAL FORK`, verify each option row has `**User-visible:**` + `**Product trajectory:**` substrings. Retry until present.

Bump version frontmatter 25 → 26 and update project-context note appending the v26 update line.

### Deliverable 3 — Update phase-planner.md v1 → v2 with FUNCTIONAL / TECHNICAL-PREREQUISITE chunk tagging

**Target**: `/root/projects/phi/.claude/agents/phase-planner.md` (currently v1)

Modify Step 5 in the 11-step procedure to require:

1. Each CH-NN row carries a NEW **`chunk-type`** field ∈ {FUNCTIONAL, TECHNICAL-PREREQUISITE}.
2. FUNCTIONAL chunks MUST name the user-visible feature delivered at close.
3. TECHNICAL-PREREQUISITE chunks MUST cite (a) the FUNCTIONAL chunk they unblock + (b) a one-sentence rationale a non-technical user can understand.
4. NEW decomposition discipline: when a candidate chunk's deliverables mix user-visible feature work + supporting infrastructure with no user delivery, the phase-planner MUST evaluate splitting into 2 chunks. Heuristic: if the supporting infrastructure represents ≥ 30% of the candidate chunk's effort estimate, propose a split.
5. NEW cap on TECHNICAL-PREREQUISITE chains: no more than **2 consecutive** TECHNICAL-PREREQUISITE chunks before a FUNCTIONAL chunk lands. Long technical-prerequisite chains hide product progress from the user; force interleaving with feature delivery.

Bump version frontmatter 1 → 2.

### Deliverable 4 — Extend forward-scope row shape

**Target**: applied to all NEW forward-scope files authored by phase-planner v2; documented in `phase-planner.md` step 5 output schema.

Forward-scope §5 per-chunk scope summary table MUST gain 2 columns:

| ... existing columns ... | **Chunk-type** | **User-visible delivery** |
|---|---|---|

`Chunk-type` ∈ {FUNCTIONAL, TECHNICAL-PREREQUISITE}.
`User-visible delivery` is one short line. For TECHNICAL-PREREQUISITE: `unblocks CH-NN (ships <feature>)`.

Forward-scope §1 per-chunk narrative blocks MUST gain 2 explicit lines:
- `Functional outcome:` 1-paragraph user-visible capability statement (or `NONE this chunk — technical prerequisite for CH-NN`).
- `Defers (with product impact):` enumerate any features NOT shipping this chunk + allocation + product impact during deferral.

### Deliverable 5 — Update chunk-initiate skill gate-1 AskUserQuestion 4-line template

**Target**: `/root/projects/phi/.claude/skills/chunk-initiate/SKILL.md` (Phase 1.5 gate-1 fork-lock rules at lines 118-123)

Replace the current `(a) high-level user-impact + (b) pros/cons` template with a STRICT 4-line template per fork option's `description` field:

```
**User-visible:** <what users perceive if this option lands>
**Product trajectory:** <how this affects overall product trajectory — what becomes easier/harder downstream>
**Cycle scope:** <effort + cascade scope — engineering tradeoff for this chunk>
**Defers (if chosen):** <features NOT shipping this chunk; allocation chunk-IDs>
```

The 4th line ("Defers (if chosen)") is NEW — it surfaces what the user is implicitly NOT choosing when they pick this fork. Closes the "perception that essential features will be lost" gap.

For forks labeled `TECHNICAL FORK` (zero user-visible delta across all options), the orchestrator MAY use the 2-line minimal template `Cycle scope` + `Defers (if chosen)` only — release from User-visible + Product trajectory requirements.

### Deliverable 6 — Author NEW feature-inventory doc as cross-chunk product-trajectory tracker

**Target**: NEW file at `/root/projects/phi/baby-phi/docs/specs/v0/feature-inventory.md`

**Shape** (sections):
- **§1 Purpose**: cross-chunk product-trajectory tracker; lets the user verify v0 product coverage at any planning gate.
- **§2 Feature inventory** (table): every user-facing product feature in the v0.1 build plan. Columns:
  ```
  | Feature ID | Feature (user-facing name) | v0 scope | Closing chunk(s) | Deferred sub-aspects | v0 vs final state |
  ```
- **§3 Deferred catalogue**: every M*-DEFERRED-NN marker + every D-CH<NN>-FOLLOWUP drift, indexed by feature. For each: feature impact + user-visible state in v0 vs final + allocation chunk + cross-chunk dependency chain.
- **§4 Cross-chunk dependency graph**: ASCII art showing feature → chunk dependencies for v0 (mirrors forward-scope §4 but at product-feature granularity, not engineering-effort granularity).
- **§5 Revisit triggers**: when this doc should be re-authored (every new milestone forward-scope; every M*-DEFERRED-NN deferral routing decision).

**Authored at this plan**: orchestrator hand-authors v0 first cut. Pulls feature list from:
- `docs/specs/v0/concepts/{agent.md, ontology.md, system-agents.md, permissions/*}`
- `docs/specs/plan/build/build-plan-v01-36d0c6c5.md` (milestone-level scope per M1..M7b)
- Existing M6 forward-scope §1 chunks (CH-28..CH-38)
- Existing M*-DEFERRED-NN markers + drift files

### Deliverable 7 — Retroactive re-frame of m6-forward-scope-8b7a8bcd.md

**Target**: `/root/projects/phi/baby-phi/docs/specs/plan/forward-scope/m6-forward-scope-8b7a8bcd.md`

Apply Deliverables 3 + 4 retroactively:
- Add `Chunk-type` + `User-visible delivery` columns to §5 summary table for all 11 chunks (CH-28..CH-38).
- Add `Functional outcome:` + `Defers (with product impact):` lines to each §1 chunk narrative block.
- Re-evaluate CH-29..CH-38 decomposition: any chunk that should split further per Deliverable 3's new heuristics? Surface candidates as `## Open question` entries appended to §6.
- Verified-header amended with CH-28-retro-driven amendment line citing this plan archive's hex.

**Sequence**: this deliverable runs AFTER Deliverables 1-5 land + Deliverable 6 first cut authored. The retroactive re-frame validates the new shape against an existing forward-scope before CH-29 opens under the new rules.

---

## Critical files

**MODIFIED (5)**:
1. `/root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_1/process/per-chunk-planning-template.md` — Deliverable 1 (§2.5 NEW + §3 fork-row refine + §4 deferred-functionality extend)
2. `/root/projects/phi/.claude/agents/chunk-planner.md` — Deliverable 2 (v25 → v26 with P-plan-1-v26 user-facing fork framing rule)
3. `/root/projects/phi/.claude/agents/phase-planner.md` — Deliverable 3 (v1 → v2 with FUNCTIONAL / TECHNICAL-PREREQUISITE tagging + decomposition discipline + chain cap)
4. `/root/projects/phi/.claude/skills/chunk-initiate/SKILL.md` — Deliverable 5 (gate-1 4-line template + Defers line + TECHNICAL FORK 2-line release)
5. `/root/projects/phi/baby-phi/docs/specs/plan/forward-scope/m6-forward-scope-8b7a8bcd.md` — Deliverable 7 (retroactive re-frame to new column shape + per-chunk narrative)

**NEW (2)**:
1. `/root/projects/phi/baby-phi/docs/specs/v0/feature-inventory.md` — Deliverable 6 (NEW cross-chunk product-trajectory tracker)
2. `/root/projects/phi/baby-phi/docs/specs/plan/review-and-docs/chunk-decomposition-and-fork-framing-<8hex>.md` — verbatim plan archive (orchestrator-applied post-approval)

**CHANGELOG (1)**:
1. `/root/projects/phi/.claude/agents/_changelog.md` — new top-prepended row covering all version bumps + new doc + plan archive

**Reused references (read-only)**:
- `/root/projects/phi/baby-phi/docs/specs/plan/build/ch-28-agentprofile-cardinality-redesign-0412eb06/{plan.md, retrospective.md, cycle-audit.md}` — empirical evidence anchoring the rule changes
- `/root/projects/phi/.claude/agents/_changelog.md` line 1 (CH-28 retro batch row) — context for what already shipped at CH-28 retro

---

## Sequencing

Seven sequential phases. User-approval gate after each phase.

**Phase 1** — Deliverable 1: per-chunk-planning-template update (~0.3d)
**Phase 2** — Deliverable 2: chunk-planner v26 update (~0.3d)
**Phase 3** — Deliverable 3: phase-planner v2 update (~0.3d)
**Phase 4** — Deliverable 5: chunk-initiate skill 4-line gate-1 template (~0.2d)
**Phase 5** — Deliverable 6: feature-inventory.md first cut hand-authored (~0.5-1d depending on milestone breadth)
**Phase 6** — Deliverable 7: m6-forward-scope retroactive re-frame (~0.3d)
**Phase 7** — Changelog row + plan archive + commit (~0.2d)

**Total estimated effort**: ~2-2.8 engineer-days.

**NOT in scope this plan**:
- No re-decomposition of M7+ chunks (M7+ §3 markers stay as-is until M7 plan-mode opens).
- No re-author of pre-CH-28 closed-cycle plans (immutable per chunk-archive-plan v3).
- No re-author of pre-CH-28 closed-cycle ADRs.
- No CH-29+ plan-mode opening (CH-29 opens under new rules AFTER this plan completes in a separate session).

---

## Verification

End-to-end verification after all 7 deliverables land:

```bash
# Deliverable 1: per-chunk-planning-template has new §2.5 + refined §3 + extended §4
grep -nE "^## §2\.5 — Functional outcome|^## §3 — Forks|User-visible delivery|TECHNICAL FORK|Product impact during deferral" \
  /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_1/process/per-chunk-planning-template.md

# Deliverable 2: chunk-planner v26 with NEW P-plan-1-v26 rule
grep -E "^version: 26$" /root/projects/phi/.claude/agents/chunk-planner.md
grep -c "P-plan-1-v26" /root/projects/phi/.claude/agents/chunk-planner.md  # expect ≥ 1

# Deliverable 3: phase-planner v2 with FUNCTIONAL tagging
grep -E "^version: 2$" /root/projects/phi/.claude/agents/phase-planner.md
grep -cE "FUNCTIONAL|TECHNICAL-PREREQUISITE" /root/projects/phi/.claude/agents/phase-planner.md  # expect ≥ 3

# Deliverable 5: chunk-initiate skill 4-line template + Defers line
grep -nE "User-visible:|Product trajectory:|Cycle scope:|Defers \(if chosen\):" \
  /root/projects/phi/.claude/skills/chunk-initiate/SKILL.md

# Deliverable 6: feature-inventory.md exists with all sections
ls /root/projects/phi/baby-phi/docs/specs/v0/feature-inventory.md
grep -cE "^## §[1-5]" /root/projects/phi/baby-phi/docs/specs/v0/feature-inventory.md  # expect ≥ 5

# Deliverable 7: m6-forward-scope reframed
grep -c "Chunk-type" /root/projects/phi/baby-phi/docs/specs/plan/forward-scope/m6-forward-scope-8b7a8bcd.md  # expect ≥ 1
grep -c "Functional outcome:" /root/projects/phi/baby-phi/docs/specs/plan/forward-scope/m6-forward-scope-8b7a8bcd.md  # expect 11 (CH-28..CH-38)

# Plan archive landed
ls /root/projects/phi/baby-phi/docs/specs/plan/review-and-docs/chunk-decomposition-and-fork-framing-*.md

# 4 CI guards stay green (no source code changes; check-doc-links validates new + reframed docs' relative links)
bash /root/projects/phi/baby-phi/scripts/check-doc-links.sh
bash /root/projects/phi/baby-phi/scripts/check-ops-doc-headers.sh
bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh
bash /root/projects/phi/baby-phi/scripts/check-spec-drift.sh

# Working tree state
git -C /root/projects/phi status --short
git -C /root/projects/phi/baby-phi status --short
```

**Acceptance**: all 7 deliverables exist at cited paths; new template + chunk-planner v26 + phase-planner v2 + chunk-initiate skill enforce the new rules; feature-inventory.md ships with all sections populated; m6-forward-scope §5 + §1 reframed; 4 CI guards GREEN; user explicitly approves each phase gate.

---

## NOT this plan

- Does NOT modify any source code under `modules/crates/`.
- Does NOT open any CH-NN plan-mode (CH-29 opens under new rules in a separate session after this plan completes).
- Does NOT re-author pre-CH-28 closed-cycle artifacts (plans, ADRs, drifts — all immutable per chunk-archive-plan v3).
- Does NOT touch M7+ forward-scope decomposition (stays as topic-markers until M7 plan-mode).
- Does NOT commit any work — at each phase boundary the user owns commit timing.
