<!-- Last verified: 2026-05-23 by Claude Code (CH-28b P4 — filed at chunk-seal per F3.a planner-rec lock + ADR-0064 §D64.5: phi-core 0.8.0 ships Composition I as an opt-in tree-structured braking layer (`revert_to_state` tool + `RevertApplied` event + `RevertRenderPolicy`). baby-phi opts OUT at session launch via `AgentLoopConfig.revert_pending: None` carrier-fix at `launch.rs:577` (P2 deliverable). This drift tracks the deferred adoption work (the 4 prerequisites itemised in §"Remediation scope") to a future M6+ FUNCTIONAL chunk. Allocation: `M6+-FUTURE-COMPOSITION-I-ADOPTION` placeholder (no specific CH-NN slot reserved at CH-28b close; future planning session decides). Cycle hex `d5b776ac`.) -->

# D-PHICORE-08-FOLLOWUP-01 — Composition I adoption (deferred to future M6+ FUNCTIONAL chunk)

## Identification
- **ID**: D-PHICORE-08-FOLLOWUP-01
- **Phase of origin**: CH-28b P4 chunk-seal (2026-05-23) — filed per F3.a planner-rec lock at gate-1.5 + ADR-0064 §D64.5 deferral decision.
- **Discovery source**: phi-core 0.8.0 breaking-change release (2026-05-23) — Composition I shipped as opt-in tree-structured braking layer with `BasicAgent::with_revert_tool()` builder + `RevertApplied` event + `RevertRenderPolicy` tuning surface. i-phi project's pre-spawn diagnostic (`i-phi/docs/v0/proposal/plan/build/phi-core-revert-tool-27c894f6/plan.md` §DIAGNOSTIC SECTION A) predicted baby-phi adoption is a separate feature-work chunk distinct from absorbing the 0.8.0 compile-restoration surface.
- **Date discovered**: 2026-05-23
- **Status**: `discovered`
- **Bucket**: B — follow-on engine-scope widening (architectural — opt-in feature adoption + agent-skill teaching across the agent prompt surface).
- **Severity**: LOW
- **Tags**: `phi-core-0.8`, `composition-i`, `revert-tool`, `braking-layer`, `m6-future`, `agent-skill`, `opt-in-feature`
- **Blocks**: nothing within CH-28b; the carrier-fix at `launch.rs:577 revert_pending: None` + explicit match-arm at `cli/agent.rs:668 AgentEvent::RevertApplied { .. } => {}` together close the breaking-change absorption. Composition I adoption is the NEXT feature-work concern.
- **Blocked-by**: nothing — CH-28b ships the dependency baseline shift (phi-core 0.7.1 → 0.8.x) which unblocks adoption. All adoption prerequisites are inside baby-phi's source tree + skill/prompt surface; no further phi-core work needed.
- **Closing chunk**: **M6+-FUTURE-COMPOSITION-I-ADOPTION** (placeholder; per chunk-planner v13 non-terminal-drift rule the placeholder names an explicit-named-future-allocation rather than `TBD`; no specific CH-NN slot reserved at CH-28b close; future planning session decides whether adoption lands as a dedicated FUNCTIONAL chunk OR as a feature-work axis bundled into an existing M6+ FUNCTIONAL chunk).

