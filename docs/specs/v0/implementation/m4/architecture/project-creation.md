<!-- Last verified: 2026-04-23 by Claude Code -->

# Architecture — Project Creation Wizard (page 10)

**Status: [EXISTS]** — landed at M4/P6 (Shape A full, Shape B 4-outcome state machine; Shape B **materialisation-after-both-approve** deferred to M5 per C-M5-6).

Page 10 covers the **two project shapes**:

- **Shape A** — single owning org, immediate materialisation.
- **Shape B** — co-owned by two orgs; submit creates a 2-approver pending `AuthRequest`; project materialises only on both-approve (M5) or is killed on any non-Approved terminal state (M4 ships this path fully).

Template A grant fires **automatically** via the domain event bus (ADR-0028) on every `HAS_LEAD` edge write.

## Surfaces

| Tier | Path | Entry point |
|---|---|---|
| HTTP (create) | `POST /api/v0/orgs/:org_id/projects` | `server/src/handlers/projects.rs::create` |
| HTTP (approve) | `POST /api/v0/projects/_pending/:ar_id/approve` | `server/src/handlers/projects.rs::approve_pending` |
| CLI (create) | `phi project create --org-id --name --shape shape_a\|shape_b [--co-owner-org-id] --lead-agent-id [--member-ids csv] [--okrs-file path]` | `cli/src/commands/project.rs::create_impl` |
| CLI (approve) | `phi project approve-pending --ar-id --approver-id [--deny]` | `cli/src/commands/project.rs::approve_pending_impl` |
| Web | `(admin)/organizations/[id]/projects/new` | SSR + Server Action; 6-section form |

Business logic: `server/src/platform/projects/create.rs`.

## Submit flow (Shape A)

1. Validate payload (`name` non-empty, `shape=A` must not supply `co_owner_org_id`).
2. Validate org + lead + members exist and belong to owning org(s).
3. Validate OKRs (`measurement_type.is_valid_value(value)` on every KR; no duplicate ids; every KR references an existing Objective).
4. Call `Repository::apply_project_creation` — single BEGIN/COMMIT that writes Project + `HAS_LEAD` + `HAS_AGENT`* + `HAS_SPONSOR`* + `BELONGS_TO` edges atomically.
5. Emit `platform.project.created` (Alerted).
6. Emit `DomainEvent::HasLeadEdgeCreated` on the bus → the `TemplateAFireListener` (ADR-0028) issues the lead grant + emits `TemplateAAdoptionFired` audit asynchronously.

HTTP response: `201 Created` with `{outcome: "materialised", project_id, lead_agent_id, has_lead_edge_id, owning_org_ids, audit_event_id}`.

## Submit flow (Shape B)

1. Validate payload (`co_owner_org_id` required, must not equal `org_id`).
2. Resolve both owning orgs; validate lead/members belong to *either* owning org.
3. Validate OKRs.
4. Resolve the two approver agents — first Human-kind agent in each owning org (the CEO at org-creation time).
5. Build a `Pending` `AuthRequest` with two `ApproverSlot`s in `Unfilled` state. Persist via `Repository::create_auth_request`.
6. Emit `platform.project_creation.pending` (Logged).

HTTP response: `202 Accepted` with `{outcome: "pending", pending_ar_id, approver_ids, audit_event_id}`.

## Approve flow (Shape B)

Each approver drives their slot via `POST /api/v0/projects/_pending/:ar_id/approve` with body `{approver_id, approve: true|false}`. The handler:

1. Loads the AR; verifies `kinds` contains `"#shape:b:project_creation"`; rejects terminal-state ARs with `PENDING_AR_ALREADY_TERMINAL`.
2. Locates the approver's slot; rejects with `APPROVER_NOT_AUTHORIZED` if the caller isn't listed.
3. Calls `auth_requests::transitions::transition_slot` — the state machine aggregates the new resource + request state automatically.
4. Persists the updated AR via `Repository::update_auth_request`.
5. If the request is still active (`Pending` / `InProgress`), returns `200 OK` with `{outcome: "still_pending", ar_id}`.
6. If terminal: emit `platform.project_creation.denied` for `Denied`/`Partial` terminals; on `Approved` the handler returns `{outcome: "terminal", state: "approved", project_id: null}` — **materialisation-after-approve deferred to M5/C-M5-6** (see below).

