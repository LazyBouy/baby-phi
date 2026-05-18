<!-- Last verified: 2026-05-17 by Claude Code (CH-26-d1cb9e1f P-SEAL — flipped Proposed → Accepted; §D61.4 + §D61.5 amended with advisory-only-revision (orchestrator-approved at gate-3 2026-05-16) + CH-27 carve-out cross-ref to D-CH26-FOLLOWUP-01; cycle hex `d1cb9e1f`.) -->
<!-- Last verified: 2026-05-17 by Claude Code (CH-26-d1cb9e1f P0 draft, Proposed) -->

# ADR-0061 — Org/Project as Composite resources + `tags` field + ≥ 7 handler refactor

**Status: Accepted**

**Authors**: Claude Code (orchestrator + chunk-planner v14 + chunk-implementer v9)

**Chunk**: CH-26 (cycle hex `d1cb9e1f`)

**Milestone**: M5.3 (second carve-out chunk; closes D-philosophy-02 HIGH-A)

**Forks** (gate-1 user-lock 2-of-3 DIVERGENT — F1.b wide-handler-refactor + F2.b tags-field-on-structs; F3 default 3 auditors aligned; cumulative cross-cycle divergent forks now 10-of-12 = 83%).

---

## §0 — Context

Concept doc [`concepts/core-philosophy.md`](../../../concepts/core-philosophy.md) line 16 claims *"Organization has Projects (A Resource Type)"*; lines 28-29 claim *"Organizations own Resources"* + *"Projects own Resources"*. Concept doc [`concepts/permissions/01-resource-ontology.md`](../../../concepts/permissions/01-resource-ontology.md) §"Composite Classes (8)" enumerates the closed 8-variant table; §"Instance Identity Tags" mandates every composite instance carry `{kind}:{instance_id}`.

Code at the start of CH-26:

- `Composite` enum at [`modules/crates/domain/src/model/composites.rs:20-45`](../../../../../../modules/crates/domain/src/model/composites.rs) has **8 variants** (`ExternalServiceObject` / `ModelRuntimeObject` / `ControlPlaneObject` / `MemoryObject` / `SessionObject` / `AuthRequestObject` / `InboxObject` / `OutboxObject`). Organization + Project are NOT in the closed-set.
- Permission Check selectors match `org:O` / `project:P` URIs (engine.rs:212+249+257 — CH-25 wired these via the synth-owner-grant rule), BUT the engine cannot enumerate Org/Project as `Composite` resource classes for action-applicability checks, and `seed_catalogue_entry_for_composite` cannot register Org/Project entries because no `Composite::OrganizationObject` / `Composite::ProjectObject` variant exists to pass.
- `Organization` (`nodes.rs:362`) + `Project` (`nodes.rs:460`) structs do NOT carry a `tags: Vec<String>` field; instance-identity tags `organization:<uuid>` / `project:<uuid>` are not materialised on the row.
- Admin-page-1 (Orgs) + admin-page-3 (Projects) handlers (`orgs::{list, show, create, dashboard}` + `projects::{create, detail, resolvers, agent_supervisor}`) DO NOT invoke `handler_support::check_permission` — they gate only via `AuthenticatedSession` extractors + per-handler bespoke logic. Grep `check_permission` over `server/src/platform/{orgs,projects}/` returns **0 hits** at chunk-open.

Drift [`D-philosophy-02`](../drifts/D-philosophy-02.md) (HIGH-A) captures this gap. The drift's §Where-visible-in-code:39 invariant: `grep -rn "check_permission" modules/crates/server/src/platform/{orgs,projects}/` expects **0 hits while open; ≥ 3 hits post-remediation**.

CH-26 v2 closes the gap by (per user-locked F1.b + F2.b):

