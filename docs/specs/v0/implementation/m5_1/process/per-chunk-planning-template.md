<!-- Last verified: 2026-05-26 by Claude Code (Chunk D `36caa39f` intermediate-stabilization plan §"Chunk D" — outer phi chunk-planner v32 iter-2 re-arch, orchestrator-direct paperwork cycle). Adds `## §1 — Locked fork details — [DRAFT planner-rec bodies; pre-lock]` populated-at-iter-1 template structure consumed by chunk-planner v32 P-plan-1-v32 + outer CLAUDE.md P-orch-8 gate-1.5 skip-condition. Project-agnostic (consumed by both baby-phi AND i-phi planner cycles). Companion: deprecates chunk-planner v22 P13 / v23 P-plan-3 ALWAYS-FIRE iter-2-respawn for planner-rec-clean cycles via 2-3-cycle cross-project hold-period. -->
<!-- Last verified: 2026-05-20 by Claude Code (post-CH-28 retro standards-update batch: NEW §2.5 Functional outcome mandatory section + §3 fork-row format extension for user-facing framing + §4 Drifts closed table extended with Product impact during deferral column; closes CH-28-observed gaps in chunk-decomposition + fork-framing + deferred-feature-visibility per plan archive chunk-decomposition-and-fork-framing-76e04080.md). -->
<!-- Last verified: 2026-04-27 by Claude Code -->
<!-- Post-CH-22 addition (2026-04-27): §3.C "User-facing documentation impact map" is now mandatory; §10 "Docs aspect" extended to cover the user-facing doc tier (architecture / operations / user-guide). Reason: the milestone-era pattern shipped three peer doc trees per milestone, but chunks were silently dropping that tier — operators got stale docs. Pre-CH-22 chunks (CH-01, CH-02, CH-K8S-PREP, CH-22) are grandfathered with backfill bundled in this codification commit. -->

# Per-chunk planning template

Every implementation chunk opened after M5.1 ships its own standalone plan file before any code moves. This document is the canonical template. Drafters copy it verbatim into `baby-phi/docs/specs/plan/build/<8hex>-<chunk-name>.md` and fill every section. Incomplete templates do not qualify for `ExitPlanMode` approval.

The template bakes in the M5.1/P3 Q1–Q7 planning decisions (see [`forward-scope/remaining-scope-post-m5-p7-22035b2a.md`](../../../../plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md) §7). Chunk-plan authors do not re-litigate those decisions; they apply them.

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

## Pre-§1 — "Forks for orchestrator" section (planner-authored above §1; format rules added 2026-05-20 per CH-28 retro plan archive `chunk-decomposition-and-fork-framing-76e04080.md`)

