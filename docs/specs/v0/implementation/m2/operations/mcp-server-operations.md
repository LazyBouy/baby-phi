<!-- Last verified: 2026-04-21 by Claude Code -->

# Operations — MCP Servers

**Status: [EXISTS]** — page 03 shipped with M2/P6.

This runbook is for the platform admin who operates the
`/api/v0/platform/mcp-servers` surface day-to-day. It covers:

- Registering a new MCP / external service (§1).
- Narrowing `tenants_allowed` and the cascade audit trail (§2).
- Archiving a server (§3).
- Health-probe incidents — shape-only in M2; real probe lands in M7b (§4).
- Emergency playbook: accidental over-narrow (§5).

All events described here are **Alerted**-tier and chain into the
`audit_events` hash-chain — the same machinery M2 uses for vault and
model-provider writes. See
[`../../m1/architecture/audit-events.md`](../../m1/architecture/audit-events.md)
for the envelope.

## 1. Registering a server

Two surfaces:

- **CLI**:
  `baby-phi mcp-server add --display-name <NAME> --endpoint <ENDPOINT> [--secret-ref <slug>] [--tenants-allowed all|uuid1,uuid2]`.
- **Web**: `/mcp-servers` → *Register an MCP server*.

`--endpoint` is phi-core's `McpClient` transport argument verbatim:

- `stdio:///path/to/server [arg1 arg2 …]` — for local processes.
- `http://…` / `https://…` — for remote servers.

**Referential integrity:** when `--secret-ref` is set, the handler
rejects with `SECRET_REF_NOT_FOUND` (400) if the slug does not already
exist in the vault. Add the secret first (`baby-phi secret add --slug
… --material-file …`) or drop the flag for unauthenticated services.

**Audit event:** `platform.mcp_server.registered` (Alerted). Diff
carries the full after-snapshot: `display_name`, `kind`, `endpoint`,
`secret_ref`, `tenants_allowed`, `status`, timestamps.

**Grant shape:** a per-instance grant is issued on
`external_service:<id>` with `fundamentals = [NetworkEndpoint,
SecretCredential, Tag]` — matches `Composite::ExternalServiceObject`.
The engine's Case D (P4.5 detour) picks this up with the URI-scoped
selector at invocation time.

## 2. Narrowing `tenants_allowed` — the cascade path

This is the contract-dense operation on page 03.

**Surfaces:**

- CLI:
  `baby-phi mcp-server patch-tenants --id <uuid> --tenants-allowed all|uuid1,uuid2 --confirm-cascade`.
  The `--confirm-cascade` flag is **required** — without it, the CLI
  aborts with `EXIT_CASCADE_ABORTED` (5).
- Web: Per-row *Patch tenants* → `PatchTenantsDialog`. The dialog runs
  an in-browser `is_narrowing` check (mirrors the Rust one exactly)
  and shows a prominent amber banner when narrowing is detected.

**Semantics:**

| Transition | Cascade | Audit events |
|---|---|---|
| same set | none (no-op — no repo write) | none |
| `Only(a) → Only(superset of a)` | none (widen) | new Template-E AR only |
| `Only(a) → Only(subset of a)` | revoke every grant descending from an AR whose requestor is a now-excluded org | 1× `platform.mcp_server.tenant_access_revoked` + Nx `auth_request.revoked` |
| `Only(a) → All` | none (widen) | new Template-E AR only |
| `All → Only(subset)` | **none in M2** (M2 cannot enumerate "every org"); M3 wires the full enumeration path via the platform org index | new Template-E AR — the "audit-logged Template-E" signature is still present even though no cascade grants are affected |

**Summary event diff shape** (`platform.mcp_server.tenant_access_revoked`):

