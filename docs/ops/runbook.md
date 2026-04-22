<!-- Last verified: 2026-04-22 by Claude Code -->

# baby-phi operations runbook

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
