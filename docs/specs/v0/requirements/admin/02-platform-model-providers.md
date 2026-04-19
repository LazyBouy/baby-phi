<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Requirements doc — admin page, Phase 2 of fresh-install journey -->

# 02 — Platform Model Providers

## 2. Page Purpose + Primary Actor

The platform admin registers **LLM model provider bindings** as `model/runtime_object` composite instances in the platform-level `resources_catalogue`. Each entry pairs a provider (Anthropic, OpenAI, a local runtime, etc.) with a named default model the provider exposes. Registered models become available for tenant orgs to reference via cross-org grants.

**Primary actor:** Human Agent holding `[allocate]` on `system:root` — the platform admin claimed in Phase 1.

## 3. Position in the Journey

- **Phase:** 2 of 9 (Platform-level resource setup) — page 1 of 4.
- **Depends on:** Phase 1 complete (platform admin exists).
- **Enables:** orgs created in Phase 3 can reference these models; agents in Phase 5 bind to them via `ModelConfig`.

## 4. UI Sketch

```
┌─────────────────────────────────────────────────────────────────┐
│ Platform > Model Providers                     [+ Add Provider] │
├─────────────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ id                         │ provider  │ status │ [actions] │ │
│ ├─────────────────────────────────────────────────────────────┤ │
│ │ claude-sonnet-default      │ anthropic │ ●  ok  │ [⋯]       │ │
│ │ claude-opus-default        │ anthropic │ ●  ok  │ [⋯]       │ │
│ │ gpt-4o-default             │ openai    │ ●  ok  │ [⋯]       │ │
│ │ local-llama-3-8b           │ local     │ ○ idle │ [⋯]       │ │
│ └─────────────────────────────────────────────────────────────┘ │
│                                                                  │
│ Registered: 4    Usable by tenant orgs: 4                       │
└─────────────────────────────────────────────────────────────────┘
```

Empty state: "No models registered yet. [+ Add First Provider]" — models are the prerequisite for any LLM Agent to function.
Error state: inline toast "Failed to list providers — retry."

## 5. Read Requirements

- **R-ADMIN-02-R1:** The page SHALL list every `model/runtime_object` instance in the platform's `resources_catalogue`, showing id, provider, default model, and current status (ok / idle / error).
- **R-ADMIN-02-R2:** The page SHALL display usage counts per provider (number of tenant orgs currently referencing this entry).

## 6. Write Requirements

- **R-ADMIN-02-W1:** The admin SHALL be able to add a new model provider entry, supplying: `id`, `provider`, `model`, `tenants_allowed` (default `'*'`), and pointer to the secret id (registered via [04-platform-credentials-vault.md](04-platform-credentials-vault.md)) that the provider requires for authentication.
- **R-ADMIN-02-W2:** Adding a new entry SHALL be expressed as an Auth Request of type Template E targeting `control_plane_object:platform-catalogue` with `scope: [allocate]`. The platform admin auto-approves (owner). On approval, the `model/runtime_object` instance is added to the platform catalogue atomically.
- **R-ADMIN-02-W3:** The admin SHALL be able to archive an entry via the row's "⋯" menu. Archival SHALL be rejected with error `ENTRY_IN_USE` if any tenant org currently references the entry; the error message lists the referencing orgs and their grants so the admin can revoke those first.
- **R-ADMIN-02-W4:** Validation: `id` SHALL be unique across the catalogue; `provider` SHALL be one of the registered providers; the referenced secret SHALL exist.

## 7. Permission / Visibility Rules

All rules resolve to grants held by the Human Agent viewing or acting on the page.

- **Page access** — requires `[read, list]` on `control_plane_object:platform-catalogue`. Held by the platform admin via their `[allocate]` on `system:root` (sub-grants: `[read, list, allocate]` on catalogue entries).
- **Add entry** — requires `[allocate]` on `control_plane_object:platform-catalogue`. Platform admin only.
- **Archive entry** — same `[allocate]`.

