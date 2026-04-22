<!-- Last verified: 2026-04-21 by Claude Code -->

# Architecture — cascading tenant revocation (page 03)

**Status: [EXISTS]** — shipped with M2/P6.

When an admin narrows an MCP server's `tenants_allowed`, the platform
revokes every grant descending from an Auth Request requested by a
now-excluded org. This is the most contract-dense operation in M2 —
it's the first time a single HTTP write mutates many graph nodes at
once.

## The invariant

Given an MCP server with `tenants_allowed = T_old` and a PATCH to
`T_new ⊂ T_old`, let `D = T_old \ T_new` be the dropped org set.
After the PATCH, every grant G where:

1. `G.descends_from = AR_id` for some AR with
   `AR.requestor = Organization(org)` and `org ∈ D`, AND
2. `G.revoked_at IS NULL` before the PATCH,

must have `G.revoked_at = <patch timestamp>` after. Grants not
matching the filter are untouched.

## The call graph

```
HTTP PATCH /api/v0/platform/mcp-servers/:id/tenants
    │
    └─► handlers::platform_mcp_servers::patch_tenants
          │
          └─► platform::mcp_servers::patch_tenants::patch_mcp_tenants
                │  1. Load current row (404 if unknown)
                │  2. Classify delta:
                │     - Same set → no-op (no repo write, no audit)
                │     - Widening → Repository::patch_mcp_tenants (overwrite)
                │     - Narrowing → Repository::narrow_mcp_tenants (cascade)
                │  3. For narrowing: emit per-AR auth_request.revoked events
                │  4. Emit summary platform.mcp_server.tenant_access_revoked
                ▼
          Response: PatchTenantsResponse { cascade: Vec<…>, audit_event_id }
```

`Repository::narrow_mcp_tenants` runs the DB sweep inside a single
SurrealQL transaction (store impl) or a single mutex-guarded walk
(in-memory impl). Both return the same `Vec<TenantRevocation>` shape
— one entry per `(dropped_org, AR)` pair, carrying the list of
revoked grant ids.

## The classification (`is_narrowing`)

Computed both server-side (Rust) and client-side (TypeScript in the
`PatchTenantsDialog`). The two implementations must agree on every
combination; the web test suite pins this explicitly.

| `T_old` | `T_new` | classification |
|---|---|---|
| `All` | `All` | same set (no-op) |
| `All` | `Only(ids)` | narrowing (but M2 returns empty cascade; see note) |
| `Only(a)` | `All` | widening |
| `Only(a)` | `Only(b)` | narrowing iff `a ⊄ b` |

### Note on `All → Only`

Semantically this IS narrowing — orgs that were implicitly allowed
are now excluded. But `narrow_mcp_tenants` can't enumerate "every
org" without a platform-wide org index. M2 returns an empty cascade
for this transition; M3 will wire the full enumeration once the
platform org index becomes load-bearing.

The Template-E AR is still created, and the handler still emits the
summary event (with empty `revoked_orgs`), so the audit trail
captures the intent even though no grants flipped.

## The audit trail

Three event types per narrowing PATCH:

1. **Template-E Auth Request** — self-approved platform-admin write.
   Every non-bootstrap admin write in M2 opens one of these; it's the
   provenance root for the cascade events.
2. **`auth_request.revoked`** — **one per affected AR**. Carries:
   - `auth_request_id` — the AR being revoked (sets `target_entity_id`).
   - `org` — the dropped org that triggered the revocation.
   - `revoked_grants` — explicit list of grant ids.
   - `reason: "mcp_tenant_narrow"` — so M3+ delegated-revocation
     cascades can reuse the same event type.
   - `mcp_server_id` — for cross-navigation.
3. **`platform.mcp_server.tenant_access_revoked`** (summary) — one
   per PATCH. Carries the full lists (`revoked_orgs`,
   `affected_auth_requests`) + counts.

The per-AR events are emitted **before** the summary so a reader
walking the chain forward sees the individual revocations grouped
under the summary event.

## The forward-only contract

Revocation is not reversible. Even if the admin immediately widens
the set back, the revoked grants stay revoked — the widening PATCH
creates a new Template-E AR but does **not** re-issue grants (that's
M3+ work when delegated grants become common).

This is intentional: the audit reviewer must see a clean "X
grants revoked at T1; Y new grants issued at T2" trail rather than a
confusing "grants un-revoked" transition that doesn't match any
permission-system state machine.

## phi-core leverage

- `TenantSet` (the enum being narrowed) is **phi-only**.
  phi-core has no tenancy model; tenancy is a platform-governance
  concern layered on top.
- The live `McpClient` is never consulted during the cascade — the
  narrow walks graph state only. `McpClient` is reused from phi-core
  purely for the health-probe path (shape-only in M2; real probe in
  M7b — see [`../operations/mcp-server-operations.md`](../operations/mcp-server-operations.md) §4).

## See also

- [`../operations/mcp-server-operations.md`](../operations/mcp-server-operations.md) — the day-to-day runbook,
  including the emergency over-narrow playbook.
- [`../user-guide/mcp-servers-usage.md`](../user-guide/mcp-servers-usage.md) — operator-facing
  walkthrough (CLI + web).
- Repository trait: [`modules/crates/domain/src/repository.rs`](../../../../../../modules/crates/domain/src/repository.rs)
  — `narrow_mcp_tenants` signature + docstring.
- Business logic: [`modules/crates/server/src/platform/mcp_servers/patch_tenants.rs`](../../../../../../modules/crates/server/src/platform/mcp_servers/patch_tenants.rs).
- Proptests: [`modules/crates/domain/tests/mcp_cascade_props.rs`](../../../../../../modules/crates/domain/tests/mcp_cascade_props.rs)
  — monotonic grant count, narrow idempotence, no over-revocation.
