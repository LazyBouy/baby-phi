<!-- Last verified: 2026-05-17 by Claude Code (CH-26 / ADR-0061 P-DOCS — composite-resources-model design page paired with ADR-0061 §D61.1-§D61.7; documents the OrganizationObject + ProjectObject Composite variants, `tags: Vec<String>` field on Organization + Project structs, migration `0018_org_project_tags.surql`, the ≥ 7 advisory-only handler refactor (CH-27 carve-out tightens advisory → blocking gate per D-CH26-FOLLOWUP-01), and the K8s posture. Cycle hex `d1cb9e1f`.) -->

# Composite resources model (Org/Project) — design page

> **Status:** [EXISTS] as of CH-26 (M5.3). The `Composite::OrganizationObject` + `Composite::ProjectObject` variants ship at [`modules/crates/domain/src/model/composites.rs`](../../../../../../modules/crates/domain/src/model/composites.rs); the `tags: Vec<String>` field on `Organization` + `Project` ships at [`domain/src/model/nodes.rs`](../../../../../../modules/crates/domain/src/model/nodes.rs); migration `0018_org_project_tags.surql` ships at [`store/migrations/`](../../../../../../modules/crates/store/migrations/); the ≥ 7 advisory `check_permission` invocations land across [`server/src/platform/orgs/`](../../../../../../modules/crates/server/src/platform/orgs/) + [`server/src/platform/projects/`](../../../../../../modules/crates/server/src/platform/projects/). For normative concept-doc references read [`concepts/core-philosophy.md`](../../../concepts/core-philosophy.md) lines 16, 28, 29 + [`concepts/permissions/01-resource-ontology.md`](../../../concepts/permissions/01-resource-ontology.md) §"Composite Classes" + §"Instance Identity Tags".

---

## §1 — What this page covers

The concept doc [`concepts/core-philosophy.md`](../../../concepts/core-philosophy.md) asserts at line 16 that *"Organization has Projects (A Resource Type)"* and at lines 28-29 that *"Organizations own Resources"* and *"Projects own Resources"*. Concept doc [`concepts/permissions/01-resource-ontology.md`](../../../concepts/permissions/01-resource-ontology.md) §"Composite Classes (8)" enumerates the closed 8-variant ontology and §"Instance Identity Tags" mandates every composite instance carry `{kind}:{instance_id}`. Pre-CH-26 the `Composite` enum was missing `OrganizationObject` + `ProjectObject` variants and `Organization` / `Project` structs lacked a `tags: Vec<String>` field, leaving the resource-axis of these claims unhonored. Drift [`D-philosophy-02`](../drifts/D-philosophy-02.md) (HIGH-A) captured the gap.

CH-26 lifts the philosophy claim from concept doc into typed Rust:

- §2 — NEW `Composite::OrganizationObject` + `Composite::ProjectObject` variants and their `constituents()` shape (per §D61.1).
- §3 — NEW `tags: Vec<String>` field on `Organization` + `Project` structs + instance-identity tag derivation rule (per §D61.2 + §D61.4).
- §4 — Migration `0018_org_project_tags.surql` — column add + backfill + catalogue seed (per §D61.3).
- §5 — Advisory-only ≥ 7 handler refactor through `handler_support::check_permission` + the CH-27 carve-out that will tighten advisory → blocking (per §D61.5 + drift [`D-CH26-FOLLOWUP-01`](../drifts/D-CH26-FOLLOWUP-01.md)).
- §6 — K8s posture.
- §7 — Acceptance test surface.

ADR-0061 records the design decisions (sub-decisions §D61.1–§D61.7); this page is the operator-facing description.

---

## §2 — NEW `Composite::OrganizationObject` + `ProjectObject` variants (per §D61.1)

CH-26 adds two new variants to the closed-set `Composite` enum at [`domain/src/model/composites.rs`](../../../../../../modules/crates/domain/src/model/composites.rs):

```rust
pub enum Composite {
    // ... existing 8 variants ...
    OrganizationObject,
    ProjectObject,
}
```

`Composite::ALL` cardinality flips **8 → 10**; the invariant test renames `all_contains_exactly_eight → _ten` and 5 cardinality-literal sites flip in lockstep (see ADR-0061 §D61.1 for the per-line list).

`as_str()` returns `"organization_object"` + `"project_object"`. `kind_tag()` returns `"#kind:organization"` + `"#kind:project"`. `kind_name()` returns `"organization"` + `"project"`.

