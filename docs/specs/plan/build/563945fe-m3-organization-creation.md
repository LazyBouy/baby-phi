# Plan: M3 — Organization Creation + Dashboard (admin pages 06–07)

> **Legend:**
> - `[STATUS: ⏳ pending]` — not yet done
> - `[STATUS: ✓ done]` — already complete
> - `[STATUS: n/a]` — reference / meta section

## Context  `[STATUS: n/a]`

M2 (Platform Setup, pages 02–05) shipped at 99% composite confidence: 20 commitments closed across 9 phases; 511 Rust + 36 Web tests green; `handler_support` shim, Template E auto-approve, `SurrealAuditEmitter` with per-org hash chain, `Grant.fundamentals` + engine Case D, phi-core reuse lint enforced workspace-wide.

**M3 is the first milestone where audit events leave the root hash chain** and start per-org chains (`org_scope = Some(org_id)`). It's also the first milestone that provisions agents governed by phi-core types (system agents' `blueprint: phi_core::AgentProfile`) rather than only **platform-level** resource bindings (M2's model providers + MCP servers). M3 ships admin **page 06 (org creation wizard)** and **page 07 (org dashboard)** as vertical slices; it establishes the multi-step-wizard web pattern (reused by M4+) and the **per-phase phi-core-leverage + confidence-check** discipline the user has pinned as a hard commitment.

**The build-plan M3 entry is 6 lines** ([build plan §M3](../../projects/phi/phi/docs/specs/plan/build/36d0c6c5-build-plan-v01.md)); this plan is the fully-resolved version with per-phase commitments and a phi-core reuse map.

**What M2 taught us (applied preventively to M3):**

1. **Vertical-slice discipline scales.** M2's P4–P7 vertical-slice shape (Rust + CLI + Web + ops doc per page) kept each phase reviewable. M3 keeps it: P4 = page 06 vertical, P5 = page 07 vertical.
2. **Shared utilities are load-bearing.** M2's `handler_support` and `spawn_claimed` paid off after the second caller. M3 adds `spawn_claimed_with_org` + per-org audit proptest in P3 BEFORE the vertical slices begin.
3. **phi-core leverage ambiguity causes drift.** The "platform defaults uses `parse_config`?" question in M2/P7 wasted planning cycles. M3 pins phi-core leverage **per phase** with explicit import lists so execution never re-debates the boundary.
4. **Confidence checks need a fixed structure, not graduated rigor.** M2 closed at 99% because every phase ran a 3-aspect (code + docs + phi-core leverage) audit at close. M3 keeps this discipline — **every phase, no exceptions**.
5. **Ghost residuals cost time.** M2 budgeted 0.5 days for "AgentGovernanceProfile rename cleanup" that never needed doing (M2 chose to keep the wrapping type named `AgentProfile`). M3's pre-flight is a 2-hour audit pass, not a phase.

**Archive location for this plan:** `phi/docs/specs/plan/build/<8-hex>-m3-organization-creation.md` (first execution step copies this plan verbatim, matching the M1/M2 convention).

---

## Part 1 — Pre-implementation gap audit  `[STATUS: ⏳ pending]`

Cross-check of admin pages 06–07 requirements, concept docs, and current M2-close code. Findings:

| # | Finding | Source | Fix |
|---|---|---|---|
| G1 | **`HAS_LEAD` edge is missing.** `permissions/07` Template A's trigger ("when an agent becomes a project lead") references `HAS_LEAD` but the `Edge` enum has no `HasLead` variant. M3 doesn't wire Template A *behaviour* (M5 does) but **the edge type must exist** so the Template A pure-fn constructor in P2 can name it as the trigger condition, and so the `EDGE_KIND_NAMES` compile-time count test lands its new value now rather than during M5's crunch. | `model/edges.rs` (66 variants today); `permissions/07` | P1 adds `HasLead { id, from: ProjectId, to: AgentId }` variant. Bumps `EDGE_KIND_NAMES` count to 67. Documents in `concepts/ontology.md`. |
| G2 | **`OrganizationDefaults` has no explicit node type.** Concept docs treat the defaults snapshot as properties of the `Organization` node (06-W2 puts them inline in the wizard POST body). M2/P7's `PlatformDefaults` singleton is explicitly non-retroactive per ADR-0019. | `06-org-creation-wizard.md` W2; ADR-0019 | **Embed**, don't introduce a sibling composite. New fields on `Organization`: `consent_policy`, `audit_class_default`, `authority_templates_enabled`, `defaults_snapshot: OrganizationDefaultsSnapshot`. `OrganizationDefaultsSnapshot` lives in `domain/src/model/composites_m3.rs` and wraps phi-core's `ExecutionLimits` + `ContextConfig` + `RetryConfig` directly. |
| G3 | **Organization node is minimal today.** `Organization { id, display_name, created_at }` — no `vision`, `mission`, `consent_policy`, `audit_class_default`, `authority_templates_enabled`, `defaults_snapshot`, `system_agents` refs. | `model/nodes.rs` Organization struct | P1 extends the struct. Migration 0003 adds corresponding columns (`FLEXIBLE TYPE object` for phi-core-wrapping fields, matching M2/P7's table pattern). |
| G4 | **No `TokenBudgetPool` composite.** Pages 06 (R-W3 step 6: `token_budget_pool.initial_allocation`) and 07 (R6: "Page SHALL display token-budget utilisation as `used / total`") both reference it. No code today. | `06` W3; `07` R6; `model/composites.rs` | P1 adds `TokenBudgetPool` struct in `composites_m3.rs` (pure phi — phi-core has no budget-pool counterpart). Fields: `id: NodeId`, `owning_org: OrgId`, `initial_allocation: u64`, `used: u64`, `created_at`. |
| G5 | **Repository has no org-scoped list methods.** Dashboard (page 07) needs `list_agents_in_org`, `list_projects_in_org`, `list_active_auth_requests_for_org`, `list_recent_audit_events_for_org`. M2 has `list_active_auth_requests_for_resource` but no org-scope filter. | `repository.rs` | P2 adds 5 org-scoped list methods + in-memory + SurrealDB impls. |
| G6 | **Template A/B/C/D have no pure-fn builders.** M2 shipped only `domain::templates::e::build_auto_approved_request`. Page 06 W3 requires per-enabled-template adoption Auth Requests (A/B/C/D options). | `domain/src/templates/e.rs`; `06` W3 step 6 | P2 adds `domain::templates::{a,b,c,d}.rs` pure-fn builders mirroring `e.rs`'s `BuildArgs` shape. Each returns an `Approved` Auth Request. **Orchestrator that composes them with the CEO as approver lives in P4's server business logic, NOT domain** — it depends on the CEO-grant ordering (business concern, not pure fn). |
| G7 | **No batch audit-event emission.** Org creation in P4 emits `OrganizationCreated` + N × `AuthorityTemplateAdopted` events in one transaction. Today each emit is a separate trait-object call. | `audit/mod.rs`; `handler_support/audit.rs` | P3 adds `emit_audit_batch(&dyn AuditEmitter, Vec<AuditEvent>) -> Result<Vec<AuditEventId>, ApiError>` that emits in order (chain continuity preserved). |
| G8 | **Per-org hash chain is wired but untested.** `SurrealAuditEmitter::emit` already calls `self.repo.last_event_hash_for_org(event.org_scope)` with `Option<OrgId>`. M2 only exercised `None`. | `store/src/audit_emitter.rs:45-59`; `repository.rs last_event_hash_for_org` | P3 adds `two_orgs_interleaved_emit_props.rs` proptest asserting two interleaved orgs produce two independent chains (no cross-contamination). No code change — only verification. |
| G9 | **No web multi-step wizard primitives.** M2 web pages are single-form. The wizard (page 06) is 8 steps with autosaving draft, review-diff preview, forward/back navigation. Unprecedented in phi's web tier. | `modules/web/app/(admin)/*` | P1 adds `modules/web/app/components/wizard/{StepShell,StepNav,DraftContext,ReviewDiff}.tsx` primitives. M4+ reuses them (project-creation wizard). |
| G10 | **No session-storage draft pattern.** Wizard autosave needs ephemeral per-admin persistence. Server-side drafts (separate `organization_drafts` table + 2 endpoints + GC policy) are a week-scale detour; M7b's OAuth migration would re-plumb them anyway. | `06` W1 | **Client-side session storage with localStorage fallback.** M3 does NOT ship a `organization_drafts` table. Plan docs an M7b upgrade path if multi-admin orgs become common. |
| G11 | **No org-scope CLI surface.** CLI has no `org` subcommand. | `cli/src/commands/mod.rs` | P4 adds `phi org {create,list,show}` + P5 adds `phi org dashboard`. `org create` accepts `--from-layout <ref>` for seeding from the 10 reference-layout YAMLs. |
| G12 | **No `spawn_claimed_with_org` fixture.** Every M3 acceptance test needs an org baseline. Reimplementing the wizard sequence inline in each test duplicates ~50 lines. | `server/tests/acceptance_common/admin.rs` | P3 adds `spawn_claimed_with_org(with_metrics: bool) -> ClaimedOrg { admin: ClaimedAdmin, org_id, ceo_agent_id, system_agents: [AgentId; 2] }`. |
| G13 | **Audit-event builders for M3 don't exist.** Need `OrganizationCreated`, `AuthorityTemplateAdopted`, `OrgDashboardRead` (if we audit reads — TBD). | `audit/events/m2/*` pattern | P2 adds `audit/events/m3/orgs.rs` with `organization_created`, `authority_template_adopted` builders. Dashboard GET is not audited (read, routine) — matches M2 pattern. |
| G14 | **Dashboard polling strategy is a coin flip.** R-ADMIN-07-N1 says "30s poll OR WebSocket push." WebSocket push requires pub/sub infra (not in M3/M4/M5/M6 scope). | `07` N1 | **30s client-side `setInterval` via Server Actions for M3.** Document M7b upgrade path in `architecture/org-dashboard.md`. No server-side change. |
| G15 | **Template F is under-specified.** `nodes.rs` documents F as "Break-glass (elevated audit-class, mandatory post-incident review)" but `permissions/07` describes only A-E. Concept-doc gap predates M3. | `model/nodes.rs TemplateKind`; `permissions/07` | **Do NOT add a Template F concept doc in M3** (scope creep). **Do NOT drop F from TemplateKind** (migration churn). M3's architecture doc records F as "reserved for M6 break-glass work; M3 does not adopt F at org creation." |
| G16 | **`create_agent` is single-row; system-agent provisioning needs 2 agents + 2 profiles atomically.** Partial failure mid-provisioning would leave orphaned nodes (violates 06 W4). | `repository.rs create_agent`; `06` W4 | P3 wraps the compound P4 create in a single `Repository::apply_org_creation(payload) -> RepositoryResult<...>` method that runs every write inside one SurrealQL transaction (M2's `apply_bootstrap_claim` established the pattern). Returns ids + auth-request ids for audit emission. |
| G17 | **CEO Human Agent needs an inbox + outbox composite pair.** 06 W3 step 4 + N3 (CEO receives welcome message). M2 doesn't create inbox/outbox for any agent. | `06` W3, N3 | P4 adds inbox + outbox creation inside the compound `apply_org_creation`. No phi-core counterpart — these are `Composite::InboxObject` / `OutboxObject` instances tagged `org:<id>`. N3's actual channel delivery is M7b (we just write the inbox row; the delivery hook is a post-M3 concern). |
| G18 | **No reference-layout test fixtures.** The 10 reference layouts prescribe complete org shapes; page 06's acceptance scenarios reference layouts #01, #02, #04 explicitly. | `docs/specs/v0/organizations/01..10-*.md`; `06` §11 | P5 acceptance harness adds `fixtures/reference_layouts/` with 3 machine-readable YAMLs (minimal-startup, mid-product-team, regulated-enterprise) that the `phi org create --from-layout <ref>` CLI consumes. Full 10-layout fixture parity is deferred to M8's "10 layouts become fixture builders across the suite". |
| G19 | **phi-core leverage commitment is not phase-explicit.** M2 plan's §1.5 was milestone-level; execution phases re-debated boundaries (see P5/P7 retrospectives). | User's explicit requirement for M3 | **Every M3 phase in Part 4 carries a dedicated `### phi-core leverage` subsection** with concrete import paths + "phi-native" flags. Same structure for `### Confidence check` subsection per phase. |

### Confidence target: **≥ 98 % at first review**, ≥ 99 % after P6 close re-audit.

Higher than M2's 97% first-review target because M2 infrastructure (handler_support, Template E, SurrealAuditEmitter, per-org hash chain, Grant.fundamentals, phi-core reuse lint) absorbs most of M3's infra risk. Remaining risk: wizard multi-step pattern + 10-reference-layout fidelity. Mitigated by P1's wizard-primitives-first approach and P5's reference-layout fixture seeding.

---

## Part 1.5 — phi-core reuse map  `[STATUS: ⏳ pending]`

**Principle** (per M2's G18 + D16, unchanged for M3): phi is a consumer of phi-core. Every M3 surface that overlaps a phi-core type uses the phi-core type directly or wraps it; re-implementations are reject-on-review.

Legend: ✅ direct reuse • 🔌 wrap (phi field holds phi-core type) • 🚫 no phi-core counterpart

| Surface | phi-core type / API | M3 use site | Mode |
|---|---|---|---|
| **Page 06 — Org Creation Wizard** | | | |
| System agent blueprint | `phi_core::agents::profile::AgentProfile` | Every system agent's `blueprint` field (already wrapped on `AgentProfile` node per M2/P0) — M3 provisions 2 instances per org | ✅ (inherited from M2) |
| System agent execution budget | `phi_core::context::execution::ExecutionLimits` | Per-system-agent limits default to the org's `defaults_snapshot.execution_limits` (wraps phi-core's) | ✅ |
| System agent model binding | `phi_core::provider::model::ModelConfig` via M2's `ModelRuntime` | Both system agents bound via `secret_ref` + registered model provider | ✅ |
| Org defaults snapshot — execution | `phi_core::context::execution::ExecutionLimits` | `OrganizationDefaultsSnapshot.execution_limits` field | 🔌 |
| Org defaults snapshot — context | `phi_core::context::config::ContextConfig` | `OrganizationDefaultsSnapshot.context_config` field | 🔌 |
| Org defaults snapshot — retry | `phi_core::provider::retry::RetryConfig` | `OrganizationDefaultsSnapshot.retry_config` field | 🔌 |
| Org defaults snapshot — agent blueprint | `phi_core::agents::profile::AgentProfile` | `OrganizationDefaultsSnapshot.default_agent_profile` field | 🔌 |
| **Organization node fields** | | | |
| `consent_policy`, `audit_class_default`, `authority_templates_enabled`, `vision`, `mission`, `system_agents` | (none) | Pure phi — governance concerns with no phi-core counterpart | 🚫 |
| Inbox / Outbox for CEO | (none) | `Composite::InboxObject` / `OutboxObject` — M1 domain | 🚫 |
| Authority Template A/B/C/D constructors | (none) | `domain::templates::{a,b,c,d}.rs` — phi's governance plane, phi-core has nothing here | 🚫 |
| Token budget pool | (none) | `TokenBudgetPool` composite — phi's economic-resource model | 🚫 |
| **Page 07 — Org Dashboard** | | | |
| Aggregate reads (agent counts, project counts, AR counts, budget utilisation, recent events) | (none) | Pure repo traversal; no phi-core reuse opportunity | 🚫 |
| Dashboard polling cadence | (none) | 30 s `setInterval` via Server Actions (M7b upgrade to WebSocket) | 🚫 |
| **CLI / Seal (P4–P6)** | | | |
| Shell completion generator | `clap_complete` (non-phi-core; already wired in M2/P8's `completion` subcommand) | New `org` subcommand auto-wires into existing completion surface | ✅ (zero net work — clap_complete walks the subcommand tree at build time) |

### Why `Organization` is not a wrap of `phi_core::session::model::Session`

This is a load-bearing judgement call — the phi-core reuse mandate can push toward "wrap everything that looks session-ish," but applied here it would produce a type-level lie. Four reasons `Organization` stays pure phi rather than wrapping `phi_core::Session`:

1. **Different lifetimes.** `phi_core::Session` is a per-execution-trace record — it has `turns: Vec<Turn>`, `loop_records: Vec<LoopRecord>`, `status: LoopStatus`, `ended_at: Option<DateTime>`. Its lifetime is minutes-to-hours, bounded by one agent's one task. An `Organization` is a multi-year governance aggregate with members, policies, budget pool, audit scope; it has no "end state" in the execution-trace sense.
2. **Different level in the hierarchy.** Sessions are owned by Agents; Agents are members of Orgs. The containment is `Organization → Agent → Session` (three levels). Wrapping `Session` to represent an `Organization` would collapse two levels and make the Agent-owned relationship structurally impossible to express.
3. **No field overlap.** Look at `phi_core::Session`'s field set — **zero** of them make sense on an Organization (`turns`? `loop_records`? `status`?). Compare to M2's four wraps (`ExecutionLimits`, `AgentProfile`, `ContextConfig`, `RetryConfig`) which have **every** field meaningful on `PlatformDefaults` / `OrganizationDefaultsSnapshot`. The reuse mandate's intent is "reuse where fields genuinely overlap"; shoehorning a wrap where they don't creates a mandatory-empty-field type that future readers will misinterpret.
4. **phi-core's `Session` becomes load-bearing at M5, not M3.** When session launch lands, each `phi_core::Session` will be persisted by phi with a foreign key to its owning Agent (and transitively to its Org). That's **containment**, not wrap — Org has many Sessions (queryable); Org is not one Session. Plan §1.5's "deferred (✅)" tag for `Session` / `LoopRecord` / `Turn` / `AgentEvent` captures exactly this relationship.

   **This does NOT preclude Org-level drill-down into Session details.** Operators on the Org dashboard (page 07) will absolutely need to navigate from "recent audit events" or "agents in this org" into individual `phi_core::Session` replay / trace views. That drill-down is a **navigation pattern across the FK-containment graph** (`Organization → Agent → Session`), not a type relationship. The clickable audit-event rows in M3's dashboard today link to an audit-log detail page; once M5 ships session persistence, those same rows will deep-link into the session trace view by following `actor_agent_id → agent's recent sessions → chosen session`. No wrap is needed for this — reuse of `phi_core::Session` happens inside the session-launch handler at M5, and the Org-level UI just offers the navigation entry points. **P5's Web dashboard anticipates this**: the `RecentAuditEvents` panel renders a row as a link, and the link-target in M3 is the audit-log page; in M5+ the same row resolves to a session-trace URL when the event has a session provenance attached. This is a routing evolution, not a schema one.

**Confirmed: no phi-core session-scope / workspace / team concept maps to `Organization` today, and the right relationship (containment via Agent FK) is already reserved for M5. Org-level drill-down into `phi_core::Session` details happens through FK navigation at M5+, not through wrapping.** Documenting this explicitly so the reuse linter does not later surface a false positive AND so the next planner doesn't re-debate the boundary.

**Enforcement**: `scripts/check-phi-core-reuse.sh` (already hard-gated since M2/P3) continues to enforce no phi redeclarations of phi-core types. P6 close-audit spot-checks every M3 composite for phi-core type references. This judgement is also pinned as **D11** in Part 3.

---

## Part 2 — Commitment ledger  `[STATUS: ⏳ pending]`

| # | Commitment | M3 deliverable | Phase | Verification |
|---|---|---|---|---|
| C1 | M2 pre-flight delta log | 9-item audit pass surfacing drift between M2-close state and M3 assumptions | P0 | Written to `docs/specs/v0/implementation/m3/architecture/m2-preflight-delta.md` |
| C2 | Organization + defaults snapshot + TokenBudgetPool types | Extended `Organization` struct + `OrganizationDefaultsSnapshot` + `TokenBudgetPool` composites; `HasLead` edge; 67-edge count test | P1 | `domain/tests/m3_model_counts.rs` asserts 67 edges + every new struct serde round-trips |
| C3 | Migration 0003 applies forward-only | `0003_org_creation.surql` extends `organization` table + adds `token_budget_pool` table | P1 | `store/tests/migrations_0003_test.rs` — fresh DB applies; noop on already-applied DB |
| C4 | Web wizard primitives | `modules/web/app/components/wizard/{StepShell,StepNav,DraftContext,ReviewDiff}.tsx` + unit tests | P1 | `modules/web/__tests__/wizard_primitives.test.tsx` |
| C5 | Repository org-scoped list surface | 5 new methods (`list_agents_in_org`, `list_projects_in_org`, `list_active_auth_requests_for_org`, `list_recent_audit_events_for_org`, + a `list_adoption_auth_requests_for_org` for dashboard) + both impls | P2 | `domain/tests/in_memory_m3_test.rs` + `store/tests/repo_m3_surface_test.rs` |
| C6 | Template A/B/C/D pure-fn builders | `domain::templates::{a,b,c,d}.rs` + per-builder proptests | P2 | `domain/tests/template_{a,b,c,d}_props.rs` — aggregate shape + CEO-as-approver + serde round-trip per template |
| C7 | M3 audit event builders | `OrganizationCreated`, `AuthorityTemplateAdopted` in `audit/events/m3/orgs.rs` | P2 | Unit tests in builder file (class = Alerted, `org_scope = Some(org_id)`, diff shape stable) |
| C8 | Per-org hash-chain isolation proptest | `domain/tests/two_orgs_audit_chain_props.rs` — 50 cases interleaving emits across two orgs | P3 | Proptest green; no cross-org hash contamination |
| C9 | Compound `apply_org_creation` repo method + batch audit emit | Single-transaction write + `emit_audit_batch` helper | P3 | `store/tests/apply_org_creation_tx_test.rs` — partial-failure rollback; `handler_support_test.rs` adds batch-emit case |
| C10 | `spawn_claimed_with_org` test fixture | `tests/acceptance_common/admin.rs::spawn_claimed_with_org -> ClaimedOrg` | P3 | `server/tests/spawn_claimed_with_org_smoke.rs` |
| C11 | Page 06 vertical (org creation wizard) | Business logic + handlers + CLI (`org {create,list,show}`) + 8-step Web wizard + ops doc | P4 | `server/tests/acceptance_orgs_create.rs` — 9+ scenarios (5 validation errors + happy + 409 collision + rollback + 3 reference-layout-fidelity); CLI snapshot + web wizard component tests |
| C12 | Page 07 vertical (org dashboard) | Business logic + handler + CLI (`org dashboard`) + Web dashboard + ops doc | P5 | `server/tests/acceptance_orgs_dashboard.rs` — 6+ scenarios; CLI snapshot + web dashboard component tests |
| C13 | Cross-page acceptance + metrics extension | `acceptance_m3.rs` (bootstrap → org create → dashboard audit-chain verification) + `acceptance_metrics.rs` extension | P6 | `acceptance_m3.rs` green; metrics scrape records org-creation request |
| C14 | CLI completion auto-extension | `phi completion <shell>` now lists the `org` subcommand (clap_complete walks the tree) | P6 | `cli/tests/completion_help.rs` regression test asserts `org` appears in bash/zsh/fish/powershell scripts |
| C15 | CI extensions | `rust.yml` acceptance job extended; `check-ops-doc-headers.sh` still green for new M3 ops docs | P6 | `.github/workflows/rust.yml` green on PR |
| C16 | Ops docs + M3 troubleshooting + runbook aggregation | `m3/user-guide/troubleshooting.md` + `docs/ops/runbook.md` §M3 section + 2 per-page ops runbooks | P4, P5, P6 | `check-doc-links.sh` + `check-ops-doc-headers.sh` green |
| C17 | **phi-core reuse mandate enforced per-phase** | Every phase section in Part 4 carries a dedicated `### phi-core leverage` subsection; `scripts/check-phi-core-reuse.sh` stays green | P1–P6 | P6 close audit spot-checks every new M3 composite / node extension for phi-core type references; script zero-hit on forbidden duplications |
| C18 | **Per-phase 3-aspect confidence check** | Every phase section in Part 4 carries a dedicated `### Confidence check` subsection with target % + what the close verifies (code correctness + docs accuracy + phi-core leverage) | P0–P6 | Each phase explicitly reports its close-confidence figure before the next phase opens; M3-final re-audit at P6 aggregates |

Target: **18 commitments closed** at P6. Plus P6 independent re-audit mirroring M2/P8.

---

## Part 3 — Decisions made up-front  `[STATUS: ⏳ pending]`

| # | Decision | Rationale |
|---|---|---|
| D1 | **OrganizationDefaults embeds on the `Organization` node; no sibling composite.** `OrganizationDefaultsSnapshot` struct lives in `domain/src/model/composites_m3.rs` and is a field on `Organization`, not a separate node. | Mirrors ADR-0019's non-retroactive semantics: each org captures its own snapshot at creation; no independent lifecycle for the snapshot; matches `06-W2`'s inline wizard payload shape. A separate composite would add a node type with no membership/ownership semantics of its own. |
| D2 | **Template F stays in `TemplateKind` as reserved.** M3 does not adopt F at org creation. Architecture doc flags F as "reserved for M6 break-glass work." | Dropping F would require migration-0003 redefining `template.kind`'s ASSERT clause (churn for no value). Adding a concept doc for F is out of M3 scope. |
| D3 | **Wizard autosave = client-side session storage + localStorage fallback.** No `organization_drafts` table; migration 0003 contains no draft-storage work. | Server-side drafts need endpoints + GC policy + M7b OAuth re-plumb; session storage gives refresh tolerance (the real R-ADMIN-06-W1 requirement) with zero new backend surface. Plan docs an M7b upgrade if multi-admin orgs become common. |
| D4 | **Dashboard polling = 30 s client-side `setInterval` via Server Actions.** No pub/sub infrastructure in M3. | R-ADMIN-07-N1 explicitly allows "30 s OR WebSocket". WebSocket requires pub/sub not in scope until M7b. Architecture doc pins the upgrade path. |
| D5 | **Add `HasLead` edge variant in M3/P1** even though Template A behaviour is M5 work. | Edge inventory's `EDGE_KIND_NAMES` count is asserted at compile time (`edges.rs:596`). Adding edges incrementally per M3/M4/M5 regresses M2's "foundation in P1" discipline. P1 pre-wires the enum variant; M5 wires the trigger. |
| D6 | **Compound org-creation transaction = one Repository method.** `Repository::apply_org_creation(payload) -> RepositoryResult<OrgCreationReceipt>` runs every write in one SurrealQL transaction. Handlers compose audit events outside the tx. | `06-W4` (atomic rollback on partial failure). Mirrors M1's `apply_bootstrap_claim` pattern — known-working precedent. |
| D7 | **Pure-fn per-template builders in `domain`, orchestrator in `server`.** `domain::templates::{a,b,c,d}.rs` are pure fns; `build_template_suite_adoption` (batch orchestrator that picks CEO as approver) lives in `server/src/platform/orgs/create.rs`. | Pure fns stay proptest-friendly; orchestrator depends on the CEO's grant existing (business concern, not pure-fn concern). Clean layer separation. |
| D8 | **System agents' model binding = org's default provider (if set) else platform default.** Both system agents reference a `secret_ref` pointing at a vault slug; the actual model choice comes from an org-level "default_model_provider" field (added to Organization in P1). | Acceptance scenario 1 (minimal-startup) uses the platform's Claude Sonnet; acceptance scenario 2 (regulated-enterprise) may pick a different one. Defaulting the system-agent binding lets the wizard skip the question. |
| D9 | **CEO inbox message for N3 is written, not delivered.** M3 writes the inbox row; actual channel delivery (Slack/email webhook) is M7b when real alerting wires up. | Channel delivery needs real outbound HTTP + secrets-vault read path (stakeholder notification policy). Not in M3 scope per build plan. Row-write preserves audit continuity. |
| D10 | **Per-phase phi-core leverage + confidence check are structural, not optional.** Every phase section in Part 4 has explicit `### phi-core leverage` + `### Confidence check` subsections. If a phase has zero phi-core reuse (e.g. P5 dashboard), the section says so explicitly and names the phi-native reason. | User's hard commitment for this plan. Removes the "re-debate the boundary per phase" failure mode M2/P5 hit. |
| D11 | **`Organization` is NOT a wrap of `phi_core::session::model::Session`.** `Organization` is a governance aggregate; `phi_core::Session` is an execution-trace record. Different lifetimes (years vs. hours), different levels (Org → Agent → Session is three levels; a wrap collapses two), zero field overlap. phi-core's `Session` becomes load-bearing at M5 via a foreign-key **containment** relationship, not a wrap. | See Part 1.5's "Why `Organization` is not a wrap of `phi_core::session::model::Session`" subsection for the full four-reason argument. Pinned here so P4/P5 executors don't re-debate. |
| D12 | **System agents inherit their execution context from `Organization.defaults_snapshot`; no per-agent `ExecutionLimits` / `ContextConfig` / `RetryConfig` / `CompactionPolicy` nodes are materialized at M3.** The snapshot (already freezing `phi_core::ExecutionLimits`, `phi_core::ContextConfig`, `phi_core::RetryConfig` per ADR-0019) is the single source of truth. Per-agent overrides land at M5 session launch if and only if a genuine per-agent override is needed; until then, the invocation path reads the snapshot. | Keeps each phi-core type in **one place per org** — the exact goal of the phi-core reuse mandate (phi/CLAUDE.md §phi-core Leverage). Creating per-agent `ExecutionLimits` nodes at M3 would duplicate `phi_core::ExecutionLimits` across (1 org + 2 system agents) = three copies per org, creating a non-retroactive-semantics headache: if the org defaults are re-frozen later, do per-agent copies follow? No good answer. Deferring the node materialization to "only when actually needed" (M5) sidesteps the problem entirely. Documented in ADR-0023. |

---

## Part 4 — Implementation phases  `[STATUS: ⏳ pending]`

Seven phases (P0 → P6). Each phase has five subsections: **Goals** · **Deliverables** · **phi-core leverage** · **Tests added** · **Confidence check**. Every phase closes with `cargo fmt/clippy/test + npm test/typecheck/lint/build + check-doc-links + check-ops-doc-headers + check-phi-core-reuse` all green and the commitment-ledger row(s) ticked.

### P0 — M2 pre-flight delta log (~2–4 hours)

#### Goals
Validate that M2-close's inventory still holds before M3 opens. Write findings to `m3/architecture/m2-preflight-delta.md`. No code churn.

#### Deliverables
1. 9-item pre-flight audit pass confirming: `HAS_CEO` / `HAS_AGENT` / `MEMBER_OF` / `HAS_SUBORGANIZATION` edges exist; `Template::kind` supports A-F; migration 0002 asserts kinds include A-F; `SurrealAuditEmitter` already handles `Some(org_id)` (no code change needed — only a proptest in P3); `create_organization` repo method exists; `HAS_LEAD` edge is missing (→ G1 → P1 work).
2. `docs/specs/v0/implementation/m3/architecture/m2-preflight-delta.md` written with findings + `[STATUS]` per item.

#### phi-core leverage
None — audit phase. Confirm no drift since M2/P8 closed `scripts/check-phi-core-reuse.sh` green.

#### Tests added
0 (audit phase).

#### Confidence check
**Target: N/A** (not an implementation phase). Close criterion: every item in the 9-item list has a `still-valid | stale | missing` tag with file+line reference. If >1 item is `stale`, open a P0.5 remediation phase before P1 opens (not expected).

---

### P1 — Foundation: IDs, nodes, composites, migration, docs tree, web wizard primitives (~2–3 days)

#### Goals
Every M3 surface has stable types + a running migration + wizard primitives + docs seed before P2 begins.

#### Deliverables
1. **Edge enum**: add `HasLead { id, from: ProjectId, to: AgentId }` variant. Bump `EDGE_KIND_NAMES` compile-time count to 67. Update `concepts/ontology.md` edge table + update `m1_edge_type_safety.rs` count constant.
2. **Organization struct extension** in `domain/src/model/nodes.rs`: add `vision: Option<String>`, `mission: Option<String>`, `consent_policy: ConsentPolicy`, `audit_class_default: AuditClass`, `authority_templates_enabled: Vec<TemplateKind>`, `defaults_snapshot: OrganizationDefaultsSnapshot`, `default_model_provider: Option<ModelProviderId>`, `system_agents: Vec<AgentId>`.
3. **New composites** in `domain/src/model/composites_m3.rs`:
   - `ConsentPolicy` enum: `Implicit`, `OneTime`, `PerSession` (phi-only).
   - `OrganizationDefaultsSnapshot` struct wrapping `phi_core::{ExecutionLimits, ContextConfig, RetryConfig, AgentProfile}` directly (four phi-core wraps) + phi retention + alert channels.
   - `TokenBudgetPool` struct: `id`, `owning_org`, `initial_allocation: u64`, `used: u64`, `created_at` (phi-only).
4. **Migration 0003** `store/migrations/0003_org_creation.surql`:
   - Extend `organization` table with new columns (`FLEXIBLE TYPE object` for `defaults_snapshot`, enum-as-string for `consent_policy` / `audit_class_default`).
   - New `token_budget_pool` table.
   - No `organization_drafts` table per D3.
5. **Docs tree seed** `docs/specs/v0/implementation/m3/{README,architecture,user-guide,operations,decisions}/` with `<!-- Last verified -->` headers on every stub file; sketch M3 README with 7-phase status table.
6. **Web wizard primitives** in `modules/web/app/components/wizard/`:
   - `StepShell.tsx` — step container with heading + content slot + error alert slot.
   - `StepNav.tsx` — Back / Next / Save-draft / Submit buttons; disabled states.
   - `DraftContext.tsx` — React Context wrapping sessionStorage + localStorage fallback; `useDraft<T>(key)` hook.
   - `ReviewDiff.tsx` — before/after panel for the wizard review step.
7. **CLI scaffolding**: `cli/src/commands/org.rs` skeleton with stubbed subcommands (`create`, `list`, `show`, `dashboard`). `dashboard` stays disabled until P5.
8. **`ADR-0020`** `decisions/0020-organization-defaults-embedded.md` — D1 rationale.
9. **`ADR-0021`** `decisions/0021-wizard-autosave-session-storage.md` — D3 rationale.

#### phi-core leverage
- **Wrapped in `OrganizationDefaultsSnapshot`**: `phi_core::context::execution::ExecutionLimits`, `phi_core::context::config::ContextConfig`, `phi_core::provider::retry::RetryConfig`, `phi_core::agents::profile::AgentProfile` — four direct wraps (identical pattern to M2/P7's `PlatformDefaults`).
- **Baby-phi-native** (🚫): `ConsentPolicy`, `TokenBudgetPool`, `HasLead` edge, every new field on `Organization` except `defaults_snapshot`.
- **`scripts/check-phi-core-reuse.sh`** continues hard-gated in CI.

#### Tests added (~12)
- `domain/tests/m3_model_counts.rs` — 67-edge count, new composites serde round-trip (3 tests).
- `domain/src/model/composites_m3.rs` unit tests — `OrganizationDefaultsSnapshot::from_platform_defaults(&PlatformDefaults)` produces correct snapshot (non-retroactive invariant); `TokenBudgetPool` arithmetic guards (2 tests).
- `store/tests/migrations_0003_test.rs` — apply / noop / broken (3 tests).
- `modules/web/__tests__/wizard_primitives.test.tsx` — `StepShell` renders, `StepNav` disabled-state logic, `DraftContext` reads + writes sessionStorage (4 tests).

#### Confidence check
**Target: ≥97%.** Close audit verifies:
- **Code correctness**: `cargo test --workspace` green; `cargo clippy -Dwarnings` green; `npm run test/typecheck/lint/build` green.
- **Docs accuracy**: m3 docs tree has 0 broken links; every stub carries `Last verified` header + Status tag; ADR-0020 + ADR-0021 status = Proposed (flip to Accepted at P4/P5 close).
- **phi-core leverage**: `check-phi-core-reuse.sh` zero hits; `OrganizationDefaultsSnapshot` struct definition shows 4 `phi_core::` types as field types (manual grep).

---

### P2 — Repository expansion + Template A/B/C/D builders + M3 audit events (~2 days)

#### Goals
Domain + store surfaces needed by P4/P5 handlers are green and proptested.

#### Deliverables
1. **Org-scoped list methods** in `domain/src/repository.rs`:
   - `list_agents_in_org(org: OrgId) -> Vec<Agent>` + in-memory + SurrealDB impls.
   - `list_projects_in_org(org: OrgId) -> Vec<Project>` (Project struct today is minimal; M4 fleshes it).
   - `list_active_auth_requests_for_org(org: OrgId) -> Vec<AuthRequest>` (filters `auth_requests` where `requestor` is an Agent/Org belonging to `org` AND state ∈ non-terminal).
   - `list_recent_audit_events_for_org(org: OrgId, limit: usize) -> Vec<AuditEvent>` (org_scope = Some(org) OR actor ∈ org members).
   - `list_adoption_auth_requests_for_org(org: OrgId) -> Vec<AuthRequest>` (filters by `provenance_template` ∈ Template{A,B,C,D} AND resource URI belongs to org).
2. **Template A/B/C/D pure-fn builders** `domain/src/templates/{a,b,c,d}.rs`:
   - Each file mirrors `e.rs`'s `BuildArgs` shape but with the template-specific trigger context (Template A: `project_lead: AgentId, project: ProjectId`; Template B: `delegating_agent, delegation_loop`; Template C: `agent: AgentId, org_tree_path: Vec<String>`; Template D: `agent: AgentId, project: ProjectId, role: String`).
   - Each returns an adoption-shaped `AuthRequest` in `Approved` state — the CEO is the sole approver (the CEO principal is an input to the builder).
   - Each builder is a pure fn (no I/O, proptest-friendly).
3. **M3 audit event builders** `domain/src/audit/events/m3/orgs.rs`:
   - `organization_created(actor, org, ceo_agent_id, provenance_ar_id, timestamp) -> AuditEvent` — class = Alerted, org_scope = Some(org.id), target = org's NodeId.
   - `authority_template_adopted(actor, org_id, template_kind, adoption_ar_id, timestamp) -> AuditEvent` — class = Alerted, org_scope = Some(org_id).
4. **M3 audit events mod** `domain/src/audit/events/m3/mod.rs` — `pub mod orgs`.

#### phi-core leverage

Retrofitted to Q1/Q2/Q3 structure post-P3.0. Every claim verified by code re-audit of shipped P2 artefacts (see [leverage checklist §Backstory](../../v0/implementation/m3/architecture/phi-core-leverage-checklist.md#backstory--the-m3p3-leverage-slip)).

**Q1 — Direct imports.** Grep `^use phi_core` in `domain/src/templates/{a,b,c,d,adoption}.rs`, `domain/src/audit/events/m3/orgs.rs`, and the 5 new repo methods in `domain/src/repository.rs` / `domain/src/in_memory.rs` / `store/src/repo_impl.rs` returns **zero hits**. Confirmed by grep at P2 close.

**Q2 — Transitive payload.** Walking each deliverable's data types one level deep:
- Template A/B/C/D adoption builders return `AuthRequest` — fields are `PrincipalRef`, `Vec<ResourceSlot>`, `AuthRequestState`, `AuditClass`, `Option<TemplateId>`, `Vec<String>`, `DateTime<Utc>`. **Zero phi-core types.**
- Org-scoped list methods return `Vec<Agent>` / `Vec<ProjectId>` / `Vec<AuthRequest>` / `Vec<AuditEvent>`. Each struct's fields are 100% phi governance types. **Zero phi-core types.**
- `organization_created` audit builder reads `Organization` but **deliberately excludes `defaults_snapshot` from the diff JSON** ([orgs.rs:90-95](../../v0/implementation/../../../modules/crates/domain/src/audit/events/m3/orgs.rs)) — if it had included the snapshot, phi-core types would transit via serialised JSON. Current scope keeps the diff phi-core-free by construction.
- `authority_template_adopted` audit builder reads only ids + `TemplateKind`. **Zero phi-core types.**

**Q3 — Candidates considered and rejected.** Walking the [phi-core module inventory](../../v0/implementation/m3/architecture/phi-core-leverage-checklist.md#2-for-each-deliverable-ask-q1--q2--q3):
- `phi_core::agents::profile::AgentProfile` — not applicable to P2 deliverables. Template builders don't read profiles; list methods return `Agent` (not `AgentProfile`); audit builders don't touch profiles. Profile materialisation lands in P3.
- `phi_core::provider::model::ModelConfig` — not applicable. M2/P6 already wraps it in `ModelRuntime`; P2 doesn't create `ModelRuntime` rows or wire `UsesModel` edges. P3 does the wiring.
- `phi_core::context::execution::ExecutionLimits` / `ContextConfig` / `RetryConfig` — not applicable. P2 neither reads nor writes `Organization.defaults_snapshot`; the snapshot is a P1 deliverable, consumed in P3.
- `phi_core::types::event::AgentEvent` — **not a substitute** for `domain::audit::AuditEvent`. Per CLAUDE.md §Orthogonal surfaces, the governance audit log and the runtime telemetry stream are intentionally disjoint. P2's audit builders use `domain::audit::AuditEvent` by design.
- `phi_core::session::Session` / `LoopRecord` / `Turn` — not applicable. No session work in P2; session surface lands M5 (D11).
- `phi_core::agent_loop::*` — not applicable. P2 doesn't invoke the loop.
- `phi_core::config::AgentConfig` / schema — not applicable. P2 doesn't parse external configs.
- `phi_core::tools::*` / `phi_core::mcp::*` / `phi_core::openapi::*` — not applicable. P2 wires no tool/MCP/OpenAPI bindings.

**Conclusion.** P2 genuinely sits on phi's **governance plane** — templates, authority requests, audit events, org-scoped list methods — an orthogonal surface per CLAUDE.md. The absence of phi-core reuse is legitimate, not a miss.

**Reuse discipline**: `check-phi-core-reuse.sh` green at P2 close; no phi redeclarations of phi-core types introduced.

#### Tests added (~22)
- `domain/tests/in_memory_m3_test.rs` — 5 new list methods × 2 cases each (populated + empty) = 10.
- `store/tests/repo_m3_surface_test.rs` — 5 list methods × 1 case each = 5.
- `domain/tests/template_a_props.rs` — shape + aggregate Approved + CEO-slot-filled + serde (4 cases minimum; proptest bumps to 50).
- `domain/tests/template_b_props.rs` — same (4 cases / 50 proptest).
- `domain/tests/template_c_props.rs` — same.
- `domain/tests/template_d_props.rs` — same.
- `domain/src/audit/events/m3/orgs.rs` unit tests — `organization_created` + `authority_template_adopted` shape + class = Alerted + org_scope = Some(org_id) (3 tests).

#### Confidence check
**Target: ≥97%.** Close audit verifies:
- **Code correctness**: all 22 new tests green; every org-scoped list method returns empty `Vec` on unknown-org-id (not `Err`).
- **Docs accuracy**: `m3/architecture/authority-templates.md` fleshes out the A/B/C/D adoption flow (M5 wires trigger firing; M3 wires adoption); `Last verified` refreshed.
- **phi-core leverage**: `scripts/check-phi-core-reuse.sh` zero hits; manual grep confirms no `phi_core::` imports in `domain/src/templates/{a,b,c,d}.rs` (these are pure phi).

---

### P3 — handler_support extensions + batch audit + compound tx + harness (~1–2 days)

#### Goals
Server-side primitives needed by P4's compound write are in place; two-org hash-chain isolation is proptested.

#### Deliverables
*Each bullet carries a phi-core tag per the [leverage checklist](../../v0/implementation/m3/architecture/phi-core-leverage-checklist.md) §4.*

1. **Compound tx repo method**: `Repository::apply_org_creation(payload: OrgCreationPayload) -> RepositoryResult<OrgCreationReceipt>` — single SurrealQL tx writing Organization + 2 system agents (`AgentKind::Llm`, `owning_org = Some(org_id)`) + 2 `AgentProfile` nodes + CEO Agent (`AgentKind::Human`) + CEO Channel + CEO inbox + CEO outbox + CEO grant (`[allocate]` on `org:<id>`) + `TokenBudgetPool` + adoption auth requests + the edge set `HasProfile` / `MemberOf` / `HasCeo` / `HasMember` / `HasInbox` / `HasOutbox` / `HasChannel`. Receipt carries every new id for audit-batch emission. **Note**: `UsesModel` edges are **NOT** wired at P3 — the M2 `uses_model` table schema constrains TO-side to the vestigial `model_config` table (not M2/P6's new `model_runtime`); per-agent model binding is deferred to M5 session launch when the runtime invocation path exists. M3 dashboard reads `Organization.default_model_provider` directly. **[phi-core: 🔌 `phi_core::agents::profile::AgentProfile` transits via `AgentProfile.blueprint` (×2 system agents), derived from `Organization.defaults_snapshot.default_agent_profile` with per-role `name`/`system_prompt` tweaks; ♻ `phi_core::ExecutionLimits` / `ContextConfig` / `RetryConfig` — **inherit** from `Organization.defaults_snapshot` (ADR-0023); no per-agent nodes created. `phi_core::ModelConfig` — transit deferred to M5 along with `UsesModel` edge wiring.]**
2. **Batch audit emit** `server/src/handler_support/audit.rs::emit_audit_batch(&dyn AuditEmitter, Vec<AuditEvent>) -> Result<Vec<AuditEventId>, ApiError>` — emits in input order; fails fast with `AUDIT_EMIT_FAILED` on first error. **[phi-core: 🏗 phi-native — governance audit log; `domain::audit::AuditEvent` is explicitly orthogonal to `phi_core::types::event::AgentEvent` per CLAUDE.md §Orthogonal surfaces.]**
3. **Test fixture** `tests/acceptance_common/admin.rs::spawn_claimed_with_org(with_metrics: bool) -> ClaimedOrg` where `ClaimedOrg { admin: ClaimedAdmin, org_id: OrgId, ceo_agent_id: AgentId, system_agents: [AgentId; 2] }`. Internally calls `spawn_claimed` + drives the wizard POST flow to create a minimal-startup-shaped org. **Stub body at P3** (POST endpoint doesn't exist yet); P4 replaces with the real wizard submission. **[phi-core: 🏗 phi-native — test harness; phi-core has no test-fixture counterpart for governance flows.]**
4. **Per-org audit chain proptest** `domain/tests/two_orgs_audit_chain_props.rs` — 50 cases interleaving emits across two orgs; assert each chain's hash sequence is independent. **[phi-core: 🏗 phi-native — chains `AuditEvent` records; per-org hash chain is a phi governance invariant.]**

#### phi-core leverage

Structured per the [leverage checklist](../../v0/implementation/m3/architecture/phi-core-leverage-checklist.md) (Q1 / Q2 / Q3).

**Q1 — Direct imports.** The P3 code (`apply_org_creation` in the repo trait + both impls + `emit_audit_batch` + fixture + proptest) has **zero** `use phi_core::…` statements. Every imported path resolves to phi (`crate::model::…`, `crate::audit::…`, `crate::repository::…`).

**Q2 — Transitive payload.** The `apply_org_creation` compound tx's payload (`OrgCreationPayload`) and receipt (`OrgCreationReceipt`) carry phi-core types through phi wrapper field types:
- **`phi_core::agents::profile::AgentProfile`** — transits via `AgentProfile.blueprint: phi_core::AgentProfile` (×2 system agents). Baseline cloned from `Organization.defaults_snapshot.default_agent_profile` (also phi-core); per-role `name` + `system_prompt` tweaks applied before persist. **First materialised production instances in the project.**
- **`phi_core::provider::model::ModelConfig`** — transits via `ModelRuntime.config: phi_core::ModelConfig` when the `UsesModel` edge is added per system agent against `Organization.default_model_provider`. `ModelRuntime` is not created by P3 (already exists from M2/P6); P3 only wires the edge.
- **♻ Inherit-not-duplicate** (ADR-0023, D12): `phi_core::context::execution::ExecutionLimits`, `phi_core::context::config::ContextConfig`, `phi_core::provider::retry::RetryConfig` — frozen once on `Organization.defaults_snapshot` at creation time; P3 does **not** duplicate them into per-agent nodes. System agents read from the snapshot at invoke time.

**Q3 — Candidates considered and rejected.** Walking the [phi-core module inventory](../../v0/implementation/m3/architecture/phi-core-leverage-checklist.md#2-for-each-deliverable-ask-q1--q2--q3):
- `phi_core::agent_loop::AgentLoopConfig` — not relevant. P3 doesn't invoke the loop; only persists configuration precursors. Loop invocation is M5.
- `phi_core::agents::Agent` / `BasicAgent` — not relevant. P3 persists phi governance `Agent` nodes (kind + owning_org + display_name); runtime Agent trait instantiation is M5.
- `phi_core::agents::SubAgent` — not relevant. M6/M7 multi-agent coordination work; out of M3 scope.
- `phi_core::config::AgentConfig` / schema — not relevant. P3 does not parse external configs; the wizard writes directly into phi governance structs.
- `phi_core::mcp::McpClient` — not relevant. P3 wires no MCP bindings; those live on `ExternalService` rows (M2/P6 + M5 agent-binding).
- `phi_core::session::Session` / `LoopRecord` / `Turn` / `SessionRecorder` — not relevant. No sessions launched at org-creation time; session persistence begins M5 (see D11 in Part 3).
- `phi_core::types::AgentEvent` — **not relevant, and critically not a substitute** for `domain::audit::AuditEvent`. The `emit_audit_batch` helper operates on the latter; conflating the two is an orthogonal-surface mistake per CLAUDE.md.
- `phi_core::types::tool::AgentTool` / `ToolResult` — not relevant. No tool execution in org-creation flow.
- `phi_core::tools::PrunTool` — not relevant. Context management is M5 work.
- `phi_core::provider::StreamProvider` / `StreamConfig` — not relevant. No streaming calls in compound tx.

**Confirmation**: `SurrealAuditEmitter::emit`'s existing call to `self.repo.last_event_hash_for_org(event.org_scope)` (from M2/P3) is what makes per-org chains work — no code change, only verification via the new proptest.

#### Tests added (~5)
- `store/tests/apply_org_creation_tx_test.rs` — happy path + mid-tx failure rolls back every write (2 tests).
- `server/tests/handler_support_test.rs` — extended with `emit_audit_batch_is_in_order` + `emit_audit_batch_fails_fast` cases (2 new tests).
- `domain/tests/two_orgs_audit_chain_props.rs` — 1 proptest, 50 cases.
- `server/tests/spawn_claimed_with_org_smoke.rs` — fixture boots + org exists + CEO agent exists (1 test).

#### Confidence check
**Target: ≥97%.** Close audit verifies (positive assertions, not only negative — per checklist §6):
- **Code correctness**: compound-tx rollback proven (intermediate write failure leaves no orphans); batch-emit order preserved under every permutation of 4 events (exhaustive small-N check); two_orgs proptest asserts zero cross-chain hash collisions across 50 interleavings.
- **Docs accuracy**: `m3/architecture/per-org-audit-chain.md` written and linked from M1's `audit-events.md` (explains the M1/M2/M3 scope boundary: M1/M2 writes go to `None`, M3 writes go to `Some(org_id)`); ADR-0023 flipped to Accepted; leverage-checklist cross-linked from every new phase doc touching phi-core types.
- **phi-core leverage (positive greps)**:
  - Compile-time type coercion test proves each system agent's `AgentProfile.blueprint` field is exactly `phi_core::agents::profile::AgentProfile` (function `is_phi_core_agent_profile(_: &phi_core::…::AgentProfile) {}` called with `&profile.blueprint`).
  - Integration test queries SurrealDB post-`apply_org_creation` and asserts **zero rows** on `execution_limits`, `retry_policy`, `cache_policy`, `compaction_policy` tables (ADR-0023 inherit-from-snapshot invariant).
  - Integration test queries `agent_profile` table post-tx and asserts row count = 2 (system agents) + pre-existing; each blueprint's `system_prompt` field matches the planned role-specific string.
  - Integration test queries edges and asserts the `UsesModel` edge exists for each system agent pointing at the org's `default_model_provider` → `ModelRuntime` (which wraps `phi_core::ModelConfig`).
- **phi-core leverage (negative sanity)**: `check-phi-core-reuse.sh` zero hits; no phi redeclarations of phi-core types introduced.

---

### P4 — Page 06 vertical: Org Creation Wizard (~3–4 days, **largest phase**)

#### Goals
End-to-end org creation: Rust business logic + HTTP handler + CLI `phi org {create,list,show}` + Web 8-step wizard.

#### Deliverables
*Each bullet carries a phi-core tag per the [leverage checklist](../../v0/implementation/m3/architecture/phi-core-leverage-checklist.md) §4.*

1. **Business logic** `server/src/platform/orgs/`:
   - `mod.rs` — shared types (`OrgCreationPayload`, `CreatedOrg`, `OrgError` with stable codes). **[phi-core: 🔌 `OrgCreationPayload` transitively carries `phi_core::AgentProfile` (× system-agent blueprints, cloned from snapshot) + all 4 `OrganizationDefaultsSnapshot` phi-core wraps.]**
   - `create.rs` — orchestrator: validate input → resolve system-agent profiles from `Organization.defaults_snapshot.default_agent_profile` (clone + per-role `system_prompt` override) → build compound payload → `Repository::apply_org_creation` → compose `OrganizationCreated` + N `AuthorityTemplateAdopted` events → `emit_audit_batch`. **[phi-core: 🔌 direct use of `phi_core::agents::profile::AgentProfile` for `.clone()` + field tweaks; ♻ `ExecutionLimits` / `ContextConfig` / `RetryConfig` inherit from snapshot (ADR-0023).]**
   - `list.rs` — org list scoped to the calling admin. **[phi-core: 🏗 phi-native — `Vec<Organization>` summary; `defaults_snapshot` elided from list payload.]**
   - `show.rs` — single-org detail (org fields + member count + project count + recent-events summary — the dashboard-preview payload). **[phi-core: 🔌 if `Organization.defaults_snapshot` is serialised in detail payload, phi-core types transit via JSON; default is elided for summary view per audit-diff precedent.]**
   - **Orchestrator `build_template_suite_adoption(org_id, ceo_principal, enabled: &[TemplateKind], ...)` lives here** (not in domain), per D7. **[phi-core: 🏗 phi-native — governance orchestration; templates have no phi-core counterpart.]**
2. **Handler** `server/src/handlers/orgs.rs` — routes: `POST /api/v0/orgs`, `GET /api/v0/orgs`, `GET /api/v0/orgs/:id`. **[phi-core: 🔌 wire payload carries `phi_core::AgentProfile` transitively (see §Q2 below).]**
3. **Router wiring** in `router.rs`. **[phi-core: 🏗 phi-native.]**
4. **CLI** `cli/src/commands/org.rs` — `create`, `list`, `show` subcommands. `create` accepts either direct flags OR `--from-layout <ref>` to seed from a reference-layout YAML fixture. **[phi-core: 🏗 phi-native — clap-driven CLI; YAML deserialises to phi wire types.]**
5. **Reference-layout fixtures** (3 of 10): `cli/fixtures/reference_layouts/{minimal-startup,mid-product-team,regulated-enterprise}.yaml`. Full 10-layout parity is M8 scope. **[phi-core: 🏗 phi-native — YAML governance shapes; no phi-core type surfaces in fixture fields.]**
6. **Web wizard page** `modules/web/app/(admin)/organizations/`:
   - `page.tsx` / `[id]/page.tsx` / `new/page.tsx` / per-step components `Step{1..8}*.tsx` / `actions.ts` / `lib/api/orgs.ts`. **[phi-core: 🏗 phi-native — React/Next.js UI; no phi-core imports on the web tier (phi-core is a Rust library; web uses TS wire types instead).]**
7. **AdminSidebar** flip: `{href: "/organizations", label: "Organizations", ready: true}`. **[phi-core: 🏗 phi-native.]**
8. **Ops doc** `m3/operations/org-creation-operations.md` — failure/rollback playbook, template-adoption audit trail, CEO-invite message delivery. **[phi-core: 🏗 phi-native — governance ops.]**
9. **`ADR-0022`** `decisions/0022-org-creation-compound-transaction.md` — D6 rationale. **[phi-core: n/a — decision record.]**

#### phi-core leverage

Structured per the [leverage checklist](../../v0/implementation/m3/architecture/phi-core-leverage-checklist.md) (Q1 / Q2 / Q3).

**Q1 — Direct imports.** `server/src/platform/orgs/create.rs` directly imports and uses `phi_core::agents::profile::AgentProfile` when cloning the snapshot baseline and overriding `name`/`system_prompt` for each system agent role. Zero direct imports on web tier (TypeScript/Next.js uses phi wire types). CLI layer has zero direct phi-core imports.

**Q2 — Transitive payload.**
- **`phi_core::agents::profile::AgentProfile`** — transits through `OrgCreationPayload`'s system-agent blueprint fields and persists via `AgentProfile.blueprint`. Server↔wire serialisation preserves phi-core fields in JSON (validated by payload round-trip tests).
- **`phi_core::ExecutionLimits` / `ContextConfig` / `RetryConfig`** — transit via `OrgCreationPayload.defaults_snapshot: OrganizationDefaultsSnapshot` (4 phi-core wraps). The wizard's default path leaves these at the platform defaults (no user override); advanced-mode wizard tweaks write into the snapshot before POST.
- **`phi_core::provider::model::ModelConfig`** — does **not** transit through P4's payload. `Organization.default_model_provider: ModelProviderId` is just the id; the full `ModelRuntime.config: phi_core::ModelConfig` stays in its M2/P6 row and is referenced via `UsesModel` edge. The wire payload carries only the id.

**Q3 — Candidates considered and rejected.** Walking the [phi-core module inventory](../../v0/implementation/m3/architecture/phi-core-leverage-checklist.md#2-for-each-deliverable-ask-q1--q2--q3):
- `phi_core::agents::Agent` / `BasicAgent` — not applicable. P4 persists phi governance agents; runtime `Agent` trait use is M5 session launch.
- `phi_core::config::AgentConfig` / schema — not applicable. The wizard writes into governance types, not external AgentConfig TOML/YAML.
- `phi_core::session::*` — not applicable. Org creation does not launch sessions (D11).
- `phi_core::agent_loop::*` / `phi_core::tools::*` / `phi_core::mcp::*` / `phi_core::openapi::*` — not applicable. P4 wires no loops, tool execution, MCP, or OpenAPI bindings.
- `phi_core::types::AgentEvent` — not applicable. P4's audit events are `domain::audit::AuditEvent` (governance log per CLAUDE.md).
- `phi_core::provider::StreamProvider` — not applicable. No streaming calls in creation path.

**Net**: P4 adds **one** new `use phi_core::agents::profile::AgentProfile;` in `create.rs` (for the clone + tweak). Web tier and CLI tier remain phi-core-import-free.

#### Tests added (~30)
- `server/tests/acceptance_orgs_create.rs` — 9+ scenarios:
  - 5 validation errors (empty `org_id`, bad regex, empty name, no templates, no model provider, negative budget, bad CEO channel) = 5-7.
  - Happy path (minimal-startup reference layout) — 1.
  - 409 duplicate org_id — 1.
  - Partial-failure rollback (inject repo error mid-tx) — 1.
  - Reference-layout fidelity (minimal-startup, mid-product-team, regulated-enterprise create the exact shapes prescribed) — 3.
- `cli/tests/org_create_help.rs` + `cli/tests/org_from_layout_smoke.rs` — help snapshot + YAML consumption (4 tests).
- `modules/web/__tests__/wizard_page06.test.tsx` — per-step component render + validation; `useDraft` round-trip; `StepNav` disabled-state at each step; `ReviewDiff` renders (16 tests; can parametrise over steps for 8 render tests + 8 validation tests).

#### Confidence check
**Target: ≥98%.** Close audit verifies (positive assertions per checklist §6):
- **Code correctness**: all 30 new tests green; compound-tx rollback verified on real SurrealDB (via `acceptance_orgs_create.rs`); audit chain continuity (9 events per minimal-startup: 1 Organization + 2 templates adopted + ... — exact count pinned in a test assertion).
- **Docs accuracy**: `m3/architecture/org-creation.md` matches shipped handler + references ADR-0020/0022; 3 reference-layout fixtures YAML-valid + CLI-parseable; `m3/operations/org-creation-operations.md` status flipped to `[EXISTS]`.
- **phi-core leverage (positive greps)**:
  - `server/src/platform/orgs/create.rs` grep confirms `use phi_core::agents::profile::AgentProfile;` is present (not re-declared locally).
  - Round-trip test: wizard payload → JSON → deserialised `OrgCreationPayload` → every system agent's blueprint is `phi_core::AgentProfile` (type-coercion function).
  - Post-creation SurrealDB query shows 2 rows in `agent_profile` per created org with phi-core-shaped blueprint JSON (fields present: `profile_id`, `name`, `system_prompt`, `thinking_level`, `temperature`, `max_tokens`, `config_id`, `skills`, `workspace`).
  - ADR-0023 invariant re-verified: 0 rows on `execution_limits` / `retry_policy` / `cache_policy` / `compaction_policy` tables post-creation.
- **phi-core leverage (negative sanity)**: `check-phi-core-reuse.sh` zero hits.

---

### P5 — Page 07 vertical: Organization Dashboard (~2 days)

#### Goals
End-to-end dashboard: Rust business logic + HTTP handler + CLI `phi org dashboard` + Web dashboard page with live-polling panels.

#### Deliverables
1. **Business logic** `server/src/platform/orgs/dashboard.rs` — aggregate read orchestrator calling the 5 org-scoped list methods from P2 + token-budget utilisation read + CTA-card visibility logic (shown when counter < threshold).
2. **Handler** `GET /api/v0/orgs/:id/dashboard` returning consolidated `DashboardSummary` JSON.
3. **CLI** `cli/src/commands/org.rs::dashboard` subcommand (enabled in P5; stubbed in P1). `--id <uuid>` + `--json` for scripting.
4. **Web dashboard page** `modules/web/app/(admin)/organizations/[id]/dashboard/page.tsx` with panels:
   - `OrgHeader` (name, vision, mission, role).
   - `AgentsSummary` (grouped by kind — Human/Intern/Contract/System).
   - `ProjectsSummary` (count + Shape A/B breakdown — Shape info is aspirational in M3; M4 populates).
   - `PendingAuthRequests` — polls every 30 s (D4).
   - `AlertedEventsCount` — last-24h count; polls every 30 s.
   - `TokenBudget` — `used / total` from `token_budget_pool`.
   - `RecentAuditEvents` — 5 most recent, clickable.
   - `AdoptedTemplates` — read-only list.
   - `EmptyStateCtaCards` — 4 cards (Add Agent / Create Project / Templates / System Agents) shown only when counters near zero (R-ADMIN-07-R8).
5. **Ops doc** `m3/operations/org-dashboard-operations.md` — polling cadence, M7b upgrade path to WebSocket, dashboard data-freshness SLOs.

#### phi-core leverage

Structured per the [leverage checklist](../../v0/implementation/m3/architecture/phi-core-leverage-checklist.md) (Q1 / Q2 / Q3).

**Q1 — Direct imports.** Zero. Dashboard aggregate reads call phi repo methods (`list_agents_in_org`, `list_projects_in_org`, `list_active_auth_requests_for_org`, `list_recent_audit_events_for_org`, `list_adoption_auth_requests_for_org`, `get_organization`). All return phi types.

**Q2 — Transitive payload.** The `DashboardSummary` JSON surfaced to the client contains:
- `OrgHeader` — name/vision/mission/role: pure phi.
- `AgentsSummary` — counts by `AgentKind`: pure phi.
- `ProjectsSummary` — count: phi ids only.
- `PendingAuthRequests` — `AuthRequest` fields: pure phi.
- `AlertedEventsCount` — integer count: no type carriage.
- `TokenBudget` — `used / total` from `TokenBudgetPool`: pure phi (no phi-core counterpart for budget pools).
- `RecentAuditEvents` — `AuditEvent` with `diff: serde_json::Value`; `organization_created` diff **deliberately excludes `defaults_snapshot`** (P2 precedent), so phi-core types don't transit.
- `AdoptedTemplates` — `Template` records: pure phi.
- `EmptyStateCtaCards` — UI flags: no type carriage.

**Net transit: none.** If a future revision elects to surface the snapshot in `OrgHeader` (so operators can see the frozen phi-core defaults without drill-down), phi-core types would transit via JSON — that's an explicit design choice deferred beyond M3.

**Q3 — Candidates considered and rejected.** Walking the [phi-core module inventory](../../v0/implementation/m3/architecture/phi-core-leverage-checklist.md#2-for-each-deliverable-ask-q1--q2--q3):
- `phi_core::agents::profile::AgentProfile` — not applicable. `AgentsSummary` shows counts, not profile details. If a future agents-list drill-down surfaces per-agent blueprints, profiles will transit through that endpoint (not P5's scope).
- `phi_core::session::Session` / `LoopRecord` / `Turn` — not applicable directly. The `RecentAuditEvents` panel will in M5+ deep-link to session traces when `actor_agent_id` ties to a session with `session_id` provenance (D11 / Q6) — M3's row-level link targets the audit-log detail page.
- `phi_core::ExecutionLimits` / `ContextConfig` / `RetryConfig` — not applicable. Dashboard doesn't render per-agent limits at M3. If operators need to inspect effective limits for an agent, an M5/M6 "agent detail" page will drill into them.
- `phi_core::types::Usage` — not applicable. `TokenBudget` at M3 is org-level budget pool (phi economic resource), not phi-core's per-turn token usage. If future dashboards surface phi-core `Usage` stats aggregated per-agent, that's M5+ session-analytics work.
- `phi_core::types::AgentEvent` — not applicable. Dashboard reads `AuditEvent` rows; conflation is forbidden per CLAUDE.md.
- `phi_core::provider::*` / `phi_core::agent_loop::*` / `phi_core::mcp::*` / `phi_core::tools::*` — not applicable. No runtime-side surfaces on the dashboard.

**Conclusion.** P5 sits entirely on phi's governance-plane aggregate-read surface. The absence of phi-core reuse is legitimate, not a miss.

#### Tests added (~16)
- `server/tests/acceptance_orgs_dashboard.rs` — 6 scenarios: fresh-org empty state + CTA cards shown; populated-org counters correct; 403 on non-member GET; token-budget reflects writes; polling-latency (two sequential GETs see audit-event delta); filtered view for project-lead role.
- `cli/tests/org_dashboard_help.rs` — help snapshot + `--json` roundtrip (2 tests).
- `modules/web/__tests__/dashboard_page07.test.tsx` — per-panel render + empty-state CTA cards + polling interval setup + wire translator round-trip (8 tests).

#### Confidence check
**Target: ≥98%.** Close audit verifies:
- **Code correctness**: all 16 new tests green; dashboard GET is read-only (no audit event emitted — matches M2 pattern for list ops); polling works under cypress/Playwright smoke (deferred: M7b).
- **Docs accuracy**: `m3/architecture/org-dashboard.md` + `m3/user-guide/org-dashboard-usage.md` shipped; runbook ops status flipped to `[EXISTS]`; 30 s polling cadence pinned.
- **phi-core leverage (positive grep)**: P5 serde-layer tests assert the `DashboardSummary` JSON schema **excludes** any phi-core-wrapping fields (no `execution_limits`, `context_config`, `retry_config`, `default_agent_profile` at the top level of any panel's payload — each panel's shape is pinned in a snapshot test). If a future phase wants to surface snapshot fields, the JSON-schema snapshot must be updated deliberately.
- **phi-core leverage (negative sanity)**: `check-phi-core-reuse.sh` zero hits; no phi local duplications of `Organization` / `AgentProfile` / `ExecutionLimits`.

---

### P6 — Seal: cross-page acceptance + CLI completion + CI + runbook + re-audit (~1–2 days)

#### Goals
M3 closes. Cross-page acceptance proves compose; CI extensions include new binaries; runbook + troubleshooting aggregate M3; independent re-audit targets ≥99%.

#### Deliverables
1. **Cross-page acceptance** `server/tests/acceptance_m3.rs` — one end-to-end scenario:
   - `spawn_claimed` → POST `/api/v0/orgs` (wizard payload) → GET `/api/v0/orgs/:id/dashboard` → verify audit chain (expected 5+ events: OrganizationCreated + N AuthorityTemplateAdopted + optional system-agent provisioning trace); hash-chain continuity verified on `org_scope = Some(org_id)`.
2. **Acceptance metrics extension** `server/tests/acceptance_metrics.rs` — scraper also asserts an org-creation path recorded in axum-prometheus HTTP metrics (non-zero `POST /api/v0/orgs` count).
3. **CI updates** `.github/workflows/rust.yml`:
   - Extend `acceptance` job with `acceptance_orgs_create` + `acceptance_orgs_dashboard` + `acceptance_m3` binaries.
   - `check-ops-doc-headers` job stays; new M3 ops docs must carry the header.
4. **CLI completion auto-extension**: no code changes (clap_complete walks the subcommand tree). Add a regression test `cli/tests/completion_help.rs` asserts `org` appears in bash/zsh/fish/powershell output.
5. **Ops runbook M3 section** in `docs/ops/runbook.md` — mirrors M2's structure: 2 per-page runbook index + M3 error-code reference + incident playbooks (e.g. "wizard submit fails halfway", "dashboard shows stale data — flush + re-poll").
6. **M3 troubleshooting** `m3/user-guide/troubleshooting.md` with tables for every M3 stable code (`ORG_ID_IN_USE`, `TEMPLATE_NOT_ADOPTABLE`, `SYSTEM_AGENT_PROFILE_MISSING`, etc.) + CLI exit codes inherited from M2.
7. **Independent re-audit** (mirrors M2/P8): 3 parallel Explore agents cover (a) Rust implementation across P0–P5, (b) docs + verification matrix, (c) per-page vertical integrity (page 06 + 07 fidelity to requirements docs). Target ≥99%; remediate LOW findings in the same session before M3 closes.
8. **Archive this plan** at `phi/docs/specs/plan/build/<8hex>-m3-organization-creation.md`.

#### phi-core leverage

Structured per the [leverage checklist](../../v0/implementation/m3/architecture/phi-core-leverage-checklist.md) (Q1 / Q2 / Q3).

**Q1 — Direct imports.** Zero new phi-core imports. `clap_complete` (not a phi-core type; M2/P8's CLI completion helper) auto-surfaces `org {create,list,show,dashboard}` in every shell's completion script via subcommand-tree walk.

**Q2 — Transitive payload.** `acceptance_m3.rs` exercises the full org-creation → dashboard → audit-chain flow; it inherits P3+P4+P5's phi-core transit (system agents' `phi_core::AgentProfile` blueprints persisted via `apply_org_creation`, dashboard reads excluding phi-core fields per P5's schema pin). The acceptance test itself imports zero phi-core types — it asserts on HTTP response JSON and audit chain state, both of which are phi wire types / SurrealDB rows.

**Q3 — Candidates considered and rejected.** The re-audit agents (P6.7) spot-check every M3 composite + node extension against phi-core for miss-leverage opportunities. Candidates explicitly ruled out:
- `phi_core::types::Usage` — not applicable. Metrics extension asserts HTTP request count (axum-prometheus), not token usage.
- `phi_core::session::SessionRecorder` — not applicable at M3. Acceptance test does not launch sessions; D11 / Q6 define M5 integration.
- `phi_core::types::AgentEvent` — not applicable. Acceptance test asserts on `AuditEvent` chain state.
- `phi_core::agent_loop::*` / `phi_core::tools::*` / `phi_core::mcp::*` / `phi_core::openapi::*` — not applicable. No loop invocation, tool execution, or external bindings in P6.

**Net**: P6 is pure sealing work; no new phi-core imports. The leverage invariants from P3/P4/P5 (positive greps, inherit-from-snapshot) are re-verified by the independent agent audit.

#### Tests added (~4)
- `acceptance_m3.rs` — 1 end-to-end scenario.
- `acceptance_metrics.rs` — 1 extension assertion (org-path metric recorded).
- `cli/tests/completion_help.rs` — 1 regression test for `org` appearing in completion.
- P6 re-audit may surface LOW findings; budget ~1 test for targeted remediation.

#### Confidence check
**Target: ≥99%** via **independent Explore-agent audit** (M2/P8 precedent). Close criteria:
- **Code correctness**: `acceptance_m3.rs` green; `cargo test --workspace` green; all CI jobs (`rust.yml`: fmt / clippy / test / proptest / acceptance / audit / deny / phi-core-reuse / doc-links / ops-doc-headers) green.
- **Docs accuracy**: `m3/README.md` shows every phase ✓; ADRs 0020/0021/0022/0023 accepted; `docs/ops/runbook.md` has M3 section; `m3/user-guide/troubleshooting.md` covers every M3 stable code; [leverage checklist](../../v0/implementation/m3/architecture/phi-core-leverage-checklist.md) cross-linked from every phase's phi-core subsection.
- **phi-core leverage (positive reaudit)**: re-audit agent walks every phase's Q1/Q2/Q3 subsection against the shipped code; confirms deliverable-level tags match actual imports and transitive carriage. ADR-0023's inherit-from-snapshot invariant re-verified (0 rows in per-agent policy tables across every org created by the test suite). All positive-grep assertions from P3/P4/P5 re-run green on the M3-final codebase.
- **phi-core leverage (negative sanity)**: `scripts/check-phi-core-reuse.sh` zero hits; re-audit agent spot-checks every M3 composite (`OrganizationDefaultsSnapshot`, `TokenBudgetPool`, `ConsentPolicy`) + every new Organization field for phi-core type references where applicable.

---

**Total phase estimate: ~11 calendar days ≈ 2.5 weeks.** Within the build-plan M3 envelope (~2 weeks) + 0.5 week buffer for the wizard-primitives-first discipline.

---

## Part 5 — Testing strategy  `[STATUS: ⏳ pending]`

M3 aggregate additions:

| Layer | New M3 count | Purpose |
|---|---|---|
| Domain unit (composites_m3 + template A/B/C/D builders + audit builders) | ~15 | Type shapes + pure helpers |
| Domain proptest (template A/B/C/D + two-orgs audit chain + OrgDefaultsSnapshot non-retroactive) | ~6 proptests | Behaviour invariants |
| Store unit (migration 0003) | 3 | Migration shape |
| Store integration (repo org-scoped surface + apply_org_creation tx) | ~7 | Persistence + compound-tx |
| Server unit (batch emit + handler helpers) | ~3 | handler_support extensions |
| Server integration (org creation + dashboard + m3 cross-page + metrics) | ~22 | HTTP contract + compound flow |
| Acceptance E2E (orgs_create + orgs_dashboard + acceptance_m3) | ~16 scenarios | Real axum + real SurrealDB |
| CLI integration (org subcommand help + from-layout + completion regression) | ~7 | Subcommand + YAML parity |
| Web unit (wizard primitives + wire translators + dashboard panels) | ~22 | Pure React + translators |
| Web SSR smoke (wizard page + dashboard page render path) | ~4 | SSR probe + auth gate |
| **M3 added total** | **~105 tests** (slightly above Plan-agent's 80–85 estimate; parametric wizard-step tests keep this in check if we go high on web) | — |

**Post-M3 workspace target**: 511 (M2 close) + ~83 Rust ≈ **~594 Rust**; 36 + ~22 Web ≈ **~58 Web**; **~652 combined**, up from M2's 547.

**Key invariants shipping in M3**:
- Two-org audit-chain isolation (no cross-contamination).
- Org-creation compound-transaction atomicity (all-or-nothing rollback on partial failure).
- Template A/B/C/D pure-fn builders produce Approved AR with CEO-slot filled.
- OrganizationDefaultsSnapshot non-retroactive (PlatformDefaults change does NOT mutate existing orgs — pinned by proptest from M2/P7, re-verified in P1).
- `spawn_claimed_with_org` fixture produces a deterministic minimal-startup-shaped org reproducible across CI.

**Fixtures planned**:
- `cli/fixtures/reference_layouts/{minimal-startup,mid-product-team,regulated-enterprise}.yaml` — 3 of 10 reference layouts; full parity is M8.
- `server/tests/acceptance_common/admin.rs::spawn_claimed_with_org` — extended harness.

---

## Part 6 — Documentation  `[STATUS: ⏳ pending]`

Root: `phi/docs/specs/v0/implementation/m3/`. Layout mirrors M2.

```
implementation/m3/
├── README.md                                  7-phase index + ADR table
├── architecture/
│   ├── overview.md                            M3 system map
│   ├── m2-preflight-delta.md                  P0's 9-item delta log
│   ├── organization-defaults-embedded.md      D1 durable publication
│   ├── org-creation.md                        P4 compound-tx flow
│   ├── org-dashboard.md                       P5 aggregate-read architecture
│   ├── authority-templates.md                 A/B/C/D constructors + adoption flow (M5 wires trigger firing)
│   ├── per-org-audit-chain.md                 M2/M3 scope boundary (None vs Some(org_id))
│   ├── wizard-primitives.md                   P1 web wizard building blocks
│   ├── phi-core-reuse-map.md                  durable §1.5 publication
│   └── server-topology.md                     extends M2 with /orgs routes
├── user-guide/
│   ├── org-creation-walkthrough.md            end-to-end 8-step tour
│   ├── org-dashboard-usage.md                 operator-facing walkthrough
│   ├── cli-reference-m3.md                    new `org` subcommand surface
│   └── troubleshooting.md                     M3 stable codes + recovery
├── operations/
│   ├── org-creation-operations.md             failure / rollback playbook
│   └── org-dashboard-operations.md            polling cadence + SLOs
└── decisions/
    ├── 0020-organization-defaults-embedded.md            D1
    ├── 0021-wizard-autosave-session-storage.md           D3
    └── 0022-org-creation-compound-transaction.md         D6
```

**Conventions** (unchanged from M2, enforced by `doc-links.yml` + `check-ops-doc-headers.sh`): `<!-- Last verified: YYYY-MM-DD by Claude Code -->` on line 1; status tags `[EXISTS]` / `[PLANNED M3/Pn]` / `[CONCEPTUAL]`; relative-link depth `../../../../../../modules/` from m3/{architecture,user-guide,operations,decisions}/; ADR numbering continues from M2 (0020+); docs for a phase land in the same commit as that phase's code.

---

## Part 7 — CI / CD extensions  `[STATUS: ⏳ pending]`

1. **`rust.yml` `acceptance` job** — extend `--test` list with 3 new binaries: `acceptance_orgs_create`, `acceptance_orgs_dashboard`, `acceptance_m3`. Keep `--test-threads 1` (acceptance_metrics still needs the OnceLock discipline).
2. **`rust.yml` `phi-core-reuse` job** — unchanged (already hard-gated in M2). P6 close audit verifies it stays green.
3. **`doc-links.yml`** — new M3 docs tree; no rule change required (P1 seeds correct 6-`../` depth).
4. **`ops-doc-headers` job** — unchanged; new M3 ops runbooks must carry the `Last verified` header (enforced).
5. **`spec-drift.yml`** — extend grep set with `R-ADMIN-0[67]-*` requirement ids.

---

## Part 8 — Verification matrix  `[STATUS: ⏳ pending]`

| # | Commitment | Test / check |
|---|---|---|
| C1 | M2 pre-flight delta log | `m3/architecture/m2-preflight-delta.md` written with 9 items; no LOW remediation needed |
| C2 | M3 composites + `HasLead` edge + 67-edge count | `domain/tests/m3_model_counts.rs` |
| C3 | Migration 0003 forward-only | `store/tests/migrations_0003_test.rs` |
| C4 | Web wizard primitives | `modules/web/__tests__/wizard_primitives.test.tsx` |
| C5 | Repository org-scoped surface | `domain/tests/in_memory_m3_test.rs` + `store/tests/repo_m3_surface_test.rs` |
| C6 | Template A/B/C/D pure-fn builders | `domain/tests/template_{a,b,c,d}_props.rs` |
| C7 | M3 audit event builders | unit tests in `audit/events/m3/orgs.rs` |
| C8 | Per-org hash-chain isolation | `domain/tests/two_orgs_audit_chain_props.rs` |
| C9 | Compound tx + batch emit | `store/tests/apply_org_creation_tx_test.rs` + `server/tests/handler_support_test.rs` batch-emit cases |
| C10 | `spawn_claimed_with_org` harness | `server/tests/spawn_claimed_with_org_smoke.rs` |
| C11 | Page 06 org creation vertical | `server/tests/acceptance_orgs_create.rs` + CLI help + web wizard component tests |
| C12 | Page 07 org dashboard vertical | `server/tests/acceptance_orgs_dashboard.rs` + CLI help + web dashboard component tests |
| C13 | Cross-page acceptance + metrics | `server/tests/acceptance_m3.rs` + `acceptance_metrics.rs` extension |
| C14 | CLI completion auto-extension | `cli/tests/completion_help.rs` regression |
| C15 | CI extensions | `.github/workflows/rust.yml` green on PR |
| C16 | Ops docs + runbook + troubleshooting | `check-doc-links.sh` + `check-ops-doc-headers.sh` green |
| C17 | phi-core reuse per-phase | each phase's `### phi-core leverage` subsection + `check-phi-core-reuse.sh` green across every phase close |
| C18 | Per-phase 3-aspect confidence check | each phase's `### Confidence check` subsection reports target % + close verification + actual % before next phase opens |

**First-review confidence target: ≥ 98 %**. Post-P6 re-audit: ≥ 99 %.

---

## Part 9 — Execution order  `[STATUS: ⏳ pending]`

0. **Archive this plan** → `phi/docs/specs/plan/build/<8hex>-m3-organization-creation.md`. Generate token via `openssl rand -hex 4`. (~2 min)
1. **P0 pre-flight delta log** — 9-item audit; write findings; decide whether any remediation opens a P0.5 (not expected). (~2–4 h)
2. **P1 Foundation** — `HasLead` edge + `Organization` extension + `composites_m3` + migration 0003 + docs tree + web wizard primitives + ADR-0020/0021 + CLI scaffolding. (~2–3 d)
3. **P2 Domain + repo expansion** — org-scoped list methods + Template A/B/C/D pure-fn builders + M3 audit events. (~2 d)
4. **P3 handler_support extensions** — compound `apply_org_creation` + `emit_audit_batch` + `spawn_claimed_with_org` + two-org chain proptest. (~1–2 d)
5. **P4 Page 06 vertical** — org creation (Rust + CLI + Web wizard) + reference-layout fixtures + ADR-0022. (~3–4 d)
6. **P5 Page 07 vertical** — org dashboard (Rust + CLI + Web). (~2 d)
7. **P6 Seal** — cross-page acceptance + metrics extension + CI updates + runbook + troubleshooting + independent re-audit → ≥99%. (~1–2 d)
8. **Re-audit → remediation → 100 %** (mirrors M2 post-P8 pass).
9. **Tag milestone** — `git tag v0.1-m3` in `phi` submodule (user-managed per M1/M2 precedent).

**Total estimate: ~11 calendar days ≈ 2.5 weeks.** Within build-plan M3's 2-week envelope + 0.5-week buffer.

---

## Part 10 — Critical files  `[STATUS: n/a]`

**New** (~40 production files + ~30 test files + ~18 docs):
- `modules/crates/domain/src/model/composites_m3.rs` — `ConsentPolicy`, `OrganizationDefaultsSnapshot`, `TokenBudgetPool`.
- `modules/crates/domain/src/templates/{a,b,c,d}.rs` — per-template pure-fn builders.
- `modules/crates/domain/src/audit/events/m3/{mod,orgs}.rs` — M3 event builders.
- `modules/crates/store/migrations/0003_org_creation.surql` — schema migration.
- `modules/crates/store/src/repo_impl_m3.rs` — M3 repo method impls + `apply_org_creation` compound tx.
- `modules/crates/server/src/platform/orgs/{mod,create,list,show,dashboard}.rs` — business logic.
- `modules/crates/server/src/handlers/orgs.rs` — HTTP shims.
- `modules/crates/cli/src/commands/org.rs` — CLI subcommand + reference-layout loader.
- `modules/crates/cli/fixtures/reference_layouts/{minimal-startup,mid-product-team,regulated-enterprise}.yaml` — fixture YAMLs.
- `modules/crates/server/tests/acceptance_common/admin.rs` — `spawn_claimed_with_org`.
- `modules/web/app/components/wizard/{StepShell,StepNav,DraftContext,ReviewDiff}.tsx` — reusable primitives.
- `modules/web/app/(admin)/organizations/**/*.{tsx,ts}` — pages 06 + 07 + shared.
- `modules/web/lib/api/orgs.ts` + `modules/web/lib/api/org-dashboard.ts` — wire translators.
- `docs/specs/v0/implementation/m3/**/*.md` — full tree (18 docs).

**Modified**:
- `modules/crates/domain/src/model/{nodes,edges,mod}.rs` — `Organization` extension + `HasLead` edge + `EDGE_KIND_NAMES` bump.
- `modules/crates/domain/src/repository.rs` — 5 new org-scoped list methods + `apply_org_creation`.
- `modules/crates/domain/src/in_memory.rs` — matching in-memory impls.
- `modules/crates/server/src/handler_support/audit.rs` — `emit_audit_batch`.
- `modules/crates/server/src/router.rs` — 3 new routes + 1 dashboard route.
- `modules/crates/server/src/platform/mod.rs` — `pub mod orgs`.
- `modules/crates/server/src/handlers/mod.rs` — `pub mod orgs`.
- `modules/crates/cli/src/main.rs` + `cli/src/commands/mod.rs` — new `Org` subcommand.
- `modules/web/app/(admin)/components/AdminSidebar.tsx` — "Organizations" entry (`ready: true`).
- `.github/workflows/rust.yml` — extended acceptance job's `--test` list.
- `docs/ops/runbook.md` — M3 section.
- `docs/specs/v0/implementation/m1/user-guide/troubleshooting.md` — cross-link to new m3/ troubleshooting.
- `docs/specs/v0/concepts/ontology.md` — `HAS_LEAD` edge added to inventory table.

---

## Part 11 — Open questions (non-blocking)  `[STATUS: n/a]`

Track in per-phase exec notes, not here:
- **Q1** (G15 / D2): Template F's purpose — reserved for M6 break-glass. Revisit in M6 planning.
- **Q2** (G16 / D6): Compound-tx scope at M4 — does project creation reuse `apply_org_creation`-style single-tx, or a new `apply_project_creation`? Likely the latter; confirm at M4 planning.
- **Q3** (D9): CEO inbox message delivery — M7b wires real channel delivery; confirm the inbox-row shape in M3 is forward-compatible when M7b adds the delivery hook.
- **Q4** (D4): Dashboard polling → WebSocket push migration — M7b. Document expected behaviour change + a feature flag for gradual rollout.
- **Q5** (G18): 10 reference-layout fixture parity — M3 ships 3; M8's release-prep lands the other 7. Confirm at M8 planning.
- **Q6** (D11 follow-through): Org → Agent → Session drill-down link target. M3's dashboard `RecentAuditEvents` rows link to the audit-log page by event id. M5's session-launch work should upgrade the link so rows with a session-provenance attachment resolve to `/sessions/<session_id>` (phi_core::Session trace view) instead. Confirm at M5 planning that the audit-event schema carries enough provenance (`session_id` on every agent-loop-generated event) to support this deep-link without a follow-up migration.

---

## What stays unchanged  `[STATUS: n/a]`

- Concept docs (`docs/specs/v0/concepts/`) are the source of truth; M3 surface-count corrections land in the build plan if discovered during P0 pre-flight.
- M2 ships unchanged; M3 extends (new types + routes + audit events + docs tree), doesn't refactor M2 surfaces.
- `phi-core` is a library dependency; M3 consumption mirrors M2's pattern — no new phi-core coupling beyond the 4 already wrapped in P7's `PlatformDefaults` (which `OrganizationDefaultsSnapshot` snapshot-copies).
- The `handler_support` shim (session extractor + Permission Check wrapper + audit emitter + ApiError envelope) — M3 extends with `emit_audit_batch` but does not refactor the existing surface.
