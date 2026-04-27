<!-- Last verified: 2026-04-24 by Claude Code -->

# Per-chunk planning template

Every implementation chunk opened after M5.1 ships its own standalone plan file before any code moves. This document is the canonical template. Drafters copy it verbatim into `baby-phi/docs/specs/plan/build/<8hex>-<chunk-name>.md` and fill every section. Incomplete templates do not qualify for `ExitPlanMode` approval.

The template bakes in the M5.1/P3 Q1–Q7 planning decisions (see [`forward-scope/22035b2a-remaining-scope-post-m5-p7.md`](../../../../plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md) §7). Chunk-plan authors do not re-litigate those decisions; they apply them.

## File location & naming

- **Path**: `baby-phi/docs/specs/plan/build/<8hex>-<chunk-name>.md`
- **8hex token**: generate via `openssl rand -hex 4` at draft time (same convention as forward-scope file name).
- **chunk-name**: lowercase kebab-case; derived from the forward-scope `CH-NN` chunk name. Example: `CH-02 Real agent_loop wiring` → `<8hex>-ch-02-real-agent-loop-wiring.md`.
- **Header line 1**: `<!-- Last verified: YYYY-MM-DD by Claude Code -->` (the `check-doc-links.sh` guard enforces this).

## Uniform application (Q7 decision)

**Every chunk uses this template — including doc-only chunks** (CH-19 & CH-20 in the forward-scope inventory). Doc-only chunks approve faster because there is no code to review, but the planning + close ritual is identical.

## Mandatory sections (12)

Each numbered section is mandatory. Sections may be lengthy or short depending on chunk scope but may not be omitted. A section whose content is legitimately "N/A" must say so with one-line justification.

---

### §1 — Context & principle

- **Why this chunk** — one paragraph. What user-visible or concept-fidelity gap does this chunk close? Which drift IDs make it necessary?
- **Quality-over-speed restatement** — restate the M5.1 governing principle: *"Concept docs are source-of-truth; implementation aligns to them. Drift is discovered, documented, and planned-through — never accumulated silently."* One-sentence chunk-specific application.
- **Forward-scope reference** — explicit link to the `CH-NN` row in [`forward-scope/22035b2a-remaining-scope-post-m5-p7.md`](../../../../plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md) §5.

### §2 — Concept alignment walk

Full table of every concept-doc claim the chunk touches:

| Concept doc | § anchor | Claim (verbatim or close paraphrase) | Status at chunk-open | Target status at chunk-close |
|---|---|---|---|---|
| `concepts/<doc>.md` | `§<anchor>` | `"..."` | honored / contradicted / partially-honored / silent-in-code / concept-aspirational | honored / out-of-scope-for-chunk |

Rules:
- **Coverage** — every concept doc whose claims the chunk's code will touch appears in the table. No "we'll find out at implementation time."
- **Permissions subtree hook** — if any of `permissions/01`–`permissions/09` docs are touched, `permissions/README.md` MUST be cited as the entry invariants source.
- **phi-core-mapping hook** — if any surface overlaps with phi-core types, `concepts/phi-core-mapping.md` MUST appear in the table with the relevant row cited.

### §3 — phi-core leverage map

Full table of every phi-core type the chunk may overlap with:

| phi-core type | Current handling in baby-phi | Classification | Action in chunk |
|---|---|---|---|
| `phi_core::X::Y` | not used / imported / wrapped / duplicated | direct-reuse / wrap / inherit / reject | import it / wrap it / keep orthogonal |

Additional mandatory sub-fields:
- **Expected import-count delta at chunk close** — numeric prediction. Example: *"+3 phi-core imports across `domain/src/agents/model.rs` and `server/src/platform/agents/create.rs`."*
- **Positive close-audit greps** — the exact commands the post-chunk audit will run. Example: *`grep -rn "phi_core::agents::profile::AgentProfile" modules/crates/ | wc -l` — expect ≥ 3.*
- **Forbidden-duplication greps** — exact commands that must return 0 hits. Example: *`grep -rn "^struct AgentProfile" modules/crates/ | grep -v "phi_core::"` — expect 0.*

Per [`baby-phi/CLAUDE.md`](../../../../../../CLAUDE.md) §"phi-core Leverage" rules 1–5. `scripts/check-phi-core-reuse.sh` MUST stay green at chunk close.

### §4 — Drifts closed

List every drift file in [`../drifts/`](../drifts/) this chunk transitions to `remediated` / `renegotiated` / `accepted-as-is`:

| Drift ID | File | Severity | Transition | Notes |
|---|---|---|---|---|
| `D-new-NN` | `../drifts/D-new-NN.md` | HIGH / MEDIUM / LOW | in-chunk-plan → remediated | (if renegotiated, link the ADR) |

