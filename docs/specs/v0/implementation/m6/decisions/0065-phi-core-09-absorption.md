<!-- Last verified: 2026-05-24 by Claude Code (CH-28c-40214078 P3 final, Accepted; 5 sub-decisions across 3 forks: F1.b CH-28c interstitial (RESOLVED at prerequisite commit `b0ed0bb`) + F2.a `phi-core = "0.9"` semver-range + F3.a NEW drift `D-PHICORE-09-FOLLOWUP-01` at canonical `m6/drifts/`; absorbs phi-core 0.9.0 breaking-change release (per-turn debug capture + async-trait migration baseline shift); ZERO carrier-fix work (AgentLoopConfig gains no new fields per CHANGELOG line 76 — even smaller surface than CH-28b which carried 1 carrier-fix); explicit `AgentEvent::TurnRequest { .. } => {}` arm at `cli/agent.rs` between CH-28b RevertApplied arm (line 674) and existing wildcard for future-reader discoverability; `session_recorder.rs:166-170` `matches!()` single-pattern site intentionally NOT modified per §D65.3 scope-narrow (paralleling CH-28b §D64.6); opt-in posture preserved — per-turn debug capture NOT enabled at session launch; deferred adoption tracked via `D-PHICORE-09-FOLLOWUP-01` to future M7+ FUNCTIONAL chunk `M7+-FUTURE-PER-TURN-DEBUG-CAPTURE-ADOPTION` placeholder.) -->

# ADR-0065 — phi-core 0.9.0 absorption (per-turn debug capture + async-trait migration; baseline shift)

**Status: Accepted**

**Authors**: Claude Code (orchestrator + chunk-planner v28 + chunk-implementer v18)

**Chunk**: CH-28c (cycle hex `40214078`)

**Milestone**: M6 (foundation tier; second consecutive interstitial "CH-NNc" chunk after CH-28b; TECHNICAL-PREREQUISITE)

**Decision-summary** (one line): shift baby-phi's phi-core dependency baseline from 0.8.x to 0.9.x via semver-range (`phi-core = "0.9"`) to absorb the per-turn debug capture + async-trait migration breaking-change release; add explicit `AgentEvent::TurnRequest { .. } => {}` arm at `cli/agent.rs` between CH-28b RevertApplied arm and existing wildcard for future-reader discoverability; intentionally NOT modify `session_recorder.rs:166-170` (single-pattern `matches!()` site — adding the arm would change boolean semantics; paralleling CH-28b §D64.6); defer per-turn debug capture adoption (`SessionRecorderConfig::capture_turn_requests: true` flag flip + observability surfacing + JSON-size policy + operator inspection UI) to future M7+ FUNCTIONAL chunk via NEW drift `D-PHICORE-09-FOLLOWUP-01`; async-trait migration is a no-op consumer-side (baby-phi has ZERO `BlockCompactionStrategy` / `InputFilter` impls + ZERO `AgentLoopConfig` lifecycle Fn closures).

---

## Forks

