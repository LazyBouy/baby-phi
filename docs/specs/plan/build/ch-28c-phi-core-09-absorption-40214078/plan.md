<!-- Last verified: 2026-05-24 by Claude Code (CH-28c chunk-planner iter-1 v28 — TECHNICAL-PREREQUISITE; absorbs phi-core 0.9.0 breaking-change release; 5 phases P1+P2+P3+P-SEAL (P3 absorbs ADR + drift + feature-inventory + verified-header amends); SMALL audit envelope (1 auditor letter A) per §11 sizing override rationale paralleling CH-28b; zero phi-core leverage delta; K8s-neutral on all 7 axes; predicted ZERO compile breaks per pre-spawn verification + diagnostic; cycle hex `40214078`.) -->

# CH-28c — phi-core 0.9.0 absorption (per-turn debug capture + async-trait migration; baseline shift)

**Forward-scope row**: [`docs/specs/plan/forward-scope/m6-forward-scope-8b7a8bcd.md`](../../forward-scope/m6-forward-scope-8b7a8bcd.md) §1 lines 69–79 + §5 table line 336.

**Cycle hex**: `40214078`.

**Chunk-type**: TECHNICAL-PREREQUISITE.

**Severity**: LOW. **Effort**: 0.3–0.5 ed.

**Prerequisite**: CH-28b (cycle `d5b776ac`, closed 2026-05-23 commit `01e59f2`).

---

## Forks for orchestrator

> Cross-cycle divergence callout (chunk-planner v12; updated 2026-05-23 per CH-28b retro tracker): cumulative cross-cycle divergent forks for baby-phi is **14-of-19 (~74%)** at gate-1 fork-locks; the modal outcome is divergence on tighter / more-fragmented / more-defensive options. CH-28b held the count steady at 14-of-19 (0-of-3 divergent: F1 RESOLVED at prerequisite commit, F2 + F3 planner-rec rubber-stamped). CH-28c expects the same shape: ALL THREE forks below resolve to planner-rec at gate-1 (F1.b RESOLVED at prerequisite commit `b0ed0bb` per chunk-initiate Option A; F2.a + F3.a planner-rec rubber-stamps at gate-1.5). Treat each fork as a confirmation prompt, not an open architectural decision.

### F1 — Chunk numbering convention (TECHNICAL FORK — no user-visible delta — pick on engineering merit only)

