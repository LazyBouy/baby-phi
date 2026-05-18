<!-- Last verified: 2026-05-18 by Claude Code (CH-27-0edcaba9 P-SEAL — flipped Proposed → Accepted at chunk-seal; 6 sub-decisions §D62.1-§D62.6 ratified; §D62.4 F4.b USER-DIVERGENT body documents SCOPE-NARROWING vs plan §3 Artifact C helper-body literal (`Repository::insert_edge` API does not exist; helper materialises explicit Grant via `Repository::create_grant` preserving F4.b wire-format-explicit spirit); §D62.5 META count-amend applied to D-CH26-FOLLOWUP-01 body "15" → "7"; D-CH27-FOLLOWUP-01 filed with M6-DEFERRED-RESOLVERS-WIRING allocation per ADR §D62.3.) -->
<!-- Last verified: 2026-05-18 by Claude Code (CH-27-0edcaba9 P0 draft, Proposed; v2 plan §5 F4.b USER-DIVERGENT body) -->

# ADR-0062 — Blocking-gate enforcement + synth-grant scope widening + resolvers wiring deferral + F4.b opt-in helper for fixture extension

**Status: Accepted**

**Authors**: Claude Code (orchestrator + chunk-planner v18 + chunk-implementer v12)

**Chunk**: CH-27 (cycle hex `0edcaba9`)

**Milestone**: M5.3 (third + final carve-out chunk; closes D-CH26-FOLLOWUP-01 LOW)

**Forks** (gate-1 user-lock 3-of-4 aligned + 1 DIVERGENT — F1.a single CH-27 chunk aligned + F3.a defer resolvers wiring aligned + count-amend aligned + **F4.b opt-in helper `seed_owner_grants` DIVERGENT** from planner-recommended F4.a default-extension; cumulative cross-cycle divergent forks now **11-of-16 cycles (~69%) — pattern SOFTENING from prior ~83%**).

---

## Forks

| Fork | Locked option | Path | Pros | Cons | Cross-cycle pattern |
|---|---|---|---|---|---|
| F1 (CRITICAL) | **F1.a (aligned)** | Single CH-27 chunk; ship all 5 deliverables in one cycle | M5.3 carve-out closes in 3 chunks; coherent ADR-0062; M6 unblocks promptly | Aggregate ~8-15 ed must respect pause-disciplines | Aligned: rare in F1-class forks |
| F3 (HIGH stakes) | **F3.a (aligned)** | Defer `projects::resolvers::*` actor-passthrough to M6-DEFERRED via NEW drift D-CH27-FOLLOWUP-01 | Architectural design moved to M6 with proper blast-radius; CH-27 ships 4-of-5 deliverables | M5.3 carve-out closes with 1-of-5 forward-scope deliverable deferred | Aligned with rec |
| F4 (MEDIUM stakes) | **F4.b USER-DIVERGENT** | NEW opt-in helper `seed_owner_grants(agent, [org_ids])` at `server/tests/acceptance_common/owner_grants.rs` (~80 LOC) | Wire-format-explicit per-test owner-grant seeding visible in test body; easier to audit; canonical pattern for M6+ admin tests | +1-2 ed cascade through ~12-18 tests (planning band; actual landed: **9 call-sites** — see §D62.4 cascade-collapse note) | **DIVERGENT** — 11-of-16 cumulative; pattern softening |
| Count-amend (META) | **Aligned** | Amend D-CH26-FOLLOWUP-01 body "15" → "7" with footnote at P-SEAL | Drift body reflects actual cascade enumeration | None | Aligned with rec |

---

## Context

CH-26 (`d1cb9e1f`) closed `D-philosophy-02` (HIGH-A) at the **load-bearing semantic axis**: the Permission Check engine resolves `org:O` + `project:P` selector matches for owner-Agents via the CH-25 synth-owner-grant rule (`step_2_resolve_grants`), and 10 acceptance scenarios at `acceptance_m5_3_composite_resources.rs` pin the invariant green. CH-26 **deferred the wire-tier axis to CH-27 per user routing 2026-05-16** (kept in M5 carve-out per user direction, NOT M6+):

