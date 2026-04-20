<!-- Last verified: 2026-04-20 by Claude Code -->

# Auth Request state machine

P4 ships the Auth Request lifecycle as a **pure**, proptest-covered state
machine in the `domain` crate. Every Grant in the system (except those
descending directly from the System Bootstrap Template) traces back to
an approved Auth Request, so this module is the structural foundation
for P5 bootstrap and every later approval flow.

- Module: [`modules/crates/domain/src/auth_requests/`](../../../../../../modules/crates/domain/src/auth_requests/mod.rs)
- Concept doc: [`concepts/permissions/02-auth-request.md`](../../../concepts/permissions/02-auth-request.md)
- ADR: [0010 — Per-slot aggregation for Auth Requests](../decisions/0010-per-slot-aggregation.md)

## 9-state lifecycle

```text
     ┌──── Draft ────────────────────┐ cancel
     │   (requestor editing)         ▼
     │                          Cancelled
     │ submit
     ▼
   Pending ─────────── cancel ────▶ Cancelled
     │  (all slots Unfilled)        ▲
     │ any slot filled              │
     ▼                               │ cancel
   InProgress ──────────────────────┘
     │  (slots being filled / reconsidered)
     │
     ├─ every resource Approved ─────────────▶ Approved ─┬─ revoke ─▶ Revoked
     │                                                    │
     ├─ every resource Denied ───────────────▶ Denied     └─ expire ─▶ Expired
     │                                             (terminal)
     │                                         ┌─ override_approve ─▶ Approved
     ├─ mixed Approved/Denied (co-owner split)─▶ Partial ─┼─ close_as_denied  ─▶ Denied
     │                                                    ├─ revoke           ─▶ Revoked
     │                                                    └─ expire           ─▶ Expired
     │
     └─ valid_until elapsed ───────────────▶ Expired
```

**Terminal states.** Six of the nine are terminal:

- **Closed** (no transitions out; slots locked) — `Denied`, `Expired`,
  `Revoked`, `Cancelled`.
- **Semi-open** (slot reconsideration + owner override permitted) —
  `Partial`.
- **Semi-terminal** (admits `revoke` / `expire`) — `Approved`.

## Aggregation — the truth flows upward

```text
  ApproverSlotState (3)      ──aggregate──▶    ResourceSlotState (5)      ──aggregate──▶    AuthRequestState (9)
  ─────────────────────                         ─────────────────────                         ───────────────────
  Unfilled                                      InProgress                                    Draft
  Approved                                      Approved                                      Pending
  Denied                                        Denied                                        InProgress
                                                Partial                                       Approved
                                                Expired                                       Denied
                                                                                              Partial
                                                                                              Expired
                                                                                              Revoked
                                                                                              Cancelled
```

The rules are transcribed from `concepts/permissions/02` §Per-Resource
State Derivation and §State Machine:

### Per-resource (slots → resource)

| Slot configuration | Resource state |
|---|---|
| All slots `Approved` | `Approved` |
| All slots `Denied` | `Denied` |
| Any `Denied` + any `Approved` | `Partial` |
| Any `Unfilled` (any mix of Approved / Denied too) | `InProgress` |
| Empty slot list | `InProgress` *(degenerate)* |

### Request-level (resources → request)

| Resource configuration | Request state |
|---|---|
| All `Approved` | `Approved` |
| All `Denied` | `Denied` |
| Any `InProgress` + some slot filled | `InProgress` |
| Any `InProgress` + no slots filled | `Pending` |
| Mix of `Approved`/`Denied`, no `InProgress` | `Partial` |
| Any resource `Partial`, no `InProgress` | `Partial` |

Aggregation is a no-op on **sticky** states: once a request enters
Approved / Denied / Expired / Revoked / Cancelled (or Draft), the only way
out is an **explicit** transition call — slot edits cannot move the
state.

## Transition API

Every function in
[`modules/crates/domain/src/auth_requests/transitions.rs`](../../../../../../modules/crates/domain/src/auth_requests/transitions.rs)
returns `Result<AuthRequest, TransitionError>`. No mutation in place; a
rejected op leaves the caller's `AuthRequest` untouched.

| Function | From | To | Notes |
|---|---|---|---|
| `submit(req, at)` | `Draft` | `Pending` | Stamps `submitted_at` |
| `transition_slot(req, r, s, state, at)` | any active or `Partial` | aggregated | Stamps `responded_at` / `reconsidered_at` on slot |
| `reconsider_slot(req, r, s, at)` | as above | aggregated | Convenience wrapper (slot → `Unfilled`) |
| `cancel(req, at)` | `Draft` / `Pending` / `InProgress` | `Cancelled` | Requestor-initiated |
| `override_approve(req, at)` | `Partial` | `Approved` | Owner forces every slot to `Approved` |
| `close_as_denied(req, at)` | `Partial` | `Denied` | Owner forces every slot to `Denied` |
| `expire(req, now)` | `Pending` / `InProgress` / `Approved` / `Partial` | `Expired` | Requires `valid_until ≤ now` |
| `revoke(req, actor, reason, at)` | `Approved` / `Partial` | `Revoked` | Lives in [`revocation.rs`](../../../../../../modules/crates/domain/src/auth_requests/revocation.rs); returns the audit event to persist |

