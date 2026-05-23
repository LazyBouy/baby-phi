<!-- Last verified: 2026-05-23 by Claude Code (CH-28b iter-1 plan draft; phi-core 0.8.0 absorption / Composition I baseline shift; TECHNICAL-PREREQUISITE chunk; cycle hex `d5b776ac`; chunk-planner v28 + per-chunk-planning-template Pre-§1 Forks-section format + v23 P-plan-3 ALWAYS-FIRE Locked fork details + v22 P13 + v25 §3.F SCHEMAFULL checklist N/A; 3 forks all RESOLVED at plan-mode session 2026-05-23 OR planner-rec rubber-stamp at gate-1; Direct-approval criteria all 7 hold per §12; audit envelope SMALL (1 auditor letter A) per audit-envelope-size skill; phi-core import baseline 57 / leverage-sites Δ +0 / forbidden-duplication greps green; K8s 7-axis zero delta; ADR-0064 drafted with 6 sub-decisions §D64.1–§D64.6; NEW drift D-PHICORE-08-FOLLOWUP-01 filed at m6/drifts/; cycle hex `d5b776ac`.) -->

# CH-28b — phi-core 0.8.0 absorption (Composition I braking layer; baseline shift)

**Chunk slug**: `ch-28b-phi-core-08-absorption`
**Cycle hex**: `d5b776ac`
**Chunk-type**: TECHNICAL-PREREQUISITE
**Audit envelope**: SMALL (1 auditor letter A)
**Forward-scope row**: [`forward-scope/m6-forward-scope-8b7a8bcd.md`](../../forward-scope/m6-forward-scope-8b7a8bcd.md) lines 56–67 (§1 narrative) + line 319 (§5 table row)

---

## §1 — Locked fork details (per chunk-planner v23 P-plan-3 ALWAYS-FIRE + v28 §1-position codification + per-chunk-planning-template Pre-§1 Forks-section format)

All three forks for this chunk are TECHNICAL FORK or USER-RESOLVED AT PRE-CHUNK-PLAN session (no quality concerns). The Locked fork details section is emitted here at §1 front-of-plan per chunk-planner v28 P-plan-3-v28 + the v23 P-plan-3 ALWAYS-FIRE rule so the implementer + auditors + reader can immediately see what each lock means without scrolling.

#### F1 = F1.b — CH-28b interstitial (TECHNICAL FORK, RESOLVED at prerequisite commit `563dcda` 2026-05-23)

**Code-level binding**: the chunk slug is `ch-28b` (not `ch-29` or `ch-28.5`); the forward-scope row is inserted as the second row of the Foundation tier between CH-28 and CH-29, NOT renumbering CH-29..CH-38; the cycle folder is `ch-28b-phi-core-08-absorption-d5b776ac`. First-of-kind "CH-NNa/b" suffix in the baby-phi cycle-index naming — minimum-edit footprint (~5 doc surfaces touched at the prerequisite commit vs ~30 surfaces for F1.a renumber alternative).

**Rationale (user-locked at plan-mode session 2026-05-23)**: F1.a (renumber) would cascade across forward-scope + concept-doc cross-refs + feature-inventory.md + 2-3 ADRs; F1.c (CH-28.5 dot suffix) is parallel to M5.3 carve-out pattern but visually breaks the integer cycle-index sortable convention. F1.b is the surgical choice: tags the cycle as an interstitial without disturbing the integer chunk numbering of CH-29..CH-38.

**Defers**: no defer — chunk numbering convention shift is concretely applied at the prerequisite commit; future cycles may follow the precedent.

#### F2 = F2.a — `phi-core = "0.8"` (semver-range; TECHNICAL FORK; planner-rec rubber-stamp expected at gate-1)

**Code-level binding**: workspace `Cargo.toml` workspace-dependency row reads `phi-core = "0.8"` (no patch literal); `cargo update -p phi-core` selects the highest 0.8.x patch currently published on crates.io (today: 0.8.0). Future phi-core 0.8.x patches flow through to baby-phi automatically on the next `cargo update` invocation, without requiring a Cargo.toml edit per patch.

**Rationale**: F2.b (exact pin `phi-core = "0.8.0"`) would require a Cargo.toml commit on every phi-core patch; F2.c (`^0.8.0` semver-explicit) is equivalent to F2.a in semver-resolution semantics but adds visual noise. F2.a is the canonical Cargo idiomatic form for "I trust semver-compatible updates."

**Defers**: nothing — Cargo.lock pinning provides reproducibility within a given commit; semver-range gives flexibility on the next bump.

#### F3 = F3.a — file new drift at baby-phi's canonical drift directory (BOUNDED USER-VISIBLE; planner-rec rubber-stamp expected at gate-1)

**Code-level binding**: NEW drift file at `docs/specs/v0/implementation/m6/drifts/D-PHICORE-08-FOLLOWUP-01-composition-i-adoption.md` (parallel to existing `m6/drifts/D-CH28-FOLLOWUP-01-blueprint-upserted-template-fanout.md`); feature-inventory.md §3 Deferred catalogue gains a `D-PHICORE-08-FOLLOWUP-01` row.

**Rationale**: baby-phi's canonical drift directory convention is `docs/specs/v0/implementation/m<N>/drifts/` where `m<N>` matches the milestone in which the chunk that FILED the drift sits (CH-28b ⊂ M6, so `m6/drifts/`). Verified at plan-draft time: existing m6 drift `D-CH28-FOLLOWUP-01-blueprint-upserted-template-fanout.md` (filed by CH-28) is the precedent. F3.b (embed deferral note in feature-inventory.md only, no drift file) would short-circuit the drift lifecycle + lose the rich blocked-by / closing-chunk metadata that the chunk-planner v13 non-terminal-drift discipline depends on; F3.a is the structural choice.

**User-visible perspective**: the user observes nothing at CH-28b close (Composition I is opt-in and NOT enabled); the drift file becomes the canonical lookup for "when will my agents stop accumulating dead conversation branches between turns?". After the future Composition I adoption chunk (no CH-NN slot reserved yet), the drift's Status flips `discovered → remediated` and the feature-inventory row's User-visible delta column reflects the new behavior.

**Product trajectory**: agents stay on the pre-0.8.0 monotonically-growing-context posture in v0 (compaction still works, but is the ONLY relief). Post-adoption: agents call `revert_to_state` between turns to drop dead branches; context stays lean; compaction fires less often.

**Defers**: Composition I adoption (`.with_revert_tool()` call site + RevertRenderPolicy tuning + revert-discipline skill teaching the agent the four categories failure/tangent/completion/step-summary) → future M6+ FUNCTIONAL chunk via this drift.

---

## Forks for orchestrator

> All three forks below are RESOLVED at the plan-mode session 2026-05-23 OR planner-rec rubber-stamp expected at gate-1 (TECHNICAL FORK). No quality concerns; user can ratify all three locks at gate-1 ExitPlanMode. See §1 above for plain-English semantics of each lock.

### F1 — Chunk numbering and m6-forward-scope insertion shape — **TECHNICAL FORK** (no user-visible delta — pick on engineering merit only)

| Option | Pros | Cons | Status |
|---|---|---|---|
| F1.a — CH-29 (renumber CH-29..CH-38 → CH-30..CH-39) | Preserves integer-only chunk numbering convention | ~30 edits across forward-scope + concept-doc cross-refs + feature-inventory.md + 2-3 ADRs; touches more surfaces than the chunk actually delivers | NOT chosen |
| F1.b (planner-rec) — **CH-28b interstitial** | ~5 edits at the prerequisite commit; surgical scope; first-of-kind suffix establishes precedent | First-of-kind "CH-NNa/b" suffix in cycle-index — slight novelty cost on naming convention | **LOCKED at gate-1.5 (RESOLVED at prerequisite commit `563dcda` 2026-05-23 per user plan-mode lock)** |
| F1.c — CH-28.5 carve-out (dot suffix, parallel to M5.3 carve-out pattern) | Parallel to M5.3 → CH-27 carve-out precedent | Visually breaks integer cycle-index sortable convention | NOT chosen |

### F2 — Dep-string-form for `phi-core` — **TECHNICAL FORK** (no user-visible delta — pick on engineering merit only)

| Option | Pros | Cons | Status |
|---|---|---|---|
| F2.a (planner-rec) — **`phi-core = "0.8"` (semver-range)** | Picks any 0.8.x patch automatically on next `cargo update`; idiomatic Cargo form for "trust semver-compatible updates" | Requires team trust in phi-core's semver discipline | LOCKED at gate-1.5 (planner-rec rubber-stamp expected) |
| F2.b — `phi-core = "0.8.0"` (exact-pin) | Pins to exact patch; immune to upstream semver violations | Requires a Cargo.toml commit on every phi-core patch | NOT chosen |
| F2.c — `phi-core = "^0.8.0"` (semver-explicit) | Equivalent to F2.a in semver resolution | Adds visual noise without benefit | NOT chosen |

### F3 — Drift directory routing for `D-PHICORE-08-FOLLOWUP-01` — BOUNDED USER-VISIBLE

| Option | User-visible (what the user perceives) | Pros | Cons + Product trajectory | Status |
|---|---|---|---|---|
| F3.a (planner-rec) — **file new drift at baby-phi's canonical drift directory** (`docs/specs/v0/implementation/m6/drifts/D-PHICORE-08-FOLLOWUP-01-composition-i-adoption.md`) | Composition I adoption appears in the drift backlog with explicit "filed at CH-28b, closes at M6+ FUNCTIONAL chunk TBD" trace — the user can grep drifts for "Composition I" + find one canonical entry | Canonical drift-lifecycle discipline; rich blocked-by / closing-chunk / discovery-source metadata; feature-inventory.md §3 row gives the user-visible v0-vs-final delta translation | One small new file added; chunk-planner v13 non-terminal-drift rule requires explicit `M*-DEFERRED-NN` allocation OR explicit named successor chunk. **Product trajectory:** adoption tracked + visible; sequencing of when-to-adopt becomes a deliberate user/planner decision rather than a forgotten TODO. | LOCKED at gate-1.5 (planner-rec rubber-stamp expected) |
| F3.b — embed deferral note in feature-inventory.md only, no drift file | Composition I adoption appears as a §3 row in feature-inventory only; the user sees it as a v0-vs-final capability delta but cannot trace it back to a code-tier drift | Lighter footprint (no new drift file) | Short-circuits drift lifecycle; loses blocked-by metadata; feature-inventory rows are NOT designed as primary drift trackers. **Product trajectory:** adoption decision risks being lost in feature-inventory updates over time; less discoverable for future implementer dispatch. | NOT chosen |

