# Plan: M4 — Agents + Projects (admin pages 08–11)

> **Legend:**
> - `[STATUS: ⏳ pending]` — not yet done
> - `[STATUS: ✓ done]` — already complete
> - `[STATUS: n/a]` — reference / meta section

## Context  `[STATUS: n/a]`

**Why this milestone now.** M3 closed at 99.1% composite confidence (633 Rust + 55 Web tests, ADR-0023 invariant verified, four phi-core types wrapped at Organization snapshot, per-org audit chains opened). M3 shipped admin pages 06 (org creation wizard) and 07 (org dashboard) but **deliberately stubbed six M4 carryovers** (C-M4-1 through C-M4-6 in the base build plan §M4) — the dashboard today renders zero counters for agents-by-role and projects-by-shape because the domain model doesn't yet have `AgentRole` or `Project` surfaces. M3's `HasLead` edge variant exists with zero production writes. The M3→M4 carryover items are blocking for a useful dashboard and for first-session launch at M5, so M4 is the right next milestone.

**What M4 ships.** Admin pages 08 (agent roster list), 09 (agent profile editor), 10 (project creation wizard — Shape A + Shape B co-owned), 11 (project detail). Each page as a vertical slice: Rust business logic + HTTP handler + CLI subcommand + Next.js web page + acceptance tests + ops doc. First milestone that materialises the `Project` node and writes the `HAS_LEAD` edge in production; first milestone to introduce a two-approver Auth-Request flow (Shape B); first milestone to extend the agent surface with `AgentRole` (Employee / Intern / Contract / System).

**What M4 does NOT ship** (explicit deferrals to M5 / M8; **P0 updates base build plan `docs/specs/plan/build/36d0c6c5-build-plan-v01.md` to pin ALL of these as named commitments in the appropriate milestone section, paralleling the existing C-M5-1 / C-M5-2 items added at M3 close**):

**Deferred to M5** — pinned as new base-plan commitments at P0:
- **C-M5-3** — `phi_core::session::Session` / `LoopRecord` / `Turn` persistence. phi's governance `Session` node (the FK-containment target of page 11's "Recent sessions" panel) materialises at M5 session launch. Includes SurrealDB `session` / `loop_record` / `turn` tables + serde wrapping of phi-core's three session types + `RUNS_IN` edge from Session to Project (so page 11's panel populates). This is M5's headline deliverable — M4's dashboard + page 11 render empty lists pending this work.
- **C-M5-4** — `AgentTool` per-agent binding. phi-core's `AgentTool` trait is invoked at session-start time; an agent's tool set is resolved from (a) the agent's `AgentProfile.blueprint` (currently phi-core's default tool list) or (b) a per-agent override (deferred; parallels the per-agent `ExecutionLimits` override pattern M4 introduced). M5 wires the resolver + surfaces the tool list on session-start via `GET /api/v0/sessions/:id/tools`. Scope excludes MCP tool registration (that's M2/P6 territory, shipped).

**Deferred to M5** — already pinned at M3 close:
- **C-M5-1** — Template graph node persistence at adoption time.
- **C-M5-2** — `UsesModel` edge wiring at session launch.

**Deferred to M8** — pinned as a new base-plan commitment at P0 (mirroring C-M5-1/2 shape):
- **C-M8-1** — `phi project create --from-layout <ref>` + 3–5 project-layout YAML fixtures (parallels M3's org-layout fixtures). Per user decision D-M4-6. Revisit when test volume demands fixture seeding.

**Permanently deferred** (not pinned — architectural decisions, not scope slips):
- External-config YAML parsing (`phi_core::config::parser`) — governance writes are CRUD, not external config.
- `phi_core::context::ContextConfig` per-agent override — stays org-default per ADR-0023 inherit-from-snapshot. M5 may revisit if demand surfaces.
- `phi_core::provider::retry::RetryConfig` per-agent override — same as above. M5 may revisit.

**What M4 DOES ship per user decisions at planning close:**
- **Per-agent `ExecutionLimits` override** on page 09 editor (opt-in per-agent, stored in a new `agent_execution_limits` table keyed by agent_id; inherit-from-snapshot remains the default). Requires new ADR (0027) layering on top of ADR-0023: ADR-0023 pins the default, 0027 adds opt-in override.
- **Template A firing event-listener subscription** wired at M4 (not deferred to M5). Requires new domain-event-bus infra + ADR-0028 (edge-change subscription architecture). Every `HAS_LEAD` edge write emits a domain event; the `TemplateAFireListener` subscribes and calls the pure-fn grant builder + persists the Grant + emits audit.
- **`AgentRole` applies to ALL agent kinds** (Human + LLM). Single enum `{ Executive, Admin, Member, Intern, Contract, System }` with a `is_valid_for(kind)` validation rule — first three are Human-only, last three are LLM-only. Requires a concept-doc update at P0 (`concepts/agent.md §Agent Taxonomy` currently treats only Intern/Contract/System as roles; M4 extends to Human subkinds).

**Base plan entry**: [§M4 in build plan](/root/projects/phi/phi/docs/specs/plan/build/36d0c6c5-build-plan-v01.md) — 4 lines of scope + six M4 carryovers. This plan is the fully-resolved version.

**Archive location for this plan**: `phi/docs/specs/plan/build/<8-hex>-m4-agents-and-projects.md`. First execution step (P0) archives this verbatim.

**What M3 taught us (applied preventively to M4):**

1. **phi-core leverage pre-audit BEFORE P1 opens.** M3's P3 "leverage = None" slip happened because the structural Q1/Q2/Q3 discipline was added mid-milestone. M4 runs the pre-audit at P0 (this plan's Part 1.5 contains the audit already); every phase's `### phi-core leverage` subsection in Part 4 is Q1/Q2/Q3-structured up-front with deliverable-level tags + positive-grep close assertions.
2. **Phase-boundary pause is mandatory.** M3's user-enforced discipline paid off — I'll pause at every phase boundary for user review. Re-iterating the commitment here.
3. **Confidence % at every phase close, no exceptions.** M3 only started volunteering percentages after user request at P5 close. M4 reports a 3-aspect confidence (code correctness + docs accuracy + phi-core leverage) at every phase close before opening the next phase.
4. **Base-plan carryovers are load-bearing.** M3 almost shipped without updating the base plan's M4/M5 carryover sections. M4 will update M5's base-plan carryover entries at P8 close if anything slips out.
5. **Two-approver flows demand a proptest at foundation.** Shape B (two-approver Auth Request) is the first two-approver governance flow in the codebase. M4's P3 adds a `shape_b_approval_matrix_invariants` proptest BEFORE the page 10 vertical opens, pinning the 4-outcome decision table (both-approve, both-deny, mixed A/D, mixed D/A).
6. **Schema-snapshot tests prevent wire-shape drift.** M3/P5's four-tier schema-snapshot test (unit + acceptance + cross-page + web) proved its worth. M4 applies the same pattern to the project-detail wire shape AND the agent-profile-editor payload, preventing accidental phi-core transit at later milestone edits.

---

## Part 1 — Pre-implementation gap audit  `[STATUS: ⏳ pending]`

Cross-check of admin pages 08–11 requirements + M3/P6 close state + M3's six carryover items. Findings:

| # | Finding | Source | Fix location |
|---|---|---|---|
| G1 | **`Project` struct is absent.** `modules/crates/domain/src/model/nodes.rs` has no `Project` struct. `Repository::list_projects_in_org` returns `Vec<ProjectId>` with a SurrealDB stub impl returning `vec![]`. Concept doc ([project.md](/root/projects/phi/phi/docs/specs/v0/concepts/project.md) §Properties) prescribes ~12 fields including OKRs + resource_boundaries + status state-machine. | C-M4-3 / C-M4-5 | P1 — new `Project` struct + `ProjectStatus` enum + embedded `Objective` / `KeyResult` value objects + migration 0004 |
| G2 | **`AgentRole` discriminator missing.** `Agent.kind` has only `Human` / `Llm`. Per user decision at planning close, M4 ships a **6-variant enum spanning both kinds**: `{ Executive, Admin, Member, Intern, Contract, System }` with `is_valid_for(kind)` validation (first three Human-only, last three LLM-only). Requires a concept-doc update at `concepts/agent.md §Agent Taxonomy`. | C-M4-2 | P1 — add `Agent.role: Option<AgentRole>` field + validation; P0 drafts concept-doc amendment |
| G3 | **`HasLead` edge has zero production writes.** Added as enum variant at M3/P1 for Template A's pure-fn constructor to name. `grep` for `Edge::HasLead` outside of tests returns zero hits. | C-M4-4 | P3 + P6 — compound-tx write at project creation + acceptance test that verifies the edge exists post-creation |
| G4 | **`ProjectShape` enum missing.** Page 10 requirements distinguish Shape A (single-org) and Shape B (co-owned). No enum exists. Dashboard `shape_a` / `shape_b` counters are placeholders. | C-M4-3 | P1 — new `ProjectShape { A, B }` enum with snake_case serde (`"shape_a"` / `"shape_b"`) |
| G5 | **`apply_project_creation` compound tx method missing.** M3 shipped `apply_org_creation` (one SurrealQL BEGIN…COMMIT for ~20 writes). Project creation needs analogous — Project + `HAS_LEAD` edge + `HAS_AGENT` edges + `HAS_SPONSOR` edge + `BELONGS_TO` edge(s) + OKR serde-embedded on row + optional template-adoption AR in one tx. Shape B variant creates a pending Auth Request instead of the project. | C-M4-4 + new | P3 — `apply_project_creation` domain trait method + two impls + rollback test |
| G6 | **Shape B two-approver Auth Request routing.** Pre-existing `AuthRequest.resource_slots[*].approvers: Vec<ApproverSlot>` already supports multiple slots. But no two-org-approver shape is wired; no state-machine test covers the 4-outcome decision matrix (both-approve, both-deny, mixed A/D, mixed D/A). | Page 10 §W3 | P3 — `shape_b_approval_matrix_invariants` proptest; P6 — page 10 acceptance covering all 4 outcomes |
| G7 | **Template A firing logic (s05) not wired.** Template A is a *pure-fn adoption* at M3 (each org self-approves at creation). The *firing* — "when a `HAS_LEAD` edge is created, issue a Grant to the newly-assigned lead" — is M4 work. Per user decision at planning close, M4 ships BOTH the pure-fn grant builder AND the event-listener subscription (previously deferred to M5). | Base plan §M4 | P2 — pure-fn grant builder `fire_grant_on_lead_assignment(...)`; P3 — domain event bus infra (`domain/src/events/`); P3 — `TemplateAFireListener` subscriber; P6 — acceptance verifies grant issued automatically on `HAS_LEAD` edge write via the subscription path. |
| G7b | **No domain event bus exists.** Required for Template A subscription + future reactive listeners. Design: in-process event bus `trait EventBus: Send + Sync` with `emit(event)` + `subscribe(handler)` methods; `Arc<dyn EventBus>` injected via `AppState`. Compound-tx handlers emit events AFTER successful commit (fail-safe: if event emission fails, commit is already durable; event-retry machinery is M7b). | New (from user decision) | P3 — `domain/src/events/{mod,bus,listeners}.rs`; ADR-0028 pins the architecture |
| G8 | **Inbox/outbox auto-creation at agent creation (s03).** M3 created these only for CEO at org creation. M4's page 09 create mode (R-ADMIN-09-W1) requires atomic inbox/outbox creation per agent. Needs the compound-tx pattern for agent creation. | Page 09 §W1 | P3 — `apply_agent_creation(payload)` compound tx or extend agent-creation handler to bundle inbox/outbox writes; P5 — acceptance tests verify composites exist post-creation |
| G9 | **No `AgentProfile` edit path.** Page 09 edit mode (R-ADMIN-09-W2) lets operators change `blueprint.system_prompt` / `temperature` / `thinking_level` / `parallelize` / per-project-role. Today `AgentProfile` has no PATCH handler. Immutability guards needed on `id`, `kind`, `base_organization`. | Page 09 §W2 | P5 — `PATCH /api/v0/agents/:id/profile` handler + diff-producing audit event `AgentProfileUpdated` |
| G10 | **No M4 audit-event builders.** Need `AgentCreated`, `AgentProfileUpdated`, `ProjectCreated`, `ProjectCreationPending` (Shape B), `TemplateAAdoptionFired` (lead-assignment grant firing). | Multiple | P2 — `domain/src/audit/events/m4/{agents,projects,templates}.rs` |
| G11 | **`parallelize` enforcement scope clarity.** Field exists on `AgentProfile`. Page 09 R-ADMIN-09-W3 requires `1 <= parallelize <= org_cap` on create + edit. Base plan §M4 says "`parallelize` field enforced at session-start time" — that's M5 (session launcher). M4 scope: creation/edit validation + storage. M5 scope: session-start gating. | Base plan | P1 — document the split explicitly in ADR; P5 — validation on create/edit paths |
| G11b | **Per-agent `ExecutionLimits` override infra missing.** Per user decision at planning close, page 09 allows operators to override `max_turns` / `max_total_tokens` / `max_duration` / `max_cost` per-agent. Today only `Organization.defaults_snapshot.execution_limits` exists (org-level). Needs a new `agent_execution_limits` table keyed by agent_id, with "inherit" meaning no row exists. Invariant: per-agent values must be ≤ the org-snapshot values (cannot exceed org ceiling). ADR-0023 stays authoritative for the default path; ADR-0027 adds opt-in override as a layered extension. | User decision | P1 — new `AgentExecutionLimitsOverride` composite + migration 0004 `agent_execution_limits` table; P2 — repo methods `get_agent_execution_limits_override`, `set_agent_execution_limits_override`, `clear_agent_execution_limits_override`; P5 — page 09 form fields editable with "Revert to org default" button. |
| G12 | **`list_agents_with_role` repo method missing.** Page 08 (R-ADMIN-08) lists agents filtered by role. Dashboard C-M4-2's `AgentsSummary.{intern,contract,employee,system}` buckets need this method. | C-M4-2 | P2 — new repo method `list_agents_in_org_by_role(org, role: Option<AgentRole>) -> Vec<Agent>` |
| G13 | **No project-level repo surface.** Dashboard's `ProjectsSummary.shape_a` / `shape_b` counters need a repo read. No `get_project`, `list_projects_by_shape`, or `count_projects_by_shape_in_org` exist. | Multiple | P2 — 4-5 new repo methods (get / list / count / list-by-shape / list-led-by-agent) |
| G14 | **OKR value-object structs missing.** Concept doc prescribes `Objective` + `KeyResult` as embedded value objects on Project. No Rust structs exist. | G1 follow-on | P1 — alongside `Project` struct |
| G15 | **Web wizard primitives — Shape B needs a two-approver review step variant.** M3's `ReviewDiff` primitive renders a single before/after pane; Shape B's wizard review step (step 6) shows "pending co-owner approval after submit" language distinct from Shape A's "will be created immediately". | M3 carryover | P1 — either extend `ReviewDiff` with an optional `pending_approvers` prop OR add a sibling `ShapeBPendingApprovalNotice` component |
| G16 | **CLI surfaces `agent` + `project` don't exist.** M3 shipped `phi org {create,list,show,dashboard}`. M4 needs `phi agent {list,show,create,update}` and `phi project {create,show,list,update-okrs}`. | Page 08-11 | P4 + P5 + P6 + P7 — per-vertical CLI subcommand; P8 completion-regression test |
| G17 | **Ontology edge count.** 67 edges at M3 close. Concept doc for Project lists edges `HAS_SPONSOR`, `HAS_AGENT`, `HAS_LEAD`, `HAS_TASK`, `HOLDS_GRANT`, `HAS_CONFIG`, `HAS_SUBPROJECT`, `BELONGS_TO`. Check which are present vs new. `HAS_LEAD` exists. `HAS_SPONSOR`, `HAS_SUBPROJECT`, `BELONGS_TO` need spot-check. Unlikely any new variant needed since many of these were pre-wired at M3/P1 alongside `HAS_LEAD`. | Concept doc | P0 audit — verify; P1 adds only what's genuinely missing |
| G18 | **`spawn_claimed_with_org_and_project` fixture missing.** M3 shipped `spawn_claimed_with_org`. M4's page 10/11 acceptance tests + dashboard's `ProjectsSummary` verification need a pre-populated fixture with 1 org + 1 project + 2 agents (lead + member). | Test ergonomics | P3 — new fixture extending `spawn_claimed_with_org` |
| G19 | **Reference-layout fixtures for projects?** M3 ships 3 org-creation layouts. Concept docs reference 5 project layouts (`projects/01-flat-single-project.md` through `projects/05-*`). Do we need `phi project create --from-layout <ref>` at M4? Most reference layouts contain 1 Shape A + 1 Shape B in their org shape. | Scope | P0 decision — recommend **no project-level `--from-layout` at M4**; project layouts live per-org and get different shapes per scenario, so fixture-reuse is limited. Revisit at M8. |

### Confidence target: **≥ 98 % at first review**, ≥ 99 % after P8 close re-audit.

Matches M3's bar. Risk areas: Shape B two-approver flow (first two-approver governance surface in the codebase) + dashboard rewrite callback chain (P1 types → P2 repo evolution → P8 retroactive dashboard update). Mitigated by P3's invariant proptest and P1's frozen-contract discipline.

---

## Part 1.5 — phi-core reuse map (M4)  `[STATUS: ⏳ pending]`

**Principle** (unchanged from M2/M3): phi is a consumer of phi-core. Every M4 surface overlapping a phi-core type uses phi-core's type directly or wraps it; re-implementations are reject-on-review per [CLAUDE.md §phi-core Leverage](/root/projects/phi/phi/CLAUDE.md).

**Pre-audit discipline** (Q1/Q2/Q3 per [leverage checklist](/root/projects/phi/phi/docs/specs/v0/implementation/m3/architecture/phi-core-leverage-checklist.md) §2): walked at P0 BEFORE any implementation. Per-phase close assertions pinned in Part 4.

Legend: ✅ direct reuse • 🔌 wrap (phi field holds phi-core type) • ♻ inherit from snapshot (no per-agent duplication per ADR-0023) • 🏗 build-from-scratch (phi-native).

### Page 08 — Agent Roster List (M4/P4)

| Surface | phi-core type / API | Mode |
|---|---|---|
| Agent row: `blueprint` field | `phi_core::agents::profile::AgentProfile` (inherited wrap since M3/P1) | ♻ Inherit via existing `AgentProfile.blueprint` wrap |
| Agent's execution budget display | `phi_core::context::execution::ExecutionLimits` (from org snapshot) | ♻ Inherit (read from `Organization.defaults_snapshot`; not duplicated per-agent per ADR-0023) |
| Role / `parallelize` / project_role map | (none — phi governance) | 🏗 Build-native |
| Session-count display | (none) | 🏗 Build-native (aggregate read; `phi_core::Session` not relevant until M5) |

### Page 09 — Agent Profile Editor (M4/P5) **← M4's phi-core-heaviest phase**

| Surface | phi-core type / API | Mode |
|---|---|---|
| Form field: name, system_prompt, thinking_level, temperature, personality | `phi_core::agents::profile::AgentProfile` fields | ✅ Direct — form binds to phi-core types; handler clones into `AgentProfile.blueprint` |
| Form field: max_turns, max_tokens, max_duration, max_cost | `phi_core::context::execution::ExecutionLimits` | ✅ Direct + 🔌 Wrap — page 09 shows editable fields. **Per user decision, per-agent override ships at M4** (stored in new `agent_execution_limits` table keyed by agent_id). Default path (no row): agent inherits from `Organization.defaults_snapshot.execution_limits`. Override path (row exists): serde into `phi_core::ExecutionLimits` directly, bounded ≤ org snapshot values. ADR-0027 pins the opt-in override; ADR-0023 still authoritative for default. Form includes "Revert to org default" button (deletes the override row). |
| ModelConfig dropdown | `phi_core::provider::model::ModelConfig` via M2's `ModelRuntime` | ✅ Direct (dropdown populates from `ModelRuntime.config.id` values in org's catalogue) |
| `parallelize` field | (none — phi governance) | 🏗 Build-native. NOT on phi-core's `ExecutionLimits` (which is per-loop); `parallelize` is multi-loop-concurrency. Orthogonal concerns. |
| `AgentRole` + `base_organization` + `project_role` map | (none) | 🏗 Build-native (M4 governance additions) |
| Audit event `AgentProfileUpdated.diff` | `serde_json::Value` (already used across M1-M3 audit events) | ♻ Inherit existing pattern |