When the chunk plan has architecture/scope decisions that need user-lock at gate-1, the planner adds a `## Forks for orchestrator` section ABOVE §1 (this section is NOT part of the numbered 12 — it's planner-authored only when forks exist; chunks with no user-decidable forks omit it).

**Fork row format (mandatory for FUNCTIONAL chunks; release-by-label for TECHNICAL-PREREQUISITE chunks)**:

Each fork has a header table with one row per option:

```
### F<N> — <fork-name>

| Option | User-visible (what the user perceives) | Pros | Cons + Product trajectory | Status |
|---|---|---|---|---|
| F<N>.a (planner-rec) | <one-sentence behavior the end user perceives if this option ships> | <2-3 bullet pros, 1-sentence each> | <1-2 bullet cons + **Product trajectory:** <how this affects long-term product trajectory — what capabilities become easier/harder downstream>> | LOCKED / NOT chosen |
| F<N>.b (...) | ... | ... | ... | ... |
```

**Discipline**:
- The **User-visible** column states what the END USER perceives — NOT the implementation layer. Avoid architectural jargon ("wire-format-explicit", "auditability", "operator inspection window"). Frame in user-perceivable behavior ("templates can be shared across N agents", "schema applies before data migration so operators can inspect", etc.).
- The **Product trajectory** line at the end of cons states the long-term product impact (better/worse for which downstream capabilities) — distinct from this chunk's engineering tradeoff.
- For forks where **all options share zero user-visible delta** (purely engineering choice; e.g., `tokio::sync::Mutex` vs `std::sync::Mutex`), the fork header MUST be labeled `**TECHNICAL FORK** (no user-visible delta — pick on engineering merit only)`. This releases the planner from the User-visible + Product trajectory requirements for that fork; the row format collapses to standard pros/cons.
- The chunk-planner agent (v26+) self-checks own draft for these substrings at draft-end + retries until present.

**Locked fork details appendix** (existing v23 rule, preserved): when ≥ 1 fork locks at gate-1, the planner adds a `### Locked fork details` subsection BELOW the fork tables with one `#### F<N> = F<N>.<letter>` sub-section per locked fork carrying 3-6 sentences of plain-English semantics of what the lock means for implementer + downstream consumers + which open-questions it closes.

**Why this format**: CH-28 retro observed that forks were framed in engineering terms ("hybrid Blueprint table", "split migrations") that the user could not translate to product-level decisions. The new mandatory **User-visible** column closes the framing gap structurally.

---

## §1 — Locked fork details (per chunk-planner v32 iter-1 template; planner-rec bodies pre-filled)

> **Added 2026-05-26 per Chunk D `36caa39f` intermediate-stabilization plan §"Chunk D"** — outer phi chunk-planner v32 iter-2 re-arch. Project-agnostic; consumed by both baby-phi AND i-phi planner cycles. Companion to chunk-planner v32 P-plan-1-v32 + outer CLAUDE.md gate-1.5 P-orch-8 skip-condition + chunk-initiate Phase 1.5 Step A flip + chunk-archive-plan hard-assertion.

> **Pre-lock state at iter-1 draft**: the planner ships draft bodies for each fork's planner-rec option below at iter-1 plan-draft time (NOT as a placeholder; NOT deferred to iter-2 re-spawn). Each `#### F<N> = F<N>.<planner-rec-letter>` subsection carries 3–6 sentences of Code-level binding / Rationale / Defers. **At gate-1 lock**:
>
> - If user selects planner-rec for ALL forks → archive directly at iter-1 (per outer CLAUDE.md P-orch-8 skip-condition + chunk-initiate Phase 1.5 Step A skip-condition); these bodies become the locked record verbatim.
> - If user selects USER-DIVERGENT on any fork → iter-2 re-spawn fires per Step B material-scope-expansion; ONLY the divergent F<N> subsections re-authored; planner-rec subsections preserved at iter-1 draft.

```markdown
#### F<N> = F<N>.<planner-rec-letter> *(pre-lock draft; finalizes at gate-1)*

**Code-level binding** (3 sentences): <what code site changes; what struct/trait/method/file:line absorbs the lock; how the locked option manifests in source-tree shape (NEW file vs EXTEND file vs trait-method addition vs config schema add)>.

**Rationale** (3 sentences): <why this planner-rec over the named alternatives in §"Forks for orchestrator"; cite the decisive tradeoff axis (user-visible delivery vs cycle scope vs precedent-alignment vs cascade footprint); reference any cross-cycle pattern that informed the recommendation>.

**Defers (if chosen)** (3 sentences): <what features NOT shipping under this option; cite allocation chunk-IDs (e.g., M6-DEFERRED-NN / CH-NN+) or "none deferred" if the option ships the full surface for this chunk; note any downstream consumer that inherits the deferred state>.

#### F<N+1> = F<N+1>.<planner-rec-letter> *(pre-lock draft; finalizes at gate-1)*

... <repeat for each fork in `## Forks for orchestrator`>
```

**Rules**:

- **Always-populated at iter-1**: the planner authors all `#### F<N>` subsections in the iter-1 draft. No "to be filled at gate-1.5 iter-2 re-spawn" placeholder. The chunk-template-validate-locked-appendix skill enforces this at planner end-of-draft + at chunk-archive-plan hard-assertion (belt-and-suspenders).
- **3-sentence minimum** per body block (Code-level binding / Rationale / Defers each ≥ 3 sentences; total ≥ 9 sentences per `#### F<N>` block).
- **TECHNICAL FORK release**: for `**TECHNICAL FORK** (no user-visible delta — pick on engineering merit only)`-labeled forks, the §1 subsection bodies MAY collapse Code-level binding to a 1-sentence form citing the engineering-merit decision; Rationale + Defers retain 3-sentence minimums.
- **USER-DIVERGENT path**: at iter-2 re-spawn after gate-1, ONLY the subsection(s) corresponding to divergent fork(s) are re-authored (the planner-rec letter changes to the user-divergent letter; the body re-derives Code-level binding / Rationale / Defers for the divergent option). Planner-rec subsections preserve their iter-1 wording.

**Cross-references**:

- chunk-planner v32 P-plan-1-v32 — origin rule (defines the iter-1 template change + deprecates v22 P13 / v23 P-plan-3 ALWAYS-FIRE iter-2 re-spawn for planner-rec-clean cycles).
- chunk-template-validate-locked-appendix skill — mechanical 4-step PASS/FAIL validation at planner end-of-draft + chunk-archive-plan invocation.
- chunk-archive-plan v4 hard-assertion — invokes chunk-template-validate-locked-appendix BEFORE minting cycle-folder; refuses archive on FAIL.
- chunk-initiate Phase 1.5 Step A skip-condition — flips iter-2 re-spawn from ALWAYS-FIRE to UNLESS-planner-rec-clean-AND-§1-populated.
- Outer CLAUDE.md gate-1.5 P-orch-8 — orchestrator-side skip-condition mirror.
- User memory `feedback_locked_fork_details_appendix.md` — original directive (locked-fork-details section before approval; v32 satisfies via iter-1-populated bodies, not iter-2 re-spawn).

---

> **Note on section numbering (v32 transition)**: the body sections below (§1 Context & principle through §12 Verification recipe) are conceptually §2–§13 under the v32 template (§1 above carries Locked fork details). Existing template body retains the §1–§12 numbering for backward compatibility with pre-CH-32-canonical plans. Future plans authored under v32 MAY renumber §1→§2 / §2→§3 / ... / §12→§13 to fully align with the v32 convention; the chunk-template-validate-locked-appendix skill is renumber-agnostic (validates the `## §1 — Locked fork details` heading regardless of body-section numbering).

---

### §1 — Context & principle

- **Why this chunk** — one paragraph. What user-visible or concept-fidelity gap does this chunk close? Which drift IDs make it necessary?
- **Quality-over-speed restatement** — restate the M5.1 governing principle: *"Concept docs are source-of-truth; implementation aligns to them. Drift is discovered, documented, and planned-through — never accumulated silently."* One-sentence chunk-specific application.
- **Forward-scope reference** — explicit link to the `CH-NN` row in [`forward-scope/remaining-scope-post-m5-p7-22035b2a.md`](../../../../plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md) §5.

### §2 — Concept alignment walk

Full table of every concept-doc claim the chunk touches:

| Concept doc | § anchor | Claim (verbatim or close paraphrase) | Status at chunk-open | Target status at chunk-close |
|---|---|---|---|---|
| `concepts/<doc>.md` | `§<anchor>` | `"..."` | honored / contradicted / partially-honored / silent-in-code / concept-aspirational | honored / out-of-scope-for-chunk |

Rules:
- **Coverage** — every concept doc whose claims the chunk's code will touch appears in the table. No "we'll find out at implementation time."
- **Permissions subtree hook** — if any of `permissions/01`–`permissions/09` docs are touched, `permissions/README.md` MUST be cited as the entry invariants source.
- **phi-core-mapping hook** — if any surface overlaps with phi-core types, `concepts/phi-core-mapping.md` MUST appear in the table with the relevant row cited.

### §2.5 — Functional outcome (mandatory; added 2026-05-20 per CH-28 retro plan archive `chunk-decomposition-and-fork-framing-76e04080.md`)

Every chunk plan MUST declare whether it is FUNCTIONAL (ships user-visible capability) or TECHNICAL-PREREQUISITE (purely structural; no user-visible delivery; unblocks downstream FUNCTIONAL chunk). The declaration is a 1-paragraph statement at chunk-plan-draft time so the user reading the plan can immediately answer "what does this chunk give me as an end user?" without inferring from technical details.

**Format**:

For **FUNCTIONAL** chunks:
```
**Chunk-type**: FUNCTIONAL
**User-visible delivery**: <one-paragraph description of what the end user can do post-chunk-close that they could not do before>.
```

For **TECHNICAL-PREREQUISITE** chunks:
```
**Chunk-type**: TECHNICAL-PREREQUISITE
**User-visible delivery**: NONE this chunk.
**Unblocks**: <CH-NN+M> which ships <user-visible feature>.
**Why this prerequisite**: <one-sentence explanation a non-technical user can understand — e.g., "the synthesis read-path makes templates shareable across agents, which the supervisor surface at CH-36 needs to render correctly">.
```

**Discipline at plan-draft**: the planner must NOT bundle FUNCTIONAL + TECHNICAL-PREREQUISITE deliverables in a single chunk. If the candidate chunk mixes user-visible feature work + ≥ 30% supporting infrastructure with no user delivery, the planner MUST propose a split into 2 chunks (per phase-planner v2 decomposition discipline). The split rationale lands in §2.5 + the second chunk's plan picks up the TECHNICAL-PREREQUISITE tag with explicit "Unblocks" line.

**Cross-cycle observation** (CH-28 evidence): chunks that bundle user-visible capabilities + multi-axis infrastructure ramp the iteration count (CH-28 ran 5 plan iterations in part because the chunk bundled hybrid Blueprint + edge rename + split migrations + composite-write + acceptance suite). Splitting reduces per-chunk decision surface + makes fork choices tractable.

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

**Caveats** (CH-15 retro §5 rows 2 + 3; chunk-planner v8 effective 2026-05-08):

- **Typed-multi-value cascade pause-threshold sizing.** When a cascade is a typed-multi-value return-type change (e.g., `Grant → Vec<Grant>`, `T → CascadeResult { ... }`), the deliberate-edit count is mechanically inflated by destructure-from-tuple → destructure-from-Vec test-amendment patterns. Size the pause-threshold against the **deliberate-non-test-amendment edit count** (NEW production behaviour) **separately** from the **test-amendment edit count** (mechanical destructure ripple). CH-15 Artifact A predicted ~12 deliberate edits / threshold 18; actual was 29 edits — 14 of those were test-amendments (mechanical), 15 were the genuine production change. Sizing the threshold against 15 vs 14 separately would have kept the cycle within band. Cross-ref CH-14 ADR-0053 §D53.4 (`CascadeResult` typed return precedent) + CH-15 ADR-0054 §D54.3 (`Vec<Grant>` typed return).
- **Canonical phi-core baseline grep**: use `grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l` (NO trailing `::`) as the canonical phi-core import-count measurement. Both `use phi_core::Foo` AND `use phi_core;` (rare) AND `pub use phi_core::Bar` count toward the baseline. CH-15 caught a measurement-method drift: plan + auditor used `grep -rn "use phi_core"` (returns 49); orchestrator used `grep -rn "use phi_core::"` (returns 48 — strictest form). Same physical state, different measurements. Codify the no-`::` form across all 4 audit lanes (planner draft + implementer post-phase report + auditor verification + orchestrator gate-4 MUST-RUN). Per CH-08 retro Row 3's canonical grep-pattern naming rule applied here.

**Caveats** (CH-14 retro §5 rows 5 + 11; chunk-planner v7 effective 2026-05-08):

- **Migration-number reservation.** When the plan adds a new SQL migration, run `ls /root/projects/phi/baby-phi/modules/crates/store/migrations/` at draft-time and capture the **next-free slot** (highest existing 4-digit prefix + 1). Do NOT assume sequential numbering from the chunk's hex order. CH-14 plan said `0011` was next-free but actual was `0014` (slots 0011/0012/0013 already taken by CH-23/CH-10/CH-11). Implementer corrected at P0 with no harm, but the plan-vs-reality drift is avoidable by greppting at draft-time. Cite the next-free slot in §3 + the migration deliverable bullet.
- **Typed multi-value cascade-result precedent (`CascadeResult` pattern).** When a cascade method must return multiple semantically-distinct lists (e.g., "revoked grants" + "cascaded ARs", or "transitioned-states" + "emitted-events"), prefer a typed struct (`CascadeResult { revoked_grants: Vec<GrantId>, cascaded_ars: Vec<AuthRequestId> }`) over raw `Vec<X>` or tuple returns. CH-14 ships this at `domain/src/repository.rs:172-182` for the recursive-revoke cascade (`Repository::revoke_grants_by_descends_from_recursive`). Rationale: handler-side iteration becomes self-documenting; future cascade-result extensions (e.g., adding a third list field) don't break callsite signatures or trait-method shapes; field names compose with `cascade.revoked_grants.len()` more readably than `cascade.0.len()`. Cite this precedent when the chunk introduces or extends a cascade method.

**Caveats** (CH-02b-i-phi retro Row 1, cycle hex `57b20bda`; chunk-planner v17 effective 2026-05-17):

- **Per-fork pause-threshold re-derivation after gate-1 fork-locks.** §3 cascade-thresholds (file count cap, per-file LOC cap, Cargo.lock churn cap) are derived at plan-draft, BEFORE gate-1 locks the forks. When the orchestrator locks a fork that materially expands scope (e.g., F4.b 6 handlers vs F4.a 3; F-error.b thiserror enum vs F-error.a anyhow), the plan-draft thresholds may no longer reflect the locked scope. The plan §3 cascade paragraph MUST emit a **per-fork pause-threshold table** that lists each fork × its impact on the cascade-vector thresholds; the orchestrator at gate-1 re-derives the active thresholds by summing locked-option deltas before handing the implementer the chunk-open invariant set. CH-02b precedent: `src/daemon/ipc/server.rs` shipped at 354 LOC vs plan-draft 250 LOC cap (1.5× predicted 150); planner under-predicted F2.a transport-dual complexity, and threshold was NOT re-derived after gate-1. Companion rule at chunk-implementer.md v11 — implementer MUST AskUserQuestion at the breach (not just log in phase-close report).

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

### §3.D — Forward-scope-vs-concept-doc precedence (added 2026-05-08 per CH-15 retro Row 4, cycle hex `c3f46f17`)

**Binding rule**: when the forward-scope row's literal text disagrees with a closed concept-doc invariant — e.g., **closed action vocabulary** (concept-doc 03 §"Standard Action Vocabulary" — closed 34-verb set; `Action::CANONICAL.len() == 34` invariant), **fixed migration order**, **frozen schema**, **closed namespace tag set** — **the concept doc wins**. The forward-scope row's wording becomes a documented re-interpretation in the relevant ADR sub-decision. Mechanical procedure:

1. **Planner-side detection** (chunk-planner v8, per CH-15 retro Row 5): when reading the forward-scope row, grep concept-doc invariants for closed-set / fixed-order / frozen-schema language. If the forward-scope row's literal terms (e.g., "actions `session.start` / `session.tool_invoke` / `session.read_memory`") are NOT present in the concept-doc closed set, flag this in plan §"Forks for orchestrator" as **CRITICAL fork requiring user-lock** with explicit re-interpretation rationale documented in the ADR.
2. **ADR-side documentation**: the relevant ADR sub-decision MUST carry a "forward-scope row literal re-interpretation note" explaining (a) what the row's literal text was, (b) which concept-doc invariant it conflicted with, (c) how the chunk re-interprets the row's wording (typically as scoping-gloss describing logical reaches rather than literal artifacts), (d) what the actual implementation surface is. CH-15 ADR-0054 §D54.2 + §D54.8 is the precedent.
3. **User-lock outcome**: the user explicitly confirms the re-interpretation at gate-1; the cycle proceeds with the concept-doc-aligned shape.

Rationale: concept docs are source-of-truth for the v0 platform per `baby-phi/CLAUDE.md` "Documentation Alignment" + "Working Discipline" principles; forward-scope rows are scoping artifacts that may have been authored before all concept-doc invariants were closed. Codifying the precedence prevents future cycles from silently expanding closed sets to honor stale forward-scope wording. Cross-ref ADR-0054 §D54.2 + §D54.8 (CH-15 first instance).

### §3.C — User-facing documentation impact map

**Binding rule (codified post-CH-22):** every chunk evaluates which user-facing docs (architecture / operations / user-guide tier under `docs/specs/v0/implementation/<milestone>/`) its changes affect, and either (a) updates them in-chunk, or (b) explicitly defers each one with a stated reason and a successor chunk reference. Pre-CH-22 chunks (CH-01, CH-02, CH-K8S-PREP, CH-22) are grandfathered — their backfill is bundled with the codification commit.

The mandate applies because the milestone-era pattern was three peer doc trees per milestone (`mX/architecture/`, `mX/operations/`, `mX/user-guide/`). When chunks replaced milestones as the unit of work, they kept governance-tier doc updates (drifts, ADRs, concept-audit matrix) but silently dropped the user-facing tier. Without an explicit per-chunk gate, the user-facing docs go stale relative to the code — operators reading them get an out-of-date mental model.

**Mandatory 3-tier evaluation table** (chunk plan fills in for its surface):

| Tier | File pattern | This chunk touches? | Action |
|---|---|---|---|
| **Architecture** | `docs/specs/v0/implementation/<milestone>/architecture/<feature>.md` — design, data flow, phi-core leverage call-outs | list each file; mark "no change" if the design didn't shift | (a) update in-chunk OR (b) defer with reason + successor-chunk reference |
| **Operations** | `docs/specs/v0/implementation/<milestone>/operations/<feature>-operations.md` — error-code reference, audit-event dictionary, incident playbooks, metrics | list each file; mark "no change" if no new codes / playbooks land | (a) update in-chunk OR (b) defer |
| **User-guide** | `docs/specs/v0/implementation/<milestone>/user-guide/{<feature>-walkthrough,cli-reference-mN,troubleshooting}.md` — operator tours, CLI commands, stable error codes | list each file; mark "no change" if no operator-visible behaviour shifts | (a) update in-chunk OR (b) defer |

**Amend-don't-add precedence rule (added 2026-05-09 per CH-17 retro Row 3, cycle hex `40c4d759`):** when planner predicts a `m<N>/user-guide/` NEW file for a chunk-narrative walkthrough, the §3.C row MUST evaluate whether an existing `m<prev>/user-guide/*` file (the discoverable single-walkthrough surface for users) would be a more appropriate amendment target than fragmenting users across a 2-file walkthrough. Document the evaluation rationale inline in the §3.C row; if amendment chosen, cite the target file + planned subsection title (e.g., `"CH-NN amendment — <feature> (date)"`). CH-17 caught this at audit B iter 1: plan §3.C row 4 predicted NEW `m5_2/user-guide/session-live-events-walkthrough.md`; orchestrator-applied Trivial-1L corrected to `m5/user-guide/first-session-walkthrough.md` "CH-17 amendment — live SSE tail (2026-05-09)" subsection. Default to amendment unless the existing m<prev>/user-guide/* file is a wholly different audience (e.g., admin vs operator) — then NEW is justified.

**Rules:**
- Every doc the chunk's code makes stale MUST appear in the table — no "we'll find out at seal time."
- "Defer" decisions need a successor chunk ID (or `M<n>-tag-close` for milestone batches), not just a reason. Open-ended deferrals are not permitted; they reproduce exactly the gap this rule closes.
- Doc updates run as deliverables in §7 phases (typically the seal phase). They are not after-the-fact appendices.
- The §10 "Docs aspect" close criterion now covers BOTH governance docs (drifts/ADRs/matrix — the existing scope) AND this user-facing tier. A chunk that ships code but skips a §3.C doc with no defer-decision fails Docs aspect.

**Mid-flight discovery.** If a phase makes a doc stale that wasn't anticipated in §3.C, pause via `AskUserQuestion` and add a row to the table before the phase closes — same pattern as §2 concept-contradiction discovery and §3.B K8s-blocker discovery.

### §3.E — Anticipated gate-2.5 candidates (added 2026-05-11 per CH-24 retro Row 6, cycle hex `5778bb77`; chunk-planner v13)

**Optional planner-led section** (planner inserts when chunk-shape suggests gate-2.5 candidates; orchestrator MAY add at gate-1 review if planner missed any).

CH-24 demonstrated mid-cycle architectural scope expansion as a viable workflow: P-NEW-TESTS authoring surfaced `recent_sessions: Vec::new()` hardcoded placeholder at `detail.rs:229`; user locked close-in-chunk at gate-2.5; new phase `P-FLIP-RECENT-SESSIONS` inserted; ADR-0059 ratified + drift remediated in-cycle. The pattern is first-of-kind across all chunks.

**Surfaces likely to be touched by P-NEW-TESTS / P-DOCS authoring that may surface mid-flight discoveries** (planner enumerates per chunk):

- Doc-comments referencing deferred-but-shipping behaviour (e.g., "deferred to M5 per Dxx" — likely flips during authoring).
- Placeholder `Vec::new()` / `Default::default()` returns with an inline "ships at M5+" comment.
- Stale transitional struct shapes (e.g., `RecentSessionStub` style placeholders awaiting C-Mn flip).
- ADR sub-decisions marked Proposed at chunk-open that may be ratified/rejected mid-cycle.

**Per candidate**: surface a "if discovered at gate-2.5, route to <option-A close-in-chunk via P-FLIP-<X> phase> OR <option-B file follow-up drift + retrospective routing>" recommendation. This pre-loads the gate-2.5 fork rather than synthesising it at-discovery.

**Default rule**: no candidates → write "(none anticipated)" + proceed. The section is empty for ratification chunks (no code surface), small for medium chunks (1-2 candidates), large for milestone-seal chunks (3-5 candidates).

### §4 — Drifts closed + Deferred functionality

List every drift file in [`../drifts/`](../drifts/) this chunk transitions to `remediated` / `renegotiated` / `accepted-as-is`:

| Drift ID | File | Severity | Transition | Notes |
|---|---|---|---|---|
| `D-new-NN` | `../drifts/D-new-NN.md` | HIGH / MEDIUM / LOW | in-chunk-plan → remediated | (if renegotiated, link the ADR) |

Rules:
- Every drift in the forward-scope inventory's `CH-NN` chunk MUST appear here.
- If the chunk discovers new drifts mid-flight (see §6 *mid-flight pause*), the new drifts are added to this table before chunk seal.
- Drift status transitions happen at chunk seal, not earlier. The lifecycle rules in [`drift-lifecycle.md`](./drift-lifecycle.md) govern permitted transitions.
- **Non-terminal drifts MUST cite explicit `M*-DEFERRED-NN` allocation** (added 2026-05-11 per CH-24 retro Row 4, cycle hex `5778bb77`; chunk-planner v13). Drifts left at `Status: discovered` / `scoped` whose `Impl chunk` field reads `TBD` / `TBD — likely M6+` / `TBD pending design` are non-compliant. At plan-draft time, the planner inspects every drift file the plan touches; any `TBD` marker triggers a P-DOCS or P-SEAL deliverable to promote it to an explicit `M<N>-DEFERRED-<NN>` allocation (cross-referencing the relevant forward-scope §M6+/M7+/M7b section). For NEW drift files filed by the chunk (mid-flight discovery), the planner MUST populate `Impl chunk` with an explicit allocation at file-creation time. Never write `TBD`. CH-24 retro housekeeping precedent: 1-line patch to `D-new-28`'s stale `CH-19 (+ M6 review)` → `M6-DEFERRED-01`.

#### §4.A — Deferred functionality (mandatory; added 2026-05-20 per CH-28 retro plan archive `chunk-decomposition-and-fork-framing-76e04080.md`)

For every drift / follow-up / `M*-DEFERRED-NN` marker the chunk files OR carries forward, this sub-section adds a **user-facing translation row** so the user can trace deferred work back to product capability:

| Drift ID | User-visible feature deferred | Product impact during deferral | Allocation chunk | Cross-chunk dep |
|---|---|---|---|---|
| `D-CH<NN>-FOLLOWUP-<MM>` | <one-phrase product feature name from the user's POV — NOT the technical layer> | <one-sentence description of what the user observes in the deferred state vs the final state, e.g., "agents sharing a template see independent profile changes instead of synchronized changes"> | `CH-<NN+M>` (or `M<N>-DEFERRED-<NN>` if cross-milestone) | <upstream chunks that must land first for the allocation chunk to ship> |

**Rules**:
- Every NEW drift filed by the chunk MUST have a row here, NOT just the §4 transition table above.
- "User-visible feature deferred" cell MUST avoid technical jargon (no "trait method", "edge variant", "schema field"); it names what the END USER would describe wanting to do.
- "Product impact during deferral" cell MUST state behavior in v0 vs final state in language a non-technical user understands.
- Cross-ref to `docs/specs/v0/feature-inventory.md` — the deferred catalogue index in §3 of that doc must be kept in sync.

**Why this sub-section**: CH-28 retro surfaced that `D-CH28-FOLLOWUP-01` (listener template-tier fan-out → M6-DEFERRED-04 / CH-36) framed the deferral architecturally ("5-step traversal not implemented") but did NOT name the user-visible product impact ("agents sharing a template see independent profile changes vs synchronized"). The user reading the drift could not trace the deferred work back to a product capability or assess v0 vs final state. §4.A closes that gap structurally.

### §5 — ADRs drafted

Each chunk that makes a non-trivial architectural or convention decision MUST draft an ADR:

- **ADR number assignment** (Q6 decision): At chunk-plan drafting time, run `ls baby-phi/docs/specs/v0/implementation/*/decisions/*.md 2>/dev/null | xargs -I{} basename {} .md | grep -oE "ADR-[0-9]+" | sort -u | tail -5` to see the current highest ADR. Pick the next free sequential number. Record it in the chunk plan. Never allocate ADR numbers opportunistically mid-chunk.
- **Draft status at plan draft** — `Proposed`.
- **Flip to `Accepted`** — at the phase close that ships the decision, or at chunk seal if the ADR covers the chunk holistically.

For each ADR list: number, title, drafted-at-phase, decision-summary (one line), expected flip-to-Accepted phase.

**ADR-body checklist (v2026-05-04 per CH-13 retrospective, cycle hex `d4fe1b7c`):** every ADR drafted by a chunk MUST include:

1. **§"Forks" header with explicit user-lock outcome.** Direct-approval cycle: `Forks (all planner-recommended at chunk-open; user-locked at plan approval to F1.A / F2.A / F3.A)`. Divergent cycle: `Forks (F1 user-locked to F1.B at plan approval — diverges from planner recommendation F1.A; F2 / F3 at planner-recommendation)`. The header MUST capture the lock-state, not just the planner-recommendation.
2. **§"Cross-references" with all 4 categories.** (a) originating concept-doc + section + line range; (b) closed drift(s) by ID; (c) prior ADRs cited as precedent; (d) **forward-scope row** that scoped this chunk (path + line range). The forward-scope cross-reference is mandatory — the ADR is the bridge between concept-doc and code, and must trace back to the chunk-source. Closes CH-13 Audit B Trivial-1L gaps F-AUDB-1 + F-AUDB-2.
3. **Pre-existing-behaviour preservation note** (added 2026-05-08 per CH-14 retro Row 10, cycle hex `5803bb94`; **formula relaxed 2026-05-10 per CH-19 retro Row 1, cycle hex `2c520ba7`** — 3 documented variations now permitted for sub-decisions where the strict "shipped at M5/P<n> close (date YYYY-MM-DD)" formula doesn't fit): if your ADR sub-decision changes audit-event emission semantics, AR/Grant state-transition behaviour, or any other prior runtime invariant, **document the pre-existing behaviour explicitly** in the relevant sub-decision body. CH-14 ADR-0053 §D53.7 needed mid-cycle clarification: the level-0 adoption AR's companion `auth_request.revoked` event from `revoke_ar` continues to be discarded — pre-CH-14 behaviour preserved verbatim. Future ADRs reading the historical record should be able to identify which pre-existing behaviours are now relied-upon invariants vs which were intentionally changed.

**Format (strict)**: a sentence inside the sub-decision like "Pre-existing behaviour preserved: `<X>` (see `<file:line>` for the historical implementation; CH-NN does not change this)" or "Behaviour changed: `<X>` (was `<old>` per `<prior ADR §>`; CH-NN flips to `<new>` because `<reason>`)".

**Format (3 permitted variations per CH-19 retro Row 1)** — when the strict formula doesn't fit a sub-decision shape, use one of these documented variations (spirit preserved: identify what's pre-existing vs new):
- **(a) Deferred-scope variation**: when the sub-decision ratifies a deferral (e.g., M6+ implementation surface), use *"Pre-existing scaffold preserved: <X> (deferred-marker chunk-assignment unchanged at <M6-target>; CH-NN ratifies the deferral, does not implement)"*. CH-19 ADR-0057 §D57.8 (Inbox/Outbox M6-DEFERRED-02) + §D57.9 (token-economy M6-or-M7-DEFERRED) precedent.
- **(b) Multi-milestone-pattern variation**: when the sub-decision ratifies a pattern that doesn't have a single shipped-at date (e.g., a convention emerged across multiple M-tags), use *"Pre-existing implementation preserved: <X> (pattern emerged across <M1-M5 tags>; CH-NN ratifies the convention as canonical, does not change shipped code)"*. CH-19 ADR-0057 §D57.10 (Org/Project template-as-config refresh) precedent.
- **(c) Never-shipped-yet variation**: when the sub-decision ratifies the absence of a feature (e.g., "no new web tests, defer to Playwright"), use *"Pre-existing absence preserved: <X> (no shipped behaviour to change; CH-NN ratifies the deferral as canonical convention)"*. CH-19 ADR-0057 §D57.6 (D7.5 no new web tests) precedent.

**Spirit-of-rule check**: regardless of strict-vs-variation, every Pre-existing-behaviour preservation note MUST identify (i) what was the case before this chunk, (ii) whether this chunk changes it, (iii) where the historical evidence lives. The 3 variations don't loosen the spirit — they accommodate sub-decision shapes that lack a single shipped-at date.

**ADR top-level section enumeration (v17 added 2026-05-17 per CH-02b-i-phi retro Row 2, cycle hex `57b20bda`; HIGH + mid-cycle confirmed)** — plan §5 MUST explicitly list every ADR top-level section the implementer authors. The canonical i-phi/baby-phi ADR shape is:

1. `## Forks` (header table; Direct-approval vs Divergent form)
2. `## Context` (chunk-graph + forward-scope citations)
3. `## Sub-decisions` (one `### §D<N>.<M>` per fork + supporting decisions; each ends with the pre-existing-behaviour preservation note)
4. `## Cross-references` (4 categories: (a) concept-doc + line range; (b) closed drifts; (c) prior ADRs as precedent; (d) forward-scope row)
5. `## Consequences` (one `### For CH-<NN>` subsection per downstream chunk the ADR forward-routes to)
6. **`## Revisit triggers`** — 3-7 bullets listing conditions that would warrant revisiting the ADR (each cites a specific §D<N>.<M> that would need re-opening)
7. `## Verification` (commands to replay verification)

The plan §5 ADR enumeration MUST name these 7 sections explicitly so the implementer doesn't omit them. CH-02b precedent: ADR-0003 first-iter omitted §"Revisit triggers" entirely + §"Consequences ### For CH-06" — Audit-B iter 1 caught both; 2 Trivial-multi orchestrator patches landed the missing content. v17 codifies the enumeration so future cycles don't re-incur the gap.

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
- **Buffer ceiling widened to ×1.30 on Artifact-C-cascade chunks** (added 2026-05-09 per CH-17 retro Row 5, cycle hex `40c4d759`): when plan §8 lower-bound test count is contingent on an Artifact C cascade (assertion-edit fan-out — e.g., a struct-field-add cascading through ~12 fixture-assertion sites), buffer ceiling widens from ×1.20 to **×1.30**. Rationale: assertion-edit cascades inflate test-amendment counts mechanically + integration-test additions stack on top. CH-17 ran [1484, 1488] plan band → [1488, 1495] audit-prompt band → 1491 actual; the ×1.30 ceiling on Artifact-C cycles avoids future gate-discrepancy noise. **Mid-cycle scope-expansion rule (added same row):** when user-driven scope expansion lands at gate-3 (e.g., integration tests added per user-locked re-dispatch), the implementer MUST re-stamp plan §8 band immediately + log in implementation report; auditor-prompt-side band-widening is a band-aid, not a primary fix.
- **MUST-SHIP / MAY-COVER test enumeration split** (added 2026-05-09 per CH-17 retro Row 6): split §8 test enumeration into two named sub-sections:
  - **`MUST-SHIP`** — named test files that MUST exist as files-on-disk by chunk-seal (e.g., `server/tests/sse_live_stream_test.rs`, `cli/tests/<feature>_test.rs`, `acceptance_<feature>::<test_name>`). These are the planner's binding contract about what the chunk delivers.
  - **`MAY-COVER`** — band-floor surrogate tests (unit-test additions in existing modules, builder unit tests, audit-event builder tests). Count toward the test-count target but are not MUST-SHIP.
  - The chunk-implementer prompt (v7+) MUST treat MUST-SHIP-absent-at-chunk-seal as a chunk-seal **blocker** (do NOT silently substitute MAY-COVER coverage). CH-17 first implementer-spawn dropped MUST-SHIP `sse_live_stream_test.rs` per band-floor surrogate substitution; user-driven gate-3 re-dispatch closed the gap with +4h scope.
- **Layer breakdown** — unit / integration / acceptance / e2e counts.
- **Named test files** — list the new test file paths (per MUST-SHIP / MAY-COVER split above).
- **Named expected-still-green tests** — anything fragile that the chunk's changes risk breaking; re-verified at chunk close. **Grep-verify against actual repo state (v17 added 2026-05-17 per CH-02b-i-phi retro Row 3, cycle hex `57b20bda`)**: before emitting this list, run `grep -hE "^fn (test_|smoke_)" <PROJECT_ROOT>/tests/*.rs` (or equivalent for the project's test-naming convention) and use the actual fn names verbatim. Do NOT paraphrase from prior-cycle plan or retrospective text — those may have drifted. CH-02b precedent: plan §8 listed CH-02a carry-forward test names that didn't match actual fn names (`test_daemon_start_and_programmatic_shutdown_returns_within_5s` vs actual `test_daemon_starts_and_shuts_down_via_programmatic_shutdown`); no harm, but wording-drift; codified at v17 for cheap insurance.

