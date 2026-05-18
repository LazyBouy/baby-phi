<!-- Last verified: 2026-05-18 by Claude Code (CH-27-0edcaba9 plan v2 re-plan after gate-1 user-lock divergent F4.b (Opt-in helper seed_owner_grants over planner-recommended F4.a Default-extension); F1.a + F3.a + count-amend all aligned) -->

# CH-27 plan — Blocking-gate enforcement + resolvers wiring + acceptance fixture extension (cycle hex `0edcaba9`)

Forward-scope: [§2.5 lines 277-289](../../forward-scope/22035b2a-remaining-scope-post-m5-p7.md). Drift closed: [`D-CH26-FOLLOWUP-01`](../../../v0/implementation/m5_3/drifts/D-CH26-FOLLOWUP-01.md) (LOW). Next ADR: **ADR-0062**.

> **v2 re-plan banner (2026-05-18)**: gate-1 user-locks applied. F1.a (Single CH-27) ✓ aligned. F3.a (Defer resolvers wiring to M6) ✓ aligned. **F4.b (Opt-in helper `seed_owner_grants`) ✅ USER-DIVERGENT** (planner-recommended F4.a Default-extension). Count-amend (15 → 7) at P-SEAL ✓ aligned. All v1 content preserved; v2 additions follow chunk-planner v9 surfacing-not-suppressing.

---

## Forks for orchestrator

> ⚠️ **CROSS-CYCLE DIVERGENCE PATTERN**: planner-recommendation has diverged from user-lock in **11 of last 16 cycles (~69% cumulative; pattern SOFTENING from prior ~83%)** — divergent: CH-15 `c3f46f17` F5.B / CH-17 `40c4d759` F5.B / CH-18 `c77937bc` F3.B / CH-20 `240616a4` F1.B / CH-24 `5778bb77` F1.B + F-D59.2.b + F-D59.3.b / CH-25 `1e01618e` F1.b / CH-26 `d1cb9e1f` F1.b + F2.b / i-phi CH-02a `1bd3bdd1` F5.b / CH-02b `57b20bda` F4.b + F-error.b / **CH-27 `0edcaba9` F4.b (Opt-in helper `seed_owner_grants`)**. Non-divergent: CH-19 `2c520ba7` + i-phi CH-01 `95c96df7` + **CH-27 F1.a + F3.a + count-amend (3-of-4 aligned)**. The cumulative rate stepped DOWN from ~83% → ~69% in CH-27 (3 aligned + 1 divergent in single cycle), indicating **increasing planner-recommendation acceptance** at the mid-band. The consistent F<X>.b expansion-divergence still persists but at lower per-cycle rate. User systematically prefers wire-format-explicit + opt-in-visible options at gate-1. **Treat divergence as still-the-modal outcome for tighter-scope forks, but planner-rec acceptance is rising.** v13 divergence-aware framing remains applied to all forks below.

### F1 — Scope split: single chunk vs CH-27a + CH-27b? (CRITICAL — v18 R1 cost-framework escalation)

v18 R1 (handler-refactor CheckContext-build cost addendum) prescribes ~1.5-2 ed/handler when refactoring from bespoke-gated baseline to engine-invoking. **CRITICAL CLARIFICATION**: at CH-26 close, the **CheckContext-build cost is ALREADY PAID** — all 7 handlers carry full `CheckContext { ... } + Manifest { ... }` construction (verified: `orgs/list.rs:128-151`, `orgs/show.rs:145-168`, `orgs/create.rs:`+`orgs/dashboard.rs:`+`projects/{create,detail,agent_supervisor}.rs`). What CH-27 must change is the **consumption pattern only**: `check_permission(&ctx, &manifest, &NoopMetrics).is_ok()` → `?`-propagation via `denial_to_api_error`. Per-handler cost drops to ~0.3-0.5 ed (consumption flip + response-status audit + bespoke-gate-shadow simplification).

Adjusted v18 R1 estimate breakdown:
- Handler consumption-flip (7 handlers × ~0.3-0.5 ed): ~2-3.5 ed (NOT ~10-14 ed — CheckContext already paid)
- Synth-grant widening (single 1-line array edit + ~4 engine tests): ~1-1.5 ed
- Resolvers wiring (architectural — see F3): ~1-4 ed depending on F3 lock
- Acceptance fixture extension (M3+M4+M5 suites; predicted ~12-18 tests need Edge::Owns seeding via `seed_owner_grants`): ~3-5 ed under F4.b (opt-in helper, +1-2 ed over F4.a default-extension)
- Re-enable + rename 2 advisory-only scenarios (~0.5 ed)
- ADR-0062 + drift housekeeping (~0.5-1 ed)

**Aggregate**: ~8-15 ed under F4.b lock (vs forward-scope ~6-10 ed). Forward-scope's estimate sits at upper-band ceiling with F4.b's +1-2 ed delta. Pause discipline tightens accordingly (see Per-fork pause-threshold table).

| Option | Path | Aggregate ed | Pros | Cons |
|---|---|---|---|---|
| **F1.a — Locked at gate-1** | Single CH-27 chunk; ship all 5 deliverables in one cycle | ~8-15 ed (under F4.b lock) | M5.3 carve-out closes in 3 chunks per user intent 2026-05-16; coherent ADR-0062 covers the full wire-tier story; M6 plan-open unblocked promptly | Aggregate hits 1.5× forward-scope upper if pause-discipline fails to fire; large phase count + 3-auditor envelope |
| F1.b (split — only if scope exceeds 12 ed mid-cycle) | CH-27a (handler-flip + synth-grant + tests-renamed) ~4-5 ed; CH-27b (fixture extension via opt-in helper + ADR consolidation) ~6-9 ed | ~10-14 ed | Each chunk fits ~5 ed envelope; fixture-extension cascade gets isolated audit | M5.3 carve-out extends 3 → 4 chunks (forward-scope amendment); doubles plan + audit overhead; orchestrator-side coordination cost |

**Locked at gate-1: F1.a (Single CH-27)** ✓ aligned with planner-rec. Rationale: CheckContext-build cost is pre-paid at CH-26, dropping v18 R1's predicted ~10-14 ed/handler-refactor band to ~2-3.5 ed; aggregate ~8-15 ed is within forward-scope upper-band PLUS 1.5× pause-discipline ceiling per cascade rule (under F4.b lock the upper band needs 12 ed pause-trip, refined from v1's 11 ed). Plan §7 P-PAUSE-CHECK triggers if aggregate exceeds 12 ed at mid-cycle (F1.b fallback).

### F3 — Resolvers wiring approach (HIGH stakes — architectural)

The `projects::resolvers::*` traits (`AdoptionArResolver`, `ActorResolver`, `TemplateCAdoptionArResolver`, `TemplateDAdoptionArResolver` at `domain/src/events/listeners.rs:244-507`) are **background fire-listeners** with NO actor parameter — `resolve(&self, project: ProjectId) -> Option<...>` / `resolve(&self, org: OrgId) -> Option<AgentId>`. The drift's "no actor parameter" framing is correct. Wiring `check_permission` requires one of:

| Option | Path | Cost | Pros | Cons |
|---|---|---|---|---|
| **F3.a — Locked at gate-1** | **DEFER resolvers wiring to M6-DEFERRED-NEW** (file new drift; document deferral rationale in ADR-0062 §META) | ~0 ed in-cycle; +0.5 ed drift filing | Surface architectural mismatch as M6 carve-out scope (cleaner blast-radius isolation); CH-27 ships 4-of-5 deliverables; honors v18 R1 cost framework realistically | M5.3 carve-out closes with 1-of-5 forward-scope deliverable formally deferred — user routing 2026-05-16 may have anticipated full closure |
| F3.b | Extend resolver traits with optional `actor: Option<AgentId>` parameter; wire `check_permission` on the read-tier when actor is Some; tolerate None (current background-fire pattern preserved) | ~3-4 ed | Closes deliverable 3 fully in-cycle; resolver layer becomes uniformly actor-aware | Trait-shape change cascades through 4 trait defs + 4 `Repo*Resolver` impls + 1 production wiring site + ~6 static-test stubs; mid-cycle architectural risk |
| F3.c | Reframe deliverable 3: ship a NEW HTTP read-tier helper that wraps the resolver + adds the gate-check on the HTTP path; leave background fire-listener resolvers actor-blind | ~1.5-2 ed | Clean separation: background tier stays background; HTTP tier gets the gate via a thin wrapper | Adds a new helper (~50 LOC); slight code-surface duplication; M6 will need to converge back if/when admin pages need same path |

**Locked at gate-1: F3.a (Defer to M6-DEFERRED)** ✓ aligned with planner-rec. Rationale: the drift body explicitly says "*projects::resolvers::* background trait impls NOT wired through `check_permission` — no actor parameter on resolver trait shape; awaits actor-passthrough design*" — actor-passthrough design IS an architectural decision exceeding M5.3 carve-out blast-radius envelope. M6 plan-open will design this coherently across all background-listener trait shapes. NEW drift `D-CH27-FOLLOWUP-01` filed at P-SEAL with explicit `M6-DEFERRED-RESOLVERS-WIRING` allocation.

### F4 — Acceptance fixture extension default vs opt-in (MEDIUM stakes — USER-DIVERGENT lock)

The acceptance fixture extension question: do we extend `acceptance_common::admin::spawn_claimed_with_org` to seed Edge::Owns BY DEFAULT (zero-config for callers) OR add a NEW opt-in helper `seed_owner_grants(...)`?

| Option | Path | Cost | Pros | Cons |
|---|---|---|---|---|
| F4.a (planner-recommended; **NOT locked**) | DEFAULT extension: `spawn_claimed_with_org` already calls `apply_org_creation` (verified `admin.rs:346`) — CH-25 ADR-0060 §D60.1 already emits Edge::Owns at `apply_org_creation` AS production behaviour. **Edge::Owns IS already seeded by default**; the fixture extension is to verify no test bypasses `apply_org_creation` to hand-craft an Org without Edge::Owns | ~0 ed cascade; ~1-2 ed verification scan + targeted fixture amendments | Honors CH-25's principle that Edge::Owns is intrinsic to org-creation; tests that break are tests that hand-craft without compound-tx (a smell to fix anyway) | Predicting which tests hand-craft requires grep + read pass; implicit per-test wiring is less audit-visible |
| **F4.b — Locked at gate-1 (USER-DIVERGENT)** | **OPT-IN: NEW helper `seed_owner_grants(agent, [org_ids])` at `server/tests/acceptance_common/`**; the helper emits Edge::Owns for each (agent, org_id) pair, forces the engine's synth-owner-grant rule into the manifest, and may accept an optional `[grants_to_seed]` param for explicit non-owner-grant scenarios; identified M3+M4+M5 acceptance tests get explicit `seed_owner_grants(ceo, [org_id])` call (or equivalent explicit-grant seeding) | ~3-5 ed (helper ~80 LOC + per-test explicit seeding cascade ~12-18 tests) | **Explicit per-test wire-up of owner-grant seeding visible in test body**; easier to audit; wire-format-explicit per cross-cycle user preference; no implicit fixture coupling | Cascade through ~12-18 acceptance test files; deliberate-edit count compounds; +1-2 ed over F4.a |