| Fork | Locked option | Path | Pros | Cons | Status |
|---|---|---|---|---|---|
| **F1** (TECHNICAL) | **F1.b RESOLVED at prerequisite commit `b0ed0bb` 2026-05-24** | CH-28c interstitial chunk numbering (NOT renumbering CH-29..CH-38; NOT dot-suffix CH-28b.5) | ~5 doc-surface edits at the prerequisite commit; surgical scope; second activation of CH-NNa/b suffix precedent established at CH-28b; validates durability under repeated activation | Suffix depth advances from CH-NNa → CH-NNc (still alphabetic; no novelty cost beyond CH-28b's first-of-kind) | **LOCKED RESOLVED** at chunk-initiate Phase 1 Option A (user-locked at plan-mode session 2026-05-24); not a planner-rec divergence — F1.b was the user's locked choice at the prerequisite-commit phase, ratified at gate-1.5 rubber-stamp |
| **F2** (TECHNICAL) | **F2.a planner-rec rubber-stamp at gate-1.5** | `phi-core = "0.9"` (semver-range; auto-selects highest 0.9.x patch on next `cargo update`) | Idiomatic Cargo form for "trust semver-compatible updates"; no Cargo.toml commit needed per phi-core patch; Cargo.lock pinning provides reproducibility within a given commit; matches the form ratified at CH-28b §D64.1 (single-form-precedent across consecutive absorption chunks) | Requires team trust in phi-core's semver discipline (mitigated by phi-core being a sibling project under the same workspace) | **LOCKED planner-rec** at gate-1.5 |
| **F3** (BOUNDED USER-VISIBLE) | **F3.a planner-rec rubber-stamp at gate-1.5** | File NEW drift at baby-phi's canonical drift directory `m6/drifts/D-PHICORE-09-FOLLOWUP-01-per-turn-debug-capture-adoption.md` (parallel to existing `D-PHICORE-08-FOLLOWUP-01` row precedent) | Canonical drift-lifecycle discipline; rich blocked-by / closing-chunk / discovery-source metadata; feature-inventory.md §3 row gives the user-visible v0-vs-final translation; per chunk-planner v13 non-terminal-drift rule the placeholder `M7+-FUTURE-PER-TURN-DEBUG-CAPTURE-ADOPTION` satisfies explicit-named-allocation requirement | One small new file added (~150 LOC drift body) | **LOCKED planner-rec** at gate-1.5 |

**Cross-cycle divergence pattern**: 0-of-3 user-DIVERGENT forks at gate-1 for CH-28c. F1 was RESOLVED at prerequisite-commit phase before plan-spawn (out-of-band user lock per chunk-initiate Option A); F2 + F3 are planner-rec rubber-stamps at gate-1.5. Cumulative cross-cycle divergent forks for baby-phi remains at 14-of-19 (~74%) — CH-28c does not advance the count (no divergence). Second consecutive cycle holding the count steady at 14-of-19 (CH-28b also did not advance).

---

## Context

**Why this chunk.** phi-core 0.9.0 shipped on 2026-05-24 (one day after 0.8.0) with a breaking-change release introducing TWO bundled surfaces:

1. **Per-turn debug capture** — opt-in observability feature via `SessionRecorderConfig::capture_turn_requests: bool` (default `false`). New types `BlockProvenance` / `ProvenanceRole` / `AnnotatedRequestPayload` carry the fully-assembled LLM wire payload (system prompt + post-`convert_to_llm()` `Vec<Message>` array + tool definitions + parallel-indexed per-block provenance) exactly once per turn via the new `AgentEvent::TurnRequest` variant. Opt-in persistence onto `Turn::request_payload: Option<AnnotatedRequestPayload>` (with `#[serde(default)]` for back-compat).

2. **Async-trait migration** — `BlockCompactionStrategy` + 9 of 11 `AgentLoopConfig` lifecycle Fns + `InputFilter::filter()` become async (the 2 tool-update hooks `BeforeToolExecutionUpdateFn` + `AfterToolExecutionUpdateFn` remain sync per CHANGELOG line 12 deferral).

baby-phi pinned `phi-core = "0.8"` at the CH-28b close (2026-05-23). The cumulative phi-core delta since the last baby-phi bump comprises both surfaces above. Pre-spawn verification confirmed ZERO compile breaks across all relevant axes per the i-phi diagnostic:

- `AgentEvent::TurnRequest` new variant — covered by existing `_ => {}` wildcard at `cli/agent.rs:675` + field-wildcard `..` in `session_recorder.rs:166-170` `matches!()` block.
- `LlmMessage.provenance_hint` new field — baby-phi uses `LlmMessage::new(...)` constructor at both call sites (`session_recorder.rs:483` + `launch.rs:578`); no struct-literal patterns to break.
- `BlockCompactionStrategy` → `#[async_trait]` — baby-phi has ZERO custom impls.
- 9 of 11 `AgentLoopConfig` lifecycle Fns become async — baby-phi sets ALL 9 hook fields to `None` at `launch.rs:542-552`; no closure construction sites.
- `InputFilter::filter()` → `async fn` — baby-phi has ZERO custom impls.
- `AgentLoopConfig` gains NO new fields (per CHANGELOG line 76) — zero carrier-fix work needed (smaller surface than CH-28b which carried 1 carrier-fix at `launch.rs:577 revert_pending: None`).

**Concept-doc precedence.** phi-core's CHANGELOG §[0.9.0] + the new `phi-core/docs/concepts/debugging.md` are the upstream concept docs that drive this absorption. baby-phi's concept docs accommodate the new phi-core types via the existing extensibility pattern — no concept-doc body changes required for the absorption itself; the deferred adoption work (tracked via `D-PHICORE-09-FOLLOWUP-01`) will introduce concept-doc additions at its closing chunk.

**Forward-scope reference.** `docs/specs/plan/forward-scope/m6-forward-scope-8b7a8bcd.md` §1 lines 69–79 (CH-28c narrative block) + §5 table line 336. The CH-28c row was inserted as a prerequisite commit at `b0ed0bb` 2026-05-24 BEFORE chunk-planner spawn per F1.b lock; chunk-planner consumed the row as input to the §1 deliverables enumeration.

**Downstream consumers.** ALL M6 chunks CH-29..CH-38 by virtue of the workspace compiling on the 0.9.x baseline. In particular: CH-29 (M6-DEFERRED-02 messaging substrate); CH-33 (first FUNCTIONAL M6 chunk consuming `phi_core::types::tool::AgentTool` for `read_inbox` + `send_message` tools — AgentTool surface unchanged at 0.8 → 0.9.0); future M7+ per-turn debug capture adoption chunk (the `D-PHICORE-09-FOLLOWUP-01` closing chunk); future M6+ Composition I adoption chunk (the `D-PHICORE-08-FOLLOWUP-01` closing chunk).

**i-phi diagnostic provenance.** `/root/projects/phi/i-phi/docs/v0/proposal/plan/build/phi-core-0.9.0/plan.md` §"baby-phi absorption" lines 267–273 predicted baby-phi absorption impact as LOW (0.5–1 ed; no `BlockCompactionStrategy` impls; exhaustive `AgentEvent` matches already wildcard-covered; no lifecycle-Fn closures at construction sites). Pre-spawn verification confirmed the diagnostic — actual surface even smaller (ZERO compile errors + 1 cosmetic match-arm + paperwork; CH-28c is 1 carrier-fix smaller than CH-28b).

---

## Sub-decisions

### §D65.1 — F2.a `phi-core = "0.9"` semver-range form chosen over F2.b exact-pin + F2.c semver-explicit

**Decision**: workspace `Cargo.toml:17` reads `phi-core = "0.9"` (NOT `"0.9.0"` exact-pin, NOT `"^0.9.0"` semver-explicit). Per Cargo's semver-resolution semantics, `"0.9"` matches any 0.9.x patch; `cargo update -p phi-core` selects the highest published 0.9.x patch at update time (today: 0.9.0). Future phi-core 0.9.x patches flow into baby-phi automatically on the next `cargo update` invocation without a Cargo.toml commit per patch. Cargo.lock pinning provides reproducibility within a given commit.

**Pre-existing-behaviour preservation note**: pre-existing scaffold preserved — workspace `Cargo.toml:17` already used the canonical semver-range form `phi-core = "0.8"` since CH-28b close 2026-05-23 (which itself paralleled the pre-existing 0.7.1 form). The shift to `"0.9"` continues the single-form-precedent established at CH-28b §D64.1 across consecutive absorption chunks. CH-28c is the second activation of the F2.a precedent, validating durability under repeated activation.

### §D65.2 — F1.b CH-28c interstitial chunk-numbering convention chosen over F1.a renumber + F1.c CH-28b.5 dot-suffix

**Decision**: the cycle is named `CH-28c` (interstitial suffix); the cycle folder is `ch-28c-phi-core-09-absorption-40214078`; the forward-scope row is inserted as a NEW third row of the Foundation tier between CH-28b and CH-29 (NOT renumbering CH-29..CH-38 to CH-30..CH-39). This is the second activation of the "CH-NNa/b/c" suffix convention established at CH-28b §D64.2. Going CH-28a → CH-28b → CH-28c validates that the suffix lineage extends naturally as additional interstitial absorption chunks arrive (future phi-core breaking-change releases during M6 will likely continue the CH-NNc/d/e lineage rather than renumber).

**Pre-existing-behaviour preservation note**: pre-existing scaffold preserved — the "CH-NNa/b" suffix convention was established at CH-28b §D64.2 as canonical for "interstitial chunks that absorb upstream breaking-changes without renumbering downstream cycles." CH-28c is the second activation of the convention, validating durability under repeated activation. The convention's repeatability (CH-NN → CH-NNa → CH-NNb → CH-NNc) is a structural property of the alphabetic suffix scheme.

### §D65.3 — F3.a explicit `TurnRequest` arm at cli/agent.rs only; session_recorder.rs single-pattern `matches!()` left unchanged

**Decision**: at `modules/crates/cli/src/commands/agent.rs` between the existing CH-28b `AgentEvent::RevertApplied { .. } => {}` arm (line 674) and the existing `_ => {}` wildcard arm (line 675), insert an explicit `AgentEvent::TurnRequest { .. } => {}` arm (with an inline 9-line CH-28c-citing comment block paralleling CH-28b §D64.6 RevertApplied comment block style). The arm body is a no-op (per-turn debug capture is NOT enabled at CH-28c close per §D65.4 deferral); the arm is cosmetic + signals 0.9.0 variant awareness for future readers + future-proofs the cli surface when per-turn debug capture adoption fires (at the adoption-chunk the no-op body becomes the audit-event-emission + payload-size-logging + optional-truncation-policy invocation surface).

**At `modules/crates/domain/src/session_recorder.rs:166-170`** (the `matches!()` block matching a SINGLE pattern `PhiCoreAgentEvent::AgentStart { session_id, .. }`), DO NOT add an explicit `TurnRequest` arm. Rationale (parallels CH-28b §D64.6 scope-narrow verbatim): the `matches!()` block returns a boolean (`true` if the event matches `AgentStart`, `false` otherwise); it is single-pattern + compile-time-exhaustive without modification (since `matches!()` already returns `false` for any non-matched variant including `TurnRequest`). Adding a `TurnRequest` arm to the single-pattern `matches!()` would change boolean semantics — the block would then return `true` for BOTH `AgentStart` AND `TurnRequest` events, which is semantically wrong (the block's intent is to recognize `AgentStart` specifically for session-id binding). The future per-turn debug capture adoption chunk MAY extend the `matches!()` to a multi-pattern form OR replace with an exhaustive `match` block if `TurnRequest` warrants governance visibility (see `D-PHICORE-09-FOLLOWUP-01` prerequisite #2).

**Pre-existing-behaviour preservation note**: pre-existing wildcard arm preserved — `cli/agent.rs:675` `_ => {}` already covers the new `TurnRequest` variant at compile-time (since `AgentEvent` remains `#[non_exhaustive]` from 0.8.0, the wildcard is required-or-permitted by the compiler); the explicit `TurnRequest { .. } => {}` arm added by CH-28c is cosmetic + does NOT shadow the wildcard semantically (the wildcard still catches any future variants beyond `TurnRequest` and `RevertApplied`). The `session_recorder.rs:166-170` `matches!()` site is preserved EXACTLY as-is — single-pattern boolean recognition continues to return `true` only for `PhiCoreAgentEvent::AgentStart`. The CH-28b §D64.6 scope-narrow precedent is the canonical pattern; CH-28c §D65.3 is the second application of the pattern, validating durability.

### §D65.4 — F3.a NEW drift `D-PHICORE-09-FOLLOWUP-01` filed for per-turn debug capture adoption deferral

**Decision**: per-turn debug capture adoption (the 4 prerequisites enumerated in the `D-PHICORE-09-FOLLOWUP-01` drift body — `SessionRecorderConfig::capture_turn_requests: true` flag flip + optional `TurnRequest` recorder surfacing in baby-phi's observability layer + JSON-size-implications policy + operator inspection UI / CLI surface) is deferred to a future M7+ FUNCTIONAL chunk. The allocation placeholder is `M7+-FUTURE-PER-TURN-DEBUG-CAPTURE-ADOPTION` (per chunk-planner v13 non-terminal-drift rule explicit-named-allocation requirement; NOT `TBD`). No specific CH-NN slot is reserved at CH-28c close — future planning session decides whether adoption lands as a dedicated FUNCTIONAL chunk OR bundles into an existing M7 observability chunk that already touches `SessionRecorder` or session-JSON serialization.

**Pre-existing-absence preserved (never-shipped-yet variant per chunk-planner v24 P-plan-2)**: no per-turn debug capture adoption surface has shipped in baby-phi (the feature is opt-in upstream + not yet enabled). The adoption is a NEW feature axis introduced by phi-core 0.9.0; baby-phi's pre-0.9.0 posture (no per-turn debug payload persistence; debugging unexpected LLM responses requires manual reconstruction from `Turn.input_messages`) is preserved by virtue of the `SessionRecorderConfig::default()` `capture_turn_requests: false` at `cli/src/commands/agent.rs:636`. CH-28c absorbs the type space without enabling the feature.

### §D65.5 — Async-trait migration is a no-op consumer-side; no adoption deferral needed

**Decision**: phi-core 0.9.0's async-trait migration (`BlockCompactionStrategy` + 9 of 11 `AgentLoopConfig` lifecycle Fns + `InputFilter::filter()` becoming async) is internal-only to phi-core's trait shapes. baby-phi has ZERO consumers of any of these surfaces:

- ZERO `impl phi_core::context::BlockCompactionStrategy for` lines anywhere under `modules/crates/`.
- ZERO `impl phi_core::*::InputFilter for` lines anywhere under `modules/crates/`.
- ZERO `AgentLoopConfig` lifecycle Fn closures at construction sites — `launch.rs:542-552` sets ALL 9 hook fields (`before_loop` / `after_loop` / `before_turn` / `after_turn` / `on_error` / `before_tool_execution` / `after_tool_execution` / `before_compaction_start` / `after_compaction_end`) to `None`.

Async-fication is therefore a no-op consumer-side at CH-28c — no compile breaks, no carrier-fix needed, no follow-on adoption work needed. No drift filed for async-trait migration.

**Pre-existing-behaviour preservation note**: pre-existing scaffold preserved — baby-phi's `launch.rs:542-552` AgentLoopConfig struct-literal already sets all 9 lifecycle hook fields to `None`. The 0.7.1 → 0.8 → 0.9 progression has preserved this `None`-everywhere posture across every absorption chunk. If a future baby-phi chunk introduces a custom `BlockCompactionStrategy` or `InputFilter` impl, OR adds a non-`None` closure to any of the 9 lifecycle hook fields, that chunk MUST use `#[async_trait]` at impl time per phi-core's 0.9.0 trait shape; revisit §D65.5 if such an impl materializes. The no-op nature of async-fication at CH-28c close does NOT warrant a drift — the feature is structurally absorbed without behavioral change.

---

## Cross-references

**(a) Concept doc + line range** (per CH-13 retro Row 1 ADR-structure discipline):
- [`phi-core/CHANGELOG.md`](../../../../../../../phi-core/CHANGELOG.md) §[0.9.0] lines 21–186 — release notes enumerating the 2 bundled surfaces (per-turn debug capture + async-trait migration) + Added items + Migration guidance.
- [`phi-core/docs/concepts/debugging.md`](../../../../../../../phi-core/docs/concepts/debugging.md) — per-turn debug capture design source-of-truth: opt-in flow via `SessionRecorderConfig::capture_turn_requests`, JSON-size implications, debugging recipe for reconstructing wire payloads.
- [`docs/specs/v0/concepts/phi-core-mapping.md`](../../../concepts/phi-core-mapping.md) full doc — phi-core type ↔ baby-phi consumer mapping table (no mapping-row edits required at CH-28c; new phi-core types `BlockProvenance`/`ProvenanceRole`/`AnnotatedRequestPayload` + new `AgentEvent::TurnRequest` variant + new `Turn::request_payload` field become available but NOT imported until adoption fires).

**(b) Closed drift(s) by ID**: NONE closed at CH-28c. NEW drift filed at chunk-seal: `D-PHICORE-09-FOLLOWUP-01-per-turn-debug-capture-adoption.md` (Status: `discovered`; Severity: LOW; Bucket: B; Closing chunk: `M7+-FUTURE-PER-TURN-DEBUG-CAPTURE-ADOPTION` placeholder).

**(c) Prior ADRs cited as precedent**:
- [`ADR-0064`](0064-phi-core-08-absorption.md) (CH-28b 0.8.0 absorption) — directly-parallel pattern; ADR-0065 mirrors the section shape (Forks / Context / Sub-decisions / Cross-references / Consequences / Revisit triggers / Verification) + sub-decision body conventions exactly. CH-28b's §D64.1 (semver-range form) ↔ §D65.1; §D64.2 (CH-NNa/b suffix convention) ↔ §D65.2; §D64.6 (explicit-arm scope-narrow) ↔ §D65.3; §D64.4 + §D64.5 (drift filing + feature-deferral) ↔ §D65.4. CH-28c carries ZERO carrier-fix (vs CH-28b's §D64.3 `revert_pending: None` carrier-fix) because phi-core 0.9.0's `AgentLoopConfig` gains no new fields.
- [`ADR-0059`](../../m5_2/decisions/0059-recent-sessions-api-surface-flip.md) §D59.2 (CH-25 P-SEAL `response_format` carrier-fix precedent) — mentioned as inherited-pattern even though CH-28c ships zero carrier-fix; CH-25 + CH-28b are the two prior cycles that carried `AgentLoopConfig` carrier-fixes, establishing the structural pattern (minimal 1-line additive field with semantic-default = opt-out + inline CH-NN-citing comment block). CH-28c benefits from the pattern's existence (the precedent comment-block style transfers to the explicit-arm comment block at §D65.3) without invoking it as carrier-fix.
- [`ADR-0063`](0063-agent-profile-cardinality-n-to-1.md) §D63.3 — CH-28 forward-scope-row-as-prerequisite-commit precedent. CH-28c's F1.b RESOLVED at prerequisite commit `b0ed0bb` follows the same chunk-initiate Option A pattern (user-locked at plan-mode session; forward-scope row inserted BEFORE chunk-planner spawn; chunk-planner consumes the row as input).

**(d) Forward-scope row**: [`m6-forward-scope-8b7a8bcd.md`](../../../../plan/forward-scope/m6-forward-scope-8b7a8bcd.md) §1 lines 69–79 (CH-28c narrative block) + §5 table line 336 (CH-28c summary table row). The row was authored as a prerequisite commit at `b0ed0bb` 2026-05-24 BEFORE chunk-planner spawn per F1.b lock + chunk-initiate Option A.

---

## Consequences

### For CH-29 (next M6 chunk)

CH-29 (next M6 chunk per forward-scope §5; M6-DEFERRED-02 messaging substrate per the forward-scope's deferred-from-M5-P7 chain) consumes the 0.9.x baseline transparently — `phi-core = "0.9"` in workspace `Cargo.toml`; `cargo build --workspace` GREEN on first try with no carrier-fix required (phi-core 0.9.0's `AgentLoopConfig` gains no new fields per CHANGELOG line 76). CH-29's `AgentMessage` value-object substrate work is unaffected by phi-core 0.9.0's per-turn debug capture + async-trait migration (the `AgentMessage` surface is internal to baby-phi's `domain` crate + does not consume phi-core's new `BlockProvenance` / `ProvenanceRole` / `AnnotatedRequestPayload` / `TurnRequest` types).

### For CH-33 (first FUNCTIONAL M6 chunk)

CH-33 (first FUNCTIONAL M6 chunk per forward-scope §5; ships `read_inbox` + `send_message` tools via phi-core's `AgentTool` trait) consumes the 0.9.x baseline transparently — the `AgentTool` surface is unchanged at 0.8 → 0.9.0 per phi-core CHANGELOG §[0.9.0] (no `AgentTool` trait modifications in 0.9.0). CH-33 can build/test its tool surface against the absorbed baseline without any phi-core surprises.

### For M7+ observability chunk (`D-PHICORE-09-FOLLOWUP-01` closing chunk)

The closing chunk for `D-PHICORE-09-FOLLOWUP-01` inherits:
- A clean phi-core 0.9.x baseline (no further breaking-changes to absorb — assuming no 0.10.x ships before adoption).
- The 4 prerequisites enumerated in the drift body §"Remediation scope" — (1) `SessionRecorderConfig::capture_turn_requests: true` flag flip (env-var / CLI flag / per-agent config-field toggle); (2) optional `TurnRequest` recorder surfacing in baby-phi's observability layer (audit-event projection for capacity-tracking); (3) JSON-size-implications policy (per-agent toggle / per-cycle toggle / max-payload-bytes truncation / rolling-window retention); (4) operator inspection UI / CLI surface (`phi session show --include-request-payload`).
- A starting point in `cli/agent.rs` where the explicit `TurnRequest { .. } => {}` arm body is currently a no-op; the adoption chunk replaces the no-op body with audit-event emission + payload-size logging + optional truncation policy invocation.
- An open design choice at `session_recorder.rs:166-170` `matches!()` site — adoption chunk decides whether to extend to multi-pattern, replace with exhaustive `match`, or leave the single-pattern boolean recognition unchanged + thread `TurnRequest` recognition through a separate code path.

**Aggregate estimated effort for the adoption chunk**: ~0.8-1.5 ed dedicated; ~0.4-0.8 ed if bundled into an existing M7 observability chunk that already touches `SessionRecorder` or session-JSON serialization.

### For future M6+ Composition I adoption chunk (`D-PHICORE-08-FOLLOWUP-01` closing chunk)

Unaffected by CH-28c — Composition I adoption is orthogonal to per-turn debug capture adoption. Both drifts may bundle into a single "M7+ phi-core feature adoption" omnibus chunk if planning capacity allows; both are individually deferable.

---

## Revisit triggers

- phi-core 0.10.x ships with another breaking-change release → revisit §D65.1 semver-range form (consider tighter pin if upstream semver discipline drifts; e.g., flip to `phi-core = "0.9.x"` exact-pin if 0.10 introduces an unexpectedly disruptive surface). Likely a CH-28d (or similar) interstitial cycle per the precedent.
- Per-turn debug capture adoption chunk approaches dispatch → revisit §D65.4 deferral + close drift `D-PHICORE-09-FOLLOWUP-01` (Status flip `discovered` → `remediated`; feature-inventory.md §3 row "User-visible state at final" column updated to reflect the post-adoption behavior).
- More than 3 chunks defer additional work to the `M7+-FUTURE-PER-TURN-DEBUG-CAPTURE-ADOPTION` placeholder → revisit explicit named allocation (chunk-planner v13 non-terminal-drift rule may trigger orchestrator AskUserQuestion for tighter allocation; e.g., split the placeholder into `M7+-FUTURE-CAPTURE-FLAG-FLIP` + `M7+-FUTURE-INSPECTION-UI` if the prerequisites split across two natural chunks).
- baby-phi introduces a custom `BlockCompactionStrategy` or `InputFilter` impl, OR a non-`None` closure on any `AgentLoopConfig` lifecycle Fn field → revisit §D65.5 async-trait-migration-no-op claim (the consumer-side adoption then DOES need async-trait wiring at impl-time per phi-core's 0.9.0 trait shape).
- phi-core CHANGELOG adds a NEW `AgentEvent` variant before per-turn debug capture adoption ships → revisit §D65.3 explicit-arm coverage at `cli/agent.rs` (existing `_ => {}` wildcard continues to cover compile-time exhaustiveness; the new variant's explicit-arm could be added per the same cosmetic pattern OR left to the wildcard, depending on future-reader discoverability preferences).
- Operator demand for inspection of fully-assembled LLM wire payloads grows (e.g., a debugging incident where reconstruction-from-`Turn.input_messages` proves insufficient) → re-prioritize §D65.4 adoption from M7+ to an earlier M6 chunk.

---

## Verification

End-to-end verification recipe per CH-28c plan §12:

```bash
# 1. Cargo.toml workspace-deps row
grep -nE '^phi-core = "0.9"$' /root/projects/phi/baby-phi/Cargo.toml
# expect: 1 hit at line 17

# 2. Cargo.lock regenerated against phi-core 0.9.x
grep -nA 1 '^name = "phi-core"' /root/projects/phi/baby-phi/Cargo.lock | grep '^version'
# expect: version = "0.9.0" (or higher 0.9.x)

# 3. Explicit match-arm at cli/agent.rs
grep -nE 'AgentEvent::TurnRequest' /root/projects/phi/baby-phi/modules/crates/cli/src/commands/agent.rs
# expect: ≥ 1 hit at a line between the existing RevertApplied arm (~674) and the existing _ => {} wildcard

# 4. session_recorder.rs matches!() site NOT modified (per §D65.3)
git -C /root/projects/phi/baby-phi diff HEAD -- modules/crates/domain/src/session_recorder.rs | grep -c 'TurnRequest'
# expect: 0

# 5. NEW drift filed
ls /root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/drifts/D-PHICORE-09-FOLLOWUP-01-per-turn-debug-capture-adoption.md
# expect: file exists

# 6. ADR-0065 filed with Status: Accepted
head -10 /root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/decisions/0065-phi-core-09-absorption.md
# expect: Status: Accepted; 5 sub-decisions §D65.1–§D65.5 referenced

# 7. feature-inventory.md §3 row added
grep -nE '### D-PHICORE-09-FOLLOWUP-01' /root/projects/phi/baby-phi/docs/specs/v0/feature-inventory.md
# expect: ≥ 1 hit

# 8. Cycle-index row appended for 40214078
grep -n '40214078' /root/projects/phi/baby-phi/docs/specs/plan/build/_cycle-index.md
# expect: ≥ 1 hit at active-cycles tail

# 9. CH-28b carrier-fix invariant intact
grep -n 'revert_pending: None' /root/projects/phi/baby-phi/modules/crates/server/src/platform/sessions/launch.rs
# expect: ≥ 1 hit

# 10. CH-28b explicit-arm invariant intact
grep -nE 'AgentEvent::RevertApplied' /root/projects/phi/baby-phi/modules/crates/cli/src/commands/agent.rs
# expect: ≥ 1 hit

# 11. CH-25 P-SEAL invariant intact
grep -n 'response_format: phi_core::provider::traits::ResponseFormat::default' /root/projects/phi/baby-phi/modules/crates/server/src/platform/sessions/launch.rs
# expect: 1 hit

# 12. Workspace test count preserved
/root/rust-env/cargo/bin/cargo test --workspace --no-fail-fast --manifest-path /root/projects/phi/baby-phi/Cargo.toml -j 4
# expect: 1599 passed / 0 failed

# 13. phi-core import baseline preserved
grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l
# expect: 57

# 14. Clippy + fmt + CI guards
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy -j 4 --workspace --all-targets --manifest-path /root/projects/phi/baby-phi/Cargo.toml
# expect: 0 warnings
/root/rust-env/cargo/bin/cargo fmt --all --manifest-path /root/projects/phi/baby-phi/Cargo.toml -- --check
# expect: 0 diff
bash /root/projects/phi/baby-phi/scripts/check-doc-links.sh
bash /root/projects/phi/baby-phi/scripts/check-ops-doc-headers.sh
bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh
bash /root/projects/phi/baby-phi/scripts/check-spec-drift.sh
# expect: all 4 exit 0
```
