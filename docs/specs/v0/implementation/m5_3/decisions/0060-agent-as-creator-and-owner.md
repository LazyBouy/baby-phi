<!-- Last verified: 2026-05-16 by Claude Code (CH-25-1e01618e P-SEAL — ADR flipped Proposed → Accepted. All sub-decisions §D60.1–§D60.6 ratified per user-locked F1.b / F2.a / F3.a / F4.a / F5.a / F6.b. Drift D-philosophy-01 transitions discovered → remediated in the same chunk-seal. EDGE_KIND_NAMES cardinality flipped 71 → 72 at all 9 enumerated literal sites. Acceptance test `acceptance_m5_3_owner_grant.rs` extant.) -->
<!-- Last verified: 2026-05-15 by Claude Code (CH-25-1e01618e P0 draft, Proposed) -->

# ADR-0060 — Agent-as-creator-and-owner edge model + owner-grant Permission Check rule

**Status: Accepted**

**Authors**: Claude Code (orchestrator + chunk-planner v13 + chunk-implementer v8)

**Chunk**: CH-25 (cycle hex `1e01618e`)

**Milestone**: M5.3 (first carve-out chunk; closes D-philosophy-01 HIGH-A)

**Forks** (CH-25 planner-recommendation: F1.a / F2.a / F3.a / F4.a / F5.a / F6.b; user-locked at plan approval to **F1.b / F2.a / F3.a / F4.a / F5.a / F6.b** — F1 DIVERGENT from planner-recommendation; divergence-aware framing applied to F1 + F2 + F4 per chunk-planner v13 80%-cumulative-divergence rule).

---

## §0 — Context

Concept doc [`concepts/core-philosophy.md`](../../../concepts/core-philosophy.md) (BINDING SPEC SOURCE, promoted 2026-04-28) claims at lines 9, 23, 24, 31, 32:

- *"Agent owns Organization"*
- *"Agent create Projects"*
- *"Agents own Resources"*
- *"Every Resource must have a creator"*
- *"Every Resource ownership must be tracked to the creator - Provenance"*

Code at the start of M5.3 has `Agent.owning_org: Option<OrgId>` (the **inverse** direction — Org-owns-Agent), but no edge or field expressing Agent→Org/Project ownership/creation provenance. Permission Check has no auto-grant rule that gives the owner-Agent admin authority over its owned Org/Project. Drift [`D-philosophy-01`](../drifts/D-philosophy-01.md) (HIGH-A) captures this gap.

CH-25 closes the gap by:

1. Adding a **NEW `Edge::Owns`** variant on the `Edge` enum (Agent → Org/Project ownership, with typed `OwnedResourceId` payload).
2. Emitting `Owns` + `Created` edges inside `apply_org_creation` + `apply_project_creation` compound transactions.
3. Synthesising an **owner-grant rule** inside `step_2_resolve_grants` (the canonical Permission Check pipeline Step 2): for every `Owns` edge `(agent → org|project)` the engine synthesises `Grant { holder: Agent(agent), action: [Allocate, Transfer], resource: org|project }` candidate grants and merges them into the candidate pool BEFORE selector matching.
4. Flipping the closed-set invariant `EDGE_KIND_NAMES.len() == 71` to **`== 72`** (a controlled invariant evolution — invariant test renamed `_seventy_one → _seventy_two` + literals updated at 9 enumerated sites).
5. Fixing the `permissions-audit` skill's broken `jq` predicate (CH-24 retro R5 carry-forward) — orthogonal to the code surface but bundled here per F5.a routing.

---

## §1 — Decisions

### §D60.1 — Owner-edge shape (USER-LOCKED F1.b, DIVERGENT)

**Decision**: Add a NEW `Edge::Owns { id: EdgeId, from: AgentId, to: OwnedResourceId }` variant on the `Edge` enum, where `OwnedResourceId` is a NEW domain-tier enum:

```rust
pub enum OwnedResourceId {
    Org(OrgId),
    Project(ProjectId),
}
```

The variant carries `from: AgentId` (typed, NOT generic `NodeId`) and `to: OwnedResourceId` (typed). Both endpoints are typed at the variant payload level to enforce the Agent→Org/Project semantics at compile time. A new typed constructor `Edge::new_owns(agent: &AgentId, owned: OwnedResourceId) -> Edge` is provided alongside the existing `new_owned_by` / `new_created` constructors. `EDGE_KIND_NAMES` cardinality flips 71 → 72 with the new entry `"OWNS"` placed in the Governance — Ownership section of the array (alongside `OWNED_BY`, `CREATED`, `ALLOCATED_TO`).