**Locked at gate-1 (USER-DIVERGENT): F4.b — Opt-in helper `seed_owner_grants`**. Cross-cycle pattern: user prefers wire-format-explicit options at gate-1; CH-26 F2.b (tag-field-on-struct + backfill) precedent + CH-02b-i-phi F-error.b (thiserror enum) precedent + CH-25 F1.b (canonical-resource-flip) precedent — F<X>.b expansion-divergence is structurally durable (now 9 cycles: baby-phi CH-15/17/18/20/24/25 + CH-26 F1.b/F2.b + i-phi CH-02a F5.b + CH-02b F4.b+F-error.b + **CH-27 F4.b**).

### Count fix (15 → 7)

**Locked at gate-1: Amend at P-SEAL** ✓ aligned. D-CH26-FOLLOWUP-01 body currently reads "15 advisory check_permission invocations" but the verified §3 Artifact A cascade enumerates exactly **7** invocations across the 7 handlers. P-SEAL deliverable amends drift body inline: "15" → "7 advisory check_permission invocations" with footnote citing the v1 cascade enumeration. Tracked as §D62.5 META amendment.

---

## §1 — Context & principle

**Why this chunk**: CH-26 closed D-philosophy-02 at the **load-bearing semantic axis** (engine matches `org:O` + `project:P` selectors for owner-Agents via the CH-25 synth-owner-grant rule; 10 acceptance scenarios green) but DEFERRED the **wire-tier axis** to CH-27 per user routing 2026-05-16 (kept in M5 carve-out, NOT M6+). At CH-26 close, the 7 admin handlers in `orgs::{list,show,create,dashboard}` + `projects::{create,detail,agent_supervisor}` carry `check_permission(...).is_ok()` ADVISORILY — the bespoke `AuthenticatedSession` + role-check + AR-filter gates remain the HTTP-tier rejection surface. CH-27 closes drift `D-CH26-FOLLOWUP-01` (LOW) by tightening to blocking gates via `denial_to_api_error` (CH-25 wire convention), extending the synth-owner-grant rule to cover `Action::Observe` + `Action::Inspect` for owner-Agents, extending M3+M4+M5 acceptance fixtures via NEW opt-in helper `seed_owner_grants` (F4.b user-lock), and re-enabling 2 advisory-only-renamed scenarios under their original verbs.

**Quality-over-speed restatement**: *"Concept docs are source-of-truth; implementation aligns to them. Drift is discovered, documented, and planned-through — never accumulated silently."* — CH-27 closes the deliberately-scoped CH-26 follow-up at the wire-tier axis with full acceptance + audit coverage before M5.3 carve-out closes + M6 plan-mode opens.

**Forward-scope reference**: [§2.5 lines 277-289](../../forward-scope/22035b2a-remaining-scope-post-m5-p7.md) (CH-27 row; M5.3 carve-out extension).

---

## §2 — Concept alignment walk

