<!-- Last verified: 2026-04-23 by Claude Code -->

# Page 12 — Authority Template Adoption architecture

**Status**: `[EXISTS]` as of M5/P5. Business logic lives in
[`server::platform::templates`](../../../../../../modules/crates/server/src/platform/templates/);
HTTP surface in
[`server::handlers::templates`](../../../../../../modules/crates/server/src/handlers/templates.rs).
Migration 0005 flipped template uniqueness from name to kind per
[ADR-0030](../decisions/0030-template-node-uniqueness.md) at M5/P1.

## HTTP surface

Five routes registered in
[`router.rs`](../../../../../../modules/crates/server/src/router.rs):

| Method | Path | Handler | Purpose |
|---|---|---|---|
| GET  | `/api/v0/orgs/:org_id/authority-templates` | `templates::list` | 4-bucket listing (R-ADMIN-12-R1/R2/R3) |
| POST | `/api/v0/orgs/:org_id/authority-templates/:kind/approve` | `templates::approve` | Transition pending adoption AR → Approved (R-ADMIN-12-W1) |
| POST | `/api/v0/orgs/:org_id/authority-templates/:kind/deny` | `templates::deny` | Transition pending adoption AR → Denied (R-ADMIN-12-W2) |
| POST | `/api/v0/orgs/:org_id/authority-templates/:kind/adopt` | `templates::adopt` | Create + auto-approve new adoption AR (R-ADMIN-12-W3) |
| POST | `/api/v0/orgs/:org_id/authority-templates/:kind/revoke` | `templates::revoke` | Forward-only cascade over `DESCENDS_FROM` grants (R-ADMIN-12-W4) |

`:kind` is one of `a`, `b`, `c`, `d`. Sending `e` / `e-always`
returns 400 `TEMPLATE_E_ALWAYS_AVAILABLE` (Template E has no
adoption lifecycle per R-ADMIN-12-R3). Sending `system_bootstrap` /
`f` returns 409 `TEMPLATE_KIND_NOT_ADOPTABLE`.

## 4-bucket listing semantics

`GET /authority-templates` returns
[`TemplatesListing`](../../../../../../modules/crates/server/src/platform/templates/list.rs):

```jsonc
{
  "pending":   [{"kind":"a", "auth_request_id":"…", "fires_count":N, "submitted_at":"…"}],
  "active":    [/* same row shape */],
  "revoked":   [/* same row shape */],
  "available": ["c", "d", "e-always"]
}
```

Bucket rules:
- **Pending** — adoption AR in `Pending` or `Partial` state.
- **Active** — adoption AR in `Approved` state. At M3 org-creation, Template A / B / C / D adopted by the wizard are immediately Approved (Template-E shape).
- **Revoked** — adoption AR in `Revoked` state. Surfaces for audit traceability.
- **Available** — `TemplateKind` values in `{A, B, C, D}` with no live adoption (Denied / Expired / Cancelled / never-adopted all count as available), plus the fixed `"e-always"` sentinel per R-ADMIN-12-R3.

`fires_count` is the number of grants ever fired under the adoption AR (active + already-revoked). Pre-cascade reading; `revoke` reports the count of grants that flipped on this call.

## Adopt flow (R-ADMIN-12-W3)

`templates::adopt::adopt_template_inline`:
1. Refuse if the request is for E or a non-adoptable kind.
2. Refuse if the org has a live adoption AR for `kind` (Pending / Partial / Approved) — returns 409 `TEMPLATE_ADOPTION_ALREADY_PENDING` or `TEMPLATE_ADOPTION_ALREADY_ACTIVE`.
3. Resolve the org's CEO (first Human-kind agent).
4. Call `domain::templates::adoption::build_adoption_request` — Template-E-shaped, immediately Approved.
5. Persist via `Repository::create_auth_request`.
6. Emit `platform.template.adopted { mode: "adopt_inline", template_kind }` audit (Alerted).

## Revoke flow (R-ADMIN-12-W4 + R-ADMIN-12-N3)

`templates::revoke::revoke_template`:
1. Refuse if kind is E or non-adoptable, or if the reason is empty.
2. Locate the active adoption AR via `find_adoption_ar(org, kind)`.
3. Refuse if the AR is not in `Approved` state (returns 409 `TEMPLATE_ADOPTION_TERMINAL`).
4. Transition the AR → Revoked via `domain::auth_requests::revoke`.
5. Call `Repository::revoke_grants_by_descends_from(ar.id, now)` — forward-only cascade over every live grant whose `descends_from == ar.id`.
6. Emit `platform.template.revoked { template_kind, grant_count_revoked, reason }` audit (Alerted).

The cascade is **forward-only** per
[system/s04-auth-request-state-transitions.md] — the already-revoked grants stay revoked; the cascade flips live grants only. Idempotent on repeat (second revoke returns 409 immediately; no double-cascade).

## Template fire rules (recap)

| Template | Trigger edge | Grant issued (at fire time) |
|---|---|---|
| A | `HAS_LEAD` edge created (M4/P3) | Project lead gets `[read, inspect, list]` on `project:<id>`. |
| B | Shape B AR both-approve (M4/P6) | Mutual co-governance — each co-owner org gets grants on the project. |
| C | `MANAGES` edge created (M5/P3) | Manager gets `[read, inspect]` on `agent:<subordinate>`. |
| D | `HAS_AGENT_SUPERVISOR` edge created (M5/P3) | Supervisor gets `[read, inspect]` on `project:<p>/agent:<supervisee>`. |
| E | On-demand via AR (no passive fire) | Custom per AR; user drives. |

At M5, the passive-fire listeners for A + C + D are wired in
[`state::build_event_bus_with_m5_listeners`](../../../../../../modules/crates/server/src/state.rs).
Page 12 controls WHETHER those listeners actually fire by transitioning the adoption AR — a Revoked adoption AR means future trigger-edge events don't mint grants (the fire-listeners check AR state via `find_adoption_ar`).

## P5 advisory — D4.1 carry-forward

Adopted templates mint grants at fire time, which become Permission-Check inputs at session-launch time. At M5, the launch chain gates on Step 0 (Catalogue) only; steps 1-6 are advisory ([D4.1 drift](../../../../plan/build/01710c13-m5-templates-system-agents-sessions.md#p4-drift-addenda)). So even a fully adopted Template A/C/D won't refuse a launch at M5 — it populates the Decision trace that surfaces on the preview endpoint + LaunchReceipt. M6+ tightens the gate when the per-action manifest catalogue lands.

## phi-core leverage

**Zero new phi-core imports at P5** (pure phi governance plane). Import count stays at 25 lines (P4 close baseline).

## Cross-references

- [ADR-0030](../decisions/0030-template-node-uniqueness.md) — template uniqueness via kind.
- [M4 template-a-firing.md](../../m4/architecture/template-a-firing.md) — Template A firing precedent.
- [Event bus M5 extensions](./event-bus-m5-extensions.md) — DomainEvent variants driving Template C / D firings.
- [M5 plan §P5](../../../../plan/build/01710c13-m5-templates-system-agents-sessions.md).
