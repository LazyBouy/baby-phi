<!-- Last verified: 2026-04-23 by Claude Code -->

# Page 12 — Authority Template Adoption architecture

**Status**: [PLANNED M5/P5] — stub seeded at M5/P0; filled when P5
vertical ships.

Page 12 drives operator approval / denial / inline-adoption /
revoke-cascade across the 5 authority templates (A / B / C / D /
E). Migration 0005 flips template uniqueness from name to kind per
[ADR-0030](../decisions/0030-template-node-uniqueness.md).

## Scope

- `GET  /api/v0/orgs/:org_id/authority-templates` — 4-bucket list (pending / active / revoked / available).
- `POST /api/v0/orgs/:org_id/authority-templates/:kind/approve`
- `POST /api/v0/orgs/:org_id/authority-templates/:kind/deny`
- `POST /api/v0/orgs/:org_id/authority-templates/:kind/adopt` (auto-approves; sole approver is the adopter per R-ADMIN-12-W3).
- `POST /api/v0/orgs/:org_id/authority-templates/:kind/revoke` — cascade.

## Template fire rules (summary)

| Template | Trigger edge | Grant issued |
|---|---|---|
| A | `HAS_LEAD` edge created (M4/P3) | Project lead gets `[read, inspect]` on the project's owning org. |
| B | Shape B AR both-approve (M4/P6) | Mutual co-governance — each co-owner org gets grants on the project. |
| C | `MANAGES` edge created (M5/P3) | Manager gets `[read, inspect]` on `agent:<subordinate>`. |
| D | `HAS_AGENT_SUPERVISOR` edge created (M5/P3) | Supervisor gets `[read, inspect]` on the supervisee's project scope. |
| E | On-demand via AR (no passive fire) | Custom per AR; user drives. |

## Revoke-cascade

Walks `DESCENDS_FROM` provenance edges from the adoption AR,
forward-only revokes each dependent Grant, emits
`AuthorityTemplateRevoked { grant_count_revoked, template_kind }`.
Preserves audit history (revoke is state transition, not delete).

## Cross-references

- [ADR-0030](../decisions/0030-template-node-uniqueness.md).
- [M4 template-a-firing.md](../../m4/architecture/template-a-firing.md) — Template A precedent.
- [M5 plan §P5](../../../../plan/build/01710c13-m5-templates-system-agents-sessions.md).