### Page 10 — Project Creation Wizard (M4/P6)

| Surface | phi-core type / API | Mode |
|---|---|---|
| Project struct (node) | (none — phi governance composite) | 🏗 Build-native |
| `Objective` / `KeyResult` value objects | (none — phi-core has no OKR/planning concept) | 🏗 Build-native |
| `ProjectShape { A, B }` enum | (none) | 🏗 Build-native |
| `ResourceBoundaries` (subset of org catalogue) | (none) | 🏗 Build-native (phi catalogue concept from M2) |
| Lead picker → agent list | Inherited `AgentProfile.blueprint` shown per row | ♻ Inherit |
| Template E Auth Request for Shape B | (none — phi governance) | ♻ Inherit existing M2 Template E pattern |

### Page 11 — Project Detail (M4/P7)

| Surface | phi-core type / API | Mode |
|---|---|---|
| Project header + OKRs + resource boundaries | (none) | 🏗 Build-native |
| Agent roster panel | `AgentProfile.blueprint` per row | ♻ Inherit |
| "Recent sessions in this project" panel | **phi `Session` node** — NOT `phi_core::Session`. phi's governance Session persists at M5; M3 ships panel placeholder rendering empty list until M5. | 🏗 Build-native (placeholder at M4) |

### Template A firing (s05) — in `domain/src/templates/a.rs` + `domain/src/events/listeners.rs`

| Surface | phi-core type / API | Mode |
|---|---|---|
| Pure-fn grant builder `fire_grant_on_lead_assignment(lead: AgentId, project: ProjectId, now) -> Grant` | (none) | 🏗 Build-native (governance grant issuance; phi-core has no Grant concept) |
| Domain event `HasLeadEdgeCreated { project: ProjectId, lead: AgentId, at: DateTime }` | (none) | 🏗 Build-native — phi governance event; phi-core has no edge-change event concept |
| `trait EventBus` + `TemplateAFireListener` + bus wiring in `AppState` | (none) | 🏗 Build-native — reactive infra on phi's governance plane |

### Why no `phi_core::Session` at M4

Per D11 in M3 plan (pinned here at M4 too): `phi_core::Session` is an execution-trace record (minutes-to-hours lifetime, three levels deep under Project → Agent → Session). M4's page 11 "recent sessions" panel renders the phi governance `Session` node (persisted at M5), not phi-core's in-memory object. The two types have zero field overlap; collapsing them would break the FK-containment hierarchy.

### Q3 — candidates rejected with reasons

Walking the phi-core module inventory per checklist §2:

