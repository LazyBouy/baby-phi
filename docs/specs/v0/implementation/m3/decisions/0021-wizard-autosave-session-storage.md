<!-- Last verified: 2026-04-22 by Claude Code -->

# ADR-0021 — Wizard autosave via client-side session storage

**Status: Accepted** — landed in M3/P1.

## Context

R-ADMIN-06-W1 requires the org-creation wizard to "persist draft
across sessions" so an admin mid-way through the 8-step wizard does
not lose work on refresh. Two architectures:

1. **Server-side drafts.** A new `organization_drafts` table; `POST
   /api/v0/orgs/drafts` creates/updates a draft row; the wizard
   hydrates from `GET /api/v0/orgs/drafts/{draft_id}`; a
   garbage-collection policy reaps unfinished drafts (e.g. after 7
   days). Robust across browsers/devices and survives a full laptop
   replacement.
2. **Client-side session storage.** Browser `sessionStorage` for the
   live tab + `localStorage` as a fallback for cross-tab /
   refresh-tolerance. No new backend surface.

## Decision

**Option 2 — client-side session storage with localStorage
fallback.** Wizard steps push into a React Context
([`DraftContext`](../../../../../../modules/web/app/components/wizard/DraftContext.tsx))
that writes both `sessionStorage` (primary) and `localStorage`
(fallback). No new server-side table, no new API endpoints, no GC
policy, no migration-0003 draft-storage work.

## Consequences

### Pros

- **Zero new backend surface for M3.** Server-side drafts would
  require: the table, 2 endpoints (GET/PUT draft), a GC policy, and
  re-plumbing at M7b when OAuth lands (a session-cookie-scoped draft
  belongs to the admin-who-authenticated-THIS-session, but M7b
  migrates identity semantics). Shipping drafts now means rewriting
  them at M7b.
- **Refresh tolerance is sufficient.** R-ADMIN-06-W1's "persist
  across sessions" primary concern is "refresh the page mid-wizard
  and don't lose what I typed." `sessionStorage` handles that;
  `localStorage` handles cross-tab within the same browser.
- **Matches the M3 operator model.** M3's platform admin is a
  single principal (the claimant of the platform bootstrap). They
  do not share the wizard draft across multiple devices /
  colleagues. The multi-admin-shared-draft case is M7b work.

### Cons

- **Lost on device swap.** An admin who starts the wizard on a
  laptop and finishes on a tablet loses the draft. Mitigated: the
  wizard flows top-to-bottom in under ~15 minutes on any machine
  that can see the server; device swap mid-wizard is unusual.
- **Storage quotas.** `sessionStorage` is ~5 MB per origin. The
  wizard draft is <50 KB even with verbose YAML config; quota is
  not a concern.
- **No audit trail on drafts.** A persisted draft would give
  compliance a "who started an org creation and didn't finish"
  record. Deferred to M7b when server-side drafts land.

## Upgrade path (M7b)

If multi-admin orgs become common, the `organization_drafts` table
(with columns `draft_id, admin_agent_id, payload, created_at,
expires_at`) + `POST / PUT / DELETE /api/v0/orgs/drafts` would layer
on top of the existing wizard without changing the client's state
model — `DraftContext` would gain a "sync to server" side-effect
alongside `sessionStorage`, and the final wizard submit would
DELETE the server draft on success.

## phi-core leverage

None. Draft-persistence is a web-tier UX concern; phi-core has no
concept of UI draft state. Baby-phi-native.

## References

- [`../../../plan/build/563945fe-m3-organization-creation.md`](../../../../plan/build/563945fe-m3-organization-creation.md) §G10 / §D3.
- R-ADMIN-06-W1: [`../../../requirements/admin/06-org-creation-wizard.md`](../../../requirements/admin/06-org-creation-wizard.md).
- [`../architecture/wizard-primitives.md`](../architecture/wizard-primitives.md) — the `DraftContext` implementation.