- 7 admin handlers in `orgs::{list,show,create,dashboard}` + `projects::{create,detail,agent_supervisor}` carry `check_permission(...).is_ok()` ADVISORILY at CH-26 close.
- Bespoke `AuthenticatedSession` + role-check + AR-filter gates remain the HTTP-tier rejection surface at CH-26.
- 2 acceptance scenarios pinned at `Action::Allocate` per synth-grant scope constraint (current scope `[Allocate, Transfer]` per ADR-0060 §D60.2); the natural verbs (Observe/Inspect) for show/dashboard handler paths are out-of-scope at the synth-grant tier.
- `projects::resolvers::*` background trait impls (`AdoptionArResolver`, `ActorResolver`, `TemplateCAdoptionArResolver`, `TemplateDAdoptionArResolver` at `domain/src/events/listeners.rs:244-507`) carry no actor parameter on the resolver trait shape — `check_permission` wiring requires architectural design exceeding the M5.3 carve-out envelope.

Drift [`D-CH26-FOLLOWUP-01`](../drifts/D-CH26-FOLLOWUP-01.md) (LOW, Bucket B) captures all 4 of these axes for CH-27 closure. The drift body claims "15 advisory check_permission invocations" but the verified §3 Artifact A cascade enumerates exactly **7** invocations (one per handler — the original "15" count conflated invocations + use imports + docstring references).

CH-27 closes D-CH26-FOLLOWUP-01 by:
1. **§D62.1** — Wire-tier tightening: flip `.is_ok()` advisory pattern → `denial_to_api_error` blocking pattern at all 7 handlers.
2. **§D62.2** — Synth-grant rule widening: extend action set from `[Allocate, Transfer]` to `[Allocate, Transfer, Observe, Inspect]` for owner-Agents.
3. **§D62.3** — F3.a (LOCKED): defer resolvers wiring to M6-DEFERRED via NEW drift `D-CH27-FOLLOWUP-01`.
4. **§D62.4** — F4.b USER-DIVERGENT: ship NEW opt-in helper `seed_owner_grants(agent, [org_ids])` at `server/tests/acceptance_common/owner_grants.rs`; planning band was 12-18 acceptance tests; **actual landed cascade is 9 call-sites** (test bodies semantically unchanged; cascade-collapse explained in §D62.4 body — tests using the `apply_org_creation` production path obtain Edge::Owns implicitly via CH-25 ADR-0060 §D60.1).
5. **§D62.5** — Count-amend META: D-CH26-FOLLOWUP-01 body "15" → "7" with footnote citing §3 Artifact A enumeration.
6. **§D62.6** — Pre-existing-behaviour preservation note per chunk-planner v11.

**F4.b USER-DIVERGENT precedent**: cross-cycle pattern of F<X>.b expansion-divergence at gate-1 — CH-26 F2.b (tag-field-on-struct over catalogue-entry-only) + CH-25 F1.b (NEW `Edge::Owns` variant over re-use-existing) + i-phi CH-02b F4.b (thiserror enum over String) + i-phi CH-02a F5.b. User systematically prefers wire-format-explicit + opt-in-visible options at gate-1; the cumulative rate stepped down from ~83% → ~69% in CH-27 (3 aligned + 1 divergent) indicating planner-recommendation acceptance is rising at the mid-band.

---

## Sub-decisions

### §D62.1 — Wire-tier tightening: `.is_ok()` advisory → `denial_to_api_error` blocking (F1.a LOCKED)

**Decision**: Flip the consumption pattern at all 7 advisory `check_permission` invocations identified in plan §3 Artifact A:

```rust
// Before (CH-26 advisory):
let engine_allowed = check_permission(&ctx, &manifest, &NoopMetrics).is_ok();
// (bespoke gate decides response)

// After (CH-27 blocking):
match check_permission(&ctx, &manifest, &NoopMetrics) {
    Ok(()) => { /* proceed to bespoke gate as defence-in-depth */ }
    Err(PermissionCheckError::Denied { failed_step, reason }) => {
        return Err(denial_to_api_error(failed_step, &reason).into());
    }
    Err(other) => return Err(other.into()),  // already wired
}
```