**Rationale (user-supplied at gate-1)**: explicit-and-visible edge semantics over the structural cleanliness of re-using generic OWNED_BY (planner-recommended F1.a path). The Agent → Org/Project ownership relation is distinct enough (typed payload, dedicated semantics, dedicated relation table at SurrealDB) to warrant its own variant rather than re-using generic OWNED_BY (which stays Memory-as-Resource focused). The CH-25 implementation thus does NOT relax `impl Resource for OrgId` / `impl Resource for ProjectId` — Org/Project STAY Principal-only per the v0 ontology invariant at `principal_resource.rs:182-186`.

**Closed-set evolution**: `EDGE_KIND_NAMES.len()` flips 71 → 72; the invariant test renames `edge_kind_names_is_exactly_seventy_one → _seventy_two` and the literal cardinality is updated at 9 enumerated sites (see §D60.6). This is a controlled invariant evolution, NOT a break — the invariant test is renamed + the literal updated together.

**Pre-existing-behaviour preservation note** (per chunk-planner v11): *"Pre-existing implementation preserved: typed `Edge::new_owned_by` + `Edge::new_created` + `Edge::new_allocated_to` constructors (shipped at ADR-0015 / M1 close); CH-25 adds a NEW `Edge::Owns` variant + `Edge::new_owns` constructor but does not change existing OwnedBy/Created/AllocatedTo semantics. Historical evidence lives at `domain/src/model/edges.rs:617-643` (existing typed constructors)."*

**Struct-literal-vs-typed-constructor distinction** (per user-directive at gate-2): the F1.b path produces an asymmetric emission pattern at the Org/Project compound-tx sites:

- **`Edge::Owns`** is emitted via the **typed constructor** `Edge::new_owns(&ceo_agent, OwnedResourceId::Org(org_id))` (or `OwnedResourceId::Project(project_id)`). The constructor's signature constrains the resource-end to the closed 2-variant enum at compile time, so wrong-pair endpoints (e.g., passing a `UserId` to the `Org` arm) fail to type-check. Trybuild fixture `owns_rejects_user_as_owned_resource.rs` enforces this invariant.

- **`Edge::Created`** is emitted via **struct-literal form** at the Org/Project creation sites:

  ```rust
  Edge::Created {
      id: EdgeId::new(),
      from: NodeId::from_uuid(*creator_agent.as_uuid()),
      to: NodeId::from_uuid(*org_id.as_uuid()),  // or *project_id.as_uuid()
  }
  ```

  The typed constructor `Edge::new_created<P: Principal, R: Resource>` requires both `Principal` AND `Resource` trait impls on the endpoints. Under F1.b (Resource trait NOT relaxed), `OrgId` / `ProjectId` are Principal-only — they do NOT implement `Resource` — so the typed constructor would fail to type-check at these call sites. The struct-literal form bypasses the trait bound by accepting raw `NodeId` endpoints; the typed `Edge::Created` variant payload `(from: NodeId, to: NodeId)` already erases endpoint kinds at the variant level (a pre-existing design choice from ADR-0015), so the wire representation is identical regardless of which form constructed it. The in-memory backend uses its internal `CreationEdge { creator: NodeId, resource: NodeId }` row which mirrors the same `NodeId, NodeId` shape; the SurrealDB backend uses the existing `created` relation table via `RELATE $f -> created -> $t` (the same path used by `upsert_creation_raw`).

  This asymmetry is the cost of NOT relaxing the Resource trait (which would have unified both emissions under the typed-constructor form). The trade-off is documented as accepted at gate-1 user-lock: F1.b's explicit-edge-semantics for `Owns` is worth the loss of typed-constructor uniformity for `Created` at the org/project creation sites. Trybuild fixture `created_rejects_org_as_resource.rs` continues to enforce the Principal-only constraint at the typed-constructor surface (it stays green under F1.b — the constructor itself remains usable for `(Agent, Memory)` pairs, etc.).

### §D60.2 — Owner-grant action set (LOCKED F2.a)

**Decision**: synth-owner-grant carries `action: [Action::Allocate, Action::Transfer]`.

