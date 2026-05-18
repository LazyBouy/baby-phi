<!-- Last verified: 2026-05-02 by Claude Code -->

# Multi-agent baby-phi chunk pipeline — agent set, skills, orchestration

**Type:** meta / process. No code changes in baby-phi or phi-core. Deliverables: 4 agent files + 6 skill files at the repo root under `.claude/`, an archived meta-plan + index under `baby-phi/docs/specs/agentic-workflow/`, a cycle-artifact pointer-index under `baby-phi/docs/specs/plan/build/`, CLAUDE.md addenda at repo + submodule levels.

**Estimated effort:** ~1.2 engineer-day for the full landing. Subsequent agent/skill refinements happen incrementally via retrospectives.

---

## Context

Today the chunk pipeline (pick → plan → implement → test → audit → seal → commit) runs end-to-end through me. Every step — the 12-section plan, every phase's code edits, every test run, both audits, and the chunk-close paperwork — concentrates on one thread. This caps parallelism and makes every chunk a one-shot, no-replay process: lessons are tacit, not codified.

The user wants to flip the loop. A 4-agent specialized set runs the chunk lanes; I evolve into the **reviewer / approver / process-refiner / retrospective-builder**. The quality bar (phi-core leverage, K8s compatibility, concept-doc fidelity, audit envelope, ADR rigor) must not slip — it must improve, because retrospectives explicitly target process gaps and propose template/CLAUDE.md updates.

User-locked forks (2026-05-02):
1. **Plan approval**: Planner drafts → I gate everything (forks via AskUserQuestion + ExitPlanMode). PLUS I should auto-approve when criteria are clearly met (no-fork scenarios — see §"Direct-approval criteria" below).
2. **Audit-fix loop**: Implementer re-spawned with audit feedback. PLUS Planner re-spawn for plan-level findings. PLUS a **final-by-me cycle re-audit** after all sub-agent audits green — quality and thoroughness over cycle completion. My job is observe + document + refine.
3. **Models**: **Opus across all 4 agents** (revised at plan-review 2026-05-02 from earlier "sonnet for implementer + retrospector" — quality bar trumps cost).
4. **Retrospective scope**: Process + standards updates + audit-cycle gap analysis (close gaps that emerge from the audit-fix loop). One **consolidated retrospective per cycle** (not per iteration).
5. **Cycle artifacts (NEW at plan-review 2026-05-02).** Every cycle produces 4 hex-tagged document classes — plan, per-iteration audit logs, cycle-level audit doc (consolidates all iterations + my final audit), retrospective. All share the same 8-hex cycle ID generated at plan-open. Naming + locations specified in §8.

---

## My evolved role (the human-paired reviewer)

I am no longer a doer in the chunk lane. I am the **gate-keeper between phases**, the **diff-reviewer**, the **audit-verifier**, and the **retrospective-driver**. Per `feedback_agent_verification.md` (memory): I never trust an agent's summary — I read every diff, run spot-checks, and only mark a phase complete after personally verifying.

| Phase | What I do |
|---|---|
| Pre-plan | Confirm prereqs from forward-scope; pick chunk; pass to Planner |
| Plan review | Read the planner's draft. Validate test strategy (§7 phase tests + §8 totals reasonable + property tests where state machines are touched). Apply Direct-approval criteria. If pass → ExitPlanMode myself with the agent's plan. If escalate → AskUserQuestion for forks, then ExitPlanMode after user answers |
| Implementation review | Per phase: read the diff, run `cargo test` + clippy myself, verify phi-core grep, check test count matches plan + tests cover plan-claimed cases, mark phase complete |
| Audit review | Read each iteration's audit log (written by auditor agent), spot-check 1–2 random claims by reading the cited code, decide PASS/FAIL or partial. Escalate if FAIL. **For architectural FAILs, update the plan in-place via Planner re-spawn before re-implementation** |
| **Final cycle re-audit** | After all sub-agent auditors return green, **I personally perform an end-to-end cycle re-audit**: re-read every diff, re-run full workspace tests + 4 CI guards, run phi-core-leverage-check + k8s-readiness-check skills myself, verify all paperwork (ADR Status, drift status, concept-doc bumps, K8s ledger, plan archive, audit logs). May re-trigger Implementer or Planner re-spawn if I find anything. Aligned with quality-over-completion. Output: cycle-audit doc that consolidates all iterations + my findings |
| Retrospective | Spawn Retrospector (opus) AFTER my final cycle audit returns clean. Review its draft, finalize, propose template/CLAUDE.md/agent-prompt updates. Get user sign-off on standards changes |
| Commit | User-driven (unchanged from current cadence) |