1. Adding `Composite::OrganizationObject` + `Composite::ProjectObject` variants (cardinality 8 → 10) with `constituents() = [DataObject, IdentityPrincipal, Tag]`.
2. Adding `tags: Vec<String>` field to Organization + Project structs (with `#[serde(default)]` for backward-compat).
3. Migration `0018_org_project_tags.surql` — column add + backfill + catalogue seed.
4. Extending `apply_org_creation` + `apply_project_creation` compound transactions to populate instance-identity tags + seed catalogue entries at-creation-time.
5. Refactoring ≥ 7 admin-page-1/3 handlers to invoke `handler_support::check_permission` (closes D-philosophy-02:39 invariant FULLY at ≥ 7 hits, well above the 3-hit threshold).
6. NEW acceptance test `acceptance_m5_3_composite_resources.rs` with engine-level + per-handler PASS/FAIL scenarios.

---

## §1 — Decisions

### §D61.1 — Composite enum cardinality flip 8 → 10 (LOCKED — closed-set evolution)

**Decision**: Add two new variants to the closed `Composite` enum at `domain/src/model/composites.rs:20-45`:

```rust
pub enum Composite {
    // ... existing 8 variants ...
    OrganizationObject,
    ProjectObject,
}
```

Both variants reflect the Org/Project dual nature: governance container (carrying lifecycle / mission / vision / member-set semantics) + first-class resource (the entity grants reference at the resource-axis). The `constituents()` body returns `[Fundamental::DataObject, Fundamental::IdentityPrincipal, Fundamental::Tag]` for each — the same shape as `ControlPlaneObject` (which expresses the same dual-nature semantics for the platform-control-plane root).

`Composite::ALL` array cardinality flips 8 → 10; invariant tests in `composites.rs:182-249` rename `all_contains_exactly_eight → _ten` and flip literals at 5 cardinality sites:
- `composites.rs:50` — `pub const ALL: [Composite; 8]` → `[Composite; 10]`
- `composites.rs:184` — `assert_eq!(Composite::ALL.len(), 8)` → `== 10`
- `composites.rs:189-191` — distinct-variants test
- `composites.rs:206-207` — `kind_tags_are_distinct`
- `composites.rs:240-241` — `as_str_is_distinct_per_variant`

`as_str()` emits `"organization_object"` + `"project_object"`. `kind_tag()` emits `"#kind:organization"` + `"#kind:project"`. `kind_name()` returns `"organization"` + `"project"`.

**Rationale**: controlled invariant evolution. Same shape as CH-25 ADR-0060 §D60.6's 71 → 72 EDGE_KIND_NAMES evolution. NOT a closed-set break — the invariant test is renamed + the literal updated together.

**Pre-existing-behaviour preservation note**: *"Pre-existing scaffold preserved: 8 existing Composite variants ship at M1 + M3 + M4. CH-26 adds 2 NEW variants to the closed set; does not change semantics of existing variants. `constituents()` arms for existing variants are unchanged."*

### §D61.2 — `tags: Vec<String>` field on Organization + Project structs (USER-LOCKED F2.b, DIVERGENT)

**Decision**: Add `#[serde(default)] pub tags: Vec<String>` field to `Organization` (`nodes.rs:362`) and `Project` (`nodes.rs:460`) structs.

Precedent: `Session.tags`, `Memory.tags`, `Channel.tags`, `AgentCredential.tags` at `nodes.rs:836, 1062, 1074, 1082, 1269` — 4 existing node-types already carry `tags: Vec<String>` with `#[serde(default)]`. CH-26 extends the canonical pattern to Organization + Project.

Instance-identity tag (`organization:<uuid>` for Organization; `project:<uuid>` for Project) populated at-creation-time via `apply_org_creation` / `apply_project_creation` compound-tx extension. Backfill migration `0018_org_project_tags.surql` (§D61.3) populates extant rows.

**Dual-nature framing preserved**: Org/Project remain Principals (per `principal_resource.rs:182-186` — Principal-only invariant unchanged) AND become tag-bearing Composite resources. No `impl Resource for OrgId/ProjectId` — same as CH-25 ADR-0060 §D60.1 precedent (which preserved the Principal-only invariant under F1.b).

