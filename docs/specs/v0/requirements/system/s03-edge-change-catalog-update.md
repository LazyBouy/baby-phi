<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Requirements doc — system flow -->

# s03 — Edge-Change Catalog Update

## Purpose

The `agent-catalog-agent` (second standard System Agent) observes graph-edge changes on Agent-related edges and maintains an up-to-date, queryable catalogue of all active Agents in each organisation. Other agents consult the catalogue for roster information without having to traverse the graph themselves.

## Trigger

Graph mutation events on any of: `MEMBER_OF`, `HAS_AGENT`, `HAS_LEAD`, `HAS_PROFILE`, `HAS_SUBORGANIZATION`, and Agent node creation/archival.

## Preconditions

- `agent-catalog-agent` is provisioned (auto-added by [admin/06-org-creation-wizard.md](../admin/06-org-creation-wizard.md)) and in `running` state for the affected org.
- The agent holds the three grants defined in [concepts/system-agents.md § Agent Catalog Agent § Grants Held](../../concepts/system-agents.md#grants-held-by-the-agent-1).

## Behaviour

- **R-SYS-s03-1:** The flow SHALL subscribe to the event stream for the enumerated edge types and react within 5 seconds of each event (soft SLO; contributes to catalog-agent status "lag" vs "syncing" — see [admin/08-agent-roster-list.md R7](../admin/08-agent-roster-list.md#5-read-requirements)).
- **R-SYS-s03-2:** On Agent creation, the flow SHALL add the agent to the catalogue with: `agent_id`, `kind`, `profile_ref`, `parallelize`, `base_organization`, `current_project`, `MEMBER_OF` org_id, `created_at`.
- **R-SYS-s03-3:** On Agent archival, the flow SHALL mark the catalogue entry `active: false`. The entry remains for historical queries.
- **R-SYS-s03-4:** On `HAS_LEAD` / `HAS_AGENT` edge changes, the flow SHALL update the catalogue's role-index mapping: `{org_id, project_id, role} → [agent_id]`.
- **R-SYS-s03-5:** On `HAS_PROFILE` changes (profile ref swap), the flow SHALL update the cached profile snapshot on the catalogue entry.
- **R-SYS-s03-6:** The flow SHALL expose a query surface via the `query_agents` tool whose manifest is defined in [concepts/system-agents.md § Queryable Interface](../../concepts/system-agents.md#queryable-interface-example). Query results are read-only snapshots filtered by the caller's grants.
- **R-SYS-s03-7:** Updates to the catalogue are atomic per event. Concurrent updates follow the LWW consistency rule documented in [concepts/coordination.md § Design Decisions](../../concepts/coordination.md#design-decisions-v0-defaults-revisitable).

## Side effects

- The catalogue `control_plane_object` instance is updated with each event.
- Audit events are emitted only for state-changing updates (not for reads): `AgentCatalogUpdated { event_type, entity_id, diff }`.
- The "catalog-agent status" indicator used on [admin/08-agent-roster-list.md R7](../admin/08-agent-roster-list.md#5-read-requirements) derives from this flow's queue depth.

## Failure modes

- **Queue saturation** → status transitions `syncing → lag`; if queue remains saturated >5 min, status becomes `error` and the org dashboard [admin/07-organization-dashboard.md](../admin/07-organization-dashboard.md) shows a banner.
- **Storage error on update** → retry with backoff; after 3 failures, event moves to dead-letter; `AgentCatalogUpdateFailed` alerted event emitted.
- **Agent catalog agent disabled** → no updates; all queries return stale data with a staleness indicator; the admin is strongly warned not to disable this agent.

## Observability

- Metrics: `baby_phi_agent_catalog_queue_depth`, `baby_phi_agent_catalog_update_duration_seconds`, `baby_phi_agent_catalog_lag_seconds` (time between triggering event and catalogue update).
- Audit events: `AgentCatalogUpdated`, `AgentCatalogUpdateFailed`.

## Cross-References

**Concept files:**
- [concepts/system-agents.md § Agent Catalog Agent](../../concepts/system-agents.md#agent-catalog-agent).
- [concepts/ontology.md](../../concepts/ontology.md) — the edge types subscribed to.

**Admin page provisioning this flow:**
- [admin/06-org-creation-wizard.md](../admin/06-org-creation-wizard.md) — creates the agent at org adoption.
- [admin/13-system-agents-config.md](../admin/13-system-agents-config.md) — tune / disable.
- [admin/08-agent-roster-list.md](../admin/08-agent-roster-list.md) — consumer (shows catalog status indicator).
- [admin/09-agent-profile-editor.md](../admin/09-agent-profile-editor.md) — upstream creator of Agent nodes.

**Related flows:**
- [s04-auth-request-state-transitions.md](s04-auth-request-state-transitions.md) — the catalogue is ALSO written when Template adoption creates grants that reference agents.