**Rationale**: forward-scope §2.5 row literal text says *"auto-issue `[admin, transfer]`"*, but **`Action::Admin` does NOT exist in the canonical 34-verb set** at `action.rs:53-58 + :250-285 + :341-345`. The Authority category contains: `Delegate`, `Approve`, `Escalate`, `Allocate`, `Transfer`. The forward-scope literal `[admin, transfer]` re-interprets to canonical `[Action::Allocate, Action::Transfer]` per chunk-planner v8 §3.D mechanical procedure (concept-doc invariant wins over forward-scope literal). `Allocate` is the canonical org-control-plane Authority verb (used today for CEO grants at `claim.rs:241 + orgs/create.rs:181`); `Transfer` enables the owner-Agent to move ownership.

**Pre-existing-behaviour preservation note**: *"Pre-existing scaffold preserved: `Action::Wildcard` used by bootstrap-AR Grant + CEO `[Allocate]` grants (shipped at ADR-0014 / claim.rs:241 + orgs/create.rs:181 / M1+M3 close); CH-25 adds a synth-only auto-owner-grant carrying the new `[Allocate, Transfer]` action set; does not change pre-existing Action verbs. `Action::CANONICAL.len() == 34` invariant preserved."*

### §D60.3 — Owner-grant firing location (LOCKED F3.a)

**Decision**: synth-owner-grant fires **inside `step_2_resolve_grants`** body (after the 3 existing `collect()` calls for `agent_grants` / `project_grants` / `org_grants`).

For every `org_id` in `ctx.agent_owned_orgs` slice, the engine synthesises:

```rust
Grant {
    holder: PrincipalRef::Agent(ctx.subject.agent_id),
    action: vec![Action::Allocate, Action::Transfer],
    resource: ResourceRef { uri: format!("org:{}", org_id) },
    fundamentals: vec![],  // empty — synth grants carry no fundamentals shaping (the Allocate verb already implies org-control-plane reach)
    descends_from: None,  // synth — no Auth Request provenance
    delegable: false,     // synth-only; cannot delegate further
    issued_at: ctx.now,
    revoked_at: None,
}
```

… and pushes as `Candidate { tier: ScopeTier::Agent, grant: <above> }` (most-specific tier so it competes correctly with explicit grants in Step 5 strictest-wins composition). Same shape for `agent_owned_projects` with `uri: format!("project:{}", project_id)`.

**Rationale**: Step 2 is the canonical pipeline stage for collecting candidate grants from typed sources (agent/project/org tiers). Synthesising at Step 2 (vs a pre-Step-0a synth-pass) preserves the 6-step + Step 2a pipeline ordering invariant from ADR-0008 + ADR-0051 + ADR-0054. Selector matching at Step 3, audit-class composition at Step 5, and decision emission at Step 6 all work unchanged because the synth-grant looks identical to any other Candidate in the pool.

**Pre-existing-behaviour preservation note**: *"Pre-existing implementation preserved: 6-step + Step 2a Permission Check pipeline (shipped at ADR-0008 / M1 close, refined at CH-07 / ADR-0051 + CH-15 / ADR-0054); CH-25's F3.a path extends `step_2_resolve_grants` body with synth-grant collection from `ctx.agent_owned_orgs/projects`; preserves all 6 steps' ordering + does not change Step 1/3/4/5/6 bodies. The synth-grant looks identical to any other Candidate at the type level, so downstream Step 5 strictest-wins composition is unchanged."*

### §D60.4 — Acceptance scope (LOCKED F4.a)

**Decision**: NEW acceptance file `server/tests/acceptance_m5_3_owner_grant.rs` with the canonical scenario:

1. Bootstrap via `claim` → human Agent A1.
2. A1 creates Org O1 via `apply_org_creation` (CEO = A1; creator = A1; owner = A1 via `Edge::Owns { from: A1, to: OwnedResourceId::Org(O1) }`).
3. A child Agent A2 is created under O1 (CH-22 child-creation flow).
4. A1 disables A2 via the `disable_system_agent` handler **WITHOUT** any explicit prior `[disable]` grant on A2.
5. The Permission Check engine's synth-owner-grant rule (§D60.3) synthesises `[Allocate, Transfer]` on `org:{O1}`. The selector matching at Step 3 expands the manifest reach for `disable` on agent-in-O1 to be covered by `[Allocate]` on `org:{O1}` (the org-control-plane Authority verb).
6. The disable handler succeeds; A2 is durably `active: false`; the audit event is emitted via the synthesized grant.

