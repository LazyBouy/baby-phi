<!-- Last verified: 2026-04-30 by Claude Code -->

# CH-23 — Template C/D HTTP edge handlers + end-to-end acceptance

**Plan file token:** `60a2eb09` (generated 2026-04-30 at chunk-open via `openssl rand -hex 4`).
**Plan archive path (verbatim copy):** `baby-phi/docs/specs/plan/build/ch-23-template-cd-http-edges-and-e2e-60a2eb09.md` (per the new slug-first convention).
**Chunk ID:** CH-23.
**Severity:** ⚠ MEDIUM (the chunk closes a real gap; the gap is not load-bearing for M5 close).
**Expected effort:** ~3 engineer-days. The forward-scope row estimated 0.5d under the assumption that HTTP emitters for `MANAGES` + `HAS_AGENT_SUPERVISOR` already existed; plan-time exploration disproved that. Per user decision (2026-04-30) we widen the chunk to ship the missing HTTP handlers + Edge variants alongside the verification suite.
**Hard prerequisites:** CH-21 + CH-22 (both sealed); ADR-0034 (durable agent lifecycle — disable/archive must be honored before edge creation).
**Chunks unblocked at close:** any future chunk that needs to mutate manager/supervisor relationships from the API surface.

---

## Context

### The simple version

Template A (Project Lead) has a real production HTTP path: `platform/projects/create.rs:417` emits `DomainEvent::HasLeadEdgeCreated` after the compound tx commits, and the `TemplateAFireListener` mints the manager grant. Templates C (`MANAGES`) and D (`HAS_AGENT_SUPERVISOR`) have listeners + property tests + adoption builders, but **zero production emit sites** — there is no HTTP, CLI, or background path that writes the underlying edge or fires the trigger event. The listeners fire only in unit tests that construct the event by hand.

CH-23 closes that gap. It ships the two missing edges as first-class graph variants, two compound-tx repo methods that create the edge + emit the audit, two HTTP handlers, and an end-to-end acceptance suite (`acceptance_system_flows_s05.rs`) with four scenarios.

### What this chunk does NOT do

- Does NOT add DELETE / unset endpoints (un-assigning a manager / supervisor). Per concept-doc 07 the trigger is forward-only at v0; revocation is a separate operation. Deferred.
- Does NOT mutate the existing `TemplateAFireListener` / `TemplateCFireListener` / `TemplateDFireListener` bodies. They already fire correctly — the gap is upstream emitters, not downstream listeners.
- Does NOT add CLI commands for the new endpoints. The CLI surface for org-mutation lives in M6 per the build plan; CH-23 stays on the HTTP API.
- Does NOT add a Web UI for setting manager / supervisor relationships. Same reasoning.
- Does NOT validate cross-org or cross-project boundary rules beyond same-org / same-project membership. Tighter contractor-model rules are owned by D-new-20 (out of scope).

### User-decided forks (locked at plan-review, 2026-04-30)

1. **Scope** — ship HTTP edge handlers as part of CH-23 (not bus-level only, not deferred). Closes the production gap as the test does.
2. **Drift tracking** — N/A. The HTTP gap closes inside CH-23, no new drift entry needed.
3. **Route shape (Q3)** — Per-relationship REST: `POST /api/v0/orgs/:org/agents/:agent/manager` + `POST /api/v0/projects/:project/agents/:agent/supervisor`. The relationship is a sub-resource of the agent; conventional REST; no generic edge concept exposed in the public API.
4. **Edge field carrier (Q4)** — Carry `org` / `project` on the edge: `Edge::Manages { id, from, to, org }` + `Edge::HasAgentSupervisor { id, from, to, project }`. Listener gets the scope directly, no `member_of` lookup at event-emit time.
5. **Repository method shape (Q5)** — Two dedicated methods + typed receipts: `Repository::create_manages_edge -> ManagesEdgeReceipt { edge_id, audit_event_id }` + `Repository::create_has_agent_supervisor_edge -> HasAgentSupervisorEdgeReceipt`. Mirrors the `HasLead` precedent at `repository.rs:300`.

### Forward-scope reference

[`forward-scope/remaining-scope-post-m5-p7-22035b2a.md`](baby-phi/docs/specs/plan/forward-scope/remaining-scope-post-m5-p7-22035b2a.md) §1 lines 203–207 + §5 inventory row line 431.

---

## §1 — Why this chunk (one paragraph)