---

## §1 — Agent set (4 agents)

All agent files live at `/root/projects/phi/.claude/agents/`. Frontmatter: `name`, `description`, `model`, `tools`, `skills`, `version` (for retrospective-driven prompt evolution).

### A1. `chunk-planner` (opus)

**Owns.** §1–§12 of the per-chunk-template. Drafts the plan to the plan-mode plan file. Identifies forks and surfaces them in a "Forks for orchestrator" section near the top so I can ask the user. Generates the 8-hex archive token, creates the cycle folder `baby-phi/docs/specs/plan/build/<slug>-<8hex>/`, writes plan to `<that folder>/plan.md`.

**Frontmatter.**
```yaml
---
name: chunk-planner
description: Drafts the 12-section per-chunk plan from a forward-scope entry. Performs phi-core leverage analysis, K8s readiness eval, ADR draft, audit-envelope sizing. Surfaces locked forks for orchestrator review.
model: opus
tools: Read, Grep, Glob, Bash, Write
skills: chunk-template-fill, phi-core-leverage-check, k8s-readiness-check, audit-envelope-size, chunk-archive-plan
version: 1
---
```

**Tools rationale.** Read/Grep/Glob/Bash for codebase exploration. Write for the plan file ONLY (instruction-enforced — "the only file you may write is the plan file path the orchestrator passed you"). NO Edit (planner never modifies source). NO commit. Cannot ExitPlanMode (orchestrator-only).

**Quality bar embedded in prompt.**
- Every one of the 12 sections must be filled — none skipped.
- §3 phi-core leverage: positive AND forbidden greps explicit; import-count delta predicted.
- §3.B K8s readiness: 7-axis evaluation; new blocker classes flagged with proposed deferred-ledger entry.
- §3.C user-facing docs: 3-tier evaluation; defer decisions justified.
- §5 ADR: D-numbers (D47.1, D47.2, ...) used; ADR file path proposed.
- §11 audit plan: agent count picked via `audit-envelope-size` skill; both audits drafted with ≤ 600-word prompt each.
- §10 close criteria: implementation confidence target = `claims-honored / claims-in-scope` ≥ 9/10.
- Any fork the planner cannot decide from forward-scope + precedent gets a "Locked-fork (Q-N)" entry with 2–3 options + recommendation.

**Output handoff.** Planner writes to plan file + plan archive. Returns to orchestrator with a structured summary: chunk slug, archive path + token, list of forks needing user input (or "none — direct-approval candidate"), confidence estimate.

### A2. `chunk-implementer` (opus)

**Owns.** Phase-by-phase execution per the approved plan. Runs phase tests + clippy + fmt at each phase boundary. Closes chunk paperwork: drift-status flips, ADR Status flip, concept-doc header bumps, K8s deferred-ledger entries, migration test row-count bumps.

**Frontmatter.**
```yaml
---
name: chunk-implementer
description: Executes phases per an approved chunk plan. Runs tests, clippy, fmt at each phase boundary. Handles drift/ADR/concept-doc/K8s paperwork at chunk close. Patches per audit feedback when re-spawned.
model: opus
tools: Read, Edit, Write, Bash, Grep, Glob
skills: ci-guards-run, phi-core-leverage-check
version: 1
---
```

**Tools rationale.** Full read+write toolset. Bash for cargo + git status (NO `git commit` — orchestrator-only). NO ExitPlanMode (orchestrator-only).

**Quality bar embedded in prompt.**
- Run plan's per-phase pause-discipline checks; STOP and report if any pause condition fires.
- After each phase: run `cargo fmt --check`, `RUSTFLAGS="-Dwarnings" cargo clippy -j 4 --workspace --all-targets`, `cargo test -j 4 --workspace -- --test-threads=1`. Cap at `-j 4` per memory `feedback_cargo_jobs_cap.md`.
- All cargo invocations use `/root/rust-env/cargo/bin/cargo`.
- After all phases: run all 4 CI guards via `ci-guards-run` skill.
- Report test-count delta vs plan's expected count.
- Re-spawn behavior: when given an audit report, address every PASS/FAIL claim; do NOT touch out-of-scope code.
- NEVER commit. NEVER push.

**Output handoff.** Per-phase report (diff summary, test counts, clippy/fmt status). Final report with test count + 4-CI-guards status.

### A3. `chunk-auditor` (opus)

