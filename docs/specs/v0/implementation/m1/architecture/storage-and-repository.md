<!-- Last verified: 2026-04-20 by Claude Code -->

# Architecture — storage and the Repository trait

M1/P2 grows the `Repository` trait from the M0 `ping()` stub into the full
surface the rest of the milestone builds on: node CRUD, grants, auth
requests, typed ownership edges (see [ADR-0015](../decisions/0015-type-safe-ownership-edges.md)),
bootstrap credentials, the resources catalogue, and audit events.

The trait lives in
[`modules/crates/domain/src/repository.rs`](../../../../../../modules/crates/domain/src/repository.rs);
the SurrealDB implementation is in
[`modules/crates/store/src/repo_impl.rs`](../../../../../../modules/crates/store/src/repo_impl.rs);
an in-memory fake for tests is in
[`modules/crates/domain/src/in_memory.rs`](../../../../../../modules/crates/domain/src/in_memory.rs).

## Object-safe trait + typed free-function wrappers

The `Repository` trait is dispatched through `Arc<dyn Repository>`, which
means it must stay object-safe — no generic methods. That conflicts with
the plan's commitment to compile-time type safety on the three ownership
edges (`owned_by` / `created` / `allocated_to`), since those accept the
Resource / Principal type unions.

Resolution (ADR-0015): the trait exposes **raw** methods that take
`NodeId` + `NodeId`; **typed free functions** in the same module use the
sealed `Principal` / `Resource` marker traits and delegate to the raw
methods. Callers should prefer the typed entry points:

```rust
use domain::repository::{self, Repository};

// Compile-time safe — wrong-pair types fail to build:
repository::upsert_ownership(&repo, &memory_id, &user_id, None).await?;
repository::upsert_creation(&repo, &agent_id, &memory_id).await?;
repository::upsert_allocation(&repo, &org_id, &project_id, &resource, auth_id).await?;
```

The raw methods (`upsert_ownership_raw`, etc.) remain accessible for
callers that already have a `NodeId` (e.g. bulk migration scripts); they
just don't give the compile-time guarantee.

## Trait surface (36 async methods)

Grouped by concern. P2 shipped 33; P5 added `apply_bootstrap_claim`
(the atomic seven-writes batch for the s01 adoption flow) and P6
added `list_bootstrap_credentials` (needed by the claim handler's
verify-per-row scan — see ADR-0011 §Lookup). The final post-P9 audit
added `get_audit_event` so the P9 acceptance suite can read an audit
row by id and assert class + provenance directly (not by proxy).

| Group | Methods |
|---|---|
| Health | `ping` |
| Agent + Org CRUD | `create_agent`, `get_agent`, `create_agent_profile`, `create_user`, `create_organization`, `get_organization`, `create_template`, `create_channel`, `create_inbox`, `create_outbox`, `create_memory`, `create_consent`, `create_tool_authority_manifest`, `get_admin_agent` |
| Grants | `create_grant`, `get_grant`, `revoke_grant`, `list_grants_for_principal` |
| Auth Requests | `create_auth_request`, `get_auth_request`, `update_auth_request`, `list_active_auth_requests_for_resource` |
| Ownership edges (raw) | `upsert_ownership_raw`, `upsert_creation_raw`, `upsert_allocation_raw` |
| Bootstrap credentials | `put_bootstrap_credential`, `find_unconsumed_credential`, `consume_bootstrap_credential`, `list_bootstrap_credentials` |
| Bootstrap flow | `apply_bootstrap_claim` |
| Resources catalogue | `seed_catalogue_entry`, `catalogue_contains` |
| Audit | `write_audit_event`, `get_audit_event`, `last_event_hash_for_org` |

Plus three free-function wrappers in the same module — the typed entry
points for ownership edges.

## Persistence conventions

### Record IDs mirror domain IDs

Every domain newtype `*Id` serializes as a UUID string. The SurrealDB
record id uses `type::thing('agent', $uuid)` so the DB-side id carries
the same UUID the domain uses. One source of truth, no translation
tables.

### CONTENT + OMIT id for simple structs

The 9 "simple" nodes (Agent, AgentProfile, User, Organization, Template,
Channel, Inbox, Outbox, Memory, Consent, ToolAuthorityManifest) serialize
through the same pattern:

- On CREATE: `serde_json::to_value(struct)` → strip `id` → `CONTENT $body`.
- On SELECT: `SELECT * OMIT id FROM type::thing(...)` → deserialize →
  inject caller-supplied ID back → return domain struct.

This keeps the adapter's implementation short (≤15 lines per method).

### Rich types translate explicitly

`Grant` and `AuthRequest` need a different shape at the storage layer:

- `Grant.holder: PrincipalRef` flattens to `holder_kind: string` +
  `holder_id: string` (lets Permission Check query by holder index).
- `Grant.resource: ResourceRef` flattens to `resource_uri: string` (same
  reason).
