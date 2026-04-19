<!-- Status: CONCEPTUAL -->
<!-- Last verified: 2026-04-15 by Claude Code -->
<!-- Requirements doc — system flow -->

# s05 — Template Adoption Grant Fires

## Purpose

Authority Templates (A, B, C, D, E) are **pre-authorised allocations**: at adoption time, an Auth Request with `scope: [allocate]` is approved. That single approval covers every future Grant the template will fire as matching graph events occur. This flow watches the relevant graph events and, for each active template, materialises Grants whose provenance references the template's adoption Auth Request.

## Trigger

Graph edge mutations that match any active template's fire rule:

- **Template A** (Project Lead Authority): `HAS_LEAD` edge created from Project → Agent.
- **Template B** (Direct Delegation Authority): `DELEGATES_TO` edge created between Agents at a Loop.
- **Template C** (Hierarchical Org Chart): agent appointed to a node in the org tree (new `MANAGES`/`REPORTS_TO` edge).
- **Template D** (Project-Scoped Role): `HAS_AGENT` edge created with `role: supervisor` on a Project.
- **Template E** (Explicit): manual per-case; does not fire automatically — the admin or requestor submits the Auth Request directly.

## Preconditions

- The template has been Adopted for the relevant org (see [admin/12-authority-template-adoption.md](../admin/12-authority-template-adoption.md)).
- The adoption Auth Request is in Active state (not Revoked).

## Behaviour

- **R-SYS-s05-1:** On each triggering edge mutation, the flow SHALL check whether any active template's fire rule matches. Multiple templates may fire on the same event (e.g., Template A and Template B both trigger on a `HAS_LEAD` that follows a `DELEGATES_TO`).
- **R-SYS-s05-2:** For each matching template, the flow SHALL materialise a Grant with the template's YAML grant shape (see [concepts/permissions/05 § Authority Templates](../../concepts/permissions/05-memory-sessions.md#authority-templates-formerly-the-authority-question)), substituting the specific agent/project/resource references from the triggering event.
- **R-SYS-s05-3:** The Grant's `provenance` SHALL structurally reference the template's adoption Auth Request (via the `DESCENDS_FROM` edge). This chains the authority back to the org admin's original approval.
- **R-SYS-s05-4:** The Grant's `revocation_scope` SHALL be `revoke_when_edge_removed` — removing the triggering edge (e.g., demoting the lead) auto-revokes the Grant forward-only.
- **R-SYS-s05-5:** Fires SHALL be gated by consent per [concepts/permissions/06 § Three Consent Policies](../../concepts/permissions/06-multi-scope-consent.md#three-consent-policies):
  - `implicit` — Grant fires without a Consent prerequisite.
  - `one_time` / `per_session` — the Grant fires but is effective only when the relevant Consent record is in Acknowledged state at Permission Check time (Step 6).
- **R-SYS-s05-6:** For co-owned resources, the flow SHALL apply the per-co-owner consent rule from [concepts/permissions/06 § Co-Ownership × Multi-Scope rule 6](../../concepts/permissions/06-multi-scope-consent.md#co-ownership--multi-scope-session-access) — one Grant is issued but its effective use is gated by the intersection of each co-owner's consent policies.

## Side effects

- New Grant nodes with `HOLDS_GRANT` edges from the target Agent.
- Audit events: `AuthorityTemplateGrantFired { template_id, grant_id, agent_id, triggering_edge }`.
- If `revocation_scope: revoke_when_edge_removed`, future removal of the triggering edge will invoke [s04-auth-request-state-transitions.md](s04-auth-request-state-transitions.md)'s revocation cascade.

## Failure modes

- **Multiple simultaneous fires** → batched; each is a separate Grant writes, but all within the same logical transaction per triggering event (no partial cross-template state).
- **Template misconfiguration** (grant YAML has an invalid reference) → that template's fire is skipped with an alerted `TemplateFireSkipped` event; other templates still fire; admin alerted to fix the template.
- **Consent gating absence** → in `one_time`/`per_session` orgs, Grants fire but Permission Check denies invocation until Consent is Acknowledged; no error, just a naturally Pending permission check.

## Observability

- Metrics: `baby_phi_template_fires_total{template_id, org_id}`, `baby_phi_template_fires_per_event` (histogram), `baby_phi_template_fire_latency_seconds`.
- Audit events: `AuthorityTemplateGrantFired`, `TemplateFireSkipped`.

## Cross-References

**Concept files:**
- [concepts/permissions/05 § Authority Templates](../../concepts/permissions/05-memory-sessions.md#authority-templates-formerly-the-authority-question) — the 5 template definitions.
- [concepts/permissions/07 § Templates Are Pre-Authorized Allocations](../../concepts/permissions/07-templates-and-tools.md#templates-are-pre-authorized-allocations).
- [concepts/permissions/06 § Three Consent Policies](../../concepts/permissions/06-multi-scope-consent.md#three-consent-policies).

**Admin page provisioning this flow:**
- [admin/12-authority-template-adoption.md](../admin/12-authority-template-adoption.md) — Approve/Revoke actions there directly control whether this flow fires.

**Related flows:**
- [s04-auth-request-state-transitions.md](s04-auth-request-state-transitions.md) — the adoption Auth Request itself transitions there; revocation cascade from template revocation also lives there.
- [s03-edge-change-catalog-update.md](s03-edge-change-catalog-update.md) — the same edge events that trigger this flow also update the agent catalogue.