## The 4-outcome decision matrix (Shape B)

| Slot 1 | Slot 2 | AR terminal | Project materialises? | Audit emitted at terminal |
|---|---|---|---|---|
| Approve | Approve | `Approved` | ✅ (via M5/C-M5-6 — at M4 returns null) | none at M4 (M5 adds `platform.project.created`) |
| Approve | Deny | `Partial` | ❌ | `platform.project_creation.denied` |
| Deny | Approve | `Partial` | ❌ | `platform.project_creation.denied` |
| Deny | Deny | `Denied` | ❌ | `platform.project_creation.denied` |

The 4-outcome matrix is pinned at three tiers:

1. **Domain proptest** — `domain/tests/shape_b_approval_matrix_props.rs` (50 cases) drives the state machine through every `(slot1, slot2)` combination.
2. **Server acceptance** — `server/tests/acceptance_projects_create.rs` scenarios for each outcome against a live HTTP stack.
3. **Pure decision helper** — `create::should_materialize_project_from_ar_state` (unit-tested; mirrors the proptest predicate) so the approval handler never re-derives the condition.

## Deferred to M5 (C-M5-6)

Shape B's Approved branch currently returns `project_id: null` because the `CreateProjectInput` (name, OKRs, leads, members, token_budget, resource_boundaries) captured at submit time is not persisted anywhere the approve handler can read back. M5 adds a `shape_b_pending_projects` sidecar table (migration 0005), repo methods to write/read/delete it, and flips the approve-handler's Approved branch to call `materialise_project` with the reconstructed input. At M4 the state machine + all error audits fire correctly; only the materialisation call is gated. See base build plan §C-M5-6.

## phi-core leverage (Q1 / Q2 / Q3)

- **Q1 direct imports: 0.** Grep `use phi_core::` in `server/src/platform/projects/` → empty (only `resolvers.rs` had transitive use; the M4/P6 `create.rs` adds none). Project creation is pure phi governance; phi-core has no Project / OKR / ProjectShape concept.
- **Q2 transitive: 0 at the P6 wire tier.** The request body + response body carry baby-phi-governance types only. Agent rows (with `blueprint: phi_core::AgentProfile`) are surfaced by page 08's roster endpoint, not by page 10.
- **Q3 rejections:** `phi_core::Session` (M5), `phi_core::AgentEvent` (agent-loop telemetry — orthogonal to `DomainEvent`), `ContextConfig` / `RetryConfig` (inherit-from-snapshot per ADR-0023). All documented.

Positive close-audit grep:

```bash
grep -En '^use phi_core::' modules/crates/server/src/platform/projects/create.rs
# → 0 lines (invariant)
```

## Invariants

1. **Arity enforcement** — `apply_project_creation` rejects `owning_orgs.len() != 1` for Shape A and `!= 2` for Shape B. Tested by `store/tests/apply_project_creation_tx_test.rs::shape_b_arity_violation_is_rejected_before_open_tx`.
2. **4-outcome matrix** — pinned by 50-case proptest + 4 acceptance scenarios.
3. **Template A fires** — every successful Shape A materialisation (or M5's Shape B materialisation) emits `HasLeadEdgeCreated`; the listener persists the lead grant + emits `TemplateAAdoptionFired`.
4. **Pre-tx validation** — org existence, lead/member membership, OKR shape all checked BEFORE opening the compound tx so rollback on validation failure is not required.

## References

- [ADR-0025 — Shape B two-approver flow](../decisions/0025-shape-b-two-approver-flow.md)
- [ADR-0028 — Domain event bus + Template A subscription](../decisions/0028-domain-event-bus.md)
- [Shape A vs Shape B](shape-a-vs-shape-b.md)
- [Event bus architecture](event-bus.md)
- [Template A firing](template-a-firing.md)
- [Project model](project-model.md)
- [Requirements admin/10](../../../requirements/admin/10-project-creation-wizard.md)
- [phi-core leverage map §Page 10](phi-core-reuse-map.md)
- [Project creation ops runbook](../operations/project-creation-operations.md)
- [M4 plan archive §P6](../../../../plan/build/a634be65-m4-agents-and-projects.md)