---

### §1.1 — Why this chunk

phi-core 0.8.0 shipped on 2026-05-23 with a breaking-change release (Composition I — opt-in tree-structured "braking" layer with `revert_to_state` tool). baby-phi pinned `phi-core = "0.7.1"` at the M5 close (last patch shipped 2026-05-16, doc-only). The cumulative phi-core delta since the last baby-phi bump includes the 0.8.0 breaking changes:

- `LlmMessage` gains 3 new public fields (`node_id`, `parent_id`, `tags`) — direct struct-literal construction breaks; constructor `LlmMessage::new()` shields all baby-phi call sites.
- `AgentEvent` becomes `#[non_exhaustive]` + gains `RevertApplied { .. }` variant — exhaustive matches in downstream crates require either an explicit `RevertApplied` arm or a wildcard `_ =>`.
- `AgentLoopConfig` gains `revert_pending: Option<Arc<Mutex<Vec<RevertRequest>>>>` — direct struct-literal construction breaks; constructor `BasicAgent::build_config()` shields most call sites; baby-phi has exactly ONE direct struct-literal site (`server/src/platform/sessions/launch.rs:526-568`) that requires the field addition.

Pre-spawn verification confirmed baby-phi's absorption surface is even smaller than the i-phi diagnostic predicted (1 compile break at the AgentLoopConfig struct-literal; zero compile breaks at the other two surfaces). This chunk shifts the dependency baseline to `phi-core = "0.8"` so the rest of M6 runs on the up-to-date phi-core; it does NOT enable Composition I (deferred via NEW drift `D-PHICORE-08-FOLLOWUP-01`).

The drift this chunk closes is the urgent need to absorb the breaking-change release BEFORE the next FUNCTIONAL M6 chunk (CH-33 first FUNCTIONAL chunk per forward-scope) opens. The drift this chunk files (D-PHICORE-08-FOLLOWUP-01) is the deferred adoption of Composition I, allocated to a future M6+ FUNCTIONAL chunk.

### §1.2 — Quality-over-speed restatement

*"Concept docs are source-of-truth; implementation aligns to them. Drift is discovered, documented, and planned-through — never accumulated silently."* — applied here as: the phi-core 0.8.0 breaking-change surface is absorbed in a dedicated TECHNICAL-PREREQUISITE chunk with explicit deliverables + acceptance verification rather than being bundled into a FUNCTIONAL chunk where it would be lost in noise + create cross-cutting deviation risk.

### §1.3 — Forward-scope reference

[`forward-scope/m6-forward-scope-8b7a8bcd.md`](../../forward-scope/m6-forward-scope-8b7a8bcd.md) §1 Foundation tier lines 56–67 + §5 table row line 319. Row text reads: *"CH-28b — phi-core 0.8.0 absorption (Composition I braking layer; baseline shift) · MEDIUM · 0.5–1d · Chunk-type: TECHNICAL-PREREQUISITE · Functional outcome: NONE this chunk · Defers: Composition I adoption to future M6+ FUNCTIONAL chunk via D-PHICORE-08-FOLLOWUP-01."*

---

## §2 — Concept alignment walk