## 8. Event & Notification Requirements

- **R-ADMIN-02-N1:** On successful add (W1), the page SHALL display toast "Provider `<id>` registered; `<n>` tenant orgs may reference it" and emit audit event `ModelProviderRegistered { id, provider, secret_ref, registered_by, alerted: true }`.
- **R-ADMIN-02-N2:** On archive (W3), the page SHALL require a confirmation dialog listing referencing orgs and emit audit event `ModelProviderArchived { id, archived_by, alerted: true }`.
- **R-ADMIN-02-N3:** The status column SHALL update live — failed health checks change the status to `error` and trigger an alerted audit event `ModelProviderHealthDegraded { id, reason }`.

## 9. Backend Actions Triggered

Adding a provider (W1) triggers:
- Auth Request submitted + auto-approved by platform admin.
- `model/runtime_object` instance added to platform `resources_catalogue` (see [permissions/01 § Resource Catalogue](../../concepts/permissions/01-resource-ontology.md#resource-catalogue)).
- Health-check poller started against the provider endpoint.
- Audit event `ModelProviderRegistered` emitted.

Archiving (W3) triggers:
- Dependency check — list referencing orgs.
- On confirmation, catalogue entry marked `archived`; health poller stopped.
- Audit event `ModelProviderArchived` emitted.

## 10. API Contract Sketch

```
GET  /api/v0/platform/model-providers
     → 200: { entries: [...], summary: {...} }

POST /api/v0/platform/model-providers
     Body: { id, provider: "anthropic"|"openai"|"local", model, secret_ref, tenants_allowed }
     → 201: { model_runtime_id, catalogue_auth_request_id, audit_event_id }
     → 400: Validation errors
     → 409: id collision / secret missing

POST /api/v0/platform/model-providers/{id}/archive
     → 200: { archived_at, audit_event_id }
     → 409: ENTRY_IN_USE { referencing_orgs: [...] }
```

## 11. Acceptance Scenarios

**Scenario 1 — register Anthropic default.**
*Given* the platform admin has completed Phase 1 and holds `[allocate]` on `system:root`, and the `anthropic-api-key` secret exists in the vault, *When* they register a `claude-sonnet-default` entry, *Then* the entry appears in the platform catalogue, is available for reference by any subsequently-created org (mirroring the `model_runtime_objects` list in [organizations/10-platform-infra.md](../../organizations/10-platform-infra.md)), and `ModelProviderRegistered` (alerted) is audit-logged.

**Scenario 2 — cannot archive referenced entry.**
*Given* `claude-sonnet-default` is referenced by three tenant orgs (e.g., `minimal-startup`, `mid-product-team`, `consultancy-strict`), *When* the platform admin attempts to archive it, *Then* the action is rejected with `ENTRY_IN_USE` and the three referencing orgs listed. The admin must first revoke each tenant's grant to this entry before archival.

## 12. Cross-References

**Concept files:**
- [concepts/permissions/01 § Resource Catalogue](../../concepts/permissions/01-resource-ontology.md#resource-catalogue) — the precondition rule; entries here populate the catalogue.
- [concepts/permissions/01 § Composite Classes](../../concepts/permissions/01-resource-ontology.md#composite-classes-8) — `model/runtime_object` definition.
- [concepts/organization.md](../../concepts/organization.md) — how orgs reference platform resources.

**Related admin pages:**
- [01-platform-bootstrap-claim.md](01-platform-bootstrap-claim.md) — the prerequisite page.
- [04-platform-credentials-vault.md](04-platform-credentials-vault.md) — where the `secret_ref` points to.
- [09-agent-profile-editor.md](09-agent-profile-editor.md) — where an Agent's `ModelConfig.id` field selects from this catalogue.

**Org layouts exercised in Acceptance Scenarios:**
- [organizations/10-platform-infra.md](../../organizations/10-platform-infra.md) — the platform-infra layout's `model_runtime_objects` catalogue section is exactly what this page populates.
