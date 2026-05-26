<!-- Last verified: 2026-05-25 by Claude Code (CH-28d-79b7386b P3 final, Accepted; 5 sub-decisions across 4 forks: F1.b CH-28d interstitial (RESOLVED at prerequisite commit `af037cf`) + F2.a `phi-core = "0.10"` semver-range + F3.a 11-line carrier-fix comment block at `launch.rs:577-593` (2 field-adds with no-op defaults) + F4.a 1 NEW drift `D-PHICORE-10-FOLLOWUP-01` at canonical `m6/drifts/` covering 4 axes A/B/C/D; absorbs phi-core 0.10.0 breaking-change release (RevertRenderPolicy + CurrentToolExecution AgentLoopConfig field-additions + final 2-of-11 lifecycle Fn async migration); 2 carrier-fixes at `launch.rs:526` (mid-tier surface between CH-28c's 0 carrier-fixes + CH-28b's 1 carrier-fix); FQN-only field-add paths preserve raw `use phi_core` line count at 57 (Δ +0 raw lines; +1 semantic leverage-site via `phi_core::types::node_tag::RevertRenderPolicy::default()`); ADR-0034 §D34.6 per-request-stateless architectural lock cited in §D66.3 to justify `current_tool: None`; opt-in posture preserved across all 4 deferred axes; Status: Accepted directly at P3 per CH-28c D-1 precedent — P-SEAL Proposed→Accepted flip skipped.) -->

# ADR-0066 — phi-core 0.10.0 absorption (RevertRenderPolicy + CurrentToolExecution AgentLoopConfig field-additions + final async-Fn migration; baseline shift)

**Status: Accepted**

**Authors**: Claude Code (orchestrator + chunk-planner v31 + chunk-implementer v18)

**Chunk**: CH-28d (cycle hex `79b7386b`)

**Milestone**: M6 (foundation tier; third consecutive interstitial "CH-NNd" chunk after CH-28b + CH-28c; TECHNICAL-PREREQUISITE)

**Decision-summary** (one line): shift baby-phi's phi-core dependency baseline from 0.9.x to 0.10.x via semver-range (`phi-core = "0.10"`) to absorb the new-API + final async-Fn migration breaking-change release; add 2 new field-adds (`revert_render_policy: phi_core::types::node_tag::RevertRenderPolicy::default()` + `current_tool: None`) at `launch.rs:577-593` AgentLoopConfig struct-literal via FQN paths + 11-line CH-28d-citing carrier-fix comment block (mirroring CH-25 `response_format` + CH-28b `revert_pending` precedent at the same site); cross-reference ADR-0034 §D34.6 per-request-stateless architectural lock to justify `current_tool: None` (baby-phi never instantiates `phi_core::BasicAgent` so the introspection slot has no consumer at v0); file 1 NEW drift `D-PHICORE-10-FOLLOWUP-01` covering 4 axes A/B/C/D adoption deferral (revert-render-policy wire-through / current-tool-timeout introspection / detect_interpreter pub-shim adoption / CurrentToolExecution standalone consumption) to a future M7+ architecture-shift OR observability adoption chunk; async-Fn migration of `BeforeToolExecutionUpdateFn` + `AfterToolExecutionUpdateFn` is a no-op consumer-side (baby-phi has zero consumers; both fields are `None` at the only call site).

---

## Forks

