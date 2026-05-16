<!-- Last verified: 2026-05-16 by Claude Code (CH-25 / ADR-0060 P3 — agent-ownership-model design page paired with ADR-0060 §D60.1-§D60.6; documents NEW `Edge::Owns` variant + `OwnedResourceId` enum + owner-grant synth at `step_2_resolve_grants` + EDGE_KIND_NAMES 71→72 evolution + F1.b user-locked path rationale.) -->

# Agent ownership model — design page

> **Status:** [EXISTS] as of CH-25 (M5.3). The `Edge::Owns` variant + `OwnedResourceId` enum ship at [`modules/crates/domain/src/model/edges.rs`](../../../../../../modules/crates/domain/src/model/edges.rs) + [`domain/src/model/ids.rs`](../../../../../../modules/crates/domain/src/model/ids.rs); the synth-owner-grant lives in [`domain/src/permissions/engine.rs::step_2_resolve_grants`](../../../../../../modules/crates/domain/src/permissions/engine.rs); the new Repository methods `list_agent_owned_orgs` + `list_agent_owned_projects` land at [`domain/src/repository.rs`](../../../../../../modules/crates/domain/src/repository.rs) with two backend impls. For normative concept-doc references read [`concepts/core-philosophy.md`](../../../concepts/core-philosophy.md) lines 9, 23, 24, 31, 32 + [`concepts/permissions/04-manifest-and-resolution.md`](../../../concepts/permissions/04-manifest-and-resolution.md) §"Owner-grant auto-issue rule (CH-25 / ADR-0060)" + [`concepts/permissions/01-resource-ontology.md`](../../../concepts/permissions/01-resource-ontology.md) §"Principals + Resources".

---

## §1 — What this page covers

The concept doc [`concepts/core-philosophy.md`](../../../concepts/core-philosophy.md) asserts at lines 9, 23, 24, 31, 32 that *Agents own Organizations*, *Agents create Projects*, *Every Resource must have a creator*, and *Every Resource ownership must be tracked to the creator — Provenance*. Pre-CH-25, baby-phi's edge model had `Agent.owning_org: Option<OrgId>` (the **inverse** direction) but NO edge or field expressing Agent → Org/Project ownership/creation provenance. Drift [`D-philosophy-01`](../drifts/D-philosophy-01.md) (HIGH-A) captured the gap.

CH-25 lifts the philosophy claim from concept doc into typed Rust:

- §2 — NEW `Edge::Owns` variant + `OwnedResourceId` payload semantics (per §D60.1).
- §3 — Synth-owner-grant rule inside `step_2_resolve_grants` (per §D60.3).
- §4 — EDGE_KIND_NAMES 71→72 cardinality evolution rationale + why F1.b over F1.a (per §D60.6).
- §5 — Acceptance scenario walk.
- §6 — K8s posture.

ADR-0060 records the design decisions (sub-decisions §D60.1–§D60.6); this page is the operator-facing description.

---

## §2 — NEW `Edge::Owns` variant + `OwnedResourceId` payload (per §D60.1)

CH-25 adds one new variant to the closed-set `Edge` enum at [`domain/src/model/edges.rs`](../../../../../../modules/crates/domain/src/model/edges.rs):

```rust
pub enum Edge {
    // ... existing 71 variants ...
    Owns {
        id: EdgeId,
        from: AgentId,
        to: OwnedResourceId,
    },
}
```

The `to` end-point is a NEW domain-tier closed-set enum at [`domain/src/model/ids.rs`](../../../../../../modules/crates/domain/src/model/ids.rs):

```rust
pub enum OwnedResourceId {
    Org(OrgId),
    Project(ProjectId),
}
```