The forward-scope row scoped CH-23 as a 0.5-day verification of Template C/D listener wiring; plan-time exploration found that the listeners are unreachable from any production code path because the trigger events have no emitters. The unit + property tests prove the listener bodies work in isolation; they do not prove anything end-to-end. CH-23 closes the gap by shipping `Edge::Manages` + `Edge::HasAgentSupervisor` as first-class variants, a migration that adds the two `manages` + `has_agent_supervisor` SurrealDB tables, two compound-tx repo methods that create the edge + emit the audit + return a typed receipt (mirroring the `HasLead` precedent), two HTTP handlers behind the existing CEO/admin auth, and the originally-scoped `acceptance_system_flows_s05.rs` with four scenarios (Template C end-to-end, Template D end-to-end, A+C+D simultaneous, cross-listener ordering). The chunk produces real production-grade coverage instead of theatre.

---

## §2 — Concept alignment walk

| Concept doc | § anchor | Claim | Status at chunk-open | Status at chunk-close |
|---|---|---|---|---|
| [`permissions/07-templates-and-tools.md`](baby-phi/docs/specs/v0/concepts/permissions/07-templates-and-tools.md) | §"Template C — Manager Grant" | Template C fires when a `MANAGES` edge is created within an org, minting a `[read, inspect]` grant on the subordinate's agent fundamental | partially-honored — listener exists; no producer | honored — `POST /api/v0/orgs/:org/agents/:agent/manager` writes the edge + emits `ManagesEdgeCreated`, listener fires, grant persists |
| `permissions/07-templates-and-tools.md` | §"Template D — Supervisor Grant" | Template D fires when a `HAS_AGENT_SUPERVISOR` edge is created within a project, minting a `[read, inspect]` grant on the supervisee's project-scoped agent fundamental | partially-honored — listener exists; no producer | honored — `POST /api/v0/projects/:p/agents/:agent/supervisor` writes the edge + emits the trigger |
| `ontology.md` | §"Edges" | `MANAGES` + `HAS_AGENT_SUPERVISOR` are first-class graph edges between Agents | silent-in-code (variants don't exist on `Edge` enum) | honored — both variants land on `Edge` with type-safe constructors |

---

## §3 — phi-core leverage map

| phi-core type | Action in this chunk |
|---|---|
| (none) | — |

CH-23 is baby-phi-native. phi-core has no concept of org-internal management hierarchies or project-scoped supervision. Zero `use phi_core::` imports added or removed.

**Positive close-audit greps**:
```bash
grep -n "Edge::Manages\b\|Edge::HasAgentSupervisor\b" modules/crates/domain/src/model/edges.rs   # ≥ 2 each
grep -n "create_manages_edge\|create_has_agent_supervisor_edge" modules/crates/domain/src/repository.rs   # ≥ 2 each
ls modules/crates/store/migrations/0011_*.surql                                                  # exists
ls modules/crates/server/tests/acceptance_system_flows_s05.rs                                    # exists
grep -n "POST.*agents/:.*manager\|POST.*agents/:.*supervisor" modules/crates/server/src/lib.rs   # ≥ 2 routes registered
```

**Forbidden-duplication / regression greps**:
```bash
grep -rn "use phi_core::" modules/crates/server/src/platform/orgs/agents/manager.rs              # 0
grep -rn "use phi_core::" modules/crates/server/src/platform/projects/agents/supervisor.rs       # 0
```

---

## §3.B — K8s readiness check

| Axis | This chunk's surface | New blocker? |
|---|---|---|
| **A1** in-process state | None new; the in-process `EventBus` already broadcasts to all subscribed listeners. | No |
| **A2** IPC channels | New HTTP routes are stateless behind the existing CEO-session JWT layer. | No |
| **A3** pod-local resources | None. | No |
| **A4** migration runner | New migration 0011 (`0011_manages_supervisor_edges.surql`) defines the two tables. Idempotent under repeated runs (per the `_migrations` ledger). | No |
| **A5** trait-shape requirement | Two new `Repository` methods. Both backends (`InMemoryRepository`, `SurrealStore`) implement them. | No |
| **A6** cross-pod state sharing | The new edges live in SurrealDB; queries span pods identically post-migration. | No |
| **A7** audit hash-chain symmetry | The new audit events `manages.edge.created` + `has_agent_supervisor.edge.created` follow the existing canonical-bytes pattern (BLAKE3). The listener-emitted `template.c.grant_fired` + `template.d.grant_fired` already exist and stay byte-stable. | No |

**Conclusion:** K8s-neutral. No M7b ledger entry added.

---

## §3.C — User-facing documentation impact

| Tier | File | Action |
|---|---|---|
| Concept | [`permissions/07-templates-and-tools.md`](baby-phi/docs/specs/v0/concepts/permissions/07-templates-and-tools.md) | Verified-header bump noting CH-23 lifts the Template C / D HTTP path into production. Doc body UNCHANGED. |
| Concept | [`ontology.md`](baby-phi/docs/specs/v0/concepts/ontology.md) | Verified-header bump noting `MANAGES` + `HAS_AGENT_SUPERVISOR` Edge variants now exist. Doc body UNCHANGED. |
| Decision | `m5_2/decisions/0046-template-cd-http-edges.md` (NEW) | Full ADR — see §5. |
| Architecture | [`m1/architecture/permission-check-engine.md`](baby-phi/docs/specs/v0/implementation/m1/architecture/permission-check-engine.md) | Optional verified-header bump if the Template-C/D end-to-end path was previously documented as "PLANNED"; check before edit. |
| Operations | (none) | The new endpoints follow existing CEO-session auth — no new ops doc needed. |
| User-guide | (none) | Web UI for these flows is M6+. |

---

## §4 — Drifts closed

| Drift ID | File | Severity | Transition |
|---|---|---|---|
| (none) | — | — | — |

CH-23 does not close a tracked drift — the gap was undocumented at plan-open. Per locked Q2 (2026-04-30), the gap is closed inside CH-23 itself rather than tracked as a new drift.

**Index updates:** none — `drifts/README.md` and `_concept-audit-matrix.md` untouched.

---

## §5 — ADR drafted

ADR numbering: highest issued = ADR-0045 (CH-09). Next-free = **ADR-0046**.

| ADR | Title | Decision summary |
|---|---|---|
| **ADR-0046** | Template C/D HTTP edge handlers + production-grade trigger emission | **D46.1** Two new `Edge` variants land at `modules/crates/domain/src/model/edges.rs`: `Manages { id: EdgeId, from: AgentId, to: AgentId, org: OrgId }` and `HasAgentSupervisor { id: EdgeId, from: AgentId, to: AgentId, project: ProjectId }`. Per locked Q4: `org` and `project` are carried on the edge so the listener's event payload fields can be populated without a follow-up lookup. **D46.2** Two new `Repository` trait methods land at `modules/crates/domain/src/repository.rs`: `create_manages_edge(org, manager, subordinate) -> RepositoryResult<ManagesEdgeReceipt { edge_id, audit_event_id }>` and `create_has_agent_supervisor_edge(project, supervisor, supervisee) -> RepositoryResult<HasAgentSupervisorEdgeReceipt { ... }>`. Both follow the compound-tx-receipt pattern from `HasLead` (`repository.rs:300`). Per locked Q5. **D46.3** Migration `modules/crates/store/migrations/0011_manages_supervisor_edges.surql` adds two SCHEMAFULL tables: `manages` (id, org_id, manager, subordinate, created_at) + `has_agent_supervisor` (id, project_id, supervisor, supervisee, created_at) with UNIQUE indexes on `(org_id, manager, subordinate)` and `(project_id, supervisor, supervisee)` to prevent duplicate edges. **D46.4** Two new HTTP handlers at `modules/crates/server/src/platform/orgs/agents/manager.rs` (route `POST /api/v0/orgs/:org/agents/:agent/manager`) + `modules/crates/server/src/platform/projects/agents/supervisor.rs` (route `POST /api/v0/projects/:project/agents/:agent/supervisor`). Per locked Q3 — per-relationship REST resource shape, not a generic `/edges` endpoint. Both behind existing CEO-session auth + admin role check. **D46.5** Both handlers emit `DomainEvent::ManagesEdgeCreated` / `HasAgentSupervisorEdgeCreated` AFTER the compound tx + audit, mirroring the `HasLead` post-commit emit pattern at `projects/create.rs:417`. **D46.6** Validation rules at handler level: (a) target agent + manager/supervisor exist, (b) target agent + manager/supervisor share an org membership (Manages) or both belong to the project (HasAgentSupervisor), (c) target agent's `active = true`, (d) idempotency — a re-POST of the same triple returns 200 + the existing edge id rather than 409. **D46.7** Acceptance suite at `modules/crates/server/tests/acceptance_system_flows_s05.rs` ships 4 scenarios per the forward-scope row: (1) Template C fires on MANAGES, (2) Template D fires on HAS_AGENT_SUPERVISOR, (3) A+C+D simultaneous (a project lead with manager + supervisor relationships triggers all three listeners), (4) cross-listener ordering — verifies the in-process bus dispatches events in subscription-registration order, with each listener's audit emission interleaved deterministically. **D46.8** No CLI / Web UI surface in CH-23 — locked at plan-review, M6+ owns those layers. **D46.9** No DELETE endpoints in CH-23 — un-assigning manager / supervisor is a separate operation; deferred to whichever future chunk owns the wider org-mutation surface. **D46.10** Same-org / same-project membership check is at handler boundary; tighter cross-scope contractor rules (D-new-20) remain out of scope. |

ADR file: [`m5_2/decisions/0046-template-cd-http-edges.md`](baby-phi/docs/specs/v0/implementation/m5_2/decisions/0046-template-cd-http-edges.md) (NEW).

---

## §6 — Prior-chunk regression re-verification

| Upstream | Invariant | Verification |
|---|---|---|
| Post-CH-09 baseline | `cargo test --workspace -- --test-threads=1` ≈ 1198; 4 CI guards green; clippy clean under `-Dwarnings` | `bash scripts/{check-doc-links,check-ops-doc-headers,check-phi-core-reuse,check-spec-drift}.sh`<br>`cargo test -j 4 --workspace -- --test-threads=1` |
| CH-21 / ADR-0040 + 0041 | Memory-extraction listener body intact | `cargo test -j 4 -p server --test acceptance_memory_extraction -- --test-threads=1` |
| CH-22 / ADR-0035 | Agent-catalog listener body intact | `cargo test -j 4 -p server --test acceptance_system_flows_s03 -- --test-threads=1` |
| CH-01 / ADR-0034 | `Agent.active` durable-state guard intact — handler must reject manager/supervisor assignment to an archived agent | new acceptance test scenario covers this |
| Migration runner | Migration 0011 applies once + safe to re-run | `cargo test -j 4 -p store --test migrations_test -- --test-threads=1` (extend the existing test to cover version 11) |
| Audit hash chain | BLAKE3 chain stays byte-stable for existing event types | `cargo test -j 4 -p server --test acceptance_memory_extraction -- --test-threads=1` |

---

## §7 — Phases

**Phase count: 4** → audit envelope = **2 agents** (medium-large chunk).

### P1 — Edge variants + Repository methods + migration (~1d)

**Goal.** Land the two new `Edge` variants + their repo methods + migration 0011 + extend the migrations integration test. End-state: workspace builds; all P1 unit tests pass; existing tests pass.

**Deliverables.**

1. **Add `Edge::Manages` + `Edge::HasAgentSupervisor`** at [`modules/crates/domain/src/model/edges.rs`](baby-phi/modules/crates/domain/src/model/edges.rs) following the `HasLead` pattern (lines 462 etc.). Add to the `as_str()` match arm and the `EDGE_TYPE_NAMES` const.

2. **Two new `Repository` trait methods** at [`modules/crates/domain/src/repository.rs`](baby-phi/modules/crates/domain/src/repository.rs):
   ```rust
   async fn create_manages_edge(
       &self,
       org: OrgId,
       manager: AgentId,
       subordinate: AgentId,
       at: DateTime<Utc>,
       actor: ActorRef,
   ) -> RepositoryResult<ManagesEdgeReceipt>;

   async fn create_has_agent_supervisor_edge(
       &self,
       project: ProjectId,
       supervisor: AgentId,
       supervisee: AgentId,
       at: DateTime<Utc>,
       actor: ActorRef,
   ) -> RepositoryResult<HasAgentSupervisorEdgeReceipt>;
   ```
   Each method's compound-tx body: validate inputs (agents exist + active + same scope), insert the edge row, emit the audit event, return the receipt. Mirrors the `HasLead` precedent.

3. **Migration `0011_manages_supervisor_edges.surql`** at `modules/crates/store/migrations/`:
   - `DEFINE TABLE manages SCHEMAFULL;` + 5 fields + `DEFINE INDEX manages_unique ON manages FIELDS org_id, manager, subordinate UNIQUE;`
   - `DEFINE TABLE has_agent_supervisor SCHEMAFULL;` + 5 fields + `DEFINE INDEX has_agent_supervisor_unique ON has_agent_supervisor FIELDS project_id, supervisor, supervisee UNIQUE;`
   - Register in `EMBEDDED_MIGRATIONS` at version 11, slug `manages_supervisor_edges`.
   - Extend `migrations_test.rs` to expect 11 rows + version 11 / slug `manages_supervisor_edges`.

4. **Implement both methods on both backends** — `InMemoryRepository` (HashMaps) + `SurrealStore` (compound-tx via SurrealDB transaction).

**Tests (P1).** ~12 unit + integration tests:
- Edge variant `as_str()` for both new variants
- `Edge::EDGE_TYPE_NAMES` count bumps + roundtrip
- Repository.create_manages_edge happy-path (in-memory)
- Repository.create_has_agent_supervisor_edge happy-path (in-memory)
- Both backends reject when target agent is archived (`active=false`)
- Both backends reject when manager/supervisor is in different org/project
- Idempotency: re-create returns same edge id, no duplicate row
- SurrealStore migration applies cleanly + UNIQUE index rejects duplicates
- Audit event emitted with stable canonical bytes

**Confidence target.** ≥ 95%.

**Pause discipline.** PAUSE if:
- The compound-tx primitive used by `create_project` ([`projects/create.rs:417`](baby-phi/modules/crates/server/src/platform/projects/create.rs)) doesn't generalize cleanly to per-edge creation — may need a dedicated transaction helper at `repo_impl.rs`.
- Migration 0011 conflicts with an unanticipated existing `manages` or `has_agent_supervisor` placeholder table in `0001_initial.surql` (verify before applying).
- The UNIQUE-index pattern fails under SurrealDB's actual edge-row semantics (test on embedded backend before declaring done).

---

### P2 — HTTP handlers + route registration (~0.7d)

**Goal.** Land the two HTTP handlers behind existing CEO-session auth + register the routes. End-state: handlers respond 201 / 200 / 4xx correctly; route table tests pass.

**Deliverables.**

1. **`platform/orgs/agents/manager.rs`** (NEW) — `POST /api/v0/orgs/:org/agents/:agent/manager` body `{manager_id: AgentId}`:
   - CEO-session auth + admin role check
   - Validate manager + subordinate both belong to the org + both `active=true`
   - Call `Repository::create_manages_edge`
   - Emit `DomainEvent::ManagesEdgeCreated` AFTER receipt returns
   - Return 201 + `{edge_id, audit_event_id}`
   - Return 200 + same body if edge already exists (idempotent)
   - Return 400 if same agent assigned as manager (self-loop)
   - Return 404 if either agent doesn't exist
   - Return 409 if either agent is archived

2. **`platform/projects/agents/supervisor.rs`** (NEW) — `POST /api/v0/projects/:project/agents/:agent/supervisor` body `{supervisor_id: AgentId}`:
   - Same shape, scoped to project membership.

3. **Route registration** at the existing axum router (likely `server/src/lib.rs` or `server/src/router.rs`).

4. **Handler unit tests** — happy + each 4xx case, against `InMemoryRepository`.

**Tests (P2).** ~8 handler-level tests + route-table inclusion test.

**Confidence target.** ≥ 95%.

**Pause discipline.** PAUSE if the existing CEO-session auth middleware doesn't cleanly cover `/orgs/:org` and `/projects/:project` scoped routes (some routes may have scope-specific auth checks the handlers must call manually).

---

### P3 — `acceptance_system_flows_s05.rs` (4 scenarios) (~0.5d)

**Goal.** Ship the originally-scoped end-to-end test file using the new HTTP path.

**Deliverables.** [`modules/crates/server/tests/acceptance_system_flows_s05.rs`](baby-phi/modules/crates/server/tests/acceptance_system_flows_s05.rs) (NEW). Pattern follows `acceptance_system_flows_s03.rs`:

1. **Scenario 1 — Template C fires on MANAGES.** Bootstrap claimed org with a manager + subordinate Agent; subscribe `TemplateCFireListener`; POST manager assignment; assert (a) `manages` edge persisted, (b) `Grant` minted on `agent:<subordinate-uuid>` held by manager, (c) audit chain has both `manages.edge.created` + `template.c.grant_fired` events with correct provenance.

2. **Scenario 2 — Template D fires on HAS_AGENT_SUPERVISOR.** Same pattern, project-scoped. Subscribe `TemplateDFireListener`; POST supervisor assignment; assert grant on `project:<p-uuid>/agent:<supervisee-uuid>`.

3. **Scenario 3 — A+C+D simultaneous.** Bootstrap a project with: lead agent (Template A trigger via project create) + manager assignment (Template C) + project-supervisor assignment (Template D). Subscribe all three listeners. Assert all three grants exist + each listener's audit fires exactly once + no cross-talk (Template A grant doesn't carry Template C/D scope semantics).

