<!-- Last verified: 2026-04-22 by Claude Code -->

# Architecture — Org Dashboard (page 07)

**Status: [PLANNED M3/P5]** — fleshed out when P5 ships.

Aggregate-read surface: agents / projects / pending AR count / alerted
event count / token budget utilisation / recent audit events /
adopted templates, plus empty-state CTA cards. 30 s client-side
polling per D4; M7b upgrade path to WebSocket push.

See:
- [`../../../plan/build/563945fe-m3-organization-creation.md`](../../../../plan/build/563945fe-m3-organization-creation.md) §P5.
- [`../../../requirements/admin/07-organization-dashboard.md`](../../../requirements/admin/07-organization-dashboard.md) — page 07 requirements.

## phi-core leverage

None — pure baby-phi aggregate reads. Documented explicitly so P5's
close audit doesn't surface a false "did we miss phi-core reuse?"
finding. Org-level drill-down into individual `phi_core::Session`
replay / trace views is a navigation pattern via FK containment
(M5+); M3 dashboard rows link to the audit-log page, and M5 rewires
link targets to session-trace URLs when events carry session
provenance. See D11 in the plan archive.
