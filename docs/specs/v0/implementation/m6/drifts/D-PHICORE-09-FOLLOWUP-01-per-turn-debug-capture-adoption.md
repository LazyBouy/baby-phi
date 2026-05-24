<!-- Last verified: 2026-05-24 by Claude Code (CH-28c P3 — filed at chunk-seal per F3.a planner-rec lock + ADR-0065 §D65.4: phi-core 0.9.0 ships per-turn debug capture as opt-in via `SessionRecorderConfig::capture_turn_requests: bool` (default `false`) + new types `BlockProvenance` / `ProvenanceRole` / `AnnotatedRequestPayload` + `AgentEvent::TurnRequest` variant + `Turn::request_payload: Option<AnnotatedRequestPayload>` field. baby-phi absorbs the type surface but does NOT enable capture at session launch — `SessionRecorderConfig::default()` at `cli/src/commands/agent.rs:636` keeps `capture_turn_requests = false`. This drift tracks the deferred adoption work (the 4 prerequisites itemised in §"Remediation scope") to a future M7+ FUNCTIONAL chunk. Allocation: `M7+-FUTURE-PER-TURN-DEBUG-CAPTURE-ADOPTION` placeholder (no specific CH-NN slot reserved at CH-28c close; future planning session decides). Cycle hex `40214078`.) -->

# D-PHICORE-09-FOLLOWUP-01 — Per-turn debug capture adoption (deferred to future M7+ FUNCTIONAL chunk)

## Identification
- **ID**: D-PHICORE-09-FOLLOWUP-01
- **Phase of origin**: CH-28c P3 chunk-seal (2026-05-24) — filed per F3.a planner-rec lock at gate-1.5 + ADR-0065 §D65.4 deferral decision.
- **Discovery source**: phi-core 0.9.0 breaking-change release (2026-05-24) — per-turn debug capture shipped as opt-in via `SessionRecorderConfig::capture_turn_requests: bool` (default `false`) + new types `BlockProvenance` / `ProvenanceRole` / `AnnotatedRequestPayload` + `AgentEvent::TurnRequest` variant + `Turn::request_payload: Option<AnnotatedRequestPayload>` (with `#[serde(default)]` for back-compat). i-phi project's pre-spawn diagnostic (`i-phi/docs/v0/proposal/plan/build/phi-core-0.9.0/plan.md` §"baby-phi absorption" lines 267–273) predicted baby-phi adoption work is a separate feature-work chunk distinct from absorbing the 0.9.0 compile-restoration surface.
- **Date discovered**: 2026-05-24
- **Status**: `discovered`
- **Bucket**: B — follow-on engine-scope widening (opt-in observability feature adoption + JSON-size-implications documentation + operator inspection UI).
- **Severity**: LOW
- **Tags**: `phi-core-0.9`, `per-turn-debug-capture`, `observability`, `opt-in-feature`, `m7-future`, `debugging`
- **Blocks**: nothing within CH-28c; the dependency baseline shift + explicit `TurnRequest` match-arm + zero-impact verification together close the breaking-change absorption. Per-turn debug capture adoption is the NEXT feature-work concern.
- **Blocked-by**: nothing — CH-28c ships the dependency baseline shift (phi-core 0.8 → 0.9.x) which unblocks adoption. All adoption prerequisites are inside baby-phi's source tree + operator-tooling surface; no further phi-core work needed.
- **Closing chunk**: **M7+-FUTURE-PER-TURN-DEBUG-CAPTURE-ADOPTION** (placeholder; per chunk-planner v13 non-terminal-drift rule the placeholder names an explicit-named-future-allocation rather than `TBD`; no specific CH-NN slot reserved at CH-28c close; future planning session decides whether adoption lands as a dedicated FUNCTIONAL chunk OR as an axis bundled into an existing M7+ observability chunk).

