<!-- Last verified: 2026-04-20 by Claude Code -->

# System Bootstrap flow (s01)

P5 ships the atomic bootstrap flow: `phi-server bootstrap-init`
generates the single-use credential; the claim handler materialises the
System Bootstrap Template adoption as a regular Auth Request in
`Approved` state and issues the `[allocate]`-on-`system:root` Grant that
roots every subsequent authority chain in the system.

- Install-side module: [`modules/crates/server/src/bootstrap/init.rs`](../../../../../../modules/crates/server/src/bootstrap/init.rs)
- Claim business logic: [`modules/crates/server/src/bootstrap/claim.rs`](../../../../../../modules/crates/server/src/bootstrap/claim.rs)
- Credential hashing: [`modules/crates/server/src/bootstrap/credential.rs`](../../../../../../modules/crates/server/src/bootstrap/credential.rs)
- Atomic repository method: [`Repository::apply_bootstrap_claim`](../../../../../../modules/crates/domain/src/repository.rs)
- Concept doc: [`concepts/permissions/02`](../../../concepts/permissions/02-auth-request.md) §System Bootstrap Template
- Requirement docs:
  [`requirements/system/s01`](../../../requirements/system/s01-bootstrap-template-adoption.md) +
  [`requirements/admin/01`](../../../requirements/admin/01-platform-bootstrap-claim.md)
- ADR: [0011 — Bootstrap credential: argon2id-hashed, stdout-delivered, single-use](../decisions/0011-bootstrap-credential-single-use.md)

## Two phases — init and claim

```
╔══════════════════════════════════════════════════════════════════════╗
║                    Install-time (run ONCE per install)               ║
╚══════════════════════════════════════════════════════════════════════╝

    $ phi-server bootstrap-init
    ┌──────────────────────────────────────────┐
    │  rand::OsRng → 32 raw bytes              │
    │  base64url-no-pad encode                 │
    │  prefix with "bphi-bootstrap-"           │
    │           │                              │
    │           ▼                              │
    │  argon2id::hash_password(plaintext)      │
    │           │                              │
    │           ▼                              │
    │  Repository::put_bootstrap_credential    │
    │    → stores the PHC-encoded hash         │
    │      in `bootstrap_credentials.digest`    │
    │           │                              │
    │           ▼                              │
    │  println!(plaintext) — ONCE              │
    │  (admin copies it, plaintext is GONE     │
    │   from the server after the process exits) │
    └──────────────────────────────────────────┘


╔══════════════════════════════════════════════════════════════════════╗
║                 Claim-time (at most ONCE per install)                ║
╚══════════════════════════════════════════════════════════════════════╝

    POST /api/v0/bootstrap/claim          (P6 wires the HTTP handler;
        { bootstrap_credential,            P5 ships the pure business logic
          display_name,                    execute_claim(&dyn Repository,
          channel_kind, channel_handle }    ClaimInput))

      ┌─────────────────────────────────────────────────────────────┐
      │ Step 1 — Validate input (400 on empty fields)               │
      └───────────────────────┬─────────────────────────────────────┘
                              ▼
      ┌─────────────────────────────────────────────────────────────┐
      │ Step 2 — Reject if admin already exists (409)               │
      │   Repository::get_admin_agent()                             │
      └───────────────────────┬─────────────────────────────────────┘
                              ▼
      ┌─────────────────────────────────────────────────────────────┐
      │ Step 3 — Scan bootstrap_credentials, verify each argon2id   │
      │          hash against supplied plaintext                    │
      │   → unconsumed match: continue                              │
      │   → consumed match:   403 BOOTSTRAP_ALREADY_CONSUMED        │
      │   → no match:         403 BOOTSTRAP_INVALID                 │
      └───────────────────────┬─────────────────────────────────────┘
                              ▼
      ┌─────────────────────────────────────────────────────────────┐
      │ Step 4 — Build the seven entities + catalogue seeds:        │
      │   • Human Agent (R-SYS-s01-2)                               │
      │   • Channel (from submitted kind + handle)                  │
      │   • Inbox + Outbox (R-SYS-s01-3)                            │
      │   • Bootstrap Auth Request (R-SYS-s01-1)                    │
      │       state = Approved, requestor = approver = system:genesis│
      │   • Grant `[allocate]` on `system:root`  (R-SYS-s01-4)       │
      │       delegable = true; descends_from = Auth Request.id     │
      │   • PlatformAdminClaimed audit event (Alerted)              │
      │   • Catalogue seeds: system:root + inbox:… + outbox:…       │
      └───────────────────────┬─────────────────────────────────────┘
                              ▼
      ┌─────────────────────────────────────────────────────────────┐
      │ Step 5 — Repository::apply_bootstrap_claim(&claim)          │
      │   Single SurrealDB BEGIN / COMMIT transaction:              │
      │     CREATE agent                                            │
      │     CREATE channel                                          │
      │     CREATE inbox_object                                     │
      │     CREATE outbox_object                                    │
      │     CREATE auth_request                                     │
      │     CREATE grant                                            │
      │     CREATE audit_events                                     │
      │     CREATE resources_catalogue × N                          │
      │     UPDATE bootstrap_credentials SET consumed_at = now      │
      │                                                             │
      │   Any error inside the transaction rolls back EVERYTHING.   │
      │   The credential stays unconsumed → admin may retry.        │
      └───────────────────────┬─────────────────────────────────────┘
                              ▼
      201 Created
        { human_agent_id, inbox_id, outbox_id, grant_id,
          bootstrap_auth_request_id, audit_event_id }
```