| Fork | Locked option | Path | Pros | Cons | Status |
|---|---|---|---|---|---|
| **F1** (TECHNICAL) | **F1.b RESOLVED at prerequisite commit `af037cf` 2026-05-25** | CH-28d interstitial chunk numbering (NOT renumbering CH-29..CH-38; NOT dot-suffix CH-28c.5) | ~5 doc-surface edits at the prerequisite commit; surgical scope; third consecutive activation of CH-NNa/b/c suffix precedent established at CH-28b + CH-28c; validates durability under thrice-repeated activation (CH-28a → CH-28b → CH-28c → CH-28d suffix lineage) | Suffix depth advances from CH-NNa → CH-NNd (still alphabetic; no novelty cost beyond CH-28c's second-of-kind) | **LOCKED RESOLVED** at chunk-initiate Phase 1 Option A (user-locked at plan-mode session 2026-05-25); not a planner-rec divergence — F1.b was the user's locked choice at the prerequisite-commit phase, ratified at gate-1.5 rubber-stamp |
| **F2** (TECHNICAL) | **F2.a planner-rec rubber-stamp at gate-1.5** | `phi-core = "0.10"` (semver-range; auto-selects highest 0.10.x patch on next `cargo update`) | Idiomatic Cargo form for "trust semver-compatible updates"; no Cargo.toml commit needed per phi-core patch; Cargo.lock pinning provides reproducibility within a given commit; matches the form ratified at CH-28b §D64.1 + CH-28c §D65.1 (single-form-precedent across 3 consecutive absorption chunks; 3rd activation) | Requires team trust in phi-core's semver discipline (mitigated by phi-core being a sibling project under the same workspace) | **LOCKED planner-rec** at gate-1.5 |
| **F3** (TECHNICAL) | **F3.a planner-rec rubber-stamp at gate-1.5** | 11-line carrier-fix comment block + 2 field-add lines at `launch.rs:577-593` (between CH-28b `revert_pending: None,` block and closing `};`); cites ADR-0066 §D66.3 + D-PHICORE-10-FOLLOWUP-01 + ADR-0034 §D34.6 | Future readers see full context inline (why default? why no-op? where is adoption tracked? why per-request-stateless lock applies?); mechanical-replication of established carrier-fix-comment pattern from CH-25 `response_format` (~10 LOC) + CH-28b `revert_pending` (~9 LOC); auditor verifies via shape-match | +13 LOC at `launch.rs` (2 field-add lines + 11-line comment); slightly larger than CH-28c's +0 LOC code delta + matches CH-28b's +16 LOC code delta | **LOCKED planner-rec** at gate-1.5 |
| **F4** (BOUNDED USER-VISIBLE) | **F4.a planner-rec rubber-stamp at gate-1.5** | File 1 NEW drift `D-PHICORE-10-FOLLOWUP-01-new-api-adoption-cluster.md` at baby-phi's canonical drift directory `m6/drifts/` covering 4 axes A/B/C/D (parallel to existing `D-PHICORE-08-FOLLOWUP-01` + `D-PHICORE-09-FOLLOWUP-01` row precedents) | Canonical drift-lifecycle discipline; rich blocked-by / closing-chunk / discovery-source metadata; single discoverable entry enumerates ALL 4 deferred axes; per chunk-planner v13 non-terminal-drift rule the placeholder `M7+-FUTURE-ARCHITECTURE-SHIFT-OR-OBSERVABILITY-ADOPTION` satisfies explicit-named-allocation requirement; 4 axes share a common architectural prerequisite cluster (long-lived-agent shape per ADR-0034 §D34.6 OR hook-script dispatch path OR pause/cancel surface) so single drift cleanly captures the cluster | One small new file added (~120 LOC drift body); 4 axes share a single drift-lifecycle entry rather than 4 individually-trackable entries (mitigated by per-axis breakdown in the drift body's §"Remediation scope") | **LOCKED planner-rec** at gate-1.5 |

**Cross-cycle divergence pattern**: 0-of-4 user-DIVERGENT forks at gate-1 for CH-28d. F1 was RESOLVED at prerequisite-commit phase before plan-spawn (out-of-band user lock per chunk-initiate Option A); F2 + F3 + F4 are planner-rec rubber-stamps at gate-1.5 (resolved internally per user-direction "resolve forks internally" 2026-05-25). Cumulative cross-cycle divergent forks for baby-phi remains at 14-of-19 (~74%) — CH-28d does not advance the count (no divergence). Third consecutive cycle holding the count steady at 14-of-19 (CH-28b + CH-28c also did not advance).

---

## Context

**Why this chunk.** phi-core 0.10.0 shipped on 2026-05-25 (one day after 0.9.0 which shipped one day after 0.8.0) with a breaking-change release introducing FIVE bundled new-API surfaces + the final 2-of-11 lifecycle Fn async-trait migration:

1. **`AgentLoopConfig.revert_render_policy: RevertRenderPolicy` new public field** — kind-aware decay window for Composition I `Lesson`/`Note` trunk-context tag rendering. Active only when `active_node_id.is_some()` (revert mode).

2. **`AgentLoopConfig.current_tool: Option<Arc<Mutex<Option<CurrentToolExecution>>>>` new public field** — shared slot the agent loop writes around every `AgentTool::execute()` invocation. Backs the new `BasicAgent::current_tool_timeout(&self) -> Option<Duration>` introspection method.

3. **`BasicAgent::with_revert_render_policy(self, RevertRenderPolicy) -> Self` new builder method** — per-agent tunable for the decay-window policy when long-lived-agent shape exists.

4. **`BasicAgent::current_tool_timeout(&self) -> Option<Duration>` new introspection method** — pause-time tool-timeout introspection backed by the `current_tool` slot.

5. **`detect_interpreter` `pub` visibility flip + `CurrentToolExecution` struct re-export from `crate::context`** — `phi_core::agent_loop::script_callback::detect_interpreter` becomes consumable for hook-script dispatch paths; `phi_core::types::context::CurrentToolExecution` becomes nameable for standalone consumers.

PLUS: the final **2-of-11 lifecycle Fn async-trait migration** — `BeforeToolExecutionUpdateFn` + `AfterToolExecutionUpdateFn` become async (the 9-of-11 migration from 0.9.0 already covered `BlockCompactionStrategy` + the other 9 `AgentLoopConfig` lifecycle Fns + `InputFilter::filter()`).

baby-phi pinned `phi-core = "0.9"` at the CH-28c close (2026-05-24 commit `8fc2018`). The cumulative phi-core delta since the last baby-phi bump comprises the 5 new-API surfaces above + the final 2-of-11 async-Fn migration. Pre-spawn verification confirmed exactly **2 COMPILE BREAKS at 1 site** (`server/src/platform/sessions/launch.rs:526` AgentLoopConfig struct-literal missing 2 new fields `revert_render_policy` + `current_tool`) across all relevant axes per the i-phi diagnostic:

