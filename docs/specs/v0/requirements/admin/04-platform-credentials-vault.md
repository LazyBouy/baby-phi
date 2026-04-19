<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Requirements doc — admin page, Phase 2 of fresh-install journey -->

# 04 — Platform Credentials Vault

## 2. Page Purpose + Primary Actor

The platform admin stores and manages `secret/credential` entries that model providers (page 02) and MCP servers (page 03) reference by id. Each entry has a designated **custodian** — the Human Agent responsible for rotating that specific credential. The vault never displays secret material to anyone other than the custodian (and even then, read access goes through a confirmation flow with an alerted audit event).

**Primary actor:** platform admin Human Agent. May delegate custody of individual entries to other Human Agents via a Template E Auth Request.

## 3. Position in the Journey

- **Phase:** 2 of 9 — page 3 of 4.
- **Depends on:** Phase 1 complete.
- **Enables:** [02-platform-model-providers.md](02-platform-model-providers.md) and [03-platform-mcp-servers.md](03-platform-mcp-servers.md) to reference vault entries by id.

## 4. UI Sketch

```
┌──────────────────────────────────────────────────────────────────┐
│ Platform > Credentials Vault                    [+ Add Secret]  │
├──────────────────────────────────────────────────────────────────┤
│ ┌──────────────────────────────────────────────────────────────┐ │
│ │ id                         │ custodian       │ last rotated  │ │
│ ├──────────────────────────────────────────────────────────────┤ │
│ │ anthropic-api-key          │ agent:alex       │ 2026-04-01   │ │
│ │ openai-api-key             │ agent:alex       │ 2026-04-01   │ │
│ │ github-mcp-client-secret   │ agent:bob        │ 2026-03-15   │ │
│ │ slack-mcp-bot-token        │ agent:alex       │ 2026-04-10   │ │
│ └──────────────────────────────────────────────────────────────┘ │
│                                                                   │
│ Rotation due soon: 1 (github-mcp-client-secret)                  │
│ [#sensitive tags applied by default]                             │
└──────────────────────────────────────────────────────────────────┘
```

Empty state: "No secrets registered. MCP servers and model providers requiring auth will be unable to connect."
Error state: toast "Failed to contact vault backend — retry."

**Secret material is never displayed on the list view.** A "Reveal" action on a row requires the requesting Human Agent to be the designated custodian AND to complete a confirmation flow; the reveal itself emits an alerted audit event.

## 5. Read Requirements

- **R-ADMIN-04-R1:** The page SHALL list every `secret/credential` entry in the platform catalogue, showing id, custodian agent id, last rotated timestamp, and rotation-due flag. Secret material SHALL NOT be displayed.
- **R-ADMIN-04-R2:** The page SHALL show a counter of secrets with rotation overdue (configurable threshold; default 90 days).
- **R-ADMIN-04-R3:** Each entry SHALL auto-carry the `#sensitive` tag in its catalogue tag set; the page SHALL indicate this visually.

## 6. Write Requirements

- **R-ADMIN-04-W1:** The admin SHALL be able to add a new secret entry, supplying: `id`, `custodian` (agent_id), and the secret material (entered in a masked field, never echoed). The material is stored encrypted; only the custodian's `[read]` grant + confirmation unlocks it.
- **R-ADMIN-04-W2:** The admin SHALL be able to rotate a secret (replace its material with a new value), resetting the last-rotated timestamp.
- **R-ADMIN-04-W3:** The admin SHALL be able to reassign custody via a Template E Auth Request. The new custodian must approve acceptance (inbound Auth Request on their side).
- **R-ADMIN-04-W4:** The custodian of a specific entry (not necessarily the platform admin) SHALL be able to invoke "Reveal" to view the secret material. Reveal requires a confirmation dialog and emits an alerted audit event.
- **R-ADMIN-04-W5:** The admin SHALL be able to archive an entry. Archival rejected with `ENTRY_IN_USE` if referenced by any model provider or MCP server entry.
- **R-ADMIN-04-W6:** Validation: `id` unique; `custodian` must be an existing Human Agent or designated service principal; secret material must be non-empty and under size limit.

## 7. Permission / Visibility Rules

- **Page access (list view)** — `[read, list]` on `secret/credential` entries scoped to the platform catalogue. Platform admin has it; designated custodians have it only for their own assigned entries.
- **Add / archive / rotate entry** — `[allocate]` on `control_plane_object:platform-catalogue`. Platform admin only.
- **Reveal secret material** — requires the requester to be the custodian AND to hold `[read]` on that specific `secret/credential` entry with `purpose: reveal` constraint. Reveal is intentionally a privileged, audit-visible action.
- **Reassign custody** — `[allocate]` on the specific entry. Platform admin by default; delegated custodians may propose but require platform-admin approval.