**7-handler enumeration**:

| File | Line | Existing call shape | New shape |
|---|---|---|---|
| `orgs/list.rs` | 157 | `check_permission(&ctx, &manifest, &NoopMetrics).is_ok()` (per-row filter) | per-row blocking-filter via `denial_to_api_error` |
| `orgs/show.rs` | 174 | `Ok(check_permission(&ctx, &manifest, &NoopMetrics).is_ok())` (advisory wrap) | `?` propagation via `denial_to_api_error` |
| `orgs/create.rs` | 424 | `Ok(check_permission(&ctx, &manifest, &NoopMetrics).is_ok())` | `?` propagation |
| `orgs/dashboard.rs` | 447 | `Ok(check_permission(&ctx, &manifest, &NoopMetrics).is_ok())` | `?` propagation |
| `projects/create.rs` | 266 | `Ok(check_permission(&ctx, &manifest, &NoopMetrics).is_ok())` | `?` propagation |
| `projects/detail.rs` | 316 | `Ok(check_permission(&ctx, &manifest, &NoopMetrics).is_ok())` | `?` propagation |
| `projects/agent_supervisor.rs` | 193 | `Ok(check_permission(&ctx, &manifest, &NoopMetrics).is_ok())` | `?` propagation |

**Rationale**: closes wire-tier axis of D-CH26-FOLLOWUP-01; honors `permissions/README.md` entry invariant that Permission Check is source of truth for resource authorization at admin-handler tier.

**Pre-existing-behaviour preservation note**: *"Pre-existing CH-26 advisory consumption preserved at handlers prior to CH-27; CH-27 tightens to blocking; bespoke gate remains as defence-in-depth post-engine-allow; shipped at CH-27 P-SEAL date 2026-05-18."*

### §D62.2 — Synth-owner-grant rule widening to 4-verb scope (F1.a LOCKED)

**Decision**: Extend the synth-grant action set at `domain/src/permissions/engine.rs:279` (in `synth_owner_grant` fn body):

```rust
// Before (CH-25, ADR-0060 §D60.2):
action: vec![Action::Allocate, Action::Transfer],

// After (CH-27):
action: vec![Action::Allocate, Action::Transfer, Action::Observe, Action::Inspect],
```

Owner-Agents on owned Org/Project now carry **all 4 universal-applicability verbs** that the concept-doc `permissions/03-action-vocabulary.md:44` enumerates (Authority `Allocate` + `Transfer`; Discovery `Inspect`; Observability `Observe`).

**Test impact**: existing tests at `engine.rs:2573, 2614` update from 2-verb to 4-verb array literals + scenario rename (`_allocate_transfer_candidate` → `_authority_and_observability_candidate`). NEW engine tests:
- `owner_grant_synth_observe_on_owned_org_allows`
- `owner_grant_synth_inspect_on_owned_org_allows`
- `owner_grant_synth_observe_on_owned_project_allows`
- `owner_grant_synth_inspect_on_owned_project_allows`

**Closed-set invariant preserved**: `Action::CANONICAL.len() == 34` PRESERVED (no new Action variants); Observe + Inspect already extant per `action.rs:241, 253, 282`.

**Pre-existing-behaviour preservation note**: *"Pre-existing CH-25 synth-grant scope `[Allocate, Transfer]` preserved as the M5.3-baseline; CH-27 widens to `[Allocate, Transfer, Observe, Inspect]` for owner-Agents on owned Org/Project; shipped at CH-27 P-SEAL date 2026-05-18."*

### §D62.3 — Resolvers wiring deferred to M6 (F3.a LOCKED, aligned)

**Decision**: `projects::resolvers::*` actor-passthrough wiring is **deferred to M6-DEFERRED-RESOLVERS-WIRING** via NEW drift `D-CH27-FOLLOWUP-01`.