4. **Scenario 4 — cross-listener ordering.** Two listeners subscribed to the same event variant — assert deterministic dispatch order (subscription order). Use a shared atomic counter in test helpers to record observed order.

**Tests (P3).** 4 acceptance tests. Workspace total: post-CH-09 (1198) + ~12 P1 + ~8 P2 + 4 P3 = **~1222 tests**.

**Confidence target.** ≥ 95%.

**Pause discipline.** PAUSE if scenario 4 reveals the in-process `EventBus` has nondeterministic dispatch order — a real bug, would need to be tracked separately as a new drift before this scenario can pass.

---

### P4 — ADR Accepted + concept-doc bumps + audit + seal (~0.5d)

**Goal.** Ratify ADR-0046. Bump 2 concept-doc headers. Spawn 2 audit agents. Seal.

**Deliverables.**

1. ADR-0046 flipped from `Proposed` → `Accepted` at chunk seal.
2. Concept-doc verified-header bumps on `permissions/07` + `ontology.md`. Doc bodies UNCHANGED.
3. Spawn 2 audit agents per §11.

**Confidence target.** ≥ 99%.

**Pause discipline.** PAUSE if either audit reports a finding.

---

## §8 — Tests summary

- **Expected total at chunk close:** 1198 baseline + ~12 P1 + ~8 P2 + 4 P3 = **~1222 tests**.
- **New test files:** acceptance suite at `server/tests/acceptance_system_flows_s05.rs`; unit tests inline with each new module.
- **Cross-impl consistency:** Repository methods exercised against both `InMemoryRepository` and `SurrealStore::open_embedded` in P1.
- **Audit-chain canonical-bytes check:** ADR-0040/0041 invariant — re-run `acceptance_memory_extraction` to confirm the chain is stable.