`constituents()` returns `[Fundamental::DataObject, Fundamental::IdentityPrincipal, Fundamental::Tag]` for each — the same shape as `Composite::ControlPlaneObject`, reflecting the Org/Project dual nature:

- **DataObject** — the governance container's mutable state (display_name, mission, vision, lifecycle status).
- **IdentityPrincipal** — Org/Project ARE Principals (per `principal_resource.rs:182-186`); they carry the "who" axis when grants reference them as `holder` or `target`.
- **Tag** — every composite carries an implicit `#kind:{name}` tag plus a per-instance `{kind}:{instance_id}` identity tag (see §3 below).

**Closed-set invariant evolution discipline**: this is the same shape as CH-25 ADR-0060 §D60.6's 71 → 72 EDGE_KIND_NAMES evolution. Adding variants is permitted; removing them is not. The 5-site literal cascade is the proof-by-test that the cardinality change is total.

---

## §3 — `tags: Vec<String>` field on Organization + Project + instance-identity (per §D61.2 + §D61.4)

CH-26 adds a `tags: Vec<String>` field to `Organization` (`nodes.rs:362`-region) and `Project` (`nodes.rs:460`-region):

```rust
pub struct Organization {
    // ... existing fields ...
    #[serde(default)]
    pub tags: Vec<String>,
}

pub struct Project {
    // ... existing fields ...
    #[serde(default)]
    pub tags: Vec<String>,
}
```

The `#[serde(default)]` attribute makes pre-CH-26 JSON rows backward-compatibly deserialisable: rows persisted before the migration ran will deserialise with `tags = vec![]`.

**Precedent**: 4 existing node-types already carry the same shape — `Session.tags`, `Memory.tags`, `Channel.tags`, `AgentCredential.tags`. CH-26 extends the canonical pattern to Org/Project.

### Instance-identity tag

At create-time, every new Organization gets `tags: vec![format!("organization:{}", org_id)]` (analogously for Project: `vec![format!("project:{}", project_id)]`). The compound transactions extend to populate this in-line:

- `server/src/platform/orgs/create.rs::apply_org_creation` populates `Organization.tags` on the assembled struct before the row INSERT, then calls `seed_catalogue_entry_for_composite(None, &format!("org:{}", org_id), Composite::OrganizationObject)?`.
- `server/src/platform/projects/create.rs::apply_project_creation` does the symmetric work for Project (with `owning_org = Some(parent_org)`).

Both backends (`store/src/repo_impl.rs` SurrealDB + `domain/src/in_memory.rs` test backend) mirror the writes; the compound-tx is atomic — partial-state-on-error is impossible.

### Why F2.b (tags field on struct) over F2.a (catalogue-entry-only)?

