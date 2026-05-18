<!-- Last verified: 2026-05-17 by Claude Code (CH-26 P-DOCS — filed at chunk-seal per user-routed CH-27 carve-out decision 2026-05-16: keep advisory → blocking gate tightening + synth-grant widening + resolvers wiring + acceptance-fixture extension IN M5 carve-out rather than M6+ deferral; closing chunk CH-27 NOT M6-DEFERRED-NN.) -->

# D-CH26-FOLLOWUP-01 — Advisory → blocking gate tightening + synth-grant widening + resolvers wiring + acceptance-fixture extension

## Identification
- **ID**: D-CH26-FOLLOWUP-01
- **Phase of origin**: CH-26 chunk-seal (2026-05-17) — filed per user-routed CH-27 carve-out decision 2026-05-16.
- **Discovery source**: chunk-implementer scope-revision (mid-cycle); orchestrator-confirmed at gate-3.
- **Date discovered**: 2026-05-16
- **Status**: `discovered`
- **Bucket**: B — advisory → blocking gate tightening + follow-on engine-scope widening
- **Severity**: LOW
- **Tags**: `advisory-to-blocking`, `synth-grant-scope-widening`, `resolvers-actor-passthrough`, `acceptance-fixture-extension`, `m5-3-carveout`
- **Blocks**: nothing within M5; CH-27 (its closing chunk) ships in the M5.3 carve-out extension before M6 plan-open.
- **Blocked-by**: nothing — all the surfaces it touches ship at CH-26 close.
- **Closing chunk**: **CH-27** (NOT M6-DEFERRED-NN — kept in M5 carve-out per user direction 2026-05-16).

## Concept alignment
- **Concept doc(s)**: [`concepts/core-philosophy.md`](../../../concepts/core-philosophy.md) line 16, 28, 29 (further-honored at the wire-tier axis post-CH-27); [`concepts/permissions/04-manifest-and-resolution.md`](../../../concepts/permissions/04-manifest-and-resolution.md) §"Owner-grant auto-issue rule (CH-25 / ADR-0060)" (synth-grant scope widening).
- **Contradiction at CH-26 close**: the CH-26 handler refactor consumed `engine.check_permission` results **advisorily** — the bespoke `AuthenticatedSession` / role-check / AR-filter gates remained the HTTP-tier rejection surface. Per ADR-0061 §D61.5 (Accepted): *"the engine.check_permission invocations across the 7 refactored handlers are CONSUMED ADVISORILY — bespoke gates remain as wire-tier rejection surface. The load-bearing semantic claim (engine matches `org:O` + `project:P` selectors for owner-Agents per the synth-owner-grant rule) is fully shipped + validated by acceptance tests; the wire-tier tightening to blocking gates is deferred to CH-27."*
- **Classification**: `partial-implementation-of-load-bearing-form` (the load-bearing semantic claim ships fully; the wire-tier tightening is the deferred half).
- **phi-core leverage status**: `N/A` — Permission Check engine + Repository + handler stack are baby-phi-native.

## Plan vs. reality
- **Plan §3 + ADR-0061 §D61.5 said (F1.b user-lock)**: ≥ 7 handler refactor sites; engine result mapped to HTTP 403 via `denial_to_api_error` per CH-25 wire convention.
- **Reality at CH-26 chunk-seal**:
  - 15 `engine.check_permission` invocations land across 7 handlers (`orgs::{list, show, create, dashboard}` + `projects::{create, detail, agent_supervisor}`) — D-philosophy-02:39 ≥ 3-hit invariant FULLY met (15 ≫ 3).
  - All invocations consumed advisorily (engine verdict captured but bespoke gate decides response status).
  - `projects::resolvers::*` background trait impls NOT wired through `check_permission` — no actor parameter on resolver trait shape.
  - CH-25 synth-owner-grant scope (`step_2_resolve_grants`) covers `[Action::Allocate, Action::Transfer]` only — `Action::Observe` + `Action::Inspect` (the natural verbs for show/list/dashboard) NOT covered.
  - Acceptance suite `acceptance_m5_3_composite_resources.rs` pinned the engine-verdict tests at `Action::Allocate` to work within the synth-grant scope; 2 scenarios (`owner_inspect_via_show_organization_handler_path`, `owner_observe_via_dashboard_handler_path`) renamed to `owner_allocate_via_*_handler_path` per the constraint.
- **Root cause**: scope-revision during P2 — the WIDE-scope F1.b user-lock anticipated blocking gates but the implementer-time discovery surfaced that the M3 + M4 + M5 acceptance suite would break en-masse without Edge::Owns / explicit-grant seeding extensions. Routing the tightening to a NEW M5-carveout chunk (CH-27) where the fixture extension can be designed coherently produces a cleaner contract than gate-2 inline correction would have.

## Where visible in code
- **Files**:
  - `modules/crates/server/src/platform/orgs/{list,show,create,dashboard}.rs` — 15 advisory `check_permission` invocations.
  - `modules/crates/server/src/platform/projects/{create,detail,agent_supervisor}.rs` — handler refactor.
  - `modules/crates/server/src/platform/projects/resolvers/*.rs` — NOT wired; awaits actor-passthrough design.
  - `modules/crates/domain/src/permissions/engine.rs::step_2_resolve_grants` — synth-owner-grant rule (covers `[Allocate, Transfer]`).
  - `modules/crates/server/tests/acceptance_m5_3_composite_resources.rs` — 2 scenarios pinned at `Action::Allocate` per synth-grant scope constraint.