| Concept doc | § anchor | Claim (close paraphrase) | Status at chunk-open | Target status at chunk-close |
|---|---|---|---|---|
| [`permissions/04-manifest-and-resolution.md`](../../../v0/concepts/permissions/04-manifest-and-resolution.md) | §"Owner-grant auto-issue rule (CH-25 / ADR-0060)" line ~ | "Owner-grant synthesised for owned Org/Project URIs carries `[Allocate, Transfer]`" | partially-honored (covers Authority verbs only; not Discovery/Observability) | honored (extended to `[Allocate, Transfer, Observe, Inspect]`) |
| [`permissions/03-action-vocabulary.md`](../../../v0/concepts/permissions/03-action-vocabulary.md) | line 44 (universal-applicability) | "Discovery, Authority, and Observability apply universally — every fundamental has list/inspect, delegate/allocate/transfer, and observe/log/attest" | silent-in-code for owner-tier (synth-grant doesn't synthesize Observe/Inspect on owned Org/Project) | honored (synth-grant covers all 4 verbs uniformly for owner-Agents) |
| [`core-philosophy.md`](../../../v0/concepts/core-philosophy.md) | lines 16, 28, 29 | "Organization has Projects (A Resource Type)" + Org/Project own Resources | honored at load-bearing semantic axis (CH-26); partially-honored at wire-tier (advisory consumption) | honored across both axes (wire-tier becomes blocking) |
| [`m5_3/architecture/composite-resources-model.md`](../../../v0/implementation/m5_3/architecture/composite-resources-model.md) | §"Advisory at CH-26; blocking at CH-27" + NEW §"Test-fixture pattern for owner-grant-required tests" | "Engine result consumed advisorily at CH-26; CH-27 tightens to blocking via `denial_to_api_error`" + NEW: "F4.b opt-in helper `seed_owner_grants` is canonical test-fixture pattern" | partially-honored (advisory wording shipped as the documented status; helper undocumented) | honored (CH-27 ships the tightening; doc body updated to "blocking at M5.3 close"; helper documented) |
| [`m5_3/architecture/agent-ownership-model.md`](../../../v0/implementation/m5_3/architecture/agent-ownership-model.md) | §"Synth-grant scope `[Allocate, Transfer]`" | "CH-25 ships `[Allocate, Transfer]`; future widening to Observe/Inspect deferred" | partially-honored (current narrow scope shipped) | honored (CH-27 widens; doc reflects new scope) |
| [`permissions/README.md`](../../../v0/concepts/permissions/README.md) | entry invariants | Permission Check is source of truth for resource authorization | partially-honored (advisory at admin-handler tier) | honored (blocking at admin-handler tier) |

**Permissions subtree hook**: `permissions/README.md` cited above as entry invariants source. ✅

**phi-core-mapping hook**: N/A — CH-27 touches Permission Check engine + handlers + acceptance fixtures; no phi-core type overlap. `phi-core-mapping.md` not cited.

---

## §3 — phi-core leverage map

| phi-core type | Current handling in baby-phi | Classification | Action in chunk |
|---|---|---|---|
| `phi_core::*` (any) | All 57 existing import sites preserved | direct-reuse | NONE — chunk touches engine + handler + acceptance tiers only; F4.b helper is baby-phi-defined (no phi-core import) |

**Expected import-count delta at chunk close**: **+0** (no new phi-core imports; no removed imports). F4.b helper `seed_owner_grants` is baby-phi-defined (lives in `server::tests::acceptance_common`; consumes `domain::repository::Repository` via test fixtures; no phi-core dependency).

**Positive close-audit grep**: `grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l` — expect **57** (matches CH-26 baseline preserved at gate-4).

**Forbidden-duplication greps**:
- `grep -rn "^struct AgentEvent\|^struct ExecutionLimits\|^struct ModelConfig\|^struct AgentProfile\|^struct Session" /root/projects/phi/baby-phi/modules/crates/ | grep -v "phi_core::"` — expect 0.
- `bash /root/projects/phi/baby-phi/scripts/check-phi-core-reuse.sh` — expect exit 0.

### §3 cascade-artifact discipline

#### Artifact A — `check_permission(...).is_ok()` consumption-flip cascade

**(a) grep**: `git -C /root/projects/phi/baby-phi grep -nE 'check_permission\(&ctx' modules/crates/server/src/platform/orgs/ modules/crates/server/src/platform/projects/`

**(b) raw count**: **7** invocations (verified 2026-05-18).

**(c) per-file breakdown**:
- `modules/crates/server/src/platform/orgs/create.rs:424` — `check_permission(&ctx, &manifest, &NoopMetrics).is_ok()`
- `modules/crates/server/src/platform/orgs/dashboard.rs:447` — same shape
- `modules/crates/server/src/platform/orgs/list.rs:157` — same shape
- `modules/crates/server/src/platform/orgs/show.rs:174` — same shape
- `modules/crates/server/src/platform/projects/agent_supervisor.rs:193` — same shape
- `modules/crates/server/src/platform/projects/create.rs:266` — same shape
- `modules/crates/server/src/platform/projects/detail.rs:316` — same shape

**Pause threshold**: if cascade exceeds **11 invocations** (1.5× predicted 7), pause via AskUserQuestion — would indicate scope expansion beyond planner's read of CH-26 deliverables.

**Per-handler tightening pattern** (~0.3-0.5 ed each):
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

**Handler-gating verification (v14 R1)**: each of the 7 handlers DOES invoke `check_permission` as of CH-26 (grep above returns 7 hits). The literal scenarios in plan §7 P3 are exercisable; v14 R1 check **PASSES**.

#### Artifact B — Synth-grant Action-array widening cascade

**(a) grep**: `git -C /root/projects/phi/baby-phi grep -nE 'Action::Allocate, Action::Transfer\]' modules/crates/domain/src/permissions/engine.rs`

**(b) raw count**: **at least 4 hits** (1 production + 3 test assertions).

**(c) per-file breakdown** (engine.rs):
- `:279` — `action: vec![Action::Allocate, Action::Transfer],` (production synth-grant builder at `synth_owner_grant` fn)
- `:2573` — test assertion `vec![Action::Allocate, Action::Transfer]` in `owner_grant_synth_owned_org_produces_allocate_transfer_candidate`
- `:2614` — same in `owner_grant_synth_owned_project_produces_allocate_transfer_candidate`

**Pause threshold**: if production hits > 1 (additional synth-grant builders discovered), pause. Test-array hits are expected.

**Widening pattern**: production array flips to `vec![Action::Allocate, Action::Transfer, Action::Observe, Action::Inspect]`; test names + array literals updated to match. NEW tests for Observe + Inspect verdicts on owned Org / owned Project.

**Verbatim concept-doc anchors** (v9 pre-flight per chunk-planner standards):
- `Action::CANONICAL` at `action.rs:250-285` includes `Observe` + `Inspect` (verified canonical — closed-set invariant `len() == 34` preserved).
- Concept-doc 03 line 44 universal-applicability claim covers all 4 verbs for owner-Agents.

#### Artifact C — Acceptance fixture extension cascade (F4.b USER-LOCKED opt-in helper path)

**(a) grep**: `git -C /root/projects/phi/baby-phi grep -lE 'orgs::list|orgs::show|orgs::dashboard|orgs::create|show_organization|org_dashboard|list_organizations|create_organization|projects::create|projects::detail|projects::agent_supervisor|show_project|create_project|list_agent_supervisors' modules/crates/server/tests/`

**(b) raw count**: **14 acceptance test files** exercise the 7 refactored handlers.

**(c) per-file breakdown**:
1. `acceptance_m3.rs`
2. `acceptance_m4.rs`
3. `acceptance_m5_3_composite_resources.rs` (CH-26-shipped)
4. `acceptance_memory_extraction.rs`
5. `acceptance_orgs_dashboard.rs`
6. `acceptance_per_session_consent_gating.rs`
7. `acceptance_projects_create.rs`
8. `acceptance_projects_detail.rs`
9. `acceptance_system_flows_s05.rs`
10. `identity_materialization_acceptance.rs`
11. `project_create_slot_fill_read_access_test.rs`
12. `project_create_submit_access_test.rs`
13. `project_creation_access_test.rs`
14. `show_organization_post_filter_test.rs`

**F4.b NEW helper cascade artifact** (USER-LOCKED, planner-rec was F4.a default-extension):

The NEW helper `seed_owner_grants(agent, [org_ids])` lives at `server/tests/acceptance_common/owner_grants.rs` (OR extends `m5_bootstrap.rs` — implementer chooses at P-FIXTURES based on existing module shape, with clear preference for NEW module file for audit-visibility per F4.b spirit). The helper:

- Emits Edge::Owns for each (agent, org_id) pair via direct `Repository::insert_edge` invocation (test-mode bypass of full compound-tx where the test wants to exercise ONLY the owner-grant path, NOT the full apply_org_creation).
- Forces the engine's synth-owner-grant rule into the manifest by ensuring Edge::Owns is queryable at engine `step_2_resolve_grants` time.
- May accept optional `[grants_to_seed]` param for explicit non-owner-grant scenarios (e.g., `seed_owner_grants_with_explicit_grants(agent, [(org_id, vec![Action::Observe])])`).
- **Per chunk-planner v13 R3 dep-direction verification**: lives in `server::tests::acceptance_common` (test-only scope; consumes `domain::repository::Repository` via test fixtures — dep direction `server → domain` ✓ canonical).
- **No phi-core import change** — helper is baby-phi-defined; uses `domain::model::{AgentId, OrgId, Edge}` exclusively.
- Predicted helper LOC: ~80 LOC (signature + per-pair edge insertion + optional grants_to_seed branch + brief doc-comment).

**Identification grep** (P-FIXTURES deliverable 2): `git -C /root/projects/phi/baby-phi grep -nE 'apply_org_creation|spawn_claimed_with_org' modules/crates/server/tests/ | grep -v 'seed_owner_grants'` — list tests that USE the org-creation/spawn path but DO NOT explicitly seed owner grants. Cross-reference §3 Artifact C 14-file list to surface tests that exercise the 7 refactored handlers and need explicit `seed_owner_grants(ceo, [org_id])` calls. Predicted: **12-18 tests** need the fixture extension.

**Pause threshold (F4.b cascade)**: if cascade exceeds **22 test sites** (1.5× predicted 15 mid-band of 12-18), pause to re-scope per F1.b split fallback. Aggregate ed pause-trip: 12 ed (revised up from v1's 11 ed under F4.a, to account for F4.b's +1-2 ed delta).

#### Artifact D — Re-enable + rename 2 acceptance scenarios

**(a) grep**: `git -C /root/projects/phi/baby-phi grep -nE 'owner_allocate_via_show_organization_handler_path|owner_allocate_via_dashboard_handler_path' modules/crates/server/tests/acceptance_m5_3_composite_resources.rs`

**(b) raw count**: **2 fn names** at lines 332 + 415 (verified).

**(c) per-file breakdown**: both at `acceptance_m5_3_composite_resources.rs`; rename → `owner_inspect_via_show_organization_handler_path` (line 332) + `owner_observe_via_dashboard_handler_path` (line 415); Action verb flips from `Action::Allocate` → `Action::Inspect` + `Action::Observe`.

#### Per-fork pause-threshold table (v17, refreshed for v2)

| Fork | If locked | Δ file-count cap | Δ aggregate ed cap | Δ test count |
|---|---|---|---|---|
| F1.a (LOCKED, single chunk) | Single CH-27 | ≤ 22 fixture sites under F4.b | ≤ 12 ed (1.5× upper-band trip; raised from v1 11 ed under F4.a) | +12 to +25 tests |
| F1.b (split CH-27a + CH-27b — fallback) | Multi-chunk | per-chunk ≤ 8 fixture sites | per-chunk ≤ 7 ed | per-chunk +6 to +15 tests |
| F3.a (LOCKED, defer resolvers) | Resolvers DEFERRED via D-CH27-FOLLOWUP-01 | unchanged | -1 to -4 ed | unchanged |
| F3.b (not-locked, trait re-shape) | Resolvers wired | +1 file (~6 resolver test stubs) | +3-4 ed | +6 to +10 tests |
| F3.c (not-locked, HTTP wrapper) | Resolvers wired at HTTP wrapper | +1 file (~50 LOC wrapper) | +1.5-2 ed | +3 to +5 tests |
| F4.a (NOT LOCKED, default-ext) | Default extension via apply_org_creation | unchanged | -1 to -2 ed | -2 to -5 tests vs F4.b |
| **F4.b (LOCKED, opt-in helper `seed_owner_grants`)** | **Opt-in helper** | **+1 file (~80 LOC helper)** | **+2-3 ed** | **+2 to +6 net new tests (12-18 fixture-extension are in-place edits; +2 NEW helper scenarios per §8)** |

Orchestrator at gate-1 has locked F1.a + F3.a + F4.b. Re-derived aggregate pause-thresholds: aggregate ed cap **12 ed**, fixture-site cap **22 sites**, test count band **[1570, 1574]** (see §8). Implementer at chunk-open is handed the **re-derived** thresholds.

---

### §3.B — K8s microservice readiness check

| Axis | Surface | Blocker? | Action |
|---|---|---|---|
| **A1** (in-process state) | No new mutexes / RwLocks / OnceCells; F4.b helper is stateless function | no | none |
| **A2** (IPC channel) | No new mpsc/broadcast/watch | no | none |
| **A3** (pod-local resource) | No new file handles / sockets / sub-processes | no | none |
| **A4** (migration runner) | NO new migration (CH-27 changes engine logic + handler consumption + test fixture; no schema change) | no | none |
| **A5** (trait-shape) | F3.a deferral preserves background-listener trait shape. F4.b helper consumes `Repository` trait via test fixtures (no trait shape change) | no | none |
| **A6** (cross-pod state) | No new cross-pod read-after-write expectation | no | none |
| **A7** (audit hash-chain) | No new audit writer; existing emitters preserved | no | none |

**Conforming-criteria check against ADR-0033**: D33.1 SessionRegistry trait untouched ✓. D33.2 SurrealStore::open_remote untouched ✓. D33.3 SIGTERM untouched ✓. D33.4 EventBus untouched ✓.

**Conclusion**: **K8s-neutral**. No new blockers introduced; no CHK8S-D-NN entry required.

### §3.C — User-facing documentation impact map

| Tier | File | This chunk touches? | Action |
|---|---|---|---|
| **Architecture** | [`m5_3/architecture/composite-resources-model.md`](../../../v0/implementation/m5_3/architecture/composite-resources-model.md) | YES — body needs amendment ("advisory at CH-26" → "blocking at CH-27") **+ NEW §"Test-fixture pattern for owner-grant-required tests"** (F4.b helper documentation per v2 deliverable 5) | (a) update in-chunk at P-DOCS |
| **Architecture** | [`m5_3/architecture/agent-ownership-model.md`](../../../v0/implementation/m5_3/architecture/agent-ownership-model.md) | YES — synth-grant scope section needs widening to `[Allocate, Transfer, Observe, Inspect]` | (a) update in-chunk at P-DOCS |
| **Architecture** | [`m1/architecture/permissions-engine.md`](../../../v0/implementation/m1/architecture/permissions-engine.md) (if cited synth-grant scope) | grep at P0 to confirm — if any "synth owner-grant" mention with the prior 2-verb scope, amend | (a) update in-chunk at P-DOCS, or "no change" if doc doesn't cite the literal scope |
| **Operations** | [`m5_3/operations/composite-resources-operations.md`](../../../v0/implementation/m5_3/operations/composite-resources-operations.md) | YES — error-code reference now lists denial mappings; advisory-only operator-visible behaviour flips to blocking 403/404 | (a) update in-chunk at P-DOCS |
| **Operations** | `m1/operations/permission-check-operations.md` (if exists; verify at P0) | grep at P0 | (a) update if exists / "no change" otherwise |
| **User-guide** | `m5_3/user-guide/*` (if exists) | grep at P0 | Amend if exists per CH-17 retro Row 3 (amend-don't-add precedence); else "no change — no user-guide tier exists at m5_3" |

**P0 grep deliverable**: enumerate all `m*/architecture/`, `m*/operations/`, `m*/user-guide/` files mentioning "advisory at M5" / "advisory at CH-26" / "advisory consumption" / "FOLLOWUP-01" / "deferred per CH-27" / "blocking at CH-27" / `[Allocate, Transfer]` / `step_2_resolve_grants` / `seed_owner_grants`. Each match either gets updated in-chunk OR explicitly noted "no change".

### §3.D — Forward-scope-vs-concept-doc precedence

Verified clean. Forward-scope row §2.5 line 284 calls out `Action::Observe` + `Action::Inspect` — both ARE in `Action::CANONICAL` (verified at `action.rs:250-285`). Closed-set invariant `Action::CANONICAL.len() == 34` PRESERVED. No new Action variants. No closed-set break.

Forward-scope row line 286 mentions "M3 + M4 + M5 acceptance fixtures" + cites `show_organization_post_filter_test.rs` + `m5_orgs_bootstrap_to_org_list_visibility` — verified file exists at `modules/crates/server/tests/show_organization_post_filter_test.rs`; `m5_orgs_bootstrap_to_org_list_visibility` test fn presence to verify at P0.

### §3.E — Anticipated gate-2.5 candidates

Per chunk-planner v13:

1. **Candidate 1**: `m5_3/architecture/composite-resources-model.md` body may carry "Advisory at M5.3 close" wording surface that needs flipping to "Blocking at M5.3 close (post-CH-27)" — likely flips during P-DOCS authoring. If discovered: route to P-DOCS inline (close in chunk).
2. **Candidate 2**: ADR-0061 §D61.5 + §D61.7 advisory-only-revision amendments may need P-SEAL cross-ref to ADR-0062 (forward-routing). If discovered: Trivial-multi P-SEAL inline.
3. **Candidate 3 — Bespoke-gate dead-code potential at handler sites** (NEW v2 per re-plan §3.E deliverable): once blocking is canonical, the bespoke gate (e.g., `check_auth_request_access` for AR-list filtering at `show.rs:79`) may be safely removable as dead-code. **Surface as gate-2.5 candidate** with route-decision: ≤ 0.5 ed per-site → in-chunk cleanup at P2; > 0.5 ed aggregate → forward-routing drift `D-CH27-FOLLOWUP-02` for M6+ cleanup.
4. **Candidate 4 — Per-handler audit-event emission decision** (NEW v2): any blocking-gate denial → emit audit event? Currently only `D-new-12` (M1 baseline) emits audit events at AuthRequest tier (NOT at admin-handler denial tier). Surface as gate-2.5 candidate with route-decision: if user routes "emit at handler denial" → in-chunk P2 deliverable (~0.5 ed/handler ×7 = ~3-4 ed); if defer → file `D-CH27-FOLLOWUP-03` for M6+ audit-tier-widening.
5. **Candidate 5** (if F3.b/c locked, currently F3.a locked so DORMANT): resolver trait re-shape mid-cycle complications. If discovered: pause + AskUserQuestion (F3 sub-fork escalation). **DORMANT under F3.a lock.**

---

## §4 — Drifts closed

| Drift ID | File | Severity | Transition | Notes |
|---|---|---|---|---|
| `D-CH26-FOLLOWUP-01` | [`m5_3/drifts/D-CH26-FOLLOWUP-01.md`](../../../v0/implementation/m5_3/drifts/D-CH26-FOLLOWUP-01.md) | LOW | discovered → remediated (at P-SEAL) | Wire-tier tightening + synth-grant widening + acceptance fixture extension via F4.b opt-in helper shipped; resolvers wiring deferred per F3.a (LOCKED) with NEW M6-DEFERRED drift filed |

### Non-terminal drift allocation (v13 rule)

NEW drift `D-CH27-FOLLOWUP-01` will be filed at P-SEAL with `Impl chunk = M6-DEFERRED-RESOLVERS-WIRING` (explicit allocation, NOT `TBD`). Drift body: resolver actor-passthrough architectural design + wiring at M6 plan-open.

Possible additional drifts (per §3.E gate-2.5 candidates 3 + 4):
- `D-CH27-FOLLOWUP-02` (if bespoke-gate cleanup forward-routed): `Impl chunk = M6-DEFERRED-BESPOKE-GATE-CLEANUP`.
- `D-CH27-FOLLOWUP-03` (if audit-event emission forward-routed): `Impl chunk = M6-DEFERRED-AUDIT-HANDLER-TIER`.

Both contingent on gate-2.5 candidate surfacing during implementation.

---

## §5 — ADRs drafted

**ADR-0062** — Blocking-gate enforcement + synth-grant scope widening + resolvers wiring deferral + F4.b opt-in helper for fixture extension (M5.3 carve-out closure).

- Drafted-at-phase: **P0** (Proposed).
- Flip-to-Accepted: **P-SEAL**.
- Decision-summary: ratify wire-tier tightening from advisory → blocking via `denial_to_api_error`; widen synth-owner-grant scope to `[Allocate, Transfer, Observe, Inspect]`; defer `projects::resolvers::*` actor-passthrough wiring (F3.a locked) to M6-DEFERRED; extend M3+M4+M5 acceptance fixtures via NEW opt-in helper `seed_owner_grants` (F4.b locked, USER-DIVERGENT from planner-rec F4.a default-extension); amend D-CH26-FOLLOWUP-01 body count "15" → "7".

### Top-level ADR section enumeration (v17 R2)

ADR-0062 MUST author all 7 canonical sections:

1. **`## Forks`** — **Divergent form** (F4.b USER-DIVERGENT from planner-rec F4.a); table lists all 4 forks with lock outcome + cross-cycle divergence cite.
2. **`## Context`** — chunk-graph (CH-25 → CH-26 → CH-27 M5.3 carve-out arc) + forward-scope §2.5 line 277-289 cite + D-CH26-FOLLOWUP-01 cite + F4.b cross-cycle divergence pattern cite (CH-26 F2.b precedent).
3. **`## Sub-decisions`** — one `### §D62.<M>` per fork + supporting decisions (refreshed for v2):
   - **§D62.1** — Wire-tier tightening pattern (consumption-flip `.is_ok()` → `?` via `denial_to_api_error`); 7-handler enumeration table at `orgs/{list,show,create,dashboard}.rs` + `projects/{create,detail,agent_supervisor}.rs`.
   - **§D62.2** — Synth-owner-grant rule widening — add Observe + Inspect to action set (current: Allocate + Transfer per ADR-0060 §D60.2; new: `[Allocate, Transfer, Observe, Inspect]`).
   - **§D62.3** — Resolvers wiring deferred to M6 (F3.a LOCKED); NEW drift `D-CH27-FOLLOWUP-01` with `M6-DEFERRED-RESOLVERS-WIRING` allocation; deferred-scope variation Pre-existing-behaviour note per v11.
   - **§D62.4** — **F4.b USER-DIVERGENT** — NEW `seed_owner_grants(agent, [org_ids])` test-helper at `server/tests/acceptance_common/owner_grants.rs` (~80 LOC) for fixture extension; documents the wire-format-explicit choice over F4.a default-extension; identified 12-18 acceptance tests get explicit `seed_owner_grants(ceo, [org_id])` call (semantically unchanged test bodies); rationale: cross-cycle user-preference for opt-in-visible test wire-up (CH-26 F2.b + CH-25 F1.b + i-phi CH-02b F4.b precedent).
   - **§D62.5** — **Count-amend META** — D-CH26-FOLLOWUP-01 body amended from "15 advisory check_permission invocations" → "7 advisory check_permission invocations" with footnote citing §3 Artifact A enumeration (7 handlers verified at `orgs/{list,show,create,dashboard}.rs` + `projects/{create,detail,agent_supervisor}.rs`).
   - **§D62.6** — **Pre-existing-behaviour preservation note per v11** — for each sub-decision: §D62.1 strict form ("Pre-existing CH-26 advisory consumption preserved at handlers; CH-27 tightens to blocking; shipped at CH-27 P-SEAL date YYYY-MM-DD"); §D62.2 strict form; §D62.3 deferred-scope variation ("Pre-existing scaffold preserved: projects::resolvers::* background trait shape unchanged; CH-27 ratifies the deferral via D-CH27-FOLLOWUP-01, does not implement"); §D62.4 strict form ("Pre-existing CH-26 acceptance fixtures preserved with explicit seed_owner_grants seeding added at touched test sites"); §D62.5 META META variation ("Pre-existing drift body preserved with footnote-corrected cardinality").

4. **`## Cross-references`** — (a) concept-doc + lines: `permissions/04-manifest-and-resolution.md` synth-grant section + `permissions/03-action-vocabulary.md:44` universal-applicability + `core-philosophy.md:16,28,29` + `composite-resources-model.md` + `agent-ownership-model.md`; (b) closed drift: `D-CH26-FOLLOWUP-01`; (c) prior ADRs cited: `0060-agent-as-creator-and-owner.md` (CH-25, synth-grant rule precedent), `0061-org-project-as-composite-resources.md` (CH-26, advisory-consumption rationale + F2.b precedent for F4.b USER-DIVERGENT), [`m1/decisions/0008-permission-check-as-pipeline.md`](../../../v0/implementation/m1/decisions/0008-permission-check-as-pipeline.md) (Step-2 pipeline invariants), [`m2/decisions/0018-handler-support-module.md`](../../../v0/implementation/m2/decisions/0018-handler-support-module.md) (`denial_to_api_error` precedent); (d) forward-scope row: §2.5 lines 277-289.
5. **`## Consequences`** — one `### For CH-NN` subsection per downstream chunk this forward-routes to: `### For M5.3 closure` (M6 plan-open unblocks); `### For M6 plan-open` (resolver-actor-passthrough scope inherited via D-CH27-FOLLOWUP-01); `### For future M7+ admin pages` (blocking-gate pattern is canonical, advisory→blocking handoff retired; `seed_owner_grants` is canonical test-fixture pattern for owner-grant-required tests).
6. **`## Revisit triggers`** — 6 bullets each citing a `§D62.<M>` that would warrant re-opening: (a) §D62.1 — new admin handler added that needs `check_permission` wiring; (b) §D62.2 — synth-grant scope further widened (e.g., new `Action::Audit` variant added to canonical set); (c) §D62.3 — M6 admin pages opens AND resolver wiring still deferred; (d) §D62.4 — fixture-extension cascade hits ≥ 22 sites mid-cycle (F4.b pause-trip) OR new owner-grant-required test category emerges that `seed_owner_grants` doesn't cover; (e) §D62.5 — drift body cardinality assertions need re-cross-checking after CH-27 close; (f) gate-2.5 Candidates 3 + 4 — bespoke-gate dead-code reveals dead role-check logic / audit-tier-widening request from M6+.
7. **`## Verification`** — commands the reviewer replays: grep enumeration for `denial_to_api_error` over orgs+projects platform (expect ≥ 7 hits); `Action::Observe` + `Action::Inspect` enumeration over `step_2_resolve_grants`; `seed_owner_grants` extant + ≥ 12 callsite count; canonical phi-core import count `grep -rn "use phi_core" modules/crates/ | wc -l == 57`; `cargo test --workspace` band match.

---

## §6 — Prior-chunk regression re-verification

| Upstream chunk | Invariant this chunk relies on | Re-verification command |
|---|---|---|
| **CH-26** | `Composite::ALL.len() == 10` (Org + Project as Composite) | `git -C /root/projects/phi/baby-phi grep -nE 'Composite::ALL.*10' modules/crates/domain/src/model/composites.rs` |
| **CH-26** | 7 handlers carry advisory `check_permission(&ctx, ...)` invocations | `git -C /root/projects/phi/baby-phi grep -cE 'check_permission\(&ctx' modules/crates/server/src/platform/{orgs,projects}/` expects ≥ 7 |
| **CH-26** | `tags: Vec<String>` on Organization + Project structs | `git -C /root/projects/phi/baby-phi grep -nE 'pub tags: Vec<String>' modules/crates/domain/src/model/nodes.rs` expects ≥ 2 |
| **CH-26** | Migration 0018 idempotent + applied | `cargo test -p store --test migrations_test` expects all green |
| **CH-25** | `Edge::Owns` extant + emitted at `apply_org_creation` | `git -C /root/projects/phi/baby-phi grep -nE 'Edge::Owns' modules/crates/domain/src/model/edges.rs` |
| **CH-25** | `EDGE_KIND_NAMES.len() == 72` invariant | `git -C /root/projects/phi/baby-phi grep -nE 'EDGE_KIND_NAMES.*72' modules/crates/domain/src/model/edges.rs` |
| **CH-25** | `synth_owner_grant` at `engine.rs:275`; synth-grant scope `[Allocate, Transfer]` at `:279` | grep at chunk-open; CH-27 widens at P1 |
| **CH-25** | `list_agent_owned_orgs` + `list_agent_owned_projects` Repository methods | `git -C /root/projects/phi/baby-phi grep -nE 'list_agent_owned_orgs\|list_agent_owned_projects' modules/crates/domain/src/repository.rs` |
| **M1 (ADR-0008)** | 6-step Permission Check pipeline preserved | `cargo test -p domain --test permissions_test` expects all engine tests green |
| **M2 (ADR-0018)** | `handler_support::check_permission` + `denial_to_api_error` extant | `git -C /root/projects/phi/baby-phi grep -nE 'pub fn denial_to_api_error' modules/crates/server/src/handler_support/permission.rs` |
| **Baseline** | `cargo test --workspace` test count = **1568/0/2** (CH-26 close) | `cargo test --workspace 2>&1 \| tail -10` |
| **Baseline** | phi-core import count = **57** | `grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ \| wc -l` |
| **Baseline** | 4 CI guards green | `bash scripts/check-{doc-links,ops-doc-headers,phi-core-reuse,spec-drift}.sh` |

---

## §7 — Phases within the chunk

### P0 — Scaffolding + pre-conditions re-verify + ADR-0062 drafted Proposed

**Goal**: re-verify §6 baseline; verify forward-scope-vs-concept-doc precedence clean; draft ADR-0062 scaffold Proposed (with v2 §D62.4 F4.b USER-DIVERGENT body); identify §3.C doc-impact scope via grep enumeration.

**Deliverables**:
1. §6 carry-forward grep checks all green (state count).
2. `Action::Observe` + `Action::Inspect` confirmed in `Action::CANONICAL` (action.rs:250-285).
3. §3.C doc-grep emitted: `git -C /root/projects/phi/baby-phi grep -rnE 'advisory at M5\|advisory at CH-26\|advisory consumption\|FOLLOWUP-01\|\[Allocate, Transfer\]\|seed_owner_grants' modules/ docs/specs/v0/implementation/m*/architecture/ docs/specs/v0/implementation/m*/operations/ docs/specs/v0/implementation/m*/user-guide/` — populate §3.C table rows accordingly.
4. ADR-0062 scaffold drafted at `m5_3/decisions/0062-blocking-gate-and-synth-grant-widening.md` with §D62.1-§D62.6 sub-decisions Proposed; §D62.4 body carries F4.b USER-DIVERGENT framing per v2 re-plan.
5. Cycle-index row appended at `_cycle-index.md` per chunk-implementer v9 paperwork checklist (R2): NEW verified-header line at TOP describing chunk-open with status `in-flight`.
6. Phi-core HEAD delta enumeration per chunk-planner v14 R3: `git -C /root/projects/phi/phi-core log --oneline <CH-26-close-SHA>..HEAD` — classify each commit; if API-additive/breaking touches AgentLoopConfig / StreamConfig / AgentEvent / other phi-core types baby-phi imports, plan carrier-fix at P-NEW-TESTS.

**Tests**: none new; baseline preserved.

**Concept-alignment check**: §2 rows pre-verified at "status at chunk-open"; no transitions yet.

**phi-core leverage check**: predicted Δ +0 confirmed at baseline (57 imports).

**User-facing doc updates**: §3.C table populated; per-row defer/in-chunk decisions locked.

**Confidence target**: **100%** (scaffolding phase).

**Pause discipline**: pause if (a) phi-core HEAD delta has uncommitted feature work → AskUserQuestion per v14 R3 protocol; (b) §6 carry-forward grep returns mismatches → architectural FAIL.

### P1 — Synth-grant scope widening to `[Allocate, Transfer, Observe, Inspect]`

**Goal**: widen `synth_owner_grant` to emit 4-verb scope for owner-Agents; add engine-level acceptance for Observe + Inspect verdicts.

**Deliverables**:
1. `engine.rs:279` flips `action: vec![Action::Allocate, Action::Transfer]` → `vec![Action::Allocate, Action::Transfer, Action::Observe, Action::Inspect]`.
2. Test names + array literals at `engine.rs:2573, 2614` updated to match (rename `_allocate_transfer_candidate` → `_authority_and_observability_candidate` OR similar; flip array literal).
3. NEW engine tests:
   - `owner_grant_synth_observe_on_owned_org_allows`
   - `owner_grant_synth_inspect_on_owned_org_allows`
   - `owner_grant_synth_observe_on_owned_project_allows`
   - `owner_grant_synth_inspect_on_owned_project_allows`
4. Existing test `owner_grant_synth_multiple_owned_resources_produces_one_candidate_each` updated to assert the 4-verb action vec instead of 2-verb.

**Tests**: +4 engine acceptance tests; ~3 existing tests get updated assertions (still green after update).

**Concept-alignment check**: §2 row `permissions/04-manifest-and-resolution.md` flips partially-honored → honored; §2 row `permissions/03-action-vocabulary.md:44` flips silent-in-code → honored.

**phi-core leverage check**: Δ +0 preserved.

**User-facing doc updates**: none yet — defer to P-DOCS.

**Confidence target**: **≥ 99%** (content phase).

**Pause discipline**: pause if any existing engine test breaks unexpectedly (would signal upstream regression on synth-grant flow).

### P2 — Wire-tier blocking-gate tightening at 7 handlers

**Goal**: flip `check_permission(&ctx, ...).is_ok()` consumption pattern → `denial_to_api_error` blocking pattern at all 7 handlers. Preserve bespoke gates as defence-in-depth post-engine-allow.

**Deliverables**:
1. `orgs/list.rs:157` — flip consumption pattern + map denials → list-empty (or per-row filter pattern depending on existing semantics — verify at implementation; if per-row filter, the engine result IS the filter).
2. `orgs/show.rs:174` — flip → `?` propagation via `denial_to_api_error`.
3. `orgs/create.rs:424` — flip → `?` propagation.
4. `orgs/dashboard.rs:447` — flip → `?` propagation.
5. `projects/create.rs:266` — flip → `?` propagation.
6. `projects/detail.rs:316` — flip → `?` propagation.
7. `projects/agent_supervisor.rs:193` — flip → `?` propagation.
8. Each flipped handler audited for response-status correctness (403 vs 404 vs 401 vs 400 per CH-25 wire convention at `permission.rs:76-`).
9. Per gate-2.5 Candidate 3: bespoke-gate dead-code cleanup OPTIONAL inline if ≤ 0.5 ed (single-line removal of now-redundant role check). If > 0.5 ed: forward-routing drift `D-CH27-FOLLOWUP-02`.
10. Per gate-2.5 Candidate 4: audit-event emission decision — if user routes "emit at handler denial" → +1 emit line per handler (~0.5 ed/handler ×7 = ~3-4 ed); if defer → file `D-CH27-FOLLOWUP-03`.

**Tests**: 2 advisory-only-renamed scenarios re-enabled + Action verb flipped:
- `acceptance_m5_3_composite_resources.rs:332` `owner_allocate_via_show_organization_handler_path` → `owner_inspect_via_show_organization_handler_path` (Action::Allocate → Action::Inspect).
- `acceptance_m5_3_composite_resources.rs:415` `owner_allocate_via_dashboard_handler_path` → `owner_observe_via_dashboard_handler_path` (Action::Allocate → Action::Observe).

**Concept-alignment check**: §2 row `composite-resources-model.md` flips partially-honored → honored at wire-tier; §2 row `permissions/README.md` flips partially-honored → honored.

**phi-core leverage check**: Δ +0 preserved.

**User-facing doc updates**: none yet — defer to P-DOCS.

**Confidence target**: **≥ 97%** (content phase; cascade risk through fixture extension at P-FIXTURES under F4.b lock).

**Pause discipline**: PAUSE if a tightened handler exposes HTTP-status semantic ambiguity NOT covered by §3 Artifact A pattern (e.g., needs 401 vs 403 distinction not in `denial_to_api_error`). Pause if cascade exceeds 11 invocations (1.5× predicted 7) — would signal scope expansion.

### P-FIXTURES — M3 + M4 + M5 acceptance fixture extension via F4.b opt-in helper

**Goal (v2 — F4.b USER-LOCKED)**: ship NEW opt-in helper `seed_owner_grants` + add explicit `seed_owner_grants(ceo, [org_id])` calls (or equivalent explicit-grant seeding) to identified M3+M4+M5 acceptance tests that exercise the 7 refactored handlers. Test bodies remain semantically unchanged.

**Deliverables (v2 — F4.b USER-LOCKED)**:

1. **NEW helper `seed_owner_grants` at `server/tests/acceptance_common/owner_grants.rs`** (or extend `m5_bootstrap.rs` based on existing module shape — implementer chooses with clear preference for NEW module for audit-visibility per F4.b spirit). Helper signature:
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
   ```
   Plus optional variant `seed_owner_grants_with_explicit_grants(agent, [(org_id, vec![Action])])` for non-owner-grant test scenarios. Predicted ~80 LOC including doc-comments.

2. **Identify M3+M4+M5 acceptance tests that may break under blocking gates**: grep for tests that use `apply_org_creation` + handler-tier operations without explicit grant seeding. Mechanical: `git -C /root/projects/phi/baby-phi grep -nE 'apply_org_creation|spawn_claimed_with_org' modules/crates/server/tests/ | grep -v seed_owner_grants` — cross-reference §3 Artifact C 14-file list. **Predicted: 12-18 tests**.

3. **For each identified test, add explicit `seed_owner_grants(ceo, [org_id])` call** (or equivalent explicit-grant seeding via `seed_owner_grants_with_explicit_grants` for the non-owner-grant cases). Test bodies remain semantically unchanged (same scenario, explicit owner-grant wire-up visible).

4. **Run cargo test --workspace; confirm all tests green; no test-count regression** from fixture extension alone (the explicit-seeding calls are in-place setup-tier edits).

5. **Document the helper at `m5_3/architecture/composite-resources-model.md` NEW §"Test-fixture pattern for owner-grant-required tests"** (v2 deliverable per F4.b lock). Documentation:
   - Why F4.b over F4.a (cross-cycle user-preference for wire-format-explicit fixture patterns)
   - Helper signature + intended usage pattern
   - When to use `seed_owner_grants` vs `seed_owner_grants_with_explicit_grants`
   - Forward-routing note: M6+ admin handlers should follow the same canonical fixture pattern

**Tests (v2 — F4.b lock)**: 12-18 existing tests get fixture-extension (in-place edits with explicit `seed_owner_grants` call; **no net test count delta from fixture extension itself** — same scenarios). Possible NEW 1-2 acceptance scenarios specifically exercising the helper + synth-grant Observe/Inspect widening at end-to-end HTTP+fixture level (~2 tests counted in §8 band).

**Concept-alignment check**: §2 rows preserved (no transitions at this phase); composite-resources-model.md §"Test-fixture pattern" added at P-DOCS will flip helper docs to honored.

**phi-core leverage check**: Δ +0 preserved (helper is baby-phi-defined; consumes `domain::repository::Repository` only).

**User-facing doc updates**: documented inline at P-FIXTURES deliverable 5 then ratified at P-DOCS.

**Confidence target**: **≥ 97%**.

**Pause discipline**: PAUSE if fixture-extension cascade exceeds 22 test sites (1.5× predicted 15 mid-band of 12-18 under F4.b) — would indicate scope expansion; consult F1.b split fallback. PAUSE if test bodies require non-trivial semantic change (helper doesn't cover an emergent test category) → AskUserQuestion + escalate (revisit §D62.4).

### P3 — Per-handler load-bearing acceptance + Observe/Inspect synth-grant integration tests

**Goal**: ship end-to-end scenarios validating blocking gate at HTTP tier + synth-grant Observe/Inspect verdict propagation through full handler-stack.

**Deliverables**:
1. NEW scenario `unauthorized_actor_blocked_at_show_organization_returns_403` — non-owner Agent invoking `show_organization` returns HTTP 403 via `denial_to_api_error` mapping.
2. NEW scenario `unauthorized_actor_blocked_at_org_dashboard_returns_403`.
3. NEW scenario `unauthorized_actor_blocked_at_project_detail_returns_403`.
4. NEW scenario `unauthorized_actor_blocked_at_list_agent_supervisors_returns_403`.
5. Existing scenario `owner_inspect_via_show_organization_handler_path` (renamed at P2) extended with HTTP-tier assertion (returns 200, not 403) + uses `seed_owner_grants(ceo, [org_id])` per F4.b convention.
6. Existing scenario `owner_observe_via_dashboard_handler_path` (renamed at P2) extended similarly with `seed_owner_grants` call.

**Tests**: +4 NEW scenarios + 2 renamed + extended (each using `seed_owner_grants` per F4.b canonical pattern).

**Concept-alignment check**: §2 row `core-philosophy.md` flips partially-honored → honored at wire-tier axis.

**phi-core leverage check**: Δ +0 preserved.

**User-facing doc updates**: none yet — defer to P-DOCS.

**Confidence target**: **≥ 99%**.

**Pause discipline**: PAUSE if HTTP-tier shape verification reveals semantic mismatch (e.g., bespoke role-gate fires BEFORE engine, so engine-deny scenarios still go through bespoke path).

### P-DOCS — User-facing docs + concept-doc amendments + drift housekeeping

**Goal**: amend all §3.C-listed docs; bump verified-headers; update concept-audit-matrix row for `core-philosophy.md` "Org has Projects" claim follow-up; ratify F4.b helper documentation.

**Deliverables**:
1. `m5_3/architecture/composite-resources-model.md` body amended: "advisory at CH-26" → "blocking at CH-27 (post-M5.3 close)"; NEW §"Test-fixture pattern for owner-grant-required tests" section ratified per F4.b deliverable 5; verified-header bump.
2. `m5_3/architecture/agent-ownership-model.md` body amended: synth-grant scope widened to 4-verb; verified-header bump.
3. `m5_3/operations/composite-resources-operations.md` body amended: error-code reference now lists 403/404 mapping; verified-header bump.
4. `m5_3/drifts/_concept-audit-matrix.md` "Org has Projects" row: follow-up D-CH26-FOLLOWUP-01 marker flipped `(LOW, CH-27 carve-out)` → `(LOW, **remediated at CH-27**)`.
5. P-DOCS doc-sync sweep per orchestrator gate-2 widened-sweep rule (CH-15 retro Row 1): grep all `m*/architecture/` + `m*/operations/` + `m*/user-guide/` for canonical stale-narrative phrase set; patch any matches.
6. Concept-doc `permissions/04-manifest-and-resolution.md` synth-grant scope mention amended (if cited literal 2-verb scope).
7. Concept-doc `permissions/03-action-vocabulary.md:44` universal-applicability verified preserved.

**Tests**: none new.

**Concept-alignment check**: §2 rows finalized at target status.

**phi-core leverage check**: Δ +0 final.

**User-facing doc updates**: complete §3.C table actions; ALL rows transition to "updated in-chunk" or "no change verified".

**Confidence target**: **≥ 99%**.

**Pause discipline**: no anticipated pauses.

### P-SEAL — Chunk-seal paperwork

**Goal**: flip drift + ADR + verified-headers + cycle-index row to chunk-seal state; amend D-CH26-FOLLOWUP-01 cardinality (15 → 7).

**Deliverables**:
1. Flip ADR-0062 Proposed → Accepted.
2. Flip `D-CH26-FOLLOWUP-01` Status `discovered` → `remediated` with CH-27 ✓ lifecycle entry. **Amend drift body cardinality "15 advisory check_permission invocations" → "7 advisory check_permission invocations" with footnote citing §3 Artifact A enumeration (per §D62.5 META).**
3. F3.a (LOCKED): FILE NEW drift `D-CH27-FOLLOWUP-01` with `Impl chunk = M6-DEFERRED-RESOLVERS-WIRING` (explicit allocation per v13 rule). Drift body: resolver actor-passthrough design + wiring scope at M6.
4. If gate-2.5 Candidates 3 + 4 forward-routed: FILE `D-CH27-FOLLOWUP-02` (`M6-DEFERRED-BESPOKE-GATE-CLEANUP`) + `D-CH27-FOLLOWUP-03` (`M6-DEFERRED-AUDIT-HANDLER-TIER`) per their respective in-chunk-vs-defer routing decisions.
5. `m5_3/drifts/README.md` row D-CH26-FOLLOWUP-01: "Closes at" → CH-27 ✓.
6. `_cycle-index.md` CH-27 row Status: prepend NEW verified-header per chunk-implementer v9 R2 (keep `Iterations = pending` and `Status = in-flight` — orchestrator owns the transitions per `_cycle-index.md` row-lifecycle paragraph: gate-3 → ready-for-audit; gate-4 close → audited-pending-retro; Phase 6/7 close → retro-complete + Iterations to final count).
7. Forward-scope §2.5 CH-27 row amended: append CLOSED marker with cycle hex + chunk-seal date + ADR-0062 cite.
8. Post-M5.3 actions paragraph (line 293) verified ready: "M6 plan-mode opens after CH-27 close" — CH-27 close NOW.
9. Cardinality-reference cascade grep + concept-doc section-anchor sync cascade grep per chunk-implementer v12 R3 + R4.

**Tests**: final `cargo test --workspace` run.

**Concept-alignment check**: §2 all rows at target status; concept-audit-matrix updated.

**phi-core leverage check**: Δ +0 final; `check-phi-core-reuse.sh` green; `use phi_core` count = 57.

**User-facing doc updates**: P-DOCS deliverables ratified.

**Confidence target**: **≥ 99%** (seal phase).

**Pause discipline**: no anticipated pauses.

---

## §8 — Tests summary

**Baseline at chunk-open**: **1568/0/2** (CH-26 close).

**MUST-SHIP (v2 — F4.b lock)**:
- 4 NEW synth-grant Observe/Inspect engine tests at `domain/src/permissions/engine.rs` test module (P1).
- 4 NEW HTTP-tier 403-block scenarios at `acceptance_m5_3_composite_resources.rs` (P3).
- 2 RENAMED + extended scenarios at `acceptance_m5_3_composite_resources.rs` — `owner_inspect_via_show_organization_handler_path` + `owner_observe_via_dashboard_handler_path` (P2 rename + P3 HTTP assertion extension + `seed_owner_grants` call).
- 12-18 existing acceptance tests get fixture-extension via `seed_owner_grants` call (P-FIXTURES; in-place edits; **no net test count delta from fixture extension** — same scenarios, explicit owner-grant seeding).

**MAY-COVER** (band-floor surrogates):
- 1-2 NEW acceptance scenarios specifically for the `seed_owner_grants` helper + synth-grant widening at end-to-end HTTP-tier (~2 tests; counted in upper-band ceiling).
- Per-handler 403-block unit-test additions in existing module-level tests (if surfaced at P2 implementation).
- Bespoke-gate-shadow simplification unit tests (P2 Candidate 3 — if in-chunk routed).
- Audit-event emission tests (P2 Candidate 4 — if in-chunk routed).

**Layer breakdown** (predicted under F1.a + F3.a + F4.b LOCKED):
- Unit / engine: **+4** (synth-grant Observe/Inspect)
- Integration / acceptance: **+4** (HTTP 403-block) **+2 renamed-extended** **+0-2 MAY-COVER helper-specific scenarios**
- Total NEW + extended (excluding fixture-extension in-place edits which are 0-delta): **+10 (lower-band)** to **+14 (upper-band)**

**Expected total at chunk-close (v2 — F4.b lock)**:
- **Pre-CH-27 baseline: 1568/0/2.**
- **Lower band: 1568 + 2 (MUST-SHIP minimum delta — count `synth-grant Observe/Inspect engine tests` + 0 MAY-COVER) = ~1570.** (Consumption-flip alone is no-op test-wise; synth-grant widening + Inspect/Observe re-enable adds the new tests.)
- **Upper band: 1568 + 6 (MUST-SHIP +4 engine + +2 MAY-COVER helper-specific scenarios; renamed extensions and 403-block scenarios partially overlap existing test-mod counts) = ~1574.**
- **Predicted band: [1570, 1574]** (post-fixture-extension; +2-6 net new tests; fixture-extension of 12-18 existing tests is in-place edits with no delta).

**Note on band derivation (v2)**: the v1 band [1578, 1585] assumed F4.a default-extension (cascade absorbed into existing test counts). The v2 band [1570, 1574] reflects F4.b's tighter accounting: the +4 NEW engine tests + the 2 rename-extensions are the core delta; helper-specific MAY-COVER scenarios add 0-2 more; HTTP 403-block scenarios partially overlap existing per-handler test groups (re-count under sub-mod boundaries).

**Named test files** (existing — extended in this chunk):
- `modules/crates/server/tests/acceptance_m5_3_composite_resources.rs` (rename 2 + add 4)
- `modules/crates/domain/src/permissions/engine.rs` (engine test module — add 4)

**Named test files** (NEW under F4.b LOCKED):
- `modules/crates/server/tests/acceptance_common/owner_grants.rs` — NEW F4.b helper module (~80 LOC; not a test file but contains the helper consumed by 12-18 existing tests).

**Named expected-still-green tests** (carry-forward; v17 grep-verified against actual repo state):
- `owner_allocate_via_create_organization_handler_path` (line 244 `acceptance_m5_3_composite_resources.rs`) — stays green under widened synth-grant (Allocate verb preserved); uses `seed_owner_grants(ceo, [org_id])` per F4.b convention.
- `owner_allocate_via_create_project_handler_path` (line 490) — stays green; uses `seed_owner_grants` per F4.b convention.
- `non_owner_denied_via_show_organization_handler_path` (predicted-existing at `acceptance_m5_3_composite_resources.rs`; verify at P0 actual fn names via `grep -hE "^async fn|^fn (test_|smoke_)" modules/crates/server/tests/acceptance_m5_3_composite_resources.rs`).
- CH-25 `acceptance_m5_3_owner_grant.rs` all scenarios green (synth-grant infrastructure).
- M3 + M4 + M5 named acceptance suites per §3 Artifact C breakdown — all 14 files exercise the refactored handlers; all must stay green post-fixture-extension via `seed_owner_grants` calls.

---

## §9 — Pre-chunk gate

**Reading list (mandatory)**:
1. [`concepts/permissions/04-manifest-and-resolution.md`](../../../v0/concepts/permissions/04-manifest-and-resolution.md) §"Owner-grant auto-issue rule (CH-25 / ADR-0060)".
2. [`concepts/permissions/03-action-vocabulary.md`](../../../v0/concepts/permissions/03-action-vocabulary.md) line 22 (table) + line 44 (universal-applicability) + line 250-285 (`Action::CANONICAL` enum).
3. [`concepts/core-philosophy.md`](../../../v0/concepts/core-philosophy.md) lines 16, 28, 29.
4. [`concepts/permissions/README.md`](../../../v0/concepts/permissions/README.md) entry invariants.
5. [`m5_3/drifts/D-CH26-FOLLOWUP-01.md`](../../../v0/implementation/m5_3/drifts/D-CH26-FOLLOWUP-01.md) full body.
6. [`m5_3/decisions/0061-org-project-as-composite-resources.md`](../../../v0/implementation/m5_3/decisions/0061-org-project-as-composite-resources.md) §D61.5 + §D61.7 (advisory-only-revision rationale + CH-27 carve-out routing) + §D61.4 (F2.b USER-DIVERGENT precedent for F4.b lock).
7. [`m5_3/decisions/0060-agent-as-creator-and-owner.md`](../../../v0/implementation/m5_3/decisions/0060-agent-as-creator-and-owner.md) §D60.2 + §D60.3 (synth-grant scope baseline).
8. [`m5_3/architecture/composite-resources-model.md`](../../../v0/implementation/m5_3/architecture/composite-resources-model.md) (current "advisory at CH-26" body).
9. [`m5_3/architecture/agent-ownership-model.md`](../../../v0/implementation/m5_3/architecture/agent-ownership-model.md) (current 2-verb synth-grant body).
10. [`forward-scope/22035b2a-remaining-scope-post-m5-p7.md`](../../forward-scope/22035b2a-remaining-scope-post-m5-p7.md) §2.5 CH-26 + CH-27 rows.
11. [`baby-phi/CLAUDE.md`](../../../../CLAUDE.md) phi-core Leverage section.
12. CH-26 plan archive: `docs/specs/plan/build/ch-26-org-project-as-composite-d1cb9e1f/plan.md` (cascade methodology + handler enumeration + F2.b precedent).
13. CH-25 plan archive: `docs/specs/plan/build/ch-25-<slug>-1e01618e/plan.md` (synth-grant rule rationale).
14. **Conditional (chunk-planner v8)**: this chunk touches `domain::permissions::engine::step_2_resolve_grants` body → READ `server/src/platform/sessions/launch.rs` body + `preview.rs` body (manifest-shape preconditions). Per CH-11 retro Row 6.
15. `server/src/platform/orgs/list.rs` + `show.rs` + `create.rs` + `dashboard.rs` bodies (the 4 org handlers).
16. `server/src/platform/projects/create.rs` + `detail.rs` + `agent_supervisor.rs` bodies (the 3 project handlers).
17. `server/src/handler_support/permission.rs` body (`denial_to_api_error` mapping).
18. `server/tests/acceptance_common/admin.rs` + `m5_bootstrap.rs` bodies (fixture entry points — establish F4.b helper location/extension choice).
19. `domain/src/events/listeners.rs:235-350` (resolver trait shapes — context for F3.a deferral).
20. `domain/src/permissions/engine.rs:200-320` + tests `:2520-2740` (synth-grant builder + existing tests).

**Carry-forward invariants** (explicit list, verified green at chunk open):
- `cargo test --workspace` test count = **1568/0/2** (matches CH-26 close baseline).
- `scripts/check-phi-core-reuse.sh` exit 0.
- `scripts/check-doc-links.sh` exit 0.
- `scripts/check-ops-doc-headers.sh` exit 0.
- `scripts/check-spec-drift.sh` exit 0.
- `modules/` diff against chunk-open git HEAD is empty.
- `grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l` = **57**.
- `Composite::ALL.len() == 10` invariant green (CH-26 carry).
- `EDGE_KIND_NAMES.len() == 72` invariant green (CH-25 carry).
- `Action::CANONICAL.len() == 34` invariant green (M1 baseline).

**Pending decisions carried into this chunk**: F1.a + F3.a + F4.b + count-amend ALL LOCKED at gate-1 (v2 re-plan).

---

## §10 — Close criteria

**4 aspects** (each graded pass / fail):

- **Code aspect**: all 7 phases' deliverables shipped including F4.b NEW `seed_owner_grants` helper at `server/tests/acceptance_common/owner_grants.rs` (~80 LOC); cargo test workspace within [1570, 1574] band (v2 refresh); clippy green under `RUSTFLAGS="-Dwarnings"`; fmt --check green.
- **Docs aspect**:
  - Governance tier: D-CH26-FOLLOWUP-01 lifecycle entry remediated + cardinality amended (15 → 7); ADR-0062 Accepted with §D62.4 F4.b USER-DIVERGENT body; concept-audit-matrix updated; verified-headers bumped on all touched docs.
  - User-facing tier: §3.C table rows ALL transition to "updated in-chunk" or "no change verified — explicit defer-decision with successor-chunk reference"; `composite-resources-model.md` NEW §"Test-fixture pattern for owner-grant-required tests" section added (F4.b helper documentation).
- **phi-core leverage aspect**: import-count Δ +0 (= 57); `check-phi-core-reuse.sh` exit 0; forbidden-duplication greps return 0; F4.b helper has zero phi-core imports.
- **Concept alignment aspect**: every §2 row reaches target chunk-close status; no rows remain `contradicted`.

**2 confidence %**:
- **Implementation confidence %** (v2 — F4.b lock widens numerator) = **target ≥ 9.5/10** (~24/25 claims-honored across the v2-widened scope: 7 handlers + 4 engine tests + 4 HTTP scenarios + 2 renamed + F4.b helper + 12-18 fixture-extensions + F4.b helper docs + count amend; the 1 remaining claim is the F3.a-deferred resolvers wiring filed as drift `D-CH27-FOLLOWUP-01`). If gate-2.5 Candidates 3+4 both routed in-chunk: target 10/10.
- **Documentation confidence %** = **target 9/9 docs touched verified cross-checkable against code + concept + ADR-0062 without ambiguity** = 100% (includes NEW composite-resources-model.md §"Test-fixture pattern" section).

**Composite target**: ≥ **9.5/10** (`min(impl%, doc%, code-binary, phi-core-binary, concept-binary)`).

**Explicit close-target discipline**: close report states ALL FIVE measures with named numerators/denominators. No aspect-averaging. No rounding.

**P-SEAL paperwork checklist** (v2026-05-03 + addenda):
- Every modified doc's verified-header description matches body diff exactly.
- Every `_concept-audit-matrix.md` row Status flipped letter-for-letter from plan §2 target column.
- Cycle-index row prepended NEW verified-header at TOP per chunk-implementer v9 R2.
- Cargo-clean discipline placement (1) honored: each `cargo test --workspace` followed by `cargo clean --manifest-path`.
- v13 non-terminal-drift rule honored: if new drift filed at P-SEAL, `Impl chunk` carries explicit `M*-DEFERRED-NN` allocation (NEVER `TBD`).
- F4.b helper documented at `composite-resources-model.md` NEW section (v2 deliverable).
- D-CH26-FOLLOWUP-01 cardinality amended (15 → 7) with footnote at body level + Lifecycle history entry.

---

## §11 — Post-chunk independent audit plan

**Phase count**: 7 (P0 + P1 + P2 + P-FIXTURES + P3 + P-DOCS + P-SEAL).

**Audit envelope**: **Large (3 agents)** — Audit A (code + phi-core + K8s) + Audit B (concept + docs + ADR) + Audit C (carry-forward regression).

### Audit A scaffold (code + phi-core + K8s) — ≤ 600 words

```
You are auditing CH-27 in baby-phi at /root/projects/phi/baby-phi/. Read-only on source. Plan at docs/specs/plan/build/ch-27-blocking-gate-enforcement-resolvers-wiring-0edcaba9/plan.md.

Verify each claim with file:line citation:
1. All 7 handler invocations at `orgs/{list,show,create,dashboard}.rs` + `projects/{create,detail,agent_supervisor}.rs` have flipped consumption pattern from `.is_ok()` → `denial_to_api_error` propagation (engine result IS BLOCKING — non-Allow returns HTTP 403/404 via wire convention).
2. `synth_owner_grant` at `engine.rs:~275-290` carries `action: vec![Action::Allocate, Action::Transfer, Action::Observe, Action::Inspect]` (4 verbs).
3. 4 NEW engine acceptance tests exist + green for synth-grant Observe + Inspect on owned Org + owned Project.
4. 4 NEW HTTP-tier 403-block scenarios exist + green at `acceptance_m5_3_composite_resources.rs`.
5. 2 RENAMED scenarios `owner_inspect_via_show_organization_handler_path` + `owner_observe_via_dashboard_handler_path` exist + green; both use `seed_owner_grants(ceo, [org_id])` per F4.b lock convention.
6. **F4.b helper verification (v2)**: `seed_owner_grants` extant at `server/tests/acceptance_common/owner_grants.rs` (~80 LOC) OR at extended `m5_bootstrap.rs`; signature matches plan §3 Artifact C body; helper consumes `Repository` trait via test fixtures (no phi-core import); optional `seed_owner_grants_with_explicit_grants` variant present if non-owner-grant scenarios surfaced.
7. **F4.b call-site count (v2)**: `git grep -nE 'seed_owner_grants\(' modules/crates/server/tests/` returns ≥ 12 hits across M3+M4+M5 acceptance suites (predicted 12-18 fixture-extension sites).
8. cargo test --workspace within band [1570, 1574]; 0 failed; ignored count preserved at 2.
9. phi-core leverage Δ +0: `grep -rn "use phi_core" modules/crates/ | wc -l` = 57. check-phi-core-reuse.sh exit 0.
10. K8s axes 7/7 honored per plan §3.B (no new in-process state / IPC channel / migration / trait-shape break).
11. RUSTFLAGS="-Dwarnings" clippy --workspace --all-targets returns 0 warnings (NOTE: this command may be sandbox-blocked; mark NOT-EXECUTED-IN-AUDIT and let orchestrator gate-4 close).
12. Per-handler response-status correctness: 403 for engine-deny (owner missing); 404 for unknown Org/Project IDs; 401 for unauth; 400 for shape errors (verify via per-handler test PASS/FAIL pattern).

PASS/FAIL each. ≤ 600 words.
```

### Audit B scaffold (concept + docs + ADR) — ≤ 600 words

```
You are auditing CH-27's concept-fidelity + docs-fidelity. Read-only.

Verify each claim:
1. ADR-0062 Accepted at `m5_3/decisions/0062-blocking-gate-and-synth-grant-widening.md` with §D62.1-§D62.6 sub-decisions ratified. All 7 canonical ADR sections present (Forks / Context / Sub-decisions / Cross-references / Consequences / Revisit triggers / Verification).
2. Each §D62.<M> sub-decision body ends with a Pre-existing-behaviour preservation note per v11 (strict form for §D62.1+§D62.2+§D62.4+§D62.5; deferred-scope variation for §D62.3 per F3.a lock; META variation for §D62.5 count-amend).
3. ADR §"Cross-references" populates all 4 categories: (a) concept-doc + line; (b) closed drift `D-CH26-FOLLOWUP-01`; (c) prior ADRs `0060` + `0061` (with F2.b precedent cited for F4.b USER-DIVERGENT) + `m1/0008` + `m2/0018` (milestone-prefixed paths per chunk-planner v6); (d) forward-scope §2.5 lines 277-289.
4. **§D62.4 F4.b USER-DIVERGENT body present (v2)**: explicitly cites cross-cycle pattern (CH-26 F2.b + CH-25 F1.b + i-phi CH-02b F4.b precedents); helper signature documented; planner-rec F4.a alternative path documented; rationale for user-lock surfaced.
5. **§D62.5 count-amend META body present (v2)**: D-CH26-FOLLOWUP-01 body amended from "15" → "7" with footnote citation to §3 Artifact A enumeration.
6. Drift `D-CH26-FOLLOWUP-01` Status = remediated; cardinality amendment "15" → "7" applied to body + Lifecycle history entry for CH-27 chunk-seal present.
7. drifts/README.md row D-CH26-FOLLOWUP-01: "Closes at" → CH-27 ✓.
8. `_concept-audit-matrix.md` `core-philosophy.md` "Org has Projects (Project = Resource Type)" row Status preserved at honored; CH-26 ✓ marker preserved; CH-27 ✓ marker added per follow-up.
9. `m5_3/architecture/composite-resources-model.md` body amended: "advisory at CH-26" wording replaced with "blocking at CH-27 post-M5.3 close"; **NEW §"Test-fixture pattern for owner-grant-required tests" section present documenting F4.b helper** (v2 deliverable); verified-header bumped.
10. `m5_3/architecture/agent-ownership-model.md` body amended: synth-grant scope widened narrative; verified-header bumped.
11. `m5_3/operations/composite-resources-operations.md` body amended: error-code reference; verified-header bumped.
12. P-DOCS doc-sync sweep grep returns 0 stale-narrative phrase matches across `m*/architecture/` + `m*/operations/` + `m*/user-guide/`.
13. K8s axes table classification N/A (no new blocker class introduced).
14. Plan archive at `docs/specs/plan/build/ch-27-blocking-gate-enforcement-resolvers-wiring-0edcaba9/plan.md` exists with cycle hex `0edcaba9` AND v2 re-plan banner.
15. Cycle-index row appended with cycle hex; verified-header line prepended at TOP per chunk-implementer v9 R2.
16. F3.a (LOCKED): NEW drift `D-CH27-FOLLOWUP-01` filed with explicit `M6-DEFERRED-RESOLVERS-WIRING` allocation (NOT `TBD`).
17. If gate-2.5 routed: `D-CH27-FOLLOWUP-02` + `D-CH27-FOLLOWUP-03` with explicit `M6-DEFERRED-*` allocations.
18. Forward-scope §2.5 CH-27 row amended at P-SEAL with CLOSED marker.

PASS/FAIL each. ≤ 600 words.
```

### Audit C scaffold (carry-forward regression) — ≤ 600 words

```
You are auditing CH-27's carry-forward regression posture. Read-only.

Verify each claim:
1. CH-26 acceptance suite still green: cargo test -p server --test acceptance_m5_3_composite_resources (10 original scenarios + 4 NEW 403-block + renamed 2 = ~16 total).
2. CH-25 acceptance suite still green: cargo test -p server --test acceptance_m5_3_owner_grant.
3. M3 acceptance suite still green: cargo test -p server --test acceptance_m3.
4. M4 acceptance suite still green: cargo test -p server --test acceptance_m4.
5. M5 acceptance suites still green: cargo test -p server --test acceptance_m5_orgs / acceptance_m5_projects / acceptance_m5_sessions / acceptance_m5_agents / acceptance_m5_memory.
6. show_organization_post_filter_test.rs all scenarios green post-fixture-extension via `seed_owner_grants` calls.
7. Migration runner test green: cargo test -p store --test migrations_test (slot 0018 preserved; no new migration).
8. Permission engine tests green: cargo test -p domain --test permissions_test (Action::CANONICAL.len() == 34 + new synth-grant tests pass).
9. Composite tests green: cargo test -p domain composites_test (cardinality 10 preserved).
10. Edge tests green: cargo test -p domain edges_test (EDGE_KIND_NAMES.len() == 72 preserved).
11. **F4.b helper wire-contract strip verification (v2)**: `seed_owner_grants` helper consumes `domain::repository::Repository` only (no phi-core import); helper-defined at `server/tests/acceptance_common/`; no helper consumption from outside `tests/` tree (test-only scope).
12. **F4.b helper call-site enumeration (v2)**: 12-18 acceptance tests across §3 Artifact C 14-file list call `seed_owner_grants(ceo, [org_id])` (or `seed_owner_grants_with_explicit_grants` variant); test bodies semantically unchanged from CH-26.

PASS/FAIL each. ≤ 600 words.
```

**Audit pass criteria**: any new drift discovered → its own drift file before chunk seals; any concept contradiction → fixed-in-chunk OR renegotiated with user OR converted to drift with future-chunk assignment.

---

## §12 — Verification section (end-to-end recipe)

```bash
cd /root/projects/phi/baby-phi

# 1. CI guards
bash scripts/check-doc-links.sh
bash scripts/check-ops-doc-headers.sh
bash scripts/check-phi-core-reuse.sh
bash scripts/check-spec-drift.sh

# 2. Workspace health
/root/rust-env/cargo/bin/cargo fmt --all -- --check
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy -j 4 --workspace --all-targets
/root/rust-env/cargo/bin/cargo test -j 4 --workspace 2>&1 | tail -10

# 3. Chunk-specific code-level greps (Artifact A — wire-tier tightening)
git -C /root/projects/phi/baby-phi grep -nE 'check_permission\(&ctx' modules/crates/server/src/platform/orgs/ modules/crates/server/src/platform/projects/
# Expect: 7 hits (one per handler — preserved from CH-26)

git -C /root/projects/phi/baby-phi grep -nE 'denial_to_api_error' modules/crates/server/src/platform/orgs/ modules/crates/server/src/platform/projects/
# Expect: ≥ 7 hits (one per blocking-gate handler — NEW at CH-27)

# 4. Chunk-specific code-level greps (Artifact B — synth-grant widening)
git -C /root/projects/phi/baby-phi grep -nE 'Action::Allocate, Action::Transfer, Action::Observe, Action::Inspect' modules/crates/domain/src/permissions/engine.rs
# Expect: 1 hit at synth_owner_grant fn body

# 5. Chunk-specific test enumeration (Artifact A — re-enabled scenarios)
git -C /root/projects/phi/baby-phi grep -nE 'owner_inspect_via_show_organization_handler_path|owner_observe_via_dashboard_handler_path|unauthorized_actor_blocked' modules/crates/server/tests/acceptance_m5_3_composite_resources.rs
# Expect: ≥ 6 hits (2 renamed + 4 NEW 403-block)

# 6. F4.b helper extant (v2 NEW)
git -C /root/projects/phi/baby-phi grep -nE '^pub (async )?fn seed_owner_grants' modules/crates/server/tests/acceptance_common/
# Expect: ≥ 1 hit (helper defined at owner_grants.rs OR extended m5_bootstrap.rs)

# 7. F4.b helper call-site count (v2 NEW)
git -C /root/projects/phi/baby-phi grep -nE 'seed_owner_grants\(' modules/crates/server/tests/ | wc -l
# Expect: ≥ 12 hits (12-18 acceptance tests get fixture-extension per §3 Artifact C; lower bound 12 = pause-trip floor inverted)

# 8. F4.b helper test-only scope verification (v2 NEW)
git -C /root/projects/phi/baby-phi grep -rnE 'seed_owner_grants' modules/crates/ | grep -v 'modules/crates/server/tests/' | wc -l
# Expect: 0 (helper consumed only from tests/ tree)

# 9. Carry-forward invariants
git -C /root/projects/phi/baby-phi grep -nE 'Composite::ALL.*10|EDGE_KIND_NAMES.*72|Action::CANONICAL.*34' modules/crates/domain/src/
# Expect: 3 hits (one per invariant)

grep -rn "use phi_core" /root/projects/phi/baby-phi/modules/crates/ | wc -l
# Expect: 57

# 10. Drift status verification
grep -l "Status.*remediated" /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_3/drifts/D-*.md | wc -l
# Expect: 2+ (D-philosophy-01 + D-philosophy-02 + D-CH26-FOLLOWUP-01) — verify final count at P-SEAL

# 11. D-CH26-FOLLOWUP-01 cardinality amendment verification (v2 NEW)
grep -nE '7 advisory check_permission invocations' /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_3/drifts/D-CH26-FOLLOWUP-01.md
# Expect: ≥ 1 hit (cardinality amendment "15" → "7" applied to body)

# 12. ADR-0062 verification
grep -E '^## (Forks|Context|Sub-decisions|Cross-references|Consequences|Revisit triggers|Verification)' /root/projects/phi/baby-phi/docs/specs/v0/implementation/m5_3/decisions/0062-blocking-gate-and-synth-grant-widening.md | wc -l
# Expect: 7 (all canonical ADR sections present)

# 13. Cargo-clean after each test run (per chunk-implementer v9 placement-1)
/root/rust-env/cargo/bin/cargo clean --manifest-path /root/projects/phi/baby-phi/Cargo.toml
```

---

## Plan-draft pre-archive line-number re-verification (v5 — added per CH-07 retro)

Re-ran key greps immediately before archive write (v1: 2026-05-18; v2 re-plan: 2026-05-18):
- `check_permission(&ctx` 7 hits at the 7 stated handler files — VERIFIED.
- `synth_owner_grant` at `engine.rs:275` — VERIFIED.
- Synth-grant scope `vec![Action::Allocate, Action::Transfer]` at `engine.rs:279` — VERIFIED.
- Test `owner_allocate_via_show_organization_handler_path` at `acceptance_m5_3_composite_resources.rs:332` — VERIFIED.
- Test `owner_allocate_via_dashboard_handler_path` at `acceptance_m5_3_composite_resources.rs:415` — VERIFIED.
- `denial_to_api_error` extant at `handler_support/permission.rs:76` — VERIFIED.
- phi-core baseline import count 57 — VERIFIED via canonical grep.
- Plan archive folder at `docs/specs/plan/build/ch-27-blocking-gate-enforcement-resolvers-wiring-0edcaba9/` — VERIFIED.
- **v2 NEW**: `seed_owner_grants` helper path target `server/tests/acceptance_common/` directory extant — VERIFIED.
- **v2 NEW**: existing `acceptance_common::admin.rs` + `m5_bootstrap.rs` modules present (helper location candidates) — VERIFIED.

No line-number drift between in-flight reading + v2 plan-archive write.