## Concept alignment
- **Concept doc(s)**: [`phi-core/docs/concepts/debugging.md`](../../../../../../../phi-core/docs/concepts/debugging.md) — design source-of-truth for per-turn debug capture: opt-in flow via `SessionRecorderConfig::capture_turn_requests`, JSON-size implications, debugging recipe for reconstructing wire payloads. [`phi-core/CHANGELOG.md`](../../../../../../../phi-core/CHANGELOG.md) §[0.9.0] lines 21–186 — release notes enumerating the 2 bundled surfaces (per-turn debug capture + async-trait migration) + Added items + Migration guidance.
- **Contradiction at CH-28c close**: NONE at the user-facing surfaces. The pre-0.9.0 posture (no per-turn debug payload persistence; debugging unexpected LLM responses requires manual reconstruction from `Turn.input_messages` + ad-hoc re-application of `convert_to_llm()`) is preserved verbatim; agents continue to operate identically. The per-turn debug capture surface is opt-in upstream + remains opt-out in baby-phi by virtue of `SessionRecorderConfig::default()` (which sets `capture_turn_requests = false`).
- **Classification**: `feature-deferral` (opt-in feature; CH-28c absorbs the type surface + compile-restoration breaking-change surface only + defers the adoption per F3.a lock at gate-1.5).
- **phi-core leverage status**: `direct-reuse-available` — adoption brings new `use phi_core::session::recorder::SessionRecorderConfig` (already imported at `cli/src/commands/agent.rs:1`) + new field references on `SessionRecorderConfig::capture_turn_requests` + read-access on `Turn::request_payload` + new type imports `use phi_core::types::event::{BlockProvenance, ProvenanceRole, AnnotatedRequestPayload}` at the future adoption chunk. NO new types should be created in baby-phi; the feature-work is wiring + operator-UI surface + size-policy authoring, not type-system extension.

