<!-- Last verified: 2026-04-20 by Claude Code -->

# ADR-0010: Per-slot aggregation for Auth Requests

## Status

Accepted — 2026-04-20 (M1 / P4).

## Context

An Auth Request carries a list of resource slots, each with one or more
approver slots (see
[`concepts/permissions/02-auth-request.md`](../../../concepts/permissions/02-auth-request.md)
§Schema). The concept doc specifies the aggregation semantics in two
dense tables:

- **Per-resource aggregation** (slots → resource state, 5 values).
- **Request-level aggregation** (resources → request state, 9 values).

The implementation question for M1/P4 is: how do we represent these
two layers of aggregation in a way that (a) pins the concept-doc rules
without runtime drift, (b) stays pure enough to proptest, and (c) leaves
enough room for the owner-override and revocation flows to slot in
without extra indirection.

## Decision

### The slot stays the source of truth; everything else is derived.

1. **The 9-state `AuthRequestState` enum is authoritative at the
   request level** — it drives what operations are legal (legal
   transition table), what the Permission Check engine sees, and how
   audit events are tagged.
2. **The 5-state `ResourceSlotState` and 3-state `ApproverSlotState`
   enums are stored redundantly** for query convenience (a `WHERE
   resource_slots[WHERE state = 'Partial']` query is O(1) with the
   stored state), but are **always** computable from the slot vector.
3. **Aggregation is a pair of pure functions** in
   [`domain::auth_requests::state`](../../../../../../modules/crates/domain/src/auth_requests/state.rs):
   - `aggregate_resource_state(&[ApproverSlot]) -> ResourceSlotState`
   - `aggregate_request_state(&AuthRequest) -> AuthRequestState`
4. **Every `transition_*` function re-runs aggregation** after
   mutating slot state. The resource-level state on the `AuthRequest`
   struct is refreshed from the aggregator; the request-level state is
   refreshed if the aggregator's output differs from the current
   state. This guarantees the stored state never drifts from the
   slot-derived truth.
5. **Sticky terminal states short-circuit aggregation.** Once a
   request is Draft / Approved / Denied / Expired / Revoked /
   Cancelled, the aggregator returns the current state unchanged.
   Leaving these states requires an **explicit** transition call
   (`submit`, `revoke`, `expire`, `override_approve`,
   `close_as_denied`). This models the concept doc's "closed terminal"
   semantics: slots can't re-open them.

### The slot-state enum is minimal; refinements are properties of the slot.

The concept doc mentions `skipped_due_to_timeout` as a slot-level flag
used by the escalation machinery. M1/P4 does **not** land escalation —
it's M2+ — so the flag lives as a future extension on
`ApproverSlot` rather than a new state. Keeping `ApproverSlotState` at
three values (Unfilled / Approved / Denied) matches the core decision
shape; skipped-due-to-timeout is a presentation concern layered on top.

### Legal transitions are gated by a separate table, not encoded in the aggregator.

The aggregator answers "given the slot configuration, what state should
the request be in?" It does **not** answer "is it legal for the state
to move from A to B?" The latter is handled by
[`legal_request_transition(from, to) -> bool`](../../../../../../modules/crates/domain/src/auth_requests/state.rs)
— a separate pure function. Every `transition_*` helper consults it
before stamping the new state.

Splitting these two concerns lets the aggregator stay purely descriptive
(no policy; just math) and the legal table stay purely prescriptive (no
slot awareness; just arrows). The `illegal_transition_never_succeeds`
proptest covers the legal table; the aggregation proptests cover the
math.

## Consequences

Positive:

- **No drift.** The stored resource/request state is refreshed from the
  slot truth on every mutation. A bug in slot handling would surface as
  either an aggregation-proptest failure or a legal-transition-proptest
  failure; there's no "stale denormalized state" class.
- **Slot independence falls out naturally.** Because slot mutations go
  through `transition_slot(req, r, s, ...)` and only touch the
  addressed slot before re-aggregating, the
  `slot_independence_one_slot_change_does_not_mutate_others`
  proptest is a direct consequence, not a hand-coded guard.
