<!-- Last verified: 2026-05-25 by Claude Code (CH-28d chunk-planner iter-1 v31 — TECHNICAL-PREREQUISITE; absorbs phi-core 0.10.0 breaking-change release; 4 phases P1+P2+P3+P-SEAL (P3 absorbs ADR + drift + feature-inventory + verified-header amends); SMALL audit envelope (1 auditor letter A) per §11 sizing override rationale paralleling CH-28b + CH-28c — 3rd consecutive activation; zero phi-core leverage delta in terms of leverage-sites (1 NEW FQN-only reference at single field-add site; baseline raw `use phi_core` line count preserved at 57); K8s-neutral on all 7 axes; predicted 2 COMPILE BREAKS at 1 site (launch.rs:526 AgentLoopConfig struct-literal missing 2 new fields `revert_render_policy` + `current_tool`) closed at P2; cycle hex TBD (assigned by chunk-archive-plan post-approval).) -->

# CH-28d — phi-core 0.10.0 absorption (RevertRenderPolicy + CurrentToolExecution AgentLoopConfig field-additions + final async-Fn migration; baseline shift)

**Forward-scope row**: [`docs/specs/plan/forward-scope/m6-forward-scope-8b7a8bcd.md`](../../forward-scope/m6-forward-scope-8b7a8bcd.md) §1 lines 82–92 + §5 table (CH-28d row).

**Cycle hex**: TBD (assigned by `chunk-archive-plan` skill post-approval; planner-proposed token `8d0a2761` — orchestrator mints canonical via `openssl rand -hex 4`).

**Chunk-type**: TECHNICAL-PREREQUISITE.

**Severity**: LOW. **Effort**: 0.3–0.5 ed.

**Prerequisite**: CH-28c (cycle `40214078`, closed 2026-05-24 commit `8fc2018`).

---

## Forks for orchestrator

> Cross-cycle divergence callout (chunk-planner v12; updated 2026-05-24 per CH-28c retro tracker): cumulative cross-cycle divergent forks for baby-phi is **14-of-19 (~74%)** at gate-1 fork-locks; the modal outcome is divergence on tighter / more-fragmented / more-defensive options. CH-28b held the count steady at 14-of-19 (0-of-3 divergent); CH-28c held it steady (0-of-3 divergent). CH-28d expects the same shape: ALL FOUR forks below resolve to planner-rec at gate-1 per orchestrator's pre-decided locks recorded in the plan-mode plan at `/root/.claude/plans/sharded-discovering-stearns.md`. Treat each fork as a confirmation prompt, not an open architectural decision.

### F1 — Chunk numbering convention (TECHNICAL FORK — no user-visible delta — pick on engineering merit only)

