<!-- Last verified: 2026-05-23 by Claude Code (CH-28b-d5b776ac P-SEAL final, Accepted; orchestrator-applied Trivial-1L verified-header refresh at gate-3 audit-A iter-1 side observation — body Status flipped Proposed → Accepted at P-SEAL but verified-header initial draft retained "Proposed" phrasing; refreshed here for header/body consistency; 6 sub-decisions across 3 forks: F1.b CH-28b interstitial + F2.a `phi-core = "0.8"` semver-range + F3.a NEW drift `D-PHICORE-08-FOLLOWUP-01` at canonical `m6/drifts/`; absorbs phi-core 0.8.0 breaking-change release (Composition I baseline shift); 1-line `AgentLoopConfig.revert_pending: None` carrier-fix at `launch.rs:577` paralleling CH-25 P-SEAL `response_format` precedent; explicit `AgentEvent::RevertApplied { .. } => {}` arm at `cli/agent.rs:668` for future-reader discoverability; `session_recorder.rs:166-170` `matches!()` single-pattern site intentionally NOT modified per §D64.6 scope-narrow; opt-out posture preserved — Composition I NOT enabled at session launch; deferred adoption tracked via `D-PHICORE-08-FOLLOWUP-01` to future M6+ FUNCTIONAL chunk `M6+-FUTURE-COMPOSITION-I-ADOPTION` placeholder.) -->

# ADR-0064 — phi-core 0.8.0 absorption (Composition I braking layer; baseline shift)

**Status: Accepted**

**Authors**: Claude Code (orchestrator + chunk-planner v28 + chunk-implementer v18)

**Chunk**: CH-28b (cycle hex `d5b776ac`)

**Milestone**: M6 (foundation tier; first interstitial "CH-NNa/b" chunk; TECHNICAL-PREREQUISITE)

**Decision-summary** (one line): shift baby-phi's phi-core dependency baseline from 0.7.1 to 0.8.x via semver-range (`phi-core = "0.8"`) to absorb the Composition I breaking-change release; apply the 1-line `AgentLoopConfig.revert_pending: None` carrier-fix at `launch.rs:577` paralleling CH-25's `response_format` precedent; add explicit `AgentEvent::RevertApplied { .. } => {}` arm at `cli/agent.rs:668` for future-reader discoverability; intentionally NOT modify `session_recorder.rs:166-170` (single-pattern `matches!()` site — adding the arm would change boolean semantics); defer Composition I adoption (`.with_revert_tool()`) to future M6+ FUNCTIONAL chunk via NEW drift `D-PHICORE-08-FOLLOWUP-01`.

---

## Forks

| Fork | Locked option | Path | Pros | Cons | Status |
|---|---|---|---|---|---|
| **F1** (TECHNICAL) | **F1.b RESOLVED at prerequisite commit `563dcda` 2026-05-23** | CH-28b interstitial chunk numbering (NOT renumbering CH-29..CH-38; NOT dot-suffix CH-28.5) | ~5 doc-surface edits at the prerequisite commit; surgical scope; first-of-kind "CH-NNa/b" suffix establishes precedent for future interstitial breaking-change-absorption chunks | First-of-kind suffix in cycle-index naming convention — slight novelty cost | **LOCKED RESOLVED** at plan-mode session (user-locked); not a planner-rec divergence — F1.b was the user's locked choice at the prerequisite-commit phase, ratified at gate-1.5 rubber-stamp |
| **F2** (TECHNICAL) | **F2.a planner-rec rubber-stamp at gate-1.5** | `phi-core = "0.8"` (semver-range; auto-selects highest 0.8.x patch on next `cargo update`) | Idiomatic Cargo form for "trust semver-compatible updates"; no Cargo.toml commit needed per phi-core patch; Cargo.lock pinning provides reproducibility within a given commit | Requires team trust in phi-core's semver discipline (mitigated by phi-core being a sibling project under the same workspace) | **LOCKED planner-rec** at gate-1.5 |
| **F3** (BOUNDED USER-VISIBLE) | **F3.a planner-rec rubber-stamp at gate-1.5** | File NEW drift at baby-phi's canonical drift directory `m6/drifts/D-PHICORE-08-FOLLOWUP-01-composition-i-adoption.md` (parallel to existing `D-CH28-FOLLOWUP-01` row precedent) | Canonical drift-lifecycle discipline; rich blocked-by / closing-chunk / discovery-source metadata; feature-inventory.md §3 row gives the user-visible v0-vs-final translation; per chunk-planner v13 non-terminal-drift rule the placeholder `M6+-FUTURE-COMPOSITION-I-ADOPTION` satisfies explicit-named-allocation requirement | One small new file added (~150 LOC drift body) | **LOCKED planner-rec** at gate-1.5 |

