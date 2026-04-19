<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->

# NFR — Security

> Security properties of the permission model translated to testable invariants. These are **mandatory invariants** — the implementation MUST maintain them or it is broken. Each is phrased so that a test (unit, property-based, or penetration) can assert it.

## Permission model invariants

- **R-NFR-security-1 — No ambient authority.** Every resource access SHALL go through the Permission Check ([concepts/permissions/04 § Formal Algorithm (Pseudocode)](../../concepts/permissions/04-manifest-and-resolution.md#formal-algorithm-pseudocode)). There SHALL NOT exist any code path that reads or writes a resource without first verifying grants. Test: attempt to bypass by calling a lower-level API directly; the call SHALL fail with a permission error.
- **R-NFR-security-2 — Catalogue precondition (Step 0).** No resource access SHALL succeed on a resource not present in the owning org's `resources_catalogue`. Test: attempt to access a resource with a valid-looking reference but no catalogue entry; Permission Check SHALL return `Denied(reason="resource not in owning org's catalogue", failed_step=0)`.
- **R-NFR-security-3 — Grant provenance is structural.** Every Grant SHALL have a `DESCENDS_FROM` edge to an Auth Request (except the single Bootstrap Adoption grant which descends from the hardcoded template). Test: grep the graph for Grant nodes without `DESCENDS_FROM`; expect zero (or exactly one, the bootstrap).
- **R-NFR-security-4 — Forward-only revocation.** Revoking a Grant SHALL NOT alter any past audit-log entries. Test: revoke a Grant and verify the session reads it covered still appear in the audit log with their original `audit_class`.
- **R-NFR-security-5 — Consent independence.** For co-owned resources, each co-owner's `consent_policy` SHALL be evaluated independently at Permission Check Step 6 per [concepts/permissions/06 § rule 6](../../concepts/permissions/06-multi-scope-consent.md#co-ownership--multi-scope-session-access). Test: with Acme `implicit` + Beta `one_time`, an access lacking a Beta Consent SHALL return `Pending` even if Acme is satisfied.
- **R-NFR-security-6 — Ceiling intersection.** For co-owned resources with conflicting ceilings, the effective ceiling SHALL be the intersection per [concepts/permissions/06 § rule 2](../../concepts/permissions/06-multi-scope-consent.md#co-ownership--multi-scope-session-access). Test: Acme allows `bash`, Beta forbids; a grant attempt authorising `bash` on a joint-research resource SHALL be rejected.
- **R-NFR-security-7 — No authority escalation through tool composition.** An agent SHALL NOT gain capabilities by invoking a chain of tools that individually have lesser grants. The Permission Check runs on every tool invocation; the intersection of the agent's grants and each tool's manifest is the upper bound. Test: agent has `[read]` on fs but not `[write]`; a tool that reads-then-writes SHALL fail the second reach.
- **R-NFR-security-8 — Archived records remain auditable.** Archived Auth Requests SHALL be retrievable via `inspect_archived` ([concepts/permissions/02 § Retention Policy](../../concepts/permissions/02-auth-request.md#retention-policy)) with appropriate approval. Test: archive a record; query with `inspect_archived`; the record's full content is returned.

## Credential and secret handling

- **R-NFR-security-9 — Secrets never in logs.** No secret/credential material SHALL appear in application logs, audit events, or any API response except the vault's explicit `Reveal` endpoint ([admin/04-platform-credentials-vault.md W4](../admin/04-platform-credentials-vault.md#6-write-requirements)). Test: grep for known test secret values in logs after a session that consumes them; expect zero hits.
- **R-NFR-security-10 — Reveal is audit-alerted.** Every secret reveal SHALL emit an alerted audit event BEFORE the secret is returned to the caller. Test: revoke audit logging mid-reveal; reveal should fail closed (fail-safe).
- **R-NFR-security-11 — Credential rotation.** The system SHALL track `last_rotated_at` on every `secret/credential` entry and surface rotation-due reminders via [system/s06-periodic-triggers.md](../system/s06-periodic-triggers.md). Overdue rotation produces alerted events.

## Cross-org isolation

- **R-NFR-security-12 — No implicit cross-org access.** An agent in org A SHALL NOT have access to any resource in org B without an explicit Template E Auth Request approved by B's admin. Test: enumerate cross-org grants; every one has a referenced Auth Request with approver in the target org.
- **R-NFR-security-13 — Tenant isolation of shared resources.** When a tenant org references a platform-level resource (e.g., `claude-sonnet-default` from [organizations/10-platform-infra.md](../../organizations/10-platform-infra.md)), the reference is via a grant with `tenants_allowed` constraint. Narrowing the constraint revokes outstanding tenant-side uses forward-only.

## Fail-safe defaults

- **R-NFR-security-14 — Deny by default.** An unrecognised action, an ambiguous grant match, or any uncovered reach SHALL default to Deny. No "fail open" paths.
- **R-NFR-security-15 — Consent timeout defaults to Deny.** Per [concepts/permissions/06 § Per-Session Consent](../../concepts/permissions/06-multi-scope-consent.md#per-session-consent), a consent that times out SHALL default to Deny unless the org has explicitly configured `approval_timeout_default_response: allow`.

## Bootstrap credential

- **R-NFR-security-16 — Bootstrap is single-use.** A bootstrap credential SHALL be invalidated the moment it is consumed, even if the downstream claim subsequently fails. A failed claim requires a new install or a manual admin-override to produce a new credential.
- **R-NFR-security-17 — Bootstrap credential is not logged.** The bootstrap credential value SHALL NEVER appear in logs; only its digest is recorded for audit correlation.

## Cross-references

- [concepts/permissions/04 § Formal Algorithm (Pseudocode)](../../concepts/permissions/04-manifest-and-resolution.md#formal-algorithm-pseudocode) — the check invariants bind to.
- [concepts/permissions/02 § Retention Policy](../../concepts/permissions/02-auth-request.md#retention-policy) — retention + deletion rules.
- [concepts/permissions/06 § Co-Ownership × Multi-Scope Session Access](../../concepts/permissions/06-multi-scope-consent.md#co-ownership--multi-scope-session-access).
- [system/s01-bootstrap-template-adoption.md](../system/s01-bootstrap-template-adoption.md) — bootstrap flow.
