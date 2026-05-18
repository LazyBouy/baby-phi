<!-- Last verified: 2026-05-18 by Claude Code (CH-27 P-DOCS — filed at chunk-seal per ADR-0062 §D62.3 (F3.a LOCKED user-aligned deferral): `projects::resolvers::*` actor-passthrough architectural design exceeds M5.3 carve-out blast-radius envelope. M6 plan-open inherits this scope via M6-DEFERRED-RESOLVERS-WIRING allocation (explicit per chunk-planner v13 non-terminal-drift rule). Cycle hex `0edcaba9`.) -->

# D-CH27-FOLLOWUP-01 — `projects::resolvers::*` actor-passthrough wiring deferred to M6

## Identification
- **ID**: D-CH27-FOLLOWUP-01
- **Phase of origin**: CH-27 chunk-seal (2026-05-18) — filed per ADR-0062 §D62.3 (F3.a LOCKED at gate-1, user-aligned with planner-recommendation).
- **Discovery source**: chunk-planner v18 — surfaced at planning time via §3 fork F3 architectural analysis; orchestrator-locked at gate-1.
- **Date discovered**: 2026-05-18
- **Status**: `discovered`
- **Bucket**: B — follow-on engine-scope widening (architectural — background-listener trait shape)
- **Severity**: LOW
- **Tags**: `resolvers-actor-passthrough`, `background-listener-trait-shape`, `m6-deferred`, `m5-3-carveout-close`
- **Blocks**: nothing within M5; M5.3 carve-out closes at CH-27 with this axis deferred.
- **Blocked-by**: nothing — CH-27 ships the wire-tier + synth-grant + fixture-extension axes; this axis is the deferred 4th deliverable from D-CH26-FOLLOWUP-01.
- **Closing chunk**: **M6-DEFERRED-RESOLVERS-WIRING** (per chunk-planner v13 non-terminal-drift rule explicit M*-DEFERRED-NN allocation requirement; NOT `TBD`).

## Concept alignment
- **Concept doc(s)**: [`concepts/core-philosophy.md`](../../../concepts/core-philosophy.md) line 16, 28, 29 (Org-has-Projects + Org/Project own Resources — already honored at the load-bearing semantic + wire-tier axes post-CH-26 + CH-27; this drift covers the background-tier read-path widening).
- **Contradiction at CH-27 close**: NONE at user-facing surfaces. The `projects::resolvers::*` background trait impls at `domain/src/events/listeners.rs:244-507` are fire-listener traits (`AdoptionArResolver`, `ActorResolver`, `TemplateCAdoptionArResolver`, `TemplateDAdoptionArResolver`) with `resolve(&self, project: ProjectId) -> Option<...>` / `resolve(&self, org: OrgId) -> Option<AgentId>` signatures — **no actor parameter on the trait shape**. They run inside the in-process event bus background dispatcher; they do NOT participate in the HTTP-tier handler chain. The blocking-gate tightening at CH-27 (ADR-0062 §D62.1) closes the HTTP-tier handler chain wire-tier axis; the background-listener read-tier remains the M6 work.
- **Classification**: `architectural-deferral` (the closing chunk's scope-control choice exceeds M5.3 carve-out envelope per the planner's v18 R1 cost framework + user-aligned F3.a lock at gate-1).
- **phi-core leverage status**: `N/A` — `projects::resolvers::*` traits + impls are baby-phi-native (in `domain` crate).

## Plan vs. reality
- **Plan §3 (CH-27 v2) said (F3.a LOCKED)**: defer `projects::resolvers::*` actor-passthrough wiring to M6 plan-open via this drift (`D-CH27-FOLLOWUP-01`) with explicit `M6-DEFERRED-RESOLVERS-WIRING` allocation.
- **Reality at CH-27 chunk-seal**: matches plan exactly. The 4 resolver trait shapes at `domain/src/events/listeners.rs:244-507` remain unchanged from CH-26 baseline. The 4 production `Repo*Resolver` impls + ~6 static-test stubs are untouched. The drift body documents the M6 design scope.
- **Root cause**: the F3 fork options at gate-1 were:
  - **F3.a (LOCKED, user-aligned)** — defer to M6 (~0 ed in-cycle, +0.5 ed for this drift filing). Architectural design moved to M6 plan-open for coherent design across all background-listener trait shapes.
  - F3.b (NOT locked) — extend trait with `Option<AgentId>` (~3-4 ed). Trait-shape change cascades through 4 trait defs + 4 `Repo*Resolver` impls + ~6 static-test stubs + 1 production wiring site. Mid-cycle architectural risk.
  - F3.c (NOT locked) — HTTP-tier wrapper (~1.5-2 ed). Adds ~50 LOC duplication; future-convergence risk.
  - User-locked F3.a at gate-1 with rationale: the resolver layer's actor-passthrough design should be coherent across ALL background-listener trait shapes (not just `projects::resolvers::*`), which exceeds the M5.3 carve-out envelope.