---

## §9 — Pre-chunk gate

### Chunk-open Step 0 — Archive

1. Generate token: `openssl rand -hex 4`.
2. Copy plan verbatim to `baby-phi/docs/specs/plan/build/ch-23-template-cd-http-edges-and-e2e-<8hex>.md` (slug-first per the new convention saved to memory at CH-09 close).
3. Update placeholders in lines 4–5 of the archived copy.
4. `bash scripts/check-doc-links.sh`.

### Reading list (mandatory)

1. [`concepts/permissions/07-templates-and-tools.md`](baby-phi/docs/specs/v0/concepts/permissions/07-templates-and-tools.md) — §"Template C" + §"Template D".
2. [`modules/crates/domain/src/templates/c.rs`](baby-phi/modules/crates/domain/src/templates/c.rs) + [`d.rs`](baby-phi/modules/crates/domain/src/templates/d.rs) — full files.
3. [`modules/crates/domain/src/events/listeners.rs`](baby-phi/modules/crates/domain/src/events/listeners.rs) lines 143–500 — Template A/C/D listener bodies.
4. [`modules/crates/domain/src/events/mod.rs`](baby-phi/modules/crates/domain/src/events/mod.rs) lines 130–155 — DomainEvent variants.
5. [`modules/crates/domain/src/model/edges.rs`](baby-phi/modules/crates/domain/src/model/edges.rs) — Edge enum + the existing `HasLead` precedent.
6. [`modules/crates/server/src/platform/projects/create.rs`](baby-phi/modules/crates/server/src/platform/projects/create.rs) lines 395–430 — production HasLeadEdgeCreated emission pattern.
7. [`modules/crates/server/tests/acceptance_system_flows_s03.rs`](baby-phi/modules/crates/server/tests/acceptance_system_flows_s03.rs) — pattern reference for the new s05 file.
8. [ADR-0034](baby-phi/docs/specs/v0/implementation/m5_2/decisions/0034-agent-durable-lifecycle.md) — durable lifecycle invariants (handlers must reject archived agents).
9. [ADR-0028](baby-phi/docs/specs/v0/implementation/m1/decisions/0028-event-bus-fail-safe.md) — bus fail-safe semantics.
10. [`modules/crates/store/migrations/0009_identity_node.surql`](baby-phi/modules/crates/store/migrations/0009_identity_node.surql) + [`0010_consent_full_shape.surql`](baby-phi/modules/crates/store/migrations/0010_consent_full_shape.surql) — recent migration patterns.