### §9 — Pre-chunk gate

The reading list + invariant check the drafter walks BEFORE `ExitPlanMode` is invoked.

**Reading list (mandatory):**
1. Every concept doc cited in §2.
2. Every drift file cited in §4.
3. Every prior-chunk plan cited in §6 (under `docs/specs/plan/build/`).
4. [`forward-scope/remaining-scope-post-m5-p7-22035b2a.md`](../../../../plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md) §5 + §7 (the chunk row + binding Q&A decisions).
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

**P-seal cycle-index row (added 2026-05-09 per CH-17 retro Row 4, cycle hex `40c4d759`):** chunk-implementer's chunk-seal paperwork explicitly includes inserting a row for this cycle into `/root/projects/phi/baby-phi/docs/specs/plan/build/_cycle-index.md` "Active cycles" table. Verification: `grep -n <cycle-hex> /root/projects/phi/baby-phi/docs/specs/plan/build/_cycle-index.md` must return ≥ 1 hit at chunk-seal. NOT an implicit follow-on of P4 — explicit MANDATORY paperwork item. Failure-mode CH-17 hit: implementer-spawn forks attention to mid-cycle scope expansion + drops the cycle-index row; orchestrator-applied Trivial-1L closed inline.

**Cargo-clean discipline operates at TWO placements** (refined 2026-05-10 per CH-18 retro Row 1, cycle hex `c77937bc`, USER DIRECTIVE: *"tests should be cleaned up immediately after the run as it may block future tests"*; supersedes the previous gate-5-close-only placement from CH-17 retro Row 1):