## Concept alignment
- **Concept doc(s)**: [`phi-core/docs/concepts/concept-brake.md`](../../../../../../../phi-core/docs/concepts/concept-brake.md) §5 Composition I — design source-of-truth for the tree-structured composition + revert-to-state semantics + `RevertCategory` enum (failure/tangent/completion/step-summary). [`phi-core/CHANGELOG.md`](../../../../../../../phi-core/CHANGELOG.md) §[0.8.0] lines 9–94 — release notes enumerating the 3 breaking-changes + 7 added items + Composition I activation form (`BasicAgent::with_revert_tool()`).
- **Contradiction at CH-28b close**: NONE at the user-facing surfaces. The pre-0.8.0 monotonically-growing-context posture (compaction via existing `BlockCompactionStrategy` + `compact_messages` is the only relief) is preserved verbatim; agents continue to operate identically. The Composition I surface is opt-in upstream + remains opt-out in baby-phi by virtue of `AgentLoopConfig.revert_pending: None`.
- **Classification**: `feature-deferral` (opt-in feature; CH-28b absorbs the compile-restoration breaking-change surface only + defers the feature-work adoption per F3.a lock at gate-1.5).
- **phi-core leverage status**: `direct-reuse-available` — adoption brings new `use phi_core::tools::revert::{RevertTool, RevertRequest}` + `use phi_core::types::node_tag::{NodeId, NodeTag, TagKind, RevertCategory, RevertRenderPolicy}` imports into baby-phi at the future adoption chunk. NO new types should be created in baby-phi; the feature-work is wiring + skill-prompt-authoring, not type-system extension.

## Plan vs. reality
- **Plan §1.3 + §4.A (CH-28b iter-1) said (F3.a LOCKED at gate-1.5)**: defer Composition I adoption to future M6+ FUNCTIONAL chunk via this drift; CH-28b ships the baseline shift + the 1-line carrier-fix + the explicit match-arm + the NEW drift filing + the feature-inventory row only.
- **Reality at CH-28b chunk-seal**: matches plan exactly. The dependency baseline flipped from `phi-core = "0.7.1"` to `phi-core = "0.8"` (Cargo.toml:17); Cargo.lock regenerated against phi-core 0.8.0; `AgentLoopConfig.revert_pending: None` carrier-fix at `launch.rs:577` paralleling CH-25 P-SEAL `response_format` precedent; explicit `AgentEvent::RevertApplied { .. } => {}` arm at `cli/agent.rs:668` for future-reader discoverability; `session_recorder.rs:166-170` `matches!()` site NOT modified (per D4 scope-narrow + ADR-0064 §D64.6); workspace test count preserved at 1599 / 0 failed; phi-core import baseline preserved at 57.
- **Root cause**: phi-core 0.8.0 ships Composition I as an opt-in feature behind `BasicAgent::with_revert_tool()`. Enabling it in baby-phi requires:
  - A code-tier wire-up at the BasicAgent construction site (TBD whether the natural site is `server/src/platform/sessions/launch.rs` or `cli/src/commands/agent.rs` BasicAgent construction or a NEW factory).
  - Optionally surfacing the `RevertApplied` event in `domain::BabyPhiSessionRecorder` for governance audit visibility (currently the explicit arm at `cli/agent.rs:668` is a no-op; the SessionRecorder's hash-chain audit-event writer at `domain/src/session_recorder.rs:166-170` is intentionally untouched per §D64.6).
  - A skill / prompt-authoring axis teaching the agent the revert discipline (the 4 `RevertCategory` cases failure / tangent / completion / step-summary) — this is the load-bearing feature-work axis; without it, enabling the tool yields a no-op (the agent has no reason to call it).
  - Tuning `RevertRenderPolicy` defaults (lesson_window_turns + lesson_window_count) for baby-phi's compaction posture; phi-core ships sensible defaults but the optimum tuning is consumer-specific.
- **Why deferred at CH-28b**: opt-in feature work is fundamentally different from compile-restoration carrier-fix work; bundling them into a single chunk would obscure the bookkeeping. The carrier-fix (this chunk) absorbs the breaking-change surface without behavior delta; the adoption (future chunk) introduces a behavior delta that warrants its own audit + skill-authoring scope.

## Where visible in code
- **Files** (CH-28b close baseline; will expand at adoption-chunk):
  - `modules/crates/server/src/platform/sessions/launch.rs:577` — `revert_pending: None,` carrier-fix preserves opt-out posture; future adoption flips this to a `Some(Arc::new(Mutex::new(Vec::new())))` or equivalent semantics per `BasicAgent::with_revert_tool()` builder convention.
  - `modules/crates/cli/src/commands/agent.rs:668` — explicit `AgentEvent::RevertApplied { .. } => {}` arm preserves discoverability; future adoption replaces the no-op body with audit-event emission + render-policy filtering.
  - `modules/crates/domain/src/session_recorder.rs:166-170` — single-pattern `matches!()` site (matches `PhiCoreAgentEvent::AgentStart { session_id, .. }` ONLY); per §D64.6 NOT modified at CH-28b. Future adoption may extend the `matches!()` to a multi-pattern form OR replace with an exhaustive `match` if `RevertApplied` warrants governance visibility.
- **Grep for regression** (CH-28b close baseline vs future adoption-chunk close target):
  - `grep -n "revert_pending: None" modules/crates/server/src/platform/sessions/launch.rs` — CH-28b: 1 hit. Adoption-chunk target: 0 hits (flips to `Some(...)`).
  - `grep -nE "with_revert_tool|RevertRenderPolicy" modules/crates/` — CH-28b: 0 hits. Adoption-chunk target: ≥ 1 hit (the builder call site + the policy tuning site).
  - `grep -nE "use phi_core::tools::revert" modules/crates/` — CH-28b: 0 hits. Adoption-chunk target: ≥ 1 hit (the import line at the wire-up site).

## Remediation scope (estimate only)

The adoption work decomposes into 4 prerequisites (per i-phi diagnostic §DIAGNOSTIC SECTION A "Adoption guidance" and phi-core CHANGELOG §[0.8.0] activation guidance):

1. **`BasicAgent::with_revert_tool()` call site** — wire the builder call at baby-phi's BasicAgent construction path. Likely site: `server/src/platform/sessions/launch.rs` (where the `AgentLoopConfig` is built today — CH-28b's P2 site) OR `cli/src/commands/agent.rs` BasicAgent construction OR a NEW factory abstracting both. The choice depends on whether the adoption-chunk extends both code paths or carves a single shared factory. Estimated effort: ~0.2-0.5 ed (~10-30 LOC depending on factory shape).