| Concept doc | § anchor | Claim (verbatim or close paraphrase) | Status at chunk-open | Target status at chunk-close |
|---|---|---|---|---|
| [`phi-core/CHANGELOG.md`](../../../../../phi-core/CHANGELOG.md) | `§[0.8.0]` lines 9–94 | "Breaking-change release. Ships Composition I — an opt-in tree-structured braking layer ... Enable via `BasicAgent::with_revert_tool()` (one line on the builder)." | silent-in-code (baby-phi pins 0.7.1) | honored (baseline shifts to 0.8.x; opt-in remains NOT enabled per F3.a defer) |
| [`phi-core/CHANGELOG.md`](../../../../../phi-core/CHANGELOG.md) | `§[0.8.0] Breaking` lines 30–50 | "`LlmMessage` gains three new public fields ... Construction via `LlmMessage::new(...)` is unaffected." | silent-in-code | honored (baby-phi has zero `LlmMessage { .. }` struct-literals — both call sites at `domain/src/session_recorder.rs:483` + `server/src/platform/sessions/launch.rs:569` use `LlmMessage::new()`; no carrier-fix needed) |
| [`phi-core/CHANGELOG.md`](../../../../../phi-core/CHANGELOG.md) | `§[0.8.0] Breaking` lines 41–46 | "`AgentEvent` becomes `#[non_exhaustive]` and gains the `RevertApplied { ... }` variant. Every exhaustive `match` against `AgentEvent` in a downstream crate now requires either an explicit `RevertApplied { ... }` arm or a wildcard `_ => ...`." | silent-in-code (baby-phi's 2 match sites already use field-wildcard `..` inside `matches!()` AND `_ => {}` catch-all) | honored (existing wildcards cover the variant; D4 adds explicit `RevertApplied { .. } => {}` arm at the cli/agent.rs exhaustive-match site for future-reader discoverability; see §3.E open question for session_recorder.rs `matches!()` site) |
| [`phi-core/CHANGELOG.md`](../../../../../phi-core/CHANGELOG.md) | `§[0.8.0] Breaking` lines 47–50 | "`AgentLoopConfig` gains `revert_pending: Option<Arc<Mutex<Vec<RevertRequest>>>>`. Struct-literal construction breaks — callers must add `revert_pending: None`." | contradicted (baby-phi has exactly 1 direct struct-literal at `server/src/platform/sessions/launch.rs:526-568` missing this field) | honored (D3 adds `revert_pending: None` literal field after the existing `response_format: ResponseFormat::default()` field at the cited site) |
| [`phi-core/docs/concepts/concept-brake.md`](../../../../../phi-core/docs/concepts/concept-brake.md) | `§5 Composition I` | "Tree-structured composition: the agent calls `revert_to_state` between turns to abandon failed/finished branches; next prompt rebuilds by walking parent-id links." | concept-aspirational (not enabled in baby-phi) | concept-aspirational (deferred to future M6+ FUNCTIONAL chunk via D-PHICORE-08-FOLLOWUP-01; CH-28b documents the deferral, does not enable) |
| [`i-phi diagnostic`](../../../../../i-phi/docs/v0/proposal/plan/build/phi-core-revert-tool-27c894f6/plan.md) | `§DIAGNOSTIC SECTION A` | "baby-phi absorption impact: LOW — predicted 1 carrier-fix at AgentLoopConfig struct-literal + 0-2 explicit match-arm additions" | honored (empirical absorption surface ≤ predicted) | honored (CH-28b implements the LOW-impact absorption per diagnostic; no surprise carrier-fixes) |
| [`baby-phi CLAUDE.md`](../../../../../CLAUDE.md) | `§phi-core Leverage` rules 1–5 | "phi is a consumer of phi-core, not a parallel implementation. Every surface that overlaps with an existing `phi_core::` type MUST reuse it directly or wrap it — never re-implement." | honored | honored (baseline shift preserves the leverage contract; ZERO new types created; existing imports unchanged) |
| [`docs/specs/v0/concepts/phi-core-mapping.md`](../../v0/concepts/phi-core-mapping.md) | full doc | "phi-core type ↔ baby-phi consumer mapping table" | honored | honored (mapping rows unchanged; new phi-core types `NodeId`/`NodeTag`/`RevertCategory`/`RevertRequest`/`RevertTool`/`RevertApplied` event become available but NOT imported until Composition I adoption fires) |

**Permissions subtree hook**: not applicable — chunk does not touch `permissions/01`–`permissions/09` docs.

**phi-core-mapping hook**: cited above. The mapping rows for `AgentEvent`, `LlmMessage`, `AgentLoopConfig` are touched (in the "consumed via..." column the variant set / field set changes upstream); since the row metadata describes type-presence rather than field cardinality, no mapping-row edits are required.

---

## §2.5 — Functional outcome

**Chunk-type**: TECHNICAL-PREREQUISITE

**User-visible delivery**: NONE this chunk.

**Unblocks**: ALL downstream M6 chunks (CH-29..CH-38) by virtue of the workspace compiling on the 0.8.x baseline. In particular, CH-33 (the first FUNCTIONAL M6 chunk, which consumes `phi_core::types::tool::AgentTool` for `read_inbox` + `send_message` tools — the AgentTool surface is unchanged across 0.7.1 → 0.8.0 per phi-core CHANGELOG). Additionally unblocks the future Composition I adoption chunk (no CH-NN slot reserved yet; tracked via `D-PHICORE-08-FOLLOWUP-01`).

**Why this prerequisite**: phi-core 0.8.0 shipped today (2026-05-23) with a 1-line carrier-fix surface for baby-phi (`AgentLoopConfig.revert_pending: None`); absorbing it now in a dedicated chunk means the next FUNCTIONAL M6 chunk opens against a fresh phi-core baseline rather than discovering the breaking-change mid-flight + requiring an unplanned compile-restoration deviation. This is the same pattern as CH-25 P-SEAL workspace-health fix at `launch.rs:558-567` (`response_format: ResponseFormat::default()`) — a 1-line carrier-fix absorbed via dedicated chunk rather than bundled into FUNCTIONAL scope.

---

## §3 — phi-core leverage map

| phi-core type | Current handling in baby-phi | Classification | Action in chunk |
|---|---|---|---|
| `phi_core::types::event::AgentEvent` | imported at `domain/src/session_recorder.rs:11` (used at session_recorder.rs:166-170 inside `matches!()` with `..` field-wildcard) + `cli/src/commands/agent.rs:11` (used at cli/agent.rs:642-669 inside `match &event { ... _ => {} }`) | direct-reuse (becomes `#[non_exhaustive]` upstream at 0.8.0; existing wildcards cover the new variant compile-wise) | D4: ADD explicit `RevertApplied { .. } => {}` arm at the cli/agent.rs site for future-reader discoverability (existing `_ => {}` already covers compile-time exhaustiveness); session_recorder.rs `matches!()` site does NOT need an additional arm (single-pattern match → see §3.E open question) |
| `phi_core::types::message::LlmMessage` | imported at `domain/src/session_recorder.rs:483` + `server/src/platform/sessions/launch.rs:569` (both call sites use `LlmMessage::new(Message::user(...))` constructor) | direct-reuse (constructor shields against the 3 new public fields) | no action — constructors absorb the upstream field additions; no struct-literal sites |
| `phi_core::AgentLoopConfig` | imported at `server/src/platform/sessions/launch.rs` (used at launch.rs:526-568 as a direct struct-literal — this is the SAME site that received CH-25's `response_format: None` repair at lines 558-567) | wrap (carrier-fix via 1-line field addition; preserves all other field semantics) | D3: ADD `revert_pending: None,` literal after the existing `response_format: phi_core::provider::traits::ResponseFormat::default()` field; preserves opt-out posture (`None` = Composition I NOT enabled at session launch) |
| `phi_core::provider::traits::ResponseFormat` | imported at `server/src/platform/sessions/launch.rs:567` (used in the same struct-literal as `ResponseFormat::default()` per CH-25 P-SEAL fix) | direct-reuse (unchanged at 0.8.0) | no action |
| `phi_core::tools::revert::RevertTool` (NEW at 0.8.0) | NOT imported | direct-reuse (becomes available; remains NOT imported until Composition I adoption chunk) | no action this chunk; deferred via D-PHICORE-08-FOLLOWUP-01 |
| `phi_core::tools::revert::RevertRequest` (NEW at 0.8.0) | NOT imported | direct-reuse (becomes available) | no action this chunk; deferred via D-PHICORE-08-FOLLOWUP-01 |
| `phi_core::types::node_tag::{NodeId, NodeTag, TagKind, RevertCategory, RevertRenderPolicy}` (NEW at 0.8.0) | NOT imported | direct-reuse (becomes available) | no action this chunk; deferred via D-PHICORE-08-FOLLOWUP-01 |
| `phi_core::types::event::AgentEvent::RevertApplied` (NEW variant at 0.8.0) | reachable via existing wildcards at 2 match sites | direct-reuse (variant covered; explicit arm added at cli/agent.rs per D4) | covered |

**Expected leverage-site delta at chunk close**: **+0 leverage-sites** (per chunk-planner v9 leverage-site methodology). All 4 currently-active phi-core leverage-sites (session_recorder.rs match site, cli/agent.rs match site, session_recorder.rs LlmMessage constructor, launch.rs AgentLoopConfig + LlmMessage constructor) remain semantically the same — the chunk modifies field-set + adds an explicit match-arm but does not introduce a new semantic use of phi-core. The chunk does NOT introduce any new `use phi_core::...` imports (the new `RevertApplied` variant is reached via the already-imported `AgentEvent` enum).

**Expected import-count delta at chunk close**: **+0 raw `use phi_core` lines** (baseline 57 lines per `grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l` at plan-draft time; expected to stay 57 at chunk-close).

**Positive close-audit greps** (the exact commands the post-chunk audit will run):

1. `grep -c "revert_pending: None" /root/projects/phi/baby-phi/modules/crates/server/src/platform/sessions/launch.rs` — expect **1** (the D3 carrier-fix at the launch.rs struct-literal).
2. `grep -nE "AgentEvent::RevertApplied" /root/projects/phi/baby-phi/modules/crates/cli/src/commands/agent.rs` — expect **≥ 1** (the D4 explicit arm).
3. `grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l` — expect **57** (unchanged from baseline; no new imports).
4. `grep -nE '^phi-core = "0.8"' /root/projects/phi/baby-phi/Cargo.toml` — expect **1** (the workspace-deps row update).

**Forbidden-duplication greps** (exact commands that must return 0 hits):

1. `grep -rn "^pub struct AgentLoopConfig" /root/projects/phi/baby-phi/modules/crates/` — expect **0** (no parallel definition).
2. `grep -rn "^pub enum AgentEvent" /root/projects/phi/baby-phi/modules/crates/` — expect **0** (no parallel definition).
3. `grep -rn "^pub struct LlmMessage" /root/projects/phi/baby-phi/modules/crates/` — expect **0** (no parallel definition).
4. `grep -rn "^pub fn revert_to_state" /root/projects/phi/baby-phi/modules/crates/` — expect **0** (no parallel revert-tool implementation; tool is opt-in via `BasicAgent::with_revert_tool()` and remains NOT enabled).

Per [`baby-phi/CLAUDE.md`](../../../../../CLAUDE.md) §"phi-core Leverage" rules 1–5. `scripts/check-phi-core-reuse.sh` MUST stay green at chunk close.

### §3 cascade-artifact discipline (per chunk-planner v4 + v17 per-fork pause-threshold table)

Cascade is structurally minimal for this chunk — the breaking-change surface has been verified pre-spawn against the actual baby-phi codebase. The 3 cascade artifacts:

**(a) Exact `git grep -nE` invocations the planner ran** (at plan-draft time on commit `563dcda`):
- `git -C /root/projects/phi/baby-phi grep -nE 'AgentLoopConfig\s*\{' modules/crates/` → 1 match at `server/src/platform/sessions/launch.rs:526`
- `git -C /root/projects/phi/baby-phi grep -nE 'LlmMessage\s*\{' modules/crates/` → 0 matches (all construction via `LlmMessage::new()`)
- `git -C /root/projects/phi/baby-phi grep -nE 'match\s+(&?event)' modules/crates/ | grep -iE 'AgentEvent|PhiCoreAgentEvent'` → 1 match at `cli/src/commands/agent.rs:642`
- `git -C /root/projects/phi/baby-phi grep -nE 'matches!\(' modules/crates/domain/src/session_recorder.rs` → 1 match at line 166 (PhiCoreAgentEvent::AgentStart pattern with `..` field-wildcard)

**(b) Raw match counts at plan-draft**:
- `AgentLoopConfig { ... }` struct-literals: **1** (the launch.rs site).
- `LlmMessage { ... }` struct-literals: **0** (constructor pattern dominates).
- Exhaustive `match` blocks against `AgentEvent` or `PhiCoreAgentEvent`: **1** (cli/agent.rs).
- `matches!()` blocks against `PhiCoreAgentEvent`: **1** (session_recorder.rs:166 — single-pattern match, does NOT need explicit `RevertApplied` arm).

**(c) Per-file breakdown + predicted edit-site count**:
- `server/src/platform/sessions/launch.rs`: 1 site (D3: insert `revert_pending: None,` line into the existing struct-literal at lines 526–568, after the `response_format` line).
- `cli/src/commands/agent.rs`: 1 site (D4: insert explicit `AgentEvent::RevertApplied { .. } => {}` arm before the existing `_ => {}` line at line 668).
- `domain/src/session_recorder.rs`: 0 sites for the `matches!()` (it's a single-pattern match — explicit arm only adds noise; see §3.E open question for re-evaluation). 0 sites for the LlmMessage call site (constructor pattern).
- `Cargo.toml` (workspace root): 1 line edit (`phi-core = "0.7.1"` → `phi-core = "0.8"`).
- `Cargo.lock`: regenerated via `cargo update -p phi-core` (NOT hand-edited).

**Predicted aggregate edit-site count**: **~3 production-code edits + 1 Cargo.toml edit + Cargo.lock regeneration + 1 NEW drift file + 1 feature-inventory.md row + verified-header amends.**

**Per-fork pause-threshold table** (per chunk-planner v17 P-fork pause-threshold re-derivation):

| Fork | Locked outcome | Δ file-count cap | Δ key-file LOC cap | Δ Cargo.lock cap |
|---|---|---|---|---|
| F1 (CH-28b interstitial) | LOCKED at gate-1.5 | unchanged (no code surface impact) | unchanged | unchanged |
| F2 (`phi-core = "0.8"` semver-range) | LOCKED (planner-rec) | unchanged | unchanged | +<200 lines (Cargo.lock regenerated; bounded by the phi-core 0.8.0 dep-tree closure) |
| F3 (drift at m6/drifts/) | LOCKED (planner-rec) | +1 file (NEW drift file) | new file ≤ 200 LOC (drift template body) | unchanged |

**Aggregate pause-threshold** (per chunk-planner v23 P-plan-1 functional-scope cap derivation): production code surface ≤ 5 LOC delta across launch.rs + agent.rs (1.5× = 7 LOC ceiling); Cargo.toml ≤ 1 LOC delta; NEW drift file ≤ 200 LOC; feature-inventory.md ≤ 10 LOC delta (one §3 row insert per existing precedent). **Implementer pauses via AskUserQuestion if aggregate production-code surface exceeds 8 LOC OR any cascade vector breaches its threshold** (1.5× the predicted ceiling).

### §3.B — K8s microservice readiness check

7-axis evaluation (per per-chunk-template §3.B + ADR-0033):

| Axis | What to check | This chunk's surface | New blocker introduced? | Action |
|---|---|---|---|---|
| **A1** | New in-process state (DashMap, RwLock, AtomicBool, Mutex, OnceCell, etc.) | None — chunk does not introduce new in-process state. `AgentLoopConfig.revert_pending: None` is opt-out at the construction site; the `Arc<Mutex<Vec<RevertRequest>>>` inner type is NEVER instantiated. | **no** | no-op |
| **A2** | New IPC channel (mpsc, broadcast, oneshot, watch, Notify) | None — chunk does not introduce new IPC channels. | **no** | no-op |
| **A3** | New pod-local resource (file handle, listener socket, sub-process, lock file, on-disk cache) | None. | **no** | no-op |
| **A4** | Migration runner / first-apply race | No new migration — chunk is code-only + paperwork; SurrealDB schema unchanged. | **no** | no-op |
| **A5** | Trait-shape requirement (does the new surface need to be trait-objects-friendly for future broker / remote-DB swap?) | No new surface; existing surfaces preserved at the same trait shapes. | **no** | no-op |
| **A6** | Cross-pod state sharing (does this introduce data that must be visible across pods?) | None. | **no** | no-op |
| **A7** | Audit hash-chain symmetry (does the chunk add a new audit writer that breaks single-writer guarantee?) | None — no audit writer changes; AgentEvent variant addition does NOT touch baby-phi's `domain::audit::AuditEvent` (orthogonal surface per phi-core-mapping). | **no** | no-op |

**Conforming-criteria check against ADR-0033 (CH-K8S-PREP)**:
- D33.1 (`SessionRegistry` trait) — NOT touched.
- D33.2 (`SurrealStore::open_remote`) — NOT touched.
- D33.3 (SIGTERM graceful shutdown) — NOT touched.
- D33.4 (`EventBus.shutdown` + `drain`) — NOT touched.

**Conclusion**: **K8s-neutral** — zero new blocker classes introduced. No CHK8S-D-NN ledger entry to file.

### §3.C — User-facing documentation impact map

3-tier evaluation:

| Tier | File pattern | This chunk touches? | Action |
|---|---|---|---|
| **Architecture** | `docs/specs/v0/implementation/m*/architecture/*.md` | No file touched. The chunk is a code-only carrier-fix + deferred-drift filing; no architectural design shifted. | (b) **defer** — no architectural surface affected; the Composition I adoption ADR (future M6+ chunk) will introduce a `docs/specs/v0/implementation/m6/architecture/composition-i-adoption.md` at that time, not now. Successor-chunk reference: future M6+ FUNCTIONAL chunk via D-PHICORE-08-FOLLOWUP-01. |
| **Operations** | `docs/specs/v0/implementation/m*/operations/*.md` | No file touched. No new error codes, audit-event dictionary entries, incident playbooks, or metrics ship at CH-28b close. | (b) **defer** — no operator-visible surface affected; future Composition I adoption chunk handles the operations doc additions. Successor-chunk reference: same as above. |
| **User-guide** | `docs/specs/v0/implementation/m*/user-guide/*.md` | No file touched. No operator-visible behaviour shifts; agents continue to operate identically. | (b) **defer** — no operator-tour or CLI-reference shift. Successor-chunk reference: same as above. |

**Rationale**: this is a TECHNICAL-PREREQUISITE chunk; per chunk-planner v26 + per-chunk-template §2.5, TECHNICAL-PREREQUISITE chunks legitimately defer all 3 tiers when no user-visible delivery ships. The chunk-zero criterion holds (CH-25 P-SEAL `response_format` carrier-fix as precedent — same pattern, same legitimate 3-tier defer).

### §3.D — Forward-scope-vs-concept-doc precedence

**Not applicable** — the chunk does not touch any closed concept-doc invariants (no action vocabulary, no fundamental kinds, no migration order, no frozen schema). The breaking-change surface is bounded by the phi-core CHANGELOG §[0.8.0], which is an upstream concept doc (NOT a baby-phi concept doc); baby-phi's concept docs (`phi-core-mapping.md`, etc.) accommodate the new phi-core types via the existing extensibility pattern.

### §3.E — Anticipated gate-2.5 candidates

**Candidate 1: explicit `RevertApplied { .. } => {}` arm at the session_recorder.rs `matches!()` site (semantic re-examination at P3)**

Per the forward-scope row's D4 description ("explicit `RevertApplied { .. } => {}` arms added at `domain/src/session_recorder.rs:166-170` + `cli/src/commands/agent.rs:642-669`"), the row implies BOTH sites get explicit arms. However, the session_recorder.rs:166-170 site is a `matches!()` block matching a SINGLE pattern (`PhiCoreAgentEvent::AgentStart { session_id, .. }`); it has no `_ =>` to extend with an explicit `RevertApplied` arm — adding `RevertApplied` to a single-pattern `matches!()` would change the boolean semantics (it would now return `true` for both `AgentStart` AND `RevertApplied` events, which is semantically wrong).

**If discovered at gate-2.5**: route to **option-A close-in-chunk via SCOPE-NARROWING** (D4 narrows to only the cli/agent.rs site; session_recorder.rs `matches!()` site is documented as "single-pattern match, explicit arm not applicable" in §3 leverage map). The plan §3 cascade-artifact discipline already reflects this narrowing (the per-file breakdown lists session_recorder.rs as 0 sites). Surface to user at gate-1 OR gate-2.5 if implementer needs explicit ratification.

**Candidate 2: Cargo.lock churn larger than predicted (e.g., transitive dep tree expansion)**

If `cargo update -p phi-core` cascades into ≥ 200-line Cargo.lock churn (beyond the F2 pause-threshold), the implementer pauses via AskUserQuestion. Likely causes: phi-core 0.8.0 added new transitive deps (none expected per CHANGELOG, but possible).

**If discovered at gate-2.5**: route to **option-A close-in-chunk** (Cargo.lock churn is structurally bounded; document the inflated cascade in cycle-audit §6 + proceed to chunk close).

**Candidate 3: tertiary phi-core 0.8.0 surface area not covered by the i-phi diagnostic** (e.g., a phi-core internal symbol baby-phi happens to import + use that has a renamed or removed API)

**If discovered at gate-2.5**: route to **option-A close-in-chunk via carrier-fix** (apply the carrier-fix in this chunk; document at cycle-audit §6) UNLESS the discovery cascade triggers > 8 LOC of carrier-fix surface, in which case route to **option-B file follow-up drift + escalate via AskUserQuestion** (the chunk's TECHNICAL-PREREQUISITE scope can absorb 1-2 small carrier-fixes; larger scope expansions warrant their own dedicated chunk).

### §3.F — SurrealDB SCHEMAFULL Semantic Checklist

**N/A this chunk; no migration ships.** Per chunk-planner v25 P-plan-1, this checklist applies only when the chunk ships a `REMOVE FIELD` / `ALTER TABLE` narrowing / new SCHEMAFULL table / narrowing-UNIQUE-index change. CH-28b ships zero SurrealDB schema changes (the breaking-change surface is phi-core's Rust API only — no SurrealDB types are touched).

---

## §4 — Drifts closed + Deferred functionality

| Drift ID | File | Severity | Transition | Notes |
|---|---|---|---|---|
| `D-PHICORE-08-FOLLOWUP-01` | `docs/specs/v0/implementation/m6/drifts/D-PHICORE-08-FOLLOWUP-01-composition-i-adoption.md` (NEW) | LOW | **NEW drift filed at chunk-seal; Status: `discovered`** | Composition I adoption deferred to future M6+ FUNCTIONAL chunk (TBD per chunk-planner v13 non-terminal-drift discipline — explicit named allocation, NOT `TBD`); per ADR-0064 §D64.5 |

**No existing drifts closed by this chunk.** This is a pure-NEW-drift-filing chunk (the chunk's purpose is to absorb the breaking-change + document the deferral).

### §4.A — Deferred functionality

| Drift ID | User-visible feature deferred | Product impact during deferral | Allocation chunk | Cross-chunk dep |
|---|---|---|---|---|
| `D-PHICORE-08-FOLLOWUP-01` | Agents abandoning failed/finished conversation branches between turns to keep active context lean | In v0 (post-CH-28b): context still grows monotonically per the pre-0.8.0 posture; compaction (existing `BlockCompactionStrategy` + `compact_messages`) is the only relief. After future Composition I adoption: agents emit `revert_to_state` tool calls between turns to drop dead branches; context stays lean; compaction fires less often. | future M6+ FUNCTIONAL chunk (no specific CH-NN slot reserved; explicit named allocation via chunk-planner v13 rule = `"M6+-FUTURE-COMPOSITION-I-ADOPTION"` placeholder pending dedicated dispatch decision) | none upstream (the absorption baseline shift at CH-28b is the only dep); downstream: ALL M6 FUNCTIONAL chunks (CH-33+) benefit conditionally if the adoption chunk lands before they ship their tool-flows |

**Cross-ref**: `docs/specs/v0/feature-inventory.md` §3 Deferred catalogue — NEW row added at chunk-seal mirroring the §3 row shape (Feature impact / User-visible state in v0 / User-visible state at final / Allocation chunk / Cross-chunk dependency) per the existing `D-CH28-FOLLOWUP-01` row precedent (feature-inventory.md:124–129).

---

## §5 — ADRs drafted

**ADR-0064** — phi-core 0.8.0 absorption (Composition I baseline shift)
- **Title**: `ADR-0064 — phi-core 0.8.0 absorption (Composition I braking layer; baseline shift)`
- **Drafted-at-phase**: P0 (plan-mode session 2026-05-23 surfaced the locks; ADR drafted at P-DOCS / P4)
- **Decision-summary**: shift baby-phi's phi-core dependency baseline from 0.7.1 to 0.8.x via semver-range to absorb the Composition I breaking-change release (3 breaking changes per CHANGELOG); apply the 1-line `AgentLoopConfig.revert_pending: None` carrier-fix; add explicit `RevertApplied` match-arm at the cli/agent.rs site; defer Composition I adoption (`.with_revert_tool()`) to future M6+ FUNCTIONAL chunk via NEW drift `D-PHICORE-08-FOLLOWUP-01`.
- **Expected flip-to-Accepted phase**: P-SEAL.
- **ADR file path**: `docs/specs/v0/implementation/m6/decisions/0064-phi-core-08-absorption.md` (per CH-28 ADR-0063 precedent at `m6/decisions/`).

**Sub-decisions** (preliminary; finalized at P-DOCS):
- **§D64.1** — F2.a semver-range form `phi-core = "0.8"` chosen over F2.b exact-pin + F2.c semver-explicit (TECHNICAL FORK; planner-rec). **Pre-existing-behaviour preservation note**: pre-existing scaffold preserved — workspace `Cargo.toml` already used semver-range `phi-core = "0.7.1"` (which is actually exact-pin-shaped per Cargo semantics — `"0.7.1"` and `"^0.7.1"` are equivalent); the shift to `"0.8"` is a structural broadening that aligns with the canonical Cargo idiom for "trust semver-compatible updates."
- **§D64.2** — F1.b CH-28b interstitial chunk-numbering convention (TECHNICAL FORK; user-locked at plan-mode session 2026-05-23). **Pre-existing-absence preserved**: no prior "CH-NNa/b" suffix convention shipped in baby-phi's cycle-index; CH-28b establishes the convention as canonical for "interstitial chunks that absorb upstream breaking-changes without renumbering downstream cycles." Variation (c) "never-shipped-yet" per chunk-planner v11 P-plan-1 + CH-19 ADR-0057 §D57.6 precedent.
- **§D64.3** — `AgentLoopConfig.revert_pending: None` carrier-fix preserves opt-out posture (Composition I NOT enabled at session launch). **Pre-existing-behaviour preservation note**: pre-existing scaffold preserved — `launch.rs:526-568` struct-literal already received CH-25's `response_format: ResponseFormat::default()` carrier-fix at lines 558–567; the CH-28b carrier-fix follows the identical pattern (minimal 1-line additive field with semantic-default = opt-out).
- **§D64.4** — F3.a drift filing at canonical `m6/drifts/` directory chosen over F3.b feature-inventory-only embedding. **Pre-existing-behaviour preservation note**: pre-existing scaffold preserved — baby-phi's drift-directory convention (`m<N>/drifts/`) ratified at CH-28 ADR-0063's filing of `D-CH28-FOLLOWUP-01` at `m6/drifts/`; CH-28b's `D-PHICORE-08-FOLLOWUP-01` filing follows the same pattern.
- **§D64.5** — Composition I adoption deferred to future M6+ FUNCTIONAL chunk via `D-PHICORE-08-FOLLOWUP-01`. **Pre-existing-absence preserved**: no Composition I adoption surface has shipped in baby-phi (the feature is opt-in upstream + not yet enabled). Variation (c) "never-shipped-yet" per chunk-planner v11.
- **§D64.6** — Explicit `AgentEvent::RevertApplied { .. } => {}` arm added at `cli/src/commands/agent.rs:642-669` for future-reader discoverability; NOT added at `domain/src/session_recorder.rs:166-170` per §3.E candidate 1 (single-pattern `matches!()` site; adding the arm would change boolean semantics — explicit-arm pattern doesn't apply). **Pre-existing-behaviour preservation note**: pre-existing wildcard arm preserved — `cli/agent.rs:668` `_ => {}` already covers the new variant at compile-time; the explicit `RevertApplied` arm is cosmetic + signals awareness of the 0.8.0 variant addition.

**ADR top-level section enumeration (per chunk-planner v17 P-plan-2)**: the implementer authors all 7 canonical sections:
1. `## Forks` (TECHNICAL FORK F1 + F2 + BOUNDED USER-VISIBLE F3; all planner-rec or RESOLVED — Direct-approval shape per cycle precedent)
2. `## Context` (cites forward-scope row + i-phi diagnostic + phi-core CHANGELOG §[0.8.0])
3. `## Sub-decisions` (§D64.1–§D64.6 above; each ends with Pre-existing-behaviour preservation note per v11 strict-or-variation form)
4. `## Cross-references` (4 categories per CH-13 retro Row 1: (a) concept doc + line range — phi-core CHANGELOG §[0.8.0] lines 9-94 + concept-brake.md §5; (b) closed drift(s) by ID — NONE closed; NEW drift D-PHICORE-08-FOLLOWUP-01 filed; (c) prior ADRs cited as precedent — ADR-0063 §D63.16 (drift-filing precedent at m6/drifts/) + CH-25 P-SEAL `response_format` carrier-fix at launch.rs:558-567 (struct-literal carrier-fix precedent); (d) forward-scope row — m6-forward-scope-8b7a8bcd.md lines 56-67 + line 319)
5. `## Consequences` (one `### For <downstream>` subsection per: (i) `### For CH-29` (next M6 chunk; consumes the 0.8.x baseline + can build/test the AgentMessage value-object substrate without any phi-core surprises), (ii) `### For CH-33` (first FUNCTIONAL M6 chunk; consumes `phi_core::types::tool::AgentTool` for `read_inbox`/`send_message` tools — AgentTool surface unchanged at 0.7.1 → 0.8.0 per CHANGELOG), (iii) `### For future M6+ Composition I adoption chunk` (D-PHICORE-08-FOLLOWUP-01 closing chunk; can call `BasicAgent::with_revert_tool()` + tune `RevertRenderPolicy` against the absorbed baseline))
6. `## Revisit triggers` — 4 bullets:
   - phi-core 0.9.x ships with another breaking-change release → revisit §D64.1 semver-range form (consider tighter pin if upstream semver discipline drifts)
   - Composition I adoption chunk approaches dispatch → revisit §D64.5 deferral + close drift D-PHICORE-08-FOLLOWUP-01
   - More than 3 chunks defer to "M6+ FUTURE COMPOSITION I" placeholder → revisit explicit named allocation (chunk-planner v13 non-terminal-drift rule may trigger orchestrator AskUserQuestion for tighter allocation)
   - phi-core CHANGELOG adds a new `AgentEvent` variant before Composition I adoption ships → revisit §D64.6 explicit-arm coverage at cli/agent.rs (existing `_ => {}` continues to cover compile-time; explicit-arm needs extending)
7. `## Verification` (commands per §12 of this plan)

---

## §6 — Prior-chunk regression re-verification

| Upstream chunk | Invariant this chunk relies on | Re-verification command |
|---|---|---|
| CH-28 (`0412eb06`) | AgentProfile cardinality redesign shipped; AgentProfileWireRow strip + read-path synthesis intact; ADR-0063 §D63.13–§D63.16 honored | `cargo test --workspace --test acceptance_m6_agent_profile_cardinality` — expect 7 acceptance tests green |
| CH-25 P-SEAL workspace-health | `response_format: ResponseFormat::default()` carrier-fix at `launch.rs:558-567` intact (the SAME struct-literal CH-28b modifies) | `grep -n "response_format: phi_core::provider::traits::ResponseFormat::default" /root/projects/phi/baby-phi/modules/crates/server/src/platform/sessions/launch.rs` — expect ≥ 1 hit |
| CH-17 / ADR-0055 §D55.1 | broadcast-tap publish at `session_recorder.rs:180-187` intact | `grep -n "broadcast_tx" /root/projects/phi/baby-phi/modules/crates/domain/src/session_recorder.rs` — expect ≥ 2 hits |
| All prior chunks (workspace-wide test count) | Baseline test count carry-forward; CH-28 close was 1599 tests | `cargo test --workspace --no-fail-fast` — expect **1599 passed** (NO new tests added at CH-28b; verification recipe in §12) |
| All prior chunks (phi-core leverage baseline) | `grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ \| wc -l` = 57 | run the grep; expect 57 |
| All prior chunks (CI guards) | 4 CI guards green | run each `bash scripts/check-*.sh`; expect exit 0 |

**Named expected-still-green tests** (grep-verified at plan-draft time via `grep -hE "^fn (test_|smoke_)" /root/projects/phi/baby-phi/modules/crates/*/tests/*.rs | head -30`) — per chunk-planner v17 P-plan-3:

- `test_session_recorder_emits_session_started_on_agent_start_for_ctx` (session_recorder_test.rs)
- `test_acceptance_m6_agent_profile_cardinality_create_with_template` (acceptance_m6_agent_profile_cardinality.rs)
- `test_acceptance_m6_agent_profile_cardinality_override_upsert` (acceptance_m6_agent_profile_cardinality.rs)
- `test_launch_session_with_default_runtime_config` (launch_test.rs)

These tests indirectly cover all 3 phi-core 0.8.0 surfaces: AgentLoopConfig struct-literal compilation (`test_launch_session_with_default_runtime_config`), AgentEvent match coverage (`test_session_recorder_*` + acceptance tests), LlmMessage constructor usage (acceptance tests). Any regression at chunk-close blocks chunk-seal.

This table runs AT CHUNK OPEN (P0 pre-flight) before any phase opens, and again at chunk seal (P-SEAL paperwork).

---

## §7 — Phases within the chunk

**§7.0 — Phase-order stress-test (per chunk-planner v25 P-plan-4)**

This chunk has **5 phases** (P1 / P2 / P3 / P4 / P-SEAL); 1 user-locked DIVERGENT fork at gate-1 (F1 was user-locked at plan-mode session — though the lock is RESOLVED rather than divergent from planner-rec since the planner-rec at plan-mode session was F1.b); zero overlapping cascade-bands (each phase has a distinct deliverable surface). v25 P-plan-4 threshold (≥ 2 of: > 5 phases / ≥ 1 user-locked DIVERGENT / cascade-band overlap) is **NOT met** (only 1 of 3 criteria met). Full §7.0 stress-test walk NOT required; abbreviated walk:

- **P1 → P2 boundary**: P1 bumps Cargo.toml + runs `cargo update`; expect 1 compile error at `launch.rs:526` (the predicted struct-literal break). P2 adds the field. Workspace-RED window: P1 close → P2 close (~minutes; well-bounded; matches CH-25 precedent pattern).
- **P2 → P3 boundary**: workspace-GREEN; P3 adds the explicit match-arm (semantic-neutral; existing wildcard covers it).
- **P3 → P4 boundary**: workspace-GREEN; P4 is paperwork (drift file + feature-inventory row + ADR draft); no code surface touched.
- **P4 → P-SEAL boundary**: workspace-GREEN; P-SEAL is verified-header amends + cycle-index Status flip.

Anticipated workspace-RED window: 1 phase boundary (P1 → P2 close). Bounded + expected per the carrier-fix pattern; matches the CH-25 P-SEAL precedent shape.

---

**P1 — Cargo bump + lock update + verify build (expect 1 predicted compile error)**

- **Goal**: shift workspace dependency baseline from `phi-core = "0.7.1"` to `phi-core = "0.8"`; regenerate Cargo.lock; verify the predicted 1 compile error at `launch.rs:526` materializes (validates the diagnostic + drives P2 scope).
- **Deliverables** (numbered; per D1 + D2 in the forward-scope row):
  1. Edit `/root/projects/phi/baby-phi/Cargo.toml` line 17: `phi-core = "0.7.1"` → `phi-core = "0.8"`.
  2. Run `/root/rust-env/cargo/bin/cargo update -p phi-core --manifest-path /root/projects/phi/baby-phi/Cargo.toml`; expect Cargo.lock regeneration with phi-core 0.8.0 + transitive deps.
  3. Run `/root/rust-env/cargo/bin/cargo build --workspace --manifest-path /root/projects/phi/baby-phi/Cargo.toml -j 4` — expect **EXACTLY 1 compile error** at `modules/crates/server/src/platform/sessions/launch.rs:526` of the form `missing field `revert_pending` in initializer of `phi_core::AgentLoopConfig``.
- **Tests**: no new tests added at P1; existing tests do not run (workspace-RED).
- **Concept-alignment check**: §2 row "AgentLoopConfig revert_pending field" transitions from `contradicted` to `contradicted` (still contradicted at P1 close — P2 flips it to `honored`).
- **phi-core leverage check**: baseline grep `grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l` should still return 57 (no source-level imports changed).
- **User-facing doc updates**: NONE (per §3.C deferred).
- **Confidence target**: ≥ 99% (mechanical Cargo bump with verified predicted single-error outcome).
- **Pause discipline**: PAUSE via AskUserQuestion IF (a) compile errors > 1 OR (b) Cargo.lock churn > 200 lines OR (c) any unanticipated phi-core API surface area surfaces (per §3.E candidate 3).

**P2 — AgentLoopConfig field add + workspace GREEN restoration**

- **Goal**: apply the D3 carrier-fix at `launch.rs:526-568`; restore workspace-GREEN.
- **Deliverables** (per D3):
  1. Open `/root/projects/phi/baby-phi/modules/crates/server/src/platform/sessions/launch.rs`. After the existing `response_format: phi_core::provider::traits::ResponseFormat::default(),` line at line 567 (last field before closing `};` at line 568), add a new line: `revert_pending: None,` — preserving the trailing comma + indentation pattern. Document with a 4-line inline comment block paralleling CH-25's `response_format` comment block style (lines 558-567), citing CH-28b chunk + ADR-0064 §D64.3 + the opt-out posture rationale.
  2. Run `/root/rust-env/cargo/bin/cargo build --workspace --manifest-path /root/projects/phi/baby-phi/Cargo.toml -j 4`; expect 0 errors.
  3. Run `/root/rust-env/cargo/bin/cargo test --workspace --manifest-path /root/projects/phi/baby-phi/Cargo.toml -j 4 --no-fail-fast`; expect **1599 passed** (baseline unchanged).
  4. Run `cargo clean --manifest-path /root/projects/phi/baby-phi/Cargo.toml` (per chunk-implementer v8 placement-1 immediate-post-test cleanup).
- **Tests**: 0 new tests; existing 1599 expected to remain green.
- **Concept-alignment check**: §2 row "AgentLoopConfig revert_pending field" flips `contradicted → honored`.
- **phi-core leverage check**: baseline grep returns 57 (unchanged); `revert_pending: None` does NOT introduce a new `use phi_core::...` line (already-imported AgentLoopConfig type used).
- **User-facing doc updates**: NONE.
- **Confidence target**: ≥ 99% (mechanical 1-line field add; CH-25 P-SEAL precedent is identical pattern).
- **Pause discipline**: PAUSE via AskUserQuestion IF test count diverges from 1599 OR any test fails OR clippy warnings appear.

**P3 — Match-arm tightening (cli + clippy + fmt)**

- **Goal**: apply the D4 explicit `RevertApplied { .. } => {}` arm at the cli/agent.rs match site (NOT at the session_recorder.rs `matches!()` site per §3.E candidate 1); ensure clippy + fmt green.
- **Deliverables** (per D4 narrowed to single site):
  1. Open `/root/projects/phi/baby-phi/modules/crates/cli/src/commands/agent.rs`. Before the existing `_ => {}` line at line 668, add an explicit arm: `AgentEvent::RevertApplied { .. } => {}` with a 2-line inline doc-comment citing CH-28b + ADR-0064 §D64.6 noting "Composition I opt-in remains NOT enabled at CH-28b; explicit arm is cosmetic + signals 0.8.0 variant awareness."
  2. Run `/root/rust-env/cargo/bin/cargo fmt --all --manifest-path /root/projects/phi/baby-phi/Cargo.toml -- --check`; expect 0 diff.
  3. Run `RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml --workspace --all-targets`; expect 0 warnings (sub-agent auditors will mark as `NOT-EXECUTED-IN-AUDIT` per sandbox-block; orchestrator gate-4 runs).
  4. Re-run `cargo test --workspace --no-fail-fast`; expect 1599 passed.
  5. `cargo clean` per chunk-implementer v8 placement-1.
- **Tests**: 0 new tests.
- **Concept-alignment check**: §2 row "AgentEvent #[non_exhaustive] + RevertApplied variant" stays `honored` (existing wildcards already covered; explicit arm adds discoverability without changing semantics).
- **phi-core leverage check**: baseline grep returns 57 (unchanged).
- **User-facing doc updates**: NONE.
- **Confidence target**: ≥ 99%.
- **Pause discipline**: PAUSE via AskUserQuestion IF clippy warnings appear OR fmt diff non-empty OR test count diverges from 1599.

**P4 — Docs + drift + paperwork (ADR + drift file + feature-inventory)**

- **Goal**: file NEW drift; add feature-inventory row; draft ADR-0064; amend touched-doc verified headers.
- **Deliverables** (per D5 + D6):
  1. Create NEW file `/root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/drifts/D-PHICORE-08-FOLLOWUP-01-composition-i-adoption.md` mirroring the existing `D-CH28-FOLLOWUP-01-blueprint-upserted-template-fanout.md` shape (verified-header line 1; ## Identification section with all fields; ## Concept alignment; ## Plan vs reality; etc.). Status: `discovered`. Bucket: B. Severity: LOW. Closing chunk: **future M6+ FUNCTIONAL chunk** (named placeholder pending dedicated dispatch decision; NOT `TBD`). Body documents the 4 Composition I adoption prerequisites: (i) `BasicAgent::with_revert_tool()` call site at `launch.rs:526-568` build path, (ii) `RevertRenderPolicy` defaults tuned for baby-phi's compaction posture, (iii) revert-discipline skill teaching agents the 4 categories failure/tangent/completion/step-summary, (iv) acceptance suite exercising `RevertApplied` event flow + render-policy filtering.
  2. Edit `/root/projects/phi/baby-phi/docs/specs/v0/feature-inventory.md` §3 Deferred catalogue (between line 122 and line 124, before the existing `D-CH28-FOLLOWUP-01` row): add a new `### D-PHICORE-08-FOLLOWUP-01 — Composition I adoption` row with Feature impact / User-visible state in v0 / User-visible state at final / Allocation chunk / Cross-chunk dependency fields per §4.A above. Bump §1 verified header date to 2026-05-23 with CH-28b citation.
  3. Create NEW ADR file `/root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/decisions/0064-phi-core-08-absorption.md` with the 7 canonical sections per §5 above; sub-decisions §D64.1–§D64.6; status: **Proposed** at P4 (flips to **Accepted** at P-SEAL).
  4. Amend verified header on `/root/projects/phi/baby-phi/docs/specs/plan/forward-scope/m6-forward-scope-8b7a8bcd.md` (already prerequisite-committed at `563dcda`; D7 collapses to a header-date freshness check — confirm the existing CH-28b authoring header is intact; no body changes).
- **Tests**: 0 new code tests. Doc-links check (`bash /root/projects/phi/baby-phi/scripts/check-doc-links.sh`) MUST pass after edits.
- **Concept-alignment check**: §2 rows for `concept-brake.md` + `phi-core CHANGELOG` reach target status (`honored` for the absorption rows; `concept-aspirational` preserved for the Composition I adoption row per F3.a defer).
- **phi-core leverage check**: baseline grep returns 57 (unchanged).
- **User-facing doc updates**: per §3.C deferred. The NEW drift file + ADR-0064 + feature-inventory row are GOVERNANCE-tier docs (not user-facing-tier per §3.C definition); they ARE first-class deliverables.
- **Confidence target**: ≥ 99% (paperwork pattern matches CH-25 + CH-28 precedents).
- **Pause discipline**: PAUSE via AskUserQuestion IF doc-links check fails OR feature-inventory.md edit cascades unexpectedly (e.g., §3 row count change breaks a §2 cross-ref).

**P-SEAL — Verified-header sweep + cycle-index Status flip + ADR-0064 Accepted flip**

- **Goal**: chunk-seal paperwork; cycle-index row appended at active-cycles tail with Status `ready-for-audit`; ADR flipped Proposed → Accepted; verified-header amends on all touched docs.
- **Deliverables**:
  1. Append cycle-index row at `/root/projects/phi/baby-phi/docs/specs/plan/build/_cycle-index.md` (per chunk-implementer P-SEAL v17 + chunk-archive-plan skill v2):
     - Hex link: `[\`d5b776ac\`](ch-28b-phi-core-08-absorption-d5b776ac/plan.md)`
     - Slug + summary: `CH-28b — phi-core 0.8.0 absorption (Composition I baseline shift; TECHNICAL-PREREQUISITE); 5 phases (P1+P2+P3+P4+P-SEAL); 1 carrier-fix at launch.rs:526 + 1 explicit match-arm at cli/agent.rs:668; NEW drift D-PHICORE-08-FOLLOWUP-01 filed; ADR-0064 Accepted with 6 sub-decisions §D64.1–§D64.6; phi-core import baseline preserved at 57; workspace test count preserved at 1599; cargo test --no-fail-fast GREEN at chunk close`
     - Phase count: `5`
     - Auditor count: `1 (audit envelope: SMALL per §11)`
     - Iterations: `pending` (per chunk-planner v16 P-SEAL canonical lifecycle — orchestrator owns the transition at retro-complete)
     - Status: `in-flight` (per chunk-planner v16 P-SEAL canonical lifecycle — orchestrator transitions at gate-3/gate-4/retro-complete)
     - Test count: `1599`
  2. Flip ADR-0064 status header: `Proposed` → `Accepted` at top of `0064-phi-core-08-absorption.md`; bump verified header on the ADR file.
  3. Amend verified headers on: `_cycle-index.md` (row 1 + row 2 comment lines), `m6-forward-scope-8b7a8bcd.md` (already prerequisite-committed), `feature-inventory.md`, NEW `D-PHICORE-08-FOLLOWUP-01-composition-i-adoption.md`.
  4. Run final verification suite per §12 below: 4 CI guards + cargo fmt --check + cargo test --workspace --no-fail-fast (expect 1599 passed) + chunk-specific greps.
  5. `cargo clean` per chunk-implementer v8 placement-1 (final placement-1 invocation; orchestrator owns placement-2 at gate-5 close).
- **Tests**: 0 new tests; full verification suite re-run.
- **Concept-alignment check**: all §2 rows at target status.
- **phi-core leverage check**: baseline grep returns 57; positive close-audit greps all green per §3.
- **User-facing doc updates**: NONE.
- **Confidence target**: ≥ 99% (composite ≥ 9/10 per §10 + §12 below).
- **Pause discipline**: PAUSE via AskUserQuestion IF cycle-index row append fails OR any verified-header amend conflicts OR any §6 carry-forward invariant regresses.

---

## §8 — Tests summary

**Expected total test count at chunk close**: **1599** (baseline from CH-28 close; ZERO new tests added at CH-28b — chunk is a code-only carrier-fix + paperwork; no new behaviour to test).

**Test-count band** (asymmetric per CH-12 retro): `[1599, 1599]` (point estimate; no buffer needed — the chunk adds 0 NEW MUST-SHIP + 0 MAY-COVER tests).

**Pause condition** (per chunk-planner v17 + v23 P-plan-1 functional-scope-derived):
- If test count > 1599 → AskUserQuestion (unexpected test additions; likely indicates scope creep or hidden behavior change).
- If test count < 1599 → AskUserQuestion (regression; tests should remain stable at baseline).

**MUST-SHIP test enumeration**: NONE this chunk.

**MAY-COVER test enumeration**: NONE this chunk.

**Layer breakdown** (no shift from CH-28 close):
- unit: per existing distribution
- integration: per existing distribution
- acceptance: 7 m6_agent_profile_cardinality + all prior
- e2e: per existing distribution

**Named expected-still-green tests** (carry-forward; per §6 above + chunk-planner v17 P-plan-3 grep-verify):
- `test_session_recorder_emits_session_started_on_agent_start_for_ctx`
- `test_acceptance_m6_agent_profile_cardinality_create_with_template`
- `test_acceptance_m6_agent_profile_cardinality_override_upsert`
- `test_launch_session_with_default_runtime_config`

Per-Tier test-cardinality breakdown (per chunk-planner v22 P-plan-1) — N/A this chunk (no new tests; existing tier distribution preserved).

---

## §9 — Pre-chunk gate

**Reading list (mandatory)** — the implementer reads BEFORE P1 opens:

1. **Forward-scope row**: [`forward-scope/m6-forward-scope-8b7a8bcd.md`](../../forward-scope/m6-forward-scope-8b7a8bcd.md) lines 56–67 + line 319 (the CH-28b row + table row).
2. **i-phi diagnostic** (primary input): [`/root/projects/phi/i-phi/docs/v0/proposal/plan/build/phi-core-revert-tool-27c894f6/plan.md`](../../../../../i-phi/docs/v0/proposal/plan/build/phi-core-revert-tool-27c894f6/plan.md) §DIAGNOSTIC SECTION A — empirical breaking-change impact for baby-phi (predicted LOW; verified at plan-draft).
3. **phi-core CHANGELOG**: [`/root/projects/phi/phi-core/CHANGELOG.md`](../../../../../phi-core/CHANGELOG.md) §[0.8.0] lines 9–94 (the 3 breaking-changes enumeration + 7 added items + documentation block).
4. **phi-core concept-brake**: [`/root/projects/phi/phi-core/docs/concepts/concept-brake.md`](../../../../../phi-core/docs/concepts/concept-brake.md) §5 Composition I (design source-of-truth for what's being deferred).
5. **CH-25 P-SEAL precedent**: read the existing `response_format: phi_core::provider::traits::ResponseFormat::default(),` carrier-fix at `/root/projects/phi/baby-phi/modules/crates/server/src/platform/sessions/launch.rs:558-567` (the structural pattern P2 replicates).
6. **CH-28 ADR-0063 (m6 ADR precedent)**: [`/root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/decisions/0063-agent-profile-cardinality-redesign.md`](../../../v0/implementation/m6/decisions/0063-agent-profile-cardinality-redesign.md) — file naming + section shape baseline for ADR-0064.
7. **CH-28 drift precedent**: [`/root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/drifts/D-CH28-FOLLOWUP-01-blueprint-upserted-template-fanout.md`](../../../v0/implementation/m6/drifts/D-CH28-FOLLOWUP-01-blueprint-upserted-template-fanout.md) — drift file shape + naming baseline for D-PHICORE-08-FOLLOWUP-01.
8. **feature-inventory.md §3 Deferred catalogue** (existing D-CH28-FOLLOWUP-01 row at lines 124–129) — row shape baseline.
9. **baby-phi CLAUDE.md** §"phi-core Leverage" rules 1–5.

**Carry-forward invariants** (explicit list, verified green at chunk open):
- `cargo test --workspace --no-fail-fast` test count = **1599** (CH-28 close baseline).
- `scripts/check-phi-core-reuse.sh` green.
- `scripts/check-doc-links.sh` green.
- `scripts/check-ops-doc-headers.sh` green.
- `scripts/check-spec-drift.sh` green.
- `grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l` = **57**.
- `modules/` diff against the chunk-open git HEAD is empty.
- Cargo.toml workspace-deps row: `phi-core = "0.7.1"` (will flip to `phi-core = "0.8"` at P1).

**Pending decisions carried into this chunk**:
- F1 / F2 / F3 forks ratified at gate-1 per ExitPlanMode + AskUserQuestion (F1 is RESOLVED at prerequisite commit `563dcda`; F2 + F3 are planner-rec rubber-stamp expected).
- §3.E candidate 1 (session_recorder.rs `matches!()` site explicit-arm omission) ratified at gate-1 OR escalated at gate-2.5 if implementer prefers explicit raised-question route.

**Chunk-ordering note**: per forward-scope row line 62, CH-28b has **no prerequisites** (CH-28 closed at `0412eb06` 2026-05-20 — orthogonal to CH-28's schema redesign).

---

## §10 — Close criteria

**Composite 4-aspect + 2 confidence % ritual**:

**4 aspects** (each pass/fail):

- **Code aspect**:
  - All 4 deliverables (D1 + D2 + D3 + D4) shipped per §7 phases.
  - `/root/rust-env/cargo/bin/cargo test --workspace --no-fail-fast -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml` returns **1599 passed / 0 failed**.
  - `RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy -j 4 --workspace --all-targets --manifest-path /root/projects/phi/baby-phi/Cargo.toml` returns 0 warnings (marked `NOT-EXECUTED-IN-AUDIT` by sub-agents; orchestrator closes at gate-4 MUST-RUN list).
  - `/root/rust-env/cargo/bin/cargo fmt --all -- --check --manifest-path /root/projects/phi/baby-phi/Cargo.toml` returns 0 diff.
- **Docs aspect**:
  - *Governance tier*: NEW drift `D-PHICORE-08-FOLLOWUP-01` filed at canonical `m6/drifts/`; ADR-0064 flipped Proposed → Accepted at P-SEAL; feature-inventory.md §3 row added; cycle-index row appended; verified-header amends on all 4 touched docs.
  - *User-facing tier* (per §3.C): all 3 rows have explicit defer-decision with successor-chunk reference (future M6+ Composition I adoption chunk via `D-PHICORE-08-FOLLOWUP-01`).
- **phi-core leverage aspect**:
  - `grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l` = **57** (baseline preserved; Δ +0 imports per §3 prediction).
  - All 4 positive close-audit greps return expected hits per §3.
  - All 4 forbidden-duplication greps return 0 hits per §3.
  - `bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh` exit 0.
- **Concept alignment aspect**:
  - All §2 table rows at target chunk-close status (4 rows at `honored`; 2 rows at `concept-aspirational` preserved per F3.a defer; 2 rows at `honored` per existing baseline).

**2 confidence %** (each with named numerator/denominator):

- **Implementation confidence %** = (claims-verified-honored-by-tests-and-code-inspection) / (total-claims-in-scope-for-chunk) = **9/10** target.
  - Numerator-9: D1 Cargo.toml bump verified by §12 grep; D2 Cargo.lock regen verified by `cargo update` exit-0; D3 carrier-fix verified by §12 grep + workspace test pass; D4 explicit match-arm verified by §12 grep + clippy pass; D5 NEW drift file verified by file presence + doc-links check; D6 feature-inventory row verified by grep; D7 verified-header amends verified by `Last verified: 2026-05-23` grep across 4 docs; phi-core import baseline preserved verified by grep; cycle-index row append verified by grep.
  - Numerator-1 retained: §3.E candidate 1 (session_recorder.rs `matches!()` site explicit-arm omission) is a planner judgment call documented in §3 leverage map + ADR §D64.6; implementer ratifies at P3 — if implementer chooses to add an explicit arm at the `matches!()` site (changing single-pattern → multi-pattern), it would change boolean semantics + require a separate code path. The 1 retained claim is "implementer adopts planner's judgment without re-litigation"; if implementer surfaces concern at P3, gate-2.5 routes per §3.E.
  - Target: ≥ 9/10 satisfies Direct-approval criterion per chunk-planner v0 + chunk-initiate skill threshold.
- **Documentation confidence %** = (doc-pages-where-independent-reader-can-cross-check-against-code-+-concept-+-ADRs-without-ambiguity) / (doc-pages-touched-in-chunk) = **4/4 = 100%** target.
  - Numerator-4: NEW drift `D-PHICORE-08-FOLLOWUP-01-composition-i-adoption.md` (independent reader can cross-check the Composition I deferral against phi-core CHANGELOG §[0.8.0] + ADR-0064 §D64.5); NEW ADR-0064 (independent reader can cross-check sub-decisions against the forward-scope row + concept docs); feature-inventory.md §3 row (independent reader can cross-check user-facing translation against drift body + ADR §D64.5); cycle-index row (independent reader can cross-check against plan.md + chunk-implementer P-SEAL report).

**Composite** = `min(impl%, doc%, code-aspect-binary, phi-core-leverage-aspect-binary, concept-alignment-aspect-binary)` = `min(90%, 100%, pass, pass, pass)` = **90%**.

**Explicit close-target discipline**: close report states ALL FIVE measures with named numerators/denominators. No aspect-averaging. No rounding up.

**P-SEAL paperwork checklist** (per chunk-planner v11 + v17):
- All 4 touched-doc verified headers match body diff exactly.
- `_concept-audit-matrix.md`: NOT touched by this chunk (no concept-doc body changes; §2 rows reach target status via existing audit-matrix shape).
- Cycle-index row appended at active-cycles tail per chunk-planner v17 + chunk-archive-plan skill v3 (the chunk-planner v22 P13 + chunk-archive-plan v3 hard-assertion for `### Locked fork details` heading fires at archive-time; verified GREEN via §1 above).
- Cargo-clean per chunk-implementer v8 placement-1 applied after each cargo test invocation.

---

## §11 — Post-chunk independent audit plan

**Phase count**: **5 (P1 + P2 + P3 + P4 + P-SEAL)** — per audit-envelope-size skill sizing rule, this lands in the **3–5 phases / Medium envelope (2 auditors A + B)** band. However, the chunk's surface area is **mechanical compile-restoration + paperwork** (no new code logic, no new tests, no new concept-doc surface), and the §1 locked-fork-details + §3 leverage map + §3.B K8s neutrality combine to keep the audit scope tight. Per audit-envelope-size skill's underlying intent (phase count is a proxy for audit complexity, not the load-bearing signal), the planner sizes this chunk as **SMALL (1 auditor letter A)** with the following justification:

**Sizing override rationale**: phase count 5 lands at the Medium boundary, but 4 of the 5 phases (P1 Cargo bump / P2 1-line carrier-fix / P3 1-line match-arm / P-SEAL paperwork) are ≤ 5 LOC each + carry mechanical-replication patterns (CH-25 + CH-28 precedents). Only P4 (drift + feature-inventory + ADR draft) has docs-fidelity surface to audit; the entire chunk's audit surface fits comfortably in one auditor's scope. The planner self-confirms SMALL via parallel reasoning to CH-25 P-SEAL workspace-health fix (also SMALL with mechanical carrier-fix shape).

**Audit envelope**: **SMALL (1 auditor letter A)** — combined code + phi-core + K8s + concept + docs + ADR coverage.

**Audit aspects (a–d)**:
- (a) Code correctness (D1 Cargo bump + D2 lock regen + D3 carrier-fix + D4 explicit arm).
- (b) Docs fidelity vs concept docs (NEW drift body matches phi-core CHANGELOG §[0.8.0]; ADR-0064 sub-decisions match §5; feature-inventory row matches drift body).
- (c) Concept alignment across §2 rows.
- (d) phi-core leverage (Δ +0 leverage-sites; baseline 57 preserved; no forbidden duplications).

### Audit A scaffold (≤ 600 words; combined code + concept + docs + phi-core + K8s)

```
You are auditing CH-28b (phi-core 0.8.0 absorption / Composition I baseline shift)
in baby-phi at /root/projects/phi/baby-phi/. Read-only on source. Plan at
docs/specs/plan/build/ch-28b-phi-core-08-absorption-d5b776ac/plan.md.

Verify each claim with file:line citation:

1. Cargo.toml workspace-deps row at /root/projects/phi/baby-phi/Cargo.toml line 17
   reads `phi-core = "0.8"` (NOT "0.7.1", NOT "0.8.0", NOT "^0.8.0"). Run:
   `grep -nE '^phi-core = "0.8"$' /root/projects/phi/baby-phi/Cargo.toml`. PASS if exit 0 + 1 hit.

2. Cargo.lock regenerated against phi-core 0.8.x. Run:
   `grep -nE '^name = "phi-core"' /root/projects/phi/baby-phi/Cargo.lock`
   followed by 2 lines for version. Expect `version = "0.8.0"` (or higher 0.8.x).

3. AgentLoopConfig carrier-fix at
   `/root/projects/phi/baby-phi/modules/crates/server/src/platform/sessions/launch.rs`
   contains `revert_pending: None,` after the existing
   `response_format: phi_core::provider::traits::ResponseFormat::default(),` field.
   Run: `grep -nB1 'revert_pending: None' modules/crates/server/src/platform/sessions/launch.rs`.
   PASS if `response_format` is the immediately-preceding line; FAIL otherwise.

4. Explicit match-arm at
   `/root/projects/phi/baby-phi/modules/crates/cli/src/commands/agent.rs`
   adds `AgentEvent::RevertApplied { .. } => {}` before the existing `_ => {}` line.
   Run: `grep -nE 'AgentEvent::RevertApplied' modules/crates/cli/src/commands/agent.rs`.
   PASS if ≥ 1 hit at a line < the existing `_ => {}` line (around line 668).

5. session_recorder.rs `matches!()` site at line 166-170 is NOT modified
   (per plan §3.E candidate 1 — single-pattern match; explicit arm doesn't apply).
   Run: `git diff HEAD~ -- modules/crates/domain/src/session_recorder.rs |
   grep -c 'RevertApplied'`. Expect 0.

6. NEW drift filed at canonical m6/drifts/ path. Run:
   `ls /root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/drifts/D-PHICORE-08-FOLLOWUP-01-composition-i-adoption.md`.
   PASS if file exists with line 1 `<!-- Last verified: 2026-05-23 by Claude Code -->` header.

7. ADR-0064 filed at canonical m6/decisions/ path with Status: Accepted at P-SEAL.
   Run: `head -10 /root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/decisions/0064-phi-core-08-absorption.md`.
   PASS if Status line reads `Status: Accepted` + sub-decisions §D64.1–§D64.6 referenced.

8. feature-inventory.md §3 row added for D-PHICORE-08-FOLLOWUP-01. Run:
   `grep -nE '### D-PHICORE-08-FOLLOWUP-01' /root/projects/phi/baby-phi/docs/specs/v0/feature-inventory.md`.
   PASS if ≥ 1 hit + row body has 5 fields (Feature impact / User-visible v0 / User-visible final / Allocation chunk / Cross-chunk dep).

9. cycle-index row appended for `d5b776ac`. Run:
   `grep -n 'd5b776ac' /root/projects/phi/baby-phi/docs/specs/plan/build/_cycle-index.md`.
   PASS if ≥ 1 hit at active-cycles tail; row shape per chunk-implementer P-SEAL v17.

10. cargo test --workspace --no-fail-fast green at expected count 1599. Run:
    `cargo test --workspace --no-fail-fast --manifest-path /root/projects/phi/baby-phi/Cargo.toml -j 4`.
    Mark NOT-EXECUTED-IN-AUDIT if sandbox-blocked; orchestrator closes at gate-4.

11. CI guards green; check-phi-core-reuse.sh exit 0; no new `use phi_core::`
    imports beyond §3 prediction (baseline 57 preserved). Run:
    `bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh` (mark
    NOT-EXECUTED-IN-AUDIT if sandbox-blocked). Run:
    `grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l`;
    expect 57.

12. CH-25 P-SEAL invariant intact: `response_format: phi_core::provider::traits::ResponseFormat::default()`
    still present at `launch.rs:567`. Run:
    `grep -n 'response_format: phi_core::provider::traits::ResponseFormat::default' modules/crates/server/src/platform/sessions/launch.rs`.
    PASS if ≥ 1 hit.

13. CH-28 invariant intact: AgentProfile cardinality redesign tests pass. Run:
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
    modules/crates/cli/src/commands/agent.rs, docs/**/*.md} are touched.

PASS/FAIL each. ≤ 600 words total.
```

**Audit pass criteria**:
- Any new drift discovered by the audit → its own drift file created BEFORE chunk seals.
- Any audit-flagged concept contradiction → either fixed in-chunk, renegotiated with user approval, or converted to a drift file with explicit future-chunk assignment.
- Chunk seal is blocked until audit returns clean on all 14 claims (with `NOT-EXECUTED-IN-AUDIT` claims closed at orchestrator gate-4 MUST-RUN).

---

## §12 — Verification section (end-to-end recipe)

Concrete commands a reviewer can run to replay the chunk's close verification:

```bash
# 1. CI guards
bash /root/projects/phi/baby-phi/scripts/check-doc-links.sh
bash /root/projects/phi/baby-phi/scripts/check-ops-doc-headers.sh
bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh
bash /root/projects/phi/baby-phi/scripts/check-spec-drift.sh

# 2. Workspace health
/root/rust-env/cargo/bin/cargo fmt --all --manifest-path /root/projects/phi/baby-phi/Cargo.toml -- --check
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml --workspace --all-targets
/root/rust-env/cargo/bin/cargo test --workspace --no-fail-fast -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml
# expect: 1599 passed / 0 failed

# 3. Cargo cleanup per chunk-implementer v8 placement-1
/root/rust-env/cargo/bin/cargo clean --manifest-path /root/projects/phi/baby-phi/Cargo.toml

# 4. Chunk-specific greps (per plan §3 positive close-audit greps)
# 4a. Cargo.toml workspace-deps row
grep -nE '^phi-core = "0.8"$' /root/projects/phi/baby-phi/Cargo.toml
# expect: 1 hit at line 17

# 4b. AgentLoopConfig carrier-fix
grep -nB1 'revert_pending: None' /root/projects/phi/baby-phi/modules/crates/server/src/platform/sessions/launch.rs
# expect: 1 hit; preceding line should be `response_format: ...`

# 4c. Explicit match-arm
grep -nE 'AgentEvent::RevertApplied' /root/projects/phi/baby-phi/modules/crates/cli/src/commands/agent.rs
# expect: ≥ 1 hit

# 4d. phi-core import baseline preserved
grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l
# expect: 57

# 5. Forbidden-duplication greps (per plan §3)
grep -rn "^pub struct AgentLoopConfig" /root/projects/phi/baby-phi/modules/crates/
# expect: 0 hits
grep -rn "^pub enum AgentEvent" /root/projects/phi/baby-phi/modules/crates/
# expect: 0 hits
grep -rn "^pub struct LlmMessage" /root/projects/phi/baby-phi/modules/crates/
# expect: 0 hits

# 6. Drift file presence
ls /root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/drifts/D-PHICORE-08-FOLLOWUP-01-composition-i-adoption.md
# expect: file exists

# 7. ADR file presence + Accepted status
head -10 /root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/decisions/0064-phi-core-08-absorption.md
# expect: Status: Accepted, sub-decisions §D64.1–§D64.6 referenced

# 8. feature-inventory.md §3 row presence
grep -nE '### D-PHICORE-08-FOLLOWUP-01' /root/projects/phi/baby-phi/docs/specs/v0/feature-inventory.md
# expect: ≥ 1 hit

# 9. Cycle-index row presence
grep -n 'd5b776ac' /root/projects/phi/baby-phi/docs/specs/plan/build/_cycle-index.md
# expect: ≥ 1 hit at active-cycles tail

# 10. CH-25 P-SEAL invariant intact
grep -n 'response_format: phi_core::provider::traits::ResponseFormat::default' /root/projects/phi/baby-phi/modules/crates/server/src/platform/sessions/launch.rs
# expect: 1 hit at line 567

# 11. CH-28 invariant intact
/root/rust-env/cargo/bin/cargo test --workspace --test acceptance_m6_agent_profile_cardinality --manifest-path /root/projects/phi/baby-phi/Cargo.toml -j 4
# expect: 7 tests passed

# 12. Drift-file status (NEW drift added at chunk-seal)
grep -l "Status.*discovered" /root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/drifts/D-PHICORE-08-FOLLOWUP-01*.md | wc -l
# expect: 1 (the NEW drift is at Status: discovered, NOT remediated)
```

---

## Direct-approval criteria checklist (per chunk-initiate skill)

| Criterion | Status | Evidence |
|---|---|---|
| No locked forks at plan-time | **PASS** | 3 forks all RESOLVED at plan-mode session 2026-05-23 (F1 user-locked + F2 + F3 planner-rec rubber-stamp expected at gate-1); no architectural ambiguity |
| Scope ≤ 1.5× forward-scope row deliverables | **PASS** | 7 deliverables (D1–D7) match forward-scope row §1 lines 56–67 enumeration exactly; no scope expansion |
| Zero phi-core leverage delta | **PASS** | §3 predicts Δ +0 leverage-sites / Δ +0 raw `use phi_core` lines; baseline 57 preserved |
| No new K8s blocker class | **PASS** | §3.B 7-axis table all rows "no impact"; K8s-neutral conclusion |
| Audit envelope ≤ medium | **PASS** | SMALL (1 auditor letter A) per §11 sizing override rationale |
| Confidence ≥ 9/10 | **PASS** | §10 implementation confidence target 9/10; documentation confidence target 4/4 = 100%; composite 90% |
| No new migration | **PASS** | §3.F SCHEMAFULL checklist N/A; zero SurrealDB schema changes |

All 7 Direct-approval criteria hold. Orchestrator expected to auto-approve via ExitPlanMode at gate-1 (with optional AskUserQuestion ratification on F2 + F3 if user prefers explicit lock confirmation).

---

## End of plan
