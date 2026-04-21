<!-- Last verified: 2026-04-21 by Claude Code -->

# User guide — MCP Servers

**Status: [EXISTS]** — page 03 shipped with M2/P6.

The MCP Servers page binds external services (phi-core MCP clients,
plus reserved slots for OpenAPI and webhook kinds) to the platform so
orgs can invoke them. It also owns the **tenant-narrowing cascade**:
when an admin shrinks `tenants_allowed`, every grant descending from
an Auth Request requested by a now-excluded org is revoked atomically
and the cascade is audited.

## Overview

- Surface: **Web** `/mcp-servers` + **CLI** `baby-phi mcp-server …` +
  HTTP `/api/v0/platform/mcp-servers`.
- Auth: the same session cookie used for every other admin page.
- Persisted composite: `ExternalService` (baby-phi-only — phi-core has
  no equivalent container).
- Live client: constructed **on demand** at probe/invocation time
  from the stored `endpoint` string via phi-core's
  `McpClient::connect_stdio` / `connect_http`
  (see `phi-core/src/mcp/client.rs` in the sibling submodule).
  Never stored.

## Fields

| Field | What it holds |
|---|---|
| `display_name` | Operator-visible label (e.g. `memory-mcp`). |
| `kind` | `mcp` (only kind wired in M2). `open_api`, `webhook`, `other` reserved. |
| `endpoint` | phi-core's transport argument **verbatim** — `stdio:///cmd args…` or `http[s]://…`. |
| `secret_ref` | Optional vault slug holding auth material. Omit for anonymous services. |
| `tenants_allowed` | `{"mode": "all"}` or `{"mode": "only", "orgs": [...]}`. |
| `status` | `ok` / `probing` / `degraded` / `error` / `archived`. M2 only sets `ok` and `archived`. |

## Web surface

Navigation: sign in as a claimed admin → sidebar → *MCP Servers*. The
page has two sections:

1. **Existing servers table** — one row per registered server. Columns:
   display name, kind, endpoint, tenants summary, secret_ref, status,
   actions (`Patch tenants` + `Archive`).
2. **Register form** — display name, kind dropdown, endpoint, optional
   `secret_ref`. Submission calls `registerServerAction` which
   forwards the session cookie and POSTs
   `/api/v0/platform/mcp-servers`.

### PatchTenantsDialog

Click *Patch tenants* on any row. The dialog:

1. Pre-populates the input with the server's current tenants spec.
2. Runs a client-side `isNarrowing` check (mirrors the Rust
   [`is_narrowing`](../../../../../../modules/crates/server/src/platform/mcp_servers/patch_tenants.rs) exactly) and shows
   an **amber warning banner** when the PATCH will trigger the
   cascade.
3. Submits via `patchTenantsAction` and renders the blast-radius
   summary: "revoked N grants across M Auth Requests covering K
   orgs".

Narrowing is cascade-irreversible — see
[`operations/mcp-server-operations.md`](../operations/mcp-server-operations.md)
§2 for the forensic audit trail and the emergency over-narrow
playbook.

## CLI surface

Four subcommands under `baby-phi mcp-server`:

| Command | Effect |
|---|---|
| `baby-phi mcp-server list [--include-archived] [--json]` | Read the catalogue. |
| `baby-phi mcp-server add --display-name <NAME> --endpoint <ENDPOINT> [--kind mcp] [--secret-ref <slug>] [--tenants-allowed all\|uuid1,uuid2]` | Register a server. |
| `baby-phi mcp-server patch-tenants --id <uuid> --tenants-allowed all\|uuid1,uuid2 --confirm-cascade` | Update `tenants_allowed`. `--confirm-cascade` is **required**. |
| `baby-phi mcp-server archive --id <uuid>` | Soft-delete. |

All subcommands re-use the same session cookie loaded from
`$XDG_CONFIG_HOME/baby-phi/session` (written by
`baby-phi bootstrap claim`).

**Exit codes** (see [`cli/src/exit.rs`](../../../../../../modules/crates/cli/src/exit.rs)):

- `0` — success.
- `1` — transport / IO (server unreachable, DNS, timeout).
- `2` — server rejected with a stable 4xx code (bad input, missing
  secret, etc.).
- `3` — internal / 5xx or unexpected shape.
- `4` — precondition failed (no saved session).
- `5` — cascade aborted (`patch-tenants` without
  `--confirm-cascade`).

## HTTP surface

Four routes under `/api/v0/platform/mcp-servers`, all gated by the
`AuthenticatedSession` extractor from `handler_support`:

| Method  | Path | Op |
|---|---|---|
| `GET`   | `/api/v0/platform/mcp-servers` | list (`?include_archived=true` to include soft-deleted) |
| `POST`  | `/api/v0/platform/mcp-servers` | register |
| `PATCH` | `/api/v0/platform/mcp-servers/:id/tenants` | update `tenants_allowed` (cascades on narrow) |
| `POST`  | `/api/v0/platform/mcp-servers/:id/archive` | soft-delete |

### Wire shapes

```jsonc
// POST /api/v0/platform/mcp-servers
{
  "display_name": "memory-mcp",
  "kind": "mcp",
  "endpoint": "stdio:///usr/local/bin/memory-mcp",
  "secret_ref": "mcp-memory-key",   // optional
  "tenants_allowed": { "mode": "all" }
}

// → 201
{
  "mcp_server_id": "<uuid>",
  "auth_request_id": "<uuid>",
  "audit_event_id": "<uuid>"
}
```

```jsonc
// PATCH /api/v0/platform/mcp-servers/:id/tenants
{
  "tenants_allowed": {
    "mode": "only",
    "orgs": ["<org-a>", "<org-b>"]
  }
}

// → 200
{
  "mcp_server_id": "<uuid>",
  "cascade": [
    {
      "org": "<dropped-org>",
      "auth_request": "<ar-uuid>",
      "revoked_grants": ["<grant-1>", "<grant-2>"]
    }
  ],
  "audit_event_id": "<uuid>"    // null when no cascade ran
}
```

### Error codes

| Code | Status | Meaning |
|---|---|---|
| `UNAUTHENTICATED` | 401 | Session cookie missing or expired. |
| `VALIDATION_FAILED` | 400 | Required field empty / malformed UUID. |
| `SECRET_REF_NOT_FOUND` | 400 | `secret_ref` does not exist in the vault. |
| `MCP_SERVER_NOT_FOUND` | 404 | `id` does not match any row. |
| `AUDIT_EMIT_FAILED` | 500 | Audit emitter returned an error — the underlying write MAY have succeeded. |
| `INTERNAL_ERROR` | 500 | Repository error — see server logs. |

## phi-core leverage

See the architecture map
[`../architecture/phi-core-reuse-map.md`](../architecture/phi-core-reuse-map.md)
§Page 03 for the full table. Highlights:

- `McpClient::connect_stdio`, `connect_http`, `list_tools()` — all
  reused verbatim. No parallel baby-phi MCP client.
- `McpToolInfo`, `ServerInfo` — phi-core's single source of truth;
  M7b's scheduled probe uses them directly.
- The only baby-phi-native MCP code is the thin timeout+retry
  wrapper in
  [`platform/mcp_servers/health_probe.rs`](../../../../../../modules/crates/server/src/platform/mcp_servers/health_probe.rs),
  because phi-core has no probe abstraction.