- `AgentLoopConfig.revert_render_policy` new public field — **1 COMPILE BREAK at 1 site** (`launch.rs:526`). Must add `revert_render_policy: phi_core::types::node_tag::RevertRenderPolicy::default()` via FQN path (no top-level `use` line added).
- `AgentLoopConfig.current_tool` new public field — **1 COMPILE BREAK at 1 site** (same site). Must add `current_tool: None` (Rust infers the `None` type from the field's `Option<Arc<Mutex<Option<CurrentToolExecution>>>>` declaration; no source site names `CurrentToolExecution` directly).
- `BeforeToolExecutionUpdateFn` + `AfterToolExecutionUpdateFn` async-trait migration — **ZERO breaks** (baby-phi has zero consumers of either type alias; both fields are `None` at the only call site `launch.rs:548-549`).
- `BasicAgent::with_revert_render_policy` + `current_tool_timeout` new methods — **ZERO breaks + ZERO opportunity for wire-through this chunk** (baby-phi is per-request stateless per ADR-0034 §D34.6; never instantiates `phi_core::BasicAgent`; both methods are unreachable from baby-phi's session-launch architecture).
- `detect_interpreter` `pub` visibility flip — **ZERO breaks + ZERO consumer** (baby-phi has no `.sh`/`.py`/`.js` hook-script dispatch path; `grep -rn "detect_interpreter\|script_callback" modules/crates/` returns 0).
- `CurrentToolExecution` struct re-export — **ZERO breaks + ZERO direct consumer** (the struct flows through baby-phi only as a generic parameter of the `current_tool` field set to `None`; no source site names the type directly).

**Concept-doc precedence.** phi-core's CHANGELOG §[0.10.0] + the extension at `phi-core/docs/concepts/debugging.md` (per-tool-call introspection design source-of-truth) are the upstream concept docs that drive this absorption. baby-phi's concept docs accommodate the new phi-core types via the existing extensibility pattern — no concept-doc body changes required for the absorption itself; the deferred adoption work (tracked via `D-PHICORE-10-FOLLOWUP-01`) will introduce concept-doc additions at its closing chunk.

**Forward-scope reference.** `docs/specs/plan/forward-scope/m6-forward-scope-8b7a8bcd.md` §1 lines 82–92 (CH-28d narrative block) + §5 table (CH-28d row). The CH-28d row was inserted as a prerequisite commit at `af037cf` 2026-05-25 BEFORE chunk-planner spawn per F1.b lock + chunk-initiate Option A; chunk-planner consumed the row as input to the §1 deliverables enumeration.

**Downstream consumers.** ALL M6 chunks CH-29..CH-38 by virtue of the workspace compiling on the 0.10.x baseline. In particular: CH-29 (M6-DEFERRED-02 messaging substrate); CH-33 (first FUNCTIONAL M6 chunk consuming `phi_core::types::tool::AgentTool` for `read_inbox` + `send_message` tools — AgentTool surface unchanged at 0.9 → 0.10.0); future M7+ phi-core feature adoption chunks (the `D-PHICORE-10-FOLLOWUP-01` closing chunk + the `D-PHICORE-09-FOLLOWUP-01` closing chunk + the `D-PHICORE-08-FOLLOWUP-01` closing chunk; possibly bundled into a single "M7+ phi-core feature adoption" omnibus chunk).

**i-phi diagnostic provenance.** `/root/projects/phi/i-phi/docs/v0/proposal/plan/build/phi-core-0.10-absorption-1baf96ae/plan.md` predicted baby-phi absorption surface as much smaller than i-phi's (i-phi closed with 3 NEW MUST-SHIP tests + 4 phi-core-blocked drift closures + BrakingConfig live wire-through; baby-phi's per-request-stateless architecture rules out 3 of 4 wire-through axes). Pre-spawn verification confirmed the diagnostic — actual surface exactly as predicted (2 compile breaks at 1 site; paperwork; no live wire-through).

**ADR-0034 §D34.6 architectural lock.** baby-phi's per-request-stateless architecture is the load-bearing invariant that justifies `current_tool: None` + Axes A+B+D deferrals. `domain::Agent` (governance principal) connects to `phi_core::types::context::AgentContext.agent_id` via ID-only at `sessions/provider.rs::build_agent_context`; baby-phi never instantiates `phi_core::BasicAgent` / `phi_core::Agent`; revisit only if a future milestone introduces long-lived in-memory chat agents. This lock is cited explicitly in §D66.3 below + in the `launch.rs:577-593` carrier-fix comment block.

---

## Sub-decisions

### §D66.1 — F2.a `phi-core = "0.10"` semver-range form chosen over F2.b exact-pin + F2.c semver-explicit

**Decision**: workspace `Cargo.toml:17` reads `phi-core = "0.10"` (NOT `"0.10.0"` exact-pin, NOT `"^0.10.0"` semver-explicit). Per Cargo's semver-resolution semantics, `"0.10"` matches any 0.10.x patch; `cargo update -p phi-core` selects the highest published 0.10.x patch at update time (today: 0.10.0). Future phi-core 0.10.x patches flow into baby-phi automatically on the next `cargo update` invocation without a Cargo.toml commit per patch. Cargo.lock pinning provides reproducibility within a given commit.

**Pre-existing-behaviour preservation note**: pre-existing scaffold preserved — workspace `Cargo.toml:17` already used the canonical semver-range form `phi-core = "0.9"` since CH-28c close 2026-05-24 (which itself paralleled the pre-existing 0.8 → 0.7.1 form). The shift to `"0.10"` continues the single-form-precedent established at CH-28b §D64.1 + CH-28c §D65.1 across 3 consecutive absorption chunks. CH-28d is the **third activation** of the F2.a precedent (CH-28b first-of-kind → CH-28c second activation → CH-28d third activation), validating durability under thrice-repeated activation.

### §D66.2 — F1.b CH-28d interstitial chunk-numbering convention chosen over F1.a renumber + F1.c CH-28c.5 dot-suffix

**Decision**: the cycle is named `CH-28d` (interstitial suffix); the cycle folder is `ch-28d-phi-core-10-absorption-79b7386b`; the forward-scope row was inserted as a NEW fourth row of the Foundation tier between CH-28c and CH-29 at prerequisite commit `af037cf` 2026-05-25 (NOT renumbering CH-29..CH-38 to CH-30..CH-39). This is the **third consecutive activation** of the "CH-NNa/b/c/d" suffix convention established at CH-28b §D64.2 and validated at CH-28c §D65.2. Going CH-28a → CH-28b → CH-28c → CH-28d validates that the suffix lineage extends naturally as additional interstitial absorption chunks arrive (future phi-core breaking-change releases during M6 will likely continue the CH-NNe/f/g lineage rather than renumber).

**Pre-existing-behaviour preservation note**: pre-existing scaffold preserved — the "CH-NNa/b/c" suffix convention was established at CH-28b §D64.2 as canonical for "interstitial chunks that absorb upstream breaking-changes without renumbering downstream cycles" and validated at CH-28c §D65.2. CH-28d is the third activation of the convention, validating durability under thrice-repeated activation. The convention's repeatability (CH-NN → CH-NNa → CH-NNb → CH-NNc → CH-NNd) is a structural property of the alphabetic suffix scheme; remains naturally extensible to CH-NNe / CH-NNf / etc. if phi-core continues to ship breaking-change releases during M6.

### §D66.3 — F3.a 2 new field-adds at launch.rs:577-593 with no-op defaults via FQN path + 11-line carrier-fix comment block

**Decision**: at `modules/crates/server/src/platform/sessions/launch.rs` between line 576 (end of existing CH-28b `revert_pending: None,` block) and the closing `};` at line 594 (originally line 577), insert the 11-line CH-28d-citing carrier-fix comment block + 2 field-add lines:

```rust
// CH-28d P2 workspace-health carrier-fix (per ADR-0066 §D66.3):
// phi-core 0.10.0 adds two new public fields to AgentLoopConfig:
// (1) `revert_render_policy: RevertRenderPolicy` — kind-aware decay
//     window for Composition I `Lesson`/`Note` trunk-context tag
//     rendering. Active only when `active_node_id.is_some()` (revert
//     mode); baby-phi does NOT enable revert mode at v0 (Composition I
//     adoption deferred per D-PHICORE-08-FOLLOWUP-01). Default value
//     preserves byte-for-byte pre-0.10 behaviour.
// (2) `current_tool: Option<Arc<Mutex<Option<CurrentToolExecution>>>>`
//     — shared slot the agent loop writes around every
//     AgentTool::execute() invocation. Backs the new
//     BasicAgent::current_tool_timeout(&self) introspection method.
//     baby-phi is per-request stateless (ADR-0034 §D34.6) + has no
//     pause-time tool-timeout surface today; `None` disables the slot.
// Live wire-through deferred to D-PHICORE-10-FOLLOWUP-01.
revert_render_policy: phi_core::types::node_tag::RevertRenderPolicy::default(),
current_tool: None,
```

Net diff: +13 LOC at `launch.rs` (2 field-add lines + 11-line comment). The fully-qualified `phi_core::types::node_tag::RevertRenderPolicy::default()` path avoids adding a `use phi_core::types::node_tag::RevertRenderPolicy;` line at the file head (preserving the workspace `use phi_core` line count at 57 raw lines per §3 prediction); `current_tool: None` uses Rust's type-inference from the `AgentLoopConfig.current_tool` field type (preserving the same property for the `CurrentToolExecution` surface — no source site names the type).

**ADR-0034 §D34.6 cross-reference**: `current_tool: None` is justified by baby-phi's per-request-stateless architectural lock. baby-phi never instantiates `phi_core::BasicAgent`; `domain::AgentId` flows into `phi_core::types::context::AgentContext.agent_id` via ID-only at `sessions/provider.rs::build_agent_context`. The introspection slot has no consumer at v0 — there is no long-lived agent reference to call `current_tool_timeout()` on, and baby-phi has no pause/cancel surface. Revisit `current_tool: None` only if a future milestone introduces long-lived in-memory chat agents (Axes A + B + D close criterion).

**Pre-existing-behaviour preservation note**: pre-existing scaffold preserved — `launch.rs:526` `AgentLoopConfig` struct-literal already carried 2 prior carrier-fix entries (CH-25 `response_format: ResponseFormat::default()` ~10 LOC + CH-28b `revert_pending: None` ~9 LOC). CH-28d's 2 field-adds + 11-line comment continue the precedent. The byte-for-byte pre-0.10 behaviour is preserved at the struct-literal: (a) `revert_render_policy: RevertRenderPolicy::default()` only takes effect when `active_node_id.is_some()` (revert mode) which baby-phi never sets at v0; (b) `current_tool: None` disables the introspection slot (the `Arc<Mutex<...>>` would otherwise be written by phi-core's agent loop around every `AgentTool::execute()` invocation; with `None` the writes are no-ops).