Both endpoints are typed at the variant payload level — `from: AgentId` is concrete (NOT a generic `Principal` bound), and `to: OwnedResourceId` is the closed 2-variant enum. The typed constructor `Edge::new_owns(&agent, OwnedResourceId::Org(org_id))` constrains the resource-end at compile time; the trybuild fixture `domain/tests/edge_type_safety/compile_fail/owns_rejects_user_as_owned_resource.rs` proves passing a `UserId` to the `Org` arm fails to compile.

### Why F1.b (NEW variant) over F1.a (re-use OWNED_BY + relax Resource trait)?

User-lock at gate-1 (rationale per ADR-0060 §D60.1): **explicit-and-visible edge semantics over structural cleanliness**. The Agent → Org/Project ownership relation is distinct enough (typed payload; dedicated SurrealDB relation table; dedicated semantics) to warrant its own variant rather than re-using the generic `Edge::OwnedBy` (which stays focused on Memory-as-Resource). Critically, F1.b does NOT relax the Resource trait — Org/Project STAY Principal-only per the v0 ontology invariant at `principal_resource.rs:182-186`. The new `OwnedResourceId` enum carries the typed resource-end without forcing `impl Resource for OrgId`.

### Struct-literal vs typed-constructor emission asymmetry

The F1.b path produces an asymmetric emission pattern at the Org/Project compound-tx sites:

- **`Edge::Owns`** is emitted via the **typed constructor** `Edge::new_owns(&ceo_agent, OwnedResourceId::Org(org_id))`. The closed 2-variant enum constrains the resource-end.

- **`Edge::Created`** is emitted via the **struct-literal form** because the typed `Edge::new_created<P: Principal, R: Resource>` constructor requires both trait impls on the endpoints, and OrgId/ProjectId are Principal-only:

  ```rust
  Edge::Created {
      id: EdgeId::new(),
      from: NodeId::from_uuid(*creator_agent.as_uuid()),
      to: NodeId::from_uuid(*org_id.as_uuid()),
  }
  ```

  The variant payload's `(from: NodeId, to: NodeId)` shape already erases endpoint kinds at the variant level (a pre-existing design choice from ADR-0015), so the wire representation is identical regardless of which form constructed it.

The asymmetry is the documented cost of NOT relaxing the Resource trait. Trybuild fixture `created_rejects_org_as_resource.rs` stays green under F1.b — the typed constructor still rejects OrgId at compile time when called.

---

## §3 — Owner-grant synthesis at `step_2_resolve_grants` (per §D60.3)

The Permission Check engine at [`domain/src/permissions/engine.rs`](../../../../../../modules/crates/domain/src/permissions/engine.rs) runs 6 steps + Step 2a (per ADR-0008 + ADR-0051 + ADR-0054). CH-25 extends Step 2 (`step_2_resolve_grants`) body with a synth-grant generator loop that fires AFTER the 3 existing `collect()` calls for `agent_grants` / `project_grants` / `org_grants`.

### Synth-grant shape

For every `org_id` in `ctx.agent_owned_orgs` slice, the engine synthesises:

```rust
Grant {
    holder: PrincipalRef::Agent(ctx.agent),
    action: vec![Action::Allocate, Action::Transfer],
    resource: ResourceRef { uri: format!("org:{}", org_id) },
    fundamentals: vec![Fundamental::IdentityPrincipal],
    descends_from: None,     // synth — no AR provenance
    delegable: true,
    issued_at: chrono::Utc::now(),
    revoked_at: None,
    approval_mode: ApprovalMode::Implicit,
    audit_class: AuditClass::Silent,
    allocate_refinement: None,
}
```

Pushed as `Candidate { tier: ScopeTier::Agent, resolved: ... }` (most-specific tier so it competes correctly with explicit grants in Step 5 strictest-wins composition). Same shape for `agent_owned_projects` with `uri: format!("project:{}", project_id)`.

### Why `[Allocate, Transfer]` on `IdentityPrincipal`?