### Carry-forward invariants (verified at chunk-open)

- `cargo test --workspace -- --test-threads=1` ≈ 1198 (post-CH-09 baseline).
- 4 CI guards green.
- ADR-0034..0045 Accepted; next-free = 0046.
- `git diff --stat HEAD -- modules/` empty.

---

## §10 — Close criteria (5-aspect)

- **Code aspect** — workspace builds; clippy under `RUSTFLAGS="-Dwarnings"` clean; `cargo test --workspace -- --test-threads=1` green at ~1222.
- **Docs aspect** — ADR-0046 Accepted; concept-doc verified-headers bumped on permissions/07 + ontology.md.
- **phi-core leverage** — import-count delta = 0; positive/forbidden greps all match expected.
- **Concept alignment** — every §2 row at target status (`honored` for both Template C + Template D rows).
- **K8s readiness** — neutral; ledger unchanged.

**Implementation confidence** = `claims-honored / claims-in-scope` = target **10/10**:
1. `Edge::Manages` + `Edge::HasAgentSupervisor` variants exist with carrier `org` / `project` field.
2. `Repository::create_manages_edge` + `create_has_agent_supervisor_edge` ship on both backends.
3. Migration 0011 adds `manages` + `has_agent_supervisor` tables with UNIQUE indexes.
4. `POST /api/v0/orgs/:org/agents/:agent/manager` returns 201 / 200 / 4xx correctly.
5. `POST /api/v0/projects/:project/agents/:agent/supervisor` returns 201 / 200 / 4xx correctly.
6. Handlers emit DomainEvent AFTER receipt, mirroring the HasLead post-commit pattern.
7. Idempotency: re-POST same triple returns 200, single edge row.
8. Archived-agent rejection wired (CH-01 / ADR-0034 invariant).
9. Acceptance suite `acceptance_system_flows_s05.rs` ships 4 scenarios all green.
10. Cross-listener ordering scenario passes (deterministic bus dispatch).