**Rationale for F2.b over planner-recommended F2.a** (catalogue-entry-based instance-identity): user systematically prefers richer-wire / more-explicit-state representation. Tags on the row are the load-bearing instance-identity surface for selector matching at Permission Check time; the catalogue entry remains the Step-0 precondition but is not a substitute for the row-level tag.

**Pre-existing-behaviour preservation note (chunk-planner v11 strict form)**: *"Pre-existing scaffold preserved: 4+ existing node-types ship `tags: Vec<String>` with `#[serde(default)]` (Session at CH-04 / Memory at CH-08 / Channel at CH-XX / AgentCredential at CH-XX). CH-26 extends the canonical pattern to Organization + Project; does not change the field-discipline mechanism."*

### §D61.3 — Migration `0018_org_project_tags.surql` (USER-LOCKED F2.b)

**Decision**: NEW migration file `modules/crates/store/migrations/0018_org_project_tags.surql` executes THREE operations in one transaction (per CH-06 + CH-16 idempotent-migration precedent + ADR-0012 forward-only-migrations):

(a) **Column add**: `DEFINE FIELD tags ON TABLE organization TYPE array<string> DEFAULT [];` + same for `project`.

(b) **Instance-identity backfill**: `UPDATE organization SET tags = ['organization:' + string::from(id)] WHERE tags IS NONE OR tags = [];` + same for `project`.

(c) **Catalogue-entry backfill**: for every extant Organization + Project row, ensure a catalogue entry exists (INSERT-OR-IGNORE semantics; idempotent via existing UPSERT path in the catalogue table).

Registered in `store/src/migrations.rs` with slug `org_project_tags` at slot 0018.

**Idempotency invariant**: replay against an already-backfilled DB MUST NOT duplicate tags or catalogue entries. UPDATE's WHERE clause filters on empty-or-none tags; catalogue uses UPSERT-on-uri semantics.

**Pre-existing-behaviour preservation note**: *"Pre-existing scaffold preserved: 17 prior migrations 0001-0017 ship at M1-M5 + CH-23 + CH-25. CH-26 adds 0018 as next-free slot; does not change existing migration bodies. Backfill mirrors CH-06 + CH-16's instance-identity-tag backfill discipline."*

### §D61.4 — Compound-tx extension: tags + catalogue-seed at-creation-time

**Decision**: `apply_org_creation` (both backends: `store/src/repo_impl.rs::apply_org_creation` + `domain/src/in_memory.rs::apply_org_creation`) extended to:

(a) Populate `Organization.tags = vec![format!("organization:{}", org_id)]` on the assembled struct.

(b) After the org-row INSERT, call `seed_catalogue_entry_for_composite(None, &format!("org:{}", org_id), Composite::OrganizationObject)?`.

Symmetric extension for `apply_project_creation`: populates `Project.tags = vec![format!("project:{}", project_id)]` + calls `seed_catalogue_entry_for_composite(Some(parent_org), &format!("project:{}", project_id), Composite::ProjectObject)?`.

Both extensions are atomic within the compound-tx (rolls back together on error).