The legal-transition table
([`state::legal_request_transition`](../../../../../../modules/crates/domain/src/auth_requests/state.rs))
is the single source of truth; every transition helper ends by calling
it via the private `enter_state` function.

## Revocation is forward-only

Once a request is `Revoked`, no transition moves it out. The proptest
[`revocation_is_forward_only`](../../../../../../modules/crates/domain/tests/auth_request_revocation_props.rs)
enforces this by attempting every public transition from a `Revoked`
request and asserting each one errors (or self-transitions into
`Revoked`).

Grants that descend from a revoked Auth Request must be revoked as a
cascade — that's a **repository concern**, kept out of the domain
state machine so the module stays pure. P5 bootstrap will wire the
cascade by querying the repository for `grant.descends_from == req.id`
and calling `revoke_grant` on each match.

Every successful `revoke` returns an `AuditEvent` pre-built with:

- `event_type = "auth_request.revoked"`
- `audit_class = AuditClass::Alerted`
- `target_entity_id` = the request's node id
- `provenance_auth_request_id` = the request id
- `prev_event_hash = None` (the `AuditEmitter` chains it)

## Retention — two-tier active/archived

Per `concepts/permissions/02` §Retention Policy:

- **Active tier** — retained in the hot path. Terminal Auth Requests
  are active for `active_window_days` after entering their terminal
  state. Default 90 days.
- **Archived tier** — after the active window elapses, the request
  moves to cold storage. Still auditable via `inspect_archived`; never
  deleted by retention alone.
- **Compliance deletion** (`delete_after_years`) is off by default in
  v0.1 and not modelled in M1.

[`retention.rs`](../../../../../../modules/crates/domain/src/auth_requests/retention.rs)
exposes:

- `active_until(req) → Option<DateTime<Utc>>` — cutoff =
  `terminal_state_entered_at + active_window_days`; `None` for
  pre-terminal requests.
- `is_archive_eligible(req, now, has_live_grants) → bool` — true iff
  the request is terminal AND the cutoff has passed AND no live grants
  remain. The caller supplies `has_live_grants` from the repository.
- `days_remaining(req, now) → Option<i64>` — clamped to zero below the
  cutoff; used by the monotonicity proptest and future observability
  dashboards.

The `auth_request_retention_props.rs` proptest pins the **monotonic
non-increasing** invariant on `days_remaining` over increasing `now`.

## Proptest coverage

Four files under
[`modules/crates/domain/tests/`](../../../../../../modules/crates/domain/tests/),
15 invariants, expanded to `PROPTEST_CASES=256` in CI by default
(≈3,840 branches per run). Files:

| File | Invariants |
|---|---|
| [`auth_request_aggregation_props.rs`](../../../../../../modules/crates/domain/tests/auth_request_aggregation_props.rs) | Resource aggregation matches the concept-doc table for every random slot configuration; request-level state follows the resource majority; aggregation is idempotent. |
| [`auth_request_transition_props.rs`](../../../../../../modules/crates/domain/tests/auth_request_transition_props.rs) | Closed terminals reject slot changes; cancel succeeds iff state is active; slot `(i, j)` change leaves every other slot untouched (slot independence); Expired is terminal; submit only from Draft. |
| [`auth_request_revocation_props.rs`](../../../../../../modules/crates/domain/tests/auth_request_revocation_props.rs) | Revocation is forward-only (every transition from Revoked errors or self-transitions); revoke only from Approved/Partial; revoke emits `Alerted` audit event with full provenance. |
| [`auth_request_retention_props.rs`](../../../../../../modules/crates/domain/tests/auth_request_retention_props.rs) | `days_remaining` is monotonically non-increasing over time; `active_until` = terminal entry + window; archive eligibility requires both elapsed window AND no live grants; pre-terminal requests are never archive-eligible. |

## What's deferred to later phases

- **Repository integration for cascading grant revocation.** P4 models the
  request-side `revoke`; P5 bootstrap calls `Repository::revoke_grant`
  for every grant whose `descends_from` matches.
- **Routing table / escalation logic.** `routing_override` and
  escalation-on-timeout are M2+ concerns (the concept doc calls them
  out but the M1 spine doesn't need them to bootstrap the platform
  admin).
- **Multi-scope session consent.** The concept doc's cross-org consent
  rules live in `06-multi-scope-consent.md` and land in M4.
- **`inspect_archived` retrieval scope.** The archive tier itself is
  modelled; the Discovery-action refinement for retrieval is M5+.