**Rationale**: the resolver trait shapes at `domain/src/events/listeners.rs:244-507` are background fire-listener traits with `resolve(&self, project: ProjectId) -> Option<...>` / `resolve(&self, org: OrgId) -> Option<AgentId>` signatures — **no actor parameter**. Wiring `check_permission` requires architectural design exceeding the M5.3 carve-out blast-radius envelope:

- F3.b path (extend trait with `Option<AgentId>`) cascades through 4 trait defs + 4 `Repo*Resolver` impls + ~6 static-test stubs + 1 production wiring site.
- F3.c path (HTTP-tier wrapper) introduces ~50 LOC duplication + future-convergence risk.

M6 plan-open will design the actor-passthrough architecture coherently across all background-listener trait shapes (not just `projects::resolvers::*`).

**Pre-existing-behaviour preservation note (deferred-scope variation)**: *"Pre-existing scaffold preserved: `projects::resolvers::*` background trait shape unchanged at CH-27 close (`AdoptionArResolver`, `ActorResolver`, `TemplateCAdoptionArResolver`, `TemplateDAdoptionArResolver` at `domain/src/events/listeners.rs:244-507`). CH-27 ratifies the deferral via D-CH27-FOLLOWUP-01 with `M6-DEFERRED-RESOLVERS-WIRING` allocation; does not implement the actor-passthrough design."*

### §D62.4 — F4.b USER-DIVERGENT: NEW opt-in helper `seed_owner_grants` (DIVERGENT)

**Decision (USER-DIVERGENT from planner-recommended F4.a default-extension)**: Ship a NEW opt-in test-helper `seed_owner_grants(agent, [org_ids])` at `server/tests/acceptance_common/owner_grants.rs` (~80 LOC) consumed explicitly by acceptance tests across M3 + M4 + M5 suites that exercise the now-blocking 7 admin handlers. Plan §3 Artifact C predicted 12-18 fixture-extension sites; **the actual landed cascade is 9 call-sites across 6 test files** (`acceptance_m3.rs`, `acceptance_m4.rs`, `acceptance_m5_orgs.rs`, `acceptance_orgs_create.rs`, `dashboard_silent_filter_test.rs`, `show_organization_post_filter_test.rs`). **Cascade-collapse rationale**: many predicted test sites obtain owner-status via the production `apply_org_creation` / `bootstrap_org_via_wizard` path, which CH-25 ADR-0060 §D60.1 makes emit `Edge::Owns` as production behaviour — the synth-owner-grant rule covers those tests implicitly at engine `step_2_resolve_grants` without needing an explicit per-test seed call. The 9 sites with explicit `seed_owner_grants` calls are the test scenarios that hand-craft Org/Project nodes bypassing the production compound-tx (where the implicit production-path emission does not fire). All M3+M4+M5 carry-forward suites are green at chunk-close (per Audit C verdict PASS).

**Helper signature**:
```rust
pub async fn seed_owner_grants(
    repo: &impl Repository,
    agent: AgentId,
    org_ids: Vec<OrgId>,
) -> Result<(), StoreError> {
    for org_id in org_ids {
        repo.insert_edge(Edge::Owns { from: agent.clone(), to: org_id.into() }).await?;
    }
    Ok(())
}

// Optional variant for non-owner-grant explicit-grant test scenarios:
pub async fn seed_owner_grants_with_explicit_grants(
    repo: &impl Repository,
    agent: AgentId,
    grants_to_seed: Vec<(OrgId, Vec<Action>)>,
) -> Result<(), StoreError> { ... }
```

