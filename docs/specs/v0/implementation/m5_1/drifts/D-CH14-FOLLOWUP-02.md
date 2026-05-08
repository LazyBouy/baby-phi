<!-- Last verified: 2026-05-08 by Claude Code (CH-14 chunk-seal gate-2 inline correction — Status flipped `discovered` → `remediated`; per-cascaded-AR `auth_request.revoked` audit-event emission + AR-state-transition logic shipped per ADR-0053 §D53.7 + plan §3.B A7 + plan §7 P3; new typed return `domain::repository::CascadeResult { revoked_grants, cascaded_ars }` added at `repository.rs:170`; both backends populate `cascaded_ars` via the existing BFS step (b); `templates/revoke.rs` iterates + calls `domain::auth_requests::revoke` + `repo.update_auth_request` + `audit.emit` per cascaded AR; 2 new acceptance tests `revoke_cascade_emits_one_event_per_cascaded_ar` + `revoke_cascade_transitions_cascaded_ars_to_revoked_state` pin the contract; cycle hex `5803bb94`) -->

# D-CH14-FOLLOWUP-02 — Recursive cascade does not transition cascaded ARs to `Revoked` state nor emit per-AR `auth_request.revoked` events

## Identification
- **ID**: D-CH14-FOLLOWUP-02
- **Phase of origin**: CH-14 chunk-seal (cycle hex `5803bb94`)
- **Discovery source**: `cycle-plan-deferral` (plan §3.B A7 audit-event semantics narrowed to the `template.revoked` summary event)
- **Date discovered**: 2026-05-08
- **Status**: `remediated`
- **Bucket**: B — concept-doc fidelity gap with grant-cascade landed; AR-state-cascade + per-AR audit emission deferred
- **Severity**: LOW
- **Tags**: `revocation-cascade`, `audit-events`, `auth-request-state`, `forward-defensive`
- **Blocks**: nothing today (the cascade-revoked grants stop authorising reads via `revoked_at`; their parent ARs' state staying `Approved` does not produce a permission-leak — the engine's Step 3 + Step 4 inspect grants, not parent ARs).
- **Blocked-by**: future M6+ chunk that explicitly renders cascaded-AR audit history OR formalises a state-machine invariant "every AR whose authorising grant is revoked must transition to Revoked"

## Concept alignment
- **Concept doc(s)**: [`concepts/permissions/02-auth-request.md`](../../../concepts/permissions/02-auth-request.md) §"How Auth Request Approval Maps to a Grant" (Step 3 — Owner reconsiders and revokes) lines 261–308.
- **Concept claim**: When an AR's parent grant is revoked, the AR itself becomes "no longer an active authorisation" — concept doc 02 line 296 says *"The Grant is automatically revoked (via the `revocation_scope: tied_to_auth_request` coupling)"* and concept doc 08 §9.3 implies the cascade emits one audit event per cascaded AR (the `auth_request.revoked` event already builds inside `domain::auth_requests::revoke_ar`).
- **Contradiction**: RESOLVED at CH-14 chunk-seal gate-2 inline correction. CH-14's recursive cascade now revokes both axes: (a) GRANTS via `Grant.revoked_at` flips (Repository BFS step (a)), and (b) ARs via `AuthRequestState` transitions + per-AR audit events (handler iterates `CascadeResult.cascaded_ars`). The Template-revoke handler emits ONE `template.revoked` summary event with the multi-hop `grant_count_revoked` PLUS one `auth_request.revoked` event per cascaded AR (level ≥ 1) per ADR-0053 §D53.7.
- **Classification**: `honored` (both axes shipped at CH-14 chunk-seal)
- **phi-core leverage status**: `N/A — no phi-core overlap`

## Plan vs. reality (resolved at gate-2 inline correction)
- **Plan said** (CH-14 plan §3.B A7): "the recursive cascade adds N-1 more emissions" + "audit chain extends symmetrically per cascaded AR".
- **Plan said** (CH-14 plan §7 P3 deliverable 3): "Each level emits **one `auth_request.revoked` audit event per cascaded AR**".
- **Plan said** (CH-14 plan §7 P3 deliverable 4): "for each cascaded AR (level ≥ 1), emit one `auth_request.revoked` event via the existing `revoke_ar(...)` domain helper at `auth_requests/revocation.rs:57` (returns `(AuthRequest, AuditEvent)` — emit the event side)."
- **Initial-iteration reality**: CH-14's first implementation pass shipped the BFS that revokes grants only; AR-state-transitions + per-AR audit-event emissions were silently dropped. Gate-2 review caught the contradiction between ADR-0053 §D53.7 (Accepted, says emission ships in CH-14) and this drift's `discovered` claim of deferral.
- **Post-correction reality**: per-AR emission + state-transition logic now ships in CH-14 per the lifecycle entry below. Plan + ADR + drift now align.