- `AuthRequest.requestor: PrincipalRef` flattens the same way.
- `AuthRequest.state` and `AuthRequest.audit_class` are mapped to/from
  snake_case strings to match the schema ASSERTs.
- `AuthRequest.resource_slots` is stored as a `FLEXIBLE TYPE array<object>`
  so the nested shape (resource, approvers, state) round-trips without
  SurrealDB stripping unknown sub-fields.

The translators live in [`store::repo_impl`](../../../../../../modules/crates/store/src/repo_impl.rs)
as `GrantRow`/`AuthRequestRow` structs + `principal_to_kind_id` helpers.

### Datetimes are RFC3339 strings

SurrealDB's `TYPE datetime` columns do not coerce from the RFC3339 strings
that chrono's default serde produces via `.bind(...)`. To keep the domain
clean (no `surrealdb::sql::Datetime` pollution), the schema uses
`TYPE string` for all datetime columns. ISO-8601 strings order
lexicographically equal to their chronological order, so `ORDER BY
timestamp` (used by `last_event_hash_for_org`) still works.

### Bytes are base64 strings

Same rationale as ADR-0013 / ADR-0014: the driver's `.bind(Vec<u8>)` path
JSON-serializes as an array of numbers which doesn't coerce into
`TYPE bytes`. Base64 strings are portable and human-debuggable. Applies
to `secrets_vault.value_ciphertext_b64`, `secrets_vault.nonce_b64`, and
`audit_events.prev_event_hash_b64`.

## In-memory fake

[`domain::in_memory::InMemoryRepository`](../../../../../../modules/crates/domain/src/in_memory.rs)
is a HashMap-backed `Repository` impl behind a cargo feature
(`in-memory-repo`). Consumers enable it in their `[dev-dependencies]`:

```toml
[dev-dependencies]
domain = { workspace = true, features = ["in-memory-repo"] }
```

It is compiled automatically when running `cargo test -p domain` via
`#[cfg(any(test, feature = "in-memory-repo"))]`. Used by the M0 server
health tests (replacing the per-file mini-fakes) and by every
domain-level unit/proptest in P3+P4.

A `set_unhealthy(bool)` knob flips `ping()` to `Err` without standing up a
broken database — that's how `/healthz/ready` failure scenarios are
tested.

## Testing

Integration tests live in
[`modules/crates/store/tests/repository_test.rs`](../../../../../../modules/crates/store/tests/repository_test.rs)
— **62 tests** spanning every method in the trait (P2 shipped 26; the
P3+ post-audit widening pass added 31 more, P5 added 3 for the
bootstrap-flow atomic batch, and the final post-P9 audit added 2 for
`get_audit_event` roundtrip + miss). Categories:

- Node CRUD round-trips (create + get; get-none; get-admin-before / after).
- Grant CRUD + revoke + list-by-principal, including every
  `PrincipalRef` variant as a holder (incl. negative cases).
- AuthRequest CRUD + full-replace update + list-active-by-resource +
  multi-slot / multi-approver round-trips.
- Typed ownership-edge wrappers (ownership / creation / allocation) +
  edge-id uniqueness under repeated upserts.
- Bootstrap credentials lifecycle (put / find / consume / list) +
  `apply_bootstrap_claim` atomic commit + rollback-on-collision
  (see ADR-0011 §Atomicity).
- Catalogue (seed / contains-hit / contains-miss / per-org isolation +
  case-sensitive lookup).
- Audit (write / last-hash-empty / last-hash-per-org-isolation +
  most-recent ordering by timestamp).

Each test boots a fresh embedded SurrealDB in its own tempdir and drops
it on scope end, so they're independent and parallel-safe.

## Testing counts (P1 → current)

Summary (cargo-test pass counts at each checkpoint). The full
column-by-column breakdown — including domain proptest + worked-trace
+ doctest rows — lives in [overview.md](overview.md) §Testing posture.

| Layer | P1 | P2 | P3+ widening | P5 | Current |
|---|---|---|---|---|---|
| Store unit (crypto + migrations) | 13 | 13 | 13 | 13 | **13** |
| Store integration (migrations + crypto vault + repository) | 2 | 29 | 59 | 62 | **64** |
| **Runnable total** (full workspace) | **46** | **82** | **186** | **260** | **299** |

## Concept references

- ADR: [0015 Type-safe ownership edges](../decisions/0015-type-safe-ownership-edges.md).
- ADR cross-ref: [0009 SurrealDB schema layout](../decisions/0009-surrealdb-schema-layout.md) — amended in P2 to point at 0015.
- Build plan row: C15 / D9 (added in r2 revision after P1 self-review surfaced Risk 1).
- Code: [`repository.rs`](../../../../../../modules/crates/domain/src/repository.rs), [`repo_impl.rs`](../../../../../../modules/crates/store/src/repo_impl.rs), [`in_memory.rs`](../../../../../../modules/crates/domain/src/in_memory.rs), [`principal_resource.rs`](../../../../../../modules/crates/domain/src/model/principal_resource.rs).