Production callsites updated in `server/src/platform/orgs/create.rs:230` (Organization struct literal in `apply_org_creation`'s pre-payload assembly path) + `server/src/platform/projects/create.rs:366` (Project struct literal in `apply_project_creation`).

**Rationale**: instance-identity tags are not a separately-emittable side-effect — they're part of the row's canonical state. The compound-tx ensures atomicity; the catalogue seed is the Step-0 precondition for Permission Check selector matching against `org:<id>` / `project:<id>`.

### §D61.5 — Handler refactor pattern: ≥ 7 handlers through `check_permission` (USER-LOCKED F1.b, DIVERGENT)

**Decision**: ≥ 7 of the 8 admin-page-1/3 handlers refactored to invoke `handler_support::check_permission(ctx, manifest, metrics)` before any state mutation or read-side filtering:

| Handler | Action verb | Selector |
|---|---|---|
| `orgs::list::list_organizations` | `Action::Observe` | `org:*` (scoped to actor's owning orgs via CH-25 `list_agent_owned_orgs`) |
| `orgs::show::show_organization` | `Action::Inspect` | `org:<O>` |
| `orgs::create::create_organization` | `Action::Allocate` | `platform:root` (platform-admin synth-grant path) |
| `orgs::dashboard::org_dashboard` | `Action::Observe` | `org:<O>` |
| `projects::create::create_project` | `Action::Allocate` | `org:<parent_org>` |
| `projects::detail::show_project` | `Action::Inspect` | `project:<P>` |
| `projects::resolvers::*` (read tier) | `Action::Observe` | `project:<P>` |
| `projects::agent_supervisor::list_agent_supervisors` | `Action::Observe` | `project:<P>` |

Each handler builds a `CheckContext` per the precedent at `secrets/reveal.rs:97` (pre-load agent / org / project grants via repo, pre-load owned-resource slices via `list_agent_owned_*`, set `set_ref_registry: &domain::permissions::NOOP_SET_REF_REGISTRY` etc.), builds a `Manifest` declaring the appropriate action + resource class, and invokes `check_permission`.

**Amendment (P-SEAL 2026-05-17 — advisory-only revision, orchestrator-approved at gate-3 with user-routed CH-27 carve-out)**: the engine.check_permission invocations across the 7 refactored handlers are **CONSUMED ADVISORILY** — bespoke gates remain as wire-tier rejection surface. The load-bearing semantic claim (engine matches `org:O` + `project:P` selectors for owner-Agents per the synth-owner-grant rule) is fully shipped + validated by acceptance tests at `server/tests/acceptance_m5_3_composite_resources.rs` (10 scenarios — 4 engine-level + 6 per-handler shape replays). The wire-tier tightening to blocking gates (`denial_to_api_error` → HTTP 403) is deferred to **CH-27** (drift [`D-CH26-FOLLOWUP-01`](../drifts/D-CH26-FOLLOWUP-01.md)) per user routing 2026-05-16 to keep the work in M5 carve-out rather than M6+. Plan §3.E candidate 1 anticipates this future tightening.

**§D61.5 resolver-skip amendment (P-SEAL 2026-05-17)**: `projects::resolvers::*` (background read-tier resolvers) were **NOT wired** through `check_permission` at CH-26 close. Root cause: the resolver trait shape has no actor parameter — wiring requires actor-passthrough design that exceeds CH-26's blast-radius envelope. CH-27 carves out the design + implementation per drift [`D-CH26-FOLLOWUP-01`](../drifts/D-CH26-FOLLOWUP-01.md). At CH-26 close the 8-row handler table above ships with 7 wired + 1 deferred (`projects::resolvers::*`); the 7 wired hit 15 `check_permission` invocation sites total — well above the D-philosophy-02:39 ≥ 3-hit invariant.

Bespoke gates (e.g., `check_auth_request_access` for AR-list filtering at `show.rs:79`; CEO-only role check at `create.rs`) PRESERVED as defence-in-depth — they run AFTER `check_permission` succeeds at CH-26 close. CH-27 will simplify these to defence-in-depth-only once the engine result is blocking (Candidate 1 in plan §3.E).

**Blast radius**: under the advisory-only path shipped at CH-26, the M3 + M4 + M5 acceptance suites are **unaffected** — the engine result is computed but does not gate the response. Under CH-27's blocking-gate path, the suite triage strategy from plan §7 P2 applies: harness updates seed `Edge::Owns` for the actor → org / project under test BEFORE invoking the handler; CH-25 owner-grant synth-rule then resolves Allow.

**Rationale for F1.b over planner-recommended F1.a** (narrow scope, defer to M7): user systematically prefers the more-defensive-against-concept-drift option. F1.b fully closes D-philosophy-02:39's ≥ 3-hit invariant in-cycle (at 15 hits ≫ 3 threshold); the advisory→blocking tightening + resolver wiring + synth-grant scope widening route to **CH-27 (M5 carve-out extension, NOT M7-DEFERRED)**, preserving the load-bearing semantic in-cycle while keeping the wire-tier work in M5 per user direction 2026-05-16.

**Pre-existing-behaviour preservation note (chunk-planner v11 multi-milestone-pattern variation)**: *"Pre-existing implementation refactored: `orgs/{list,show,create,dashboard}` + `projects/{create,detail,agent_supervisor}` handlers shipped across M3-M5 with `AuthenticatedSession`-only gating + bespoke per-handler checks. CH-26 refactors the gating to flow through `handler_support::check_permission` (advisory consumption); preserves bespoke gates as wire-tier rejection surface; CH-27 will tighten to blocking. M3 + M4 + M5 acceptance suite fixtures will be extended at CH-27 to seed Edge::Owns + instance-identity tag before invoking handlers."*

### §D61.6 — ADR number amendment META

**Decision**: Forward-scope §2.5 line 270 predicted `ADR-0043`. Actual next-free ADR slot at CH-26 P0 is **ADR-0061** (latest shipped = 0060 from CH-25). The forward-scope's `0043` prediction is documentary-stale (does not match the linear ADR ordering applied across M5.2/M5.3). META sub-decision records the +18 amendment from prediction to actual.

No action required beyond this META documentation; the forward-scope row will be amended at P-SEAL as part of cycle housekeeping per CH-20 ADR-0058 §D58.10 precedent.

### §D61.7 — F1.b absorbs F1 deferral META (AMENDED at P-SEAL 2026-05-17 — CH-27 carve-out routing)

**Decision (original)**: Under F1.b user-lock (in-cycle handler refactor), no M7-DEFERRED follow-up drift is filed for the handler-refactor scope. D-philosophy-02:39's ≥ 3-hit invariant is fully met in-cycle at 15 hits (well above threshold).

**Amendment (P-SEAL 2026-05-17 — user-routed CH-27 carve-out)**: D-philosophy-02 transitions `discovered → remediated` at the **load-bearing semantic axis** in-cycle (engine matches `org:O` + `project:P` selectors for owner-Agents per the synth-owner-grant rule — 10 acceptance scenarios green). The **wire-tier tightening axis** routes to **CH-27 (M5 carve-out extension, NOT M7-DEFERRED-NN)** per user direction 2026-05-16 *"Can we create a CH-27 to document the remaining scope and finish it in M5?"*.

NEW drift filed: [`D-CH26-FOLLOWUP-01`](../drifts/D-CH26-FOLLOWUP-01.md) (Bucket B, Severity LOW, Closing chunk CH-27). The drift body enumerates: (a) 15 advisory `check_permission` invocations to tighten to blocking; (b) `projects::resolvers::*` actor-passthrough wiring; (c) CH-25 synth-owner-grant widening from `[Allocate, Transfer]` to also cover `Observe` + `Inspect`; (d) M3 + M4 + M5 acceptance-fixture extension with Edge::Owns / explicit-grant seeding; (e) re-enabling the 2 acceptance scenarios pinned at `Action::Allocate` (`owner_allocate_via_show_organization_handler_path`, `owner_allocate_via_dashboard_handler_path`) under their original `Inspect` / `Observe` action verbs after synth-grant widens.

M5.3 carve-out extends from 2-chunk {CH-25, CH-26} → 3-chunk {CH-25, CH-26, CH-27}; M6 plan-open shifts accordingly.

**Pre-existing-behaviour preservation note**: *"Shipped at CH-26 v2 P2 close (date 2026-05-17); CH-26 v2 does not change other handlers beyond the enumerated 7 wired + 1 deferred (resolver tier). The forward-scope §2.5 line 268 step 3 is fully honored at the load-bearing semantic axis in-cycle; the wire-tier axis routes to CH-27. The M7 chunks (admin-pages forward-scope §3) inherit the canonical pattern (advisory invocation → blocking tightening at the next chunk-pair) for future handler additions."*

Plan §3.E gate-2.5 candidates that surfaced at CH-26 close:
- Candidate 1 (handler bespoke-gate dead-code cleanup) → routed into CH-27's blocking-gate-tightening scope (D-CH26-FOLLOWUP-01).
- Candidate 5 (per-handler audit-event emission) → not surfaced this cycle; deferred to M6+ per planner discretion.

Other follow-up drifts not anticipated at CH-26 close.

---

## §2 — Cross-references

- (a) **Originating concept-doc + section + line range**: `concepts/core-philosophy.md:16` (Organization has Projects as Resource Type) + `concepts/core-philosophy.md:28-29` (Orgs own Resources + Projects own Resources) + `concepts/permissions/01-resource-ontology.md:30-43` (Composite Classes (8) table) + `concepts/permissions/01-resource-ontology.md:222-263` (Instance Identity Tags) + `concepts/permissions/01-resource-ontology.md:322-346` (Composite Creation Checklist).
- (b) **Closed drift(s) by ID**: `D-philosophy-02` (HIGH-A).
- (c) **Prior ADRs cited as precedent**:
  - [`m5_3/decisions/0060-agent-as-creator-and-owner.md`](0060-agent-as-creator-and-owner.md) — CH-25 owner-grant rule + Edge::Owns wire-up + `list_agent_owned_orgs/projects` repository methods that CH-26 reuses.
  - [`m1/decisions/0008-permission-check-as-pipeline.md`](../../m1/decisions/0008-permission-check-as-pipeline.md) — 6-step pipeline invariants honored by the handler refactor.
  - [`m1/decisions/0012-forward-only-migrations.md`](../../m1/decisions/0012-forward-only-migrations.md) — idempotent-migration discipline applied to `0018_org_project_tags.surql`.
  - [`m3/decisions/0022-org-creation-compound-transaction.md`](../../m3/decisions/0022-org-creation-compound-transaction.md) — compound-tx pattern CH-26 extends at `apply_org_creation` + `apply_project_creation`.
  - [`m2/decisions/0018-handler-support-module.md`](../../m2/decisions/0018-handler-support-module.md) — `handler_support::check_permission` entrypoint CH-26 wires ≥ 7 handlers through.
- (d) **Forward-scope row**: [`forward-scope/22035b2a-remaining-scope-post-m5-p7.md`](../../../../plan/forward-scope/22035b2a-remaining-scope-post-m5-p7.md) §2.5 lines 260-271.

---

## §3 — Consequences

**Positive**:

- Philosophy alignment with concept doc `core-philosophy.md` line 16 + 28 + 29 — D-philosophy-02 closed.
- Unified resource model: Org/Project are now first-class Composite resources, queryable via the same Permission Check selectors as MemoryObject / SessionObject / etc.
- Permission Check engine handles admin-page-1/3 authorization uniformly — no bespoke parallel authorization model for future M7 admin-page chunks.
- Concept doc 01 §"Instance Identity Tags" (lines 222-263) fully honored at the row level for Org/Project (not just at the catalogue level).
- Concept doc 01 §"Composite Creation Checklist" (lines 322-346) honored documentarily via ADR-0061 §body.

**Negative / cost**:

- `Composite::ALL.len() == 8` invariant flips to `== 10` — 5 literal-cardinality sites must flip in lockstep (cascade discipline at P1).
- `tags: Vec<String>` field added to Organization + Project struct shapes — ~51 struct-literal callsites (35 Org + 16 Project) cascade with `tags: vec![]` (or production-`tags: vec![format!(...)]`) fixture/site updates.
- Migration `0018_org_project_tags.surql` requires explicit DEFINE FIELD + UPDATE + catalogue seed (A4 axis fires).
- ≥ 7 handlers gain ~30-LOC CheckContext-building preamble each (per `secrets/reveal.rs:97` precedent); approximately +200 LOC of handler scaffolding.
- M3 + M4 + M5 acceptance suite fixtures need extension to seed Edge::Owns + instance-identity tags before invoking refactored handlers.

**Neutral**:

- `Action::CANONICAL.len() == 34` invariant preserved (no new Action verbs introduced).
- `EDGE_KIND_NAMES.len() == 72` invariant preserved (CH-25's flip stays as-is).
- `phi-core` import count unchanged (Δ +0).
- No new K8s blocker class (per plan §3.B verification at chunk-open).
- 5 existing trybuild `compile_fail` fixtures STAY GREEN (Org/Project stay Principal-only under F2.b — no `impl Resource` relaxation).

---

## §4 — Implementation phases

Per [plan §7](../../../../plan/build/ch-26-org-project-as-composite-d1cb9e1f/plan.md):

- **P0** — Scaffolding + pre-conditions re-verify + this ADR drafted Proposed.
- **P1** — Composite enum cardinality 8 → 10 + invariant cascade (Artifact A + Artifact B).
- **P-FIELD-EXTEND** — `tags: Vec<String>` field add + struct-literal cascade + materialise paths (F2.b).
- **P2** — Backfill migration + compound-tx extensions + ≥ 7 handler refactor (F1.b).
- **P3** — Action-applicability smoke tests + load-bearing acceptance + per-handler PASS/FAIL scenarios.
- **P-DOCS** — User-facing docs + concept-doc amendments + drift housekeeping.
- **P-SEAL** — Flip ADR to Accepted; flip D-philosophy-02 to remediated; cycle-index row; verified-headers.

---

## §5 — Audit envelope

Per [plan §11](../../../../plan/build/ch-26-org-project-as-composite-d1cb9e1f/plan.md): **3 auditors (F3 default — large envelope)** — Audit A (code + cascade + phi-core), Audit B (docs + concept + ADR), Audit C (carry-forward regression + handler-refactor blast-radius safety).

---

## §6 — Pause-and-escalate triggers fired during P0

None. P0 baseline:

- Baseline test count **1556** (matches plan §0 verified-header).
- §6 regression-table commands all green:
  - `Composite::ALL.len() == 8` at composites.rs:50 ✓.
  - `Edge::Owns` extant at `edges.rs:519, 684, 796` ✓.
  - `EDGE_KIND_NAMES.len() == 72` at `edges.rs:546+704` ✓.
  - `list_agent_owned_orgs` + `list_agent_owned_projects` at `repository.rs:1251+1263` ✓.
  - `format!("(org|project):` ≥ 4 hits at `engine.rs` (verified 9) ✓.
  - `Action::CANONICAL.len() == 34` at `action.rs:250, 439` ✓.
  - `all_contains_exactly_eight` at `composites.rs:183` ✓ (renamed `_ten` at P1 close).
  - `seed_catalogue_entry_for_composite` extant ≥ 9 hits across both backends + 3 production callsites ✓.
  - `impl Resource for OrgId` = 0 actual impls (only 1 comment at `principal_resource.rs:186`) ✓.
- Migration next-free slot **0018** (verified `ls modules/crates/store/migrations/` shows 0001-0017 taken).
- `check_permission` callsite count over `server/src/platform/{orgs,projects}/` = **0** at chunk-open (target ≥ 7 at chunk-close).
- ADR-0061 free (highest existing = ADR-0060 from CH-25).
- Plan's predicted `tags: Vec<String>` precedent verified at 5 node-types (Session/Memory/Channel/AgentCredential/+1).

No A4/A7 conditional triggers fired beyond the anticipated migration requirement (A4: migration 0018 needed for column-add + backfill, which IS the plan's scope; not a surprise).