### §D66.4 — F4.a 1 NEW drift `D-PHICORE-10-FOLLOWUP-01` filed for 4-axis adoption deferral cluster

**Decision**: 4-axis new-API adoption (Axes A/B/C/D enumerated in the drift body — `with_revert_render_policy` wire-through / `current_tool_timeout` introspection / `detect_interpreter` pub-shim adoption / `CurrentToolExecution` standalone consumption) is deferred to a future M7+ FUNCTIONAL chunk via a single drift covering all 4 axes (chosen over F4.b 4 separate drifts). The allocation placeholder is `M7+-FUTURE-ARCHITECTURE-SHIFT-OR-OBSERVABILITY-ADOPTION` (per chunk-planner v13 non-terminal-drift rule explicit-named-allocation requirement; NOT `TBD`). No specific CH-NN slot is reserved at CH-28d close — future planning session decides per-axis routing (Axis C is independently schedulable; Axes A+B+D bundle as an architecture-shift cluster; Axis A alone is schedulable if the long-lived-agent shift lands without B+D).

**F4.a rationale (chosen over F4.b 4 separate drifts)**: none of the 4 axes is independently actionable today; all 4 would sit at `Status: discovered` for the foreseeable future. The 4 axes share a common architectural prerequisite cluster (long-lived-agent shape per ADR-0034 §D34.6 OR hook-script dispatch path OR pause/cancel surface) so a single drift cleanly captures the cluster + the long-lived-agent prerequisite is a load-bearing axis that gates 3 of the 4 axes (A + B + D). Single discoverable drift entry at `m6/drifts/D-PHICORE-10-FOLLOWUP-01-new-api-adoption-cluster.md` enumerates ALL 4 axes; future M7+ planners navigate to one canonical location instead of hunting across 4 sibling files.