- `phi_core::agent_loop/` — N/A (runtime invocation, M5+).
- `phi_core::agents::Agent` trait / `BasicAgent` — N/A (runtime agent trait; phi stores governance metadata, doesn't instantiate agents until M5).
- `phi_core::config::{parser, schema::AgentConfig}` — N/A. phi uses direct CRUD (page 09 form). External YAML blueprint parsing is not in scope. Documented explicitly so a future reviewer doesn't re-debate.
- `phi_core::context::ContextConfig` per-agent override — N/A. Stays org-default per ADR-0023.
- `phi_core::mcp`, `phi_core::openapi`, `phi_core::tools` — N/A (runtime tools; no per-agent tool binding at M4).
- `phi_core::provider::retry::RetryConfig` per-agent override — N/A. Stays org-default per ADR-0023.
- `phi_core::session::*` — N/A per D11 + M5 deferral.
- `phi_core::types::AgentEvent` / `Usage` / `ToolResult` — N/A (orthogonal surfaces per CLAUDE.md).

**Enforcement at M4 close**: `scripts/check-phi-core-reuse.sh` 0 hits; positive grep `grep -En '^use phi_core::' modules/crates/server/src/platform/agents/ modules/crates/server/src/platform/projects/` expected to show `AgentProfile` + `ExecutionLimits` + `ModelConfig` + `ThinkingLevel` imports only in `agents/profile.rs` (page 09 edit handler) — the one "phi-core-heaviest" file in M4. Every other M4 surface must have zero phi-core imports by design.

### Three phi-core surfaces M4 MIGHT miss-leverage if not pinned

1. **`phi_core::types::tool::AgentTool`** — if page 09 gains a "tools this agent can use" field mid-M4, reviewers must require `use phi_core::types::tool::AgentTool` + a catalog query. At M4 scope this feature is deferred; if introduced, it's an ADR-tier decision.
2. **`phi_core::provider::retry::RetryConfig`** — if someone adds a "per-agent retry override" field to page 09, it must wrap phi-core's type, not redeclare. Safer: stay inherit-from-snapshot per ADR-0023 and don't introduce the field at M4.
3. **`phi_core::context::ContextConfig`** — same pattern. If per-agent override surfaces, wrap. Defer unless required.

### M4 planning decisions (resolved at plan close)

| # | Decision | User-chosen answer |
|---|---|---|
| D-M4-1 | `ThinkingLevel` UI variants | Show all **5** (`Off / Minimal / Low / Medium / High`); default `Medium`. **Corrected at M4/P5 close** — phi-core's `types::usage::ThinkingLevel` enum ships 5 variants (the `Minimal` level was missed during M4/P0 planning); the editor dropdown matches the actual phi-core enum. |
| D-M4-2 | Per-agent `ExecutionLimits` override | **SHIP at M4.** New `agent_execution_limits` table; page 09 editable with "Revert to org default" button. ADR-0027 pins opt-in override; ADR-0023 stays authoritative for default. Invariant: per-agent values ≤ org snapshot values. |
| D-M4-3 | `ModelConfig` change on active-session agent | **Forbid** — return 409 `ACTIVE_SESSIONS_BLOCK_MODEL_CHANGE`. Operator must terminate sessions first. |
| D-M4-4 | Template A firing scope | **Pure-fn + event-listener subscription**, BOTH at M4. Domain event bus + `TemplateAFireListener` wired so every `HAS_LEAD` edge write triggers the grant-builder automatically. ADR-0028 pins event-bus architecture. |
| D-M4-5 | `AgentRole` scope | **Applies to all agent kinds.** Single enum `{ Executive, Admin, Member, Intern, Contract, System }` with `is_valid_for(kind)` rule. Concept doc `agent.md §Agent Taxonomy` updated at P0. |
| D-M4-6 | `phi project create --from-layout <ref>` | **Defer to M8.** P0 updates the base plan's M8 section to pin this commitment explicitly. |

All six are user-confirmed; no planning re-debate during execution.

---

## Part 2 — Commitment ledger  `[STATUS: ⏳ pending]`

| # | Commitment | M4 deliverable | Phase | Verification |
|---|---|---|---|---|
| C1 | M3 post-flight delta log | 10-item audit confirming M3→M4 state; written to `m4/architecture/m3-postflight-delta.md` | P0 | doc-link check |
| C2 | `Project` struct + `ProjectStatus` + `ProjectShape` + `Objective` + `KeyResult` value objects | Rust structs with serde, cardinality tests | P1 | `domain/tests/m4_model_counts.rs` asserts new types serde-roundtrip |
| C3 | `AgentRole` 6-variant enum + `Agent.role: Option<AgentRole>` + `is_valid_for(kind)` validation + concept-doc amendment | Enum `{ Executive, Admin, Member, Intern, Contract, System }` + migration 0004 adds column + `concepts/agent.md §Agent Taxonomy` amended | P0 (concept doc) + P1 (code + migration) | migrations_0004 test + unit tests on `is_valid_for` |
| C3b | Per-agent `ExecutionLimits` override infra | New `AgentExecutionLimitsOverride` composite (wraps `phi_core::ExecutionLimits`); migration 0004 adds `agent_execution_limits` table; invariants enforced (≤ org snapshot values). Default path = no row = inherit-from-snapshot. | P1 | migrations_0004 + proptests on override-bounds invariant |
| C3c | Domain event bus + Template A subscription | `trait EventBus` + `InProcessEventBus` impl + `TemplateAFireListener` + `AppState` wiring; emits `HasLeadEdgeCreated` after every successful `apply_project_creation` commit. | P3 | `domain/tests/event_bus_props.rs` + acceptance test that Template A grant auto-fires via subscription (not direct call) |
| C4 | Migration 0004 forward-only | `0004_agents_projects.surql` adds `project` table, extends `agent` with `role` column, adds `agent_execution_limits` table, adds `BELONGS_TO` / `HAS_SPONSOR` / `HAS_SUBPROJECT` relation tables if missing | P1 | `store/tests/migrations_0004_test.rs` — apply / noop / fresh-DB |
| C5 | Web wizard Shape B notice primitive | `ShapeBPendingApprovalNotice.tsx` (or `ReviewDiff.pending_approvers` prop); reuses M3 `StepShell`/`StepNav`/`DraftContext` | P1 | `modules/web/__tests__/shape_b_primitives.test.tsx` |
| C6 | Repository expansion (10 methods) | `list_agents_in_org_by_role`, `get_project`, `list_projects_in_org` (return-type change to `Vec<Project>`), `list_projects_by_shape_in_org`, `count_projects_by_shape_in_org`, `list_projects_led_by_agent`, `get_agent_execution_limits_override`, `set_agent_execution_limits_override`, `clear_agent_execution_limits_override`, `resolve_effective_execution_limits(agent_id)` (returns override if set else org snapshot) | P2 | `store/tests/repo_m4_surface_test.rs` + in-memory parity |
| C7 | Template A firing pure-fn + listener | `domain/src/templates/a.rs::fire_grant_on_lead_assignment(lead, project, now) -> Grant` + `domain/src/events/listeners.rs::TemplateAFireListener` subscribing to `HasLeadEdgeCreated` | P2 (pure-fn) + P3 (listener) | `domain/tests/template_a_firing_props.rs` + acceptance test that grant auto-fires on `HAS_LEAD` write |
| C8 | M4 audit event builders | `audit/events/m4/{agents.rs, projects.rs, templates.rs}` — `AgentCreated`, `AgentProfileUpdated`, `ProjectCreated`, `ProjectCreationPending`, `TemplateAAdoptionFired` | P2 | Unit tests per builder |
| C9 | Compound tx `apply_project_creation` + Shape B branch | Single transaction for Shape A; AR-creation tx for Shape B; rollback on partial failure | P3 | `store/tests/apply_project_creation_tx_test.rs` — 4 scenarios (Shape A happy, Shape B pending, rollback, post-approval materialisation) |
| C10 | `apply_agent_creation` compound tx | Atomic Agent + Inbox + Outbox + default grants + optional AgentProfile | P3 | `store/tests/apply_agent_creation_tx_test.rs` — 3 scenarios |
| C11 | Shape B approval-matrix proptest | `domain/tests/shape_b_approval_matrix_props.rs` — 4×4 outcome decision table under 50 proptest cases | P3 | Proptest green |
| C12 | `spawn_claimed_with_org_and_project` test fixture | Extends M3's fixture — 1 org + 1 Shape A project + 2 agents (lead + member) | P3 | `server/tests/spawn_claimed_with_project_smoke.rs` |
| C13 | Page 08 vertical (Agent Roster List) | Business logic + handler + CLI + Web + ops doc | P4 | `server/tests/acceptance_agents_list.rs` — 6+ scenarios |
| C14 | Page 09 vertical (Agent Profile Editor) | Business logic + handler + CLI + Web + ops doc + diff-producing audit emit | P5 | `server/tests/acceptance_agents_profile.rs` — 8+ scenarios (create, edit, validation, immutability, System read-only, active-session restriction) |
| C15 | Page 10 vertical (Project Creation Wizard) | Business logic + handler + CLI + 6-step Web wizard + Shape A + Shape B + OKR editor + ops doc | P6 | `server/tests/acceptance_projects_create.rs` — 11+ scenarios including all 4 Shape B outcomes |
| C16 | Page 11 vertical (Project Detail) | Business logic + handler + CLI + Web + ops doc | P7 | `server/tests/acceptance_projects_detail.rs` — 5+ scenarios |
| C17 | M3 dashboard retroactive rewrite | Update `dashboard_summary` to fill `AgentsSummary.{employee,intern,contract,system}` + `ProjectsSummary.{shape_a,shape_b}` from new types + repo methods | P8 | Existing `acceptance_orgs_dashboard.rs` gets 2 new assertions; M3 carryovers C-M4-1/C-M4-2/C-M4-3/C-M4-6 closed |
| C18 | Cross-page acceptance `acceptance_m4.rs` | End-to-end: bootstrap → org → create Intern on page 08 → edit its profile on page 09 → create Shape A project on page 10 → view page 11 → dashboard shows counters | P8 | `acceptance_m4.rs` green in release profile |
| C19 | CLI completion auto-extension | `completion_help.rs` regression asserts `phi agent {list,show,create,update}` + `phi project {create,show,list,update-okrs}` in all 4 shell scripts | P8 | `cli/tests/completion_help.rs` |
| C20 | CI extensions | `.github/workflows/rust.yml` acceptance job extends `--test` list with 4 new binaries | P8 | CI green on PR |
| C21 | Ops docs + M4 troubleshooting + runbook aggregation | 4 per-page ops runbooks + `m4/user-guide/troubleshooting.md` + `docs/ops/runbook.md §M4` section | P4–P8 | `check-doc-links.sh` + `check-ops-doc-headers.sh` green |
| C22 | phi-core reuse map update for M4 | `phi-core-reuse-map.md` appended with per-page tables + positive close-audit record | P8 | doc-link check |
| C23 | **phi-core leverage enforced per-phase (four-tier model)** | Every phase in Part 4 carries `### phi-core leverage` (Q1/Q2/Q3) + deliverable-level tags + positive close-audit greps | P0–P8 | `check-phi-core-reuse.sh` 0 hits every phase close; positive greps documented per phase |
| C24 | **Per-phase 3-aspect confidence check with reported %** | Every phase close reports code % + docs % + phi-core % + composite % before next phase opens | P0–P8 | Explicit numbers pinned in close audit reports |
| C25 | Independent 3-agent re-audit at P8 | Mirrors M2/P8 + M3/P6 precedent — 3 parallel Explore agents (Rust / docs / page-vertical integrity); target ≥99% composite | P8 | Agent reports captured; any MED/HIGH findings remediated in same session |

Target: **25 commitments closed** at P8.

---

## Part 3 — Decisions made up-front  `[STATUS: ⏳ pending]`

| # | Decision | Rationale |
|---|---|---|
| D1 | **`Project` is a top-level node with embedded OKR value objects.** Not a separate `Objective` table. | Matches concept doc §OKR. Keeps OKR lifecycle bound to project. SurrealDB `FLEXIBLE TYPE object` column stores the vector. |
| D2 | **`ProjectShape` enum serde form is `shape_a` / `shape_b`.** Matches the dashboard's pre-existing `AgentsSummary.shape_a` / `.shape_b` field names from M3/P5. | Wire-contract stability. Alternative `single_org` / `co_owned` would force dashboard rename. |
| D3 | **`AgentRole` lives on `Agent` (not `AgentProfile`). 6 variants spanning both kinds:** `Executive`, `Admin`, `Member` (Human-only), `Intern`, `Contract`, `System` (LLM-only). Enforced by `is_valid_for(kind) -> bool`. Role transitions (Intern → Contract, Member → Admin) are separate flows (out of M4; M5+ or separate ADR). M4 treats role as immutable post-creation. **Concept-doc amendment at P0** for `concepts/agent.md §Agent Taxonomy`. | Role is about the human/governance identity, not the blueprint. Matches `admin/09` (edit mode cannot change `kind`; same rule for `role`). User-confirmed at plan close. |
| D4 | **Shape B two-approver Auth Request uses the existing `AuthRequest.resource_slots[*].approvers: Vec<ApproverSlot>` infrastructure.** M4 adds no new AR state-machine variants — just a 2-slot shape populated at submit. | Reuses M1 Permission Check engine. No migration churn. Proven pattern. |
| D5 | **Page 09 DOES expose per-agent `ExecutionLimits` override at M4** (user decision). Opt-in override: `agent_execution_limits` row exists → use it (subject to ≤ org ceiling); no row → inherit from snapshot per ADR-0023. Form has "Revert to org default" button that DELETEs the override row. ADR-0027 layers on top of ADR-0023. | User-confirmed at plan close. Adds ~3 days scope. Retains ADR-0023 as authoritative for the default path; override is explicit opt-in with bounded values. |
| D6 | **`parallelize` enforcement split across M4 and M5.** M4 validates `1 <= parallelize <= org_cap` on create + edit. M5 gates session launch against live-session count ≤ parallelize. | Matches natural scope boundaries. M4 is CRUD; M5 is runtime gating. |
| D7 | **Template A firing at M4 = pure-fn builder + event-listener subscription** (user decision). Pure-fn `fire_grant_on_lead_assignment(lead, project, now) -> Grant` stays proptest-friendly. New in-process event bus (`trait EventBus`) emits `HasLeadEdgeCreated` after every successful `apply_project_creation` commit. `TemplateAFireListener` subscribes, calls the pure-fn, persists the Grant, emits audit. ADR-0028 pins the event-bus architecture. | User-confirmed at plan close. Adds ~2 days scope. In-process bus chosen over SurrealDB LIVE queries: works with in-memory impl, no infrastructure dependency, fail-safe (commit durable before emit). |
| D8 | **Compound tx for Shape A project creation; 2-stage for Shape B.** Shape A = one BEGIN…COMMIT (Project + edges + OKR + optional template-adoption AR). Shape B = stage 1 is Template E AR creation with 2 approver slots; stage 2 (on both-approve) triggers the Shape-A compound tx. | Matches M3's `apply_org_creation` shape. Atomicity preserved. |
| D9 | **OKR editing is *in-place* on page 11 (project detail)** — not a separate page 10 step. Page 10 collects initial OKRs; page 11 edits them post-creation. | Avoids a 7th wizard step. OKR editing is a longer-lived workflow than creation. |
| D10 | **`phi project` CLI does NOT ship `--from-layout` at M4.** Deferred to M8. **P0 updates the base build plan's M8 section** (`docs/specs/plan/build/36d0c6c5-build-plan-v01.md`) to pin this commitment explicitly so it doesn't slip. | User-confirmed at plan close. Reference project layouts are per-scenario; limited fixture reuse vs org layouts. |
| D11 | **Dashboard carryover closure is P8, not mid-milestone.** P1 defines types; P2 wires repo; P8 retrofits dashboard after all M4 machinery exists. | Avoids schema churn. M3 dashboard keeps zero-counter behaviour until P8. |
| D12 | **`HasSponsor`, `HasSubproject`, `BelongsTo` edge variants.** Audit at P0: which exist? Concept doc lists all three. M3 shipped at 67 edges. P0 reports exact current-state; P1 adds only what's genuinely missing. | Lets P1 focus on the real gap. |
| D-M4-1 | **`ThinkingLevel` UI shows all 5 variants** (`Off / Minimal / Low / Medium / High`), default `Medium`. **Corrected at M4/P5 close** — phi-core ships 5 variants (the `Minimal` level was missed during M4/P0 planning); dropdown matches the phi-core enum so future phi-core variant additions require only a dropdown refresh, not a plan amendment. | User-confirmed (with post-P5 correction). |
| D-M4-2 | **Per-agent `ExecutionLimits` override SHIPS at M4**, opt-in (override row) + inherit default. | User-confirmed. See D5. |
| D-M4-3 | **`ModelConfig` change on active-session agent returns 409**. Operator must terminate sessions first. | R-ADMIN-09-W4. |
| D-M4-4 | **Template A firing = pure-fn builder + event-listener subscription at M4**. | User-confirmed. See D7. |
| D-M4-5 | **`AgentRole` 6-variant enum applies to all agent kinds** with `is_valid_for(kind)` validation. | User-confirmed. See D3. |
| D-M4-6 | **`phi project create --from-layout` deferred to M8**; base plan M8 section updated at P0. | User-confirmed. See D10. |

---

## Part 4 — Implementation phases  `[STATUS: ⏳ pending]`

Nine phases (P0 → P8). Each phase has five subsections: **Goals** · **Deliverables** · **phi-core leverage** · **Tests added** · **Confidence check**. Every phase closes with `cargo fmt/clippy/test + npm test/typecheck/lint/build + check-doc-links + check-ops-doc-headers + check-phi-core-reuse` all green, commitment-ledger row(s) ticked, AND **reported confidence % before the next phase opens**.

### P0 — M3 post-flight delta + concept-doc amendment + base-plan update + docs-tree seed (~4–6 hours)

#### Goals
Archive this plan; verify M3→M4 boundary state; draft concept-doc amendment for expanded `AgentRole`; update base build plan's M8 section for deferred `--from-layout`; verify ontology edges against concept doc for Project; seed M4 docs tree.

#### Deliverables
1. Archive plan to `phi/docs/specs/plan/build/<8hex>-m4-agents-and-projects.md` (token via `openssl rand -hex 4`).
2. `m4/architecture/m3-postflight-delta.md` — 10-item audit (edge count still 67, `HasLead` still has zero writes, `Project` struct still absent, `AgentRole` still missing, `parallelize` field present on `AgentProfile`, dashboard still returning zero counters for role/shape buckets, M3 carryovers C-M4-1 through C-M4-6 still open in base plan, `agent_execution_limits` table missing, event-bus infra absent, etc.).
3. **Concept doc amendment** `docs/specs/v0/concepts/agent.md §Agent Taxonomy` — add `Executive / Admin / Member` as Human-side `AgentRole` variants alongside existing `Intern / Contract / System` LLM-side variants. Pin the `is_valid_for(kind)` rule. Status stays CONCEPTUAL until M4 ships it; `Last verified` refreshed.
4. **Base build plan amendment** — edit `docs/specs/plan/build/36d0c6c5-build-plan-v01.md` in two sections:
   - **§M5 `#### Carryovers from M4 — must-pick-up at M5 detailed planning`** (NEW subsection, parallel to the M3-carryover subsections already present in §M5). Content:
     - **C-M5-3** — `phi_core::session::Session` / `LoopRecord` / `Turn` persistence. M4 page 11's "Recent sessions" panel + dashboard's session-related tiles render empty until M5 ships session-launch + persistence. Baby-phi's governance `Session` node wraps phi-core's `Session` + `LoopRecord` + `Turn` types; SurrealDB tables + `RUNS_IN` edge from Session to Project. Why M5: session launch is M5's headline work; Session materialises at launch, persists at launch-complete via `SessionRecorder`. Why pinned now: M4 ships the FK-containment target (page 11) in anticipation; not pinning this commitment risks the page 11 panel regressing to "deleted" status at M5 review.
     - **C-M5-4** — `AgentTool` per-agent binding. phi-core's `AgentTool` trait resolved at session-start time; agent's tool set comes from `AgentProfile.blueprint` default OR (deferred per-agent override). M5 wires the resolver + `GET /api/v0/sessions/:id/tools`. Why M5: tools are runtime (bound at session-start), not profile (M4 didn't need them). Why pinned now: M4's page 09 deliberately excludes a "tools this agent can use" field; this commitment documents why.
   - **§M8 `#### Carryovers from M4 — must-pick-up at M8 detailed planning`** (NEW subsection):
     - **C-M8-1** — `phi project create --from-layout <ref>` + 3–5 project-layout YAML fixtures, parallels M3's org-layout fixtures. Per user decision D-M4-6 at M4 planning close. Implementation: new `cli/fixtures/project_layouts/*.yaml` + CLI load path + acceptance tests. Revisit when test volume demands fixture seeding (M8 is acceptance+polish milestone per base plan §M8).
5. **Ontology audit for Project edges** — `grep Edge::{HasSponsor,HasSubproject,BelongsTo,HasTask,HasConfig,HoldsGrant}` to report which exist vs missing. P1 adds only what's genuinely absent.
6. M4 docs-tree seed at `docs/specs/v0/implementation/m4/{README,architecture,user-guide,operations,decisions}/` with stub files carrying `<!-- Last verified -->` headers.

#### phi-core leverage
None — audit phase. Confirm `check-phi-core-reuse.sh` still green.

#### Tests added
0.

#### Confidence check
**Target: N/A** (audit phase). Close criterion: every post-flight item has a `still-valid | stale | missing` tag with file+line reference. If >1 item is `stale`, open a P0.5 remediation before P1.

---

### P1 — Foundation: types, migration, web primitives, CLI scaffolding (~4 days, expanded)

#### Goals
Every M4 type exists, migration applies, web primitives ready, CLI scaffold compiled before P2 opens.

#### Deliverables
1. **`Project` struct** in `domain/src/model/nodes.rs`: `id`, `name`, `description`, `goal: Option<String>`, `status: ProjectStatus`, `shape: ProjectShape`, `token_budget: Option<u64>`, `tokens_spent: u64`, `objectives: Vec<Objective>`, `key_results: Vec<KeyResult>`, `resource_boundaries: Option<ResourceBoundaries>`, `created_at: DateTime<Utc>`. Strongly-typed `ProjectId`.
2. **`ProjectStatus`, `ProjectShape`** enums in `domain/src/model/nodes.rs`. Serde `snake_case`.
3. **OKR value objects** `Objective` + `KeyResult` in `domain/src/model/composites_m4.rs` with their status enums + measurement-type enum per concept doc §OKR.
4. **`ResourceBoundaries`** struct — subset references into org's resources catalogue (catalogue entries referenced by id+kind).
5. **`AgentRole` 6-variant enum** in `domain/src/model/nodes.rs`: `Executive`, `Admin`, `Member`, `Intern`, `Contract`, `System`. Add `Agent.role: Option<AgentRole>` field (None for legacy agents; P8 dashboard rewrite treats None-role agents as "Unclassified" and counts them under a new `unclassified` bucket on `AgentsSummary`). Add `impl AgentRole { pub fn is_valid_for(self, kind: AgentKind) -> bool }` validation. Serde `snake_case`.
5b. **`AgentExecutionLimitsOverride`** composite in `domain/src/model/composites_m4.rs` — wraps `phi_core::context::execution::ExecutionLimits` + `agent_id: AgentId` + `created_at`. Invariant (enforced at repo layer): every field ≤ corresponding field in `Organization.defaults_snapshot.execution_limits`.
5c. **Domain event types** in `domain/src/events/mod.rs`:
   - `trait EventBus: Send + Sync` with `async fn emit(&self, event: DomainEvent)` + `fn subscribe(&self, handler: Arc<dyn EventHandler>)`.
   - `enum DomainEvent { HasLeadEdgeCreated { project: ProjectId, lead: AgentId, at: DateTime<Utc> } }` — extensible for future edge-change events.
   - `InProcessEventBus` default impl (`tokio::sync::broadcast` channel or similar).
   - `AppState.event_bus: Arc<dyn EventBus>` — injected at boot.
6. **Migration 0004** `store/migrations/0004_agents_projects.surql`:
   - Add `role` column to `agent` table with ASSERT on 6 enum variants.
   - Create `project` table with `FLEXIBLE TYPE object` for `objectives` + `key_results` + `resource_boundaries`.
   - Create `agent_execution_limits` table with `FLEXIBLE TYPE object` for the phi-core `ExecutionLimits` wrap + `owning_agent` FK; UNIQUE INDEX on `owning_agent`.
   - Check for and add (if missing per P0 audit) `BELONGS_TO`, `HAS_SPONSOR`, `HAS_SUBPROJECT`, `HAS_TASK`, `HAS_CONFIG`, `HOLDS_GRANT` relation tables.
7. **`EDGE_KIND_NAMES` count update** in `domain/src/model/edges.rs` if new edges added; update compile-time test.
8. **Web wizard primitives extension**:
   - `ShapeBPendingApprovalNotice.tsx` in `modules/web/app/components/wizard/` — renders the "your co-owner must approve before creation" panel for Shape B's review step.
   - `OKREditor.tsx` — inline Objective + KeyResult editor reusable on page 10 step 3 and page 11 detail.
9. **CLI scaffolding**:
   - `cli/src/commands/agent.rs` skeleton with `list`, `show`, `create`, `update` subcommand stubs (body returns `EXIT_NOT_IMPLEMENTED` pending P4/P5 wiring).
   - `cli/src/commands/project.rs` skeleton with `list`, `show`, `create`, `update-okrs`.
   - Register in `cli/src/main.rs`.
10. **Docs**: `m4/architecture/shape-a-vs-shape-b.md` (explains the two project shapes + two-approver flow). `m4/architecture/event-bus.md` (in-process bus architecture; subscription contract; failure semantics). ADR-0024 — `Project` + `AgentRole` (6-variant enum) typing decisions (D1, D2, D3, D12, D-M4-5). ADR-0025 — Shape B two-approver flow (D4, D8). ADR-0027 — per-agent `ExecutionLimits` override (layered on top of ADR-0023; D5, D-M4-2). ADR-0028 — domain event bus + Template A subscription (D7, D-M4-4).

#### phi-core leverage (Q1/Q2/Q3 + deliverable tags)

- **Q1 direct imports**: **ONE new at P1** — `use phi_core::context::execution::ExecutionLimits` in `domain/src/model/composites_m4.rs::AgentExecutionLimitsOverride` (the override struct wraps phi-core's type directly). All other P1 deliverables import-free.
- **Q2 transitive**: `AgentExecutionLimitsOverride` transits `phi_core::ExecutionLimits` via serde through wire + storage. Identical pattern to M3's `OrganizationDefaultsSnapshot` wrap.
- **Q3 rejections** (explicit module walk): no phi-core types map to `Project`/`AgentRole`/`ProjectShape`/`OKR`/`ResourceBoundaries`/event-bus/`DomainEvent`. All documented as build-native in Part 1.5. The event bus is orthogonal to `phi_core::types::AgentEvent` (agent-loop telemetry) — a phi governance-event concept.

Deliverable tags:
- `Project` struct [phi-core: 🏗 none — governance composite.]
- `AgentRole` enum [phi-core: 🏗 none — no HR/role concept in phi-core.]
- `Objective` + `KeyResult` [phi-core: 🏗 none — phi-core has no planning/OKR concept.]
- `AgentExecutionLimitsOverride` [phi-core: 🔌 wrap `phi_core::ExecutionLimits`; direct reuse via serde transit. Identical discipline to M3/P1's `OrganizationDefaultsSnapshot`.]
- `trait EventBus` + `DomainEvent` [phi-core: 🏗 none — orthogonal to `phi_core::AgentEvent` per CLAUDE.md §Orthogonal surfaces.]
- OKREditor.tsx / ShapeBPendingApprovalNotice.tsx [phi-core: 🏗 none.]

Positive close-audit greps:
- `grep -En '^use phi_core::' modules/crates/domain/src/model/composites_m4.rs` → **1 line** (ExecutionLimits import).
- `grep -En '^use phi_core::' modules/crates/domain/src/events/mod.rs` → 0 lines (pure phi governance-event concept).
- `check-phi-core-reuse.sh` → 0 hits (no redeclaration).
- **Compile-time coercion test**: `fn is_phi_core_execution_limits(_: &phi_core::context::execution::ExecutionLimits) {}` in `composites_m4::tests`, applied to `AgentExecutionLimitsOverride.limits` field.

#### Tests added (~20)
- `domain/tests/m4_model_counts.rs` — `Project` / `AgentRole` / `ProjectShape` / `Objective` / `KeyResult` / `AgentExecutionLimitsOverride` serde round-trip (6 tests).
- `domain/tests/agent_role_validity_props.rs` — proptest on `is_valid_for(kind)` asserting Human roles accept Human kind only and LLM roles accept Llm kind only (50 cases).
- `domain/src/model/composites_m4.rs` unit tests — OKR measurement-type Value validation + phi-core coercion test (4 tests).
- `domain/tests/event_bus_smoke.rs` — `InProcessEventBus::emit` fires registered handlers; concurrent-subscriber test (3 tests).
- `store/tests/migrations_0004_test.rs` — apply / noop / broken (3 tests).
- `modules/web/__tests__/m4_primitives.test.tsx` — ShapeBPendingApprovalNotice renders; OKREditor inline edits + validation (3 tests).

#### Confidence check
**Target: ≥ 97%.** Close criteria:
- Code correctness: `cargo test --workspace` green, clippy `-Dwarnings` green, `npm run test/lint/typecheck/build` green.
- Docs accuracy: ADR-0024 + ADR-0025 status = Proposed (flip to Accepted at P6/P7 close). Docs-link + ops-doc-headers green.
- phi-core leverage: 0 `phi_core::` imports in any new P1 file; `check-phi-core-reuse.sh` green.
- Report composite confidence % (aspect breakdown) before P2 opens.

---

### P2 — Repository expansion + Template A firing pure-fn + M4 audit events (~2 days)

#### Goals
Domain + store surfaces needed by P3+ handlers are green and proptested.

#### Deliverables
1. **Repo method additions** in `domain/src/repository.rs` with in-memory + SurrealDB impls:
   - `list_agents_in_org_by_role(org: OrgId, role: Option<AgentRole>) -> Vec<Agent>`
   - `get_project(id: ProjectId) -> Option<Project>`
   - Change `list_projects_in_org` return-type from `Vec<ProjectId>` to `Vec<Project>` (coordinated across trait + in-memory + SurrealDB + M3 dashboard call-site — dashboard uses only `.len()` today so refactor is small).
   - `list_projects_by_shape_in_org(org: OrgId, shape: ProjectShape) -> Vec<Project>`
   - `count_projects_by_shape_in_org(org: OrgId) -> (u32 shape_a, u32 shape_b)`
   - `list_projects_led_by_agent(agent: AgentId) -> Vec<Project>`
2. **Template A firing pure-fn** `domain/src/templates/a.rs::fire_grant_on_lead_assignment(lead: AgentId, project: ProjectId, now) -> Grant`. Constructs a Grant with `[read, list, inspect]` on `project:<id>` resource, holder = `lead`, provenance linked to Template A adoption AR.
3. **M4 audit event builders**:
   - `audit/events/m4/agents.rs` — `agent_created`, `agent_profile_updated`.
   - `audit/events/m4/projects.rs` — `project_created`, `project_creation_pending`, `project_creation_denied`.
   - `audit/events/m4/templates.rs` — `template_a_grant_fired`.
4. **M4 audit events mod** `audit/events/m4/mod.rs` — `pub mod agents; pub mod projects; pub mod templates;`.

#### phi-core leverage

- **Q1 direct imports**: 0 (pure phi governance domain).
- **Q2 transitive**: repo reads return `Vec<Project>` / `Vec<Agent>` — these carry phi-core wraps via `Agent.blueprint` (inherited from M3); that transit is already audited. No new transit introduced by P2.
- **Q3 rejections**: same as Part 1.5 — Template builders are pure phi (phi-core has no Template/Grant/AuthRequest concept).

Deliverable tags:
- `list_agents_in_org_by_role` [phi-core: ♻ transitive (Agent.blueprint per-row).]
- `fire_grant_on_lead_assignment` [phi-core: 🏗 none — governance grant issuance.]
- M4 audit builders [phi-core: 🏗 none — governance audit log is orthogonal per CLAUDE.md.]

Positive close-audit greps:
- `grep -En '^use phi_core::' modules/crates/domain/src/templates/a.rs modules/crates/domain/src/audit/events/m4/` → 0 lines.

#### Tests added (~16)
- `domain/tests/in_memory_m4_test.rs` — 6 new list/count methods × 2 cases (populated + empty) = 12 tests.
- `store/tests/repo_m4_surface_test.rs` — 6 methods × 1 case = 6 tests.
- `domain/tests/template_a_firing_props.rs` — 1 proptest, 50 cases — grant shape invariants (holder, resource, actions, provenance).
- Unit tests in `audit/events/m4/*.rs` — shape + class + org_scope (~5 tests).

#### Confidence check
**Target: ≥ 97%.** Close criteria same structure as P1.

---

### P3 — Compound tx + fixture + Shape B proptest + event-bus wiring + Template A listener (~3 days, expanded)

#### Goals
The compound tx primitives P5–P7 need are ready; Shape B's 4-outcome decision table is proptest-verified; the event bus is live and `TemplateAFireListener` subscribes to `HasLeadEdgeCreated`.

#### Deliverables
1. **`apply_project_creation(payload: ProjectCreationPayload) -> Result<ProjectCreationReceipt, _>`** — domain trait method + in-memory + SurrealDB impls. Shape A: Project + `HAS_LEAD` edge + `HAS_AGENT` edges + `HAS_SPONSOR` edge + `BELONGS_TO` edge(s) + audit events all in one tx. **After successful commit**, emits `HasLeadEdgeCreated` on the bus (fail-safe: emit AFTER commit durable; emit errors logged but do NOT rewind the tx). Shape B: creates Auth Request with 2 approver slots instead of project; no event emitted until post-approval materialisation call.
2. **`apply_agent_creation(payload: AgentCreationPayload) -> Result<AgentCreationReceipt, _>`** — domain trait method + in-memory + SurrealDB impls. Single tx: Agent + Inbox + Outbox + default grants + `HAS_INBOX` + `HAS_OUTBOX` + `MEMBER_OF` edge to org + optional `AgentProfile` row + optional initial `agent_execution_limits` row (if caller supplied an override at creation time).
3. **`TemplateAFireListener`** `domain/src/events/listeners.rs` — subscribes to `HasLeadEdgeCreated`, calls `fire_grant_on_lead_assignment`, persists the Grant via `repo.upsert_grant(...)`, emits `TemplateAAdoptionFired` audit event. Error path: listener errors logged with `event_id` so operator can replay; M4 does not auto-retry (M7b adds event-retry infra).
4. **`AppState` event bus wiring** — `Arc<InProcessEventBus>` created at boot, registered with `TemplateAFireListener` subscriber. Server config option to disable bus in tests that don't need reactive behaviour.
5. **Shape B approval-matrix proptest** `domain/tests/shape_b_approval_matrix_props.rs` — 50 cases covering the 4 outcomes (both-approve → project materialises + Template A fires via bus, both-deny → AR Denied + no project + NO event emitted, mixed approve/deny → AR Partial + no project, mixed deny/approve → AR Partial + no project).
6. **`spawn_claimed_with_org_and_project` fixture** in `tests/acceptance_common/admin.rs`. Returns `ClaimedProject { admin, org_id, project_id, project_lead: AgentId, project_member: AgentId, template_a_grant_id: Option<GrantId> }` — the grant id is Some(..) confirming the subscription fired. Uses `apply_org_creation` + `apply_project_creation` directly with the bus wired; HTTP wizard flows exercised at P4–P7.

#### phi-core leverage

- **Q1 direct imports**: 0 in P3 (compound tx + event bus + listener are phi plumbing).
- **Q2 transitive**: `ProjectCreationPayload.project.leads[n]: Agent` carries `phi_core::AgentProfile` via the existing blueprint wrap. `AgentCreationPayload.initial_exec_limits: Option<AgentExecutionLimitsOverride>` carries `phi_core::ExecutionLimits` via the P1 wrap. Transit patterns audited at M3 ADR-0023 + M4 ADR-0027.
- **Q3 rejections**: explicit — event bus is orthogonal to `phi_core::AgentEvent` per CLAUDE.md; no phi-core coupling on subscription infra.

Deliverable tags:
- `apply_project_creation` [phi-core: 🏗 plumbing; ♻ transitive via blueprint.]
- `apply_agent_creation` [phi-core: 🏗 plumbing; ♻ transitive via optional override.]
- `TemplateAFireListener` [phi-core: 🏗 none — pure governance reactive logic.]
- `InProcessEventBus` + `AppState` wiring [phi-core: 🏗 none — phi governance-event infra.]
- Shape B proptest [phi-core: 🏗 none.]

Positive close-audit greps:
- `grep -En '^use phi_core::' modules/crates/domain/src/events/listeners.rs` → 0 lines.
- `grep -En '^use phi_core::' modules/crates/domain/tests/shape_b_approval_matrix_props.rs` → 0 lines.

#### Tests added (~15)
- `store/tests/apply_project_creation_tx_test.rs` — Shape A happy (+ bus emits), Shape A rollback on duplicate id (no bus emit), Shape B pending (no bus emit), Shape B post-both-approve materialisation (bus emits) (4 tests).
- `store/tests/apply_agent_creation_tx_test.rs` — happy, happy with initial override, rollback, idempotency (4 tests).
- `domain/tests/shape_b_approval_matrix_props.rs` — 1 proptest, 50 cases.
- `domain/tests/template_a_listener_test.rs` — listener subscribes + fires grant + emits audit + handles repo failure gracefully (4 tests).
- `server/tests/spawn_claimed_with_project_smoke.rs` — 2 smoke tests asserting `template_a_grant_id.is_some()` after fixture boot.

#### Confidence check
**Target: ≥ 97%.** Same structure.

---

### P4 — Page 08 vertical: Agent Roster List (~2–3 days)

#### Goals
Operators can list, filter by role, and search agents in an org via HTTP + CLI + Web. Read-only page.

#### Deliverables
1. Business logic `server/src/platform/agents/list.rs` — `list_agents` orchestrator with role + text-search filters.
2. Handler `GET /api/v0/orgs/:org_id/agents` with query params `?role=<role>&search=<q>`.
3. Router wiring.
4. CLI `phi agent list --org-id <uuid> [--role <role>]`.
5. Web page `modules/web/app/(admin)/organizations/[id]/agents/page.tsx` — table with filter chips + search box + row-click → `/organizations/[id]/agents/[agent_id]` (page 09 in P5).
6. Ops doc `m4/operations/agent-roster-operations.md`.

#### phi-core leverage

- **Q1 direct imports**: 0 (read-only; uses inherited blueprint via Agent.blueprint).
- **Q2 transitive**: response carries `Agent.blueprint: phi_core::AgentProfile` per-row (inherited from M3/P1 wrap). Document explicitly so the schema-snapshot test for the list response doesn't accidentally strip it later.
- **Q3 rejections**: explicit.

Deliverable tags:
- `list_agents` orchestrator [phi-core: ♻ transitive via blueprint.]
- Web page [phi-core: opaque `Record<string, unknown>` for blueprint at web tier; matches M3 pattern.]

Positive close-audit greps:
- `grep -En '^use phi_core::' modules/crates/server/src/platform/agents/` → 0 lines (no new direct imports).

#### Tests added (~10)
- `server/tests/acceptance_agents_list.rs` — 6 scenarios: list all, filter by role, text search, empty-result, 403 on non-member, pagination (if added).
- CLI help snapshot (2 tests).
- Web component test (2 tests).

#### Confidence check
**Target: ≥ 98%.** Same structure.

---

### P5 — Page 09 vertical: Agent Profile Editor (~4 days, **M4's phi-core-heaviest phase**, expanded for ExecutionLimits override)

#### Goals
Create + edit an Agent with phi-core-typed form fields. System agents read-only. `parallelize` ceiling enforced. **Per-agent `ExecutionLimits` override** supported (opt-in; "Revert to org default" button deletes the override row).

#### Deliverables
1. Business logic `server/src/platform/agents/create.rs` — `create_agent` orchestrator using `apply_agent_creation` compound tx. Validates `parallelize <= org_cap`. Validates `role.is_valid_for(kind)`. If operator supplied initial `ExecutionLimits`, validates each field ≤ org-snapshot value; persists via `agent_execution_limits` row inside the compound tx. Emits `AgentCreated` audit.
2. Business logic `server/src/platform/agents/update.rs` — `update_agent_profile` orchestrator. Computes diff, validates immutables (id, kind, role, base_organization), rejects `ModelConfig.id` change if active sessions exist (per D-M4-3, returns 409). Handles 3 ExecutionLimits paths: (a) inherit (no override row; operator did not set limits), (b) set override (validates ≤ org ceiling; upserts row), (c) revert (deletes override row). Emits `AgentProfileUpdated` audit with structured diff INCLUDING limits source (inherit vs override) + actual values.
3. **Business logic** `server/src/platform/agents/execution_limits.rs` — `resolve_effective_limits(agent_id) -> phi_core::ExecutionLimits` (returns override if set else org snapshot); `apply_override(agent_id, new_limits)` with bounds check; `clear_override(agent_id)`.
4. Handlers `POST /api/v0/orgs/:org_id/agents` (create) and `PATCH /api/v0/agents/:id/profile` (edit) and `DELETE /api/v0/agents/:id/execution-limits-override` (revert).
5. Error codes `AGENT_ID_IN_USE`, `AGENT_IMMUTABLE_FIELD_CHANGED`, `AGENT_ROLE_INVALID_FOR_KIND`, `PARALLELIZE_CEILING_EXCEEDED`, `EXECUTION_LIMITS_EXCEED_ORG_CEILING`, `ACTIVE_SESSIONS_BLOCK_MODEL_CHANGE`, `SYSTEM_AGENT_READ_ONLY`.
6. CLI `phi agent create --org-id --name --kind --role --model-id --system-prompt --parallelize [--override-max-turns N --override-max-tokens N ...]` and `phi agent update --id --patch-json` and `phi agent revert-limits --id`.
7. Web form `modules/web/app/(admin)/organizations/[id]/agents/[agent_id]/page.tsx` (edit) + `new/page.tsx` (create). Form sections: Identity / AgentProfile / ModelConfig / **ExecutionLimits (editable with two modes: "Inherit from org" radio vs "Override" radio; when override selected, show 4 editable fields with org-ceiling hints; when revert clicked, DELETE the override row and show inherited values)** / phi governance fields (role picker honouring `is_valid_for(kind)`, parallelize) / Default Grants preview.
8. Ops doc `m4/operations/agent-profile-editor-operations.md` — covers the three ExecutionLimits paths + their audit-event shapes.

#### phi-core leverage

- **Q1 direct imports**: **THE phi-core-heavy phase**. Imports across P5 files:
  - `use phi_core::agents::profile::AgentProfile;` — form binds directly (create + update).
  - `use phi_core::context::execution::ExecutionLimits;` — **writeable** at M4 per D-M4-2; appears in create + update + `execution_limits.rs` resolver.
  - `use phi_core::provider::model::ModelConfig;` — id-field validation against org catalogue. **At M4/P5 close this dropped to deferred-M5**: phi-core's `AgentProfile` has no `model_config` field (only a stable `config_id` used for loop_id composition), and baby-phi's `AgentProfile` wrap hasn't added a per-agent model-binding extension yet. M5 adds the extension + activates the full edit flow; the 409 `ACTIVE_SESSIONS_BLOCK_MODEL_CHANGE` code path + `count_active_sessions_for_agent` stub ship today so M5 flips one predicate.
  - `use phi_core::types::ThinkingLevel;` — **5-variant** dropdown per D-M4-1 (corrected at P5 close — phi-core ships `Off / Minimal / Low / Medium / High`).
- **Q2 transitive**: patch body (`AgentProfilePatch`) carries these 4 types end-to-end via serde. Override path additionally transits `ExecutionLimits` into the `agent_execution_limits` row's `FLEXIBLE TYPE object` column.
- **Q3 rejections**: per-agent `RetryConfig` / `ContextConfig` override — still deferred (user did not approve these for M4; only `ExecutionLimits` override shipped). `AgentTool` assignment — deferred to M5. Both documented.

Deliverable tags:
- `update_agent_profile` orchestrator [phi-core: ✅ 4 direct imports; 🔌 wrap via AgentProfile.blueprint; 🔌 wrap via AgentExecutionLimitsOverride.limits.]
- `create_agent` orchestrator [phi-core: ✅ 4 direct imports (same set); 🔌 wrap on persist.]
- `resolve_effective_limits` resolver [phi-core: ✅ returns `phi_core::ExecutionLimits` by value — zero coupling loss; direct reuse of phi-core's type as the return type.]
- Web form [phi-core: opaque `Record<string, unknown>` for blueprint + model_config + execution_limits at web tier.]

Positive close-audit greps:
- `grep -En '^use phi_core::' modules/crates/server/src/platform/agents/` → expected imports in create.rs, update.rs, execution_limits.rs (4 types, possibly duplicated per file).
- Schema-snapshot test asserts the `PATCH /agents/:id/profile` response body carries `blueprint` + `execution_limits` fields (with phi-core shape) when override is set — positive transit assertion.
- Compile-time coercion tests: `fn is_phi_core_thinking_level(_: &phi_core::types::ThinkingLevel) {}`, `fn is_phi_core_execution_limits(_: &phi_core::context::execution::ExecutionLimits) {}`, `fn is_phi_core_agent_profile(_: &phi_core::agents::profile::AgentProfile) {}` applied in the update orchestrator's blueprint + override paths.

#### Tests added (~35)
- `server/tests/acceptance_agents_profile.rs` — 15 scenarios: create happy, create with Intern role (Llm kind), create with Executive role (Human kind), create rejected for role-kind mismatch, create with initial ExecutionLimits override, create rejects override > org ceiling, create rejected for kind=System (P5 is not page 13), edit happy, edit toggles override → inherit, edit toggles inherit → override, edit rejects immutable change, edit rejects model change with active sessions, edit rejects parallelize over cap, 403 on non-admin, 404, System-agent read-only in edit mode.
- Unit tests on validation + diff computation + `AgentRole::is_valid_for` (10 tests).
- `resolve_effective_limits` proptest — 50 cases asserting override-present-returns-override, override-absent-returns-snapshot, override-values-always-≤-snapshot (1 proptest).
- `DELETE /execution-limits-override` idempotency test (1 test).
- CLI help + patch-json + revert-limits parsing (4 tests).
- Web form component tests (5 tests — including override-radio toggle + revert button).

#### Confidence check
**Target: ≥ 98%.** Code correctness verified against the 4 phi-core imports (compile-time tests at 3 points); docs accuracy includes ADR-0027 (override on top of ADR-0023); phi-core leverage verified by positive greps. Per-agent override invariants (≤ org ceiling) pinned by proptest. **Report %** before P6.

---

### P6 — Page 10 vertical: Project Creation Wizard (~3–4 days, **largest phase**)

#### Goals
End-to-end project creation for Shape A (single-org, immediate materialisation) and Shape B (co-owned, two-approver flow). 6-step wizard with OKR editor.

#### Deliverables
1. Business logic `server/src/platform/projects/create.rs`:
   - Validates payload (project_id regex `[a-z][a-z0-9-]*`, name non-empty, lead in org, Shape B co-owner catalogue overlap, OKR measurement-type values).
   - Shape A: calls `apply_project_creation` compound tx; fires Template A grant for lead; emits `ProjectCreated` audit.
   - Shape B: creates Template E Auth Request with 2 slots (one per co-owner admin); emits `ProjectCreationPending` audit; deposits `AgentMessage` in each co-owner's inbox.
   - Shape B approval-completion handler (`POST /api/v0/projects/_pending/:ar_id/approve`): on second-approve, runs the Shape A compound tx + audit emit.
2. Handler `POST /api/v0/orgs/:org_id/projects`.
3. Router wiring.
4. CLI `phi project create --org-id --name --shape shape_a|shape_b [--co-owner-org-id <id>] --lead-agent-id <uuid> --member-ids <csv>`.
5. Web 6-step wizard at `modules/web/app/(admin)/organizations/[id]/projects/new/page.tsx`. Per-step components mirror M3 org wizard's 8-step shape. Step 3 uses `OKREditor.tsx` from P1. Step 6 review uses `ShapeBPendingApprovalNotice` from P1 when Shape B selected.
6. Ops doc `m4/operations/project-creation-operations.md`.

#### phi-core leverage

- **Q1 direct imports**: 0 in P6 (business logic reads inherited blueprints; writes project governance data).
- **Q2 transitive**: Lead-picker payload carries agent rows with `blueprint: phi_core::AgentProfile` per-row (inherited M3 wrap).
- **Q3 rejections**: explicit — no phi-core types at the project-creation surface.

Deliverable tags:
- `create_project` orchestrator [phi-core: ♻ transitive (lead agent's blueprint); 🏗 project + OKRs + shape are governance.]
- OKR editor [phi-core: 🏗 none.]
- Shape B approval flow [phi-core: 🏗 none — M1 AR state machine.]

Positive close-audit greps:
- `grep -En '^use phi_core::' modules/crates/server/src/platform/projects/` → 0 lines (P6 adds none).
- **Schema-snapshot test** on `CreateProjectResponse` + `ProjectDetail` wire shapes: no `defaults_snapshot` / `execution_limits` / `blueprint` keys at any depth (the project API is phi-core-strip-only; drill-down to agent blueprints happens via page 09).

#### Tests added (~30)
- `server/tests/acceptance_projects_create.rs` — 11 scenarios:
  - Shape A happy (flat-single-project layout).
  - Shape A validation (empty name, bad id, lead not in org, OKR invalid).
  - Shape A with OKRs.
  - Shape B happy (joint-research layout; both approve → materialises).
  - Shape B both-deny (no project created).
  - Shape B mixed A/D (Partial, no project).
  - Shape B mixed D/A (Partial, no project).
  - 409 duplicate project_id in org.
  - 403 caller lacks `[allocate]` on project-registry.
- CLI tests (5 tests).
- Web wizard component tests (14 tests — 6 per-step renders + 4 validation + 4 review-pane variants).

#### Confidence check
**Target: ≥ 98%.** `acceptance_projects_create::shape_b_both_approve_materialises_project` is the headline invariant test; four-outcome Shape B coverage pinned. **Report %** before P7.

---

### P7 — Page 11 vertical: Project Detail (~2 days)

#### Goals
Operators + project members can view a project: header, OKRs, resource boundaries, agent roster, recent sessions (placeholder until M5), audit trail.

#### Deliverables
1. Business logic `server/src/platform/projects/detail.rs` — `project_detail(repo, project_id, viewer) -> Result<DashboardOutcome<ProjectDetail>, _>`.
2. Handler `GET /api/v0/projects/:id`.
3. In-place OKR editor: `PATCH /api/v0/projects/:id/okrs` with body `[{kind: "objective"|"key_result", op: "create"|"update"|"delete", payload: ...}]`. Each mutation emits `ProjectOkrUpdated` audit.
4. CLI `phi project show <id>` + `phi project update-okrs --patch-json`.
5. Web page `modules/web/app/(admin)/organizations/[org_id]/projects/[id]/page.tsx` — header + OKR panel (inline editable) + roster panel + "Recent sessions" placeholder (empty state until M5).
6. Ops doc `m4/operations/project-detail-operations.md`.

#### phi-core leverage

- **Q1 direct imports**: 0.
- **Q2 transitive**: roster panel carries agent rows; each row's `blueprint` transits. Session-recency panel renders empty list (M5 populates).
- **Q3 rejections**: `phi_core::Session` deliberately out-of-scope at M4 — the "Recent sessions" panel reads phi's governance Session node (M5 surface); at M4 the query returns empty. Documented in the page's module docstring.

Deliverable tags:
- `project_detail` orchestrator [phi-core: ♻ transitive via roster's Agent.blueprint.]
- OKR patch endpoint [phi-core: 🏗 governance edits only.]
- Recent sessions panel [phi-core: 🏗 placeholder; M5 wires.]

Positive close-audit greps:
- Schema-snapshot test on `ProjectDetail` wire shape mirrors P6's strip assertion.

#### Tests added (~15)
- `server/tests/acceptance_projects_detail.rs` — 5 scenarios: happy read, 404, 403 on non-member, OKR update create/update/delete, recent-sessions placeholder empty.
- CLI tests (3 tests).
- Web component tests (6 tests).

#### Confidence check
**Target: ≥ 98%.** Same structure. **Report %** before P8.

---

### P8 — Seal: dashboard rewrite + cross-page acceptance + CI + runbook + independent re-audit (~2 days)

#### Goals
M4 closes: dashboard carryovers (C-M4-1 through C-M4-6) closed; cross-page compose verified; CI extensions + runbook + troubleshooting + independent 3-agent re-audit target ≥99%.

#### Deliverables
1. **Dashboard retroactive rewrite** in `server/src/platform/orgs/dashboard.rs`:
   - `AgentsSummary` expands from `{ total, human, llm }` to `{ total, employee, intern, contract, system, unclassified }`.
   - `ProjectsSummary` populates `{ active, shape_a, shape_b }` from real counts (replaces placeholder zeros).
   - `resolve_viewer_role` gains `ProjectLead` path — viewer is a `ProjectLead` if they have `HasLead` to any project in the org (queries `list_projects_led_by_agent`).
   - Filtered view for project leads: scope `PendingAuthRequests` to their projects; hide `AlertedEventsCount` tile.
2. **Cross-page acceptance** `server/tests/acceptance_m4.rs`: bootstrap → org → create Intern on page 08 → edit profile on page 09 → create Shape A project on page 10 → read page 11 → dashboard shows updated counters. Verify audit chain contains expected events.
3. **CI updates** `.github/workflows/rust.yml` — add `--test` flags for `acceptance_agents_list`, `acceptance_agents_profile`, `acceptance_projects_create`, `acceptance_projects_detail`, `acceptance_m4`.
4. **CLI completion regression** extend `completion_help.rs` — assert `phi agent` + `phi project` subcommand tree surfaces in all 4 shells.
5. **Ops runbook M4 section** in `docs/ops/runbook.md` — mirrors M3's structure: 4 per-page ops runbook index + M4 error-code reference + incident playbooks (Shape B approval-deadlock, orphaned Agent on create failure, dashboard role-count drift).
6. **M4 troubleshooting** `m4/user-guide/troubleshooting.md` with stable-code tables per page + CLI exit codes.
7. **phi-core reuse map update** `m4/architecture/phi-core-reuse-map.md` (or append to M3's) — M4 per-page leverage summary + P8 positive close-audit record.
8. **Independent 3-agent re-audit** (mirrors M3/P6): 3 parallel Explore agents cover:
   - (a) Rust implementation across P0–P7 (invariants, compound-tx rollback, Shape B matrix).
   - (b) Docs + commitment-ledger fidelity (C1–C25; ADRs 0024–0026 Accepted; troubleshooting codes).
   - (c) Page-vertical integrity (wire contracts Rust↔CLI↔Web for 4 pages; phi-core schema-snapshot tests at all 4 tiers including new project wire shapes; Shape B 4-outcome acceptance coverage).
9. **Base build plan M5 carryover update**: if any M4 scope slips (e.g. per-agent `ExecutionLimits` override decision), write it into the M5 `#### Carryovers from M4` section.

#### phi-core leverage

- **Q1 direct imports**: 0 in P8 (dashboard rewrite uses existing types).
- **Q2 transitive**: dashboard's `AgentsSummary` adds `unclassified` bucket (phi governance count); no phi-core transit change.
- **Q3 rejections**: explicit. P8 adds no new phi-core surface.

#### Tests added (~5)
- `acceptance_m4.rs` — 1 cross-page scenario.
- Dashboard rewrite adds 2 new assertions to existing `acceptance_orgs_dashboard.rs` (role counters correct, shape counters correct).
- P8 re-audit may surface LOW findings requiring 1-2 additional tests.

#### Confidence check
**Target: ≥ 99% via independent Explore-agent audit** (M3/P6 precedent). Close criteria:
- Code correctness: `acceptance_m4.rs` green; full workspace test count ≥ ~760 Rust + ~95 Web; all CI gates green.
- Docs accuracy: M4 README shows every phase ✓; ADRs 0024–0026 Accepted; runbook M4 section + troubleshooting complete.
- phi-core leverage: `check-phi-core-reuse.sh` 0 hits; re-audit agent spot-checks every M4 composite for phi-core type references.
- Report composite %.

---

**Total phase estimate (revised after user scope decisions): ~22–25 calendar days ≈ 3.5–4 weeks.** Exceeds base plan's 2–3 week envelope by ~1 week due to in-scope additions:
- Per-agent `ExecutionLimits` override infra (+~3 days across P1, P2, P5).
- Event bus + Template A subscription (+~2 days across P1, P3).
- 6-variant `AgentRole` concept-doc amendment (+~0.5 day at P0).

The user explicitly approved this expanded scope at plan close.

---

## Part 5 — Testing strategy  `[STATUS: ⏳ pending]`

M4 aggregate test additions (revised for expanded scope):

| Layer | New M4 count | Purpose |
|---|---|---|
| Domain unit (composites_m4 + template A firing + audit builders + event bus) | ~25 | Type shapes + pure helpers |
| Domain proptest (Shape B matrix + template A firing + agent-role validity + event bus + resolve_effective_limits override bounds) | ~5 proptests (50 cases each) | Behaviour invariants |
| Store unit (migration 0004 — includes agent_execution_limits table) | 4 | Migration shape |
| Store integration (repo M4 surface — 10 methods + 2 compound tx + override CRUD) | ~18 | Persistence + compound-tx |
| Server unit (profile diff + OKR validation + AgentRole::is_valid_for + resolve_effective_limits) | ~14 | Handler helpers |
| Server integration (4 per-page acceptance + cross-page + dashboard-retrofit + Template A subscription) | ~55 scenarios | HTTP contract + reactive flow |
| CLI integration (agent + project subcommands + --override-* + revert-limits + completion regression) | ~14 | Subcommand surface |
| Web unit (shape-B primitives + OKR editor + 4 page component suites + override-radio toggle) | ~35 | Pure React + translators |
| Web SSR smoke (4 page render paths + auth gate) | ~8 | SSR probe |
| **M4 added total** | **~178 tests** | — |

**Post-M4 workspace target**: 633 (M3 close) + ~140 Rust ≈ **~770 Rust**; 55 + ~38 Web ≈ **~93 Web**; **~865 combined**, up from M3's 688.

**Key invariants shipping in M4**:
- Shape B 4-outcome decision matrix (pinned by 50-case proptest + 4 E2E acceptance scenarios).
- `HasLead` edge written at project creation (verified post-creation query).
- Template A grant shape on lead assignment (proptest-pinned; E2E-verified).
- `parallelize` ≤ org_cap enforced on create + edit (no session-start enforcement yet — that's M5).
- Per-agent inbox/outbox atomic with Agent creation (compound-tx).
- Dashboard counters (`AgentsSummary.*`, `ProjectsSummary.shape_a/b`) reflect real counts post-P8 rewrite.
- Phi-core wire-shape strip on project endpoints (no `defaults_snapshot` / `blueprint` transit on project surfaces).

**Fixtures planned**:
- `server/tests/acceptance_common/admin.rs::spawn_claimed_with_org_and_project` — extended harness.
- NO new project reference-layout YAML fixtures at M4 (deferred to M8 per D10).

---

## Part 6 — Documentation  `[STATUS: ⏳ pending]`

Root: `phi/docs/specs/v0/implementation/m4/`. Mirrors M3 structure.

```
implementation/m4/
├── README.md                                 9-phase index + ADR table
├── architecture/
│   ├── overview.md                           M4 system map
│   ├── m3-postflight-delta.md                P0's 10-item delta log
│   ├── shape-a-vs-shape-b.md                 Two-approver flow, state matrix
│   ├── project-model.md                      Project + OKRs + ResourceBoundaries
│   ├── agent-roster.md                       Page 08 architecture
│   ├── agent-profile-editor.md               Page 09 architecture (phi-core heaviest)
│   ├── project-creation.md                   Page 10 architecture
│   ├── project-detail.md                     Page 11 architecture
│   ├── template-a-firing.md                  s05 pure-fn grant builder + M5 subscription plan
│   └── phi-core-reuse-map.md                 M4 per-page leverage summary (may link to M3's single reuse-map)
├── user-guide/
│   ├── agent-roster-walkthrough.md
│   ├── agent-profile-editor-walkthrough.md
│   ├── project-creation-walkthrough.md       Shape A + Shape B tour
│   ├── project-detail-usage.md
│   ├── cli-reference-m4.md
│   └── troubleshooting.md                    M4 stable codes
├── operations/
│   ├── agent-roster-operations.md
│   ├── agent-profile-editor-operations.md
│   ├── project-creation-operations.md        Shape B approval-deadlock playbook
│   └── project-detail-operations.md
└── decisions/
    ├── 0024-project-and-agent-role-typing.md           D1, D2, D3, D12
    ├── 0025-shape-b-two-approver-flow.md               D4, D8
    └── 0026-per-agent-execution-limits-deferred.md     D5, D-M4-2 (complements ADR-0023)
```

Conventions unchanged from M3. ADR numbering continues from M3 (0024+).

---

## Part 7 — CI / CD extensions  `[STATUS: ⏳ pending]`

1. **`rust.yml` `acceptance` job** — extend `--test` list with `acceptance_agents_list`, `acceptance_agents_profile`, `acceptance_projects_create`, `acceptance_projects_detail`, `acceptance_m4`. Keep `--test-threads 1`.
2. **`rust.yml` `phi-core-reuse` job** — unchanged (hard-gated since M2).
3. **`doc-links.yml`** — new M4 docs tree; P1 seeds correct `../../../../../../` depth for `modules/` refs.
4. **`ops-doc-headers` job** — unchanged; new M4 ops runbooks must carry the header.
5. **`spec-drift.yml`** — extend grep set with `R-ADMIN-0[89]-*`, `R-ADMIN-1[01]-*`.

---

## Part 8 — Verification matrix  `[STATUS: ⏳ pending]`

| # | Commitment | Test / check |
|---|---|---|
| C1 | M3 post-flight delta log | `m4/architecture/m3-postflight-delta.md` written; 0 stale items |
| C2 | `Project` / `ProjectShape` / `AgentRole` / OKR types | `domain/tests/m4_model_counts.rs` |
| C3 | `Agent.role` field + migration | `store/tests/migrations_0004_test.rs` |
| C4 | Migration 0004 forward-only | same |
| C5 | Web wizard Shape B primitive | `modules/web/__tests__/m4_primitives.test.tsx` |
| C6 | Repo method additions | `store/tests/repo_m4_surface_test.rs` + in-memory parity |
| C7 | Template A firing pure-fn | `domain/tests/template_a_firing_props.rs` |
| C8 | M4 audit event builders | unit tests in each builder file |
| C9 | `apply_project_creation` compound tx | `store/tests/apply_project_creation_tx_test.rs` |
| C10 | `apply_agent_creation` compound tx | `store/tests/apply_agent_creation_tx_test.rs` |
| C11 | Shape B approval matrix proptest | `domain/tests/shape_b_approval_matrix_props.rs` |
| C12 | `spawn_claimed_with_org_and_project` fixture | `server/tests/spawn_claimed_with_project_smoke.rs` |
| C13 | Page 08 vertical | `server/tests/acceptance_agents_list.rs` + CLI + web |
| C14 | Page 09 vertical | `server/tests/acceptance_agents_profile.rs` + CLI + web |
| C15 | Page 10 vertical | `server/tests/acceptance_projects_create.rs` + CLI + web |
| C16 | Page 11 vertical | `server/tests/acceptance_projects_detail.rs` + CLI + web |
| C17 | Dashboard retroactive rewrite | Extended `acceptance_orgs_dashboard.rs` assertions |
| C18 | Cross-page acceptance | `server/tests/acceptance_m4.rs` |
| C19 | CLI completion regression | `cli/tests/completion_help.rs` extension |
| C20 | CI extensions | `.github/workflows/rust.yml` green on PR |
| C21 | Ops + troubleshooting + runbook | `check-doc-links.sh` + `check-ops-doc-headers.sh` green |
| C22 | phi-core reuse map update | doc-link check |
| C23 | phi-core leverage per-phase | each phase close: positive grep + `check-phi-core-reuse.sh` green |
| C24 | Per-phase confidence % | each phase close reports 3-aspect + composite % |
| C25 | Independent 3-agent re-audit | 3 agent reports captured at P8; LOW findings remediated; composite ≥99% |

**First-review confidence target: ≥ 98 %**. Post-P8 re-audit: ≥ 99 %.

---

## Part 9 — Execution order  `[STATUS: ⏳ pending]`

0. **Archive this plan** → `phi/docs/specs/plan/build/<8hex>-m4-agents-and-projects.md`. (~2 min)
1. **P0 post-flight delta + concept-doc amendment + base-plan M8 update + ontology audit + docs-tree seed** (~4–6 h)
2. **P1 Foundation** — types (incl. 6-variant AgentRole + AgentExecutionLimitsOverride wrap + event bus), migration 0004, web primitives, CLI scaffold, ADRs 0024/0025/0027/0028. (~4 d)
3. **P2 Repository + Template A pure-fn + 10 repo methods + M4 audit events** (~2 d)
4. **P3 Compound tx + fixture + Shape B proptest + event-bus wiring + TemplateAFireListener** (~3 d)
5. **P4 Page 08 vertical (Agent Roster List)** (~2–3 d)
6. **P5 Page 09 vertical (Agent Profile Editor + ExecutionLimits override)** — M4's phi-core-heaviest phase. (~4 d)
7. **P6 Page 10 vertical (Project Creation Wizard)** — largest phase; Shape A + Shape B + auto-firing Template A via subscription. (~3–4 d)
8. **P7 Page 11 vertical (Project Detail)** (~2 d)
9. **P8 Seal** — dashboard rewrite + cross-page acceptance + CI + runbook + troubleshooting + independent 3-agent re-audit → ≥99%. (~2 d)
10. **Re-audit → remediation → 100 %** (mirrors M3 post-P6 pass).
11. **Tag milestone** — `git tag v0.1-m4` in phi submodule (user-managed per M1/M2/M3 precedent).

**Total estimate: ~22–25 calendar days ≈ 3.5–4 weeks** (exceeds base plan M4's 2–3 week envelope by ~1 week; user-approved expanded scope at plan close — see Part 1 "What M4 DOES ship per user decisions").

---

## Part 10 — Critical files  `[STATUS: n/a]`

**New** (~60 production files + ~50 test files + ~22 docs):
- `domain/src/model/composites_m4.rs` — `Objective`, `KeyResult`, `ResourceBoundaries`, `AgentExecutionLimitsOverride` (wraps `phi_core::ExecutionLimits`).
- `domain/src/templates/a.rs` — extend with `fire_grant_on_lead_assignment`.
- `domain/src/events/{mod,bus,listeners}.rs` — `trait EventBus`, `InProcessEventBus`, `DomainEvent`, `TemplateAFireListener`.
- `domain/src/audit/events/m4/{mod,agents,projects,templates}.rs` — M4 event builders.
- `store/migrations/0004_agents_projects.surql` — schema migration (incl. `agent_execution_limits` table).
- `store/src/repo_impl_m4.rs` — M4 repo method impls (10 methods) + `apply_project_creation` + `apply_agent_creation`.
- `server/src/platform/agents/{mod,list,create,update,execution_limits}.rs` — business logic.
- `server/src/platform/projects/{mod,create,detail}.rs` — business logic.
- `server/src/handlers/{agents,projects}.rs` — HTTP handlers.
- `cli/src/commands/{agent,project}.rs` — CLI subcommands.
- `modules/web/app/components/wizard/{ShapeBPendingApprovalNotice,OKREditor}.tsx` — primitives.
- `modules/web/app/(admin)/organizations/[id]/agents/` — pages 08, 09 (incl. ExecutionLimits inherit/override toggle).
- `modules/web/app/(admin)/organizations/[id]/projects/` — pages 10, 11.
- `docs/specs/v0/implementation/m4/**/*.md` — full tree (~22 docs incl. `event-bus.md` architecture doc).
- `modules/crates/server/tests/acceptance_common/admin.rs::spawn_claimed_with_org_and_project`.

**Modified**:
- `docs/specs/v0/concepts/agent.md` — §Agent Taxonomy amendment (P0) for 6-variant `AgentRole`.
- `docs/specs/plan/build/36d0c6c5-build-plan-v01.md` — M8 section amendment (P0) for deferred `--from-layout`.
- `domain/src/model/nodes.rs` — `Project`, `ProjectShape`, `ProjectStatus`, `AgentRole` (6 variants), `Agent.role` field + `is_valid_for` impl.
- `domain/src/model/edges.rs` — `EDGE_KIND_NAMES` bump if new edges needed (TBD at P0 audit).
- `domain/src/repository.rs` — 10 new methods + `list_projects_in_org` return-type change.
- `domain/src/in_memory.rs` — matching in-memory impls.
- `server/src/state.rs` — `AppState.event_bus: Arc<dyn EventBus>` added.
- `server/src/router.rs` — new routes.
- `server/src/platform/orgs/dashboard.rs` — P8 retroactive rewrite (role + shape counters; ProjectLead viewer role).
- `server/src/handlers/orgs.rs` — unchanged at M4.
- `cli/src/main.rs` + `cli/src/commands/mod.rs` — register `agent` + `project` subcommands.
- `modules/web/app/(admin)/organizations/[id]/dashboard/DashboardClient.tsx` — panel updates for new summary shapes (6 role buckets, shape A/B counts).
- `.github/workflows/rust.yml` — acceptance-job `--test` list.
- `docs/ops/runbook.md` — M4 section.
- `docs/specs/v0/implementation/m3/architecture/phi-core-reuse-map.md` — append M4 rows (or new M4-specific doc).

---

## Part 11 — Open questions (non-blocking)  `[STATUS: n/a]`

Track in per-phase exec notes:
- **Q1**: Does `EDGE_KIND_NAMES` grow in P1? P0 audit reports exactly which of `BELONGS_TO` / `HAS_SPONSOR` / `HAS_SUBPROJECT` / `HAS_TASK` are already present. Expect 2-3 might be new.
- **Q2**: `AgentRole` for Human agents — is every Human a `Human`? (Employee / Intern / Contract appear to be LLM-agent roles from `concepts/agent.md §Agent Taxonomy`.) Re-read concept doc at P0 and reconcile with R-ADMIN-09 UI (which lists `Intern / Contract / Human` as kind radio buttons — that's the `kind` field, not `role`). **Likely resolution**: M4's `AgentRole` applies only to `AgentKind::Llm` agents. Humans have `role = None` (or a constant `Human` role). Confirm at P0.
- **Q3**: Should Shape B support N-approver (3+) or stay at exactly 2? Concept doc §Shape B says "co-owned by two orgs" — 2-only at M4. If later a 3-party shape emerges, it'd be Shape C (new, not M4 scope).
- **Q4**: OKR deadline as `DateTime<Utc>` — should it be `Date` (date-only, no time-of-day)? Concept doc says `Option<DateTime>`. Stay with datetime at M4; operator UX may prefer date-picker, which is a frontend concern.
- **Q5**: Page 09's "Default Grants preview" — how many grants? `admin/09` §W1 lists ~5. M5 may add more (consent-tag grants per s02). Keep the preview data-driven (render whatever the handler returns) so it evolves without frontend change.

---

## What stays unchanged  `[STATUS: n/a]`

- Concept docs (`docs/specs/v0/concepts/`) are the source of truth; M4 surface-count deltas land in the M4 plan if discovered at P0.
- M3 ships unchanged; M4 extends (new types + routes + audit events + docs tree + dashboard retroactive fill), doesn't refactor M3 surfaces.
- `phi-core` is a library dependency; M4 consumption adds **4 new direct imports at page 09 only** (AgentProfile, ExecutionLimits, ModelConfig, ThinkingLevel). Every other M4 surface stays phi-core-import-free.
- `handler_support` shim — M4 extends with no new helpers (reuses M3's `emit_audit_batch` and `ApiError` patterns).
- Four-tier phi-core leverage enforcement model — applied consistently per-phase.

---

## Open items — all resolved at plan close

All 6 planning-decision items (D-M4-1 through D-M4-6) were confirmed by user at plan close:
- ✅ **D-M4-1**: ThinkingLevel UI shows all **5** variants (`Off / Minimal / Low / Medium / High`), default Medium. _Post-P5 correction_: original D-M4-1 said "4 variants"; phi-core actually ships 5. Dropdown matches phi-core enum.
- ✅ **D-M4-2**: Per-agent ExecutionLimits override SHIPS at M4 (user chose override path).
- ✅ **D-M4-3**: ModelConfig change on active sessions returns 409.
- ✅ **D-M4-4**: Template A firing wires BOTH pure-fn AND event-listener subscription at M4.
- ✅ **D-M4-5**: AgentRole 6-variant enum applies to all kinds (Human + LLM) with is_valid_for validation.
- ✅ **D-M4-6**: `phi project create --from-layout` deferred to M8; P0 updates base plan M8 section.

No pre-P0 blockers remain. P0 execution can open immediately upon plan approval.