**Cross-cycle divergence pattern**: 0-of-3 user-DIVERGENT forks at gate-1 for CH-28b. F1 was RESOLVED at prerequisite-commit phase before plan-spawn (out-of-band user lock); F2 + F3 are planner-rec rubber-stamps at gate-1.5. Cumulative cross-cycle divergent forks for baby-phi remains at 14-of-19 (~74%) — CH-28b does not advance the count (no divergence).

---

## Context

**Why this chunk.** phi-core 0.8.0 shipped on 2026-05-23 with a breaking-change release introducing Composition I — an opt-in tree-structured braking layer activated via `BasicAgent::with_revert_tool()`. baby-phi pinned `phi-core = "0.7.1"` at the M5 close. The cumulative phi-core delta since the last baby-phi bump includes 3 breaking changes per the phi-core CHANGELOG §[0.8.0]:

1. `LlmMessage` gains 3 new public fields (`node_id`, `parent_id`, `tags`). Direct struct-literal construction breaks; constructor `LlmMessage::new()` shields all baby-phi call sites — pre-spawn verification confirmed baby-phi has zero `LlmMessage { .. }` struct-literals.
2. `AgentEvent` becomes `#[non_exhaustive]` + gains `RevertApplied { .. }` variant. Exhaustive matches in downstream crates require either explicit `RevertApplied` arms or wildcard `_ =>`. Pre-spawn verification confirmed baby-phi's 2 match sites (`cli/agent.rs:668` + `session_recorder.rs:166`) already carry compile-time wildcards (`_ => {}` at `cli/agent.rs:668` covers the cli match; the `matches!()` block at `session_recorder.rs:166-170` is single-pattern and compile-time-exhaustive without modification).
3. `AgentLoopConfig` gains `revert_pending: Option<Arc<Mutex<Vec<RevertRequest>>>>`. Direct struct-literal construction breaks. Pre-spawn verification confirmed baby-phi has exactly ONE direct struct-literal site at `server/src/platform/sessions/launch.rs:526-568` (the SAME site that received CH-25 P-SEAL's `response_format` carrier-fix at lines 558-567).

**Concept-doc precedence.** phi-core's CHANGELOG §[0.8.0] + `concept-brake.md` §5 Composition I are the upstream concept docs that drive this absorption. baby-phi's concept docs (`phi-core-mapping.md`, etc.) accommodate the new phi-core types via the existing extensibility pattern — no concept-doc body changes required for the absorption itself; the deferred adoption work (tracked via `D-PHICORE-08-FOLLOWUP-01`) will introduce concept-doc additions at its closing chunk.

**Forward-scope reference.** `docs/specs/plan/forward-scope/m6-forward-scope-8b7a8bcd.md` §1 lines 56–67 (CH-28b narrative block) + line 319 (§5 summary table row). The CH-28b row was inserted as a prerequisite commit at `563dcda` 2026-05-23 BEFORE chunk-planner spawn per F1.b lock; chunk-planner consumed the row as input to the §1 deliverables enumeration.

**Downstream consumers.** ALL M6 chunks CH-29..CH-38 by virtue of the workspace compiling on the 0.8.x baseline. In particular: CH-33 (first FUNCTIONAL M6 chunk consuming `phi_core::types::tool::AgentTool` for `read_inbox` + `send_message` tools — AgentTool surface unchanged at 0.7.1 → 0.8.0 per CHANGELOG); future M6+ Composition I adoption chunk (the `D-PHICORE-08-FOLLOWUP-01` closing chunk).

**i-phi diagnostic provenance.** `/root/projects/phi/i-phi/docs/v0/proposal/plan/build/phi-core-revert-tool-27c894f6/plan.md` §DIAGNOSTIC SECTION A predicted baby-phi absorption impact as LOW (1 carrier-fix at AgentLoopConfig struct-literal + 0-2 explicit match-arm additions). Pre-spawn verification confirmed the diagnostic — actual surface even smaller (1 compile error + 1 cosmetic match-arm + 0 modifications at the `matches!()` site).

---

## Sub-decisions

### §D64.1 — F2.a `phi-core = "0.8"` semver-range form chosen over F2.b exact-pin + F2.c semver-explicit

**Decision**: workspace `Cargo.toml:17` reads `phi-core = "0.8"` (NOT `"0.8.0"` exact-pin, NOT `"^0.8.0"` semver-explicit). Per Cargo's semver-resolution semantics, `"0.8"` matches any 0.8.x patch; `cargo update -p phi-core` selects the highest published 0.8.x patch at update time (today: 0.8.0). Future phi-core 0.8.x patches flow into baby-phi automatically on the next `cargo update` invocation without a Cargo.toml commit per patch. Cargo.lock pinning provides reproducibility within a given commit.

**Pre-existing-behaviour preservation note**: pre-existing scaffold preserved — workspace `Cargo.toml:17` already used a literal-string version (`phi-core = "0.7.1"`) which Cargo treats as semver-compatible-or-higher within the same minor version (equivalent to `"^0.7.1"`). The shift to `"0.8"` is a structural broadening that aligns with the canonical Cargo idiom for "trust semver-compatible updates" + does not change the resolution semantics shape (only the version-range endpoints).

### §D64.2 — F1.b CH-28b interstitial chunk-numbering convention chosen over F1.a renumber + F1.c CH-28.5 dot-suffix

**Decision**: the cycle is named `CH-28b` (interstitial suffix); the cycle folder is `ch-28b-phi-core-08-absorption-d5b776ac`; the forward-scope row is inserted as a NEW second row of the Foundation tier between CH-28 and CH-29 (NOT renumbering CH-29..CH-38 to CH-30..CH-39). This is the first-of-kind "CH-NNa/b" suffix in baby-phi's cycle-index naming convention. The first-of-kind suffix establishes a precedent for future interstitial breaking-change-absorption chunks (e.g., if phi-core 0.9.0 ships during the M6 build, the corresponding absorption chunk might be CH-29b or CH-30b).

**Pre-existing-absence preserved**: no prior "CH-NNa/b" suffix convention shipped in baby-phi's cycle-index; CH-28b establishes the convention as canonical for "interstitial chunks that absorb upstream breaking-changes without renumbering downstream cycles." Variation (c) "never-shipped-yet" per chunk-planner v11 P-plan-1 + CH-19 ADR-0057 §D57.6 precedent.

### §D64.3 — `AgentLoopConfig.revert_pending: None` carrier-fix preserves opt-out posture

**Decision**: at `modules/crates/server/src/platform/sessions/launch.rs:577` (immediately after the existing `response_format: phi_core::provider::traits::ResponseFormat::default(),` field at line 567), add a new line `revert_pending: None,` with an 8-line CH-28b-citing comment block paralleling CH-25's `response_format` comment block style at lines 558-567. The `None` semantic-default preserves the opt-out posture — Composition I is NOT enabled at session launch; agents continue to operate identically to the pre-0.8.0 monotonically-growing-context shape; the `Arc<Mutex<Vec<RevertRequest>>>` inner type is NEVER instantiated; no in-process state is added; no IPC channel is added; no K8s blocker class is introduced.

**Pre-existing-behaviour preservation note**: pre-existing scaffold preserved — `launch.rs:526-568` struct-literal already received CH-25's `response_format: ResponseFormat::default()` carrier-fix at lines 558–567; the CH-28b carrier-fix follows the identical structural pattern (minimal 1-line additive field with semantic-default = opt-out, with an inline CH-NN-citing comment block). The exact pattern was ratified at CH-25 P-SEAL workspace-health fix; CH-28b is the second application of the pattern.

### §D64.4 — F3.a drift filing at canonical `m6/drifts/` directory chosen over F3.b feature-inventory-only embedding

**Decision**: NEW drift file at `docs/specs/v0/implementation/m6/drifts/D-PHICORE-08-FOLLOWUP-01-composition-i-adoption.md` (parallel to existing `m6/drifts/D-CH28-FOLLOWUP-01-blueprint-upserted-template-fanout.md`); feature-inventory.md §3 Deferred catalogue gains a `D-PHICORE-08-FOLLOWUP-01` row inserted between the existing `M7b-DEFERRED-02` and `D-CH28-FOLLOWUP-01` rows. The drift body documents the 4 Composition I adoption prerequisites per the i-phi diagnostic §DIAGNOSTIC SECTION A "Adoption guidance" (BasicAgent builder call site + optional SessionRecorder event surfacing + skill / prompt-authoring axis + `RevertRenderPolicy` tuning); the feature-inventory row provides the product-trajectory translation (v0 state vs final state).

**Pre-existing-behaviour preservation note**: pre-existing scaffold preserved — baby-phi's drift-directory convention (`m<N>/drifts/`) ratified at CH-28 ADR-0063's filing of `D-CH28-FOLLOWUP-01` at `m6/drifts/`; CH-28b's `D-PHICORE-08-FOLLOWUP-01` filing follows the same pattern + the same drift-body shape (Identification / Concept alignment / Plan vs. reality / Where visible in code / Remediation scope / Why filed as follow-on / Lifecycle history / Cross-references sections).

### §D64.5 — Composition I adoption deferred to future M6+ FUNCTIONAL chunk via `D-PHICORE-08-FOLLOWUP-01`

**Decision**: Composition I adoption (the 4 prerequisites enumerated in the `D-PHICORE-08-FOLLOWUP-01` drift body — BasicAgent::with_revert_tool() builder call site + optional RevertApplied event surfacing in BabyPhiSessionRecorder + skill / prompt teaching the agent the revert discipline + RevertRenderPolicy tuning) is deferred to a future M6+ FUNCTIONAL chunk. The allocation placeholder is `M6+-FUTURE-COMPOSITION-I-ADOPTION` (per chunk-planner v13 non-terminal-drift rule explicit-named-allocation requirement; NOT `TBD`). No specific CH-NN slot is reserved at CH-28b close — future planning session decides whether adoption lands as a dedicated FUNCTIONAL chunk OR bundles into an existing M6+ FUNCTIONAL chunk that touches BasicAgent construction.

**Pre-existing-absence preserved**: no Composition I adoption surface has shipped in baby-phi (the feature is opt-in upstream + not yet enabled). Variation (c) "never-shipped-yet" per chunk-planner v11 P-plan-1. The adoption is a NEW feature axis introduced by phi-core 0.8.0; baby-phi's pre-0.8.0 posture (monotonically-growing-context with `BlockCompactionStrategy` as the only relief) is preserved by virtue of the `revert_pending: None` opt-out posture per §D64.3.

### §D64.6 — Explicit `AgentEvent::RevertApplied { .. } => {}` arm added at `cli/agent.rs:642-669` ONLY; NOT added at `session_recorder.rs:166-170`

**Decision**: at `modules/crates/cli/src/commands/agent.rs:642-669` (the `match &event` exhaustive-match block against `AgentEvent`), insert an explicit `AgentEvent::RevertApplied { .. } => {}` arm BEFORE the existing `_ => {}` wildcard arm at line 668. The arm body is a no-op (Composition I opt-in is NOT enabled at CH-28b close per §D64.3); the arm is cosmetic + signals 0.8.0 variant awareness for future readers + future-proofs the cli surface when Composition I adoption fires (at the adoption-chunk the no-op body becomes the audit-event-emission + render-policy-filtering surface).

**At `modules/crates/domain/src/session_recorder.rs:166-170`** (the `matches!()` block matching a SINGLE pattern `PhiCoreAgentEvent::AgentStart { session_id, .. }`), DO NOT add an explicit `RevertApplied` arm. Rationale: the `matches!()` block returns a boolean (`true` if the event matches `AgentStart`, `false` otherwise); it is single-pattern + compile-time-exhaustive without modification (since `matches!()` already returns `false` for any non-matched variant including `RevertApplied`). Adding an `RevertApplied` arm to the single-pattern `matches!()` would change boolean semantics — the block would then return `true` for BOTH `AgentStart` AND `RevertApplied` events, which is semantically wrong (the block's intent is to recognize `AgentStart` specifically for session-id binding). The future Composition I adoption chunk MAY extend the `matches!()` to a multi-pattern form OR replace with an exhaustive `match` block if `RevertApplied` warrants governance visibility (see `D-PHICORE-08-FOLLOWUP-01` prerequisite #2).

**Pre-existing-behaviour preservation note**: pre-existing wildcard arm preserved — `cli/agent.rs:668` `_ => {}` already covers the new `RevertApplied` variant at compile-time (since `AgentEvent` becomes `#[non_exhaustive]` at 0.8.0, the wildcard is required-or-permitted by the compiler); the explicit `RevertApplied { .. } => {}` arm added by CH-28b is cosmetic + does NOT shadow the wildcard semantically (the wildcard still catches any future variants beyond `RevertApplied`). The `session_recorder.rs:166-170` `matches!()` site is preserved EXACTLY as-is — single-pattern boolean recognition continues to return `true` only for `PhiCoreAgentEvent::AgentStart`.

---

## Cross-references

**(a) Concept doc + line range** (per CH-13 retro Row 1 ADR-structure discipline):
- [`phi-core/CHANGELOG.md`](../../../../../../../phi-core/CHANGELOG.md) §[0.8.0] lines 9–94 — release notes enumerating the 3 breaking changes + 7 added items + Composition I activation form.
- [`phi-core/docs/concepts/concept-brake.md`](../../../../../../../phi-core/docs/concepts/concept-brake.md) §5 Composition I — design source-of-truth for tree-structured composition + `RevertCategory` semantics.
- [`docs/specs/v0/concepts/phi-core-mapping.md`](../../../concepts/phi-core-mapping.md) full doc — phi-core type ↔ baby-phi consumer mapping table (no mapping-row edits required at CH-28b; new phi-core types `NodeId`/`NodeTag`/`RevertCategory`/`RevertRenderPolicy`/`RevertTool`/`RevertRequest` + new `AgentEvent::RevertApplied` variant become available but NOT imported until adoption fires).

**(b) Closed drift(s) by ID**: NONE closed at CH-28b. NEW drift filed at chunk-seal: `D-PHICORE-08-FOLLOWUP-01-composition-i-adoption.md` (Status: `discovered`; Severity: LOW; Bucket: B; Closing chunk: `M6+-FUTURE-COMPOSITION-I-ADOPTION` placeholder).

**(c) Prior ADRs cited as precedent**:
- [`ADR-0063`](0063-agent-profile-cardinality-n-to-1.md) §D63.16 — drift-filing precedent at canonical `m6/drifts/` directory (CH-28 filed `D-CH28-FOLLOWUP-01` per §D63.16; CH-28b's `D-PHICORE-08-FOLLOWUP-01` follows the same drift-shape + naming-convention per §D64.4).
- **CH-25 P-SEAL workspace-health fix** at `launch.rs:558-567` (`response_format: phi_core::provider::traits::ResponseFormat::default()`) — structural carrier-fix precedent: minimal 1-line additive field with semantic-default = opt-out + inline CH-NN-citing comment block (CH-28b §D64.3 replicates this structural pattern as the second application; the precedent ADR was not formally numbered but lives in the CH-25 plan archive + retrospective).

**(d) Forward-scope row**: [`m6-forward-scope-8b7a8bcd.md`](../../../../plan/forward-scope/m6-forward-scope-8b7a8bcd.md) §1 lines 56–67 (CH-28b narrative block) + line 319 (§5 summary table row). The row was authored as a prerequisite commit at `563dcda` 2026-05-23 BEFORE chunk-planner spawn per F1.b lock.

---

## Consequences

### For CH-29 (next M6 chunk)

CH-29 (next M6 chunk per forward-scope §5; M6-DEFERRED-02 messaging substrate per the forward-scope's deferred-from-M5-P7 chain) consumes the 0.8.x baseline transparently — `phi-core = "0.8"` in workspace `Cargo.toml`; `cargo build --workspace` GREEN on first try with no carrier-fix required. CH-29's `AgentMessage` value-object substrate work is unaffected by phi-core 0.8.0's Composition I (the `AgentMessage` surface is internal to baby-phi's `domain` crate + does not consume phi-core's new `RevertRequest` / `RevertApplied` / `NodeTag` types).

### For CH-33 (first FUNCTIONAL M6 chunk)

CH-33 (first FUNCTIONAL M6 chunk per forward-scope §5; ships `read_inbox` + `send_message` tools via phi-core's `AgentTool` trait) consumes the 0.8.x baseline transparently — the `AgentTool` surface is unchanged at 0.7.1 → 0.8.0 per phi-core CHANGELOG §[0.8.0] "Unchanged" section. CH-33 can build/test its tool surface against the absorbed baseline without any phi-core surprises.

### For future M6+ Composition I adoption chunk (`D-PHICORE-08-FOLLOWUP-01` closing chunk)

The closing chunk for `D-PHICORE-08-FOLLOWUP-01` inherits:
- A clean phi-core 0.8.x baseline (no further breaking-changes to absorb).
- The 4 prerequisites enumerated in the drift body §"Remediation scope" — (1) `BasicAgent::with_revert_tool()` builder call site + (2) optional `RevertApplied` event surfacing in `BabyPhiSessionRecorder` + (3) skill / prompt teaching the agent the revert discipline + (4) `RevertRenderPolicy` tuning.
- A starting point in `cli/agent.rs:668` where the explicit `RevertApplied { .. } => {}` arm body is currently a no-op; the adoption chunk replaces the no-op body with audit-event emission + render-policy filtering.
- An open design choice at `session_recorder.rs:166-170` `matches!()` site — adoption chunk decides whether to extend to multi-pattern, replace with exhaustive `match`, or leave the single-pattern boolean recognition unchanged + thread `RevertApplied` recognition through a separate code path.

**Aggregate estimated effort for the adoption chunk**: ~1.2-2.3 ed dedicated; ~0.5-1 ed if bundled into an existing M6+ FUNCTIONAL chunk that already touches BasicAgent construction.

---

## Revisit triggers

- phi-core 0.9.x ships with another breaking-change release → revisit §D64.1 semver-range form (consider tighter pin if upstream semver discipline drifts; e.g., flip to `phi-core = "0.8.x"` exact-pin if 0.9 introduces an unexpectedly disruptive surface).
- Composition I adoption chunk approaches dispatch → revisit §D64.5 deferral + close drift `D-PHICORE-08-FOLLOWUP-01` (Status flip `discovered` → `remediated`; feature-inventory.md §3 row "User-visible state at final" column updated to reflect the post-adoption behavior).
- More than 3 chunks defer additional work to the `M6+-FUTURE-COMPOSITION-I-ADOPTION` placeholder → revisit explicit named allocation (chunk-planner v13 non-terminal-drift rule may trigger orchestrator AskUserQuestion for tighter allocation; e.g., split the placeholder into `M6+-FUTURE-COMPOSITION-I-WIRE` + `M6+-FUTURE-COMPOSITION-I-SKILL` if the prerequisites split across two natural chunks).
- phi-core CHANGELOG adds a NEW `AgentEvent` variant before Composition I adoption ships → revisit §D64.6 explicit-arm coverage at `cli/agent.rs:668` (existing `_ => {}` wildcard continues to cover compile-time exhaustiveness; the new variant's explicit-arm could be added per the same cosmetic pattern OR left to the wildcard, depending on future-reader discoverability preferences).

---

## Verification

End-to-end verification recipe per CH-28b plan §12:

```bash
# 1. Cargo.toml workspace-deps row
grep -nE '^phi-core = "0.8"$' /root/projects/phi/baby-phi/Cargo.toml
# expect: 1 hit at line 17

# 2. Cargo.lock regenerated against phi-core 0.8.x
grep -nE '^name = "phi-core"' /root/projects/phi/baby-phi/Cargo.lock
# expect: 1 hit; version line below should read `version = "0.8.0"` (or higher 0.8.x)

# 3. AgentLoopConfig carrier-fix
grep -nB1 'revert_pending: None' /root/projects/phi/baby-phi/modules/crates/server/src/platform/sessions/launch.rs
# expect: 1 hit; preceding context shows response_format line

# 4. Explicit match-arm
grep -nE 'AgentEvent::RevertApplied' /root/projects/phi/baby-phi/modules/crates/cli/src/commands/agent.rs
# expect: ≥ 1 hit before line 670

# 5. session_recorder.rs matches!() site NOT modified (per §D64.6)
git -C /root/projects/phi/baby-phi diff HEAD -- modules/crates/domain/src/session_recorder.rs | grep -c 'RevertApplied'
# expect: 0

# 6. NEW drift filed
ls /root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/drifts/D-PHICORE-08-FOLLOWUP-01-composition-i-adoption.md
# expect: file exists

# 7. ADR-0064 filed with Status: Accepted at P-SEAL
head -10 /root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/decisions/0064-phi-core-08-absorption.md
# expect: Status: Accepted; 6 sub-decisions §D64.1–§D64.6 referenced

# 8. feature-inventory.md §3 row added
grep -nE '### D-PHICORE-08-FOLLOWUP-01' /root/projects/phi/baby-phi/docs/specs/v0/feature-inventory.md
# expect: ≥ 1 hit

# 9. Cycle-index row appended for d5b776ac
grep -n 'd5b776ac' /root/projects/phi/baby-phi/docs/specs/plan/build/_cycle-index.md
# expect: ≥ 1 hit at active-cycles tail

# 10. CH-25 P-SEAL invariant intact
grep -n 'response_format: phi_core::provider::traits::ResponseFormat::default' /root/projects/phi/baby-phi/modules/crates/server/src/platform/sessions/launch.rs
# expect: 1 hit at line 567

# 11. Workspace test count preserved
/root/rust-env/cargo/bin/cargo test --workspace --no-fail-fast --manifest-path /root/projects/phi/baby-phi/Cargo.toml -j 4
# expect: 1599 passed / 0 failed

# 12. phi-core import baseline preserved
grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l
# expect: 57

# 13. Clippy + fmt + CI guards
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy -j 4 --workspace --all-targets --manifest-path /root/projects/phi/baby-phi/Cargo.toml
# expect: 0 warnings
/root/rust-env/cargo/bin/cargo fmt --all --manifest-path /root/projects/phi/baby-phi/Cargo.toml -- --check
# expect: 0 diff
bash /root/projects/phi/baby-phi/scripts/check-doc-links.sh && \
  bash /root/projects/phi/baby-phi/scripts/check-ops-doc-headers.sh && \
  bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh && \
  bash /root/projects/phi/baby-phi/scripts/check-spec-drift.sh
# expect: all 4 exit 0
```