```json
{
  "before": {
    "mcp_server_id": "<uuid>",
    "tenants_allowed": { "mode": "only", "orgs": ["<a>", "<b>"] }
  },
  "after": {
    "mcp_server_id": "<uuid>",
    "tenants_allowed": { "mode": "only", "orgs": ["<b>"] },
    "revoked_org_count": 1,
    "revoked_auth_request_count": <N>,
    "revoked_grant_count": <M>,
    "revoked_orgs": ["<a>"],
    "affected_auth_requests": ["<ar-1>", "<ar-2>", "..."]
  }
}
```

Per-AR event (`auth_request.revoked`) carries the individual grant
ids and a `"reason": "mcp_tenant_narrow"` tag so the M3+ delegated
revocation cascade can reuse the same event type.

**Forensics.** Every cascade is forward-only — grants flip from
`revoked_at = NULL` to `revoked_at = <cascade timestamp>`. There is no
replay path in M2; M7b introduces tooling that reconstructs the
pre-cascade state from the hash-chain.

## 3. Archiving a server

- CLI: `baby-phi mcp-server archive --id <uuid>`.
- Web: Per-row *Archive*.

**Effects:**

1. `archived_at` is set to the archive timestamp.
2. `status` flips to `Archived`.
3. An Alerted `platform.mcp_server.archived` event is emitted with
   the **pre-archive** snapshot in `before`.

Archival does **not** cascade-revoke grants on the row in M2 (the
single admin holds them; no multi-principal exposure). M3 wires the
cascade once delegated grants become common.

List responses filter archived rows by default; pass
`?include_archived=true` (CLI: `--include-archived`) to see them.

## 4. Health-probe incidents

**M2 scope:** the `platform.mcp_server.health_degraded` event
**builder** ships, but no scheduled probe runs. The
`probe_mcp_health(endpoint)` helper in
[`platform/mcp_servers/health_probe.rs`](../../../../../../modules/crates/server/src/platform/mcp_servers/health_probe.rs)
is unit-tested but not wired into production; M7b's scheduled probe
will call it.

When M7b lands, incident triage follows this sequence:

1. `health_degraded` event published → on-call receives a page.
2. Read the `reason` field — `connect timed out`, `list_tools
   failed: …`, or `unsupported scheme`.
3. Verify the server is actually down (`baby-phi mcp-server list` +
   manual probe via `baby-phi mcp-server tools <id>` — the M7b tools
   command surfaces the phi-core `list_tools()` response).
4. Either restart the underlying service, narrow
   `tenants_allowed` to quarantine until the owner responds, or
   archive the row outright.

## 5. Emergency playbook — accidental over-narrow

Scenario: an admin narrows `tenants_allowed` and the cascade summary
reveals more revoked grants than expected (e.g. because a staging
org's grants also descended from the dropped-org's ARs).

**Triage:**

1. **Stop the bleed.** Widen the `tenants_allowed` set immediately —
   the widening PATCH runs through Template-E but **does not
   re-issue the revoked grants** (revocation is forward-only). The
   widen restores *future* invocations, not historical grants.
2. **Inventory the damage.** Read the
   `platform.mcp_server.tenant_access_revoked` summary; its
   `affected_auth_requests` field is the canonical list.
3. **Re-issue grants.** For each affected AR, the original
   requestor must submit a fresh AR (M3's delegation flow is the
   long-term fix; in M2 the platform admin can construct a
   replacement Template-E AR manually).
4. **Post-incident.** File an entry in the runbook (`docs/ops/
   runbook.md` §M2 incidents) describing the trigger + recovery so
   future operators recognise the pattern.

## 6. phi-core leverage reminder

- `ExternalService` is **baby-phi-only** (phi-core has no persisted
  MCP-binding container).
- The live client is constructed on demand from the stored
  `endpoint` string via phi-core's `McpClient::connect_stdio` /
  `connect_http` at probe/invocation time — **never stored**.
- `McpToolInfo`, `ServerInfo`, `list_tools()` surfaces are phi-core's
  single source of truth; M7b's scheduled probe uses them directly.
- Health-probe timeout+retry is the only baby-phi-native MCP code
  (phi-core has no probe abstraction). The 2 s per-attempt timeout
  matches plan §P6.