| Option | Pros | Cons | Status |
|---|---|---|---|
| F1.a CH-29 renumber | • Linear naming preserves single-digit-suffix sortability | • ~30 doc-surface edits across forward-scope rows + cycle-graph references + concept-doc cross-refs cascading from CH-29..CH-38 → CH-30..CH-39 | NOT chosen |
| **F1.b CH-28c interstitial (planner-rec)** | • ~5 doc-surface edits at the prerequisite commit; surgical scope • Continues the CH-NNa/b suffix precedent established at CH-28b for "interstitial chunks that absorb upstream breaking-changes without renumbering downstream" • Aligns with the second consecutive phi-core breaking-change-absorption arriving one day after CH-28b | • Suffix depth advances from CH-NNa → CH-NNc (still alphabetic; no novelty cost beyond CH-28b's first-of-kind) | **LOCKED RESOLVED at prerequisite commit `b0ed0bb` (per chunk-initiate Option A; user chose F1.b at plan-mode session 2026-05-24)** |
| F1.c CH-28b.5 dot-suffix | • Visually surfaces "between CH-28b and CH-29" • ~5 edits | • Two coexisting interstitial conventions (CH-NNa/b + CH-NN.X) muddies cycle-index lookup | NOT chosen |

### F2 — Cargo dep-string form (TECHNICAL FORK — no user-visible delta — pick on engineering merit only)

| Option | Pros | Cons | Status |
|---|---|---|---|
| **F2.a `phi-core = "0.9"` semver-range (planner-rec)** | • Cargo idiom for "trust semver-compatible updates"; future 0.9.x patches flow into baby-phi automatically on `cargo update` without per-patch Cargo.toml commit • Cargo.lock provides reproducibility within a given commit • Matches the form ratified at CH-28b §D64.1 (`phi-core = "0.8"`) — single-form-precedent across consecutive absorption chunks | • Requires team trust in phi-core's semver discipline (mitigated by phi-core being a sibling project under the same workspace) | **LOCKED planner-rec rubber-stamp expected at gate-1.5** |
| F2.b `phi-core = "0.9.0"` exact-pin | • Most conservative; explicit floor + ceiling at a single patch | • Cargo.toml commit required per 0.9.x patch upgrade; high churn for a release line that may patch frequently | NOT chosen |
| F2.c `phi-core = "^0.9.0"` semver-explicit | • Equivalent resolution to F2.a but with explicit caret | • Idiomatic Cargo style avoids the caret when the version-string would convey it implicitly; visual noise | NOT chosen |

### F3 — Drift file location (BOUNDED USER-VISIBLE — pick on discovery + workflow merit)

| Option | User-visible (what the user perceives) | Pros | Cons + Product trajectory | Status |
|---|---|---|---|---|
| **F3.a NEW drift at canonical `m6/drifts/` (planner-rec)** | The deferred per-turn debug capture adoption surfaces in the `m6/drifts/` listing alongside CH-28b's `D-PHICORE-08-FOLLOWUP-01-composition-i-adoption.md` — discoverable via the existing M6 drifts cross-reference convention. | **User-visible:** Two consecutive phi-core feature deferrals (Composition I + per-turn debug capture) sit side-by-side in `m6/drifts/`, signalling the M6 milestone's deferral pattern at a single discoverable location. + Canonical drift-lifecycle discipline; rich blocked-by / closing-chunk / discovery-source metadata + feature-inventory.md §3 row gives the v0-vs-final user-visible-state translation + per chunk-planner v13 non-terminal-drift rule the placeholder `M7+-FUTURE-PER-TURN-DEBUG-CAPTURE-ADOPTION` satisfies the explicit-named-allocation requirement | • One small new file added (~150 LOC drift body) <br><br>**Product trajectory:** Future readers (M7+ observability planners) navigate to `m6/drifts/` to find ALL deferred phi-core feature adoptions in one place; the catalogue posture pays off as adoption pressure accumulates. | **LOCKED planner-rec rubber-stamp expected at gate-1.5** |
| F3.b Embed in feature-inventory.md only | The deferred adoption surfaces only in feature-inventory.md; no separate drift file. | • Saves ~150 LOC of new-file overhead | • Loses the drift-lifecycle metadata (blocked-by / closing-chunk / discovery-source / phase-of-origin); future closure becomes harder to track <br><br>**Product trajectory:** No catalogue of deferred phi-core features at `m6/drifts/`; future observability planners hunt for it across feature-inventory + ADRs + forward-scope rows. | NOT chosen |

---

## §1 — Locked fork details (per chunk-initiate Phase 1.5 Step A ALWAYS-FIRE + chunk-planner v23 P-plan-3 + v28 §1-position codification)

#### F1 = F1.b — CH-28c interstitial chunk numbering

**Code-level binding**: the cycle is named `CH-28c`; the cycle folder is `ch-28c-phi-core-09-absorption-40214078/`; the forward-scope row was inserted as a NEW third row of the Foundation tier between CH-28b and CH-29 at prerequisite commit `b0ed0bb` 2026-05-24 (BEFORE chunk-planner spawn per chunk-initiate Option A). NO renumbering of downstream cycles CH-29..CH-38.

**Rationale**: Continues the CH-NNa/b suffix precedent established by CH-28b — interstitial chunks that absorb upstream breaking-changes between an existing CH-NN cycle and its downstream consumers, without renumbering the downstream chain. CH-28b shipped the first-of-kind suffix for this pattern; CH-28c is the second consecutive interstitial absorption (phi-core 0.9.0 arrived one day after 0.8.0), validating the precedent's durability under repeated activation.

**Defers**: no other lock implications. Future phi-core breaking-change releases during the M6 build will likely continue the CH-NNc/d/e suffix lineage rather than renumber.

#### F2 = F2.a — `phi-core = "0.9"` semver-range form

**Code-level binding**: workspace `Cargo.toml:17` flips from `phi-core = "0.8"` (CH-28b close baseline) to `phi-core = "0.9"`. Cargo.lock regenerates via `cargo update -p phi-core` against the published 0.9.0 patch (or higher 0.9.x if patches ship between now and chunk-open).

**Rationale**: matches the form ratified at CH-28b §D64.1 — single-form-precedent across consecutive absorption chunks. Cargo's semver resolution treats `"0.9"` as `"^0.9.0"`, so future 0.9.x patches flow into baby-phi automatically without per-patch Cargo.toml commits. Cargo.lock pinning provides reproducibility within a given commit.

**Defers**: nothing — the dep-string-form decision is fully resolved at this chunk.

#### F3 = F3.a — Drift file at canonical `m6/drifts/`

**Code-level binding**: NEW file `/root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/drifts/D-PHICORE-09-FOLLOWUP-01-per-turn-debug-capture-adoption.md` mirrors the structural shape of CH-28b's `D-PHICORE-08-FOLLOWUP-01-composition-i-adoption.md` (verified-header line 1 + ## Identification + ## Concept alignment + ## Plan vs reality + ## Where visible in code + ## Remediation scope sections). feature-inventory.md §3 Deferred catalogue gains a row mirroring the existing `D-PHICORE-08-FOLLOWUP-01` row at lines 125–130.

**Rationale**: maintains canonical drift-lifecycle discipline (rich blocked-by / closing-chunk / discovery-source / phase-of-origin metadata) + co-locates two consecutive phi-core feature deferrals (Composition I + per-turn debug capture) at a single discoverable location for future M7+ observability planners. Per chunk-planner v13 non-terminal-drift rule the placeholder `M7+-FUTURE-PER-TURN-DEBUG-CAPTURE-ADOPTION` satisfies the explicit-named-allocation requirement (NOT `TBD`).

**Defers**: the per-turn debug capture adoption itself (the load-bearing feature work — flipping `SessionRecorderConfig::capture_turn_requests: true` + surfacing `Turn::request_payload` in operator views + JSON-size-implications documentation) to a future M7+ FUNCTIONAL chunk via this drift. The async-trait migration carries no adoption deferral — baby-phi has zero consumers of the 11 async-migrating Fn types + zero `BlockCompactionStrategy` / `InputFilter` impls, so async-fication is a no-op consumer-side with no follow-on work needed.

---

## §2 — Concept alignment walk

| Concept doc | Line range | Claim | Status at chunk open | Target status at chunk close |
|---|---|---|---|---|
| `phi-core/CHANGELOG.md` §[0.9.0] | lines 21–186 | Release notes enumerate 2 surfaces: per-turn debug capture (new `AgentEvent::TurnRequest` variant + `BlockProvenance` / `ProvenanceRole` / `AnnotatedRequestPayload` types + `SessionRecorderConfig::capture_turn_requests: bool` default `false` + `Turn::request_payload` field) + async-trait migration (`BlockCompactionStrategy` + 9 of 11 `AgentLoopConfig` lifecycle Fns + `InputFilter::filter()` become async; tool-update hooks stay sync) | `not-yet-absorbed` (baby-phi pinned at 0.8.x) | `honored` (baby-phi compiles on 0.9.x; explicit `TurnRequest` arm added at cli/agent.rs; deferred adoption tracked via `D-PHICORE-09-FOLLOWUP-01`) |
| `phi-core/docs/concepts/debugging.md` | (new doc at 0.9.0) | Design source-of-truth for per-turn debug capture: opt-in flow via `SessionRecorderConfig::capture_turn_requests`, JSON-size implications, debugging recipe for reconstructing wire payloads | `concept-aspirational` (baby-phi does NOT enable capture at CH-28c) | `concept-aspirational-preserved` (CH-28c absorbs the type surface but does not enable; adoption deferred to M7+ via `D-PHICORE-09-FOLLOWUP-01`) |
| `i-phi/docs/v0/proposal/plan/build/phi-core-0.9.0/plan.md` §"baby-phi absorption" | lines 267–273 | Empirical diagnostic for baby-phi: no `BlockCompactionStrategy` impls; exhaustive `AgentEvent` matches already wildcard-covered; no lifecycle-Fn closures at construction sites → estimated 0.5–1 ed | `predicted-low-impact` | `confirmed-zero-impact` (pre-spawn verification confirmed even smaller surface than predicted — see §3) |

**Concept-doc precedence note**: per chunk-planner v8 §3.D Forward-scope-vs-concept-doc precedence — the forward-scope row at `m6-forward-scope-8b7a8bcd.md` lines 69–79 references three external concept-doc surfaces (the phi-core CHANGELOG + debugging.md + i-phi diagnostic). All three are external to baby-phi's own concept-doc tree; the absorption involves no baby-phi-internal concept-doc contradictions. The deferred adoption (per-turn debug capture flipping `capture_turn_requests: true`) intersects future baby-phi observability concept-doc surface, tracked via `D-PHICORE-09-FOLLOWUP-01`.

---

## §2.5 — Functional outcome (mandatory; added 2026-05-20 per CH-28 retro plan archive `chunk-decomposition-and-fork-framing-76e04080.md`)

**Functional outcome at chunk close**: **NONE**. CH-28c is a TECHNICAL-PREREQUISITE chunk — technical prerequisite for ALL downstream M6 chunks CH-29..CH-38. Shifts baby-phi's phi-core dependency baseline from 0.8 to 0.9.x to absorb the phi-core 0.9.0 release shipped 2026-05-24.

**User-visible behavior at chunk close**: **zero delta from CH-28b close**. Agents continue to operate on the pre-0.9.0 posture; per-turn debug capture is opt-in via `SessionRecorderConfig::capture_turn_requests` (default `false`) which baby-phi does NOT enable this chunk; async-trait migration is internal-only to phi-core's trait shapes + has no impact on baby-phi consumers (zero lifecycle hooks wired; zero `BlockCompactionStrategy` impls; zero `InputFilter` impls per pre-spawn verification).

**Non-technical-user rationale**: *"phi-core released another minor version one day after 0.8.0 with two added capabilities — per-turn debug capture (currently dormant; opt-in) and an async upgrade to the agent loop's internal hook points (currently unused by baby-phi). We absorb the version baseline shift today so the rest of M6 runs on 0.9.x + future patches flow through automatically. Even smaller surface than CH-28b: literally ZERO compile breaks."*

**Defers (with product impact)**:
- **Per-turn debug capture adoption** → future M7+ observability chunk (no specific CH-NN slot reserved at CH-28c close; tracked via `D-PHICORE-09-FOLLOWUP-01`). **User-visible impact while deferred**: agents' fully-assembled LLM request payloads (system prompt + post-`convert_to_llm()` `Vec<Message>` array + tool definitions + parallel-indexed `BlockProvenance` vec) are NOT persisted to disk; debugging an unexpected LLM response requires reconstructing the input context manually from `Turn.input_messages` + ad-hoc re-application of `convert_to_llm()`. After future per-turn debug capture adoption: the exact wire payload is in `Turn.request_payload` for every captured turn (opt-in per agent / per cycle to keep session JSON size bounded).
- **Async-trait migration adoption deferral**: N/A — baby-phi never used the 9 lifecycle hooks at sync, so async-fication is a no-op consumer-side; no migration work needed at adoption time either. The drift `D-PHICORE-09-FOLLOWUP-01` covers ONLY the per-turn debug capture deferral, not async-trait migration.

---

## §3 — phi-core leverage map

**Predicted import delta**: **Δ +0 leverage-sites / Δ +0 raw `use phi_core` lines**. Baseline `grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l` = **57** (verified at chunk-planning time; matches CH-28b chunk-close baseline preserved). Predicted at chunk close: **57** (unchanged).

**Methodology** (chunk-planner v9 leverage-sites-not-import-lines per CH-03-i-phi retro P4): Per-turn debug capture types (`BlockProvenance`, `ProvenanceRole`, `AnnotatedRequestPayload`, `AgentEvent::TurnRequest`) become available in baby-phi's import space but ARE NOT imported until the future adoption chunk fires. Async-trait migration of `BlockCompactionStrategy` + 9 lifecycle Fns + `InputFilter` is internal-only to phi-core's trait shapes; baby-phi has no consumers of any of these surfaces (verified via grep below) so async-fication is a no-op consumer-side.

### Positive close-audit greps (these should fire green at chunk close)

1. **Workspace `phi_core` import count preserved**:
   ```bash
   grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l
   ```
   Expected at close: **57** (Δ +0).

2. **Cargo.toml workspace-deps row reads `phi-core = "0.9"`**:
   ```bash
   grep -nE '^phi-core = "0.9"$' /root/projects/phi/baby-phi/Cargo.toml
   ```
   Expected at close: 1 hit at line 17.

3. **Cargo.lock pin against phi-core 0.9.x**:
   ```bash
   grep -nA 1 '^name = "phi-core"' /root/projects/phi/baby-phi/Cargo.lock | grep '^version'
   ```
   Expected at close: `version = "0.9.0"` (or higher 0.9.x if a patch ships during chunk).

4. **Explicit `TurnRequest` arm at cli/agent.rs**:
   ```bash
   grep -nE 'AgentEvent::TurnRequest' /root/projects/phi/baby-phi/modules/crates/cli/src/commands/agent.rs
   ```
   Expected at close: ≥ 1 hit at a line between the existing `RevertApplied` arm (line 674) and the existing `_ => {}` wildcard (line 675).

5. **CH-28b carrier-fix invariant intact**:
   ```bash
   grep -n 'revert_pending: None' /root/projects/phi/baby-phi/modules/crates/server/src/platform/sessions/launch.rs
   ```
   Expected at close: 1 hit (preserved from CH-28b).

### Forbidden-duplication greps (these MUST be 0 at chunk close)

| Grep | Expected hits | Rationale |
|---|---|---|
| `grep -rn "^pub enum AgentEvent" /root/projects/phi/baby-phi/modules/crates/` | **0** | Re-defining `AgentEvent` in baby-phi shadows the phi-core variant + breaks the cli/agent.rs match-arm cover |
| `grep -rn "^pub struct AnnotatedRequestPayload" /root/projects/phi/baby-phi/modules/crates/` | **0** | Re-defining the new 0.9.0 payload type duplicates phi-core |
| `grep -rn "^pub enum BlockProvenance" /root/projects/phi/baby-phi/modules/crates/` | **0** | Same — duplicates phi-core's new 0.9.0 enum |
| `grep -rn "^pub enum ProvenanceRole" /root/projects/phi/baby-phi/modules/crates/` | **0** | Same |
| `grep -rn "^pub struct LlmMessage" /root/projects/phi/baby-phi/modules/crates/` | **0** | Verified pre-spawn: baby-phi only uses `LlmMessage::new(...)` constructor (2 sites at `session_recorder.rs:483` + `launch.rs:578`); no struct-literals to break under the new `provenance_hint` field |
| `grep -rn "^pub struct AgentLoopConfig" /root/projects/phi/baby-phi/modules/crates/` | **0** | Re-defining shadows phi-core; the carrier-fix path through phi-core's struct is preserved |
| `grep -rn "impl phi_core::context::BlockCompactionStrategy for" /root/projects/phi/baby-phi/modules/crates/` | **0** | No custom impls; async-trait migration is a no-op consumer-side |
| `grep -rn "impl phi_core::.*::InputFilter for" /root/projects/phi/baby-phi/modules/crates/` | **0** | No custom impls; async-trait migration is a no-op consumer-side |

### `check-phi-core-reuse.sh` enforcement

```bash
bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh
```

Expected at close: exit 0. The script enforces the rules above; if any forbidden duplication slips in, the CI guard catches it.

### Pre-spawn verification confirming ZERO compile breaks

| Class | Pre-spawn evidence |
|---|---|
| `AgentEvent::TurnRequest` new variant | ZERO breaks. `domain/src/session_recorder.rs:166-170` uses field-wildcard `..` in the `matches!()` pattern → covers any new variants without modification. `cli/src/commands/agent.rs:642-676` ends with `_ => {}` at line 675 → covers; explicit `RevertApplied { .. } => {}` arm at line 674 from CH-28b is cosmetic precedent. |
| `LlmMessage.provenance_hint` new field | ZERO breaks. Pre-spawn `grep` confirmed baby-phi uses `LlmMessage::new(Message::user(...))` at both call sites (`session_recorder.rs:483` + `launch.rs:578`); no struct-literal `LlmMessage { ... }` patterns. |
| `BlockCompactionStrategy` → `#[async_trait]` | ZERO breaks. Pre-spawn `grep -rn "impl.*BlockCompactionStrategy for" modules/crates/` returned 0 hits. baby-phi has NO custom impls. |
| 9 of 11 `AgentLoopConfig` lifecycle Fns become async | ZERO breaks. Pre-spawn read of `server/src/platform/sessions/launch.rs:542-552` confirmed all 9 hook fields (`before_loop` / `after_loop` / `before_turn` / `after_turn` / `on_error` / `before_tool_execution` / `after_tool_execution` / `before_compaction_start` / `after_compaction_end`) are set to `None`. No closure construction sites exist. |
| `InputFilter::filter()` → `async fn` | ZERO breaks. Pre-spawn `grep -rn "impl.*InputFilter for" modules/crates/` returned 0 hits. baby-phi has NO custom impls. |
| `compact_session_loops` / `BasicAgent::compact_context*` → async | ZERO breaks. Pre-spawn `grep` confirmed no direct calls in baby-phi src tree. |

---

## §3.B — K8s microservice readiness check (7-axis evaluation)

| Axis | Classification | Code anchor / Rationale |
|---|---|---|
| **A1 — In-process state** | **no impact** | No mutexes / RwLocks / in-memory caches added or removed. CH-28b's CH-25 `response_format: ResponseFormat::default()` + the new `revert_pending: None` field stay unchanged; CH-28c does NOT touch the `AgentLoopConfig` struct-literal at `launch.rs:526-568`. |
| **A2 — IPC channels** | **no impact** | No mpsc/broadcast/watch/oneshot channels added or removed. The cli/agent.rs match-arm addition is a no-op body `=> {}` in the existing `rx.recv()` event-consumption loop. |
| **A3 — Pod-local resources** | **no impact** | No file handles, sockets, or embedded SurrealDB files added. No filesystem operations. |
| **A4 — Migration runner conformance** | **no impact** | ZERO new migrations. Cargo.lock regeneration via `cargo update -p phi-core` is build-tool state, not runtime migration. |
| **A5 — Trait-shape requirement** | **no impact** | No Repository / Storage trait additions. The phi-core trait additions (`BlockCompactionStrategy` async + `InputFilter::filter` async + 9 lifecycle Fns async) are upstream-internal; baby-phi has zero impls so no trait-shape conformance work. |
| **A6 — Cross-pod state sharing** | **no impact** | No read-after-write expectations changed. The per-turn debug capture surface (when adopted in a future M7+ chunk) WILL have cross-pod state symmetry implications (each pod's `Turn::request_payload` is local), but adoption is deferred. |
| **A7 — Audit hash-chain symmetry** | **no impact** | No `domain::audit::AuditEvent` writes added or removed. The new `phi_core::AgentEvent::TurnRequest` variant routes to phi-core's `SessionRecorder` (not baby-phi's hash-chain `AuditEvent` writer); no symmetry impact. |

**Verdict**: ✅ all 7 axes resolved at "no impact". **Zero new K8s blocker class.** No `CHK8S-D-NN` ledger entry needed.

---

## §3.C — User-facing documentation impact map (3-tier evaluation)

| Tier | Doc(s) touched | Action | Defer rationale (if applicable) |
|---|---|---|---|
| Architecture | NONE | None this chunk | The phi-core CHANGELOG + debugging.md are upstream concept-doc-of-record; no baby-phi-internal architecture-tier doc requires update for the baseline shift itself. |
| Operations | NONE | None this chunk | No operator-facing flow changes; per-turn debug capture is opt-in + deferred. |
| User-guide | NONE | None this chunk | No end-user-facing surface change. |

**Governance-tier docs** (NOT user-facing per §3.C scope): NEW drift file + ADR-0065 + feature-inventory.md row are first-class deliverables (see §4 + §5 + §7).

---

## §3.D — Forward-scope-vs-concept-doc precedence (added 2026-05-08 per CH-15 retro Row 4, cycle hex `c3f46f17`)

**Forward-scope row literal terms reviewed** against external concept-doc canonical phrasing:

| Term | Source | Verdict |
|---|---|---|
| `phi-core = "0.8"` → `phi-core = "0.9"` | forward-scope §1 line 76 + phi-core CHANGELOG §[0.9.0] (release 2026-05-24) | ✅ Aligns — phi-core 0.9.0 is published; the bump is the canonical absorption form |
| `AgentEvent::TurnRequest { .. } => {}` arm at `cli/src/commands/agent.rs:~677` | forward-scope §1 line 76 + phi-core CHANGELOG §[0.9.0] §"Added" lines 79–84 | ✅ Aligns — verified at plan-draft that line 675 = `_ => {}` wildcard at chunk-open; the new arm goes between the existing `RevertApplied` arm (674) and the wildcard, landing at line 675 (the existing wildcard then shifts to 676). Note: forward-scope said "~677" which was an approximation; actual is line 675 post-insertion. |
| `D-PHICORE-09-FOLLOWUP-01` drift naming convention | forward-scope §1 line 76 | ✅ Aligns — parallels existing `D-PHICORE-08-FOLLOWUP-01` from CH-28b |
| `m6/drifts/` canonical path | forward-scope §1 line 76 + CH-28b precedent | ✅ Aligns — verified via `ls /root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/drifts/` returns 2 existing files |

**No contradictions detected.** Auto-approval blocker (chunk-planner v8 §3.D) does NOT fire.

---

## §3.E — Anticipated gate-2.5 candidates (added 2026-05-11 per CH-24 retro Row 6, cycle hex `5778bb77`; chunk-planner v13)

| Candidate | Surface | Recommendation if surfaced |
|---|---|---|
| **session_recorder.rs `matches!()` site explicit-arm omission** | `domain/src/session_recorder.rs:166-170` uses `matches!()` with field-wildcard `..` over a single pattern `PhiCoreAgentEvent::AgentStart { session_id, .. }`. Adding an explicit `TurnRequest` arm would either (a) be syntactically impossible (`matches!` doesn't compose multi-pattern usefully here since the predicate is "is it AgentStart for ctx?", not "is it any specific event?"), or (b) change boolean semantics if forced. | Plan §3 closes by NOT modifying this site — same scope-narrow rationale as CH-28b §D64.6. If a future reader proposes an edit during P2 review, ratify at gate-2.5 by re-confirming the §D65.3 scope-narrow rationale (see §5 below). |
| **launch.rs `AgentLoopConfig` struct-literal field reorder** | `server/src/platform/sessions/launch.rs:526-568` carries the existing struct-literal with `response_format: ResponseFormat::default()` (CH-25) + `revert_pending: None` (CH-28b). phi-core 0.9.0 adds NO new fields to `AgentLoopConfig` (per CHANGELOG line 76 *"`AgentLoopConfig` gains no new fields"*) so no new carrier-fix is needed at this site. | If gate-2 source-read surfaces a previously-missed field requiring carrier-fix, ratify at gate-2.5 by widening D3 to a 2-line addition + escalating via AskUserQuestion. Probability LOW given the CHANGELOG explicit statement. |
| **Async-trait dev-dep addition (per chunk-planner v22 P2 proc-macro decorator prediction)** | If any test stub in baby-phi grows to impl `phi_core::context::BlockCompactionStrategy` or `phi_core::types::InputFilter`, it would need `async-trait = "0.1"` in `[dev-dependencies]`. | Pre-spawn `grep` confirmed ZERO such impls. `async-trait = "0.1"` is ALREADY in baby-phi's workspace deps (line 43 of Cargo.toml) per the M5/P4 wave for the daemon's async traits. Predicted P-NONE; if a test stub appears at P2, ratify at gate-2.5. |

---

## §3.F — SurrealDB SCHEMAFULL semantic checklist (per chunk-planner v25 P-plan-1)

**N/A — CH-28c ships zero migrations + zero SurrealDB schema changes.** No `REMOVE FIELD` / `ALTER TABLE` narrowing / new SCHEMAFULL table / narrowing-UNIQUE-index changes. The §3.F checklist does not apply.

---

## §4 — Drifts closed + Deferred functionality

**Drifts closed**: **NONE this chunk.** CH-28c files a NEW drift `D-PHICORE-09-FOLLOWUP-01-per-turn-debug-capture-adoption.md` but closes none.

**NEW drift filed**:

### D-PHICORE-09-FOLLOWUP-01 — Per-turn debug capture adoption (deferred to future M7+ FUNCTIONAL chunk)

- **Phase of origin**: CH-28c P3 chunk-seal (2026-05-24) — filed per F3.a planner-rec lock at gate-1.5 + ADR-0065 §D65.4 deferral decision.
- **Discovery source**: phi-core 0.9.0 breaking-change release (2026-05-24) — per-turn debug capture shipped as opt-in via `SessionRecorderConfig::capture_turn_requests: bool` (default `false`) + new types `BlockProvenance` / `ProvenanceRole` / `AnnotatedRequestPayload` + `Turn::request_payload: Option<AnnotatedRequestPayload>` (with `#[serde(default)]` for back-compat).
- **Status**: `discovered`.
- **Bucket**: B — follow-on engine-scope widening (opt-in observability feature adoption + JSON-size-implications documentation).
- **Severity**: LOW.
- **Tags**: `phi-core-0.9`, `per-turn-debug-capture`, `observability`, `opt-in-feature`, `m7-future`.
- **Blocks**: nothing within CH-28c; the dependency baseline shift + explicit match-arm + zero-impact verification together close the breaking-change absorption.
- **Blocked-by**: nothing — CH-28c ships the dependency baseline shift which unblocks adoption. All adoption prerequisites are inside baby-phi's source tree + operator-tooling surface.
- **Closing chunk**: **M7+-FUTURE-PER-TURN-DEBUG-CAPTURE-ADOPTION** (placeholder; per chunk-planner v13 non-terminal-drift rule the placeholder names an explicit-named-future-allocation; no specific CH-NN slot reserved at CH-28c close).
- **Remediation scope** (estimate only): adoption decomposes into 4 prerequisites — (1) flip `SessionRecorderConfig::capture_turn_requests: true` at the recorder construction site(s) (currently `cli/src/commands/agent.rs:636` uses `SessionRecorderConfig::default()`); (2) decide JSON-size-bounding policy (per-agent flag? per-cycle flag? max-payload-bytes truncation?); (3) operator UI / CLI surface for inspecting `Turn::request_payload` (likely a `phi session show --include-request-payload` flag or similar); (4) acceptance suite exercising round-trip serialization of captured payloads. Aggregate estimate: ~0.8–1.5 ed for dedicated adoption chunk.

**Async-trait migration adoption deferral**: N/A — baby-phi has zero `BlockCompactionStrategy` / `InputFilter` impls + zero `AgentLoopConfig` lifecycle Fn closures (all 9 hooks are `None` at `launch.rs:542-552`). Async-fication is a no-op consumer-side with no follow-on adoption work needed.

---

## §5 — ADRs drafted

**NEW ADR**: `0065-phi-core-09-absorption.md` at `/root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/decisions/`.

**Status at chunk seal**: Proposed at P3 draft; flips to **Accepted** at P-SEAL.

**Authors**: Claude Code (orchestrator + chunk-planner v28 + chunk-implementer v18).

**Canonical ADR sections (per chunk-planner v17 P2 explicit ADR-section enumeration)**:

1. **Forks** — header table; F1 RESOLVED at prerequisite commit + F2 planner-rec + F3 planner-rec form (paralleling CH-28b §"Forks").
2. **Context** — chunk-graph + forward-scope citations + downstream consumer enumeration (CH-29..CH-38) + i-phi diagnostic provenance.
3. **Sub-decisions** — one `### §D65.<M>` per fork resolution + supporting decisions; each ends with a Pre-existing-behaviour preservation note (or never-shipped-yet variant per chunk-planner v11 P1 + v24 P-plan-2 for net-new surfaces).
4. **Cross-references** — 4 categories: (a) concept-doc + line range; (b) closed drifts (none); (c) prior ADRs as precedent — ADR-0064 (CH-28b 0.8.0 absorption — directly-parallel pattern), ADR-0059 §D59.2 (CH-25 P-SEAL `response_format` carrier-fix precedent, mentioned as inherited-pattern even though CH-28c ships zero carrier-fix), ADR-0063 §D63.3 (CH-28 forward-scope-row-as-prerequisite-commit precedent); (d) forward-scope row.
5. **Consequences** — `### For CH-29..CH-38` subsection per downstream consumer enumeration + `### For M7+ observability chunk` subsection forward-routing to `D-PHICORE-09-FOLLOWUP-01` closing chunk.
6. **Revisit triggers** — 4-6 bullets each citing a specific §D65.<M> that would warrant re-opening (e.g., phi-core 0.10.0 release ⟶ revisit §D65.1 dep-string-form; new in-baby-phi `BlockCompactionStrategy` impl wants async semantics ⟶ revisit §D65.5 async-trait-migration-no-op claim).
7. **Verification** — commands the reviewer can run to replay verification (mirrors §12 below).

**Sub-decisions** (5 total per planner outline):

- **§D65.1 — F2.a `phi-core = "0.9"` semver-range form** (chosen over F2.b exact-pin + F2.c semver-explicit). Pre-existing-behaviour preservation note: pre-existing scaffold preserved — workspace `Cargo.toml:17` already used semver-range form `phi-core = "0.8"` since CH-28b close (paralleling 0.7.1 → 0.8 shift); the shift to `"0.9"` continues the single-form-precedent established at CH-28b §D64.1.
- **§D65.2 — F1.b CH-28c interstitial chunk-numbering** (continues CH-28b CH-NNa/b precedent). Pre-existing-behaviour preservation note: pre-existing scaffold preserved — the CH-NNa/b suffix convention was established at CH-28b §D64.2 as canonical for "interstitial chunks that absorb upstream breaking-changes without renumbering downstream cycles"; CH-28c is the second activation of the convention, validating durability.
- **§D65.3 — F3.a explicit `TurnRequest` arm at cli/agent.rs only; session_recorder.rs single-pattern `matches!()` left unchanged** (paralleling CH-28b §D64.6 scope-narrow). Pre-existing-behaviour preservation note: pre-existing scaffold preserved — `session_recorder.rs:166-170` `matches!()` site recognises `PhiCoreAgentEvent::AgentStart { session_id, .. }` ONLY (single-pattern predicate); the explicit-arm pattern from cli/agent.rs does NOT apply because adding it would either be syntactically inert (the `..` field-wildcard already covers any variant) or would change the boolean predicate semantics. CH-28b §D64.6 established the scope-narrow ratchet for this site.
- **§D65.4 — F3.a NEW drift `D-PHICORE-09-FOLLOWUP-01` filed for per-turn debug capture adoption deferral**. Pre-existing-absence preserved (never-shipped-yet variant per chunk-planner v24 P-plan-2): no prior per-turn debug capture surface exists in baby-phi; CH-28c absorbs the type space without enabling the feature. Future adoption chunk (placeholder `M7+-FUTURE-PER-TURN-DEBUG-CAPTURE-ADOPTION`) decides JSON-size-bounding policy + operator UI surface + acceptance test shape.
- **§D65.5 — Async-trait migration is a no-op consumer-side; no adoption deferral needed**. Pre-existing-behaviour preservation note: pre-existing scaffold preserved — baby-phi has ZERO `BlockCompactionStrategy` / `InputFilter` impls + ZERO `AgentLoopConfig` lifecycle Fn closures (verified pre-spawn). phi-core 0.9.0 async-trait migration is internal-only to phi-core trait shapes; no consumer-side migration work required at CH-28c OR any future chunk (the no-op nature does not warrant a drift). If a future chunk adds a custom `BlockCompactionStrategy` or `InputFilter` impl, that chunk MUST use async-trait at impl time per phi-core's 0.9.0 trait shape; revisit §D65.5 if such an impl materializes.

---

## §6 — Prior-chunk regression re-verification

**Carry-forward invariants** (must remain green at chunk close):

| Invariant | Source chunk | Verification command | Expected |
|---|---|---|---|
| Workspace test count | CH-28b close baseline | `cargo test --workspace --no-fail-fast -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml` | **1599 passed / 0 failed** |
| phi-core import baseline | CH-28b close baseline | `grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l` | **57** |
| CH-25 P-SEAL invariant | `launch.rs:567` `response_format: ResponseFormat::default()` | `grep -n 'response_format: phi_core::provider::traits::ResponseFormat::default' modules/crates/server/src/platform/sessions/launch.rs` | ≥ 1 hit |
| CH-28b carrier-fix invariant | `launch.rs:577` `revert_pending: None` | `grep -n 'revert_pending: None' modules/crates/server/src/platform/sessions/launch.rs` | ≥ 1 hit |
| CH-28b explicit-arm invariant | `cli/agent.rs:674` `AgentEvent::RevertApplied { .. } => {}` | `grep -nE 'AgentEvent::RevertApplied' modules/crates/cli/src/commands/agent.rs` | ≥ 1 hit |
| CH-28 acceptance suite | 7 m6 agent profile cardinality tests | `cargo test --workspace --test acceptance_m6_agent_profile_cardinality --manifest-path /root/projects/phi/baby-phi/Cargo.toml -j 4` | 7 passed |
| CI guards | 4 scripts/check-*.sh | `bash /root/projects/phi/baby-phi/scripts/check-{doc-links,ops-doc-headers,phi-core-reuse,spec-drift}.sh` (4 invocations) | all exit 0 |

**Named expected-still-green tests** (carry-forward; per chunk-planner v17 P-plan-3 grep-verify against `tests/` listing — N/A for in-tree unit tests; the workspace test count is the primary regression signal):

- `test_session_recorder_emits_session_started_on_agent_start_for_ctx` (CH-17 acceptance)
- `test_acceptance_m6_agent_profile_cardinality_create_with_template` (CH-28 acceptance)
- `test_acceptance_m6_agent_profile_cardinality_override_upsert` (CH-28 acceptance)
- `test_launch_session_with_default_runtime_config` (CH-25 acceptance)
- All 1599 carry-forward (workspace `cargo test` exit 0 is the comprehensive gate).

---

## §7 — Phases within the chunk

**Phase count**: **5 phases (P1 + P2 + P3 + P-SEAL)** — wait, correction: **4 phases (P1 + P2 + P3 + P-SEAL)**. Per the per-chunk-template the P-SEAL row IS a phase. Total **4 phases**. (This is 1 fewer than CH-28b's 5 phases because CH-28c collapses CH-28b's P3 + P4 [Cargo-bump + paperwork as separate phases] into one P3 paperwork-only phase — CH-28c has ZERO additional carrier-fix work needed at any code site beyond the cli/agent.rs match-arm at P2, since `AgentLoopConfig` gains no new fields per CHANGELOG line 76.)

### P1 — Cargo bump + lock update + workspace-build verification

- **Goal**: shift workspace baseline from `phi-core = "0.8"` to `phi-core = "0.9"`; regenerate Cargo.lock against published phi-core 0.9.x; verify workspace builds GREEN on the new baseline.
- **Deliverables**:
  1. Edit `/root/projects/phi/baby-phi/Cargo.toml` line 17: `phi-core = "0.8"` → `phi-core = "0.9"`.
  2. Run `/root/rust-env/cargo/bin/cargo update -p phi-core --manifest-path /root/projects/phi/baby-phi/Cargo.toml` — regenerates Cargo.lock entries for phi-core (and any transitive dep version changes phi-core 0.9.0 introduces).
  3. Run `/root/rust-env/cargo/bin/cargo build --workspace -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml` — expect GREEN exit 0 with ZERO compile errors. Per pre-spawn verification §3, the predicted compile-error count is ZERO across all 3 breaking-change classes (AgentEvent / LlmMessage / async-trait migration).
- **Tests**: 0 new tests; existing workspace build via cargo build.
- **Concept-alignment check**: §2 row `phi-core/CHANGELOG.md §[0.9.0]` advances `not-yet-absorbed` → `partially-honored` (baseline shifted; pending P2 cosmetic arm).
- **phi-core leverage check**: baseline grep returns 57 (preserved).
- **User-facing doc updates**: N/A.
- **Confidence target**: ≥ 99% (mechanical baseline shift; the surface is fully characterized by pre-spawn verification).
- **Pause discipline**: PAUSE via AskUserQuestion IF ANY compile error appears (would indicate pre-spawn verification missed a surface — e.g., phi-core 0.9.0 shipped an unannounced breaking change OR a transitive crate also bumped). PAUSE IF `cargo update` brings additional unexpected dep version churn (e.g., a security advisory triggers a tokio bump).

### P2 — Explicit `TurnRequest` match-arm at cli/agent.rs

- **Goal**: add the explicit `AgentEvent::TurnRequest { .. } => {}` arm at `cli/src/commands/agent.rs` between the existing `RevertApplied` arm (line 674) and the existing `_ => {}` wildcard (line 675); workspace stays GREEN.
- **Deliverables**:
  1. Edit `/root/projects/phi/baby-phi/modules/crates/cli/src/commands/agent.rs` between lines 674 and 675: insert (with leading comment block per the CH-28b precedent at lines 668-673):
     ```rust
     // CH-28c P2 explicit-arm coverage (per ADR-0065 §D65.3):
     // phi-core 0.9.0's per-turn debug capture emits `TurnRequest` once
     // per turn (before the retry-loop's first provider.stream() call)
     // regardless of recorder configuration. baby-phi does NOT enable
     // capture (SessionRecorderConfig::capture_turn_requests defaults
     // to false; the recorder constructed at line 636 above uses
     // SessionRecorderConfig::default()), so this arm is cosmetic +
     // signals 0.9.0 variant awareness for future readers.
     // Adoption tracked via D-PHICORE-09-FOLLOWUP-01.
     AgentEvent::TurnRequest { .. } => {}
     ```
     The new arm + comment block adds ~9 LOC; the existing `_ => {}` wildcard shifts from line 675 to line ~684 (depending on comment-block line count).
  2. session_recorder.rs `matches!()` site at lines 166–170 is INTENTIONALLY NOT modified (per ADR-0065 §D65.3 + the CH-28b §D64.6 scope-narrow precedent). The `matches!()` block's `..` field-wildcard already covers any new `AgentEvent` variant; adding a `TurnRequest` arm would either be inert (no-op) or change the boolean predicate semantics (it tests "is this AgentStart for ctx?", not "is this any new variant?").
- **Tests**: 0 new tests; full workspace cargo build + cargo test re-verification.
  - `/root/rust-env/cargo/bin/cargo build --workspace -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml`
  - `/root/rust-env/cargo/bin/cargo test --workspace --no-fail-fast -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml` — expect 1599 passed / 0 failed.
- **Concept-alignment check**: §2 row `phi-core/CHANGELOG.md §[0.9.0]` advances `partially-honored` → `honored`.
- **phi-core leverage check**: grep returns 57 (Δ +0 — the new arm uses the already-imported `AgentEvent` from `cli/agent.rs:1` import line).
- **User-facing doc updates**: N/A.
- **Confidence target**: ≥ 99% (mechanical match-arm addition; parallels CH-28b P3 D3 deliverable).
- **Pause discipline**: PAUSE via AskUserQuestion IF cargo build fails (the predicted no-op outcome falsified) OR IF cargo test count diverges from 1599 (regression introduced unexpectedly) OR IF clippy surfaces a warning about the new arm placement.
- **Cargo cleanup**: `/root/rust-env/cargo/bin/cargo clean --manifest-path /root/projects/phi/baby-phi/Cargo.toml` per chunk-implementer v8 placement-1 (immediate-post-test cleanup).

### P3 — Docs + drift + ADR + feature-inventory + verified-header amends

- **Goal**: file NEW drift `D-PHICORE-09-FOLLOWUP-01`; add feature-inventory.md §3 row; draft ADR-0065 (Status: Proposed); amend verified headers on all touched docs.
- **Deliverables**:
  1. Create NEW file `/root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/drifts/D-PHICORE-09-FOLLOWUP-01-per-turn-debug-capture-adoption.md` mirroring the existing `D-PHICORE-08-FOLLOWUP-01-composition-i-adoption.md` shape (verified-header line 1; `## Identification` section with all fields per §4 above; `## Concept alignment` citing phi-core CHANGELOG §[0.9.0] + debugging.md; `## Plan vs reality` documenting CH-28c's deferral routing; `## Where visible in code` documenting the 1 grep regression target — `grep -nE "capture_turn_requests: true" modules/crates/` should return 0 hits at CH-28c close and ≥ 1 hit at adoption-chunk close; `## Remediation scope` documenting the 4 adoption prerequisites per §4 above). Status: `discovered`. Bucket: B. Severity: LOW. Closing chunk: **M7+-FUTURE-PER-TURN-DEBUG-CAPTURE-ADOPTION** placeholder.
  2. Edit `/root/projects/phi/baby-phi/docs/specs/v0/feature-inventory.md` §3 Deferred catalogue: add a new `### D-PHICORE-09-FOLLOWUP-01 — Per-turn debug capture adoption (phi-core 0.9.0 opt-in observability surface)` row IMMEDIATELY AFTER the existing `D-PHICORE-08-FOLLOWUP-01` row (currently at lines 125–130) and BEFORE the `D-CH28-FOLLOWUP-01` row (currently at line 132). 5 fields: Feature impact / User-visible state in v0 / User-visible state at final / Allocation chunk / Cross-chunk dependency. Bump §1 verified header date to 2026-05-24 with CH-28c citation.
  3. Create NEW ADR file `/root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/decisions/0065-phi-core-09-absorption.md` with the 7 canonical sections per §5 above; sub-decisions §D65.1–§D65.5; Status: **Proposed** at P3 (flips to **Accepted** at P-SEAL per chunk-implementer v17 ADR Status flip discipline at P-SEAL).
  4. Verified-header presence check on `/root/projects/phi/baby-phi/docs/specs/plan/forward-scope/m6-forward-scope-8b7a8bcd.md` — the CH-28c forward-scope row was inserted as a prerequisite-commit at `b0ed0bb` 2026-05-24 BEFORE chunk-planner spawn (per F1.b lock + chunk-initiate Option A). Confirm the existing authoring-time verified-header at the file's line 1 carries the CH-28c authoring citation; no body changes (the row is the authoring artifact).
- **Tests**: 0 new code tests. Doc-links check (`bash /root/projects/phi/baby-phi/scripts/check-doc-links.sh`) MUST pass after edits.
- **Concept-alignment check**: §2 rows for `phi-core CHANGELOG §[0.9.0]` + `debugging.md` + `i-phi diagnostic` all reach target status (`honored` for CHANGELOG; `concept-aspirational-preserved` for debugging.md; `confirmed-zero-impact` for i-phi diagnostic).
- **phi-core leverage check**: baseline grep returns 57 (unchanged).
- **User-facing doc updates**: per §3.C deferred (3 tiers all N/A). The NEW drift file + ADR-0065 + feature-inventory row are GOVERNANCE-tier docs (not user-facing-tier per §3.C definition); they ARE first-class deliverables.
- **Confidence target**: ≥ 99% (paperwork pattern matches CH-28b precedent exactly).
- **Pause discipline**: PAUSE via AskUserQuestion IF doc-links check fails OR feature-inventory.md edit cascades unexpectedly (e.g., §3 row count change breaks a §2 cross-ref) OR IF ADR-0065 cross-reference to ADR-0064 fails the `check-doc-links.sh` script (relative-path resolution).

### P-SEAL — Verified-header sweep + cycle-index Status flip + ADR-0065 Accepted flip

- **Goal**: chunk-seal paperwork; cycle-index row appended at active-cycles tail with Status `ready-for-audit` (left as `in-flight` per chunk-planner v16 P-SEAL canonical lifecycle — orchestrator owns transitions); ADR flipped Proposed → Accepted; verified-header amends on all touched docs.
- **Deliverables**:
  1. Append cycle-index row at `/root/projects/phi/baby-phi/docs/specs/plan/build/_cycle-index.md` (per chunk-implementer P-SEAL v17 + chunk-archive-plan skill v3):
     - Hex link: `[\`40214078\`](ch-28c-phi-core-09-absorption-40214078/plan.md)`
     - Slug + summary: `CH-28c — phi-core 0.9.0 absorption (per-turn debug capture + async-trait migration; baseline shift; TECHNICAL-PREREQUISITE); 4 phases (P1+P2+P3+P-SEAL); 0 carrier-fixes [AgentLoopConfig gains no new fields per CHANGELOG line 76] + 1 explicit match-arm at cli/agent.rs between line 674 RevertApplied arm and the existing wildcard; NEW drift D-PHICORE-09-FOLLOWUP-01 filed; ADR-0065 Accepted with 5 sub-decisions §D65.1–§D65.5; phi-core import baseline preserved at 57; workspace test count preserved at 1599`
     - Phase count: `4`
     - Auditor count: `1 (audit envelope: SMALL per §11)`
     - Iterations: `pending` (per chunk-planner v16 P-SEAL canonical lifecycle — leave Iterations = pending and Status = in-flight; orchestrator owns the transitions per `_cycle-index.md` row-lifecycle paragraph: gate-3 → ready-for-audit; gate-4 close → audited-pending-retro; Phase 6 / Phase 7 close → retro-complete + Iterations to final count)
     - Status: `in-flight` (per chunk-planner v16 P-SEAL canonical lifecycle — orchestrator transitions at gate-3/gate-4/retro-complete)
     - Test count: `1599`
  2. Flip ADR-0065 status header: `Proposed` → `Accepted` at top of `0065-phi-core-09-absorption.md`; bump verified header on the ADR file.
  3. Amend verified headers on: `_cycle-index.md` (row 1 + row 2 comment lines), `feature-inventory.md`, NEW `D-PHICORE-09-FOLLOWUP-01-per-turn-debug-capture-adoption.md`. Forward-scope file already prerequisite-committed at `b0ed0bb`; no header amend needed unless its line-1 comment requires refresh.
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

**Expected total test count at chunk close**: **1599** (baseline from CH-28b close; ZERO new tests added at CH-28c — chunk is a baseline-shift + 1 cosmetic match-arm + paperwork; no new behaviour to test).

**Test-count band** (per chunk-planner v17 + v22 P-plan-1 per-Tier breakdown): `[1599, 1599]` (point estimate; no buffer needed — the chunk adds 0 NEW MUST-SHIP + 0 MAY-COVER tests + 0 inline `#[cfg(test)]` additions).

**Pause condition**:
- If test count > 1599 → AskUserQuestion (unexpected test additions; likely indicates scope creep or hidden behavior change).
- If test count < 1599 → AskUserQuestion (regression; tests should remain stable at baseline).

**MUST-SHIP test enumeration**: NONE this chunk.

**MAY-COVER test enumeration**: NONE this chunk.

**Per-Tier test-cardinality breakdown** (per chunk-planner v22 P-plan-1) — N/A this chunk (no new tests; existing tier distribution preserved).

**Layer breakdown** (no shift from CH-28b close):
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

1. **Forward-scope row**: [`forward-scope/m6-forward-scope-8b7a8bcd.md`](../../forward-scope/m6-forward-scope-8b7a8bcd.md) §1 lines 69–79 + §5 table line 336 (the CH-28c row + table row).
2. **i-phi diagnostic** (primary input): [`/root/projects/phi/i-phi/docs/v0/proposal/plan/build/phi-core-0.9.0/plan.md`](../../../../../../i-phi/docs/v0/proposal/plan/build/phi-core-0.9.0/plan.md) §"baby-phi absorption" lines 267–273 (empirical breaking-change impact for baby-phi; predicted LOW; verified at plan-draft as zero compile breaks).
3. **phi-core CHANGELOG**: [`/root/projects/phi/phi-core/CHANGELOG.md`](../../../../../../phi-core/CHANGELOG.md) §[0.9.0] lines 21–186 (the 2 surface enumerations: per-turn debug capture + async-trait migration + Added items + Migration guidance).
4. **phi-core concept-debugging** (NEW at 0.9.0): [`/root/projects/phi/phi-core/docs/concepts/debugging.md`](../../../../../../phi-core/docs/concepts/debugging.md) (design source-of-truth for what's being deferred).
5. **CH-28b ADR-0064 precedent**: [`/root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/decisions/0064-phi-core-08-absorption.md`](../../../v0/implementation/m6/decisions/0064-phi-core-08-absorption.md) — direct parallel pattern; ADR-0065 mirrors the section shape + sub-decision body conventions exactly.
6. **CH-28b drift precedent**: [`/root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/drifts/D-PHICORE-08-FOLLOWUP-01-composition-i-adoption.md`](../../../v0/implementation/m6/drifts/D-PHICORE-08-FOLLOWUP-01-composition-i-adoption.md) — drift file shape + naming baseline for D-PHICORE-09-FOLLOWUP-01.
7. **CH-28b P-SEAL plan**: [`docs/specs/plan/build/ch-28b-phi-core-08-absorption-d5b776ac/plan.md`](../ch-28b-phi-core-08-absorption-d5b776ac/plan.md) §11 (the audit-A scaffold template that CH-28c §11 below mirrors).
8. **feature-inventory.md §3 Deferred catalogue** (existing `D-PHICORE-08-FOLLOWUP-01` row at lines 125–130) — row shape baseline.
9. **baby-phi CLAUDE.md** §"phi-core Leverage" rules 1–5.

**Carry-forward invariants** (explicit list, verified green at chunk open):
- `cargo test --workspace --no-fail-fast` test count = **1599** (CH-28b close baseline).
- `scripts/check-phi-core-reuse.sh` green.
- `scripts/check-doc-links.sh` green.
- `scripts/check-ops-doc-headers.sh` green.
- `scripts/check-spec-drift.sh` green.
- `grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l` = **57**.
- `modules/` diff against the chunk-open git HEAD is empty (no source changes pending).
- Cargo.toml workspace-deps row: `phi-core = "0.8"` (will flip to `phi-core = "0.9"` at P1).

**Pending decisions carried into this chunk**:
- F1 / F2 / F3 forks ratified at gate-1 per ExitPlanMode + AskUserQuestion (F1 is RESOLVED at prerequisite commit `b0ed0bb`; F2 + F3 are planner-rec rubber-stamp expected).
- §3.E candidate 1 (session_recorder.rs `matches!()` site explicit-arm omission) ratified at gate-1 OR escalated at gate-2.5 if implementer prefers explicit raised-question route.

**Chunk-ordering note**: per forward-scope row line 75, CH-28c has **CH-28b as prerequisite** (cycle `d5b776ac`, closed 2026-05-23 commit `01e59f2` — confirmed via the cycle-index row at line 70 + git log `e25b3a2` HEAD context for the baby-phi submodule bump).

---

## §10 — Close criteria

**Composite 4-aspect + 2 confidence % ritual**:

**4 aspects** (each pass/fail):

- **Code aspect**:
  - All 7 deliverables (D1–D7 per forward-scope row line 76) shipped per §7 phases.
  - `/root/rust-env/cargo/bin/cargo test --workspace --no-fail-fast -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml` returns **1599 passed / 0 failed**.
  - `RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy -j 4 --workspace --all-targets --manifest-path /root/projects/phi/baby-phi/Cargo.toml` returns 0 warnings (marked `NOT-EXECUTED-IN-AUDIT` by sub-agents per outer CLAUDE.md gate-4 MUST-RUN; orchestrator closes at gate-4 MUST-RUN list).
  - `/root/rust-env/cargo/bin/cargo fmt --all -- --check --manifest-path /root/projects/phi/baby-phi/Cargo.toml` returns 0 diff.
- **Docs aspect**:
  - *Governance tier*: NEW drift `D-PHICORE-09-FOLLOWUP-01` filed at canonical `m6/drifts/`; ADR-0065 flipped Proposed → Accepted at P-SEAL; feature-inventory.md §3 row added; cycle-index row appended; verified-header amends on all 4 touched docs (cycle-index row 1 + feature-inventory.md §1 + NEW drift + NEW ADR).
  - *User-facing tier* (per §3.C): all 3 rows have explicit defer-decision with successor-chunk reference (future M7+ per-turn debug capture adoption chunk via `D-PHICORE-09-FOLLOWUP-01`).
- **phi-core leverage aspect**:
  - `grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l` returns **57** (Δ +0 vs CH-28b baseline).
  - `bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh` exits 0 (marked `NOT-EXECUTED-IN-AUDIT` by sub-agent; orchestrator closes at gate-4 MUST-RUN).
  - Forbidden-duplication greps per §3 all return 0 hits.
- **Concept-alignment aspect**:
  - All §2 rows reach target status (`honored` for CHANGELOG; `concept-aspirational-preserved` for debugging.md; `confirmed-zero-impact` for i-phi diagnostic).
  - No new concept-doc contradictions surfaced; the forward-scope-vs-concept-doc precedence §3.D check returns 0 contradictions.

**2 confidence percentages**:
- **Implementation confidence**: **≥ 99%** (`claims-honored / claims-in-scope ≥ 9/10`). Per chunk-planner pre-spawn verification: ZERO compile breaks predicted across all 3 breaking-change classes; structural deliverables D1–D7 mirror CH-28b pattern; ADR + drift bodies follow CH-28b §"Forks" + §"Sub-decisions" + §"Cross-references" + §"Revisit triggers" template exactly.
- **Documentation confidence**: **4/4 = 100%** (governance-tier docs are mechanical — drift file mirrors `D-PHICORE-08-FOLLOWUP-01` shape; ADR mirrors `0064-phi-core-08-absorption.md` shape; feature-inventory row mirrors existing row; cycle-index row mirrors existing row).

**Composite**: 99% × 100% = **99%** (well above 90% gate-1 auto-approval floor).

**Direct-approval criteria** (per chunk-initiate Phase 1.5 — see also checklist at bottom of plan):
| Criterion | Status | Evidence |
|---|---|---|
| No locked forks at plan-time | **PASS** | 3 forks all resolve to planner-rec (F1 RESOLVED at prerequisite commit `b0ed0bb`; F2 + F3 planner-rec rubber-stamps expected at gate-1.5) |
| Scope ≤ 1.5× forward-scope row deliverables | **PASS** | 7 deliverables (D1–D7) match forward-scope row §1 line 76 enumeration exactly; no scope expansion |
| Zero phi-core leverage delta | **PASS** | §3 predicts Δ +0 leverage-sites / Δ +0 raw `use phi_core` lines; baseline 57 preserved |
| No new K8s blocker class | **PASS** | §3.B 7-axis table all rows "no impact"; K8s-neutral verdict |
| Audit envelope ≤ medium | **PASS** | SMALL (1 auditor letter A) per §11 sizing override rationale |
| Confidence ≥ 9/10 | **PASS** | implementation 9.9/10 + documentation 4/4; composite 99% |
| No new migration | **PASS** | §3.F SCHEMAFULL checklist N/A; zero SurrealDB schema changes |

All 7 Direct-approval criteria hold. Orchestrator expected to auto-approve via ExitPlanMode at gate-1.

---

## §11 — Post-chunk independent audit plan

**Phase count**: **4 (P1 + P2 + P3 + P-SEAL)** — per audit-envelope-size skill sizing rule, this lands in the **3–5 phases / Medium envelope (2 auditors A + B)** band. However, the chunk's surface area is **even smaller than CH-28b** (zero carrier-fixes + 1 cosmetic match-arm + paperwork; CH-28b had 1 carrier-fix + 1 match-arm + paperwork at 5 phases). Per audit-envelope-size skill's underlying intent (phase count is a proxy for audit complexity, not the load-bearing signal), the planner sizes this chunk as **SMALL (1 auditor letter A)** with the following justification:

**Sizing override rationale**: phase count 4 lands at the lower end of the Medium boundary. 3 of the 4 phases (P1 Cargo bump / P2 cosmetic match-arm / P-SEAL paperwork) are ≤ 10 LOC each + carry mechanical-replication patterns (direct CH-28b precedent). Only P3 (drift + feature-inventory + ADR draft) has docs-fidelity surface to audit; the entire chunk's audit surface fits comfortably in one auditor's scope. The planner self-confirms SMALL via parallel reasoning to CH-28b's SMALL sizing override (which auditor A iter-1 closed cleanly with 12 PASS + 2 NOT-EXECUTED-IN-AUDIT + 1 PASS-with-caveat handled via orchestrator Trivial-1L at gate-3).

**Audit envelope**: **SMALL (1 auditor letter A)** — combined code + phi-core + K8s + concept + docs + ADR coverage.

**Audit aspects (a–d)**:
- (a) Code correctness (D1 Cargo bump + D2 lock regen + D3 explicit arm).
- (b) Docs fidelity vs concept docs (NEW drift body matches phi-core CHANGELOG §[0.9.0]; ADR-0065 sub-decisions match §5; feature-inventory row matches drift body).
- (c) Concept alignment across §2 rows.
- (d) phi-core leverage (Δ +0 leverage-sites; baseline 57 preserved; no forbidden duplications).

### Audit A scaffold (≤ 600 words; combined code + concept + docs + phi-core + K8s)

```
You are auditing CH-28c (phi-core 0.9.0 absorption / per-turn debug capture +
async-trait migration baseline shift) in baby-phi at /root/projects/phi/baby-phi/.
Read-only on source. Plan at
docs/specs/plan/build/ch-28c-phi-core-09-absorption-40214078/plan.md.

Verify each claim with file:line citation:

1. Cargo.toml workspace-deps row at /root/projects/phi/baby-phi/Cargo.toml line 17
   reads `phi-core = "0.9"` (NOT "0.8", NOT "0.9.0", NOT "^0.9.0"). Run:
   `grep -nE '^phi-core = "0.9"$' /root/projects/phi/baby-phi/Cargo.toml`.
   PASS if exit 0 + 1 hit.

2. Cargo.lock regenerated against phi-core 0.9.x. Run:
   `grep -nA 1 '^name = "phi-core"' /root/projects/phi/baby-phi/Cargo.lock |
    grep '^version'`. Expect `version = "0.9.0"` (or higher 0.9.x).

3. Explicit match-arm at
   `/root/projects/phi/baby-phi/modules/crates/cli/src/commands/agent.rs`
   adds `AgentEvent::TurnRequest { .. } => {}` between the existing
   `RevertApplied { .. } => {}` arm (line 674) and the `_ => {}` wildcard.
   Run: `grep -nE 'AgentEvent::TurnRequest' modules/crates/cli/src/commands/agent.rs`.
   PASS if ≥ 1 hit at a line between 674 and the wildcard.

4. session_recorder.rs `matches!()` site at line 166-170 is NOT modified
   (per ADR-0065 §D65.3 — single-pattern matches!() site; explicit arm doesn't
   apply per the same scope-narrow rationale as CH-28b §D64.6). Run:
   `git diff HEAD~ -- modules/crates/domain/src/session_recorder.rs |
    grep -c 'TurnRequest'`. Expect 0.

5. NEW drift filed at canonical m6/drifts/ path. Run:
   `ls /root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/drifts/D-PHICORE-09-FOLLOWUP-01-per-turn-debug-capture-adoption.md`.
   PASS if file exists with line 1 `<!-- Last verified: 2026-05-24 by Claude Code -->` header.

6. ADR-0065 filed at canonical m6/decisions/ path with Status: Accepted at P-SEAL.
   Run:
   `head -10 /root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/decisions/0065-phi-core-09-absorption.md`.
   PASS if Status line reads `Status: Accepted` + sub-decisions §D65.1–§D65.5 referenced.

7. feature-inventory.md §3 row added for D-PHICORE-09-FOLLOWUP-01. Run:
   `grep -nE '### D-PHICORE-09-FOLLOWUP-01' /root/projects/phi/baby-phi/docs/specs/v0/feature-inventory.md`.
   PASS if ≥ 1 hit + row body has 5 fields (Feature impact / User-visible v0 /
   User-visible final / Allocation chunk / Cross-chunk dep).

8. cycle-index row appended for `40214078`. Run:
   `grep -n '40214078' /root/projects/phi/baby-phi/docs/specs/plan/build/_cycle-index.md`.
   PASS if ≥ 1 hit at active-cycles tail; row shape per chunk-implementer P-SEAL v17.

9. cargo test --workspace --no-fail-fast green at expected count 1599. Run:
   `cargo test --workspace --no-fail-fast --manifest-path /root/projects/phi/baby-phi/Cargo.toml -j 4`.
   Mark NOT-EXECUTED-IN-AUDIT if sandbox-blocked; orchestrator closes at gate-4.

10. CI guards green; check-phi-core-reuse.sh exit 0; no new `use phi_core::`
    imports beyond §3 prediction (baseline 57 preserved). Run:
    `bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh` (mark
    NOT-EXECUTED-IN-AUDIT if sandbox-blocked). Run:
    `grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l`;
    expect 57.

11. CH-28b carrier-fix invariant intact: `revert_pending: None` still present at
    `launch.rs`. Run:
    `grep -n 'revert_pending: None' modules/crates/server/src/platform/sessions/launch.rs`.
    PASS if ≥ 1 hit.

12. CH-28b explicit-arm invariant intact: `AgentEvent::RevertApplied { .. } => {}`
    still present at `cli/agent.rs`. Run:
    `grep -nE 'AgentEvent::RevertApplied' modules/crates/cli/src/commands/agent.rs`.
    PASS if ≥ 1 hit.

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
    {Cargo.toml, Cargo.lock, modules/crates/cli/src/commands/agent.rs, docs/**/*.md}
    are touched.

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

# 2. Workspace health (cargo with -j 4 per feedback_cargo_jobs_cap)
/root/rust-env/cargo/bin/cargo fmt --all --manifest-path /root/projects/phi/baby-phi/Cargo.toml -- --check
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml --workspace --all-targets
/root/rust-env/cargo/bin/cargo test --workspace --no-fail-fast -j 4 --manifest-path /root/projects/phi/baby-phi/Cargo.toml
# expect: 1599 passed / 0 failed

# 3. Cargo cleanup per chunk-implementer v8 placement-1
/root/rust-env/cargo/bin/cargo clean --manifest-path /root/projects/phi/baby-phi/Cargo.toml

# 4. Chunk-specific greps (per plan §3 positive close-audit greps)
# 4a. Cargo.toml workspace-deps row
grep -nE '^phi-core = "0.9"$' /root/projects/phi/baby-phi/Cargo.toml
# expect: 1 hit at line 17

# 4b. Cargo.lock pin
grep -nA 1 '^name = "phi-core"' /root/projects/phi/baby-phi/Cargo.lock | grep '^version'
# expect: version = "0.9.0" (or higher 0.9.x)

# 4c. Explicit match-arm
grep -nE 'AgentEvent::TurnRequest' /root/projects/phi/baby-phi/modules/crates/cli/src/commands/agent.rs
# expect: ≥ 1 hit between line 674 (RevertApplied arm) and the wildcard

# 4d. phi-core import baseline preserved
grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l
# expect: 57

# 5. Forbidden-duplication greps (per plan §3)
grep -rn "^pub enum AgentEvent" /root/projects/phi/baby-phi/modules/crates/
# expect: 0 hits
grep -rn "^pub struct AnnotatedRequestPayload" /root/projects/phi/baby-phi/modules/crates/
# expect: 0 hits
grep -rn "^pub enum BlockProvenance" /root/projects/phi/baby-phi/modules/crates/
# expect: 0 hits
grep -rn "^pub enum ProvenanceRole" /root/projects/phi/baby-phi/modules/crates/
# expect: 0 hits
grep -rn "^pub struct LlmMessage" /root/projects/phi/baby-phi/modules/crates/
# expect: 0 hits
grep -rn "^pub struct AgentLoopConfig" /root/projects/phi/baby-phi/modules/crates/
# expect: 0 hits
grep -rn "impl phi_core::context::BlockCompactionStrategy for" /root/projects/phi/baby-phi/modules/crates/
# expect: 0 hits
grep -rn "impl phi_core::.*::InputFilter for" /root/projects/phi/baby-phi/modules/crates/
# expect: 0 hits

# 6. Drift file presence
ls /root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/drifts/D-PHICORE-09-FOLLOWUP-01-per-turn-debug-capture-adoption.md
# expect: file exists

# 7. ADR file presence + Accepted status
head -10 /root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/decisions/0065-phi-core-09-absorption.md
# expect: Status: Accepted, sub-decisions §D65.1–§D65.5 referenced

# 8. feature-inventory.md §3 row presence
grep -nE '### D-PHICORE-09-FOLLOWUP-01' /root/projects/phi/baby-phi/docs/specs/v0/feature-inventory.md
# expect: ≥ 1 hit

# 9. Cycle-index row presence
grep -n '40214078' /root/projects/phi/baby-phi/docs/specs/plan/build/_cycle-index.md
# expect: ≥ 1 hit at active-cycles tail

# 10. CH-28b carrier-fix invariant intact
grep -n 'revert_pending: None' /root/projects/phi/baby-phi/modules/crates/server/src/platform/sessions/launch.rs
# expect: ≥ 1 hit

# 11. CH-28b explicit-arm invariant intact
grep -nE 'AgentEvent::RevertApplied' /root/projects/phi/baby-phi/modules/crates/cli/src/commands/agent.rs
# expect: ≥ 1 hit at line 674 (or its post-insertion equivalent)

# 12. CH-25 P-SEAL invariant intact
grep -n 'response_format: phi_core::provider::traits::ResponseFormat::default' /root/projects/phi/baby-phi/modules/crates/server/src/platform/sessions/launch.rs
# expect: 1 hit

# 13. CH-28 invariant intact
/root/rust-env/cargo/bin/cargo test --workspace --test acceptance_m6_agent_profile_cardinality --manifest-path /root/projects/phi/baby-phi/Cargo.toml -j 4
# expect: 7 tests passed

# 14. Drift-file status (NEW drift added at chunk-seal)
grep -l "Status.*discovered" /root/projects/phi/baby-phi/docs/specs/v0/implementation/m6/drifts/D-PHICORE-09-FOLLOWUP-01*.md | wc -l
# expect: 1 (the NEW drift is at Status: discovered, NOT remediated)
```

---

## End of plan