---

## §11 — Audit plan

**2 agents** (medium-large chunk).

### Audit A — Code correctness + phi-core leverage

> You are auditing CH-23 in baby-phi at `/root/projects/phi/baby-phi/`. Read-only. Plan: `docs/specs/plan/build/ch-23-template-cd-http-edges-and-e2e-<8hex>.md`.
>
> 1. `Edge::Manages { id, from: AgentId, to: AgentId, org: OrgId }` + `Edge::HasAgentSupervisor { id, from: AgentId, to: AgentId, project: ProjectId }` exist on the `Edge` enum at `modules/crates/domain/src/model/edges.rs` with matching `as_str()` and `EDGE_TYPE_NAMES` entries.
> 2. `Repository::create_manages_edge` + `create_has_agent_supervisor_edge` exist on the trait at `modules/crates/domain/src/repository.rs` with the documented receipt return shapes.
> 3. Both methods are implemented on `InMemoryRepository` (HashMap-backed) and `SurrealStore` (SurrealDB compound tx).
> 4. Migration 0011 (`0011_manages_supervisor_edges.surql`) defines two SCHEMAFULL tables with UNIQUE indexes; registered at version 11 / slug `manages_supervisor_edges` in `EMBEDDED_MIGRATIONS`. The migrations integration test asserts row count = 11 + version 11 entry.
> 5. HTTP handlers ship at `server/src/platform/orgs/agents/manager.rs` + `server/src/platform/projects/agents/supervisor.rs`. Routes registered at the workspace router. Both behind CEO-session auth.
> 6. Handler validation rules: target agent + counterpart exist + active + same-scope + idempotent re-POST returns 200 + same id. Each rule has a unit test.
> 7. DomainEvent emission happens AFTER repo receipt returns, mirroring `projects/create.rs:417` HasLead pattern. Both events fire on the in-process bus.
> 8. Acceptance suite `server/tests/acceptance_system_flows_s05.rs` has 4 scenarios — Template C end-to-end, Template D end-to-end, A+C+D simultaneous (all three listeners fire and grants persist), cross-listener ordering (deterministic dispatch order asserted).
> 9. `cargo test --workspace -- --test-threads=1` green at ~1222.
> 10. CI guards green; `check-phi-core-reuse.sh` exit 0; no `use phi_core::` imports in the new manager/supervisor handler modules.
> 11. CH-21 + CH-22 invariants intact: `acceptance_memory_extraction`, `acceptance_system_flows_s03` still green.
> 12. CH-01 (archived-agent rejection) invariant intact — handler-level test pins this.
> 13. Audit hash chain stays byte-stable on existing event variants.

