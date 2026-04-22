<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Requirements doc — system flow -->

# s01 — Bootstrap Template Adoption

## Purpose

Realises the axiomatic root of the authority tree. When a fresh install's first Human Agent claims the bootstrap credential (via [admin/01-platform-bootstrap-claim.md](../admin/01-platform-bootstrap-claim.md)), this flow materialises the hardcoded System Bootstrap Template as a regular Auth Request in Approved state, emitting the Grant that gives the claimant `[allocate]` on `system:root`.

This is the **one** system behaviour that cannot itself trace to an earlier Auth Request — it is the root. Every subsequent grant in the system ultimately descends from it.

## Trigger

The REST call `POST /api/v0/bootstrap/claim` handled by [admin/01-platform-bootstrap-claim.md](../admin/01-platform-bootstrap-claim.md) Section 10.

## Preconditions

- No platform admin currently exists. (A prior successful call means no claim remains possible.)
- The supplied bootstrap credential is valid, unexpired, and not already consumed.
- The supplied Human Agent profile fields pass validation.

## Behaviour

- **R-SYS-s01-1:** The flow SHALL instantiate the hardcoded `template:system_bootstrap` definition and emit a regular Auth Request with:
  - `requestor: system:genesis`
  - `kinds: [every registered composite]`
  - `scope: [allocate]`
  - `resource_slots: [{ resource: system:root, approvers: [{ approver: system:genesis, state: Approved, responded_at: now }] }]`
  - `state: Approved`
  - `provenance: template:system_bootstrap@<init_time>`
  - `audit_class: alerted`
- **R-SYS-s01-2:** The flow SHALL create a new `Agent` node of kind Human with the submitted display_name and channel. It SHALL NOT assign a `ModelConfig` / `ExecutionLimits` (Humans don't use those).
- **R-SYS-s01-3:** The flow SHALL create the Human Agent's `inbox_object` and `outbox_object` composite instances and add them to a platform-level `resources_catalogue` seed (the catalogue itself is a `control_plane_object` that the Bootstrap adoption instantiates).
- **R-SYS-s01-4:** The flow SHALL issue a Grant with `[allocate]` on `system:root` to the new Human Agent. The Grant's provenance is the Bootstrap adoption Auth Request. `delegable: true` (this grant underpins all subsequent delegation).
- **R-SYS-s01-5:** The flow SHALL mark the bootstrap credential consumed (single-use per [admin/01 W3](../admin/01-platform-bootstrap-claim.md#6-write-requirements)).
- **R-SYS-s01-6:** All steps above SHALL be **atomic** — if any step fails, the flow rolls back; the credential remains available for retry.

## Side effects

- A single alerted audit event: `PlatformAdminClaimed { human_agent_id, display_name, channel, claimed_at, bootstrap_credential_digest }`.
- The audit event references `template:system_bootstrap` in its provenance; any audit trace starting from downstream grants eventually reaches this event.

## Failure modes

- **Expired credential** → returns 403 BOOTSTRAP_EXPIRED to caller; no audit event; no graph writes.
- **Concurrent second claim attempt** → one succeeds; the other returns 409 "A platform admin has already been claimed"; concurrency is serialised on a single bootstrap state value.
- **Partial failure after Human Agent node creation but before Grant issuance** → rollback SHALL remove the Human Agent node and its inbox/outbox; if full rollback fails (rare, storage-level issue), a recovery audit event `BootstrapAdoptionIncomplete` is emitted and admin intervention is required.

## Observability

- Alerted audit event `PlatformAdminClaimed`.
- Metric: `phi_bootstrap_claims_total{result="success|invalid|expired|already_consumed"}`.

## Cross-References

**Concept files:**
- [concepts/permissions/02 § System Bootstrap Template — Root of the Authority Tree](../../concepts/permissions/02-auth-request.md#system-bootstrap-template--root-of-the-authority-tree) — the template definition.
- [concepts/permissions/04 § The Authority Chain](../../concepts/permissions/04-manifest-and-resolution.md#the-authority-chain).

**Admin page provisioning this flow:**
- [admin/01-platform-bootstrap-claim.md](../admin/01-platform-bootstrap-claim.md).

**Downstream:**
- Every subsequent Auth Request in the system traces up through this one.
