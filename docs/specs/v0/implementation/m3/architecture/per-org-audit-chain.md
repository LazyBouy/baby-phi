<!-- Last verified: 2026-04-22 by Claude Code -->

# Architecture — Per-org audit hash chain

**Status: [EXISTS]** — shipped at M3/P3 (fixture + proptest green).

## M1/M2/M3 scope boundary

The `org_scope: Option<OrgId>` column on `domain::audit::AuditEvent`
is the chain-selector. Every persisted event belongs to exactly one
chain:

| Milestone | Writes events with `org_scope` = | Example event types |
|---|---|---|
| **M1** (bootstrap) | `None` (platform root chain) | `system.bootstrap.admin_claimed` |
| **M2** (pages 02–05) | `None` (platform root chain) | `platform.model_provider.created`, `platform.mcp_server.tenants_narrowed`, `platform.defaults.put`, etc. |
| **M3** (pages 06–07) | `Some(org_id)` (per-org chain — **first milestone to do so**) | `platform.organization.created`, `authority_template.adopted` |

M4+ continues to write per-org events with `org_scope = Some(org_id)`;
subsequent org-level work (projects, agents, sessions) chains onto
the per-org chain opened at that org's `organization.created` event.

## Mechanics (unchanged since M1/P8)

`store::SurrealAuditEmitter::emit` — [`audit_emitter.rs`](../../../../../../modules/crates/store/src/audit_emitter.rs)
— runs the following synchronous sequence on every emit:

1. `let prev = repo.last_event_hash_for_org(event.org_scope).await?;`
2. `event.prev_event_hash = prev;`
3. `repo.write_audit_event(&event).await?;`

Step 1 is the chain-selector: passing `Some(org_id)` filters to the
per-org chain; `None` filters to the platform root chain. A bug
leaking a root-chain hash into a per-org chain (or vice versa) would
surface as a `prev_event_hash` mismatch in any subsequent audit-log
inspection. The proptest described below makes this
**behaviourally** unfalsifiable across 50 arbitrary interleavings.

## Verification

Three tests in the workspace exercise the per-org chain invariant at
three different layers:

1. **Domain proptest** — [`modules/crates/domain/tests/two_orgs_audit_chain_props.rs`](../../../../../../modules/crates/domain/tests/two_orgs_audit_chain_props.rs).
   Runs against `InMemoryRepository` with inline reproduction of
   `SurrealAuditEmitter`'s 3-line chain-link. 50 random interleavings
   of emits across two orgs; asserts:
   - First event of each chain has `prev_event_hash = None`.
   - Every subsequent event's `prev_event_hash = hash_event(previous
     in same chain)`.
   - No hash from chain A ever appears as `prev_event_hash` in
     chain B and vice versa.
2. **Store integration** — [`modules/crates/store/tests/audit_emitter_chain_test.rs`](../../../../../../modules/crates/store/tests/audit_emitter_chain_test.rs).
   The M2-era fixture test that exercises real SurrealDB + real
   emitter. Covers same invariants; pinned to the M2 scope but
   continues to pass at M3-close because the emitter path is
   unchanged.
3. **Store integration (M3/P3)** — [`modules/crates/store/tests/repo_m3_surface_test.rs::list_recent_audit_events_for_org_excludes_platform_root_chain`](../../../../../../modules/crates/store/tests/repo_m3_surface_test.rs).
   The dashboard-query-side assertion: an org-scoped listing must
   never surface events from the platform root chain (and vice
   versa) — the read path's complement of the write path's chain
   isolation.

## Post-M3: drill-down from audit events to phi-core sessions

D11/Q6 of the M3 plan pins that M5+ operators will drill from a
dashboard's `RecentAuditEvents` row into a `phi_core::Session` trace
when the event carries a session provenance attachment. At M3 the
audit diff does not include `session_id` (no sessions are launched
at org-creation time); M5 extends `AuditEvent` with a
`session_id: Option<SessionId>` field threaded through every
agent-loop-generated event, and the dashboard row's link target
switches from the audit-log detail page to `/sessions/<session_id>`
when present.

The key invariant M3 pins for M5+: **per-org chain continuity is
preserved across session launches** — session-related audit events
carry `org_scope = Some(org_id)` (the owning org's) and extend that
org's chain, not a per-session chain. M5's session launch handler
inherits this discipline from M3's `apply_org_creation` pattern.

## phi-core leverage

None — per-org audit is phi governance infrastructure. Phi-core
has no audit-trail surface (its [`AgentEvent`](../../../../../../../phi-core/src/types/event.rs)
enum is execution telemetry, not a hash-chained write log; see the
orthogonal-surfaces note in [phi/CLAUDE.md](../../../../../../CLAUDE.md)).
The Q3 candidate-rejection walk for this surface is the one-liner
"Not applicable — governance plane, not runtime plane."

## References

- [M3 plan §P3 / §G8](../../../../plan/build/563945fe-m3-organization-creation.md)
- [phi-core leverage checklist](phi-core-leverage-checklist.md)
- [`modules/crates/store/src/audit_emitter.rs`](../../../../../../modules/crates/store/src/audit_emitter.rs) — the (unchanged) emitter path.
- [M1 audit-events architecture](../../m1/architecture/audit-events.md) — envelope + hash-chain mechanics this extends.