PASS/FAIL each. ≤ 600 words.

### Audit B — Concept fidelity + docs fidelity

> You are auditing CH-23's concept-fidelity + docs-fidelity. Read-only.
>
> 1. ADR-0046 Accepted at `m5_2/decisions/0046-template-cd-http-edges.md` with sub-decisions D46.1–D46.10.
> 2. ADR-0046 Status field reads exactly `**Status: Accepted**` (one line, bold).
> 3. ADR-0046 documents the locked forks (Q3 route shape, Q4 edge field carrier, Q5 repo receipt shape) + the user's CH-23 scope decision (HTTP handlers in-chunk, no drift entry).
> 4. ADR-0046 cross-references concept doc 07 (Template C/D), ontology.md (MANAGES + HAS_AGENT_SUPERVISOR edges), ADR-0034 (durable lifecycle), ADR-0028 (event-bus fail-safe), ADR-0042 §D42.3 (migration runner conforming criteria).
> 5. Concept doc `permissions/07-templates-and-tools.md` verified-header bumped (CH-23 amendment line). Doc body UNCHANGED.
> 6. Concept doc `ontology.md` verified-header bumped — `MANAGES` + `HAS_AGENT_SUPERVISOR` edges now noted as `[EXISTS]` (or equivalent status flip from `[PLANNED]`).
> 7. Plan archive at `plan/build/ch-23-template-cd-http-edges-and-e2e-<8hex>.md` exists and follows the slug-first naming convention.
> 8. CH-09 invariants intact: ADR-0045 still Accepted; D-new-04 still remediated.
> 9. CH-04 + CH-05 invariants intact: ADR-0043 + ADR-0044 still Accepted; D-new-07, D-new-09, D-new-10, D-new-31 still remediated.
> 10. CH-21 + CH-22 invariants intact: ADR-0035 + ADR-0040 + ADR-0041 still Accepted; D6.1 still remediated (CH-21 + CH-22 ✓).
> 11. CH-16 + CH-01 invariants intact: ADR-0038 + ADR-0039 + ADR-0034 still Accepted; D-new-01 + D-new-23 + D6.5 + D-new-22 still remediated.
> 12. No new drift file added (per locked Q2).
> 13. `drifts/README.md` row counts and aggregate summary unchanged from post-CH-09 state.