**Owns.** Independent post-implementation audits. Spawned 1× (small chunks, ≤ 2 phases), 2× (medium, 3–5 phases — Audit A code+phi-core+K8s, Audit B concept+docs+ADR), or 3× (large, 6+ phases — adds Audit C carry-forward regression). Read-only on source code; **writes** its audit log artifact to disk so reviews + retrospectives have a durable trail.

**Frontmatter.**
```yaml
---
name: chunk-auditor
description: Independent audit of a closed chunk. Verifies code correctness, phi-core leverage compliance, K8s readiness, concept-doc fidelity, ADR rigor, drift closure. Writes a per-iteration audit log; returns the path + summary.
model: opus
tools: Read, Grep, Glob, Bash, Write
skills: phi-core-leverage-check, k8s-readiness-check, ci-guards-run
version: 1
---
```

**Tools rationale.** Read for source code (read-only mandate enforced by prompt). Bash for `cargo test` / `cargo clippy` / grep / git diff. Write for the audit log file ONLY — never touches source. Cannot Edit, cannot commit.

**Quality bar embedded in prompt.**
- Verify each numbered claim from the audit prompt with a specific file path + line number citation.
- Re-run `cargo test --workspace -- --test-threads=1` and report exact passed/failed counts (must match plan's expected).
- Re-run all 4 CI guards.
- For each PASS, cite the verifying evidence (grep output, test output, file content). For each FAIL, cite the gap.
- Audit log structure: header (cycle ID, chunk slug, audit-letter A/B/C, iteration N, auditor model, date) → tabular PASS/FAIL summary → per-claim detail with citations → final verdict.
- Length cap on the *summary* line ≤ 600 words; full citations may exceed.
- Auditor MUST NOT propose fixes — only report findings.

**Output handoff.** Path to audit log file `baby-phi/docs/specs/plan/build/<slug>-<8hex>/audit-<letter>-iter<N>.md` + 5-line summary returned inline. Orchestrator reads the log, spot-checks 1–2 claims, decides PASS / FAIL / partial.

### A4. `chunk-retrospector` (opus)

**Owns.** Post-audit consolidated retrospective per cycle (one per chunk, not per audit iteration). Synthesizes from the **cycle-audit document** (which itself consolidates every iteration's audit log + my final cycle re-audit findings) plus the diffs/test-counts/timing data: (a) what went well/poorly across plan/implement/audit phases, (b) audit-cycle gap analysis (issues found, root cause, why audit caught it / why earlier phases missed it, what would have caught it earlier), (c) proposed updates to per-chunk-template, CLAUDE.md, agent prompts, skill checklists, (d) cross-cycle pattern observations (does this finding echo a prior cycle's? — read prior retros via grep).

**Frontmatter.**
```yaml
---
name: chunk-retrospector
description: Cycle-level consolidated retrospective. Synthesizes process learnings + audit-cycle gap analysis + proposed standards updates (template, CLAUDE.md, agent prompts, skill checklists). Reads cycle-audit doc + all iteration audit logs + diffs. Writes hex-tagged retrospective.
model: opus
tools: Read, Grep, Glob, Bash, Write
skills: (none)
version: 1
---
```

**Tools rationale.** Read for full context (cycle-audit doc + iteration audit logs + plan + diffs). Bash for `git log` / `git diff` to compute timing + scope deltas. Write for the retrospective file ONLY. NO Edit (retro doesn't modify other files; standards updates surface as proposals for orchestrator+user to apply separately).

**Quality bar embedded in prompt.**
- Retro must cover 6 sections: Cycle metadata + Outcomes + Audit-cycle gaps + Process changes proposed + Standards updates proposed + Cross-cycle patterns / open questions for next cycle.
- Audit-cycle gap section: every audit FAIL/partial **and** every issue found in my final cycle re-audit must trace to a root cause + a proposed gap-closing change. (This is the user's primary intent — "identifying process gaps that emerged during audit cycles and how to close those gaps".)
- Standards updates proposed: each one must cite the gap it closes + the template/CLAUDE.md/agent-prompt anchor it changes.
- Cross-cycle patterns: grep prior retros (`baby-phi/docs/specs/plan/build/*/retrospective.md` for new folder-style cycles, plus the legacy flat layout if surfaced) for similar root-cause keywords — if pattern repeats, escalate.
- File path: `baby-phi/docs/specs/plan/build/<slug>-<8hex>/retrospective.md` (sibling of the cycle's plan + audit logs).

**Output handoff.** Retrospective file path + summary. Orchestrator reviews, finalizes, gets user sign-off on standards changes before applying them.

---

## §2 — Skills (6 skills)

All skill files live at `/root/projects/phi/.claude/skills/`. Each is a `.md` with frontmatter (`name`, `description`) + body (procedure + commands + output format).

### S1. `phi-core-leverage-check`

**Used by:** planner (at draft time, predicting deltas), implementer (self-check during implementation), auditor (verifying §3 grep table at close).

**Procedure:**
1. Run positive greps from plan's §3 — confirm expected hit counts.
2. Run forbidden greps from plan's §3 — confirm zero hits.
3. Run `bash baby-phi/scripts/check-phi-core-reuse.sh`; expect exit 0.
4. Compute `git diff --stat` for `use phi_core::` import-count delta vs predicted.

**Output:** ✅/❌ per check + cited grep output.

### S2. `k8s-readiness-check`

**Used by:** planner (drafting §3.B), auditor (verifying §3.B at close).

**Procedure:**
1. Walk the 7 axes from per-chunk-template §3.B (in-process state, IPC, pod-local resources, migration runner, trait-shape requirement, cross-pod state sharing, audit hash-chain symmetry).
2. For each axis: classify as `no impact` / `compatible` / `new blocker class`.
3. If new blocker: propose deferred-ledger entry with `CHK8S-D-NN` next-free number (look up at `baby-phi/docs/specs/v0/implementation/m7b/architecture/deferred-from-ch-k8s-prep.md`).

**Output:** Filled §3.B table + ledger entry draft (if any).

### S3. `chunk-template-fill`

**Used by:** planner only.

**Procedure:**
1. Read the canonical template at `baby-phi/docs/specs/v0/implementation/m5_1/process/per-chunk-planning-template.md`.
2. Emit a fully-filled draft with all 12 sections, every required subsection present (no `(stub)` or `TODO` lines — every cell has content or an explicit "N/A — reason").

**Output:** Draft plan markdown (handed to planner agent for further refinement).

### S4. `ci-guards-run`

**Used by:** implementer (before declaring chunk complete), auditor (verifying close).

**Procedure:**
```bash
cd /root/projects/phi/baby-phi
bash scripts/check-doc-links.sh
bash scripts/check-ops-doc-headers.sh
bash scripts/check-phi-core-reuse.sh
bash scripts/check-spec-drift.sh
```

**Output:** ✅/❌ per script + offending output if any.

### S5. `chunk-archive-plan`

**Used by:** planner only (called at chunk-open Step 0).

**Procedure:**
1. Generate token: `openssl rand -hex 4`.
2. Create cycle folder: `mkdir -p baby-phi/docs/specs/plan/build/<slug>-<8hex>/` (slug-first naming, token suffix per memory `feedback_plan_archive_naming.md`).
3. Copy plan-mode plan file to `baby-phi/docs/specs/plan/build/<slug>-<8hex>/plan.md`.
4. Update placeholders in lines 4–5 of archived copy.
5. Run `bash scripts/check-doc-links.sh` to confirm relative-link integrity.

**Output:** cycle folder path + token.

### S6. `audit-envelope-size`

**Used by:** planner (drafting §11).

**Procedure:** Apply per-chunk-template §11 sizing rule:
- Small (≤ 2 phases) → 1 audit agent (combined Audit A+B prompt)
- Medium (3–5 phases) → 2 audit agents (Audit A code+phi-core+K8s; Audit B concept+docs+ADR)
- Large (6+ phases) → 3 audit agents (adds Audit C carry-forward regression suite)

**Output:** Number + audit prompt scaffolds for §11.

---

## §3 — Workflow (per cycle)

```
[Orchestrator] → pick chunk from forward-scope → confirm prereqs
       ↓
[Planner] (opus) → openssl rand -hex 4 → cycle hex assigned
                 → mkdir plan/build/<slug>-<8hex>/
                 → draft plan to plan/build/<slug>-<8hex>/plan.md
                 → identify forks → return summary
       ↓
[Orchestrator] → apply Direct-approval criteria
       ├─ pass → ExitPlanMode myself with planner's plan
       └─ escalate → AskUserQuestion for forks → ExitPlanMode
       ↓
[Implementer] (opus) → execute Phase 1 → run tests + clippy + fmt → report
       ↓
[Orchestrator] → review diff + verify tests + spot-check phi-core grep → mark phase complete
       ↓ (loop until all phases done)
[Implementer] (opus) → final phase: paperwork (drifts/ADR/concept-doc/K8s/migration tests)
       ↓
[Orchestrator] → run ci-guards-run skill myself → verify
       ↓
[Auditor × N] (opus) → spawn per audit-envelope size
                     → each writes plan/build/<slug>-<8hex>/audit-<letter>-iter1.md
                     → return paths + 5-line summaries
       ↓
[Orchestrator] → read each audit log + spot-check claims
       ├─ all PASS → proceed to FINAL CYCLE RE-AUDIT
       ├─ tactical FAIL → re-spawn Implementer with audit log → re-spawn auditors (iter2)
       ├─ architectural FAIL → re-spawn Planner with audit log → user gate (always)
       │                      → re-spawn Implementer → re-spawn auditors (iter2)
       └─ trivial FAIL (1-line) → I patch myself → re-spawn auditors (iter2)
       ↓
       (loop until all sub-agent audits PASS — every iteration adds new audit-log files)
       ↓
[Orchestrator: FINAL CYCLE RE-AUDIT] (me) →
       re-read every diff in cycle, re-run cargo test + clippy + 4 CI guards,
       run phi-core-leverage-check + k8s-readiness-check skills myself,
       verify all paperwork + plan archive + every audit log exists,
       write plan/build/<slug>-<8hex>/cycle-audit.md consolidating
       (a) per-iteration auditor findings, (b) my findings, (c) verdict
       ├─ clean → proceed to retrospective
       ├─ findings → escalate as Architectural / Tactical / Trivial → loop back into audit cycle
       │             (this is the user's "quality and thoroughness over cycle completion" gate)
       ↓
[Retrospector] (opus) → read cycle-audit doc + all iteration audit logs + plan + diffs
                     → write plan/build/<slug>-<8hex>/retrospective.md
                     → return path + summary
       ↓
[Orchestrator] → review retrospective → propose standards updates to user
                → apply approved updates (versions bumped on agent/skill files)
       ↓
[User] → commit (unchanged from current cadence)
```

**Iteration cap:** if any audit cycle reaches **iter 3** without all PASS, I stop and escalate to user. Repeated failure on the same finding signals a structural issue the agent system can't resolve.

---

## §4 — Direct-approval criteria (when I skip user gate at plan time)

I auto-approve a plan (call ExitPlanMode myself with planner's plan) when **all** of the following hold:

1. **No locked forks.** Planner's "Forks for orchestrator" section is empty or all forks have a single defensible answer cited from forward-scope/precedent (no genuine multi-option choices).
2. **Scope matches forward-scope.** Plan's §1 effort estimate is within 1.5× of the forward-scope row's estimate. Wider scope → escalate.
3. **No phi-core leverage delta.** §3 predicts zero new `use phi_core::` imports added or removed. Any delta → escalate (changes the leverage map).
4. **No new K8s blocker class.** §3.B reports no new entry needed in the deferred-ledger. New entry → escalate.
5. **Audit envelope ≤ medium (≤ 2 auditors).** Large chunks (3 auditors, 6+ phases) always escalate — too much surface area to auto-trust.
6. **Confidence target ≥ 9/10.** Plan's §10 implementation confidence target meets the bar.
7. **No new migration.** Migrations always escalate (schema changes are high-blast-radius). Plan with `0NNN_*.surql` → escalate.

If any one condition fails, I escalate to user via AskUserQuestion (for the locked forks) + ExitPlanMode (for plan approval). User can also pre-instruct me to escalate every plan during a "high-care" period.

**Conservative default for the first 3 chunks under this system:** I escalate every plan to the user regardless of criteria, until we trust the planner's calibration. After 3 successful auto-approvable plans, I shift to criteria-based gating. (User can override.)

---

## §5 — Audit-fix loop (3 paths + final cycle re-audit)

Findings can come from two sources: (a) sub-agent auditors during normal audits, (b) my final cycle re-audit. Either source classifies findings as Tactical / Architectural / Trivial and triggers the same paths.

| Path | Trigger | Action |
|---|---|---|
| **Tactical** | Implementation correctness gap (missing test, wrong field, missed grep, paperwork omission) | Re-spawn Implementer with the audit log path verbatim as input. Implementer addresses each FAIL claim; reports diff. I review + re-spawn auditors for delta iteration (iter N+1, written to a new hex-tagged file). |
| **Architectural** | Plan-level correctness gap (wrong ADR sub-decision, wrong K8s axis, wrong concept-doc anchor, missed forward-scope row) | Re-spawn Planner with the audit log path. Planner amends plan + archive (same hex; plan file overwritten — git history preserves prior version). **Always escalate to user** at this point — plan-level redos are not auto-approvable. After user approves amended plan → re-spawn Implementer for delta-implementation → re-spawn auditors (iter N+1). |
| **Trivial** | 1-line, unambiguous fix (e.g., missing header-bump line, typo in ADR Status) | I patch myself + re-spawn auditors (iter N+1). |

**Cycle-audit document.** After all sub-agent audits PASS, I write the consolidated `baby-phi/docs/specs/plan/build/<slug>-<8hex>/cycle-audit.md`. Structure:
- Cycle metadata (chunk slug, hex, link to sibling `./plan.md`, total iterations).
- Per-iteration subsection: links to each sibling `./audit-<letter>-iterN.md` file (e.g., `./audit-A-iter1.md`, `./audit-B-iter1.md`, `./audit-A-iter2.md`) + verdict + any findings + how addressed.
- **My final orchestrator audit subsection**: my checks (re-run tests + 4 CI guards + phi-core/k8s skills + paperwork verification), findings, verdict.
- Cycle verdict: GREEN (proceed to retrospective) or AMBER/RED (loop back).

**Iteration cap:** if any audit cycle reaches **iter 3** without all PASS, I stop the agent loop and ask user to intervene. Repeated failure on the same finding signals a structural issue the agent system can't resolve.

**Retrospective input:** every audit FAIL across every iteration AND every finding from my final cycle re-audit must be logged in the retrospective's "Audit-cycle gaps" section with root cause + proposed gap-closing change. This is how process gaps get systematically closed.

---

## §6 — Quality enforcement (how the bar holds or rises)

This is the user's primary concern. Five layers of defense:

1. **Embedded quality bar in agent prompts.** Each agent's prompt includes its specific quality criteria (planner: 12 sections complete + confidence ≥ 9/10; implementer: clippy clean under -Dwarnings, test count matches plan; auditor: every claim cited with file:line, ≤ 600 words; retrospector: 5 sections + audit-cycle gap analysis).
2. **Skill-enforced procedures.** `phi-core-leverage-check`, `k8s-readiness-check`, `ci-guards-run` codify the procedures so no agent can skip a check by oversight.
3. **My personal verification of every output.** Per memory `feedback_agent_verification.md` — I read every diff, run spot-checks, never trust agent summaries. This is the strongest gate.
4. **Independent auditor agents.** Read-only opus auditors who didn't write the code do the close-audit. Same standard as today.
5. **Retrospective-driven prompt evolution.** Each chunk's retro proposes prompt/skill updates. Standards drift improves over time, doesn't degrade.

**Cross-checking matrix.**
| Output | Verified by |
|---|---|
| Planner's plan | Me (read + Direct-approval criteria) → optionally user |
| Implementer's per-phase diff | Me (read diff + run tests + spot-check grep) |
| Implementer's chunk-close paperwork | Me + Auditor B (concept-doc fidelity) |
| Each iteration's audit log | Me (spot-check 1–2 claims by reading cited file:line) |
| **Cycle as a whole, post-audit** | **Me** (final cycle re-audit — independent run of tests + CI guards + skills + paperwork verification) |
| Retrospector's retro | Me (review) → user (approves standards updates) |

**Why the final cycle re-audit matters.** Sub-agent auditors verify their assigned scope (Audit A code, Audit B docs/concept). They don't see the **whole picture**. The final-by-me audit is the only gate that re-runs the entire workspace + every CI guard + every paperwork check + every skill, against the full diff, knowing the conversation history. This catches: (a) issues that fall between audit-A's and audit-B's seams, (b) regression in carry-forward chunks the auditors didn't reread, (c) my conversation-context knowledge of what the user actually wanted that no static prompt captures. **This is the user's primary quality safeguard** — it cannot be skipped.

---

## §7 — Versioning + retrospective integration

- Each agent file has `version: N` in frontmatter. Bumped when retrospective-driven changes land.
- Each skill file has `version: N` in frontmatter. Bumped same way.
- All cycle artifacts share one **8-hex cycle ID** generated at plan-open via `openssl rand -hex 4`. **Each cycle gets its own folder** at `baby-phi/docs/specs/plan/build/<slug>-<8hex>/`. Files inside the folder:
  - `plan.md` — cycle plan
  - `audit-A-iter1.md` — Audit A, iteration 1 (and `audit-B-iter1.md`, `audit-C-iter1.md` as envelope grows)
  - `audit-A-iter2.md` — only if iter 2 happens
  - `cycle-audit.md` — consolidated by me (orchestrator final cycle re-audit)
  - `retrospective.md` — by Retrospector
- Folder naming carries both slug + hex so directory listing remains chunk-recognizable. Inner filenames stay short + uniform across cycles, simplifying grep + tooling.
- Pointer-index at `baby-phi/docs/specs/plan/build/_cycle-index.md` indexes every cycle folder with chunk slug, hex, status (in-flight / closed), iteration count, retro link.
- **Migration policy for existing flat files**: pre-existing chunks (CH-09, CH-10, CH-23, etc.) keep their flat `<slug>-<8hex>.md` filenames in `plan/build/` — historical artifacts stay where they are, do not churn. The folder convention applies only to new cycles started under this multi-agent system.
- Agent/skill changelog at `/root/projects/phi/.claude/agents/_changelog.md` records every agent-prompt or skill change (date, version bump, source retrospective hex, what changed, why).

---

## §8 — Critical files

**Step 0 of execution (before any agent files are written):**
- Generate cycle hex via `openssl rand -hex 4`.
- Copy this plan from `/root/.claude/plans/sharded-discovering-stearns.md` to `baby-phi/docs/specs/agentic-workflow/multi-agent-chunk-pipeline-<8hex>.md` per the user's archive instruction. Update placeholder lines if any. This is the durable archive of the meta-plan; future retros and design changes reference back to it.

**New (created during this landing):**
- `baby-phi/docs/specs/agentic-workflow/multi-agent-chunk-pipeline-<8hex>.md` — archived meta-plan (Step 0).
- `baby-phi/docs/specs/agentic-workflow/_index.md` — index of agentic-workflow design docs (this plan + future iterations).
- `/root/projects/phi/.claude/agents/chunk-planner.md`
- `/root/projects/phi/.claude/agents/chunk-implementer.md`
- `/root/projects/phi/.claude/agents/chunk-auditor.md`
- `/root/projects/phi/.claude/agents/chunk-retrospector.md`
- `/root/projects/phi/.claude/agents/_changelog.md`
- `/root/projects/phi/.claude/skills/phi-core-leverage-check.md`
- `/root/projects/phi/.claude/skills/k8s-readiness-check.md`
- `/root/projects/phi/.claude/skills/chunk-template-fill.md`
- `/root/projects/phi/.claude/skills/ci-guards-run.md`
- `/root/projects/phi/.claude/skills/chunk-archive-plan.md`
- `/root/projects/phi/.claude/skills/audit-envelope-size.md`
- `baby-phi/docs/specs/plan/build/_cycle-index.md` — pointer index for hex-tagged cycle artifacts.

**Modified:**
- `/root/projects/phi/CLAUDE.md` — append a "Multi-agent chunk pipeline" section pointing to `.claude/agents/`, `.claude/skills/`, and `baby-phi/docs/specs/agentic-workflow/`, documenting the orchestrator's role + Direct-approval criteria + audit-fix loop + final-cycle-re-audit gate.
- `baby-phi/CLAUDE.md` — same section appended (mirror at submodule level so baby-phi-only sessions surface the convention).

**Cycle artifacts created at runtime per chunk** (not part of this landing — listed for reference; created when the system is exercised on CH-11+):
- Folder: `baby-phi/docs/specs/plan/build/<slug>-<8hex>/`
  - `plan.md`
  - `audit-A-iter<N>.md` (× audit envelope size × iteration count, e.g., `audit-A-iter1.md`, `audit-B-iter1.md`, `audit-A-iter2.md`)
  - `cycle-audit.md`
  - `retrospective.md`

**Unchanged (verified):**
- `baby-phi/docs/specs/v0/implementation/m5_1/process/per-chunk-planning-template.md` — template stays canonical; agents reference it but don't change it. Retrospectives propose template changes; user approves separately.
- All baby-phi source code, tests, ADRs, drifts, concept docs.

---

## §9 — Verification recipe

After landing the agent set + skills, validate by running it on the next real chunk (likely **CH-11 — Per-Session consent gating**, the natural successor to CH-10).

**Dry-run validation (no chunk):**
1. Read each agent file end-to-end. Confirm frontmatter + quality bar + tools list match the spec above.
2. Read each skill file end-to-end. Confirm procedure + commands match.
3. Run `bash /root/projects/phi/baby-phi/scripts/check-doc-links.sh` to verify the new files don't break link checking.
4. `ls /root/projects/phi/.claude/agents/` → 5 files (4 agents + changelog). `ls /root/projects/phi/.claude/skills/` → 6 files.
5. Verify this plan archive exists at `baby-phi/docs/specs/agentic-workflow/multi-agent-chunk-pipeline-<8hex>.md` with correct hex; `_index.md` references it.

**Live-run validation (CH-11):**
1. Spawn `chunk-planner` agent for CH-11 with forward-scope row + prereq context.
2. Read the returned plan against the per-chunk-template §1–§12.
3. Apply Direct-approval criteria. (For CH-11 specifically: it has scope + new ADR + concept-doc bumps → escalate to user, conservative default.)
4. Through the chunk: spawn implementer phases, run audits, **perform final cycle re-audit myself**, build retro.
5. After CH-11 closes: review the retrospective. Apply approved standards updates. Update agent versions.

**Quality-floor checks (per cycle under the new system):**
- Test count delta exactly matches plan's expected delta.
- All 4 CI guards green at chunk close + at my final cycle re-audit.
- All sub-agent audits PASS (any re-spawn root-caused in retro's audit-cycle gaps section).
- My final cycle re-audit returns clean.
- All cycle artifacts present with correct hex: plan, ≥ 1 audit log per audit letter, cycle-audit, retrospective.
- Retrospective covers all 6 mandatory sections + at least 1 proposed standards update (or explicit "no changes proposed — process worked clean").

**Failure-mode signals (trigger retro deep-dive + possible plan-pause):**
- Audit iteration count ≥ 3 (cap reached → user escalation).
- Implementation phase requires > 2 patch iterations.
- Planner returns a plan that fails Direct-approval criteria but I auto-approved anyway.
- Retro proposes > 3 standards changes in a single cycle (system instability signal).
- My final cycle re-audit finds an issue that all sub-agent auditors missed → high-priority retro entry, mandatory gap-closing change.

---

## §10 — What this plan does NOT do

- Does NOT change the per-chunk-template content. Standards updates land via separate user-approved tasks, surfaced from retrospectives.
- Does NOT auto-pick chunks from forward-scope. User-driven; orchestrator confirms prereqs.
- Does NOT auto-commit. User-driven, unchanged.
- Does NOT introduce new agent types beyond the 4 specified. If a new type is needed, that's a retrospective-driven proposal (not part of this landing).
- Does NOT enforce skill-restriction in agent frontmatter — Claude Code doesn't enforce `skills:` as access control. The field documents intent; agents are expected to call those skills first when applicable. Reviewer-signal, not access control.
- Does NOT extend to phi-core-specific chunks (phi-core has its own pipeline; this is baby-phi-shaped). When phi-core needs a similar pipeline, separate retrospective-driven design.
- ~~Does NOT extend to **milestone-to-chunks decomposition**~~ — **SHIPPED at 2026-05-18 per CH-27 retro's M6 plan-mode unblock**. The 5th agent `phase-planner` (opus, v1; ~190 lines at `/root/projects/phi/.claude/agents/phase-planner.md`) consumes a milestone narrative (from base build plan) + a pre-scoping alignment audit + prior-milestone deferral markers (M*-DEFERRED-NN from prior forward-scope §3) and emits the per-milestone forward-scope document (chunk-level decomposition + dep graph + open questions). Its output is the forward-scope file that `chunk-planner` later consumes (forward-scope row → chunk plan). Plan archive: `baby-phi/docs/specs/plan/review-and-docs/m6-forward-scoping-and-rename-cleanup-af8aed16.md`. Initial use-case: post-M5.3 close M6 forward-scope authorship (Phase 3 of the M6 forward-scoping plan). The 4-agent set is now a 5-agent set; this is the first extension since the original 2026-04-NN landing; future extensions (phi-core pipeline analog; cross-submodule co-orchestration; etc.) remain retrospective-driven proposals.

---

## §11 — Estimated effort

~1.2 engineer-day for the landing:
- 0.05d — Step 0: archive this plan to `baby-phi/docs/specs/agentic-workflow/multi-agent-chunk-pipeline-<8hex>.md` + create `_index.md`.
- 0.4d — write the 4 agent files + 6 skill files (drafted from this plan; concrete prompt language, examples, embedded quality bars).
- 0.25d — write CLAUDE.md addenda (repo + baby-phi mirror) + `_cycle-index.md` + agent changelog scaffolding.
- 0.1d — dry-run validation (read every file, run doc-links check, sanity).
- 0.4d — orchestrator dry-run on a fake chunk to test handoff plumbing (no code changes, verify each agent prompt produces useful output, exercise the final-cycle-re-audit gate).

After landing, the **first live chunk under the new system** (likely CH-11) will be slower than baseline (~1.4× normal) due to handoff plumbing AND the new final-cycle-re-audit gate. By cycle 3 under the system, we expect parity with current cadence + meaningful retrospective-driven improvements. Surface-area improvements compound: better template, better agent prompts, less re-work, fewer audit re-spawns.

The final-cycle-re-audit gate intentionally adds ~0.2d per cycle. This is a tax we pay for thoroughness — per the user's locked principle "quality and thoroughness over cycle completion".