- **Grep for regression** (CH-26 close baseline vs CH-27 close target):
  - `grep -rnE 'check_permission' modules/crates/server/src/platform/{orgs,projects}/ | wc -l` — CH-26: 15. CH-27 target: ≥ 15 (no regression; resolver layer adds a few more).
  - `grep -nE 'denial_to_api_error' modules/crates/server/src/platform/{orgs,projects}/` — CH-26: ~0 wire-tier mapping calls (advisory mode); CH-27 target: ≥ 7 (each blocking gate maps).
  - `grep -nE 'Action::Observe|Action::Inspect' modules/crates/domain/src/permissions/engine.rs::step_2_resolve_grants` — CH-26: 0 in the synth-grant array (scope is `[Allocate, Transfer]`); CH-27 target: ≥ 1 hit per Action verb.

## Remediation scope (estimate only)
- **Approach (sketch — CH-27 plan)**:
  1. **Synth-grant scope widening** (~1-2 engineer-days): extend `step_2_resolve_grants` synth-grant generator to emit `[Allocate, Transfer, Observe, Inspect]` Actions for owner-Agents. Re-run CH-25 ADR-0060 §D60.2 acceptance + add 2 new scenarios (Observe + Inspect via synth-grant).
  2. **Advisory → blocking tightening** (~2-3 engineer-days): convert each of the 15 advisory invocations to `?`-propagation via `denial_to_api_error`. Audit handler-by-handler for response-status correctness (403 vs 404 vs 401 — see CH-25 wire convention).
  3. **`projects::resolvers::*` actor-passthrough** (~1-2 engineer-days): design + ship the actor parameter on the resolver trait shape; wire `check_permission` invocations on the read-tier resolvers (Action::Observe on `project:<P>`).
  4. **M3 + M4 + M5 acceptance fixture extension** (~3-5 engineer-days): seed `Edge::Owns` for the test actor → org / project in every acceptance test that exercises the now-blocking endpoints. Helpers in `acceptance_common::admin` extended to do this by default.
  5. **Re-enable advisory-only-renamed tests** (~0.5 engineer-day): the 2 scenarios in `acceptance_m5_3_composite_resources.rs` renamed `owner_allocate_via_*` revert to `owner_inspect_via_show_organization_handler_path` + `owner_observe_via_dashboard_handler_path` after the synth-grant widens.
  6. **New ADR** (likely **ADR-0062**) ratifying the wire-tier tightening + synth-grant scope expansion.
- **Implementation chunk**: **CH-27** (M5.3 carve-out extension; NOT M6-DEFERRED-NN).
- **Dependencies on other drifts**: none — all upstream surfaces (Composite variants, tags field, advisory invocations) ship at CH-26.
- **Estimated effort**: ~6-10 engineer-days inside CH-27.
- **Risk to concept alignment if deferred further**: LOW at the load-bearing-semantic axis (CH-26 ships that fully); MEDIUM at the wire-tier axis if M6 admin pages ship before CH-27 closes (M6 endpoints would need their own bespoke gate plumbing).

## Why filed as a follow-on drift (NOT M6-DEFERRED-NN)

User routing decision 2026-05-16: *"Can we create a CH-27 to document the remaining scope and finish it in M5?"*

The advisory-only → blocking-gate tightening is **scope-narrowing vs the gate-1 user-lock F1.b** — the lock said "wire ≥ 7 handlers"; the implementer shipped 15 invocations consumed advisorily. The orchestrator could have inline-corrected at gate-2 (Option B from CH-14's per-AR-emission precedent) to ship the blocking gates verbatim per plan + ADR, but the M3 + M4 + M5 acceptance fixture breakage would have required ~3-5 engineer-days of in-cycle fixture extension. Routing this work to a NEW M5-carve-out chunk gives the fixture extension a coherent design surface (CH-27 plan §3 can map the per-test extension explicitly) rather than a rushed gate-2 inline correction.

The drift is **NOT M6-DEFERRED-NN** because the user explicitly requested closing this work within the M5 carve-out — M5.3 extends from 2-chunk {CH-25, CH-26} → 3-chunk {CH-25, CH-26, CH-27}; M6 plan-open shifts accordingly.

## Lifecycle history
- 2026-05-16 — `discovered` — implementer-time scope-revision surfaced advisory-only consumption pattern; orchestrator-approved + user-routed to CH-27 carve-out.
- 2026-05-17 — drift filed at CH-26 chunk-seal; CH-27 plan-open expected next.

## Cross-references
- [`ADR-0061`](../decisions/0061-org-project-as-composite-resources.md) §D61.5 — advisory-only revision rationale.
- [`D-philosophy-02`](D-philosophy-02.md) — closed at CH-26 (load-bearing semantic axis); CH-27 closes the wire-tier axis here.
- [`agent-ownership-model.md`](../architecture/agent-ownership-model.md) — CH-25 design page for the synth-owner-grant rule that CH-27 widens.
- [`composite-resources-model.md`](../architecture/composite-resources-model.md) — CH-26 design page; §5 references this drift for the deferred tightening.
- Plan archive: [`plan/build/ch-26-org-project-as-composite-d1cb9e1f/plan.md`](../../../../plan/build/ch-26-org-project-as-composite-d1cb9e1f/plan.md).
- Resume note: [`plan/build/ch-26-org-project-as-composite-d1cb9e1f/RESUME-NOTE.md`](../../../../plan/build/ch-26-org-project-as-composite-d1cb9e1f/RESUME-NOTE.md).