**Rationale**: a single end-to-end acceptance test exercises (i) the new edge emission at `apply_org_creation`, (ii) the new synth-grant in `step_2_resolve_grants`, (iii) the new Repository methods `list_agent_owned_orgs/projects`, and (iv) the existing `disable_system_agent` handler unchanged. Mirrors CH-24's `acceptance_m5_<topic>.rs` 5-file split precedent.

**Load-bearing-form re-interpretation (CH-25 P3 implementation reality + orchestrator-approved at gate-3 dispatch)**: at P3 implementation time, the implementer discovered that `disable_system_agent` at `server/src/handlers/system_agents.rs:123-144` does NOT invoke `check_permission` (it is gated only by `AuthenticatedSession`). So step 4-6 above as literally written would NOT exercise the synth-owner-grant path — the disable would succeed regardless of whether the synth-grant fired. To preserve the §D60.4 INVARIANT ("owner-Agent gains authority over the owned-Org without an explicit persisted grant") decoupled from any single HTTP handler's gating choices, the acceptance test ships in its **load-bearing form**: (a) bootstrap → wizard org-creation → emit `Edge::Owns`; (b) call `list_agent_owned_orgs` to confirm the edge is persisted + queryable; (c) directly invoke `handler_support::check_permission` to assert the engine resolves `[Allocate]` over `org:{O1}` via the synth-owner-grant rule (no explicit persisted grant); (d) a stranger Agent with no Owns edge over O1 is denied with `NO_GRANTS_HELD` / 403 (cross-org isolation invariant). This load-bearing form exercises the same 4 chain links (edge emit / Repository query / synth-grant rule / engine result) without coupling the test to whichever specific HTTP handler happens to invoke `check_permission` at any given milestone. The acceptance test's module docstring documents this scope decision explicitly at `acceptance_m5_3_owner_grant.rs:33-46`. **M6 follow-up candidate**: wire `check_permission` into the disable handler so the literal scenario form becomes exercisable as well (M6-DEFERRED-NN allocation pending forward-scope §M6 routing).

**Pre-existing-behaviour preservation note** (multi-milestone-pattern variation form per chunk-planner v11): *"Pre-existing implementation preserved: `disable_system_agent` handler shipped at CH-01 / ADR-0034 §D34.4 (durable `active: false` flip BEFORE audit emit; idempotent re-POST). CH-25 adds an acceptance test that exercises owner-grant auto-issue at the engine level (load-bearing form) decoupled from the disable-handler; does not change the handler itself."*

### §D60.5 — R5 permissions-audit skill fix (LOCKED F5.a)

**Decision**: fix `.claude/skills/permissions-audit.md` line 31 by folding the `version: 1` predicate into the same `select(...)` filter:

```diff
- jq -c 'select(.ts >= "$start_ts" and .ts <= "$end_ts") | .version == 1' ...
+ jq -c 'select(.ts >= "$start_ts" and .ts <= "$end_ts" and .version == 1)' ...
```