Rules:
- Every drift in the forward-scope inventory's `CH-NN` chunk MUST appear here.
- If the chunk discovers new drifts mid-flight (see §6 *mid-flight pause*), the new drifts are added to this table before chunk seal.
- Drift status transitions happen at chunk seal, not earlier. The lifecycle rules in [`drift-lifecycle.md`](./drift-lifecycle.md) govern permitted transitions.

### §5 — ADRs drafted

Each chunk that makes a non-trivial architectural or convention decision MUST draft an ADR:

- **ADR number assignment** (Q6 decision): At chunk-plan drafting time, run `ls baby-phi/docs/specs/v0/implementation/*/decisions/*.md 2>/dev/null | xargs -I{} basename {} .md | grep -oE "ADR-[0-9]+" | sort -u | tail -5` to see the current highest ADR. Pick the next free sequential number. Record it in the chunk plan. Never allocate ADR numbers opportunistically mid-chunk.
- **Draft status at plan draft** — `Proposed`.
- **Flip to `Accepted`** — at the phase close that ships the decision, or at chunk seal if the ADR covers the chunk holistically.

For each ADR list: number, title, drafted-at-phase, decision-summary (one line), expected flip-to-Accepted phase.

### §6 — Prior-chunk regression re-verification

List every upstream chunk whose invariants this chunk depends on. For each, state the re-verification recipe:

| Upstream chunk | Invariant this chunk relies on | Re-verification command |
|---|---|---|
| CH-NN | `concepts/<doc>.md §<anchor>` still honored | `grep -rn "..." modules/` or named test name |

This table runs AT CHUNK OPEN before any phase opens, and again at chunk seal. Any regression produces a new drift file + surfaces as an open question for user before the chunk proceeds.

### §7 — Phases within the chunk

Each phase documented as:

- **Goal** — one paragraph.
- **Deliverables** — numbered list. File paths and key changes.
- **Tests** — new tests added, existing tests expected to still pass.
- **Concept-alignment check** — which §2 table rows this phase transitions status on; how verified.
- **phi-core leverage check** — which §3 table rows this phase transitions action on; how verified.
- **Confidence target** — ≥ X% composite (defaults: P0/scaffolding 100%, content phases ≥ 97%, seal ≥ 99%).
- **Pause discipline** — any known-upfront fork point where the phase MUST halt for `AskUserQuestion` before continuing.

### §8 — Tests summary

- **Expected total test count at chunk close** — concrete number (e.g., "973 from current 966 baseline + 7 new tests").
- **Layer breakdown** — unit / integration / acceptance / e2e counts.
- **Named test files** — list the new test file paths.
- **Named expected-still-green tests** — anything fragile that the chunk's changes risk breaking; re-verified at chunk close.

### §9 — Pre-chunk gate

The reading list + invariant check the drafter walks BEFORE `ExitPlanMode` is invoked.

**Reading list (mandatory):**
1. Every concept doc cited in §2.
2. Every drift file cited in §4.
3. Every prior-chunk plan cited in §6 (under `docs/specs/plan/build/`).
4. [`forward-scope/22035b2a-remaining-scope-post-m5-p7.md`](../../../../plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md) §5 + §7 (the chunk row + binding Q&A decisions).
5. [`baby-phi/CLAUDE.md`](../../../../../../CLAUDE.md) phi-core Leverage section.

**Carry-forward invariants** (explicit list, verified green at chunk open):
- `cargo test --workspace` test count matches the expected baseline (currently 966; update as chunks land).
- `scripts/check-phi-core-reuse.sh` green.
- `scripts/check-doc-links.sh` green.
- `scripts/check-ops-doc-headers.sh` green.
- `modules/` diff against the chunk-open git HEAD is empty (no preload edits).

**Pending decisions carried into this chunk:**
- List any forward-scope §7 Q&A that this chunk operationalises.
- List any drift-file `discovered → classified → scoped` transitions owed before the chunk can close.

**Chunk-ordering note (Q4 decision):** The user selects which chunk opens next at each chunk-open, using the forward-scope dependency graph as reference. No pre-committed sequence exists. Chunk plans do not assume a predecessor chunk's completion unless that predecessor is explicitly listed in §6 as a re-verified upstream.

### §10 — Close criteria

Composite 4-aspect + 2 confidence % ritual. **Source of truth: concept docs.** No rounding; below-target blocks close.

**4 aspects (each graded pass / fail):**
- **Code aspect** — all phases' deliverables shipped; cargo test workspace passes; clippy green under `RUSTFLAGS="-Dwarnings"`; fmt --check green.
- **Docs aspect** — every affected doc updated (status tags, verified headers, concept-audit matrix rows, drift-file lifecycle entries).
- **phi-core leverage aspect** — §3 import-count delta matches prediction ± documented variance; all forbidden-duplication greps return 0; `check-phi-core-reuse.sh` green.
- **Concept alignment aspect** — every §2 row's target-status at chunk-close achieved; none remain `contradicted`.