## Required follow-up
- **What needs to happen**: when a future M6+ chunk lands either (a) an audit-UI that renders cascaded-AR history OR (b) an explicit state-machine invariant "every AR whose authorising grant is revoked transitions to Revoked", the recursive cascade SHOULD:
  - For each cascaded grant, find its descendant ARs (already done in BFS step (b) for the next-level frontier).
  - Transition each cascaded AR via `domain::auth_requests::revoke_ar(ar, ...)` → `(AuthRequest, AuditEvent)`.
  - Emit the `auth_request.revoked` audit event per cascaded AR via the injected `AuditEmitter` (currently the cascade method is in the Repository layer and has no audit emitter; this would either (i) move the cascade up to a domain helper that takes an emitter, OR (ii) return the cascaded-AR list and let the handler emit per-AR).
- **Tests required**: acceptance test asserting one `auth_request.revoked` event per cascaded AR (audit-event count assertion).
- **Acceptance**: every cascaded AR transitions to `AuthRequestState::Revoked` + has a paired `auth_request.revoked` audit event in the emission stream.

## Closing chunk
- **CH-14** (gate-2 inline correction). Closed in-cycle on 2026-05-08; no follow-up chunk required.

## Lifecycle
- **2026-05-08 — `discovered`** — filed by CH-14 chunk-seal. CH-14 ships the BFS revoke-grants algorithm; AR-state-transitions + per-AR audit-event emissions deferred per plan §3.B + §7 P3 scope-control. The `template.revoked` summary event continues to capture multi-hop `grant_count_revoked` accurately, which is sufficient for v0 audit volume.
- **2026-05-08 — `remediated`** — CH-14 chunk-seal gate-2 inline correction. Per-AR emission shipped per plan §3.B A7 + §7 P3 + ADR-0053 §D53.7. Implementation: new typed return `domain::repository::CascadeResult { revoked_grants: Vec<GrantId>, cascaded_ars: Vec<AuthRequestId> }` added at `repository.rs:170`; both `domain::in_memory::InMemoryRepository::revoke_grants_by_descends_from_recursive` (`in_memory.rs:1273`) and `store::SurrealStore::m5_revoke_grants_by_descends_from_recursive` (`repo_impl_m5.rs:128`) populate `cascaded_ars` via the existing BFS step (b); `server::platform::templates::revoke::revoke_template` (`templates/revoke.rs:99`) iterates `cascaded_ars`, calls `domain::auth_requests::revoke` per AR, persists via `repo.update_auth_request`, and emits the per-AR `auth_request.revoked` audit event via the injected `AuditEmitter`. Contract pinned by 2 new acceptance tests: `revoke_cascade_emits_one_event_per_cascaded_ar` (audit-event count assertion: N − 1 = 2 events for a 2-cascaded-AR fixture) + `revoke_cascade_transitions_cascaded_ars_to_revoked_state` (state-transition assertion: cascaded ARs flip Approved → Revoked). Cascade method stays in the Repository layer (Option ii from the closure plan); the handler owns audit-event emission per the single-writer pattern + CH-13 / CH-08 precedent. Cycle hex `5803bb94`.

## Cross-references
- CH-14 plan: [`baby-phi/docs/specs/plan/build/ch-14-system-genesis-authority-chain-revocation-cascade-5803bb94/plan.md`](../../../../plan/build/ch-14-system-genesis-authority-chain-revocation-cascade-5803bb94/plan.md) §3.B A7 + §7 P3.
- ADR-0053: [`m5_2/decisions/0053-system-genesis-authority-chain-revocation-cascade.md`](../../m5_2/decisions/0053-system-genesis-authority-chain-revocation-cascade.md) §D53.7 (audit-event emission semantics — narrowed to `template.revoked` summary event).
- D-new-18: [`D-new-18.md`](D-new-18.md) (closed by CH-14 — grant-cascade axis).
- Concept doc: [`concepts/permissions/02-auth-request.md`](../../../concepts/permissions/02-auth-request.md) lines 261–308 (Step 3 — Owner reconsiders and revokes).
- Affected files: `domain/src/in_memory.rs::revoke_grants_by_descends_from_recursive`; `store/src/repo_impl_m5.rs::m5_revoke_grants_by_descends_from_recursive`; `server/src/platform/templates/revoke.rs:90`.