User-lock at gate-1 (rationale per ADR-0061 §D61.2): **explicit-and-visible state representation over catalogue-entry indirection**. The tags-on-row form makes the instance-identity tag a load-bearing surface for selector matching at Permission Check time (the engine resolves `org:<id>` / `project:<id>` URIs against the row's `tags`). The catalogue entry remains the Step-0 precondition for action-applicability checks, but the row-level tag is the canonical source-of-truth.

### Dual-nature framing preserved

Org/Project remain **Principal-only** at the Resource trait level (per `principal_resource.rs:182-186`). No `impl Resource for OrgId` / `impl Resource for ProjectId` lands. CH-26's `Composite::OrganizationObject` + `Composite::ProjectObject` variants are **categorical labels** in the resource ontology — they let the engine reason about Org/Project as composite kinds — without relaxing the Principal-only invariant. The owner-grant authority shipped at CH-25 (via the synth-owner-grant rule at `step_2_resolve_grants`) is what carries the actual grant resolution; CH-26 hooks Org/Project rows into selector-match via their tags.

---

## §4 — Migration `0018_org_project_tags.surql` (per §D61.3)

NEW migration file [`modules/crates/store/migrations/0018_org_project_tags.surql`](../../../../../../modules/crates/store/migrations/) executes THREE operations in one atomic transaction:

1. **Column add**:
   ```surql
   DEFINE FIELD tags ON TABLE organization TYPE array<string> DEFAULT [];
   DEFINE FIELD tags ON TABLE project TYPE array<string> DEFAULT [];
   ```

2. **Instance-identity backfill** (idempotent — UPDATE only fires when tags is empty):
   ```surql
   UPDATE organization SET tags = ['organization:' + string::from(id)]
     WHERE tags IS NONE OR tags = [];
   UPDATE project SET tags = ['project:' + string::from(id)]
     WHERE tags IS NONE OR tags = [];
   ```

3. **Catalogue-entry backfill**: for every extant Organization + Project row, ensure a catalogue entry exists (INSERT-OR-IGNORE / UPSERT semantics — re-runs do not duplicate).

Registered in `store/src/migrations.rs` with slug `org_project_tags` at slot 0018.

**Idempotency invariant**: replay against an already-backfilled DB MUST NOT duplicate tags or catalogue entries. UPDATE's WHERE clause filters on empty-or-none tags; catalogue uses UPSERT-on-uri semantics. A `migration_0018_idempotent` integration test verifies the second replay is a no-op (CH-26 P2 deliverable).

---

## §5 — Advisory-only handler refactor + CH-27 carve-out (per §D61.5)

CH-26 wires `handler_support::check_permission` invocations across ≥ 7 admin-page-1 (Orgs) + admin-page-3 (Projects) handlers. The plan §3.D-philosophy-02 invariant **`grep -rn "check_permission" modules/crates/server/src/platform/{orgs,projects}/`** flips from 0 → 15+ hits at chunk-close. The closed handlers:

| Handler | Action verb | Selector |
|---|---|---|
| `orgs::list::list_organizations` | `Observe` | `org:*` (scoped via CH-25 `list_agent_owned_orgs`) |
| `orgs::show::show_organization` | `Inspect` | `org:<O>` |
| `orgs::create::create_organization` | `Allocate` | `platform:root` (platform-admin synth-grant path) |
| `orgs::dashboard::org_dashboard` | `Observe` | `org:<O>` |
| `projects::create::create_project` | `Allocate` | `org:<parent_org>` |
| `projects::detail::show_project` | `Inspect` | `project:<P>` |
| `projects::agent_supervisor::list_agent_supervisors` | `Observe` | `project:<P>` |

### Advisory-only: load-bearing semantic, not wire-tier gate

**Implementer-time scope-revision (2026-05-16, orchestrator-approved at gate-3)**: the 15 `engine.check_permission` invocations across these handlers are **consumed advisorily** — the bespoke gates (`AuthenticatedSession` extractors + per-handler role checks + AR-filter logic) **remain the wire-tier rejection surface**. The engine result is computed and logged but does not directly map to HTTP 403; the existing bespoke logic decides response status.

The **load-bearing semantic claim** that CH-26 delivers (and that closes D-philosophy-02) is:

> The Permission Check engine matches `org:O` + `project:P` URIs as selectors against owner-Agent contexts via the CH-25 synth-owner-grant rule.

This claim is **fully shipped + tested** at CH-26: the acceptance suite at `server/tests/acceptance_m5_3_composite_resources.rs` replays the exact `CheckContext` + `Manifest` shape each refactored handler builds and asserts the engine's verdict (Allow for owner, Deny for stranger) is correct.

### CH-27 carve-out: advisory → blocking + synth-grant widening

The wire-tier tightening of advisory → blocking is **deferred to CH-27** (M5.3 carve-out extension), tracked by drift [`D-CH26-FOLLOWUP-01`](../drifts/D-CH26-FOLLOWUP-01.md) (Bucket B, Severity LOW). CH-27 ships:

- Tightening of the 15 advisory invocations to blocking gates (engine-deny → HTTP 403/404 via `denial_to_api_error`).
- Widening of the CH-25 synth-owner-grant rule (`step_2_resolve_grants`) to cover `Action::Observe` + `Action::Inspect` for owner-Agents (current scope: `[Allocate, Transfer]` per ADR-0060 §D60.2).
- Wiring `projects::resolvers::*` background trait impls through `check_permission` (skipped at CH-26 — no actor parameter in the resolver trait shape; CH-27 designs the actor-passthrough).
- M3 + M4 + M5 acceptance-fixture extension with Edge::Owns / explicit-grant seeding where the new blocking gate would otherwise break existing tests.
- Re-enabling the Inspect/Observe handler-path acceptance scenarios that CH-26 pinned at the Allocate verb (per the synth-grant scope constraint).

User routing decision (2026-05-16): keep the blocking-gate work in M5 carve-out as a NEW chunk rather than deferring to M6+. The M5.3 carve-out extends from 2-chunk {CH-25, CH-26} → 3-chunk {CH-25, CH-26, CH-27}; M6 plan-open shifts accordingly.

### Why F1.b (in-cycle handler refactor) over F1.a (defer to M7)?

User-lock at gate-1 (rationale per ADR-0061 §D61.5): **maximal-scope-against-concept-drift over narrow surgical close**. F1.b ships the load-bearing semantic claim in-cycle (15 hits ≫ the 3-hit D-philosophy-02:39 invariant target) and treats the wire-tier tightening as a follow-on contract (CH-27) rather than a deferred future-milestone item. The advisory-only result-mode is the contracted half-step that lets the load-bearing claim ship cleanly without breaking M3/M4/M5 acceptance suites (which CH-27 will extend before tightening).

---

## §6 — K8s posture

Per plan §3.B: **K8s-neutral** under F1.b + F2.b user-locks.

| Axis | Verdict | Why |
|---|---|---|
| A1 in-process state | none | Composite enum + tags field are `Copy`/`Clone`; backfill migration is one-shot |
| A2 IPC channel | none | no new IPC surface |
| A3 pod-local resource | none | tags persist in SurrealDB → cross-pod durable |
| A4 migration | NEW 0018 | idempotent UPDATE-on-empty; single-runner invariant covered by CHK8S-D-05 |
| A5 trait-shape | none | handler refactor uses existing `handler_support::check_permission` indirection; tags field is serde-friendly additive |
| A6 cross-pod state sharing | none | tags + catalogue + grants all persist in SurrealDB |
| A7 audit hash-chain symmetry | none | no new audit-event writer in canonical case |

No new blocker class is introduced. CH-27's advisory → blocking tightening is also expected K8s-neutral — the existing `denial_to_api_error` shim is the only new wire-tier surface, and it already exists.

---

## §7 — Acceptance test surface

The CH-26 acceptance suite at [`server/tests/acceptance_m5_3_composite_resources.rs`](../../../../../../modules/crates/server/tests/acceptance_m5_3_composite_resources.rs) covers 10 scenarios:

1. **catalogue_seed_succeeds_on_wizard_create** — `apply_org_creation` seeds the `org:<id>` catalogue entry at-creation-time.
2. **applies_to_composite_for_organization_and_project_object** — pure-engine smoke that `Action::Allocate` / `Inspect` / `Observe` all apply to both new Composite variants (universal categories).
3. **engine_resolves_allocate_over_owned_org_for_ceo** — engine round-trip against `org:<O>` with owner-Agent context returns Allow.
4. **stranger_denied_allocate_on_org** — engine returns Deny for an unrelated Agent.
5-6. **owner_allocate_via_show_organization_handler_path** + **stranger_allocate_via_show_organization_handler_path** — `show_organization` handler engine shape (per CH-25 synth-grant scope constraint: pinned at `Allocate` verb; CH-27 widens to `Inspect`).
7-8. **owner_allocate_via_dashboard_handler_path** + **stranger_allocate_via_dashboard_handler_path** — `dashboard_summary` handler engine shape (same constraint; CH-27 widens to `Observe`).
9-10. **owner_allocate_via_create_project_handler_path** + **stranger_allocate_via_create_project_handler_path** — `create_project` handler engine shape against the parent org.

The per-handler scenarios replay the exact `CheckContext` + `Manifest` shape each refactored handler builds. Hitting the HTTP route would not distinguish the engine verdict from the bespoke verdict (since the engine result is advisory at CH-26); replaying the engine shape directly pins the invariant the chunk closes. CH-27 will re-enable the Inspect / Observe variants of these scenarios after widening the synth-grant scope.

---

## §8 — Cross-references

- [`ADR-0061`](../decisions/0061-org-project-as-composite-resources.md) — formal design record.
- [`D-philosophy-02`](../drifts/D-philosophy-02.md) — drift closed at CH-26 P-SEAL.
- [`D-CH26-FOLLOWUP-01`](../drifts/D-CH26-FOLLOWUP-01.md) — drift filed at CH-26 for CH-27 carve-out.
- [`agent-ownership-model.md`](agent-ownership-model.md) — CH-25 design page for the Edge::Owns + synth-owner-grant rule that CH-26 builds on.
- [`composite-resources-operations.md`](../operations/composite-resources-operations.md) — operator runbook.
- Plan archive: [`plan/build/ch-26-org-project-as-composite-d1cb9e1f/plan.md`](../../../../plan/build/ch-26-org-project-as-composite-d1cb9e1f/plan.md).