**2 confidence % (each with named numerator/denominator):**
- **Implementation confidence %** = `(claims-verified-honored-by-tests-and-code-inspection) / (total-claims-in-scope-for-chunk)`. Example: *"14/15 claims honored = 93%."* The 1 remaining claim gets its own named drift file + explicit re-scope.
- **Documentation confidence %** = `(doc-pages-where-independent-reader-can-cross-check-against-code-+-concept-+-ADRs-without-ambiguity) / (doc-pages-touched-in-chunk)`. Example: *"8/8 = 100%."*

**Composite = min(impl%, doc%, code-aspect-binary, phi-core-leverage-aspect-binary, concept-alignment-aspect-binary).** A failing aspect is 0%. Composite below target blocks close.

**Explicit close-target discipline:** close report states ALL FIVE measures with named numerators/denominators. No aspect-averaging. No rounding up.

### §11 — Post-chunk independent audit plan

Drafted BEFORE implementation starts so audit scope is fixed.

**Agent count** (per guardrail 7 in M5.1 plan):
- 1 agent for small chunks (≤ 3 phases).
- 2 agents for medium chunks (4–6 phases).
- 3 agents for large chunks (7+ phases).

**Audit aspects (a–d):**
- (a) Code correctness.
- (b) Docs fidelity vs concept docs.
- (c) Concept alignment across every concept doc the chunk touched.
- (d) phi-core leverage (imports, no forbidden duplications, compile-time coercion witnesses intact).

**Audit agent prompts drafted here:**
- Each agent receives a scoped prompt naming the files it audits, the greps to run, the pass criteria, and the expected report format.
- Auditor MUST NOT be the same agent/implementer that did the work. Spawn fresh `Explore` or `general-purpose` subagents.

**Audit pass criteria:**
- Any new drift discovered by the audit → its own drift file created BEFORE chunk seals.
- Any audit-flagged concept contradiction → either fixed in-chunk, renegotiated with user approval, or converted to a drift file with explicit future-chunk assignment.
- Chunk seal is blocked until audit returns clean on all 4 aspects + all audit-discovered drifts are explicitly scoped.

### §12 — Verification section (end-to-end recipe)

Concrete commands a reviewer can run to replay the chunk's close verification.

```bash
cd /root/projects/phi/baby-phi

# 1. CI guards
bash scripts/check-doc-links.sh
bash scripts/check-ops-doc-headers.sh
bash scripts/check-phi-core-reuse.sh
bash scripts/check-spec-drift.sh

# 2. Workspace health
/root/rust-env/cargo/bin/cargo fmt --all -- --check
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy --workspace --all-targets
/root/rust-env/cargo/bin/cargo test --workspace

# 3. Chunk-specific
# <chunk-plan author inserts named tests + greps from §7 + §11>

# 4. Drift-file status
grep -l "Status.*remediated" docs/specs/v0/implementation/m5_1/drifts/D*.md | wc -l
# Expect: <previous count> + <§4 drift count>
```

---

## Chunk-open gate ritual

1. Drafter completes all 12 sections.
2. Drafter verifies §9 reading list is fully read + invariants green.
3. Drafter invokes `ExitPlanMode` with the plan file path.
4. User reviews + approves (or rejects with corrections).
5. Only after user approval does code begin.

**No exceptions — even doc-only chunks (Q7).**

## Mid-flight pause rules

Any mid-chunk discovery above convention-level triggers `AskUserQuestion` before continuing:
- New concept-contradiction not anticipated in §2.
- New phi-core type overlap not in §3.
- New drift surfaced by test or audit.
- Scope change that would breach a §10 confidence target.

"Document as drift at close" is explicitly retired for mid-flight discoveries — they are surfaced immediately.

## Close-time discipline

- No chunk closes with unresolved concept-alignment contradictions, phi-core leverage violations, or open audit findings.
- Every contradiction/finding is one of: fixed in-chunk, renegotiated with user approval (ADR), or converted into a new drift file with explicit future-chunk assignment.
- The 4-aspect + 2% composite is pinned in the close report. Composite below target = close blocked. No rounding.

## M5-scope defer rules (Q5 decision)

The forward-scope inventory lists chunks by severity. Chunks operate under these rules:

- **HIGH-severity chunks** — all 17 HIGH drifts MUST close before M5 tag ships.
- **MEDIUM-severity chunks** — evaluated case-by-case at chunk-open. User decides at that moment whether to close at M5 or defer to M6. Defer decisions recorded as drift-file status transitions (`scoped → renegotiated` with link to the future-chunk marker).
- **LOW-severity chunks** — all close in M5 via CH-19 / CH-20 (pure-doc chunks).

## Relationship to other process docs

- [`chunk-lifecycle-checklist.md`](./chunk-lifecycle-checklist.md) — step-by-step execution of this template.
- [`drift-lifecycle.md`](./drift-lifecycle.md) — the status transitions this template's §4 and §10 trigger on drift files.