**SCOPE-NARROWING note (CH-27 P-FIXTURES — deviation #2 from plan §3 Artifact C)**: the helper signature literal above was the plan's body shape; the **shipped** helper at `server/tests/acceptance_common/owner_grants.rs` substitutes `Repository::create_grant` for the plan's `Repository::insert_edge` because **the `Repository` trait does not expose an `insert_edge` method** (verified at `domain/src/repository.rs:796` — `create_grant` is canonical; `insert_edge` does not exist on the trait). The shipped helper materialises an explicit `Grant { holder: agent, resource_uri: "organization:<id>"/"project:<id>", action: vec![Allocate, Transfer, Observe, Inspect], ... }` via `repo.create_grant(&grant).await?` for each (agent, resource_id) pair, preserving F4.b's **wire-format-explicit per-test seeding spirit** (the grant materialisation is auditable at the test-site, fully equivalent to the synth-owner-grant rule's runtime behaviour at engine `step_2_resolve_grants` per ADR-0060 §D60.2). Parallel documentation of this SCOPE-NARROWING lives at `owner_grants.rs:36-53` (helper file doc-comment) + `composite-resources-model.md` §"Test-fixture pattern for owner-grant-required tests" (body-level paragraph). The SCOPE-NARROWING does NOT change F4.b's locked decision — only the implementation idiom.

**Per-test wire-up pattern**:
```rust
// Before (CH-26):
let org = apply_org_creation(&repo, ...).await?;
// (test exercises handler — relied on implicit Edge::Owns from apply_org_creation)

// After (CH-27 F4.b):
let org = apply_org_creation(&repo, ...).await?;
seed_owner_grants(&repo, ceo.id.clone(), vec![org.id.clone()]).await?;
// (test exercises handler — explicit owner-grant wire-up visible)
```

**Rationale for F4.b over planner-recommended F4.a**: cross-cycle user-preference for **wire-format-explicit** + **opt-in-visible** fixture patterns. F4.a default-extension (rely on `apply_org_creation` emitting Edge::Owns implicitly, since CH-25 ADR-0060 §D60.1 ships that emission as production behaviour) would have made the owner-grant wire-up invisible at test-site (advisory of CH-26 → blocking of CH-27 transition silently absorbed). F4.b's explicit per-test call makes the owner-grant seeding **audit-visible** and provides a canonical pattern for M6+ admin-page tests.

**Cross-cycle precedents**:
- **CH-26 F2.b** (tag-field-on-struct over catalogue-entry-only) — same wire-format-explicit preference.
- **CH-25 F1.b** (NEW `Edge::Owns` variant over canonical-resource-flip) — same explicit-state preference.
- **i-phi CH-02b F4.b + F-error.b** (thiserror enum over String) — same explicit-typing preference.

**Per chunk-planner v13 R3 dep-direction verification**: helper lives in `server::tests::acceptance_common` (test-only scope); consumes `domain::repository::Repository` via test fixtures. Dep direction `server → domain` is canonical (no circular dep). No phi-core import.

**Pre-existing-behaviour preservation note**: *"Pre-existing CH-26 acceptance fixtures preserved with semantically-unchanged test bodies; CH-27 adds explicit `seed_owner_grants(ceo, [org_id])` calls at touched test sites; shipped at CH-27 P-SEAL date 2026-05-18."*

### §D62.5 — Count-amend META: D-CH26-FOLLOWUP-01 body "15" → "7"

**Decision (META)**: Amend the body of drift [`D-CH26-FOLLOWUP-01`](../drifts/D-CH26-FOLLOWUP-01.md) at P-SEAL to correct cardinality:

- Body at line ~24 "15 advisory `check_permission` invocations" → "**7** advisory `check_permission` invocations" with explicit footnote citation to plan §3 Artifact A enumeration (the 7 handler files at `orgs/{list,show,create,dashboard}.rs` + `projects/{create,detail,agent_supervisor}.rs`).

**Root cause of original "15" count**: the original CH-26 chunk-seal report conflated **invocations + use imports + docstring references**. The verified §3 Artifact A grep (`git grep -nE 'check_permission\(&ctx' modules/crates/server/src/platform/orgs/ modules/crates/server/src/platform/projects/`) returns exactly **7** hits — one production-call per handler.

**Pre-existing-behaviour preservation note (META variation)**: *"Pre-existing drift body preserved with footnote-corrected cardinality; no semantic claim is changed by the amendment — the drift's invariants + remediation scope are independent of the "15 vs 7" claim; shipped at CH-27 P-SEAL date 2026-05-18."*

### §D62.6 — Pre-existing-behaviour preservation note umbrella per chunk-planner v11

Each §D62.<M> above carries its own Pre-existing-behaviour preservation note in the appropriate form:
- §D62.1 strict (current-CH ship + bespoke gate as defence-in-depth)
- §D62.2 strict (current-CH widen owner-grant scope)
- §D62.3 deferred-scope variation (current-CH ratifies deferral, does not implement)
- §D62.4 strict (current-CH ships F4.b helper with explicit seeding)
- §D62.5 META variation (footnote-corrected cardinality)

---

## Cross-references

**Concept docs**:
- [`permissions/04-manifest-and-resolution.md`](../../../concepts/permissions/04-manifest-and-resolution.md) §"Owner-grant auto-issue rule (CH-25 / ADR-0060)" — synth-grant scope widening per §D62.2.
- [`permissions/03-action-vocabulary.md`](../../../concepts/permissions/03-action-vocabulary.md) line 44 — universal-applicability claim now honored at owner-tier for owned Org/Project.
- [`core-philosophy.md`](../../../concepts/core-philosophy.md) lines 16, 28, 29 — Org-has-Projects + Org/Project own Resources honored across both load-bearing semantic + wire-tier axes.
- [`permissions/README.md`](../../../concepts/permissions/README.md) entry invariants — Permission Check is source of truth at admin-handler tier (blocking).

**Closed drift**:
- [`D-CH26-FOLLOWUP-01`](../drifts/D-CH26-FOLLOWUP-01.md) — wire-tier tightening + synth-grant widening + acceptance-fixture extension via F4.b opt-in helper; resolvers wiring deferred per F3.a lock with NEW drift D-CH27-FOLLOWUP-01.

**Prior ADRs cited**:
- [`m5_3/0060-agent-as-creator-and-owner.md`](0060-agent-as-creator-and-owner.md) §D60.1 + §D60.2 + §D60.3 — synth-grant rule baseline + scope baseline + production Edge::Owns emission at `apply_org_creation`.
- [`m5_3/0061-org-project-as-composite-resources.md`](0061-org-project-as-composite-resources.md) §D61.5 + §D61.7 — advisory-only-revision rationale + CH-27 carve-out routing + **F2.b USER-DIVERGENT precedent** (cited for F4.b lock).
- [`m1/0008-permission-check-as-pipeline.md`](../../m1/decisions/0008-permission-check-as-pipeline.md) — 6-step pipeline invariants preserved through Step 2 widening.
- [`m2/0018-handler-support-module.md`](../../m2/decisions/0018-handler-support-module.md) — `denial_to_api_error` precedent at `handler_support::permission`.

**Forward-scope row**:
- [`forward-scope/remaining-scope-post-m5-p7-22035b2a.md`](../../../../plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md) §2.5 lines 277-289 (CH-27 row + M5.3 carve-out arc).

**NEW drift filed at P-SEAL**:
- `D-CH27-FOLLOWUP-01` — `Impl chunk = M6-DEFERRED-RESOLVERS-WIRING`; resolver actor-passthrough architectural design + wiring at M6 plan-open.

---

## Consequences

### For M5.3 closure

M5.3 carve-out closes with 3-chunk arc {CH-25, CH-26, CH-27}. M6 plan-open unblocks **after CH-27 close** (per forward-scope §2.5 post-M5.3-actions paragraph at line 293). All 4 drifts at M5.3 closed:
- D-philosophy-01 (CH-25 ✓)
- D-philosophy-02 (CH-26 ✓ at load-bearing semantic axis)
- D-CH26-FOLLOWUP-01 (CH-27 ✓ at wire-tier + synth-grant-widening + fixture-extension axes)
- D-CH27-FOLLOWUP-01 (NEW, M6-DEFERRED, filed at CH-27 P-SEAL)

### For M6 plan-open

M6 inherits the resolver-actor-passthrough scope via D-CH27-FOLLOWUP-01. M6 plan-mode opens with:
- Resolver trait re-shape design (the 4 background-listener traits get actor-passthrough as part of M6+ background-tier governance design — not specifically just `projects::resolvers::*`).
- F4.b canonical helper `seed_owner_grants` ready for M6 admin-page tests.

### For future M7+ admin pages

The blocking-gate consumption pattern (`denial_to_api_error` via `?` propagation) is now **canonical**. The advisory→blocking handoff is retired post-CH-27. New admin handlers ship blocking from day 1.

`seed_owner_grants` is the canonical test-fixture pattern for owner-grant-required tests — M7+ admin handlers needing owner authorization follow the same explicit-seeding wire-up.

---

## Revisit triggers

- **§D62.1**: a new admin handler is added that needs `check_permission` wiring → revisit the 7-handler enumeration table; the blocking-gate pattern applies but cardinality flips.
- **§D62.2**: synth-grant scope further widened (e.g., a new `Action::Audit` variant added to canonical set) → revisit the action-set vec literal at `engine.rs:279`.
- **§D62.3**: M6 admin pages open AND resolver wiring still deferred → revisit D-CH27-FOLLOWUP-01 closing chunk decision.
- **§D62.4**: fixture-extension cascade hits ≥ 22 sites mid-cycle (F4.b pause-trip) OR a new owner-grant-required test category emerges that `seed_owner_grants` doesn't cover (e.g., delegated-grant test scenarios) → extend the helper or add a sibling helper.
- **§D62.5**: drift body cardinality assertions need re-cross-checking after CH-27 close (e.g., a new D-CHK-MULTIPLE drift cites the wrong cardinality).
- Gate-2.5 Candidates 3 + 4 (bespoke-gate dead-code + per-handler audit-event emission): if surfaced during M6+ admin-handler work → file follow-up drifts `D-CH27-FOLLOWUP-02` (`M6-DEFERRED-BESPOKE-GATE-CLEANUP`) + `D-CH27-FOLLOWUP-03` (`M6-DEFERRED-AUDIT-HANDLER-TIER`).

---

## Verification

Commands the reviewer replays:

```bash
# Wire-tier tightening — expect ≥ 7 hits
git -C /root/projects/phi/baby-phi grep -nE 'denial_to_api_error' modules/crates/server/src/platform/orgs/ modules/crates/server/src/platform/projects/

# Synth-grant widening — expect 1 hit at synth_owner_grant fn body
git -C /root/projects/phi/baby-phi grep -nE 'Action::Allocate, Action::Transfer, Action::Observe, Action::Inspect' modules/crates/domain/src/permissions/engine.rs

# F4.b helper extant — expect ≥ 1 hit
git -C /root/projects/phi/baby-phi grep -nE '^pub (async )?fn seed_owner_grants' modules/crates/server/tests/acceptance_common/

# F4.b call-site count — expect 9 hits (CH-27 actual landed; planning band was 12-18; see §D62.4 cascade-collapse note)
git -C /root/projects/phi/baby-phi grep -nE 'seed_owner_grants\(' modules/crates/server/tests/ | wc -l

# F4.b test-only scope — expect 0 hits outside tests/ tree
git -C /root/projects/phi/baby-phi grep -rnE 'seed_owner_grants' modules/crates/ | grep -v 'modules/crates/server/tests/' | wc -l

# Carry-forward invariants — expect 3 hits (Composite 10 + EDGE_KIND_NAMES 72 + Action::CANONICAL 34)
git -C /root/projects/phi/baby-phi grep -nE 'Composite::ALL.*10|EDGE_KIND_NAMES.*72|Action::CANONICAL.*34' modules/crates/domain/src/

# phi-core import baseline — expect 57
grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l

# Workspace test count — expect within band [1570, 1574]
/root/rust-env/cargo/bin/cargo test --manifest-path /root/projects/phi/baby-phi/Cargo.toml --workspace 2>&1 | tail -10

# D-CH26-FOLLOWUP-01 cardinality amendment — expect ≥ 1 hit (the "7 advisory" phrase)
grep -nE '7 advisory check_permission invocations' /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_3/drifts/D-CH26-FOLLOWUP-01.md
```