## Entity shape at the end of the flow

| Entity | Key fields |
|---|---|
| `Agent` | `kind = Human`, `owning_org = None`, caller-supplied `display_name` |
| `Channel` | kind = Slack / Email / Web; handle = caller-supplied |
| `InboxObject` / `OutboxObject` | `agent_id` points at the new human |
| `AuthRequest` | `state = Approved`; `requestor = approver = system:genesis`; `scope = [allocate]`; `resource_slots = [{ system:root → system:genesis Approved }]`; `provenance_template = UUID::nil()` (the hardcoded axiom) |
| `Grant` | `holder = Agent(new human)`; `action = [allocate]`; `resource.uri = system:root`; `descends_from = bootstrap_auth_request_id`; `delegable = true` |
| `AuditEvent` | `event_type = platform_admin.claimed`; `audit_class = Alerted`; `prev_event_hash = None` (genesis of the platform chain); `target_entity_id = new agent's node id` |
| `resources_catalogue` seeds | `system:root` (control_plane_object) + `inbox:<uuid>` + `outbox:<uuid>` |

## Atomicity — the single load-bearing guarantee

R-SYS-s01-6 requires all seven writes be atomic. The implementation
uses a single SurrealDB `BEGIN TRANSACTION … COMMIT TRANSACTION`
envelope inside [`SurrealStore::apply_bootstrap_claim`](../../../../../../modules/crates/store/src/repo_impl.rs).
If **any** query inside the envelope errors, SurrealDB rolls the batch
back — nothing survives. The store integration test
[`apply_bootstrap_claim_is_idempotent_failure_when_agent_id_collides`](../../../../../../modules/crates/store/tests/repository_test.rs)
pins this: pre-creating the target agent_id forces the inner CREATE to
fail, and the test asserts that the grant / auth request / catalogue
entries / credential-consumption **all** rolled back.

The in-memory `Repository` fake mirrors the contract: it takes a single
write-lock, performs pre-condition checks up front, then applies every
write in the same locked region. Failure paths surface before any
mutation happens.

## Error map

| Condition | Rejection | HTTP (P6) |
|---|---|---|
| Empty `display_name` / `channel_handle` / `bootstrap_credential` | `Invalid` | 400 |
| Platform admin already exists | `AlreadyClaimed` | 409 |
| Supplied credential doesn't verify any stored hash | `CredentialInvalid` | 403 `BOOTSTRAP_INVALID` |
| Supplied credential verifies a stored hash but it's already consumed | `CredentialAlreadyConsumed` | 403 `BOOTSTRAP_ALREADY_CONSUMED` |
| Repository / hashing internal error | `ClaimError::Repository` / `Hash` | 500 |

## What P5 doesn't ship (left for P6)

- The HTTP handler itself (`POST /api/v0/bootstrap/claim`,
  `GET /api/v0/bootstrap/status`). P6 will translate `ClaimOutcome` /
  `ClaimError` → axum responses, set the session cookie on success,
  and emit the Prometheus counter `phi_bootstrap_claims_total`.
- The Prometheus metric. The observability spec in
  `requirements/system/s01-bootstrap-template-adoption.md` names
  `phi_bootstrap_claims_total{result=success|invalid|expired|already_consumed}`;
  the counter landing alongside the handler is cleaner than threading
  a metrics sink through `execute_claim`.

## Test coverage

| Layer | File | Tests |
|---|---|---|
| Server unit (credential hashing) | [`credential.rs`](../../../../../../modules/crates/server/src/bootstrap/credential.rs) | 5 |
| Server unit (credential generation) | [`init.rs`](../../../../../../modules/crates/server/src/bootstrap/init.rs) | 3 |
| Server unit (claim business logic) | [`claim.rs`](../../../../../../modules/crates/server/src/bootstrap/claim.rs) | 5 (happy path; invalid credential; reused credential with admin; reused credential without admin; empty display_name) |
| Store integration (atomic SurrealDB write) | [`repository_test.rs`](../../../../../../modules/crates/store/tests/repository_test.rs) | 3 (happy path; rollback on collision; `list_bootstrap_credentials` contract) |

Total new tests: **16** (260 workspace total after P5, up from 244 after
P4).
