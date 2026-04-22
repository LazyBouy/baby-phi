<!-- Last verified: 2026-04-22 by Claude Code -->

# Architecture — M3 overview

**Status: [PLANNED M3/P6]** — full system map lands at P6 close.

M3 extends M2's platform-setup spine with admin pages 06 (org
creation wizard) and 07 (org dashboard). First milestone where audit
events open per-org chains (`org_scope = Some(org_id)`) and where
the platform provisions governance agents that wrap phi-core's
execution + blueprint types.

See:
- [`../../../plan/build/563945fe-m3-organization-creation.md`](../../../../plan/build/563945fe-m3-organization-creation.md) — full plan.
- [`m2-preflight-delta.md`](m2-preflight-delta.md) — P0 delta log confirming M2-close invariants held at M3 open.
- [`phi-core-reuse-map.md`](phi-core-reuse-map.md) — durable phi-core wrap/reuse map.
- [`../../m2/architecture/overview.md`](../../m2/architecture/overview.md) — the M2 spine this extends.

## What M3 adds on top of M2

| Area | Pre-M3 (M2 shipping) | M3 extension | Status |
|---|---|---|---|
| Graph model | 37 node kinds, 66 edges | `HasLead` edge → 67; `Organization` extended with 7 new fields | [EXISTS] (P1) |
| Composites | `composites_m2.rs` — ExternalService, ModelRuntime, PlatformDefaults, SecretCredential, TenantSet | `composites_m3.rs` — ConsentPolicy, OrganizationDefaultsSnapshot, TokenBudgetPool | [EXISTS] (P1) |
| Schema | Migrations 0001 + 0002 | Migration 0003 — extends `organization`, adds `token_budget_pool` | [EXISTS] (P1) |
| Repository | M2 list + cascade methods | 5 org-scoped list methods + compound `apply_org_creation` tx | [PLANNED M3/P2-P3] |
| Templates | SystemBootstrap + E | + A/B/C/D pure-fn builders | [PLANNED M3/P2] |
| Audit chain | `org_scope = None` exercised | `org_scope = Some(org_id)` per-org chain | [PLANNED M3/P3] |
| Handler infra | handler_support shim | `emit_audit_batch` helper + `spawn_claimed_with_org` fixture | [PLANNED M3/P3] |
| Pages | 02/03/04/05 (platform) | + 06 (org wizard) + 07 (org dashboard) | [PLANNED M3/P4-P5] |
| Web | Single-form admin pages | Multi-step wizard primitives (StepShell, StepNav, DraftContext, ReviewDiff) | [PLANNED M3/P1-P4] |
| CLI | bootstrap / secret / model-provider / mcp-server / platform-defaults / completion | + `phi org {create,list,show,dashboard}` | [PLANNED M3/P1-P5] |

Fleshed out at M3/P6 close with the shipped-code cross-reference table.