- **Owner overrides stay honest.** `override_approve` and
  `close_as_denied` flip every slot to the owner's decision before
  calling `enter_state`, so the stored state and slot truth agree
  after the override. The proptest pins this.
- **Revocation stays forward-only** without special flags:
  `legal_request_transition` gives Revoked no outbound arrows except
  the self-transition, and `is_closed_terminal(Revoked) == true`
  blocks every slot-level mutation.

Negative:

- **Storage redundancy.** The resource-level `state` on the
  `ResourceSlot` is redundant with what aggregation would compute from
  its approvers. That's the price of SurrealDB-side query ergonomics
  (e.g. "find all Partial resources in the org's active Auth
  Requests"). A cheaper representation would drop the stored field and
  compute on read, but the `list_active_auth_requests_for_resource`
  query in P2 uses the stored state to filter.
- **The legal-transition table pairs some "degenerate" arrows.** A
  single-slot single-resource request can reach `Approved` or `Denied`
  directly from `Pending` in one slot-fill call — the concept-doc
  diagram draws this as `Pending → InProgress → Approved`, but the
  aggregator compresses it. The table explicitly lists these
  "degenerate" arrows so the legal-transition proptest matches the
  aggregator's actual output. A comment in
  [`state.rs`](../../../../../../modules/crates/domain/src/auth_requests/state.rs)
  explains the divergence; a reviewer skimming the diagram might
  briefly wonder why `Pending → Approved` is admitted.
- **`InProgress → Pending` is also admitted.** The "last-filled-slot
  reconsiders" backtrack is a legitimate case in the concept doc; we
  admit it explicitly rather than force the aggregator to lie about
  the slot state. Again documented inline.

## Alternatives considered

- **Store only the request state; compute resource state on demand.**
  Rejected. Would make SurrealDB-side filtering for "active
  Partial-state requests in org X" require a scan + compute rather
  than a single-index lookup. Cheap enough when M1 has hundreds of
  requests, expensive when M3+ has millions.
- **Collapse ApproverSlotState into a richer enum with
  `skipped_due_to_timeout` as its own value.** Rejected. That
  conflates "what did the approver decide" with "what did the
  escalation machinery do." The concept doc treats escalation as a
  separate layer; we follow suit and defer it to M2+.
- **Embed the legal-transition check in the aggregator.** Rejected.
  Couples two separate concerns (slot→state math vs state→state
  policy) into one function. Splitting them gives each concern its
  own proptest and makes "is this transition legal?" callable in
  isolation (useful for UI pre-flight checks in M2+).
- **Materialise a full state-machine graph at startup and run requests
  through it.** Overkill for 9 states + 3 slot states. The direct
  `legal_request_transition(from, to) -> bool` match table is 30
  lines, compile-time-checked, and grep-friendly.

## References

- Implementation:
  [`modules/crates/domain/src/auth_requests/state.rs`](../../../../../../modules/crates/domain/src/auth_requests/state.rs)
  (aggregation + legal transitions),
  [`transitions.rs`](../../../../../../modules/crates/domain/src/auth_requests/transitions.rs)
  (public transition surface),
  [`revocation.rs`](../../../../../../modules/crates/domain/src/auth_requests/revocation.rs)
  (forward-only revoke + audit event),
  [`retention.rs`](../../../../../../modules/crates/domain/src/auth_requests/retention.rs)
  (two-tier active/archived window).
- Architecture page:
  [auth-request-state-machine.md](../architecture/auth-request-state-machine.md).
- Proptest coverage: 4 files under
  [`modules/crates/domain/tests/`](../../../../../../modules/crates/domain/tests/)
  with prefix `auth_request_`, 15 invariants total.
- Concept doc:
  [`permissions/02-auth-request.md`](../../../concepts/permissions/02-auth-request.md)
  §State Machine, §Per-Resource State Derivation, §Retention Policy.
- Plan: [015a217a-m1-permission-check-spine.md §P4](../../../../plan/build/015a217a-m1-permission-check-spine.md).
