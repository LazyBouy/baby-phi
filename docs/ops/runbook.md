<!-- Last verified: 2026-04-21 by Claude Code -->

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