**Pre-existing-absence preserved (never-shipped-yet variant per chunk-planner v24 P-plan-2)**: no prior 0.10.0 new-API surface exists in baby-phi (the features are introduced fresh by phi-core 0.10.0). CH-28d absorbs the type space without enabling any of the 4 axes. The pre-0.10.0 posture (no kind-aware tag rendering; no per-tool-call introspection; no hook-script dispatch; no `CurrentToolExecution` standalone consumption) is preserved verbatim by virtue of the no-op defaults at the `launch.rs:577-593` struct-literal + the architectural-shift prerequisites that gate adoption (long-lived-agent shape OR hook-script dispatch path OR pause/cancel surface).

### §D66.5 — Async-Fn migration is a no-op consumer-side; no adoption deferral needed

**Decision**: phi-core 0.10.0's final 2-of-11 lifecycle Fn async-trait migration (`BeforeToolExecutionUpdateFn` + `AfterToolExecutionUpdateFn` type aliases become async) is internal-only to phi-core's type-alias shapes. baby-phi has ZERO consumers of either type alias:

- `grep -rn "BeforeToolExecutionUpdateFn\|AfterToolExecutionUpdateFn" /root/projects/phi/baby-phi/modules/crates/` returns 0 hits.
- Both fields are `None` at the only `AgentLoopConfig` struct-literal site (`launch.rs:548-549`).

Async-fication is therefore a no-op consumer-side at CH-28d — no compile breaks, no carrier-fix needed, no follow-on adoption work needed. The `D-PHICORE-10-FOLLOWUP-01` drift covers ONLY the 4 new-API adoption axes (A/B/C/D); no drift is filed for the async-Fn migration itself.

**Pre-existing-behaviour preservation note**: pre-existing scaffold preserved — baby-phi's `launch.rs:548-549` AgentLoopConfig struct-literal already sets both `before_tool_execution_update: None` + `after_tool_execution_update: None`. The 0.7.1 → 0.8 → 0.9 → 0.10 progression has preserved this `None`-everywhere posture across all 4 absorption chunks. CH-28c §D65.5 already established the no-op rule for the 9-of-11 async-trait migration (`BlockCompactionStrategy` + 9 of 11 `AgentLoopConfig` lifecycle Fns + `InputFilter::filter()`); CH-28d §D66.5 closes the final 2-of-11 migration with the same rule. If a future baby-phi chunk introduces a non-`None` closure on either `before_tool_execution_update` OR `after_tool_execution_update` field, that chunk MUST use the async-future shape per phi-core's 0.10.0 type aliases; revisit §D66.5 if such a consumer materializes. The no-op nature of async-fication at CH-28d close does NOT warrant a drift — the feature is structurally absorbed without behavioral change.

---

## Cross-references