## Plan vs. reality
- **Plan §1.3 + §4 (CH-28c iter-1) said (F3.a LOCKED at gate-1.5)**: defer per-turn debug capture adoption to future M7+ FUNCTIONAL chunk via this drift; CH-28c ships the baseline shift + the 1 explicit `TurnRequest` match-arm at `cli/agent.rs` + the NEW drift filing + the feature-inventory row + the ADR-0065 paperwork only.
- **Reality at CH-28c chunk-seal**: matches plan exactly. The dependency baseline flipped from `phi-core = "0.8"` to `phi-core = "0.9"` (Cargo.toml:17); Cargo.lock regenerated against phi-core 0.9.0 (4-line churn vs CH-28b's 98-line churn — even smaller); explicit `AgentEvent::TurnRequest { .. } => {}` arm at `cli/agent.rs` between the existing CH-28b `RevertApplied` arm and the wildcard for future-reader discoverability; `session_recorder.rs:166-170` `matches!()` site NOT modified (per D3 scope-narrow + ADR-0065 §D65.3 — single-pattern boolean recognition); workspace test count preserved at 1599 / 0 failed; phi-core import baseline preserved at 57.
- **Root cause**: phi-core 0.9.0 ships per-turn debug capture as an opt-in feature behind `SessionRecorderConfig::capture_turn_requests: bool` (default `false`). Enabling it in baby-phi requires:
  - A config-tier flip at the `SessionRecorderConfig::default()` consumer site at `cli/src/commands/agent.rs:636` (currently `SessionRecorderConfig::default()` → `capture_turn_requests: false`; future adoption flips this to a builder/override that sets `capture_turn_requests: true` conditionally on a baby-phi-level toggle).
  - Optional `AnnotatedRequestPayload` surfacing in baby-phi's observability layer (e.g., a `phi session show --include-request-payload` CLI flag OR an audit-event projection that emits a `TurnRequestCaptured { session_id, turn_id, payload_size }` event for capacity-tracking).
  - JSON-size-implications policy authoring: per-turn debug capture payloads can be large (full system prompt + full message history + tool definitions per turn) + persist on every turn that captures → multi-turn long sessions accumulate substantial JSON-size in `Turn::request_payload`. baby-phi needs a bounding policy — per-agent flag? per-cycle flag? max-payload-bytes truncation with redaction marker?
  - Acceptance suite exercising round-trip serialization of captured payloads + the chosen bounding policy.
- **Why deferred at CH-28c**: opt-in observability feature work is fundamentally different from compile-restoration carrier-fix work; bundling them into a single chunk would obscure the bookkeeping. The CH-28c absorption ships ZERO compile errors (smaller surface than CH-28b which carried 1 carrier-fix); the adoption (future chunk) introduces a behavior delta (persisted JSON payloads on disk) + an operator-tooling delta (inspection surface) + a size-policy delta — together warrants its own audit + size-policy + acceptance scope.

## Where visible in code
- **Files** (CH-28c close baseline; will expand at adoption-chunk):
  - `modules/crates/cli/src/commands/agent.rs:636` — `SessionRecorder::new(SessionRecorderConfig::default())` — currently uses the default (`capture_turn_requests: false`). Future adoption flips to a builder that conditionally sets `capture_turn_requests: true` based on a baby-phi-level toggle (e.g., `PHI_CAPTURE_TURN_REQUESTS=1` env-var, or a config field).
  - `modules/crates/cli/src/commands/agent.rs:685` (post-insertion location) — explicit `AgentEvent::TurnRequest { .. } => {}` arm preserves discoverability; future adoption replaces the no-op body with audit-event emission + payload-size logging + optional truncation policy invocation.
  - `modules/crates/domain/src/session_recorder.rs:166-170` — single-pattern `matches!()` site (matches `PhiCoreAgentEvent::AgentStart { session_id, .. }` ONLY); per §D65.3 NOT modified at CH-28c. Future adoption may extend the `matches!()` to multi-pattern OR replace with exhaustive `match` if `TurnRequest` warrants governance visibility (e.g., audit-event emission per captured turn for capacity-tracking).
- **Grep for regression** (CH-28c close baseline vs future adoption-chunk close target):
  - `grep -rn "capture_turn_requests: true" /root/projects/phi/baby-phi/modules/crates/` — CH-28c: 0 hits. Adoption-chunk target: ≥ 1 hit (the builder/override site).
  - `grep -rn "SessionRecorderConfig::default()" /root/projects/phi/baby-phi/modules/crates/cli/src/commands/agent.rs` — CH-28c: 1 hit at line 636. Adoption-chunk target: 0 hits (replaced with explicit builder) OR ≥ 1 hit (if conditional toggle keeps `default()` for the false path).
  - `grep -rn "use phi_core::types::event::{?AnnotatedRequestPayload|BlockProvenance|ProvenanceRole" /root/projects/phi/baby-phi/modules/crates/` — CH-28c: 0 hits. Adoption-chunk target: ≥ 1 hit (the import line at the adoption wire site).
  - `grep -rn "Turn::request_payload" /root/projects/phi/baby-phi/modules/crates/` — CH-28c: 0 hits. Adoption-chunk target: ≥ 1 hit (the operator-inspection-tier read site).

## Remediation scope (estimate only)

The adoption work decomposes into 4 prerequisites (per i-phi diagnostic §"baby-phi absorption" lines 267–273 + phi-core 0.9.0 CHANGELOG activation guidance):

1. **`SessionRecorderConfig::capture_turn_requests: true` flag flip** — flip the recorder-config consumer site at `cli/src/commands/agent.rs:636` (currently `SessionRecorderConfig::default()`) to a builder that sets `capture_turn_requests: true` conditionally (env-var toggle? CLI flag? per-agent config field?). The choice depends on whether adoption is global (always capture) or scoped (per-agent / per-session opt-in). Likely scoped to avoid blanket JSON-size blow-up. Estimated effort: ~0.2-0.4 ed (~10-30 LOC depending on toggle-mechanism choice).

2. **Optional `TurnRequest` recorder surfacing in baby-phi's observability layer** — decide whether captured turns emit a baby-phi audit event (`TurnRequestCaptured { session_id, turn_id, payload_size }` for capacity-tracking) OR remain phi-core-recorder-only with no baby-phi-tier projection. The choice depends on whether baby-phi's governance needs to track per-turn debug capture (likely YES given the audit-hash-chain posture — JSON-size accumulation is governance-relevant). Estimated effort: ~0.3-0.6 ed (~20-60 LOC + matching audit-event-dictionary additions).

3. **JSON-size-implications policy** — author a bounding policy preventing per-turn debug capture from balloning session JSON files. Candidate axes: per-agent toggle (only debug-agents capture); per-cycle toggle (only flagged debugging cycles capture); max-payload-bytes truncation with redaction marker (e.g., truncate body bytes > 256 KiB and replace with `<TRUNCATED: N bytes>`); rolling-window retention (only last N captured turns persist; older captures are dropped). Document the policy in `m6/operations/per-turn-debug-capture-operations.md`. Estimated effort: ~0.3-0.5 ed for policy + ~0.2-0.5 ed if max-bytes truncation is chosen (truncation logic + acceptance scenarios).

4. **Operator inspection UI / CLI surface** — surface for inspecting `Turn::request_payload`: likely a `phi session show --include-request-payload [--turn N]` CLI flag (with --max-bytes display truncation for readability) OR a Web UI panel under the session-detail page. Acceptance suite exercising the inspection-path. Estimated effort: ~0.3-0.6 ed (~20-80 LOC of CLI handler + acceptance + optional Web UI).

**Aggregate effort estimate**: ~0.8-1.5 ed for a dedicated adoption chunk (with all 4 prerequisites bundled); ~0.4-0.8 ed if bundled into an existing M7 observability chunk that already touches `SessionRecorder` or session-JSON serialization.

**Implementation chunk**: **M7+-FUTURE-PER-TURN-DEBUG-CAPTURE-ADOPTION** (placeholder; no CH-NN slot reserved at CH-28c close — future planning session decides). Chunk-planner v13 non-terminal-drift rule satisfied via the explicit-named-placeholder (NOT `TBD`).

**Dependencies on other drifts**: none. CH-28c's baseline shift unblocks adoption; no upstream drift blocks this. Tangentially related: `D-PHICORE-08-FOLLOWUP-01` (Composition I adoption) — both drifts are M7+ feature-adoption deferrals from consecutive phi-core breaking-change absorption chunks; they may be bundled into a single "M7+ phi-core feature adoption" omnibus chunk if planning capacity allows.

**Risk to concept alignment if deferred further**: LOW. Per-turn debug capture is an opt-in feature; deferring it further preserves the pre-0.9.0 posture (no per-turn debug payload persistence; debugging unexpected LLM responses requires manual reconstruction). The only opportunity cost is operator debugging ergonomics — operators currently reconstruct wire payloads manually from `Turn.input_messages` + ad-hoc re-application of `convert_to_llm()`; after adoption the exact wire payload is in `Turn::request_payload` for every captured turn. No user-visible degradation if deferred; just a missed operator-tooling enhancement.

## Why filed as a follow-on drift (NOT in-CH-28c expansion)

User routing decision (codified at plan-mode session 2026-05-24 + locked at gate-1.5 ratification per F3.a):
- Feature-work scope ~0.8-1.5 ed scoped, requiring config-tier flip + observability-layer wiring + size-policy authoring + operator-UI surface.
- NOT load-bearing for CH-28c's TECHNICAL-PREREQUISITE invariants (the baseline shift + compile restoration are complete with ZERO compile errors + the cosmetic match-arm).
- Intersects M7+ FUNCTIONAL feature surface + M7 observability surface (the size-policy authoring axis especially is observability-policy work distinct from compile-restoration).

Per outer CLAUDE.md gate-5 in-M5-carve-out-vs-M6-DEFERRED routing criteria (CH-26 retro Row 6), this matches the M7+-DEFERRED pattern (analogous to M6-DEFERRED but at the M7 boundary) — NOT load-bearing for current chunk's invariants + intersects M7 feature surface + > ~10-line scoped. Routed to M7+-FUTURE-PER-TURN-DEBUG-CAPTURE-ADOPTION placeholder rather than in-chunk carve-out.

## Lifecycle history
- 2026-05-24 — `discovered` — filed at CH-28c P3 per F3.a lock at gate-1.5 + ADR-0065 §D65.4; M7+-FUTURE-PER-TURN-DEBUG-CAPTURE-ADOPTION placeholder allocation per chunk-planner v13.

## Cross-references
- [`ADR-0065`](../decisions/0065-phi-core-09-absorption.md) §D65.4 — Per-turn debug capture adoption deferred to future M7+ FUNCTIONAL chunk via this drift (Decision body + Pre-existing-absence preserved note).
- [`ADR-0065`](../decisions/0065-phi-core-09-absorption.md) §D65.3 — `AgentEvent::TurnRequest { .. } => {}` arm scope (cli/agent.rs ships explicit arm; session_recorder.rs `matches!()` site NOT modified) — explains why this drift's prerequisite #2 has an open design choice on the SessionRecorder surface.
- [`ADR-0065`](../decisions/0065-phi-core-09-absorption.md) §"Consequences ### For M7+ observability chunk" — inherited requirement amendment + adoption-chunk planning hints.
- [`ADR-0064`](../decisions/0064-phi-core-08-absorption.md) — CH-28b precedent for phi-core breaking-change absorption + opt-in feature adoption deferral via canonical drift filing; ADR-0065 mirrors the section shape + sub-decision body conventions.
- [`D-PHICORE-08-FOLLOWUP-01`](./D-PHICORE-08-FOLLOWUP-01-composition-i-adoption.md) — sibling drift from CH-28b precedent; both drifts are M6+/M7+ feature-adoption deferrals from consecutive phi-core breaking-change absorption chunks.
- [`phi-core/CHANGELOG.md`](../../../../../../../phi-core/CHANGELOG.md) §[0.9.0] lines 21–186 — per-turn debug capture release notes + activation form (`SessionRecorderConfig::capture_turn_requests: true`).
- [`phi-core/docs/concepts/debugging.md`](../../../../../../../phi-core/docs/concepts/debugging.md) — per-turn debug capture design source-of-truth.
- [`i-phi diagnostic`](../../../../../../../i-phi/docs/v0/proposal/plan/build/phi-core-0.9.0/plan.md) §"baby-phi absorption" lines 267–273 — baby-phi adoption guidance (4 prerequisites enumeration).
- Plan archive: [`plan/build/ch-28c-phi-core-09-absorption-40214078/plan.md`](../../../../plan/build/ch-28c-phi-core-09-absorption-40214078/plan.md) §1 + §3 + §4 + §5 §D65.4 — CH-28c plan body documenting the deferral.
- [`feature-inventory.md`](../../../feature-inventory.md) §3 D-PHICORE-09-FOLLOWUP-01 row — product-trajectory translation of this drift's deferred-vs-final state.