**(1) Immediate-post-test cleanup (NEW per CH-18)**: AFTER each `cargo test --workspace` invocation across the cycle (sub-agent audits A + B per chunk-auditor v7; orchestrator gate-4 final test; retrospector permissions-audit script per chunk-retrospector v4), the invoker MUST run `cargo clean --manifest-path /root/projects/phi/baby-phi/Cargo.toml` BEFORE issuing the next cargo invocation. Per-invocation cleanup ensures the next invocation starts from clean target/ and prevents accumulation across multiple test runs within a single cycle. CH-18 evidence: 2 duplicate cargo-test workspace background runs accumulated target/ to 146 GB → 100% disk → 1h24m hung. Capture disk-reclaim metric per invocation in cycle-audit §7 metrics row "Disk reclaimed per cargo-clean invocation".

**(2) Gate-5-close final cleanup (CH-17 retro Row 1, USER REQUESTED 2026-05-09, cycle hex `40c4d759`)**: the **orchestrator** runs `cargo clean` as the **last action of gate-5 close**, AFTER standards updates landed + cycle-index row flipped to `retro-complete`, just before reporting cycle-complete to user. Trigger sequence:

1. Capture `du -sh /root/projects/phi/baby-phi/target` BEFORE clean.
2. Run `/root/rust-env/cargo/bin/cargo clean --manifest-path /root/projects/phi/baby-phi/Cargo.toml`.
3. Capture `df -h /root | head -3` AFTER clean.
4. Log disk reclaimed in the cycle-audit's §7 metrics row "Disk reclaimed at gate-5 close".

**Rationale for TWO placements (not just one)**: CH-17 retro Row 1's gate-5-close-only placement was insufficient because target/ can balloon DURING gate-4 if multiple test invocations run concurrently or sequentially without cleanup (CH-18 evidence). Per-invocation cleanup at placement (1) prevents within-cycle disk-pressure incidents; gate-5 final close at placement (2) ensures clean state at chunk release. Both are now mandatory.

NOT the chunk-implementer's responsibility for placement (2) — chunk-seal happens before gate-4 MUST-RUN + gate-5 retro, both of which rebuild target/. Placement (1) IS chunk-implementer's responsibility per chunk-implementer v8 (added 2026-05-10 per CH-18 retro Row 1). chunk-auditor v7 + chunk-retrospector v4 both apply placement (1).

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