**Root cause**: in the broken form, `select(...)` filters by time window, then the pipe applies `.version == 1` to each filtered entry — but `.version == 1` is a boolean **expression** in jq, not a filter; it transforms each entry into a `true`/`false` literal. Downstream `wc -l` counts boolean literals (1 per filtered entry), so the count appears correct for some windows but the data shape is wrong (boolean stream, not entry stream). For an empty window (e.g., CH-24 retro's spot-checked window), the count becomes 0 because no entry passes the time filter — masking the predicate bug from casual inspection.

**Fix**: the `and` operator in jq is valid + folds the version filter into a single `select` predicate. After the fix, the pipeline outputs entries (compact JSON, 1 line per entry), making `wc -l` give a true entry count.

**Skill version bump**: `3 → 4`; front-matter note appended explaining the fix.

**Pre-existing-behaviour preservation note** (never-shipped-yet variation form per chunk-planner v11): *"Pre-existing absence preserved: the skill returned `0` for non-empty windows due to the `jq` predicate bug at line 31 of `.claude/skills/permissions-audit.md` (`select(...) | .version == 1` evaluates a boolean instead of filtering); CH-25 ratifies the fix as canonical convention — replace with single-predicate `select(.ts >= start AND .ts <= end AND .version == 1)` form. No prior behaviour changes (the skill never returned correct counts for any window; CH-24 retrospective's `0` count is the first observed manifestation)."*

### §D60.6 — Edge-count cardinality evolution META

**Decision**: document the `EDGE_KIND_NAMES` invariant evolution `66 → 71 → 72` across milestones with an amendment trail in two locations:

| Site | Pre-CH-25 | Post-CH-25 |
|---|---|---|
| `domain/src/model/edges.rs:521-524` narrative comment | "67 at M3 close, +2 at M4/P1 (HasSubproject, HasConfig), +2 at CH-23 (Manages, HasAgentSupervisor) = 71" | "67 at M3 close, +2 at M4/P1, +2 at CH-23, +1 at CH-25 (Owns) = 72" |
| `D-philosophy-01.md:27` drift text | "Edge enum at ... (66 variants)" | "Edge enum at ... (66 → 71 (pre-CH-25 reality) → 72 (post-CH-25 NEW Owns variant))" |

**Cardinality literal sites** (9 in total — verified at plan P0 baseline):
1. `domain/src/model/edges.rs:525` — `pub const EDGE_KIND_NAMES: [&str; 71]` → 72.
2. `domain/src/model/edges.rs:658` — test name `edge_kind_names_is_exactly_seventy_one` → `_seventy_two`.
3. `domain/src/model/edges.rs:662` — assertion `assert_eq!(EDGE_KIND_NAMES.len(), 71)` → 72.
4. `domain/src/model/edges.rs:668` — assertion `assert_eq!(set.len(), 71)` (distinct-names test) → 72.
5. `domain/src/model/mod.rs:80` — test name `ontology_has_seventy_one_edge_kinds` → `_seventy_two`.
6. `domain/src/model/mod.rs:84` — assertion `assert_eq!(EDGE_KIND_NAMES.len(), 71)` → 72.
7. `domain/tests/m3_model_counts.rs:17` — test name `edge_count_bumps_from_sixty_six_to_seventy_one` → `_seventy_two` (or kept stable with updated body; final naming at P1).
8. `domain/tests/m3_model_counts.rs:23` — comment `[&str; 71]` → 72.
9. `domain/tests/m3_model_counts.rs:26` — assertion `assert_eq!(EDGE_KIND_NAMES.len(), 71)` → 72.

(Plan §3 Artifact B enumerated 4 sites — discovered at P0 baseline that there are actually 9 literal sites once test-fn names are counted. This is within tolerance; documented for completeness.)

---

## §2 — Cross-references

- (a) **Originating concept-doc + section + line range**: `concepts/core-philosophy.md:9,23,24,31,32` + `concepts/agent.md:4` (§"Agent spawns other Agents (Ownership)" amendment) + `concepts/permissions/04-grants.md:<§"Auto-issue rules">` + `concepts/permissions/01-resource-ontology.md:<§"Principals + Resources">`.
- (b) **Closed drift(s) by ID**: `D-philosophy-01` (HIGH-A).
- (c) **Prior ADRs cited as precedent**:
  - [`m1/decisions/0015-type-safe-ownership-edges.md`](../../m1/decisions/0015-type-safe-ownership-edges.md) — the typed-Edge-constructor + Principal/Resource sealing pattern CH-25 extends with a new variant + new typed constructor.
  - [`m1/decisions/0008-permission-check-as-pipeline.md`](../../m1/decisions/0008-permission-check-as-pipeline.md) — the 6-step pipeline CH-25's F3.a extends.
  - [`m3/decisions/0022-org-creation-compound-transaction.md`](../../m3/decisions/0022-org-creation-compound-transaction.md) — the compound-tx pattern CH-25 extends at `apply_org_creation`.
  - [`m5_2/decisions/0039-human-agent-identity-guard.md`](../../m5_2/decisions/0039-human-agent-identity-guard.md) — the `disable_system_agent` ADR for the acceptance test's child-disable path.
  - [`m5_2/decisions/0053-system-genesis-authority-chain-revocation-cascade.md`](../../m5_2/decisions/0053-system-genesis-authority-chain-revocation-cascade.md) — CH-14 system-genesis precedent for synth-grants from axiomatic principals — analogous to owner-grant synthesis.
- (d) **Forward-scope row**: [`forward-scope/remaining-scope-post-m5-p7-22035b2a.md`](../../../../plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md) §2.5 lines 242-254.

---

## §3 — Consequences

**Positive**:

- Philosophy alignment with concept doc `core-philosophy.md` lines 9, 23, 24, 31, 32 — D-philosophy-01 closed.
- M6 admin pages 1 (Orgs CRUD) + 3 (Projects CRUD) can plan against a unified owner-grant model rather than bespoke per-handler permission gates.
- Provenance chain (Agent → CREATED → Org/Project) is captured for the first time; future tooling can traverse the chain for audit + compliance queries.
- Owner-grant rule is canonical (not bespoke) — the Permission Check engine handles owner-Agent admin authority uniformly across Orgs and Projects.

**Negative / cost**:

- New SurrealDB relation table `owns` requires explicit migration `0017_add_owns_relation.surql` (A4 axis fires — verified at P0).
- 9 literal-cardinality sites must flip 71 → 72 in lockstep (cascade discipline at P1).
- 2 new Repository methods (`list_agent_owned_orgs/projects`) extend the trait surface — both backends must implement.
- `CheckContext` gains 2 new fields (`agent_owned_orgs/projects`) — cascade across ~6 production construction sites (engine.rs:2227/2278, manifest.rs definition, session launch/preview loaders, the new test-fixture builder).
- `OrgCreationPayload` + `ProjectCreationPayload` extended with `creator_agent: AgentId` — cascade across ~3 + ~4 construction sites respectively.

**Neutral**:

- `Action::CANONICAL.len() == 34` invariant preserved (F2 re-interprets to canonical verbs).
- `phi-core` import count unchanged (Δ +0).
- No new K8s blocker class (per §3.B verification at chunk-open).
- 5 existing trybuild `compile_fail` fixtures STAY GREEN (Org stays Principal-only under F1.b).

---

## §4 — Implementation phases

Per [plan §7](../../../../plan/build/ch-25-agent-as-creator-and-owner-1e01618e/plan.md):

- **P0** — Scaffolding + pre-conditions re-verify + this ADR drafted Proposed.
- **P-R5-INVESTIGATE** — permissions-audit skill jq predicate fix.
- **P1** — NEW `Edge::Owns` variant + `OwnedResourceId` enum + EDGE_KIND_NAMES 71→72 cascade + emit sites at `apply_org_creation` + `apply_project_creation` + 0017 migration.
- **P2** — Owner-grant rule synthesis in `step_2_resolve_grants` + CheckContext cascade + 2 new Repository methods.
- **P3** — User-facing docs + acceptance test (`acceptance_m5_3_owner_grant.rs`).
- **P-SEAL** — Flip ADR to Accepted; flip D-philosophy-01 to remediated; cycle-index row; verified-headers.

---

## §5 — Audit envelope

Per [plan §11](../../../../plan/build/ch-25-agent-as-creator-and-owner-1e01618e/plan.md): **3 auditors (F6.b user-locked)** — Audit A (code + phi-core + edge-variant cascade), Audit B (docs + concept alignment), Audit C (cross-cutting: governance + Pre-existing-behaviour notes + R5 fix + R2/R3 + 71→72 verification).

---

## §6 — Pause-and-escalate triggers fired during P0

None. P0 baseline:

- Working tree clean (only plan folder untracked).
- Baseline test count **1537** (matches plan §6 expected).
- §6 regression-table commands all green: `Action::Transfer` present; `ScopeTier::Agent` referenced; `system_genesis_principal` present; CH-23 edge methods present; `set_agent_active` present.
- **A4 axis verdict**: SurrealDB DOES require explicit DEFINE TABLE for relations (verified `0001_initial.surql:379-381` shows explicit declarations for owned_by/created/allocated_to; `0011_manages_supervisor_edges.surql` shows CH-23 added explicit definitions for new edges). **Migration `0017_add_owns_relation.surql` is REQUIRED at P1**. Plan §3.B already anticipated this conditional; no architectural surprise — this is the expected path per ADR-0033 K8s seams + ADR-0012 migration discipline.
- **C1 verdict**: bootstrap-claim does NOT create an Org (verified `claim.rs:163` shows `owning_org: None`; no `create_organization` call). No re-routing required; C1 stays closed.
- **D-philosophy-01**: extant at `discovered`; line 27 cites "66 variants" (stale — will be amended at P-SEAL §D60.6).
- **ADR-0060**: free (highest existing ADR is 0059).