## 8. Event & Notification Requirements

- **R-ADMIN-04-N1:** Every add / rotate / archive / reassign emits an alerted audit event (`SecretAdded`, `SecretRotated`, `SecretArchived`, `SecretCustodyReassigned`). `#sensitive` resources always emit alerted events for writes regardless of the org's default.
- **R-ADMIN-04-N2:** A Reveal invocation emits alerted audit event `SecretRevealed { secret_id, revealed_to, session_id, purpose }`.
- **R-ADMIN-04-N3:** Entries approaching rotation-due threshold SHALL produce a banner; the platform admin's inbox SHALL receive an `AgentMessage` 30 days, 7 days, and 1 day before due.

## 9. Backend Actions Triggered

Add / rotate / archive (W1/W2/W5) triggers:
- Template E Auth Request + auto-approval (or custodian approval for reassignment).
- Catalogue entry update.
- Alerted audit event.
- Update to downstream references' health if the secret rotation affects their connection (model providers, MCP servers re-authenticate).

Reveal (W4) triggers:
- Custodian identity + grant check.
- Cryptographic unwrap of the stored material.
- Alerted audit event emitted BEFORE the material is returned to the requester's UI.

## 10. API Contract Sketch

```
GET  /api/v0/platform/secrets
     → 200: { entries: [{id, custodian, last_rotated_at, rotation_due, sensitive: true}] }
       (Note: no secret material in response.)

POST /api/v0/platform/secrets
     Body: { id, custodian, material: String (masked at edge) }
     → 201: { secret_id, audit_event_id }
     → 409: id collision

POST /api/v0/platform/secrets/{id}/rotate
     Body: { material: String }
     → 200: { rotated_at, audit_event_id }

POST /api/v0/platform/secrets/{id}/reveal
     Body: { confirm: true, purpose: String }
     → 200: { material: String, audit_event_id }  (alerted event fired pre-return)
     → 403: Not custodian / missing grant

POST /api/v0/platform/secrets/{id}/reassign-custody
     Body: { new_custodian: agent_id }
     → 202: { pending_auth_request_id }   (target must approve)
```

## 11. Acceptance Scenarios

**Scenario 1 — add an API key.**
*Given* the platform admin is on this page and no `anthropic-api-key` exists, *When* they submit id `anthropic-api-key`, custodian `agent:alex` (themselves), and the actual key material, *Then* the entry is added with `#sensitive` tag, `SecretAdded` (alerted) is emitted, and [02-platform-model-providers.md](02-platform-model-providers.md) can now reference `anthropic-api-key` in the `secret_ref` field of a new provider entry.

**Scenario 2 — non-custodian attempts reveal.**
*Given* `github-mcp-client-secret` has custodian `agent:bob`, *When* a different Human Agent `agent:alex` (even the platform admin) invokes Reveal on this entry, *Then* the request is rejected with `NOT_CUSTODIAN`, no material is returned, and an alerted `SecretRevealAttemptDenied` event is logged.

**Scenario 3 — archive-blocked by reference.**
*Given* `anthropic-api-key` is referenced by the `claude-sonnet-default` model provider, *When* the admin attempts to archive it, *Then* archival is rejected with `ENTRY_IN_USE`. The admin must first either reassign the provider's `secret_ref` or archive the provider entry.

## 12. Cross-References

**Concept files:**
- [concepts/permissions/01 § Resource Ontology — Two Tiers](../../concepts/permissions/01-resource-ontology.md#fundamental-classes-9) — `secret/credential` as a fundamental.
- [concepts/permissions/01 § Resource Catalogue](../../concepts/permissions/01-resource-ontology.md#resource-catalogue) — vault entries populate the catalogue.
- [concepts/permissions/07 § Tool Authority Manifest Examples § 7 `load_env`](../../concepts/permissions/07-templates-and-tools.md#7-load_env--secret-touching-tool) — a secret-touching tool's manifest shape.

**Related admin pages:**
- [02-platform-model-providers.md](02-platform-model-providers.md), [03-platform-mcp-servers.md](03-platform-mcp-servers.md) — consumers of vault entries.

**Org layouts exercised in Acceptance Scenarios:**
- [organizations/10-platform-infra.md](../../organizations/10-platform-infra.md) — the `secrets` section of the platform-infra layout is exactly what this page populates.