**(a) Concept doc + line range** (per CH-13 retro Row 1 ADR-structure discipline):
- [`phi-core/CHANGELOG.md`](../../../../../../../phi-core/CHANGELOG.md) §[0.10.0] — release notes enumerating the 5 new-API surfaces (2 AgentLoopConfig fields + 2 BasicAgent methods + detect_interpreter pub-flip + CurrentToolExecution re-export) + the final 2-of-11 lifecycle Fn async migration + Migration guidance.
- [`phi-core/docs/concepts/debugging.md`](../../../../../../../phi-core/docs/concepts/debugging.md) — per-tool-call introspection design source-of-truth: shared `Arc<Mutex<Option<CurrentToolExecution>>>` slot written around every `AgentTool::execute()` invocation; backs `BasicAgent::current_tool_timeout(&self) -> Option<Duration>` introspection method.
- [`docs/specs/v0/concepts/phi-core-mapping.md`](../../../concepts/phi-core-mapping.md) full doc — phi-core type ↔ baby-phi consumer mapping table (no mapping-row edits required at CH-28d; new phi-core types `RevertRenderPolicy` + `CurrentToolExecution` + new methods `with_revert_render_policy` + `current_tool_timeout` + `detect_interpreter` pub-shim become available but NOT imported until adoption fires).

**(b) Closed drift(s) by ID**: NONE closed at CH-28d. NEW drift filed at chunk-seal: `D-PHICORE-10-FOLLOWUP-01-new-api-adoption-cluster.md` (Status: `discovered`; Severity: LOW; Bucket: B; Closing chunk: `M7+-FUTURE-ARCHITECTURE-SHIFT-OR-OBSERVABILITY-ADOPTION` placeholder; 4 axes A/B/C/D enumerated).

**(c) Prior ADRs cited as precedent**:
- [`ADR-0065`](0065-phi-core-09-absorption.md) (CH-28c 0.9.0 absorption) — directly-parallel pattern; ADR-0066 mirrors the section shape (Forks / Context / Sub-decisions / Cross-references / Consequences / Revisit triggers / Verification) + sub-decision body conventions exactly. CH-28c's §D65.1 (semver-range form) ↔ §D66.1; §D65.2 (CH-NNa/b/c suffix convention) ↔ §D66.2; §D65.4 (drift filing + feature-deferral) ↔ §D66.4; §D65.5 (async-trait migration no-op consumer-side) ↔ §D66.5. CH-28d carries 2 carrier-fixes (vs CH-28c's 0 carrier-fixes) — the new §D66.3 sub-decision absorbs the carrier-fix work that CH-28c did not need.
- [`ADR-0064`](0064-phi-core-08-absorption.md) (CH-28b 0.8.0 absorption) — directly-parallel pattern + the `revert_pending` carrier-fix-comment-block precedent. CH-28b's §D64.3 (`revert_pending: None` carrier-fix at `launch.rs:577` with ~9-line comment block) is the shape-template for CH-28d's §D66.3 (2 field-adds + 11-line comment block at the immediately-following site `launch.rs:577-593`).
- [`ADR-0059`](../../m5_2/decisions/0059-recent-sessions-api-surface-flip.md) §D59.2 (CH-25 P-SEAL `response_format` carrier-fix precedent at the same `launch.rs:526` struct-literal site) — first-of-kind carrier-fix-comment-block (~10 LOC) establishing the structural pattern (minimal additive fields with semantic-default = opt-out + inline CH-NN-citing comment block + cross-reference to deferred adoption drift). CH-28b §D64.3 was the second activation; CH-28d §D66.3 is the third activation (with 2 field-adds instead of 1).
- [`ADR-0034`](../../m5_2/decisions/0034-agent-durable-lifecycle.md) §D34.6 — per-request-stateless architectural lock cited explicitly in §D66.3 to justify `current_tool: None` + the Axes A+B+D deferrals. baby-phi never instantiates `phi_core::BasicAgent`; the connection between `domain::Agent` (governance principal) and `phi_core::types::context::AgentContext.agent_id` is ID-only at `sessions/provider.rs::build_agent_context`. Revisit only if a future milestone introduces long-lived in-memory chat agents.
- [`ADR-0063`](0063-agent-profile-cardinality-n-to-1.md) §D63.3 — CH-28 forward-scope-row-as-prerequisite-commit precedent. CH-28d's F1.b RESOLVED at prerequisite commit `af037cf` follows the same chunk-initiate Option A pattern (user-locked at plan-mode session; forward-scope row inserted BEFORE chunk-planner spawn; chunk-planner consumes the row as input).

**(d) Forward-scope row**: [`m6-forward-scope-8b7a8bcd.md`](../../../../plan/forward-scope/m6-forward-scope-8b7a8bcd.md) §1 lines 82–92 (CH-28d narrative block) + §5 table (CH-28d summary table row). The row was authored as a prerequisite commit at `af037cf` 2026-05-25 BEFORE chunk-planner spawn per F1.b lock + chunk-initiate Option A.

---

## Consequences

### For CH-29 (next M6 chunk)

