<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Requirements doc — admin page, Phase 2 of fresh-install journey -->

# 03 — Platform MCP Servers

## 2. Page Purpose + Primary Actor

The platform admin registers **MCP server bindings** as `external_service_object` composite instances in the platform-level `resources_catalogue`. Each entry pairs an MCP endpoint (github-mcp, slack-mcp, linear-mcp, etc.) with the credential custodian and the set of tenant orgs permitted to reference it. Tenant orgs reference these entries via cross-org Auth Requests.

**Primary actor:** platform admin Human Agent (holding `[allocate]` on `system:root`).

## 3. Position in the Journey

- **Phase:** 2 of 9 — page 2 of 4.
- **Depends on:** Phase 1 complete; credentials vault (page 04) used for referenced secrets, but the two can be populated in either order — MCP entries can be added with a `secret_ref` that will be populated on the vault page.
- **Enables:** agents that depend on MCP tools (e.g., `mcp_github` from the 14-tool catalogue in [permissions/07 § Tool Authority Manifest Examples](../../concepts/permissions/07-templates-and-tools.md#tool-authority-manifest-examples)).

## 4. UI Sketch

```
┌──────────────────────────────────────────────────────────────────┐
│ Platform > MCP Servers                      [+ Register Server] │
├──────────────────────────────────────────────────────────────────┤
│ ┌──────────────────────────────────────────────────────────────┐ │
│ │ id             │ endpoint             │ status │ tenants     │ │
│ ├──────────────────────────────────────────────────────────────┤ │
│ │ mcp-github     │ mcp://github         │ ● ok   │ * (any)     │ │
│ │ mcp-slack      │ mcp://slack          │ ● ok   │ acme,beta   │ │
│ │ mcp-linear     │ mcp://linear         │ ● ok   │ * (any)     │ │
│ └──────────────────────────────────────────────────────────────┘ │
│                                                                   │
│ Registered: 3    Total active tenant references: 7               │
└──────────────────────────────────────────────────────────────────┘
```

Empty state: "No MCP servers registered. MCP tools in any tenant org will be unavailable until at least one server is registered."

## 5. Read Requirements

- **R-ADMIN-03-R1:** The page SHALL list every `external_service_object` instance in the platform catalogue, showing id, endpoint, provider kind (`mcp`, `openapi`, `webhook`, …), status, and the `tenants_allowed` set.
- **R-ADMIN-03-R2:** The page SHALL display the count of active tenant-org references per entry.

## 6. Write Requirements

- **R-ADMIN-03-W1:** The admin SHALL be able to register a new MCP server entry, supplying: `id`, `endpoint` (URL or `mcp://` form), `kind: external_service`, `secret_ref` (optional; required when the MCP server requires auth), and `tenants_allowed` (a list of org_ids or `'*'`).
- **R-ADMIN-03-W2:** Registration SHALL be expressed as a Template E Auth Request targeting `control_plane_object:platform-catalogue` with `scope: [allocate]`. Auto-approved by the platform admin (owner).
- **R-ADMIN-03-W3:** The admin SHALL be able to modify `tenants_allowed` after registration. Narrowing the set revokes any current tenant-org grants whose org_id no longer appears — a forward-only cascade per [permissions/02 § Retention Policy](../../concepts/permissions/02-auth-request.md#retention-policy).
- **R-ADMIN-03-W4:** The admin SHALL be able to archive an entry. Archival is rejected with `ENTRY_IN_USE` if any tenant currently references it.
- **R-ADMIN-03-W5:** Validation: `id` unique; `endpoint` parseable; `secret_ref` points to a vault entry that exists.

## 7. Permission / Visibility Rules

- **Page access** — `[read, list]` on `control_plane_object:platform-catalogue`. Platform admin only.
- **Register / modify / archive** — `[allocate]` on `control_plane_object:platform-catalogue`. Platform admin only.

## 8. Event & Notification Requirements

- **R-ADMIN-03-N1:** On register (W1), emit audit event `McpServerRegistered { id, endpoint, tenants_allowed, alerted: true }`.
- **R-ADMIN-03-N2:** On tenants_allowed narrow (W3) that revokes tenant grants, the page SHALL show a confirmation dialog listing affected orgs and emit `McpServerTenantAccessRevoked { mcp_id, revoked_orgs, alerted: true }` per revocation.
- **R-ADMIN-03-N3:** On archive (W4), same confirmation + `McpServerArchived`.
- **R-ADMIN-03-N4:** MCP server health is polled; `error` state triggers an alerted `McpServerHealthDegraded` event.

## 9. Backend Actions Triggered

Register (W1) triggers:
- Template E Auth Request + auto-approval.
- Catalogue entry created per [permissions/01 § Resource Catalogue](../../concepts/permissions/01-resource-ontology.md#resource-catalogue).
- Health poller started.
- Audit event.

Tenants-allowed change (W3) may trigger cross-org grant revocations — see [system/s04-auth-request-state-transitions.md](../system/s04-auth-request-state-transitions.md) for the revocation cascade.

## 10. API Contract Sketch

```
GET  /api/v0/platform/mcp-servers
     → 200: { entries: [...], total_active_references }

POST /api/v0/platform/mcp-servers
     Body: { id, endpoint, kind, secret_ref?, tenants_allowed: "*" | [org_id] }
     → 201: { mcp_id, catalogue_auth_request_id }
     → 400 / 409: validation / collision

PATCH /api/v0/platform/mcp-servers/{id}
     Body: { tenants_allowed: [...] }
     → 200: { updated_at, revoked_org_count, audit_event_ids: [...] }

POST /api/v0/platform/mcp-servers/{id}/archive
     → 200 / 409: ENTRY_IN_USE { referencing_orgs }
```

## 11. Acceptance Scenarios

**Scenario 1 — register github MCP.**
*Given* the platform admin has registered the `github-mcp-client-secret` in the vault, *When* they register `mcp-github` with `endpoint: mcp://github`, `secret_ref: github-mcp-client-secret`, `tenants_allowed: '*'`, *Then* the entry appears in the platform catalogue and any tenant org (e.g., [organizations/04-regulated-enterprise.md](../../organizations/04-regulated-enterprise.md)) can subsequently file an Auth Request to reference it.

**Scenario 2 — narrowing tenants revokes outstanding grants.**
*Given* `mcp-slack` is registered with `tenants_allowed: '*'` and currently referenced by three tenant orgs, *When* the admin narrows to `tenants_allowed: [acme, beta]` (dropping the third), *Then* the dropped org's tenant-reference grant is revoked forward-only, a confirmation dialog shows the dropped org before commit, and `McpServerTenantAccessRevoked` is emitted for the affected org.

## 12. Cross-References

**Concept files:**
- [concepts/permissions/01 § Composite Classes](../../concepts/permissions/01-resource-ontology.md#composite-classes-8) — `external_service_object` definition.
- [concepts/permissions/07 § Tool Authority Manifest Examples § 8 `mcp_github`](../../concepts/permissions/07-templates-and-tools.md#8-mcp_github--mcp-adapter-tool-composite-form) — the tool manifest that references entries here.
- [organizations/10-platform-infra.md](../../organizations/10-platform-infra.md) — the `external_services` catalogue section reflects this page's output.

**Related admin pages:**
- [04-platform-credentials-vault.md](04-platform-credentials-vault.md) — where `secret_ref` targets.
- [13-system-agents-config.md](13-system-agents-config.md) — downstream; system agents may depend on registered MCP servers.

**Related system flows:**
- [system/s04-auth-request-state-transitions.md](../system/s04-auth-request-state-transitions.md) — the Auth-Request revocation cascade triggered by W3.

**Org layouts exercised in Acceptance Scenarios:**
- [organizations/04-regulated-enterprise.md](../../organizations/04-regulated-enterprise.md), [organizations/10-platform-infra.md](../../organizations/10-platform-infra.md).