## Where visible in code
- **Files**:
  - `modules/crates/domain/src/events/listeners.rs:244-507` — 4 background fire-listener traits. Each `resolve(&self, ...)` signature has NO actor parameter.
  - `modules/crates/store/src/repo_impl.rs` — 4 `Repo*Resolver` impls bound to the trait shape.
- **Grep for regression** (CH-27 close baseline vs M6+ close target):
  - `grep -nE 'pub trait (AdoptionArResolver|ActorResolver|TemplateCAdoptionArResolver|TemplateDAdoptionArResolver)' modules/crates/domain/src/events/listeners.rs` — CH-27: 4 hits (current trait shape, no actor param). M6+ target: 4 hits (post-actor-passthrough trait re-shape OR HTTP-wrapper sibling design).
  - `grep -rnE 'check_permission.*resolvers' modules/crates/` — CH-27: 0 hits. M6+ target: ≥ 4 hits (one per resolver, post-wiring).

## Remediation scope (estimate only)
- **Approach (sketch — M6 plan)**:
  1. **Architectural design**: choose trait-re-shape (F3.b path) vs HTTP-tier wrapper (F3.c path) vs cascade-via-related-design (M6 plan-open may surface a 4th option). Likely converges on trait-re-shape if M6 admin pages need uniform actor-passthrough across all background-listener traits.
  2. **Trait re-shape (if F3.b path)**: extend the 4 trait signatures with `actor: Option<AgentId>` parameter. Wire production `Repo*Resolver` impls + ~6 static-test stubs. Wire `check_permission` invocation at each resolver's read-tier (Action::Observe on `project:<P>` / `org:<O>` URIs).
  3. **HTTP-tier wrapper (if F3.c path)**: ship a NEW `server::handler_support::project_resolver_with_gate` helper (~50 LOC) that wraps each resolver call + adds the `check_permission` invocation post-resolve. Background listeners stay actor-blind.
  4. **Acceptance test coverage**: per-resolver scenarios exercising the gate (cross-org viewer denied at the resolver tier).
- **Implementation chunk**: **M6-DEFERRED-RESOLVERS-WIRING** (per chunk-planner v13 non-terminal-drift rule).
- **Dependencies on other drifts**: none. All upstream surfaces (Composite variants, tags field, blocking gates, synth-grant widening) ship by CH-27 close.
- **Estimated effort**: ~3-4 ed (F3.b path) OR ~1.5-2 ed (F3.c path); M6 plan-open chooses.
- **Risk to concept alignment if deferred further**: LOW. The background-listener tier does not participate in user-facing handler chains; the load-bearing semantic claim ships at CH-26 + the HTTP-tier closure ships at CH-27. The drift's remediation extends the engine-gating discipline uniformly across background-tier reads but does NOT change semantic correctness at the HTTP-tier.

## Why filed as a follow-on drift (NOT M5-carve-out)

User routing decision (2026-05-18, gate-1 user-aligned F3.a lock): the `projects::resolvers::*` actor-passthrough architectural design exceeds the M5.3 carve-out blast-radius envelope. M5.3 carve-out closes with the 3-chunk arc {CH-25, CH-26, CH-27} delivering:
- CH-25: synth-grant rule + Edge::Owns variant.
- CH-26: composite-resource axis + tag-field + advisory consumption.
- CH-27: blocking-gate closure + synth-grant widening + fixture-extension via F4.b opt-in helper.

The resolver wiring would have:
- Either cascaded a non-trivial trait re-shape into M5.3 mid-cycle (F3.b path) — mid-cycle architectural risk.
- Or introduced a 50-LOC wrapper duplication at the HTTP tier (F3.c path) — convergence risk back to trait re-shape later.

M6 plan-open will design the actor-passthrough architecture coherently across all background-listener trait shapes (not just `projects::resolvers::*`) as part of M6+ admin-page work.

## Lifecycle history
- 2026-05-18 — `discovered` — filed at CH-27 P-DOCS per ADR-0062 §D62.3 F3.a LOCKED user-aligned deferral; M6-DEFERRED-RESOLVERS-WIRING allocation per chunk-planner v13.

## Cross-references
- [`ADR-0062`](../decisions/0062-blocking-gate-and-synth-grant-widening.md) §D62.3 — F3.a LOCKED user-aligned deferral rationale + Pre-existing-behaviour preservation note (deferred-scope variation per chunk-planner v11).
- [`D-CH26-FOLLOWUP-01`](D-CH26-FOLLOWUP-01.md) §"Remediation scope" — original 4-axis scope of which this is the 4th (deferred) axis.
- [`composite-resources-model.md`](../architecture/composite-resources-model.md) §"CH-27 deliverables shipped (closes D-CH26-FOLLOWUP-01)" — body documents the M6 deferral for this drift.
- Plan archive: [`plan/build/ch-27-blocking-gate-enforcement-resolvers-wiring-0edcaba9/plan.md`](../../../../plan/build/ch-27-blocking-gate-enforcement-resolvers-wiring-0edcaba9/plan.md) — CH-27 plan §3 F3 fork + §3.B K8s readiness rows for this scope.
