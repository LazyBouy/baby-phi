<!-- Last verified: 2026-04-23 by Claude Code -->

# phi operations runbook

> **Status:** partial. The full runbook is an **M7b** deliverable per the v0.1 build plan (see [`../specs/plan/build/36d0c6c5-build-plan-v01.md`](../specs/plan/build/36d0c6c5-build-plan-v01.md) §M7b). M2/P8 aggregates every shipped admin-page runbook below so operators have one cross-reference index.

## Sections that will land in M7b

- **Deploy** — pushing a new tagged release to staging + prod (Docker image pull, orchestrator rolling update, post-deploy smoke).
- **Upgrade** — moving from version N to N+1, including SurrealDB schema-migration preflight.
- **Rollback** — reverting to the previous Docker image tag and compensating migrations.
- **Backup** — on-demand `surreal export` + verifying the automated daily off-site dump.
- **Restore** — rebuilding a new node from the off-site dump; drill cadence and pass criteria.
- **Incident response** — 5 canonical scenarios with decision trees:
  1. DB process crash / corruption.
  2. TLS cert expired / renewal failed.
  3. Auth IdP outage (cascading to user login failures).
  4. Audit-log stream failure (hash-chain break).
  5. Unexpected p99 latency regression.
- **Known issues & workarounds** — living list, updated per release.

## Where to look in the meantime

Until the runbook is populated, operators should consult:

- [`../../CLAUDE.md`](../../CLAUDE.md) — workspace layout and local commands.
- [`../specs/v0/implementation/m0/operations/`](../specs/v0/implementation/m0/operations/) — M0 deployment, config-profile, TLS, CI, and security-scanning guidance.
- [`../specs/plan/build/36d0c6c5-build-plan-v01.md`](../specs/plan/build/36d0c6c5-build-plan-v01.md) — the v0.1 build plan (intent, not current runbook).

## M2 — Platform Setup (aggregated runbook index)

M2 shipped four admin pages plus shared governance infrastructure (Template-E auto-approve, handler_support shim, audit hash-chain, credentials vault crypto). Per-page ops runbooks + the P8 seal doc:

- [`../specs/v0/implementation/m2/operations/secrets-vault-operations.md`](../specs/v0/implementation/m2/operations/secrets-vault-operations.md) — page 04. Rotation policy, reveal audit trail, master-key rotation placeholder, "I lost my custodian" recovery.
- [`../specs/v0/implementation/m2/operations/model-provider-operations.md`](../specs/v0/implementation/m2/operations/model-provider-operations.md) — page 02. Registration flow, health-probe incident (shape-only in M2; real probe M7b), archival.
- [`../specs/v0/implementation/m2/operations/mcp-server-operations.md`](../specs/v0/implementation/m2/operations/mcp-server-operations.md) — page 03. Tenant-narrowing cascade semantics, emergency over-narrow playbook.
- [`../specs/v0/implementation/m2/operations/platform-defaults-operations.md`](../specs/v0/implementation/m2/operations/platform-defaults-operations.md) — page 05. Optimistic-concurrency recovery, factory reset, non-retroactive invariant.
- [`../specs/v0/implementation/m2/operations/m2-additions-to-m1-ops.md`](../specs/v0/implementation/m2/operations/m2-additions-to-m1-ops.md) — deltas to the M1 at-rest / audit-retention / schema-migrations ops pages.

### M2 stable error codes (grep-able reference)

The full catalogue lives in [`../specs/v0/implementation/m2/user-guide/troubleshooting.md`](../specs/v0/implementation/m2/user-guide/troubleshooting.md). Cross-cutting codes:

- `UNAUTHENTICATED` (401) — session cookie missing/expired.
- `VALIDATION_FAILED` (400) — request body failed shape or bounds check.
- `AUDIT_EMIT_FAILED` (500) — hash-chain write returned an error; underlying write MAY have succeeded.
- `PLATFORM_DEFAULTS_STALE_WRITE` (409) — optimistic-concurrency mismatch on page 05.
- `MODEL_PROVIDER_DUPLICATE` (409) — `(provider, config.id)` pair already registered.
- `SECRET_REF_NOT_FOUND` (400) — referenced vault slug doesn't exist.

### M2 incident playbooks

