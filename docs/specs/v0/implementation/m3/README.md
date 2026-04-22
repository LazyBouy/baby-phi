<!-- Last verified: 2026-04-22 by Claude Code -->

# M3 — Organization Creation + Dashboard

Ships admin pages 06 (org creation wizard) and 07 (org dashboard) on
top of M2's platform infrastructure. First milestone that opens
per-org audit chains (`org_scope = Some(org_id)`), provisions
governance-layer agents that wrap phi-core's
`AgentProfile` / `ExecutionLimits` / `ContextConfig` / `RetryConfig`,
and establishes the multi-step web wizard pattern reused by M4+.

Plan archive: [`../../../plan/build/563945fe-m3-organization-creation.md`](../../../plan/build/563945fe-m3-organization-creation.md).

## Phase status

| Phase | Status | Scope |
|---|---|---|
| P0 — M2 pre-flight delta log | ✓ done | [m2-preflight-delta.md](architecture/m2-preflight-delta.md) — 9-item audit pass; 8 still-valid / 1 missing (expected) / 0 stale |
| P1 — Foundation (IDs, nodes, composites, migration, docs tree, wizard primitives, CLI scaffolding) | ✓ done | `HasLead` edge + Organization struct extension + `composites_m3.rs` + migration 0003 + wizard primitives + ADR-0020 + ADR-0021 |
| P2 — Repository expansion + Template A/B/C/D builders + M3 audit events | ✓ done | 5 org-scoped list methods + `domain::templates::{a,b,c,d}` + `audit/events/m3/orgs.rs` |
| P3 — handler_support extensions + compound tx + harness | ✓ done | `apply_org_creation` + `emit_audit_batch` + `spawn_claimed_with_org` + two-org hash-chain proptest + ADR-0023 |
| P4 — Page 06 vertical (Org Creation Wizard) | ✓ done | Rust business logic + HTTP handler + CLI `org {create,list,show}` + 8-step Web wizard + reference-layout fixtures + ADR-0022 |
| P5 — Page 07 vertical (Org Dashboard) | ✓ done | Business logic + `GET /orgs/:id/dashboard` + CLI `org dashboard` + Web dashboard with 30 s polling + 7 acceptance scenarios + phi-core-strip invariant pinned |
| P6 — Seal (cross-page acceptance + metrics + CI + runbook + re-audit) | ✓ done | `acceptance_m3.rs` (2 scenarios) + CI rust.yml extension (3 new binaries) + runbook M3 section + M3 troubleshooting doc + completion regression across all shells + independent 3-agent re-audit (composite 98.4% → 99% after LOW remediation) |

## ADRs

| # | Title | Status | Decision |
|---|---|---|---|
| [0020](decisions/0020-organization-defaults-embedded.md) | Organization Defaults embedded on the Organization node | Accepted | Snapshot is a field on `Organization`, not a sibling composite. Matches ADR-0019's non-retroactive semantics. |
| [0021](decisions/0021-wizard-autosave-session-storage.md) | Wizard autosave via client-side session storage | Accepted | No `organization_drafts` table; `sessionStorage` + `localStorage` fallback. M7b upgrades to server-side drafts if multi-admin orgs become common. |
| [0022](decisions/0022-org-creation-compound-transaction.md) | Compound org-creation transaction | Accepted | One `Repository::apply_org_creation` SurrealQL tx covers every write. |
| [0023](decisions/0023-system-agents-inherit-from-org-snapshot.md) | System agents inherit execution context from org snapshot | Accepted | No per-agent `ExecutionLimits` / `ContextConfig` / `RetryConfig` / `CompactionPolicy` nodes at M3 — system agents read from `Organization.defaults_snapshot` at invoke time. Each phi-core type lives in one place per org. |

## phi-core leverage (per-phase)

Every phase section in the plan archive carries a `### phi-core
leverage` subsection structured per the
[leverage checklist](architecture/phi-core-leverage-checklist.md) — a
Q1 (direct imports) / Q2 (transitive payload) / Q3 (candidates
considered and rejected) split with deliverable-level phi-core tags
and positive close-audit grep assertions. Landed mid-M3 after the P3
"leverage = None" slip; applies retroactively (P0–P2 re-audited and
confirmed clean) and prospectively (P3+).

The durable reference table of type-level wraps lives in
[`architecture/phi-core-reuse-map.md`](architecture/phi-core-reuse-map.md);
the per-phase process discipline lives in the checklist.

**Summary**: four phi-core types wrapped at P1 (`ExecutionLimits`,
`AgentProfile`, `ContextConfig`, `RetryConfig` — inherited from
M2/P7's pattern), materialised as per-agent blueprint instances at P3
(first production uses of `phi_core::AgentProfile.clone()`), reused
again at P4 via the wizard orchestrator. P2, P5, and P6 sit entirely
on phi's governance plane (templates, auth requests, audit
events, dashboard aggregate reads, cross-page tests, CI, docs) —
legitimate phi-core-free surfaces per CLAUDE.md §Orthogonal surfaces.
P5 additionally *strips* `Organization.defaults_snapshot` from the
dashboard wire shape by design, pinned by schema-snapshot tests at
four tiers (unit + acceptance + cross-page + web) so phi-core schema
evolution never forces a polling-contract rev. D11 pins that
`Organization` is NOT a wrap of `phi_core::session::model::Session`
— drill-down to Session details happens via FK navigation at M5, not
type-level coupling. D12 / ADR-0023 pin that system agents inherit
`ExecutionLimits` / `ContextConfig` / `RetryConfig` from
`Organization.defaults_snapshot` — no per-agent duplication.

## Testing posture (plan §5)

Target: M2 close 511 Rust + 36 Web = **547** → M3 close **~650**
combined. **Final M3/P6 close: 633 Rust + 55 Web = 688**
(+122 Rust / +19 Web over M2). Per-phase close audit always runs the
same 3-aspect check (code correctness + docs accuracy + phi-core
leverage) with explicit % target. Independent 3-agent re-audit at P6
reported 99.0% / 99.2% / 99.0% (composite 99.1% after LOW
remediation — added two E2E layout-fidelity tests + flipped the
README P6 status).