| Option | Pros | Cons | Status |
|---|---|---|---|
| F1.a CH-29 renumber | • Linear naming preserves single-digit-suffix sortability | • ~30 doc-surface edits across forward-scope rows + cycle-graph references + concept-doc cross-refs cascading from CH-29..CH-38 → CH-30..CH-39 | NOT chosen |
| **F1.b CH-28d interstitial (planner-rec)** | • ~5 doc-surface edits at the prerequisite commit; surgical scope • Continues the CH-NNa/b/c suffix precedent established at CH-28b + CH-28c for "interstitial chunks that absorb upstream breaking-changes without renumbering downstream" (3rd consecutive activation) • Aligns with the third consecutive phi-core breaking-change-absorption arriving one day after CH-28c | • Suffix depth advances from CH-NNa → CH-NNd (still alphabetic; no novelty cost beyond CH-28c's second-of-kind) | **LOCKED at gate-1 (planner-rec; resolved at prerequisite commit `af037cf` per chunk-initiate Option A; user pre-decided in plan-mode session 2026-05-25)** |
| F1.c CH-28c.5 dot-suffix | • Visually surfaces "between CH-28c and CH-29" • ~5 edits | • Two coexisting interstitial conventions (CH-NNa/b/c/d + CH-NN.X) muddies cycle-index lookup | NOT chosen |

### F2 — Cargo dep-string form (TECHNICAL FORK — no user-visible delta — pick on engineering merit only)

| Option | Pros | Cons | Status |
|---|---|---|---|
| **F2.a `phi-core = "0.10"` semver-range (planner-rec)** | • Cargo idiom for "trust semver-compatible updates"; future 0.10.x patches flow into baby-phi automatically on `cargo update` without per-patch Cargo.toml commit • Cargo.lock provides reproducibility within a given commit • Matches the form ratified at CH-28b §D64.1 (`phi-core = "0.8"`) + CH-28c §D65.1 (`phi-core = "0.9"`) — single-form-precedent across 3 consecutive absorption chunks | • Requires team trust in phi-core's semver discipline (mitigated by phi-core being a sibling project under the same workspace) | **LOCKED at gate-1 (planner-rec rubber-stamp expected at gate-1.5)** |
| F2.b `phi-core = "0.10.0"` exact-pin | • Most conservative; explicit floor + ceiling at a single patch | • Cargo.toml commit required per 0.10.x patch upgrade; high churn for a release line that may patch frequently | NOT chosen |
| F2.c `phi-core = "^0.10.0"` semver-explicit | • Equivalent resolution to F2.a but with explicit caret | • Idiomatic Cargo style avoids the caret when the version-string would convey it implicitly; visual noise | NOT chosen |

### F3 — Field-add comment density (TECHNICAL FORK — affects diff verbosity + future-reader clarity)

| Option | Pros | Cons | Status |
|---|---|---|---|
| **F3.a 11-line carrier-fix comment block citing ADR-0066 + D-PHICORE-10-FOLLOWUP-01 + ADR-0034 §D34.6 (planner-rec)** — mirrors CH-25 `response_format` (~10 LOC) + CH-28b `revert_pending` (~9 LOC) precedent | • Future readers see full context for why these fields are at default + where adoption is tracked + why baby-phi's per-request-stateless architecture justifies `current_tool: None` • Mechanical-replication of established carrier-fix-comment pattern; auditor verifies via shape-match | • +13 LOC at launch.rs (2 field-add lines + 11-line comment); slightly larger than CH-28c's +0 LOC code delta + matches CH-28b's +16 LOC code delta | **LOCKED at gate-1 (planner-rec rubber-stamp expected at gate-1.5)** |
| F3.b 2-line comment (`// CH-28d 0.10.0 absorption; live wire deferred to D-PHICORE-10-FOLLOWUP-01`) + bare field-adds | • +4 LOC at launch.rs (minimal) | • Less context inline — readers must follow ADR cross-ref to understand the "no-op default" rationale + ADR-0034 §D34.6 per-request-stateless implications for `current_tool: None` | NOT chosen |

### F4 — Drift filing strategy (BOUNDED USER-VISIBLE — pick on discovery + workflow merit)

| Option | User-visible (what the user perceives) | Pros | Cons + Product trajectory | Status |
|---|---|---|---|---|
| **F4.a 1 drift `D-PHICORE-10-FOLLOWUP-01` covering 4 axes A/B/C/D (planner-rec)** | Future agents looking for "what phi-core 0.10.0 adoption work is deferred?" find 1 drift with 4 enumerated axes (revert-policy wire-through, tool-timeout introspection, detect_interpreter shim, CurrentToolExecution consumer). | **User-visible:** A single discoverable entry in `m6/drifts/` enumerates ALL 4 deferred 0.10.0 adoption axes; future M7+ observability planners navigate to one canonical location instead of hunting across feature-inventory + ADRs + forward-scope rows. + Canonical drift-lifecycle discipline; rich blocked-by / closing-chunk / discovery-source metadata + feature-inventory.md §3 row gives the v0-vs-final user-visible-state translation + per chunk-planner v13 non-terminal-drift rule the placeholder `M7+-FUTURE-ARCHITECTURE-SHIFT-OR-OBSERVABILITY-ADOPTION` satisfies the explicit-named-allocation requirement | • One small new file added (~120 LOC drift body) <br><br>**Product trajectory:** When any single axis adopts, drift body's lifecycle-entry annotation marks that axis `[ANSWERED at CH-NN]`; full closure when all 4 axes adopt. Cleaner than 4 separate drifts when none is independently actionable today; the 4 axes share the same long-lived-agent / hook-script-path / pause-cancel-surface architectural prerequisite cluster. | **LOCKED at gate-1 (planner-rec rubber-stamp expected at gate-1.5)** |
| F4.b 4 separate drifts (`-revert-policy-wire` / `-tool-timeout-introspect` / `-detect-interpreter-shim` / `-current-tool-execution-consumer`) | Each axis discoverable individually — better when axes adopt in different chunks at different milestones | • More granular tracking; more drift-allocation paperwork | **Product trajectory:** Four separate drift IDs to track; future adoption may hit only 1 axis at a time, simplifying single-axis closure. But none is independently actionable today, so 4 separate files would all sit at `Status: discovered` for the foreseeable future. | NOT chosen |

---

## §1 — Locked fork details (per chunk-initiate Phase 1.5 Step A ALWAYS-FIRE + chunk-planner v23 P-plan-3 + v28 §1-position codification)

#### F1 = F1.b — CH-28d interstitial chunk numbering

**Code-level binding**: the cycle is named `CH-28d`; the cycle folder is `phi-core-10-absorption-<8hex>/`; the forward-scope row was inserted as a NEW fourth row of the Foundation tier between CH-28c and CH-29 at prerequisite commit `af037cf` 2026-05-25 (BEFORE chunk-planner spawn per chunk-initiate Option A). NO renumbering of downstream cycles CH-29..CH-38.

**Rationale**: Continues the CH-NNa/b/c suffix precedent established by CH-28b + CH-28c — interstitial chunks that absorb upstream breaking-changes between an existing CH-NN cycle and its downstream consumers, without renumbering the downstream chain. CH-28b shipped the first-of-kind suffix for this pattern; CH-28c was the second consecutive interstitial absorption; CH-28d is the **third consecutive activation** (phi-core 0.10.0 arrived one day after 0.9.0 which arrived one day after 0.8.0), validating the precedent's durability under repeated activation.

**Defers**: no other lock implications. Future phi-core breaking-change releases during the M6 build will likely continue the CH-NNe/f/g suffix lineage rather than renumber.

#### F2 = F2.a — `phi-core = "0.10"` semver-range form

**Code-level binding**: workspace `Cargo.toml:17` flips from `phi-core = "0.9"` (CH-28c close baseline) to `phi-core = "0.10"`. Cargo.lock regenerates via `cargo update -p phi-core` against the published 0.10.0 patch (or higher 0.10.x if patches ship between now and chunk-open).

**Rationale**: matches the form ratified at CH-28b §D64.1 + CH-28c §D65.1 — single-form-precedent across 3 consecutive absorption chunks. Cargo's semver resolution treats `"0.10"` as `"^0.10.0"`, so future 0.10.x patches flow into baby-phi automatically without per-patch Cargo.toml commits. Cargo.lock pinning provides reproducibility within a given commit.

**Defers**: nothing — the dep-string-form decision is fully resolved at this chunk.

#### F3 = F3.a — 11-line carrier-fix comment block at field-add site

**Code-level binding**: at `/root/projects/phi/baby-phi/modules/crates/server/src/platform/sessions/launch.rs` immediately AFTER the existing CH-28b `revert_pending: None,` block (lines 568-576) AND BEFORE the closing `};` at line 577, insert the 11-line comment block + 2 field-add lines. Net diff: +13 LOC at launch.rs. The 11-line comment cites ADR-0066 §D66.3 + D-PHICORE-10-FOLLOWUP-01 + ADR-0034 §D34.6, explains why both new 0.10.0 fields default to no-op shapes, and signals where live wire-through adoption is tracked.

**Rationale**: mirrors the CH-25 `response_format` (~10 LOC) + CH-28b `revert_pending` (~9 LOC) carrier-fix-comment precedent established at this single struct-literal site. Future readers see the full context inline (why default? why no-op? where is adoption tracked?); auditor verifies via shape-match against the 2 precedent blocks.

**Defers**: nothing — comment density is fully resolved at this chunk.

#### F4 = F4.a — 1 drift covering 4 axes A/B/C/D

**Code-level binding**: NEW file `/root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/drifts/D-PHICORE-10-FOLLOWUP-01-new-api-adoption-cluster.md` mirrors the structural shape of CH-28b's `D-PHICORE-08-FOLLOWUP-01-composition-i-adoption.md` + CH-28c's `D-PHICORE-09-FOLLOWUP-01-per-turn-debug-capture-adoption.md` (verified-header line 1 + ## Identification + ## Concept alignment + ## Plan vs reality + ## Where visible in code + ## Remediation scope sections). Drift body enumerates 4 axes (A `with_revert_render_policy` wire-through / B `current_tool_timeout` introspection / C `detect_interpreter` shim / D `CurrentToolExecution` consumer). feature-inventory.md §3 Deferred catalogue gains a row IMMEDIATELY AFTER the existing `D-PHICORE-09-FOLLOWUP-01` row.

**Rationale**: maintains canonical drift-lifecycle discipline (rich blocked-by / closing-chunk / discovery-source / phase-of-origin metadata) + co-locates all 3 consecutive phi-core feature deferrals (Composition I + per-turn debug capture + new-API cluster) at a single discoverable location for future M7+ observability planners. Per chunk-planner v13 non-terminal-drift rule the placeholder `M7+-FUTURE-ARCHITECTURE-SHIFT-OR-OBSERVABILITY-ADOPTION` satisfies the explicit-named-allocation requirement (NOT `TBD`). All 4 axes share a common architectural prerequisite (long-lived-agent shape per ADR-0034 §D34.6 OR hook-script dispatch path OR pause/cancel surface) so a single drift cleanly captures the cluster.

**Defers**: the 4 new-API adoption axes themselves (the load-bearing wire-through work — instantiating `phi_core::BasicAgent` per request + adding pause/cancel surface + adding hook-script dispatch path + wiring `CurrentToolExecution` consumer) to a future M7+ FUNCTIONAL chunk via this drift. Async-trait migration of the 2 tool-update Fn types (`BeforeToolExecutionUpdateFn` + `AfterToolExecutionUpdateFn`) carries NO adoption deferral — baby-phi has zero consumers of either type alias (both fields are `None` at the only call site), so async-fication is a no-op consumer-side with no follow-on work needed.

---

## §2 — Concept alignment walk

| Concept doc | Line range | Claim | Status at chunk open | Target status at chunk close |
|---|---|---|---|---|
| `phi-core/CHANGELOG.md` §[0.10.0] | (latest release entry) | Release notes enumerate 5 surfaces: `AgentLoopConfig` gains 2 new public fields (`revert_render_policy: RevertRenderPolicy` + `current_tool: Option<Arc<Mutex<Option<CurrentToolExecution>>>>`); `BasicAgent::with_revert_render_policy` + `current_tool_timeout` new methods; final 2-of-11 lifecycle Fn async migration (`BeforeToolExecutionUpdateFn` + `AfterToolExecutionUpdateFn`); `detect_interpreter` `pub` visibility flip; `CurrentToolExecution` struct re-export from `crate::context` | `not-yet-absorbed` (baby-phi pinned at 0.9.x) | `honored` (baby-phi compiles on 0.10.x; 2 new fields added at launch.rs:526 via FQN paths; deferred adoption tracked via `D-PHICORE-10-FOLLOWUP-01`) |
| `phi-core/docs/concepts/debugging.md` | (per-tool-call introspection design extension at 0.10.0) | Design source-of-truth for per-tool-call introspection: shared `Arc<Mutex<Option<CurrentToolExecution>>>` slot written around every `AgentTool::execute()` invocation; backs `BasicAgent::current_tool_timeout(&self) -> Option<Duration>` introspection method | `concept-aspirational` (baby-phi does NOT enable per-tool introspection at CH-28d) | `concept-aspirational-preserved` (CH-28d absorbs the field-shape but does not enable; adoption deferred to M7+ via `D-PHICORE-10-FOLLOWUP-01` Axes A+B+D) |
| `i-phi/docs/v0/proposal/plan/build/phi-core-0.10-absorption-1baf96ae/plan.md` | (precedent absorption on i-phi side) | i-phi closed 0.10.0 absorption at cycle `1baf96ae` (3 NEW MUST-SHIP tests + 4 phi-core-blocked drift closures + BrakingConfig live wire-through); empirical signal that baby-phi's absorption surface is much smaller (per-request-stateless architecture rules out 3 of 4 wire-through axes) | `predicted-low-impact` | `confirmed-low-impact` (pre-spawn verification confirmed 2 compile breaks at 1 site; matches plan-mode prediction exactly) |
| `baby-phi/docs/specs/v0/implementation/m6/decisions/0034-domain-agent-and-phi-core-agent.md` §D34.6 | (per-request-stateless architectural lock) | baby-phi never instantiates `phi_core::BasicAgent`; per-request-stateless architecture connects `domain::AgentId` → `phi_core::types::context::AgentContext.agent_id` via ID-only flow at `sessions/provider.rs::build_agent_context` | `honored at CH-28c close` | `honored at CH-28d close` (the 0.10.0 new-API adoption deferrals Axes A+B+D rest entirely on this lock; ADR-0066 §D66.3 cites it explicitly) |

**Concept-doc precedence note**: per chunk-planner v8 §3.D Forward-scope-vs-concept-doc precedence — the forward-scope row at `m6-forward-scope-8b7a8bcd.md` lines 82–92 references three external concept-doc surfaces (the phi-core CHANGELOG + debugging.md + i-phi plan archive) + one baby-phi-internal architectural lock (ADR-0034 §D34.6). All three external sources are concept-aspirational relative to baby-phi; the absorption involves no baby-phi-internal concept-doc contradictions. The deferred adoption (per-tool introspection + revert-policy wire-through) intersects future baby-phi observability concept-doc surface, tracked via `D-PHICORE-10-FOLLOWUP-01`.

---

## §2.5 — Functional outcome (mandatory; added 2026-05-20 per CH-28 retro plan archive `chunk-decomposition-and-fork-framing-76e04080.md`)

**Functional outcome at chunk close**: **NONE**. CH-28d is a TECHNICAL-PREREQUISITE chunk — technical prerequisite for ALL downstream M6 chunks CH-29..CH-38. Shifts baby-phi's phi-core dependency baseline from 0.9 to 0.10.x to absorb the phi-core 0.10.0 release shipped 2026-05-25.

**User-visible behavior at chunk close**: **zero delta from CH-28c close**. Agents continue to operate on the pre-0.10.0 posture; both new `AgentLoopConfig` fields default to no-op shapes (`RevertRenderPolicy::default()` only takes effect when `active_node_id.is_some()` which baby-phi never sets; `current_tool: None` means no introspection slot is wired); the 2 async-flipped tool-update hook fields are `None` at the only call site; the new `BasicAgent` methods + `detect_interpreter` `pub`-flip + `CurrentToolExecution` re-export are out-of-reach for baby-phi's per-request-stateless architecture (per ADR-0034 §D34.6).

**Non-technical-user rationale**: *"phi-core released a third minor version in three days (0.8.0 → 0.9.0 → 0.10.0). 0.10.0 closes out the async-trait migration the prior 0.9.0 release started by flipping the last two tool-update hook types to async + adds two new fields to the agent loop config struct that baby-phi instantiates once. We absorb the version baseline shift today so the rest of M6 runs on 0.10.x + future patches flow through automatically. Surface is mid-way between CH-28b (1 compile break) + CH-28c (zero compile breaks): 2 mechanical field additions at 1 struct-literal site."*

**Defers (with product impact)**:
- **`BasicAgent::with_revert_render_policy` live wire-through (Axis A)** → future M7+ architecture-shift chunk (no specific CH-NN slot reserved; tracked via `D-PHICORE-10-FOLLOWUP-01` Axis A). **User-visible impact while deferred**: Composition I `Lesson`/`Note` trunk-context tag rendering uses phi-core's default decay window; operator cannot tune the decay-window policy via baby-phi config TOML.
- **`BasicAgent::current_tool_timeout` pause-time tool-timeout introspection (Axis B)** → future M7+ observability chunk (placeholder `M7+-FUTURE-ARCHITECTURE-SHIFT-OR-OBSERVABILITY-ADOPTION`). **User-visible impact while deferred**: agents have no per-tool-call introspection from outside the loop; baby-phi doesn't pause sessions mid-tool-execution today.
- **`detect_interpreter` `pub`-shim adoption (Axis C)** → out-of-scope at v0 (baby-phi has no `.sh`/`.py`/`.js` hook-script dispatch path).
- **`CurrentToolExecution` struct consumption (Axis D)** → depends on Axis A + Axis B; same long-lived-agent prerequisite.
- **Async-Fn migration deferral**: N/A — baby-phi has zero consumers of the 2 tool-update Fn types at sync, so async-fication is a no-op consumer-side; no migration work needed at adoption time either. The drift `D-PHICORE-10-FOLLOWUP-01` covers ONLY the 4 new-API adoption axes, not async-Fn migration.

---

## §3 — phi-core leverage map

**Predicted import delta** (chunk-planner v9 leverage-sites-not-import-lines methodology per CH-03-i-phi retro P4): **Δ +0 leverage-sites** at the *raw `use phi_core` line count* level (baseline 57 preserved — the 2 new field-adds use **fully-qualified path** `phi_core::types::node_tag::RevertRenderPolicy::default()` directly at the field-add site WITHOUT adding a new `use phi_core::types::node_tag::RevertRenderPolicy;` line, mirroring CH-25's `phi_core::provider::traits::ResponseFormat::default()` precedent at the same site). **Δ +1 leverage-site** at the *semantic* level (NEW transitive consumption of `RevertRenderPolicy` type via FQN; `current_tool: None` is a primitive-default that doesn't add a leverage-site).

Baseline `grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l` = **57** (verified at chunk-planning time; matches CH-28c chunk-close baseline preserved). Predicted at chunk close: **57** (unchanged at raw-line-count level; the +1 semantic leverage-site does not surface as a raw `use` line).

**Methodology note**: per chunk-planner v9 + per chunk-planner v22 P2 proc-macro decorator prediction — `RevertRenderPolicy` is a plain struct (no proc-macro decorator), so no dev-dep cascade. `CurrentToolExecution` is NOT directly imported anywhere in baby-phi (it appears only inside the generic `Option<Arc<Mutex<Option<...>>>>` type of the `current_tool` field, set to `None`); the generic parameter is fully-qualified through `phi_core::types::context::CurrentToolExecution` but baby-phi does NOT name it at any baby-phi source site — `current_tool: None` works because Rust infers the `None` type from the `AgentLoopConfig.current_tool` field type. Zero raw-import-line cost; +1 semantic leverage-site for `RevertRenderPolicy` only.

### Positive close-audit greps (these should fire green at chunk close)

1. **Workspace `phi_core` import count preserved**:
   ```bash
   grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l
   ```
   Expected at close: **57** (Δ +0 raw line count; +1 semantic leverage-site via FQN at single field-add site).

2. **Cargo.toml workspace-deps row reads `phi-core = "0.10"`**:
   ```bash
   grep -nE '^phi-core = "0.10"$' /root/projects/phi/baby-phi/Cargo.toml
   ```
   Expected at close: 1 hit at line 17.

3. **Cargo.lock pin against phi-core 0.10.x**:
   ```bash
   grep -nA 1 '^name = "phi-core"' /root/projects/phi/baby-phi/Cargo.lock | grep '^version'
   ```
   Expected at close: `version = "0.10.0"` (or higher 0.10.x if a patch ships during chunk).

4. **2 new fields present at launch.rs `AgentLoopConfig` struct-literal**:
   ```bash
   grep -n 'revert_render_policy: phi_core::types::node_tag::RevertRenderPolicy::default' /root/projects/phi/baby-phi/modules/crates/server/src/platform/sessions/launch.rs
   grep -n 'current_tool: None' /root/projects/phi/baby-phi/modules/crates/server/src/platform/sessions/launch.rs
   ```
   Expected at close: 1 hit each.

5. **CH-25 P-SEAL invariant intact** (`response_format: ResponseFormat::default()`):
   ```bash
   grep -n 'response_format: phi_core::provider::traits::ResponseFormat::default' /root/projects/phi/baby-phi/modules/crates/server/src/platform/sessions/launch.rs
   ```
   Expected at close: 1 hit.

6. **CH-28b carrier-fix invariant intact** (`revert_pending: None`):
   ```bash
   grep -n 'revert_pending: None' /root/projects/phi/baby-phi/modules/crates/server/src/platform/sessions/launch.rs
   ```
   Expected at close: 1 hit (preserved from CH-28b).

7. **CH-28c explicit-arm invariant intact** (`AgentEvent::TurnRequest`):
   ```bash
   grep -nE 'AgentEvent::TurnRequest' /root/projects/phi/baby-phi/modules/crates/cli/src/commands/agent.rs
   ```
   Expected at close: ≥ 1 hit (preserved from CH-28c).

### Forbidden-duplication greps (these MUST be 0 at chunk close)

| Grep | Expected hits | Rationale |
|---|---|---|
| `grep -rn "^pub struct RevertRenderPolicy" /root/projects/phi/baby-phi/modules/crates/` | **0** | Re-defining the new 0.10.0 type duplicates phi-core |
| `grep -rn "^pub struct CurrentToolExecution" /root/projects/phi/baby-phi/modules/crates/` | **0** | Same — duplicates phi-core's new 0.10.0 struct |
| `grep -rn "^pub type BeforeToolExecutionUpdateFn" /root/projects/phi/baby-phi/modules/crates/` | **0** | Re-defining the async-flipped type alias shadows phi-core's |
| `grep -rn "^pub type AfterToolExecutionUpdateFn" /root/projects/phi/baby-phi/modules/crates/` | **0** | Same |
| `grep -rn "^pub fn detect_interpreter" /root/projects/phi/baby-phi/modules/crates/` | **0** | Re-defining duplicates the 0.10.0 `pub`-flipped function |
| `grep -rn "^pub struct AgentLoopConfig" /root/projects/phi/baby-phi/modules/crates/` | **0** | Re-defining shadows phi-core; the carrier-fix path through phi-core's struct is preserved |
| `grep -rn "^pub enum AgentEvent" /root/projects/phi/baby-phi/modules/crates/` | **0** | Preserved from CH-28b/CH-28c |
| `grep -rn "impl phi_core::context::BlockCompactionStrategy for" /root/projects/phi/baby-phi/modules/crates/` | **0** | No custom impls; async-trait migration is a no-op consumer-side (preserved from CH-28c) |

### `check-phi-core-reuse.sh` enforcement

```bash
bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh
```

Expected at close: exit 0. The script enforces the rules above; if any forbidden duplication slips in, the CI guard catches it.

### Pre-spawn verification confirming 2 COMPILE BREAKS at 1 site

| Class | Pre-spawn evidence |
|---|---|
| `AgentLoopConfig.revert_render_policy` new public field | **1 COMPILE BREAK at 1 site**. `grep -rn 'AgentLoopConfig\s*{' modules/crates/` returns exactly 1 hit at `server/src/platform/sessions/launch.rs:526`. Must add `revert_render_policy: phi_core::types::node_tag::RevertRenderPolicy::default()` to the literal. Default preserves byte-for-byte pre-0.10 behaviour (linear `build_working_context` path; only takes effect when `active_node_id.is_some()` which baby-phi never sets). |
| `AgentLoopConfig.current_tool` new public field | **1 COMPILE BREAK at 1 site (same site as above)**. Must add `current_tool: None` to the same struct-literal. `None` disables the introspection slot; preserves byte-for-byte pre-0.10 behaviour. |
| `BeforeToolExecutionUpdateFn` / `AfterToolExecutionUpdateFn` async migration | **ZERO breaks**. `grep -rn "BeforeToolExecutionUpdateFn\|AfterToolExecutionUpdateFn" modules/crates/` returns 0 hits. baby-phi has NO references to either type alias; both fields are `None` in the only `AgentLoopConfig` struct-literal. Async-fication is a no-op consumer-side. |
| `BasicAgent::with_revert_render_policy` / `current_tool_timeout` new methods | **ZERO breaks + ZERO opportunity for wire-through this chunk**. baby-phi is per-request stateless (ADR-0034 §D34.6); never instantiates `phi_core::BasicAgent`. Both methods are unreachable from baby-phi's session-launch architecture. Deferred to Axes A + B of `D-PHICORE-10-FOLLOWUP-01`. |
| `detect_interpreter` `pub` visibility flip | **ZERO breaks + ZERO consumer**. `grep -rn "detect_interpreter\|script_callback" modules/crates/` returns 0 hits. baby-phi has no `.sh`/`.py`/`.js` hook-script dispatch path. Deferred to Axis C. |
| `CurrentToolExecution` struct re-export from `crate::context` | **ZERO breaks + ZERO direct consumer**. `grep -rn "CurrentToolExecution" modules/crates/` returns 0 hits. The struct flows through baby-phi only as a generic parameter of the `current_tool` field (set to `None`); no source site names it. Deferred to Axis D. |

---

## §3.B — K8s microservice readiness check (7-axis evaluation)

| Axis | Classification | Code anchor / Rationale |
|---|---|---|
| **A1 — In-process state** | **no impact** | No mutexes / RwLocks / in-memory caches added or removed. `current_tool: None` is a primitive default that disables the introspection slot; if a future chunk flips to `Some(Arc<Mutex<...>>)`, that chunk would be the A1-classification owner (deferred to `D-PHICORE-10-FOLLOWUP-01` Axes A+B+D). |
| **A2 — IPC channels** | **no impact** | No mpsc/broadcast/watch/oneshot channels added or removed. |
| **A3 — Pod-local resources** | **no impact** | No file handles, sockets, or embedded SurrealDB files added. No filesystem operations. |
| **A4 — Migration runner conformance** | **no impact** | ZERO new migrations. Cargo.lock regeneration via `cargo update -p phi-core` is build-tool state, not runtime migration. |
| **A5 — Trait-shape requirement** | **no impact** | No Repository / Storage trait additions. The phi-core trait additions (final 2-of-11 lifecycle Fn async migration) are upstream-internal; baby-phi has zero consumers so no trait-shape conformance work. |
| **A6 — Cross-pod state sharing** | **no impact** | No read-after-write expectations changed. The per-tool introspection surface (when adopted in a future chunk) WILL have cross-pod state symmetry implications (each pod's `current_tool` slot is local), but adoption is deferred. |
| **A7 — Audit hash-chain symmetry** | **no impact** | No `domain::audit::AuditEvent` writes added or removed. No new variant emission; the `current_tool` slot routes to phi-core's internal observation surface (not baby-phi's hash-chain `AuditEvent` writer); no symmetry impact. |

**Verdict**: ✅ all 7 axes resolved at "no impact". **Zero new K8s blocker class.** No `CHK8S-D-NN` ledger entry needed.

---

## §3.C — User-facing documentation impact map (3-tier evaluation)

| Tier | Doc(s) touched | Action | Defer rationale (if applicable) |
|---|---|---|---|
| Architecture | NONE | None this chunk | The phi-core CHANGELOG + debugging.md are upstream concept-doc-of-record; no baby-phi-internal architecture-tier doc requires update for the baseline shift itself. |
| Operations | NONE | None this chunk | No operator-facing flow changes; 2 new field defaults are no-op; adoption is deferred. |
| User-guide | NONE | None this chunk | No end-user-facing surface change. |

**Governance-tier docs** (NOT user-facing per §3.C scope): NEW drift file + ADR-0066 + feature-inventory.md row are first-class deliverables (see §4 + §5 + §7).

---

## §3.D — Forward-scope-vs-concept-doc precedence (added 2026-05-08 per CH-15 retro Row 4, cycle hex `c3f46f17`)

**Forward-scope row literal terms reviewed** against external concept-doc canonical phrasing:

| Term | Source | Verdict |
|---|---|---|
| `phi-core = "0.9"` → `phi-core = "0.10"` | forward-scope §1 line 89 + phi-core CHANGELOG §[0.10.0] (release 2026-05-25) | ✅ Aligns — phi-core 0.10.0 is published; the bump is the canonical absorption form |
| `revert_render_policy: phi_core::types::node_tag::RevertRenderPolicy::default()` + `current_tool: None` at launch.rs:526 | forward-scope §1 line 89 + phi-core CHANGELOG §[0.10.0] | ✅ Aligns — verified at plan-draft that `grep -rn 'AgentLoopConfig\s*{' modules/crates/` returns exactly 1 hit at launch.rs:526; the 2 field-adds land between the existing CH-28b `revert_pending: None,` block (lines 568-576) and the closing `};` at line 577 |
| `D-PHICORE-10-FOLLOWUP-01` drift naming convention | forward-scope §1 line 86 | ✅ Aligns — parallels existing `D-PHICORE-08-FOLLOWUP-01` from CH-28b + `D-PHICORE-09-FOLLOWUP-01` from CH-28c |
| `m6/drifts/` canonical path | forward-scope §1 line 86 + CH-28b/CH-28c precedent | ✅ Aligns — verified via `ls /root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/drifts/` returns the 2 existing PHICORE drift files |
| `ADR-0034 §D34.6` per-request-stateless cross-reference | forward-scope §1 line 84 (Non-technical-user rationale block) + ADR-0034 §D34.6 (existing) | ✅ Aligns — verified ADR-0034 §D34.6 exists + carries the per-request-stateless lock-rationale that justifies `current_tool: None` + Axes A+B+D deferrals |

**No contradictions detected.** Auto-approval blocker (chunk-planner v8 §3.D) does NOT fire.

---

## §3.E — Anticipated gate-2.5 candidates (added 2026-05-11 per CH-24 retro Row 6, cycle hex `5778bb77`; chunk-planner v13)

| Candidate | Surface | Recommendation if surfaced |
|---|---|---|
| **launch.rs `AgentLoopConfig` struct-literal field-order or missing-field discovery** | `server/src/platform/sessions/launch.rs:526-577` carries the existing struct-literal with `response_format: ResponseFormat::default()` (CH-25) + `revert_pending: None` (CH-28b). If P1 cargo build surfaces additional missing fields beyond the predicted 2 (`revert_render_policy` + `current_tool`), it would indicate phi-core 0.10.0 shipped an unannounced breaking change OR pre-spawn `grep` missed a field. | PAUSE via AskUserQuestion if the cargo build error report shows ≥ 3 missing-field errors at line 526 OR mentions any field name not in `{revert_render_policy, current_tool}`. Cross-reference phi-core's `src/agent_loop/config.rs` to verify field-set; widen D3 deliverable if necessary; escalate fork to user. |
| **Async-trait dev-dep addition (per chunk-planner v22 P-plan-2 proc-macro decorator prediction)** | If any test stub in baby-phi grows to impl a phi-core async-trait, it would need `async-trait = "0.1"` in `[dev-dependencies]`. | Pre-spawn `grep` confirmed ZERO such impls. `async-trait = "0.1"` is ALREADY in baby-phi's workspace deps per the M5/P4 wave for the daemon's async traits. Predicted P-NONE; if a test stub appears at P2, ratify at gate-2.5. |
| **`cargo update -p phi-core` unexpected transitive churn** | If `cargo update` brings additional dep version churn beyond phi-core itself (e.g., a security advisory triggers a tokio bump OR thiserror major version bump), Cargo.lock diff would exceed the predicted ≤ 50 lines. | PAUSE via AskUserQuestion if Cargo.lock diff > 100 lines OR brings a non-patch-level transitive bump that surfaces clippy warnings. Cross-check phi-core's Cargo.toml + transitive resolution; ratify at gate-2.5 if benign (e.g., serde patch); escalate if behaviour-changing. |

---

## §3.F — SurrealDB SCHEMAFULL semantic checklist (per chunk-planner v25 P-plan-1)

**N/A — CH-28d ships zero migrations + zero SurrealDB schema changes.** No `REMOVE FIELD` / `ALTER TABLE` narrowing / new SCHEMAFULL table / narrowing-UNIQUE-index changes. The §3.F checklist does not apply.

---

## §4 — Drifts closed + Deferred functionality

**Drifts closed**: **NONE this chunk.** CH-28d files a NEW drift `D-PHICORE-10-FOLLOWUP-01-new-api-adoption-cluster.md` but closes none.

**NEW drift filed**:

### D-PHICORE-10-FOLLOWUP-01 — phi-core 0.10.0 new-API adoption cluster (4 axes A/B/C/D deferred to future M7+ chunk)

- **Phase of origin**: CH-28d P3 chunk-seal (2026-05-25) — filed per F4.a planner-rec lock at gate-1.5 + ADR-0066 §D66.4 deferral decision.
- **Discovery source**: phi-core 0.10.0 breaking-change release (2026-05-25) — 5 surfaces shipped (`AgentLoopConfig` gains 2 new public fields + `BasicAgent` 2 new methods + 2 type-alias async-flips + `detect_interpreter` `pub`-flip + `CurrentToolExecution` re-export); 4 of the 5 surfaces are out-of-reach for baby-phi's per-request-stateless architecture (per ADR-0034 §D34.6).
- **Status**: `discovered`.
- **Bucket**: B — follow-on engine-scope widening (adopting 4 axes A/B/C/D would shift baby-phi from per-request-stateless to long-lived-agent architecture OR add hook-script dispatch path OR add pause/cancel surface).
- **Severity**: LOW.
- **Tags**: `phi-core-0.10`, `revert-render-policy`, `current-tool-introspection`, `detect-interpreter-shim`, `current-tool-execution-consumer`, `m7-future`.
- **Blocks**: nothing within CH-28d; the dependency baseline shift + 2 field-adds + zero-impact verification together close the breaking-change absorption.
- **Blocked-by**: nothing — CH-28d ships the dependency baseline shift which unblocks adoption. All 4 axes' adoption prerequisites are inside baby-phi's source tree + architectural-shift decisions.
- **Closing chunk**: **M7+-FUTURE-ARCHITECTURE-SHIFT-OR-OBSERVABILITY-ADOPTION** (placeholder; per chunk-planner v13 non-terminal-drift rule the placeholder names an explicit-named-future-allocation; no specific CH-NN slot reserved at CH-28d close).

**4-axis enumeration** (within the drift body):

- **Axis A — `BasicAgent::with_revert_render_policy` live wire-through**. Prerequisite: baby-phi instantiates `phi_core::BasicAgent` per request (currently does NOT — per ADR-0034 §D34.6 baby-phi is per-request stateless + the `domain::Agent` governance principal flows into `phi_core::types::context::AgentContext.agent_id` via ID-only at `sessions/provider.rs::build_agent_context`). Revisit only if a future milestone introduces long-lived in-memory chat agents. **User-visible-final**: operator can tune `Lesson`/`Note` decay-window policy via baby-phi config TOML.
- **Axis B — `BasicAgent::current_tool_timeout(&self) -> Option<Duration>` pause-time tool-timeout introspection**. Prerequisite: Axis A (long-lived-agent shape) + a baby-phi pause/cancel surface that doesn't exist today. **User-visible-final**: `PauseAcknowledged.estimated_completion_ms` UI surfaces; operator can see remaining tool-execution time before pause completes.
- **Axis C — `phi_core::agent_loop::script_callback::detect_interpreter` `pub`-shim adoption**. Out-of-scope at v0: baby-phi has no `.sh`/`.py`/`.js` hook-script dispatch path (verified via `grep -rn "script_callback\|detect_interpreter" modules/crates/` returns 0). **User-visible-final**: hooks can dispatch via interpreted scripts (shell/Python/JS) instead of binary executables only.
- **Axis D — `CurrentToolExecution` struct standalone consumption (separate from the `current_tool` slot in `AgentLoopConfig` which is taken-but-not-wired this chunk)**. Depends on Axis A + Axis B. **User-visible-final**: per-tool-call observation surface for operators / debug UIs.

- **Remediation scope** (estimate only): adoption decomposes into 4 axis-specific tracks. Axes A+B+D share the long-lived-agent architectural prerequisite (~5-10 ed for the long-lived-agent shift alone, ignoring per-axis wiring). Axis C is independent (~1-2 ed for hook-script dispatch path). Aggregate estimate: ~7-15 ed for full closure; partial closures (Axis C alone OR Axes A+B together OR all-of-A+B+D together) are independently schedulable.

**Cross-references** within the drift: phi-core CHANGELOG §[0.10.0]; `phi-core/docs/concepts/debugging.md` per-tool-call introspection extension; i-phi precedent absorption at `phi-core-0.10-absorption-1baf96ae/plan.md` (where Axes A/B/C/D are all live-wired since i-phi instantiates `phi_core::BasicAgent` per session task); baby-phi ADR-0034 §D34.6 (per-request-stateless architectural lock that justifies Axes A+B+D deferrals).

**Async-Fn migration adoption deferral**: N/A — baby-phi has zero references to `BeforeToolExecutionUpdateFn` / `AfterToolExecutionUpdateFn` (verified via grep). Async-fication is a no-op consumer-side with no follow-on adoption work needed. The drift covers ONLY the 4 new-API adoption axes.

---

## §5 — ADRs drafted

**NEW ADR**: `0066-phi-core-10-absorption.md` at `/root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/decisions/`.

**Status at chunk seal**: Proposed at P3 draft; flips to **Accepted** at P-SEAL.

**Authors**: Claude Code (orchestrator + chunk-planner v31 + chunk-implementer v18).

**Canonical ADR sections (per chunk-planner v17 P2 explicit ADR-section enumeration)**:

1. **Forks** — header table; F1 RESOLVED at prerequisite commit + F2 planner-rec + F3 planner-rec + F4 planner-rec form (paralleling CH-28b §"Forks" + CH-28c §"Forks").
2. **Context** — chunk-graph + forward-scope citations + downstream consumer enumeration (CH-29..CH-38) + i-phi precedent absorption provenance + ADR-0034 §D34.6 per-request-stateless architectural lock context.
3. **Sub-decisions** — one `### §D66.<M>` per fork resolution + supporting decisions; each ends with a Pre-existing-behaviour preservation note (or never-shipped-yet variant per chunk-planner v11 P1 + v24 P-plan-2 for net-new surfaces).
4. **Cross-references** — 4 categories: (a) concept-doc + line range (phi-core CHANGELOG §[0.10.0] + debugging.md); (b) closed drifts (none); (c) prior ADRs as precedent — **ADR-0065** (CH-28c 0.9.0 absorption — directly-parallel pattern), **ADR-0064** (CH-28b 0.8.0 absorption — directly-parallel pattern), **ADR-0059 §D59.2** (CH-25 P-SEAL `response_format` carrier-fix precedent at the same `launch.rs:526` struct-literal site), **ADR-0034 §D34.6** (per-request-stateless architectural lock that justifies `current_tool: None` + Axes A+B+D deferrals); (d) forward-scope row.
5. **Consequences** — `### For CH-29..CH-38` subsection per downstream consumer enumeration + `### For M7+ architecture-shift OR observability chunk` subsection forward-routing to `D-PHICORE-10-FOLLOWUP-01` closing chunk.
6. **Revisit triggers** — 5-6 bullets each citing a specific §D66.<M> that would warrant re-opening (e.g., phi-core 0.11.0 release ⟶ revisit §D66.1 dep-string-form; baby-phi adopts long-lived-agent architecture ⟶ revisit §D66.3 `current_tool: None` default; new in-baby-phi tool-update Fn consumer wants async semantics ⟶ revisit §D66.5 async-Fn-migration-no-op claim).
7. **Verification** — commands the reviewer can run to replay verification (mirrors §12 below).

**Sub-decisions** (5 total per planner outline):

- **§D66.1 — F2.a `phi-core = "0.10"` semver-range form** (chosen over F2.b exact-pin + F2.c semver-explicit). Pre-existing-behaviour preservation note: pre-existing scaffold preserved — workspace `Cargo.toml:17` already used semver-range form `phi-core = "0.9"` since CH-28c close (paralleling 0.7.1 → 0.8 → 0.9 shift); the shift to `"0.10"` continues the single-form-precedent established at CH-28b §D64.1 + CH-28c §D65.1 (3rd consecutive activation).
- **§D66.2 — F1.b CH-28d interstitial chunk-numbering** (continues CH-28b/CH-28c CH-NNa/b/c precedent — 3rd consecutive). Pre-existing-behaviour preservation note: pre-existing scaffold preserved — the CH-NNa/b/c suffix convention was established at CH-28b §D64.2 as canonical for "interstitial chunks that absorb upstream breaking-changes without renumbering downstream cycles"; CH-28d is the third activation of the convention, validating durability under repeated activation.
- **§D66.3 — F3.a 2 new fields at launch.rs:526 with no-op defaults via FQN path + 11-line carrier-fix comment block** (mirrors CH-25 `response_format` + CH-28b `revert_pending` precedent at the same struct-literal site). Pre-existing-behaviour preservation note: pre-existing scaffold preserved — `launch.rs:526` `AgentLoopConfig` struct-literal already carried 2 prior carrier-fix entries (CH-25 `response_format` ~10 LOC + CH-28b `revert_pending` ~9 LOC); CH-28d's 2 field-adds + 11-line comment continue the precedent. The fully-qualified `phi_core::types::node_tag::RevertRenderPolicy::default()` path avoids adding a `use phi_core::types::node_tag::RevertRenderPolicy;` line at the file head (preserving the leverage-site count at 57 raw lines); `current_tool: None` uses Rust's type-inference from the field type (preserving the same property for the `CurrentToolExecution` surface). ADR-0034 §D34.6 cross-reference explicitly justifies `current_tool: None` (baby-phi's per-request-stateless architecture rules out the introspection slot's primary use case).
- **§D66.4 — F4.a 1 drift `D-PHICORE-10-FOLLOWUP-01` filed for 4-axis deferred-adoption cluster**. Pre-existing-absence preserved (never-shipped-yet variant per chunk-planner v24 P-plan-2): no prior 0.10.0 new-API surface exists in baby-phi; CH-28d absorbs the type-space without enabling any of the 4 axes. Future adoption chunks (placeholder `M7+-FUTURE-ARCHITECTURE-SHIFT-OR-OBSERVABILITY-ADOPTION`) decide per-axis wiring + the architectural-shift prerequisites Axes A+B+D share.
- **§D66.5 — Async-Fn migration is a no-op consumer-side; no adoption deferral needed**. Pre-existing-behaviour preservation note: pre-existing scaffold preserved — baby-phi has ZERO consumers of `BeforeToolExecutionUpdateFn` / `AfterToolExecutionUpdateFn` type aliases (verified pre-spawn via grep; both fields are `None` at the only `AgentLoopConfig` struct-literal site at `launch.rs:548-549`). phi-core 0.10.0 async-Fn migration is internal-only to phi-core's type-alias shapes; no consumer-side migration work required at CH-28d OR any future chunk (the no-op nature does not warrant a drift). If a future chunk adds a custom tool-update Fn consumer, that chunk MUST use the async-future shape per phi-core's 0.10.0 type alias; revisit §D66.5 if such a consumer materializes. The CH-28c precedent §D65.5 established the same no-op rule for the 0.9.0 9-of-11 async-trait migration; CH-28d §D66.5 closes the final 2-of-11 migration with the same rule.

---

## §6 — Prior-chunk regression re-verification

**Carry-forward invariants** (must remain green at chunk close):

| Invariant | Source chunk | Verification command | Expected |
|---|---|---|---|
| Workspace test count | CH-28c close baseline | `cargo test --workspace --no-fail-fast -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml` | **1599 passed / 0 failed** |
| phi-core import baseline | CH-28c close baseline | `grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l` | **57** |
| CH-25 P-SEAL invariant | `launch.rs` `response_format: ResponseFormat::default()` | `grep -n 'response_format: phi_core::provider::traits::ResponseFormat::default' modules/crates/server/src/platform/sessions/launch.rs` | ≥ 1 hit |
| CH-28b carrier-fix invariant | `launch.rs` `revert_pending: None` | `grep -n 'revert_pending: None' modules/crates/server/src/platform/sessions/launch.rs` | ≥ 1 hit |
| CH-28b explicit-arm invariant | `cli/agent.rs` `AgentEvent::RevertApplied { .. } => {}` | `grep -nE 'AgentEvent::RevertApplied' modules/crates/cli/src/commands/agent.rs` | ≥ 1 hit |
| CH-28c explicit-arm invariant | `cli/agent.rs` `AgentEvent::TurnRequest { .. } => {}` | `grep -nE 'AgentEvent::TurnRequest' modules/crates/cli/src/commands/agent.rs` | ≥ 1 hit |
| CH-28 acceptance suite | 7 m6 agent profile cardinality tests | `cargo test --workspace --test acceptance_m6_agent_profile_cardinality --manifest-path /root/projects/phi/baby-phi/Cargo.toml -j 4` | 7 passed |
| CI guards | 4 scripts/check-*.sh | `bash /root/projects/phi/baby-phi/scripts/check-{doc-links,ops-doc-headers,phi-core-reuse,spec-drift}.sh` (4 invocations) | all exit 0 |

**Named expected-still-green tests** (carry-forward; per chunk-planner v17 P-plan-3 grep-verify against existing test inventory):

- `test_session_recorder_emits_session_started_on_agent_start_for_ctx` (CH-17 acceptance)
- `test_acceptance_m6_agent_profile_cardinality_create_with_template` (CH-28 acceptance)
- `test_acceptance_m6_agent_profile_cardinality_override_upsert` (CH-28 acceptance)
- `test_launch_session_with_default_runtime_config` (CH-25 acceptance)
- All 1599 carry-forward (workspace `cargo test` exit 0 is the comprehensive gate).

**Back-compat-preservation decision-prompt** (per chunk-planner v19 P2): N/A — no fork structurally removes any load-bearing scaffold; all 4 forks are additive (Cargo bump + 2 field-adds + drift + ADR). The 2 new `AgentLoopConfig` fields default to no-op shapes that preserve byte-for-byte pre-0.10 behaviour at the single call site.

---

## §7 — Phases within the chunk

**Phase count**: **4 phases (P1 + P2 + P3 + P-SEAL)** — matches CH-28c's phase count exactly. P3 absorbs ADR + drift + feature-inventory + verified-header amends (collapsed from CH-28b's separate P3 + P4 phases per CH-28c precedent).

### P1 — Cargo bump + lock update + workspace-build verification (EXPECTED 2 compile errors)

- **Goal**: shift workspace baseline from `phi-core = "0.9"` to `phi-core = "0.10"`; regenerate Cargo.lock against published phi-core 0.10.x; verify workspace surfaces exactly the predicted 2 compile errors at launch.rs:526.
- **Deliverables**:
  1. Edit `/root/projects/phi/baby-phi/Cargo.toml` line 17: `phi-core = "0.9"` → `phi-core = "0.10"`.
  2. Run `/root/rust-env/cargo/bin/cargo update -p phi-core --manifest-path /root/projects/phi/baby-phi/Cargo.toml` — regenerates Cargo.lock entries for phi-core (and any transitive dep version changes phi-core 0.10.0 introduces).
  3. Run `/root/rust-env/cargo/bin/cargo build --workspace -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml` — **EXPECT 2 compile errors at `server/src/platform/sessions/launch.rs:526`** reporting missing fields `revert_render_policy` + `current_tool` in the `AgentLoopConfig` struct-literal initializer. These errors are the predictable surface; proceed to P2 to close them.
- **Tests**: 0 new tests; existing workspace build via cargo build.
- **Concept-alignment check**: §2 row `phi-core/CHANGELOG.md §[0.10.0]` advances `not-yet-absorbed` → `partially-honored` (baseline shifted; pending P2 carrier-fix).
- **phi-core leverage check**: baseline grep returns 57 (preserved).
- **User-facing doc updates**: N/A.
- **Confidence target**: ≥ 99% (mechanical baseline shift; the surface is fully characterized by pre-spawn verification).
- **Pause discipline**: PAUSE via AskUserQuestion IF the compile error count is NOT exactly 2 (e.g., 0 errors → phi-core 0.10.0 shipped a non-breaking surface AND the predicted breaking fields were always optional; 3+ errors → phi-core shipped an unannounced breaking change). PAUSE IF the missing-field names are NOT exactly `{revert_render_policy, current_tool}`. PAUSE IF `cargo update` brings additional unexpected dep version churn (e.g., a security advisory triggers a tokio bump).

### P2 — 2 new field-adds at AgentLoopConfig struct-literal (closes P1 errors)

- **Goal**: append 2 new field-adds + 11-line carrier-fix comment block at `launch.rs:526-577` `AgentLoopConfig` struct-literal; workspace returns GREEN; full test suite re-verifies at baseline 1599 tests passed.
- **Deliverables**:
  1. Edit `/root/projects/phi/baby-phi/modules/crates/server/src/platform/sessions/launch.rs` between line 576 (end of existing `revert_pending: None,` block) and line 577 (closing `};`): insert the 11-line carrier-fix comment block + 2 field-add lines per F3.a:
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
     Net diff: +13 LOC (2 field-add lines + 11-line comment block; the existing closing `};` at line 577 shifts down by ~13 lines).
- **Tests**: 0 new tests; full workspace cargo build + cargo test re-verification.
  - `/root/rust-env/cargo/bin/cargo build --workspace -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml` — expect GREEN exit 0.
  - `/root/rust-env/cargo/bin/cargo test --workspace --no-fail-fast -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml` — expect 1599 passed / 0 failed.
- **Concept-alignment check**: §2 row `phi-core/CHANGELOG.md §[0.10.0]` advances `partially-honored` → `honored`.
- **phi-core leverage check**: baseline grep returns 57 (Δ +0 raw line count; +1 semantic leverage-site via FQN at single field-add site for `RevertRenderPolicy` only).
- **User-facing doc updates**: N/A.
- **Confidence target**: ≥ 99% (mechanical carrier-fix; parallels CH-25 `response_format` + CH-28b `revert_pending` precedents at the same site).
- **Pause discipline**: PAUSE via AskUserQuestion IF cargo build fails (the predicted carrier-fix-closes-errors outcome falsified — e.g., field-types differ from prediction) OR IF cargo test count diverges from 1599 (regression introduced unexpectedly) OR IF clippy surfaces a warning about the new field placement.
- **Cargo cleanup**: `/root/rust-env/cargo/bin/cargo clean --manifest-path /root/projects/phi/baby-phi/Cargo.toml` per chunk-implementer v8 placement-1 (immediate-post-test cleanup).

### P3 — Docs + drift + ADR + feature-inventory + verified-header amends

- **Goal**: file NEW drift `D-PHICORE-10-FOLLOWUP-01`; add feature-inventory.md §3 row; draft ADR-0066 (Status: Proposed); amend verified headers on all touched docs.
- **Deliverables**:
  1. Create NEW file `/root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/drifts/D-PHICORE-10-FOLLOWUP-01-new-api-adoption-cluster.md` mirroring the existing `D-PHICORE-09-FOLLOWUP-01-per-turn-debug-capture-adoption.md` shape (verified-header line 1; `## Identification` section with all fields per §4 above; `## Concept alignment` citing phi-core CHANGELOG §[0.10.0] + debugging.md + ADR-0034 §D34.6; `## Plan vs reality` documenting CH-28d's deferral routing per §4 4-axis enumeration; `## Where visible in code` documenting the 4 grep regression targets — one per axis showing the absorbed-but-not-wired posture at chunk close; `## Remediation scope` documenting the ~7-15 ed aggregate adoption estimate decomposed by axis cluster). Status: `discovered`. Bucket: B. Severity: LOW. Closing chunk: **M7+-FUTURE-ARCHITECTURE-SHIFT-OR-OBSERVABILITY-ADOPTION** placeholder.
  2. Edit `/root/projects/phi/baby-phi/docs/specs/v0/feature-inventory.md` §3 Deferred catalogue: add a new `### D-PHICORE-10-FOLLOWUP-01 — phi-core 0.10.0 new-API adoption cluster (4 axes: revert-render-policy / current-tool-introspection / detect-interpreter-shim / current-tool-execution-consumer)` row IMMEDIATELY AFTER the existing `D-PHICORE-09-FOLLOWUP-01` row (from CH-28c). 5 fields: Feature impact / User-visible state in v0 / User-visible state at final / Allocation chunk / Cross-chunk dependency. Bump §1 verified header date to 2026-05-25 with CH-28d citation.
  3. Create NEW ADR file `/root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/decisions/0066-phi-core-10-absorption.md` with the 7 canonical sections per §5 above; sub-decisions §D66.1–§D66.5; Status: **Proposed** at P3 (flips to **Accepted** at P-SEAL per chunk-implementer v17 ADR Status flip discipline at P-SEAL).
  4. Verified-header presence check on `/root/projects/phi/baby-phi/docs/specs/plan/forward-scope/m6-forward-scope-8b7a8bcd.md` — the CH-28d forward-scope row was inserted as a prerequisite-commit at `af037cf` 2026-05-25 BEFORE chunk-planner spawn (per F1.b lock + chunk-initiate Option A). Confirm the existing authoring-time verified-header at the file's line 1 carries the CH-28d authoring citation; no body changes (the row is the authoring artifact).
- **Tests**: 0 new code tests. Doc-links check (`bash /root/projects/phi/baby-phi/scripts/check-doc-links.sh`) MUST pass after edits.
- **Concept-alignment check**: §2 rows for `phi-core CHANGELOG §[0.10.0]` + `debugging.md` + `i-phi precedent` + `ADR-0034 §D34.6` all reach target status (`honored` for CHANGELOG; `concept-aspirational-preserved` for debugging.md; `confirmed-low-impact` for i-phi precedent; `honored at CH-28d close` for ADR-0034 §D34.6).
- **phi-core leverage check**: baseline grep returns 57 (unchanged).
- **User-facing doc updates**: per §3.C deferred (3 tiers all N/A). The NEW drift file + ADR-0066 + feature-inventory row are GOVERNANCE-tier docs (not user-facing-tier per §3.C definition); they ARE first-class deliverables.
- **Confidence target**: ≥ 99% (paperwork pattern matches CH-28b + CH-28c precedent exactly).
- **Pause discipline**: PAUSE via AskUserQuestion IF doc-links check fails OR feature-inventory.md edit cascades unexpectedly (e.g., §3 row count change breaks a §2 cross-ref) OR IF ADR-0066 cross-reference to ADR-0065 / ADR-0064 / ADR-0059 / ADR-0034 fails the `check-doc-links.sh` script (relative-path resolution).

### P-SEAL — Verified-header sweep + cycle-index Status flip + ADR-0066 Accepted flip

- **Goal**: chunk-seal paperwork; cycle-index row appended at active-cycles tail (left as `in-flight` per chunk-planner v16 P-SEAL canonical lifecycle — orchestrator owns transitions); ADR flipped Proposed → Accepted; verified-header amends on all touched docs.
- **Deliverables**:
  1. Append cycle-index row at `/root/projects/phi/baby-phi/docs/specs/plan/build/_cycle-index.md` (per chunk-implementer P-SEAL v17 + chunk-archive-plan skill v3):
     - Hex link: `[`<8hex>`](phi-core-10-absorption-<8hex>/plan.md)` (orchestrator-minted hex)
     - Slug + summary: `CH-28d — phi-core 0.10.0 absorption (RevertRenderPolicy + CurrentToolExecution AgentLoopConfig field-additions + final async-Fn migration; baseline shift; TECHNICAL-PREREQUISITE); 4 phases (P1+P2+P3+P-SEAL); 2 carrier-fixes at launch.rs:526 [revert_render_policy + current_tool] + 11-line comment block; NEW drift D-PHICORE-10-FOLLOWUP-01 filed (4-axis cluster); ADR-0066 Accepted with 5 sub-decisions §D66.1–§D66.5; phi-core import baseline preserved at 57 raw lines; workspace test count preserved at 1599`
     - Phase count: `4`
     - Auditor count: `1 (audit envelope: SMALL per §11)`
     - Iterations: `pending` (per chunk-planner v16 P-SEAL canonical lifecycle — leave Iterations = pending and Status = in-flight; orchestrator owns the transitions per `_cycle-index.md` row-lifecycle paragraph: gate-3 → ready-for-audit; gate-4 close → audited-pending-retro; Phase 6 / Phase 7 close → retro-complete + Iterations to final count)
     - Status: `in-flight` (per chunk-planner v16 P-SEAL canonical lifecycle — orchestrator transitions at gate-3/gate-4/retro-complete)
     - Test count: `1599`
  2. Flip ADR-0066 status header: `Proposed` → `Accepted` at top of `0066-phi-core-10-absorption.md`; bump verified header on the ADR file.
  3. Amend verified headers on: `_cycle-index.md` (row 1 + row 2 comment lines), `feature-inventory.md`, NEW `D-PHICORE-10-FOLLOWUP-01-new-api-adoption-cluster.md`. Forward-scope file already prerequisite-committed at `af037cf`; no header amend needed unless its line-1 comment requires refresh.
  4. Run final verification suite per §12 below: 4 CI guards + cargo fmt --check + cargo test --workspace --no-fail-fast (expect 1599 passed) + chunk-specific greps.
  5. `cargo clean` per chunk-implementer v8 placement-1 (final placement-1 invocation; orchestrator owns placement-2 at gate-5 close).
- **Tests**: 0 new tests; full verification suite re-run.
- **Concept-alignment check**: all §2 rows at target status.
- **phi-core leverage check**: baseline grep returns 57; positive close-audit greps all green per §3.
- **User-facing doc updates**: NONE.
- **Confidence target**: ≥ 99% (composite ≥ 9/10 per §10 below).
- **Pause discipline**: PAUSE via AskUserQuestion IF cycle-index row append fails OR any verified-header amend conflicts OR any §6 carry-forward invariant regresses OR cargo test count diverges from 1599.

---

## §8 — Tests summary

**Expected total test count at chunk close**: **1599** (baseline from CH-28c close; ZERO new tests added at CH-28d — chunk is a baseline-shift + 2 mechanical field-adds + paperwork; no new behaviour to test).

**Test-count band** (per chunk-planner v17 + v22 P-plan-1 per-Tier breakdown): `[1599, 1599]` (point estimate; no buffer needed — the chunk adds 0 NEW MUST-SHIP + 0 MAY-COVER tests + 0 inline `#[cfg(test)]` additions).

**Pause condition**:
- If test count > 1599 → AskUserQuestion (unexpected test additions; likely indicates scope creep or hidden behavior change).
- If test count < 1599 → AskUserQuestion (regression; tests should remain stable at baseline).

**MUST-SHIP test enumeration**: NONE this chunk.

**MAY-COVER test enumeration**: NONE this chunk.

**Per-Tier test-cardinality breakdown** (per chunk-planner v22 P-plan-1) — N/A this chunk (no new tests; existing tier distribution preserved).

**Layer breakdown** (no shift from CH-28c close):
- unit: per existing distribution
- integration: per existing distribution
- acceptance: 7 m6_agent_profile_cardinality + all prior
- e2e: per existing distribution

**Named expected-still-green tests** (carry-forward; per chunk-planner v17 P-plan-3 grep-verify):
- `test_session_recorder_emits_session_started_on_agent_start_for_ctx`
- `test_acceptance_m6_agent_profile_cardinality_create_with_template`
- `test_acceptance_m6_agent_profile_cardinality_override_upsert`
- `test_launch_session_with_default_runtime_config`

---

## §9 — Pre-chunk gate

**Reading list (mandatory)** — the implementer reads BEFORE P1 opens:

1. **Forward-scope row**: [`forward-scope/m6-forward-scope-8b7a8bcd.md`](../../forward-scope/m6-forward-scope-8b7a8bcd.md) §1 lines 82–92 + §5 table (CH-28d row).
2. **Plan-mode plan**: [`/root/.claude/plans/sharded-discovering-stearns.md`](/root/.claude/plans/sharded-discovering-stearns.md) — user-approved pre-cycle plan covering all 7 deliverables + 4 forks + chunk phases + expected metrics.
3. **phi-core CHANGELOG**: [`/root/projects/phi/phi-core/CHANGELOG.md`](../../../../../../phi-core/CHANGELOG.md) §[0.10.0] (release 2026-05-25; 5 surface enumerations: 2 AgentLoopConfig fields + 2 BasicAgent methods + 2-of-11 async Fn migration + detect_interpreter pub-flip + CurrentToolExecution re-export).
4. **phi-core concept-debugging**: [`/root/projects/phi/phi-core/docs/concepts/debugging.md`](../../../../../../phi-core/docs/concepts/debugging.md) (design source-of-truth for per-tool-call introspection extension).
5. **i-phi precedent absorption**: [`/root/projects/phi/i-phi/docs/v0/proposal/plan/build/phi-core-0.10-absorption-1baf96ae/plan.md`](../../../../../../i-phi/docs/v0/proposal/plan/build/phi-core-0.10-absorption-1baf96ae/plan.md) — i-phi closed 0.10.0 absorption with 4 drift closures + 3 NEW MUST-SHIP tests + BrakingConfig live wire-through; baby-phi absorption is much smaller per architectural difference.
6. **CH-28c ADR-0065 precedent**: [`docs/specs/v0/implementation/m6/decisions/0065-phi-core-09-absorption.md`](../../../v0/implementation/m6/decisions/0065-phi-core-09-absorption.md) — direct parallel pattern; ADR-0066 mirrors the section shape + sub-decision body conventions exactly.
7. **CH-28b ADR-0064 precedent**: [`docs/specs/v0/implementation/m6/decisions/0064-phi-core-08-absorption.md`](../../../v0/implementation/m6/decisions/0064-phi-core-08-absorption.md) — secondary parallel pattern + the `revert_pending` carrier-fix-comment-block precedent.
8. **CH-28c drift precedent**: [`docs/specs/v0/implementation/m6/drifts/D-PHICORE-09-FOLLOWUP-01-per-turn-debug-capture-adoption.md`](../../../v0/implementation/m6/drifts/D-PHICORE-09-FOLLOWUP-01-per-turn-debug-capture-adoption.md) — drift file shape + naming baseline for D-PHICORE-10-FOLLOWUP-01.
9. **CH-28c P-SEAL plan**: [`docs/specs/plan/build/ch-28c-phi-core-09-absorption-40214078/plan.md`](../ch-28c-phi-core-09-absorption-40214078/plan.md) §11 (the audit-A scaffold template that CH-28d §11 below mirrors).
10. **ADR-0034 §D34.6**: [`docs/specs/v0/implementation/m6/decisions/0034-domain-agent-and-phi-core-agent.md`](../../../v0/implementation/m6/decisions/0034-domain-agent-and-phi-core-agent.md) — per-request-stateless architectural lock that justifies `current_tool: None` + Axes A+B+D deferrals (cited explicitly in ADR-0066 §D66.3 + the launch.rs carrier-fix comment block).
11. **feature-inventory.md §3 Deferred catalogue** (existing `D-PHICORE-09-FOLLOWUP-01` row from CH-28c) — row shape baseline.
12. **launch.rs:526-577**: [`modules/crates/server/src/platform/sessions/launch.rs`](../../../../modules/crates/server/src/platform/sessions/launch.rs) — the `AgentLoopConfig` struct-literal carrying the existing CH-25 `response_format` block (lines 558-567) + CH-28b `revert_pending: None` block (lines 568-576); the 2 new CH-28d field-adds land between line 576 and 577.
13. **baby-phi CLAUDE.md** §"phi-core Leverage" rules 1–5.

**Carry-forward invariants** (explicit list, verified green at chunk open):
- `cargo test --workspace --no-fail-fast` test count = **1599** (CH-28c close baseline).
- `scripts/check-phi-core-reuse.sh` green.
- `scripts/check-doc-links.sh` green.
- `scripts/check-ops-doc-headers.sh` green.
- `scripts/check-spec-drift.sh` green.
- `grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l` = **57**.
- `modules/` diff against the chunk-open git HEAD is empty (no source changes pending; only the prerequisite-commit `af037cf` for forward-scope row authoring is in place).
- Cargo.toml workspace-deps row: `phi-core = "0.9"` (will flip to `phi-core = "0.10"` at P1).

**Pending decisions carried into this chunk**:
- F1 / F2 / F3 / F4 forks all pre-resolved to planner-rec at gate-1 per orchestrator's pre-decided locks in the plan-mode plan; rubber-stamp expected at gate-1.5 (F1 is RESOLVED at prerequisite commit `af037cf`; F2 + F3 + F4 are planner-rec rubber-stamps expected).
- §3.E candidate 1 (launch.rs missing-field discovery beyond 2 predicted) ratified at gate-1 OR escalated at gate-2.5 if P1 cargo build surfaces additional missing fields.

**Chunk-ordering note**: per forward-scope row line 88, CH-28d has **CH-28c as prerequisite** (cycle `40214078`, closed 2026-05-24 commit `8fc2018` — confirmed via the cycle-index row + git log HEAD context for the baby-phi submodule bump).

---

## §10 — Close criteria

**Composite 4-aspect + 2 confidence % ritual**:

**4 aspects** (each pass/fail):

- **Code aspect**:
  - All 7 deliverables (D1–D7 per forward-scope row line 89) shipped per §7 phases.
  - `/root/rust-env/cargo/bin/cargo test --workspace --no-fail-fast -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml` returns **1599 passed / 0 failed**.
  - `RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy -j 4 --workspace --all-targets --manifest-path /root/projects/phi/baby-phi/Cargo.toml` returns 0 warnings (marked `NOT-EXECUTED-IN-AUDIT` by sub-agents per outer CLAUDE.md gate-4 MUST-RUN; orchestrator closes at gate-4 MUST-RUN list).
  - `/root/rust-env/cargo/bin/cargo fmt --all -- --check --manifest-path /root/projects/phi/baby-phi/Cargo.toml` returns 0 diff.
- **Docs aspect**:
  - *Governance tier*: NEW drift `D-PHICORE-10-FOLLOWUP-01` filed at canonical `m6/drifts/`; ADR-0066 flipped Proposed → Accepted at P-SEAL; feature-inventory.md §3 row added; cycle-index row appended; verified-header amends on all 4 touched docs (cycle-index row 1 + feature-inventory.md §1 + NEW drift + NEW ADR).
  - *User-facing tier* (per §3.C): all 3 tiers explicitly N/A with defer-decision (governance-only chunk).
- **phi-core leverage aspect**:
  - `grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l` returns **57** (Δ +0 raw line count; +1 semantic leverage-site via FQN at single field-add site).
  - `bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh` exits 0 (marked `NOT-EXECUTED-IN-AUDIT` by sub-agent; orchestrator closes at gate-4 MUST-RUN).
  - Forbidden-duplication greps per §3 all return 0 hits.
- **Concept-alignment aspect**:
  - All §2 rows reach target status (`honored` for CHANGELOG; `concept-aspirational-preserved` for debugging.md; `confirmed-low-impact` for i-phi precedent; `honored at CH-28d close` for ADR-0034 §D34.6).
  - No new concept-doc contradictions surfaced; the forward-scope-vs-concept-doc precedence §3.D check returns 0 contradictions.

**2 confidence percentages**:
- **Implementation confidence**: **9.5/10 = 95%** (`claims-honored / claims-in-scope ≥ 9.5/10`). Per chunk-planner pre-spawn verification: exactly 2 COMPILE BREAKS at 1 site predicted (vs CH-28c's 0 breaks + CH-28b's 1 break); structural deliverables D1–D7 mirror CH-28b + CH-28c pattern; ADR + drift bodies follow CH-28b/CH-28c §"Forks" + §"Sub-decisions" + §"Cross-references" + §"Revisit triggers" template exactly.
- **Documentation confidence**: **4/4 = 100%** (governance-tier docs are mechanical — drift file mirrors `D-PHICORE-09-FOLLOWUP-01` shape; ADR mirrors `0065-phi-core-09-absorption.md` shape; feature-inventory row mirrors existing row; cycle-index row mirrors existing row).

**Composite**: 95% × 100% = **95%** (well above 90% gate-1 auto-approval floor).

**Direct-approval criteria** (per chunk-initiate Phase 1.5 — see also checklist at bottom of plan):
| Criterion | Status | Evidence |
|---|---|---|
| No locked forks at plan-time (or all locks are planner-rec) | **PASS** | 4 forks all resolve to planner-rec (F1 RESOLVED at prerequisite commit `af037cf`; F2 + F3 + F4 planner-rec rubber-stamps expected at gate-1.5); no DIVERGENT locks |
| Scope ≤ 1.5× forward-scope row deliverables | **PASS** | 7 deliverables (D1–D7) match forward-scope row §1 line 89 enumeration exactly; no scope expansion |
| Zero phi-core leverage delta | **PASS** | §3 predicts Δ +0 raw `use phi_core` lines; baseline 57 preserved; +1 semantic leverage-site via FQN (does not count against raw-line baseline) |
| No new K8s blocker class | **PASS** | §3.B 7-axis table all rows "no impact"; K8s-neutral verdict |
| Audit envelope ≤ medium | **PASS** | SMALL (1 auditor letter A) per §11 sizing override rationale |
| Confidence ≥ 9/10 | **PASS** | implementation 9.5/10 + documentation 4/4; composite 95% |
| No new migration | **PASS** | §3.F SCHEMAFULL checklist N/A; zero SurrealDB schema changes |

All 7 Direct-approval criteria hold. Orchestrator expected to auto-approve via ExitPlanMode at gate-1.

---

## §11 — Post-chunk independent audit plan

**Phase count**: **4 (P1 + P2 + P3 + P-SEAL)** — per audit-envelope-size skill sizing rule, this lands in the **3–5 phases / Medium envelope (2 auditors A + B)** band. However, the chunk's surface area is **mid-way between CH-28b and CH-28c** (2 carrier-fixes + paperwork; CH-28b had 1 carrier-fix + 1 match-arm + paperwork at 5 phases, CH-28c had 0 carrier-fixes + 1 cosmetic match-arm + paperwork at 4 phases). Per audit-envelope-size skill's underlying intent (phase count is a proxy for audit complexity, not the load-bearing signal), the planner sizes this chunk as **SMALL (1 auditor letter A)** with the following justification:

**Sizing override rationale**: phase count 4 lands at the lower end of the Medium boundary. 3 of the 4 phases (P1 Cargo bump / P2 2-field carrier-fix / P-SEAL paperwork) are ≤ 15 LOC each + carry mechanical-replication patterns (direct CH-28b + CH-28c precedent at the same struct-literal site). Only P3 (drift + feature-inventory + ADR draft) has docs-fidelity surface to audit; the entire chunk's audit surface fits comfortably in one auditor's scope. The planner self-confirms SMALL via parallel reasoning to CH-28b's SMALL sizing override (closed cleanly with 12 PASS + 2 NOT-EXECUTED-IN-AUDIT + 1 PASS-with-caveat handled via orchestrator Trivial-1L at gate-3) + CH-28c's SMALL sizing override (closed cleanly at iter-1 PASS). 3rd consecutive activation of the SMALL sizing for phi-core absorption chunks.

**Audit envelope**: **SMALL (1 auditor letter A)** — combined code + phi-core + K8s + concept + docs + ADR coverage.

**Audit aspects (a–d)**:
- (a) Code correctness (D1 Cargo bump + D2 lock regen + D3 2-field carrier-fix).
- (b) Docs fidelity vs concept docs (NEW drift body matches phi-core CHANGELOG §[0.10.0]; ADR-0066 sub-decisions match §5; feature-inventory row matches drift body).
- (c) Concept alignment across §2 rows.
- (d) phi-core leverage (Δ +0 raw `use phi_core` lines; +1 semantic leverage-site via FQN; baseline 57 preserved; no forbidden duplications).

### Audit A scaffold (≤ 600 words; combined code + concept + docs + phi-core + K8s)

```
You are auditing CH-28d (phi-core 0.10.0 absorption / RevertRenderPolicy +
CurrentToolExecution AgentLoopConfig field-additions + final async-Fn migration)
in baby-phi at /root/projects/phi/baby-phi/. Read-only on source. Plan at
docs/specs/plan/build/phi-core-10-absorption-<8hex>/plan.md.

Verify each claim with file:line citation:

1. Cargo.toml workspace-deps row at /root/projects/phi/baby-phi/Cargo.toml line 17
   reads `phi-core = "0.10"` (NOT "0.9", NOT "0.10.0", NOT "^0.10.0"). Run:
   `grep -nE '^phi-core = "0.10"$' /root/projects/phi/baby-phi/Cargo.toml`.
   PASS if exit 0 + 1 hit.

2. Cargo.lock regenerated against phi-core 0.10.x. Run:
   `grep -nA 1 '^name = "phi-core"' /root/projects/phi/baby-phi/Cargo.lock |
    grep '^version'`. Expect `version = "0.10.0"` (or higher 0.10.x).

3. 2 new fields present at AgentLoopConfig struct-literal at
   `/root/projects/phi/baby-phi/modules/crates/server/src/platform/sessions/launch.rs`
   between line 576 (revert_pending block) and the closing `};`. Run:
   `grep -n 'revert_render_policy: phi_core::types::node_tag::RevertRenderPolicy::default'
    modules/crates/server/src/platform/sessions/launch.rs` and
   `grep -n 'current_tool: None'
    modules/crates/server/src/platform/sessions/launch.rs`.
   PASS if 1 hit each.

4. NEW drift filed at canonical m6/drifts/ path. Run:
   `ls /root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/drifts/D-PHICORE-10-FOLLOWUP-01-new-api-adoption-cluster.md`.
   PASS if file exists with line 1 verified-header.

5. ADR-0066 filed at canonical m6/decisions/ path with Status: Accepted at P-SEAL.
   Run:
   `head -10 /root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/decisions/0066-phi-core-10-absorption.md`.
   PASS if Status line reads `Status: Accepted` + sub-decisions §D66.1–§D66.5
   referenced + ADR-0034 §D34.6 cross-referenced.

6. feature-inventory.md §3 row added for D-PHICORE-10-FOLLOWUP-01. Run:
   `grep -nE '### D-PHICORE-10-FOLLOWUP-01' /root/projects/phi/baby-phi/docs/specs/v0/feature-inventory.md`.
   PASS if ≥ 1 hit + row body has 5 fields (Feature impact / User-visible v0 /
   User-visible final / Allocation chunk / Cross-chunk dep).

7. cycle-index row appended for `<8hex>`. Run:
   `grep -n '<8hex>' /root/projects/phi/baby-phi/docs/specs/plan/build/_cycle-index.md`.
   PASS if ≥ 1 hit at active-cycles tail; row shape per chunk-implementer P-SEAL v17.

8. cargo test --workspace --no-fail-fast green at expected count 1599. Run:
   `cargo test --workspace --no-fail-fast --manifest-path /root/projects/phi/baby-phi/Cargo.toml -j 4`.
   Mark NOT-EXECUTED-IN-AUDIT if sandbox-blocked; orchestrator closes at gate-4.

9. CI guards green; check-phi-core-reuse.sh exit 0; no new `use phi_core::`
   imports beyond §3 prediction (baseline 57 preserved). Run:
   `bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh` (mark
   NOT-EXECUTED-IN-AUDIT if sandbox-blocked). Run:
   `grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l`;
   expect 57.

10. CH-25 P-SEAL invariant intact: `response_format: ResponseFormat::default()`
    still present at `launch.rs`. Run:
    `grep -n 'response_format: phi_core::provider::traits::ResponseFormat::default' modules/crates/server/src/platform/sessions/launch.rs`.
    PASS if ≥ 1 hit.

11. CH-28b carrier-fix invariant intact: `revert_pending: None` still present at
    `launch.rs`. Run:
    `grep -n 'revert_pending: None' modules/crates/server/src/platform/sessions/launch.rs`.
    PASS if ≥ 1 hit.

12. CH-28b + CH-28c explicit-arm invariants intact: `AgentEvent::RevertApplied`
    + `AgentEvent::TurnRequest` still present at `cli/agent.rs`. Run:
    `grep -nE 'AgentEvent::RevertApplied|AgentEvent::TurnRequest' modules/crates/cli/src/commands/agent.rs`.
    PASS if ≥ 2 hits.

13. CH-28 invariant intact: 7 m6 agent profile cardinality acceptance tests pass.
    Run:
    `cargo test --workspace --test acceptance_m6_agent_profile_cardinality
     --manifest-path /root/projects/phi/baby-phi/Cargo.toml -j 4`.
    Mark NOT-EXECUTED-IN-AUDIT if sandbox-blocked.

14. K8s 7-axis: ZERO new blocker class introduced. Verify §3.B classifications
    match diff (no DashMap/RwLock/AtomicBool/Mutex/OnceCell added; no
    mpsc/broadcast/oneshot/watch/Notify added; no file handle / sub-process /
    on-disk cache added; no SurrealDB migration added; no new trait surface;
    no cross-pod state shared; no new audit-event writer). Run:
    `git diff HEAD~ --stat`. PASS if no source files outside
    {Cargo.toml, Cargo.lock, modules/crates/server/src/platform/sessions/launch.rs,
     docs/**/*.md} are touched.

15. Forbidden-duplication greps return 0 hits. Run all 8 forbidden greps per
    plan §3 (RevertRenderPolicy / CurrentToolExecution / 2 type aliases /
    detect_interpreter / AgentLoopConfig / AgentEvent / BlockCompactionStrategy).
    PASS if all return 0.

PASS/FAIL each. ≤ 600 words total.
```

**Audit pass criteria**:
- Any new drift discovered by the audit → its own drift file created BEFORE chunk seals.
- Any audit-flagged concept contradiction → either fixed in-chunk, renegotiated with user approval, or converted to a drift file with explicit future-chunk assignment.
- Chunk seal is blocked until audit returns clean on all 15 claims (with `NOT-EXECUTED-IN-AUDIT` claims closed at orchestrator gate-4 MUST-RUN).

---

## §12 — Verification section (end-to-end recipe)

Concrete commands a reviewer can run to replay the chunk's close verification:

```bash
# 1. CI guards
bash /root/projects/phi/baby-phi/scripts/check-doc-links.sh
bash /root/projects/phi/baby-phi/scripts/check-ops-doc-headers.sh
bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh
bash /root/projects/phi/baby-phi/scripts/check-spec-drift.sh

# 2. Workspace health (cargo with -j 4 per feedback_cargo_jobs_cap)
/root/rust-env/cargo/bin/cargo fmt --all --manifest-path /root/projects/phi/baby-phi/Cargo.toml -- --check
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml --workspace --all-targets
/root/rust-env/cargo/bin/cargo test --workspace --no-fail-fast -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml
# expect: 1599 passed / 0 failed

# 3. Cargo cleanup per chunk-implementer v8 placement-1
/root/rust-env/cargo/bin/cargo clean --manifest-path /root/projects/phi/baby-phi/Cargo.toml

# 4. Chunk-specific greps (per plan §3 positive close-audit greps)
# 4a. Cargo.toml workspace-deps row
grep -nE '^phi-core = "0.10"$' /root/projects/phi/baby-phi/Cargo.toml
# expect: 1 hit at line 17

# 4b. Cargo.lock pin
grep -nA 1 '^name = "phi-core"' /root/projects/phi/baby-phi/Cargo.lock | grep '^version'
# expect: version = "0.10.0" (or higher 0.10.x)

# 4c. 2 new fields at launch.rs
grep -n 'revert_render_policy: phi_core::types::node_tag::RevertRenderPolicy::default' /root/projects/phi/baby-phi/modules/crates/server/src/platform/sessions/launch.rs
grep -n 'current_tool: None' /root/projects/phi/baby-phi/modules/crates/server/src/platform/sessions/launch.rs
# expect: 1 hit each

# 4d. phi-core import baseline preserved
grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l
# expect: 57

# 5. Forbidden-duplication greps (per plan §3)
grep -rn "^pub struct RevertRenderPolicy" /root/projects/phi/baby-phi/modules/crates/
# expect: 0 hits
grep -rn "^pub struct CurrentToolExecution" /root/projects/phi/baby-phi/modules/crates/
# expect: 0 hits
grep -rn "^pub type BeforeToolExecutionUpdateFn" /root/projects/phi/baby-phi/modules/crates/
# expect: 0 hits
grep -rn "^pub type AfterToolExecutionUpdateFn" /root/projects/phi/baby-phi/modules/crates/
# expect: 0 hits
grep -rn "^pub fn detect_interpreter" /root/projects/phi/baby-phi/modules/crates/
# expect: 0 hits
grep -rn "^pub struct AgentLoopConfig" /root/projects/phi/baby-phi/modules/crates/
# expect: 0 hits
grep -rn "^pub enum AgentEvent" /root/projects/phi/baby-phi/modules/crates/
# expect: 0 hits
grep -rn "impl phi_core::context::BlockCompactionStrategy for" /root/projects/phi/baby-phi/modules/crates/
# expect: 0 hits

# 6. Drift file presence
ls /root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/drifts/D-PHICORE-10-FOLLOWUP-01-new-api-adoption-cluster.md
# expect: file exists

# 7. ADR file presence + Accepted status
head -10 /root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/decisions/0066-phi-core-10-absorption.md
# expect: Status: Accepted, sub-decisions §D66.1–§D66.5 referenced

# 8. feature-inventory.md §3 row presence
grep -nE '### D-PHICORE-10-FOLLOWUP-01' /root/projects/phi/baby-phi/docs/specs/v0/feature-inventory.md
# expect: ≥ 1 hit

# 9. Cycle-index row presence
grep -n '<8hex>' /root/projects/phi/baby-phi/docs/specs/plan/build/_cycle-index.md
# expect: ≥ 1 hit at active-cycles tail (substitute the orchestrator-minted hex)

# 10. CH-25 P-SEAL invariant intact
grep -n 'response_format: phi_core::provider::traits::ResponseFormat::default' /root/projects/phi/baby-phi/modules/crates/server/src/platform/sessions/launch.rs
# expect: 1 hit

# 11. CH-28b carrier-fix invariant intact
grep -n 'revert_pending: None' /root/projects/phi/baby-phi/modules/crates/server/src/platform/sessions/launch.rs
# expect: ≥ 1 hit

# 12. CH-28b + CH-28c explicit-arm invariants intact
grep -nE 'AgentEvent::RevertApplied|AgentEvent::TurnRequest' /root/projects/phi/baby-phi/modules/crates/cli/src/commands/agent.rs
# expect: ≥ 2 hits

# 13. CH-28 invariant intact
/root/rust-env/cargo/bin/cargo test --workspace --test acceptance_m6_agent_profile_cardinality --manifest-path /root/projects/phi/baby-phi/Cargo.toml -j 4
# expect: 7 tests passed

# 14. Drift-file status (NEW drift added at chunk-seal)
grep -l "Status.*discovered" /root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/drifts/D-PHICORE-10-FOLLOWUP-01*.md | wc -l
# expect: 1 (the NEW drift is at Status: discovered, NOT remediated)
```

---

## End of plan