CH-29 (next M6 chunk per forward-scope §5; M6-DEFERRED-02 messaging substrate per the forward-scope's deferred-from-M5-P7 chain) consumes the 0.10.x baseline transparently — `phi-core = "0.10"` in workspace `Cargo.toml`; `cargo build --workspace` GREEN on first try since CH-28d's 2 carrier-fixes already absorbed the only struct-literal break. CH-29's `AgentMessage` value-object substrate work is unaffected by phi-core 0.10.0's 5 new-API surfaces + final async-Fn migration (the `AgentMessage` surface is internal to baby-phi's `domain` crate + does not consume phi-core's new types `RevertRenderPolicy` / `CurrentToolExecution` directly).

### For CH-33 (first FUNCTIONAL M6 chunk)

CH-33 (first FUNCTIONAL M6 chunk per forward-scope §5; ships `read_inbox` + `send_message` tools via phi-core's `AgentTool` trait) consumes the 0.10.x baseline transparently — the `AgentTool` surface is unchanged at 0.9 → 0.10.0 per phi-core CHANGELOG §[0.10.0] (no `AgentTool` trait modifications in 0.10.0). CH-33 can build/test its tool surface against the absorbed baseline without any phi-core surprises.

### For CH-29..CH-38 (downstream M6 chunks)

ALL downstream M6 chunks compile cleanly on the 0.10.x baseline at CH-28d close. No additional baseline shifts expected during M6 unless phi-core ships a 0.11.x release; if so, a future CH-28e (or similar) interstitial cycle continues the suffix-precedent lineage per §D66.2.

### For M7+ architecture-shift OR observability chunk (`D-PHICORE-10-FOLLOWUP-01` closing chunk)

The closing chunk(s) for `D-PHICORE-10-FOLLOWUP-01` inherit:
- A clean phi-core 0.10.x baseline (no further breaking-changes to absorb — assuming no 0.11.x ships before adoption).
- 4 axis-specific adoption tracks (per drift body §"Remediation scope"):
  - **Axis A** — `BasicAgent::with_revert_render_policy` live wire-through (~3-5 ed for long-lived-agent shift alone + ~0.3-0.5 ed for tunable wiring; depends on D-PHICORE-08-FOLLOWUP-01 closure).
  - **Axis B** — `BasicAgent::current_tool_timeout` introspection (~2-4 ed for pause/cancel surface + ~0.3-0.6 ed for introspection consumption; depends on Axis A).
  - **Axis C** — `detect_interpreter` pub-shim adoption (~1-2 ed for hook-script dispatch path; independent of Axes A/B/D).
  - **Axis D** — `CurrentToolExecution` standalone consumption (~0.3-0.6 ed for audit-event projection; depends on Axes A + B).
- Aggregate effort estimate: ~7-15 ed for full closure; partial closures independently schedulable (Axis C alone ~1-2 ed; Axes A+B+D bundle ~6-10 ed; Axis A alone ~3.5-5.5 ed).
- A starting point in `launch.rs:592-593` where the 2 field-adds carry no-op defaults; adoption-chunk(s) replace the defaults with live-wired forms (per-policy tunable for `revert_render_policy`; `Some(Arc::new(Mutex::new(None)))` for `current_tool` once a long-lived-agent shape exists).

**Aggregate estimated effort for the adoption chunk(s)**: ~7-15 ed for full closure (all 4 axes); partial closures independently schedulable; possible bundle into a single "M7+ phi-core feature adoption" omnibus chunk if planning capacity allows (along with `D-PHICORE-08-FOLLOWUP-01` Composition I adoption + `D-PHICORE-09-FOLLOWUP-01` per-turn debug capture adoption).

### For future M6+ Composition I adoption chunk (`D-PHICORE-08-FOLLOWUP-01` closing chunk)

Unaffected by CH-28d directly — Composition I adoption is orthogonal to the 4 axes A/B/C/D. HOWEVER: Axis A of `D-PHICORE-10-FOLLOWUP-01` (revert-render-policy wire-through) DEPENDS on `D-PHICORE-08-FOLLOWUP-01` closure (Composition I revert mode must be enabled before kind-aware decay-window tag rendering activates — `RevertRenderPolicy` only takes effect when `active_node_id.is_some()`). Both drifts may bundle into a single "M7+ phi-core feature adoption" omnibus chunk if planning capacity allows.

### For future M7+ per-turn debug capture adoption chunk (`D-PHICORE-09-FOLLOWUP-01` closing chunk)

Unaffected by CH-28d — per-turn debug capture adoption is orthogonal to all 4 axes A/B/C/D of `D-PHICORE-10-FOLLOWUP-01`. All 3 drifts (08 + 09 + 10) may bundle into a single "M7+ phi-core feature adoption" omnibus chunk if planning capacity allows.

---

## Revisit triggers

- phi-core 0.11.x ships with another breaking-change release → revisit §D66.1 semver-range form (consider tighter pin if upstream semver discipline drifts; e.g., flip to `phi-core = "0.10.x"` exact-pin if 0.11 introduces an unexpectedly disruptive surface). Likely a CH-28e (or similar) interstitial cycle per the precedent lineage.
- baby-phi adopts long-lived-agent architecture (per ADR-0034 §D34.6 revisit) → revisit §D66.3 `current_tool: None` default; the introspection slot becomes wireable. Triggers Axes A+B+D close criterion + `D-PHICORE-10-FOLLOWUP-01` Status flip `discovered` → `partially-remediated` (per-axis closure annotation).
- baby-phi introduces a `.sh`/`.py`/`.js` hook-script dispatch path → revisit §D66.4 Axis C deferral; the `detect_interpreter` pub-shim becomes consumable. Triggers Axis C close criterion + per-axis closure annotation in the drift body.
- baby-phi introduces a non-`None` closure on `before_tool_execution_update` OR `after_tool_execution_update` field → revisit §D66.5 async-trait-migration-no-op claim (the consumer-side adoption then DOES need async-future wiring at impl-time per phi-core's 0.10.0 type aliases).
- phi-core CHANGELOG adds NEW public fields to `AgentLoopConfig` before any of the 4 axes adopt → revisit §D66.3 carrier-fix coverage at `launch.rs:526` (existing 11-line CH-28d comment block remains; the new field-adds follow the same precedent pattern — likely a CH-28e + ADR-0067 cycle).
- Operator demand for kind-aware Composition I tag rendering tunability grows OR for per-tool-call pause-time introspection OR for interpreted-script hooks → re-prioritize §D66.4 axis-specific closures from M7+ to an earlier M6 chunk.
- 3 chunks defer additional work to the `M7+-FUTURE-ARCHITECTURE-SHIFT-OR-OBSERVABILITY-ADOPTION` placeholder → revisit explicit named allocation (chunk-planner v13 non-terminal-drift rule may trigger orchestrator AskUserQuestion for tighter allocation; e.g., split the placeholder into per-axis allocations if prerequisites split across natural chunks).

---

## Verification

End-to-end verification recipe per CH-28d plan §12:

```bash
# 1. Cargo.toml workspace-deps row
grep -nE '^phi-core = "0.10"$' /root/projects/phi/baby-phi/Cargo.toml
# expect: 1 hit at line 17

# 2. Cargo.lock regenerated against phi-core 0.10.x
grep -nA 1 '^name = "phi-core"' /root/projects/phi/baby-phi/Cargo.lock | grep '^version'
# expect: version = "0.10.0" (or higher 0.10.x)

# 3. 2 new field-adds at launch.rs (CH-28d carrier-fixes)
grep -n 'revert_render_policy: phi_core::types::node_tag::RevertRenderPolicy::default' /root/projects/phi/baby-phi/modules/crates/server/src/platform/sessions/launch.rs
grep -n 'current_tool: None' /root/projects/phi/baby-phi/modules/crates/server/src/platform/sessions/launch.rs
# expect: 1 hit each

# 4. NEW drift filed
ls /root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/drifts/D-PHICORE-10-FOLLOWUP-01-new-api-adoption-cluster.md
# expect: file exists

# 5. ADR-0066 filed with Status: Accepted
head -10 /root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/decisions/0066-phi-core-10-absorption.md
# expect: Status: Accepted; 5 sub-decisions §D66.1–§D66.5 referenced

# 6. feature-inventory.md §3 row added
grep -nE '### D-PHICORE-10-FOLLOWUP-01' /root/projects/phi/baby-phi/docs/specs/v0/feature-inventory.md
# expect: ≥ 1 hit

# 7. Cycle-index row appended for 79b7386b
grep -n '79b7386b' /root/projects/phi/baby-phi/docs/specs/plan/build/_cycle-index.md
# expect: ≥ 1 hit at active-cycles tail

# 8. CH-25 P-SEAL invariant intact (response_format)
grep -n 'response_format: phi_core::provider::traits::ResponseFormat::default' /root/projects/phi/baby-phi/modules/crates/server/src/platform/sessions/launch.rs
# expect: 1 hit

# 9. CH-28b carrier-fix invariant intact (revert_pending)
grep -n 'revert_pending: None' /root/projects/phi/baby-phi/modules/crates/server/src/platform/sessions/launch.rs
# expect: ≥ 1 hit

# 10. CH-28b + CH-28c explicit-arm invariants intact
grep -nE 'AgentEvent::RevertApplied|AgentEvent::TurnRequest' /root/projects/phi/baby-phi/modules/crates/cli/src/commands/agent.rs
# expect: ≥ 2 hits

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
bash /root/projects/phi/baby-phi/scripts/check-doc-links.sh
bash /root/projects/phi/baby-phi/scripts/check-ops-doc-headers.sh
bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh
bash /root/projects/phi/baby-phi/scripts/check-spec-drift.sh
# expect: all 4 exit 0

# 14. Forbidden-duplication greps (per plan §3)
grep -rn "^pub struct RevertRenderPolicy" /root/projects/phi/baby-phi/modules/crates/
grep -rn "^pub struct CurrentToolExecution" /root/projects/phi/baby-phi/modules/crates/
grep -rn "^pub type BeforeToolExecutionUpdateFn" /root/projects/phi/baby-phi/modules/crates/
grep -rn "^pub type AfterToolExecutionUpdateFn" /root/projects/phi/baby-phi/modules/crates/
grep -rn "^pub fn detect_interpreter" /root/projects/phi/baby-phi/modules/crates/
grep -rn "^pub struct AgentLoopConfig" /root/projects/phi/baby-phi/modules/crates/
# expect: 0 hits each
```