Per ADR-0060 §D60.2 (F2.a user-lock): the forward-scope row literal *"[admin, transfer]"* re-interprets to canonical `[Action::Allocate, Action::Transfer]` because `Action::Admin` does NOT exist in the canonical 34-verb set. The Authority category at `action.rs:341-345` contains `Delegate`, `Approve`, `Escalate`, `Allocate`, `Transfer`. `Allocate` is the canonical org-control-plane Authority verb (used today for CEO grants at `claim.rs:241 + orgs/create.rs:181`). The `IdentityPrincipal` fundamental class is the axis on which Authority-category actions operate per concept-doc 03 line 149 + line 195 — owner can allocate / transfer authority *of* their owned org/project to its child agents.

### Pipeline-ordering preservation

Step 2 is the canonical stage for collecting candidate grants from typed sources (agent/project/org tiers). Synthesising at Step 2 (vs a pre-Step-0a synth-pass) preserves the 6-step + Step 2a pipeline ordering invariant from ADR-0008 + ADR-0051 + ADR-0054. Steps 3 (Match), 4 (Constraint), 5 (Scope), 6 (Consent) all work unchanged because the synth-grant looks identical to any other `Candidate` in the pool.

### CheckContext field-add cascade

`CheckContext` gains 2 new fields (lifetime-bound slices):

```rust
pub struct CheckContext<'a> {
    // ... existing 14 fields ...
    pub agent_owned_orgs: &'a [OrgId],
    pub agent_owned_projects: &'a [ProjectId],
}
```

Production call-sites (`server/src/platform/sessions/launch.rs`, `sessions/preview.rs`, `sessions/events.rs`, `secrets/reveal.rs`) load these slices via the new Repository methods:

```rust
async fn list_agent_owned_orgs(&self, agent: AgentId) -> RepositoryResult<Vec<OrgId>>;
async fn list_agent_owned_projects(&self, agent: AgentId) -> RepositoryResult<Vec<ProjectId>>;
```

Both InMemory + SurrealDB backends implement by reading `Owns` edges where `from == agent` and pattern-matching on the `to: OwnedResourceId` variant. Empty slices → synth loop no-ops (M1-baseline slice discipline preserved).

---

## §4 — EDGE_KIND_NAMES 71 → 72 cardinality evolution (per §D60.6)

The `EDGE_KIND_NAMES` invariant evolves across milestones:

| Milestone | Variants | Additions |
|---|---|---|
| M1 close | 67 | baseline (per ADR-0015 typed-edge sealing) |
| M3 close | 67 | (no edge-kind additions) |
| M4 / P1 | 69 | +2: `HasSubproject`, `HasConfig` |
| CH-23 | 71 | +2: `Manages`, `HasAgentSupervisor` |
| **CH-25** | **72** | **+1: `Owns`** |

This is a **controlled invariant evolution**, NOT a break — the invariant test is renamed `edge_kind_names_is_exactly_seventy_one → _seventy_two` and the literal cardinality is updated at 9 enumerated sites in lockstep:

1. `domain/src/model/edges.rs:525` — `pub const EDGE_KIND_NAMES: [&str; 71] → 72`.
2. `domain/src/model/edges.rs:658` — test name `edge_kind_names_is_exactly_seventy_one → _seventy_two`.
3. `domain/src/model/edges.rs:662` — `assert_eq!(EDGE_KIND_NAMES.len(), 71) → 72`.
4. `domain/src/model/edges.rs:668` — `assert_eq!(set.len(), 71) → 72` (distinct-names test).
5. `domain/src/model/mod.rs:80` — test name `ontology_has_seventy_one_edge_kinds → _seventy_two`.
6. `domain/src/model/mod.rs:84` — `assert_eq!(EDGE_KIND_NAMES.len(), 71) → 72`.
7. `domain/tests/m3_model_counts.rs:17` — test name `edge_count_bumps_from_sixty_six_to_seventy_one → _seventy_two`.
8. `domain/tests/m3_model_counts.rs:23` — comment `[&str; 71] → 72`.
9. `domain/tests/m3_model_counts.rs:26` — `assert_eq!(EDGE_KIND_NAMES.len(), 71) → 72`.