PASS/FAIL each. ≤ 600 words.

---

## §12 — Verification recipe

```bash
cd /root/projects/phi/baby-phi

# 1. CI guards
bash scripts/check-doc-links.sh
bash scripts/check-ops-doc-headers.sh
bash scripts/check-phi-core-reuse.sh
bash scripts/check-spec-drift.sh

# 2. Build + clippy + test
cargo fmt --all -- --check
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy -j 4 --workspace --all-targets
/root/rust-env/cargo/bin/cargo test -j 4 --workspace -- --test-threads=1
# Expect: ~1222 passed / 0 failed

# 3. Positive greps
grep -n "Edge::Manages\b" modules/crates/domain/src/model/edges.rs                              # ≥ 1
grep -n "Edge::HasAgentSupervisor\b" modules/crates/domain/src/model/edges.rs                   # ≥ 1
grep -n "create_manages_edge" modules/crates/domain/src/repository.rs                           # ≥ 1
grep -n "create_has_agent_supervisor_edge" modules/crates/domain/src/repository.rs              # ≥ 1
ls modules/crates/store/migrations/0011_manages_supervisor_edges.surql                          # exists
ls modules/crates/server/tests/acceptance_system_flows_s05.rs                                   # exists
grep -c '^\*\*Status: Accepted\*\*' docs/specs/v0/implementation/m5_2/decisions/0046-template-cd-http-edges.md  # 1

# 4. Negative greps
grep -rn 'use phi_core::' modules/crates/server/src/platform/orgs/agents/manager.rs             # 0
grep -rn 'use phi_core::' modules/crates/server/src/platform/projects/agents/supervisor.rs      # 0

# 5. Targeted suites
/root/rust-env/cargo/bin/cargo test -j 4 -p server --test acceptance_system_flows_s05 -- --test-threads=1
# Expect: 4 tests pass.

# 6. Carry-forward sanity
/root/rust-env/cargo/bin/cargo test -j 4 -p server --test acceptance_system_flows_s03 -- --test-threads=1   # CH-22 still green
/root/rust-env/cargo/bin/cargo test -j 4 -p server --test acceptance_memory_extraction -- --test-threads=1  # CH-21 still green
/root/rust-env/cargo/bin/cargo test -j 4 -p store --test migrations_test -- --test-threads=1                # version 11 row asserted
```

---

## What this plan does NOT do

- DELETE / un-assign manager / supervisor endpoints.
- CLI commands or Web UI for the new flows.
- Tighter contractor cross-scope rules (D-new-20).
- Mutating the existing Template A/C/D listener bodies (already correct).
- Any change to the Permission Check engine's resolution path.

---

## Critical files

**New:**
- `modules/crates/store/migrations/0011_manages_supervisor_edges.surql` — table definitions.
- `modules/crates/server/src/platform/orgs/agents/manager.rs` — POST manager handler.
- `modules/crates/server/src/platform/projects/agents/supervisor.rs` — POST supervisor handler.
- `modules/crates/server/tests/acceptance_system_flows_s05.rs` — 4 end-to-end scenarios.
- `docs/specs/v0/implementation/m5_2/decisions/0046-template-cd-http-edges.md` — ADR.

**Modified:**
- `modules/crates/domain/src/model/edges.rs` — 2 new variants + as_str + EDGE_TYPE_NAMES.
- `modules/crates/domain/src/repository.rs` — 2 new trait methods + receipt structs.
- `modules/crates/domain/src/in_memory.rs` — implementations.
- `modules/crates/store/src/repo_impl.rs` — implementations + migration registration.
- `modules/crates/store/src/migrations.rs` — register version 11.
- `modules/crates/store/tests/migrations_test.rs` — extend to version 11.
- `modules/crates/server/src/lib.rs` (or router file) — register 2 new routes.
- Concept doc headers: `permissions/07-templates-and-tools.md`, `ontology.md`.

**Unchanged (verified by close-audit):**
- `templates/c.rs`, `templates/d.rs`, `events/listeners.rs` — listener bodies.
- `events/mod.rs` — DomainEvent variants already exist.
- `audit/events/m4/templates.rs`, `audit/events/m5/templates.rs` — existing audit events.

---

## Estimated effort

~3 engineer-days:
- 1.0d — P1 Edge variants + repo methods + migration 0011 + ~12 unit/integration tests.
- 0.7d — P2 HTTP handlers + route registration + ~8 handler tests.
- 0.5d — P3 acceptance_system_flows_s05.rs (4 scenarios).
- 0.3d — fmt / clippy / workspace test run / fix.
- 0.5d — P4 ADR Accepted + concept-doc header bumps + 2 audits + seal.