2. **Optional `RevertApplied` event surfacing in `BabyPhiSessionRecorder`** — extend `session_recorder.rs:166-170` to surface `RevertApplied` events (currently the `matches!()` block guards a single-pattern recognition; future adoption could promote to multi-pattern OR a separate exhaustive-match block). The choice depends on whether baby-phi wants per-revert audit-event emission for governance visibility (likely YES given baby-phi's audit-hash-chain posture). Estimated effort: ~0.3-0.5 ed (~20-50 LOC + matching audit-event-dictionary additions).

3. **Skill / prompt teaching the agent the revert discipline** — author or extend an existing baby-phi skill (TBD whether under `agents/` or a new `skills/revert-discipline/` path) that teaches the agent the 4 `RevertCategory` cases (failure / tangent / completion / step-summary) + when to call `revert_to_state` between turns. This is the load-bearing adoption axis — without it, enabling the tool yields a no-op (the agent has no instructional context to invoke it). Estimated effort: ~0.5-1 ed (~100-300 LOC of skill prose + acceptance scenario coverage; the SKILL.md authoring is the heavy lift).

4. **Tune `RevertRenderPolicy` defaults** — set `lesson_window_turns` + `lesson_window_count` defaults appropriate to baby-phi's compaction posture (existing `BlockCompactionStrategy` + `compact_messages` + the new tree-structured revert layer interact). phi-core ships sensible defaults; baby-phi-specific tuning requires testing across the agent flows in the adoption chunk. Estimated effort: ~0.2-0.3 ed (~5-15 LOC + acceptance scenarios validating the chosen tuning).

**Aggregate effort estimate**: ~1.2-2.3 ed for a dedicated adoption chunk; ~0.5-1 ed if bundled into an existing M6+ FUNCTIONAL chunk that already touches BasicAgent construction (e.g., a future memory or supervisor body chunk).

**Implementation chunk**: **M6+-FUTURE-COMPOSITION-I-ADOPTION** (placeholder; no CH-NN slot reserved at CH-28b close — future planning session decides). Chunk-planner v13 non-terminal-drift rule satisfied via the explicit-named-placeholder (NOT `TBD`).

**Dependencies on other drifts**: none. CH-28b's baseline shift unblocks adoption; no upstream drift blocks this.

**Risk to concept alignment if deferred further**: LOW. Composition I is an opt-in feature; deferring it further preserves the pre-0.8.0 monotonically-growing-context behavior. The only opportunity cost is that agents continue to accumulate dead conversation branches between turns (failed/finished branches stay in context until compaction fires). After adoption, agents call `revert_to_state` between turns to drop dead branches; context stays lean; compaction fires less often. No user-visible degradation if deferred; just a missed efficiency optimization.

## Why filed as a follow-on drift (NOT in-CH-28b expansion)

User routing decision (codified at plan-mode session 2026-05-23 + locked at gate-1.5 ratification per F3.a):
- Architectural design ~1.2-2.3 ed scoped, requiring code-tier wire-up + audit-event-surfacing + skill-authoring + render-policy-tuning.
- NOT load-bearing for CH-28b's TECHNICAL-PREREQUISITE invariants (the baseline shift + compile restoration are complete with the carrier-fix + explicit match-arm).
- Intersects M6+ FUNCTIONAL feature surface (the skill-authoring axis especially is feature-work distinct from compile-restoration).

Per outer CLAUDE.md gate-5 in-M5-carve-out-vs-M6-DEFERRED routing criteria (CH-26 retro Row 6), this matches the M6-DEFERRED pattern adapted for the M6+ era — NOT load-bearing for current chunk's invariants + intersects M6+ feature surface + > ~10-line scoped. Routed to M6+-FUTURE-COMPOSITION-I-ADOPTION placeholder rather than in-chunk carve-out.

## Lifecycle history
- 2026-05-23 — `discovered` — filed at CH-28b P4 per F3.a lock at gate-1.5 + ADR-0064 §D64.5; M6+-FUTURE-COMPOSITION-I-ADOPTION placeholder allocation per chunk-planner v13.

## Cross-references
- [`ADR-0064`](../decisions/0064-phi-core-08-absorption.md) §D64.5 — Composition I adoption deferred to future M6+ FUNCTIONAL chunk via this drift (Decision body + Pre-existing-absence preserved note).
- [`ADR-0064`](../decisions/0064-phi-core-08-absorption.md) §D64.6 — `AgentEvent::RevertApplied { .. } => {}` arm scope (cli/agent.rs ships explicit arm; session_recorder.rs `matches!()` site NOT modified) — explains why this drift's prerequisite #2 has an open design choice on the SessionRecorder surface.
- [`ADR-0064`](../decisions/0064-phi-core-08-absorption.md) §"Consequences ### For future M6+ Composition I adoption chunk" — inherited requirement amendment + adoption-chunk planning hints.
- [`phi-core/CHANGELOG.md`](../../../../../../../phi-core/CHANGELOG.md) §[0.8.0] lines 9–94 — Composition I release notes + activation form `BasicAgent::with_revert_tool()`.
- [`phi-core/docs/concepts/concept-brake.md`](../../../../../../../phi-core/docs/concepts/concept-brake.md) §5 — Composition I design source-of-truth.
- [`i-phi diagnostic`](../../../../../../../i-phi/docs/v0/proposal/plan/build/phi-core-revert-tool-27c894f6/plan.md) §DIAGNOSTIC SECTION A — baby-phi adoption guidance (4 prerequisites enumeration).
- Plan archive: [`plan/build/ch-28b-phi-core-08-absorption-d5b776ac/plan.md`](../../../../plan/build/ch-28b-phi-core-08-absorption-d5b776ac/plan.md) §1 + §3 + §4.A + §5 §D64.5 — CH-28b plan body documenting the deferral.
- [`feature-inventory.md`](../../../feature-inventory.md) §3 D-PHICORE-08-FOLLOWUP-01 row — product-trajectory translation of this drift's deferred-vs-final state.