- **Accidental MCP tenant over-narrow** — [`../specs/v0/implementation/m2/operations/mcp-server-operations.md`](../specs/v0/implementation/m2/operations/mcp-server-operations.md#5-emergency-playbook--accidental-over-narrow). Forward-only cascade; widening back does NOT restore revoked grants.
- **Audit-chain break** — hash-chain write failure produces `AUDIT_EMIT_FAILED`. Verify via a GET before retrying; the underlying data write may have succeeded on an earlier path. Full recovery drill lands in M7b.
- **Vault reveal denial** — operator can't decrypt a secret. Usually custodian mismatch or Permission-Check `CONSTRAINT_VIOLATION` on `purpose=reveal`; see [`../specs/v0/implementation/m2/operations/secrets-vault-operations.md`](../specs/v0/implementation/m2/operations/secrets-vault-operations.md).

## M3 — Organization Creation + Dashboard (aggregated runbook index)

M3 shipped admin pages 06 (org creation wizard) and 07 (org
dashboard) on top of M2's platform infrastructure. First milestone
where audit events leave the root chain and start per-org chains
(`org_scope = Some(org_id)`). Per-page ops runbooks:

- [`../specs/v0/implementation/m3/operations/org-creation-operations.md`](../specs/v0/implementation/m3/operations/org-creation-operations.md) — page 06. Compound-tx rollback, audit-batch emit, CEO-inbox row semantics, reference-layout fixtures.
- [`../specs/v0/implementation/m3/operations/org-dashboard-operations.md`](../specs/v0/implementation/m3/operations/org-dashboard-operations.md) — page 07. 30s polling cadence, response-code reference, three incident playbooks (stale-data / 403-unexpected / token-budget 0/0), M7b WebSocket upgrade path.

### M3 stable error codes (grep-able reference)

The full catalogue lives in [`../specs/v0/implementation/m3/user-guide/troubleshooting.md`](../specs/v0/implementation/m3/user-guide/troubleshooting.md). Newly-introduced codes at M3:

- `ORG_ID_IN_USE` (409) — POST /orgs received an org_id that already exists. Server mints ids, so this is observed on compound-tx retry.
- `TEMPLATE_NOT_ADOPTABLE` (400) — wizard payload requested adoption of Template E / F / SystemBootstrap. Only A/B/C/D are adoptable at creation time.
- `ORG_NOT_FOUND` (404) — GET /orgs/:id or /dashboard on an unknown id.
- `ORG_ACCESS_DENIED` (403) — dashboard GET from a viewer with no `MEMBER_OF` to the org. The platform admin is intentionally NOT a member of orgs they create — the CEO is.
- `VALIDATION_FAILED` (400) — M2-originating code, reused for M3 wizard field validation (empty display_name, zero token_budget, duplicate templates).
- `AUDIT_EMIT_FAILED` (500) — M2-originating; now triggered by the M3 batch emit (organization_created + N authority_template.adopted). A successful compound commit is durable before the batch runs, so an AUDIT_EMIT_FAILED after 201 means the org is persisted but its audit entries may be partial. Full recovery drill lands in M7b.

### M3 incident playbooks

- **Compound-tx partial write / rollback** — [`../specs/v0/implementation/m3/operations/org-creation-operations.md`](../specs/v0/implementation/m3/operations/org-creation-operations.md). `apply_org_creation` wraps every write in one SurrealQL transaction; any statement failure leaves zero persisted rows. Verification: after a 500 from POST /orgs, GET /orgs should NOT list the attempted id.
- **Dashboard shows stale data** — [`../specs/v0/implementation/m3/operations/org-dashboard-operations.md`](../specs/v0/implementation/m3/operations/org-dashboard-operations.md). Client-side 30s polling; server-side aggregates are pushdowns — no cache layer to flush.
- **Per-org audit chain break** — any Alerted event emitted post-org-creation should hash-chain onto the org's existing events. A `prev_event_hash = None` on a non-genesis event signals a break; investigate before emitting the next event. The `acceptance_m3::wizard_to_dashboard_preserves_audit_chain_and_counts` test pins this invariant on the happy path; M5+ session-launch reuses the same chain.

### M3 phi-core leverage enforcement (for operators touching code)

M3 is the first milestone where audit events transit a payload that
wraps phi-core types (`defaults_snapshot` via `Organization.defaults_snapshot`).
The dashboard endpoint intentionally strips this to keep polling
contracts stable. Operators extending M3 surfaces must run the
[phi-core leverage checklist](../specs/v0/implementation/m3/architecture/phi-core-leverage-checklist.md)'s
four-tier enforcement model — treating `check-phi-core-reuse.sh`
alone as sufficient is **rejected by review**. See the checklist's
"Enforcement — four-tier model" section for the required discipline.

## M4 — Agents + Projects (aggregated runbook index)

M4 shipped admin pages 08 (agent roster), 09 (agent profile editor),
10 (project creation wizard — Shape A + Shape B), 11 (project
detail). First milestone to materialise the `Project` node + write
`HAS_LEAD` edges in production. First milestone to introduce a two-
approver Auth Request flow (Shape B, co-owned projects). First
milestone to extend agents with the 6-variant `AgentRole` spanning
Human (Executive / Admin / Member) and LLM (Intern / Contract /
System) agents. First milestone to ship a domain event bus
(`TemplateAFireListener` subscribes to `HasLeadEdgeCreated` and
fires the Template A grant automatically). Per-page ops runbooks:

- [`../specs/v0/implementation/m4/operations/agent-roster-operations.md`](../specs/v0/implementation/m4/operations/agent-roster-operations.md) — page 08. Role filter UX, search-prefix semantics, roster read-back edge cases.
- [`../specs/v0/implementation/m4/operations/agent-profile-editor-operations.md`](../specs/v0/implementation/m4/operations/agent-profile-editor-operations.md) — page 09. Create + edit + revert-override flows, `ExecutionLimits` inherit vs override decision tree, `is_valid_for(kind)` guard, three ExecutionLimits paths (inherit / override-set / override-revert).
- [`../specs/v0/implementation/m4/operations/project-creation-operations.md`](../specs/v0/implementation/m4/operations/project-creation-operations.md) — page 10. Shape A (immediate) vs Shape B (two-approver) decision tree, 4-outcome approval matrix (both-approve / both-deny / mixed), Shape B materialisation deferral (C-M5-6).
- [`../specs/v0/implementation/m4/operations/project-detail-operations.md`](../specs/v0/implementation/m4/operations/project-detail-operations.md) — page 11. In-place OKR patch semantics, "Recent sessions" placeholder (empty until M5 / C-M5-3).

### M4 stable error codes (grep-able reference)

The full catalogue lives in [`../specs/v0/implementation/m4/user-guide/troubleshooting.md`](../specs/v0/implementation/m4/user-guide/troubleshooting.md). Newly-introduced codes at M4:

- `AGENT_ID_IN_USE` (409) — POST /orgs/:org_id/agents received an agent id collision (rare; server mints uuids).
- `AGENT_IMMUTABLE_FIELD_CHANGED` (400) — PATCH /agents/:id/profile tried to change `id` / `kind` / `role` / `base_organization` (immutable post-creation at M4 scope).
- `AGENT_ROLE_INVALID_FOR_KIND` (400) — Create/update tried to assign a role the `is_valid_for(kind)` predicate rejects (e.g. Intern role on a Human agent).
- `PARALLELIZE_CEILING_EXCEEDED` (400) — Create/update supplied a `parallelize` value outside `[1, org_cap]`.
- `EXECUTION_LIMITS_EXCEED_ORG_CEILING` (400) — per-agent override violates the `≤ org snapshot` invariant pinned by ADR-0027.
- `ACTIVE_SESSIONS_BLOCK_MODEL_CHANGE` (409) — PATCH tried to change `ModelConfig` while the agent has in-flight sessions (D-M4-3). **M4 note**: `Repository::count_active_sessions_for_agent` is a stub returning `Ok(0)` at M4 — the 409 code path is wired but never fires until M5 / C-M5-5 activates the real query.
- `SYSTEM_AGENT_READ_ONLY` (403) — edit attempted on a `role=system` agent; system agents are managed by platform ops only.
- `SHAPE_B_MISSING_CO_OWNER` (400) — POST /orgs/:org_id/projects requested Shape B without `co_owner_org_id`.
- `SHAPE_A_HAS_CO_OWNER` (400) — Shape A with a `co_owner_org_id` supplied (Shape A is single-org by definition).
- `CO_OWNER_INVALID` (400) — co-owner org equal to primary owner OR co-owner org not found.
- `LEAD_NOT_IN_OWNING_ORG` (400) — lead agent's `owning_org` isn't one of the project's owning orgs.
- `MEMBER_INVALID` (400) — member or sponsor id not found OR not in an owning org.
- `PROJECT_ID_IN_USE` (409) — POST /orgs/:org_id/projects received a `project_id` that already exists. Server-minted UUIDs; retry the operator form.
- `PENDING_AR_NOT_FOUND` (404) / `PENDING_AR_NOT_SHAPE_B` (400) / `PENDING_AR_ALREADY_TERMINAL` (409) — approve-pending handler call errors.
- `APPROVER_NOT_AUTHORIZED` (403) — caller on `POST /projects/_pending/:ar_id/approve` isn't one of the two approver slots. Also reused for the page-11 OKR-patch access gate.
- `PROJECT_NOT_FOUND` (404) — GET /projects/:id on an unknown id.
- `PROJECT_ACCESS_DENIED` (403) — viewer is not a member of any owning org and is not on the project roster.
- `OKR_VALIDATION_FAILED` (400) — PATCH /projects/:id/okrs violates a shape rule (duplicate id, missing parent objective, measurement-type mismatch, delete on objective with dependent KRs).
- `TRANSITION_ILLEGAL` (400) — AR state-machine transition rejected by `transition_slot` (Shape B approval flow).

### M4 incident playbooks

- **Orphaned Agent on `apply_agent_creation` partial failure** — [`../specs/v0/implementation/m4/operations/agent-profile-editor-operations.md`](../specs/v0/implementation/m4/operations/agent-profile-editor-operations.md). The compound tx wraps `Agent` + `Inbox` + `Outbox` + default grants + optional profile + optional `agent_execution_limits` row. A 500 on POST /agents means the full transaction rolled back; verify via GET /orgs/:org_id/agents. A 201 response means the full row set is durable.
- **Shape B approval-deadlock** — [`../specs/v0/implementation/m4/operations/project-creation-operations.md`](../specs/v0/implementation/m4/operations/project-creation-operations.md). If one slot approves and the other never responds, the AR stays in `Pending` forever at M4 scope (`expires_at` is set to 30 days; automatic cancellation is M5+). Operators can force-cancel by driving a `Denied` through the remaining slot with an explanatory reason.
- **Template A grant missed by the listener** — [`../specs/v0/implementation/m4/architecture/template-a-firing.md`](../specs/v0/implementation/m4/architecture/template-a-firing.md). Listener errors are logged with the emitting `event_id` so operators can replay. M4 does NOT auto-retry; a failed grant emission leaves the project materialised without a lead grant. Fire the grant manually via the admin page or wait for M7b's event-retry infra.
- **Dashboard role counters drift** — [`../specs/v0/implementation/m4/operations/agent-profile-editor-operations.md`](../specs/v0/implementation/m4/operations/agent-profile-editor-operations.md). The dashboard's `AgentsSummary` 6-role buckets derive from `Agent.role` — agents with `role=None` (pre-M4 rows OR freshly-created agents before role assignment) show up in the `unclassified` bucket. If the count is unexpectedly high, operators assign roles via page 09. The bucket is intentional; not a bug.
- **OKR patch partial audit emission** — [`../specs/v0/implementation/m4/operations/project-detail-operations.md`](../specs/v0/implementation/m4/operations/project-detail-operations.md). The patch is sequential: each entry mutates in-memory vectors, then the full Project row is upserted, then audits emit one-per-entry. If the upsert succeeded but audit emission failed mid-patch, the row holds the full post-image but the chain has only N-1 events. Run the M1 audit-chain repair playbook.

### M4 phi-core leverage — durable map

- Agent-profile editor (page 09) is M4's phi-core-heaviest file — 4 direct imports in [`agents/update.rs`](../specs/v0/implementation/m4/architecture/phi-core-reuse-map.md) (`AgentProfile`, `ExecutionLimits`, `ModelConfig`, `ThinkingLevel`). Every other M4 surface is phi-core-import-free by design.
- Compile-time coercion tests (3 per hotspot) + positive greps keep the wraps aligned; `check-phi-core-reuse.sh` runs on every PR.
- Wire-shape schema snapshots strip phi-core-wrapping fields at the dashboard + project-detail tiers (tests: `dashboard_summary_wire_shape_excludes_phi_core_fields`, `wire_shape_strips_phi_core`, `show_happy_path`'s forbidden-key assertion).
- The phi-core reuse map doc is the durable reference: [`../specs/v0/implementation/m4/architecture/phi-core-reuse-map.md`](../specs/v0/implementation/m4/architecture/phi-core-reuse-map.md). See that doc for per-page tables + the P8 close-audit record.

### M4 known deferrals (read this before reporting a gap as a bug)

Per the base build plan's pinned carryovers (see [`../specs/plan/build/36d0c6c5-build-plan-v01.md`](../specs/plan/build/36d0c6c5-build-plan-v01.md) §M5 and §M8):

- **C-M5-3**: baby-phi governance `Session` node persistence. Until M5 ships, page 11's "Recent sessions" panel returns `[]` and the dashboard has no session-count tiles.
- **C-M5-4**: Per-agent tool binding. M4 agents don't yet carry an editable tool set; `AgentTool` resolution happens at session-start time in M5.
- **C-M5-5**: `Repository::count_active_sessions_for_agent` real implementation. Stub returns `Ok(0)` at M4; `ACTIVE_SESSIONS_BLOCK_MODEL_CHANGE` never fires until M5.
- **C-M5-6**: Shape B materialisation-after-approve. `POST /projects/_pending/:ar_id/approve` returns `Terminal { state: Approved, project_id: None }` at M4; M5 wires the compound-tx materialisation path.
- **C-M5-7** (informal): roster read-back for project members/sponsors. Edges are written at creation but page 11's roster panel surfaces only the lead. Dedicated repo method lands at M5 alongside session launch.
- **C-M8-1**: `phi project create --from-layout` + 3–5 project-layout YAML fixtures. Deferred to M8 per D-M4-6.
