<!-- Last verified: 2026-04-27 by Claude Code -->
<!-- Post-CH-22 addition (2026-04-27): §3.C "User-facing documentation impact map" is now mandatory; §10 "Docs aspect" extended to cover the user-facing doc tier (architecture / operations / user-guide). Reason: the milestone-era pattern shipped three peer doc trees per milestone, but chunks were silently dropping that tier — operators got stale docs. Pre-CH-22 chunks (CH-01, CH-02, CH-K8S-PREP, CH-22) are grandfathered with backfill bundled in this codification commit. -->

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

#### §3 cascade-artifact discipline (CH-13 retro Row 2; chunk-planner v4)

For every load-bearing struct/enum/function the chunk touches with cascade impact (signature change rippling through callsites, field addition rippling through construction sites, additive enum variant rippling through `match` sites), §3 MUST paste **3 artifacts**:

1. **(a) The exact `git grep -nE` invocation** the planner ran.
2. **(b) The raw match count** at plan-draft time.
3. **(c) Per-file breakdown** of where the matches live + the predicted edit-site count for the chunk.

**Caveats** (CH-07 retro §5 rows 2 + 4; chunk-planner v5 effective 2026-05-07):

- **Per-file edit-count predictions are approximate; the aggregate band (lower–upper) is the load-bearing prediction.** Pause threshold (1.5× upper bound) is enforced on the aggregate, not per-file. CH-07 §3 Artifact D predicted 8–14 sites across ~6 files — actual was 14 across 7 files (one predicted file had 0 sites; another predicted file didn't exist; a non-predicted file carried 4 sites). The aggregate band held; per-file precision did not. Avoid promising specific files unless the planner ran a `git grep -lnE` against current HEAD and verified each file's match count.
- **When a concept-doc semantic could land at multiple pipeline steps, §3 MUST explicitly state which step + cite the rationale.** CH-07 §3 Artifact B is the exemplar: concept doc 06 line 162 ("base_org does not reach into non-member-scope sessions") could plausibly land at `step_2a_ceiling` (clamp time) OR `step_5_scope_resolution` (cascade time). The plan called out the placement explicitly + cited why (Step-2a placement preserves M1 back-compat for empty `session_org_tags` callers + applies uniformly across single-scope and multi-scope paths). Future cycles touching multi-step pipelines should follow the same discipline.

**Caveats** (CH-08 retro §5 rows 2 + 3; chunk-planner v6 effective 2026-05-08):

- **Each cascade-band citation MUST also name the canonical grep-pattern** (e.g., `git grep -nE 'field_name: None' modules/crates/ | wc -l`) so planner / implementer / auditor / orchestrator converge on one metric. CH-08 saw three different counts across lanes for the `Grant.allocate_refinement: None` cascade — implementer reported 27 sites, orchestrator gate-4 grep returned 30, Audit A returned 33 — all within tolerance (≤ 38 threshold) but a measurement-drift signal. Naming the canonical grep prevents the divergence and keeps audits citation-clean.
- **Before recommending `..Default::default()` cascade strategy, verify ALL sub-field types in the affected struct already derive `Default` — not just the top-level struct.** CH-08 caught this at P1: `Grant` could derive `Default`, but `PrincipalRef` + `ResourceRef` don't, and adding `Default` to them would invent semantically-inappropriate placeholders (e.g., empty-URI ResourceRef). Plan §3 should explicitly verify sub-field Default-derive support before recommending the strategy; otherwise the implementer falls back to per-callsite explicit-field cascade (strategy b), which is valid but heavier.

**Caveats** (CH-14 retro §5 rows 5 + 11; chunk-planner v7 effective 2026-05-08):

- **Migration-number reservation.** When the plan adds a new SQL migration, run `ls /root/projects/phi/baby-phi/modules/crates/store/migrations/` at draft-time and capture the **next-free slot** (highest existing 4-digit prefix + 1). Do NOT assume sequential numbering from the chunk's hex order. CH-14 plan said `0011` was next-free but actual was `0014` (slots 0011/0012/0013 already taken by CH-23/CH-10/CH-11). Implementer corrected at P0 with no harm, but the plan-vs-reality drift is avoidable by greppting at draft-time. Cite the next-free slot in §3 + the migration deliverable bullet.
- **Typed multi-value cascade-result precedent (`CascadeResult` pattern).** When a cascade method must return multiple semantically-distinct lists (e.g., "revoked grants" + "cascaded ARs", or "transitioned-states" + "emitted-events"), prefer a typed struct (`CascadeResult { revoked_grants: Vec<GrantId>, cascaded_ars: Vec<AuthRequestId> }`) over raw `Vec<X>` or tuple returns. CH-14 ships this at `domain/src/repository.rs:172-182` for the recursive-revoke cascade (`Repository::revoke_grants_by_descends_from_recursive`). Rationale: handler-side iteration becomes self-documenting; future cascade-result extensions (e.g., adding a third list field) don't break callsite signatures or trait-method shapes; field names compose with `cascade.revoked_grants.len()` more readably than `cascade.0.len()`. Cite this precedent when the chunk introduces or extends a cascade method.

### §3.B — K8s microservice readiness check

**Binding rule (codified by CH-01 / forward-scope Q8 — every chunk applies):** the chunk plan evaluates whether its changes introduce new K8s-deployment hurdles. The full rationale + strategic context lives in [`m7b/architecture/k8s-microservices-readiness.md`](../../m7b/architecture/k8s-microservices-readiness.md); the tactical ledger of deferred items is at [`m7b/architecture/deferred-from-ch-k8s-prep.md`](../../m7b/architecture/deferred-from-ch-k8s-prep.md). Pre-CH-01 chunks (CH-02, CH-K8S-PREP) are grandfathered.

**Mandatory 7-axis evaluation table** (chunk plan fills in for its surface):

| Axis | What to check | This chunk's surface | New blocker introduced? | Action |
|---|---|---|---|---|
| **A1** | New in-process state (`DashMap`, `RwLock`, `AtomicBool`, `Mutex`, `OnceCell`, `RefCell`, etc.) — pod-local by definition | … | yes / no | If yes, file CHK8S-D-XX entry; consider trait-shaping now |
| **A2** | New IPC channel (`mpsc`, `broadcast`, `oneshot`, `watch`, `Notify`) — pod-local by definition | … | yes / no | If yes, file CHK8S-D-XX |
| **A3** | New pod-local resource (file handle, listener socket, sub-process, lock file, on-disk cache) | … | yes / no | If yes, file CHK8S-D-XX |
| **A4** | Migration runner / first-apply race | … | yes / no | If migration added, cross-ref existing CHK8S-D-05 (leader-election lock) — generally not aggravated by single column-add migrations |
| **A5** | Trait-shape requirement (does the new surface need to be trait-objects-friendly for future broker / Redis / remote-DB swap?) | … | yes / no | If yes and not already trait-shaped, ship trait now |
| **A6** | Cross-pod state sharing (does this introduce data that must be visible across pods?) | … | yes / no | If yes, ensure storage backend is durable (SurrealDB persisted), not in-process cache |
| **A7** | Audit hash-chain symmetry (does the chunk add a new audit writer that breaks single-writer guarantee or sidesteps the existing emitter?) | … | yes / no | If yes, file CHK8S-D-XX (single-writer integrity is load-bearing for the audit chain) |

**Conforming-criteria check against ADR-0033 (CH-K8S-PREP):**
- D33.1 (`SessionRegistry` trait) — does this chunk touch the registry? If yes, does it preserve trait-object dispatch?
- D33.2 (`SurrealStore::open_remote`) — does this chunk add storage operations that work on both `open_embedded` and `open_remote`?
- D33.3 (SIGTERM graceful shutdown) — does this chunk add new `tokio::spawn` tasks that the SIGTERM handler must drain?
- D33.4 (`EventBus.shutdown` + `drain`) — does this chunk add new `EventBus` emitters or listeners?

**Conclusion paragraph.** State the chunk's overall K8s posture in one sentence: *"K8s-neutral"* (no new blockers), *"K8s-positive"* (existing blockers improved), or *"K8s-negative"* (new blockers introduced — list them with their CHK8S-D-XX ledger references).

**Mid-flight discovery.** If a phase surfaces a K8s blocker not anticipated in this section, pause via `AskUserQuestion` and add a new ledger entry before the phase closes — identical pattern to the §2 concept-contradiction discovery rule.

### §3.C — User-facing documentation impact map

**Binding rule (codified post-CH-22):** every chunk evaluates which user-facing docs (architecture / operations / user-guide tier under `docs/specs/v0/implementation/<milestone>/`) its changes affect, and either (a) updates them in-chunk, or (b) explicitly defers each one with a stated reason and a successor chunk reference. Pre-CH-22 chunks (CH-01, CH-02, CH-K8S-PREP, CH-22) are grandfathered — their backfill is bundled with the codification commit.

The mandate applies because the milestone-era pattern was three peer doc trees per milestone (`mX/architecture/`, `mX/operations/`, `mX/user-guide/`). When chunks replaced milestones as the unit of work, they kept governance-tier doc updates (drifts, ADRs, concept-audit matrix) but silently dropped the user-facing tier. Without an explicit per-chunk gate, the user-facing docs go stale relative to the code — operators reading them get an out-of-date mental model.

**Mandatory 3-tier evaluation table** (chunk plan fills in for its surface):

| Tier | File pattern | This chunk touches? | Action |
|---|---|---|---|
| **Architecture** | `docs/specs/v0/implementation/<milestone>/architecture/<feature>.md` — design, data flow, phi-core leverage call-outs | list each file; mark "no change" if the design didn't shift | (a) update in-chunk OR (b) defer with reason + successor-chunk reference |
| **Operations** | `docs/specs/v0/implementation/<milestone>/operations/<feature>-operations.md` — error-code reference, audit-event dictionary, incident playbooks, metrics | list each file; mark "no change" if no new codes / playbooks land | (a) update in-chunk OR (b) defer |
| **User-guide** | `docs/specs/v0/implementation/<milestone>/user-guide/{<feature>-walkthrough,cli-reference-mN,troubleshooting}.md` — operator tours, CLI commands, stable error codes | list each file; mark "no change" if no operator-visible behaviour shifts | (a) update in-chunk OR (b) defer |

**Rules:**
- Every doc the chunk's code makes stale MUST appear in the table — no "we'll find out at seal time."
- "Defer" decisions need a successor chunk ID (or `M<n>-tag-close` for milestone batches), not just a reason. Open-ended deferrals are not permitted; they reproduce exactly the gap this rule closes.
- Doc updates run as deliverables in §7 phases (typically the seal phase). They are not after-the-fact appendices.
- The §10 "Docs aspect" close criterion now covers BOTH governance docs (drifts/ADRs/matrix — the existing scope) AND this user-facing tier. A chunk that ships code but skips a §3.C doc with no defer-decision fails Docs aspect.

**Mid-flight discovery.** If a phase makes a doc stale that wasn't anticipated in §3.C, pause via `AskUserQuestion` and add a row to the table before the phase closes — same pattern as §2 concept-contradiction discovery and §3.B K8s-blocker discovery.

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

**ADR-body checklist (v2026-05-04 per CH-13 retrospective, cycle hex `d4fe1b7c`):** every ADR drafted by a chunk MUST include:

1. **§"Forks" header with explicit user-lock outcome.** Direct-approval cycle: `Forks (all planner-recommended at chunk-open; user-locked at plan approval to F1.A / F2.A / F3.A)`. Divergent cycle: `Forks (F1 user-locked to F1.B at plan approval — diverges from planner recommendation F1.A; F2 / F3 at planner-recommendation)`. The header MUST capture the lock-state, not just the planner-recommendation.
2. **§"Cross-references" with all 4 categories.** (a) originating concept-doc + section + line range; (b) closed drift(s) by ID; (c) prior ADRs cited as precedent; (d) **forward-scope row** that scoped this chunk (path + line range). The forward-scope cross-reference is mandatory — the ADR is the bridge between concept-doc and code, and must trace back to the chunk-source. Closes CH-13 Audit B Trivial-1L gaps F-AUDB-1 + F-AUDB-2.
3. **Pre-existing-behaviour preservation note** (added 2026-05-08 per CH-14 retro Row 10, cycle hex `5803bb94`): if your ADR sub-decision changes audit-event emission semantics, AR/Grant state-transition behaviour, or any other prior runtime invariant, **document the pre-existing behaviour explicitly** in the relevant sub-decision body. CH-14 ADR-0053 §D53.7 needed mid-cycle clarification: the level-0 adoption AR's companion `auth_request.revoked` event from `revoke_ar` continues to be discarded — pre-CH-14 behaviour preserved verbatim. Future ADRs reading the historical record should be able to identify which pre-existing behaviours are now relied-upon invariants vs which were intentionally changed. Format: a sentence inside the sub-decision like "Pre-existing behaviour preserved: `<X>` (see `<file:line>` for the historical implementation; CH-NN does not change this)" or "Behaviour changed: `<X>` (was `<old>` per `<prior ADR §>`; CH-NN flips to `<new>` because `<reason>`)".

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
- **User-facing doc updates** (post-Q9 / CH-22) — which §3.C rows this phase satisfies. Doc updates are first-class deliverables, not after-the-fact appendices. A phase that ships code making a §3.C-listed doc stale must include the doc update in its Deliverables list. Phases that touch zero user-facing docs say so explicitly.
- **Confidence target** — ≥ X% composite (defaults: P0/scaffolding 100%, content phases ≥ 97%, seal ≥ 99%).
- **Pause discipline** — any known-upfront fork point where the phase MUST halt for `AskUserQuestion` before continuing.

### §8 — Tests summary

- **Expected total test count at chunk close** — concrete number (e.g., "973 from current 966 baseline + 7 new tests"). Apply a **× 1.10–1.15 buffer** to the sum of deliverable-listed unit + integration + property + acceptance tests for the **plan §8 chunk-close prediction band**. The orchestrator-accept band is **asymmetric** (deliverable-listed sum × 1.0 lower bound; deliverable-listed sum × 1.20 upper bound) — CH-11 + CH-12 cycle data confirm healthy implementer over-shoot is biased high, not symmetric. Healthy implementer over-shoot (one-property-per-row determinism tests; round-trip helpers; paired audit-event tests; ISO-8601 / serde-format helper unit tests; precedence regression tests) is normal and should not trigger a re-plan. Outside the asymmetric accept band → AskUserQuestion. *Added v2026-05-03 per CH-11 retrospective (cycle hex `d5428c43`); refined to asymmetric ×1.0–×1.20 v2026-05-04 per CH-12 retrospective (cycle hex `6a748175`) — CH-11 actual was +22 over conservative target; CH-12 actual was bull's-eye 1365 within the 1360–1366 prediction band, confirming the buffer factor calibration on the second cycle.*
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
6. **Conditional (v2026-05-03 per CH-11 retrospective, cycle hex `d5428c43`)**: if the chunk touches `domain::permissions::engine` Step N body, the reading list MUST include `server::platform::sessions::launch.rs` body + `server::platform::sessions::preview.rs` body (the synthetic-manifest construction sites), so manifest-shape preconditions are discovered at plan time, not at implementation. CH-11 surfaced drift D4.1 (preview-path manifest-resource bug) at implementation that should have been visible at plan time.

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
- **Docs aspect** — every affected doc updated. Two scopes:
  - *Governance tier*: status tags, verified headers, concept-audit matrix rows, drift-file lifecycle entries, ADR status flips.
  - *User-facing tier* (post-CH-22): every row of the §3.C impact map either updated in-chunk or carrying an explicit defer-decision with successor-chunk reference.
- **phi-core leverage aspect** — §3 import-count delta matches prediction ± documented variance; all forbidden-duplication greps return 0; `check-phi-core-reuse.sh` green.
- **Concept alignment aspect** — every §2 row's target-status at chunk-close achieved; none remain `contradicted`.

**2 confidence % (each with named numerator/denominator):**
- **Implementation confidence %** = `(claims-verified-honored-by-tests-and-code-inspection) / (total-claims-in-scope-for-chunk)`. Example: *"14/15 claims honored = 93%."* The 1 remaining claim gets its own named drift file + explicit re-scope.
- **Documentation confidence %** = `(doc-pages-where-independent-reader-can-cross-check-against-code-+-concept-+-ADRs-without-ambiguity) / (doc-pages-touched-in-chunk)`. Example: *"8/8 = 100%."*

**Composite = min(impl%, doc%, code-aspect-binary, phi-core-leverage-aspect-binary, concept-alignment-aspect-binary).** A failing aspect is 0%. Composite below target blocks close.

**Explicit close-target discipline:** close report states ALL FIVE measures with named numerators/denominators. No aspect-averaging. No rounding up.

**P4 chunk-seal paperwork checklist (v2026-05-03 per CH-11 retrospective, cycle hex `d5428c43`):** for every modified doc with a verified-header (line 1 `<!-- Last verified: ... -->`), confirm the header description matches the body diff exactly. Mismatch → fix the header before chunk-seal. CH-11 audit B claim 7 (`_concept-audit-matrix.md`) caught a header overpromise that survived to audit time; this checklist item ensures it's caught at P4.

**P4 paperwork checklist addendum (v2026-05-04 per CH-12 retrospective, cycle hex `6a748175`):** for every `_concept-audit-matrix.md` row touched by the chunk, the new Status column value MUST be copy-pasted letter-for-letter from the plan §2 target column for that row — not the implementer's interpretation. Split-axis cases (e.g., `partially-honored → still partially honored at axis X but axis Y flips to honored`) keep the original Status (`partially-honored`) with axis qualification in the evidence cell, not a binary flip to `honored`. CH-12 Audit B iter 1 F-AUDB-1 caught a row 191 collapse-to-binary that survived to audit time; the letter-for-letter rule prevents the recurrence.

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