The narrative comment at `edges.rs:521-524` is updated with the `+1 at CH-25 (Owns)` append. Drift `D-philosophy-01.md:27` is amended with the 66 → 71 → 72 cardinality history.

---

## §5 — Acceptance scenario walk

The chunk's load-bearing acceptance test [`server/tests/acceptance_m5_3_owner_grant.rs`](../../../../../../modules/crates/server/tests/acceptance_m5_3_owner_grant.rs) exercises the full owner-grant flow:

1. Bootstrap a fresh acceptance server + claim the platform admin.
2. POST `/api/v0/orgs` via the real wizard. The compound transaction emits a CEO Agent + `Edge::Owns(CEO → Org)` + `Edge::Created(CEO → Org)`.
3. Query `Repository.list_agent_owned_orgs(CEO)` — surfaces the new Org's id.
4. Build a `CheckContext` for the CEO with **NO** explicit `agent_grants` / `project_grants` / `org_grants` — only the `agent_owned_orgs` slice populated from step 3. Build a `Manifest` requesting `[Action::Allocate]` over the owned-Org resource URI (`org:<id>`).
5. Invoke the engine via the production `check_permission` shim — expect `Decision::Allowed`.
6. Cross-check: an unrelated Agent (NO Owns edge) against the same manifest yields `Decision::Denied` at `FailedStep::Resolution` (`NO_GRANTS_HELD`) — proving the synth-grant is owner-scoped, not universal.

The test proves the **load-bearing claim** of the chunk: owner-Agent gains authority over the owned-Org without an explicit persisted grant.

---

## §6 — K8s posture

Per ADR-0060 §3 + plan §3.B verification:

| Axis | This chunk's surface | Blocker? |
|---|---|---|
| **A1** in-process state | Owner-grant synthesis is pure-function within Permission Check engine; no new in-memory state | no |
| **A2** IPC channel | none | no |
| **A3** pod-local resource | none | no |
| **A4** migration runner | new `owns` relation table at migration `0017_add_owns_relation.surql` (additive; idempotent re-run) | no |
| **A5** trait-shape | new Repository methods on existing `Repository` trait; trait-object dispatch preserved; new `OwnedResourceId` is domain-tier (R3-verified — no dep-direction violation) | no |
| **A6** cross-pod state | none — synth-grant fires from per-request CheckContext; edges persist via SurrealDB | no |
| **A7** audit hash-chain | owner-grant rule does NOT introduce a new audit-event class; underlying operations (e.g., `disable_agent`) emit their normal audit events; the synth-grant resolution itself is `AuditClass::Silent` | no |

**Conclusion**: K8s-neutral. CH-25 introduces no new K8s deployment hurdle.

---

## §7 — Cross-references

- **ADR-0060**: [`m5_3/decisions/0060-agent-as-creator-and-owner.md`](../decisions/0060-agent-as-creator-and-owner.md) — design decisions §D60.1–§D60.6.
- **Drift D-philosophy-01**: [`m5_3/drifts/D-philosophy-01.md`](../drifts/D-philosophy-01.md) — drift closed by this chunk.
- **Concept docs**: `concepts/core-philosophy.md` lines 9, 23, 24, 31, 32; `concepts/permissions/04-grants.md` §"Auto-issue rules"; `concepts/permissions/01-resource-ontology.md` §"Principals + Resources".
- **Prior ADRs cited as precedent**: ADR-0015 (typed-Edge sealing), ADR-0008 (6-step pipeline), ADR-0022 (compound-tx pattern at apply_org_creation), ADR-0034 §D34.4 (durable disable handler), ADR-0053 (system-genesis synth-grant precedent).
- **Operations page**: [`agent-ownership-operations.md`](../operations/agent-ownership-operations.md).
