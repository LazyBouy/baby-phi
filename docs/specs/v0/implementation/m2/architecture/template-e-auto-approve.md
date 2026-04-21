<!-- Last verified: 2026-04-21 by Claude Code -->

# Architecture — Template E: self-interested auto-approve

**Status: [EXISTS]**

Every M2 admin write (pages 02–05) follows the same shape: the platform
admin is simultaneously the **requestor** and the **approver** of the
write. Template E is baby-phi's name for that pattern, modelled as a
pure helper in the domain layer and consumed by every page-04+ handler.

## Why not drive the 9-state machine?

M1's Auth-Request state machine has an explicit legal-transition table
([`auth_requests/transitions.rs`][1]). There is **no `Unset → Approved`
edge** on approver slots — by design, slot transitions flow through
`Pending → Filled → Approved` to force the aggregation function to
observe every intermediate state. That works when the approver is a
different principal from the requestor, but it's busywork when they're
the same: the admin would drive their own approver slot through a
three-step dance whose outcome was predetermined the moment they hit
the endpoint.

Template E sidesteps the transition pipeline by **constructing the
Auth Request already in `Approved` state** — the single approver slot
is populated at struct-init time with `ApproverSlotState::Approved` and
a wall-clock `responded_at`. Construct-pre-approved is the same pattern
`server::bootstrap::claim` has used since M1 for the genesis
`SystemBootstrap` template; Template E generalises it.

## The helper

[`domain::templates::e::build_auto_approved_request`][2]:

```rust
pub fn build_auto_approved_request(args: BuildArgs) -> AuthRequest
```

`BuildArgs` captures everything a write needs:

- `requestor_and_approver: PrincipalRef` — the self-interested
  principal.
- `resource: ResourceRef` — the specific resource (e.g.
  `secret:anthropic-api-key`, `provider:<id>`).
- `kinds: Vec<String>` — `#kind:` filters the resulting grants should
  carry.
- `scope: Vec<String>` — concrete scope strings.
- `justification: Option<String>` — free-form human explanation,
  surfaced into the audit diff.
- `audit_class: AuditClass` — typically `Alerted` for sensitive
  writes (vault, provider registration); `Logged` for routine ops.
- `now: DateTime<Utc>` — injected clock.

The function is **pure**: no I/O, no random state outside the
generated `AuthRequestId`, no hidden clock. Callers compose
persistence (`create_auth_request` → entity write → `create_grant` →
audit emit) on top. This shape makes the helper trivially
proptest-friendly; [`domain/tests/template_e_props.rs`][3] asserts the
invariants under random inputs.

## Shape of the returned `AuthRequest`

```
AuthRequest {
    id:                      AuthRequestId::new(),
    requestor:               <args.requestor_and_approver>,
    kinds:                   args.kinds,
    scope:                   args.scope,
    state:                   AuthRequestState::Approved,     // terminal
    valid_until:             None,
    submitted_at:            args.now,
    resource_slots: vec![ResourceSlot {
        resource:            args.resource,
        approvers: vec![ApproverSlot {
            approver:        <args.requestor_and_approver>,  // same principal
            state:           ApproverSlotState::Approved,
            responded_at:    Some(args.now),
            reconsidered_at: None,
        }],
        state:               ResourceSlotState::Approved,
    }],
    justification:           args.justification,
    audit_class:             args.audit_class,
    terminal_state_entered_at: Some(args.now),               // drives retention
    archived:                false,
    active_window_days:      30,                             // DEFAULT_ACTIVE_WINDOW_DAYS
    provenance_template:     None,                            // caller sets if needed
}
```

Both the aggregate AR state AND the single slot state enter `Approved`
at construction. The aggregation function ([`auth_requests::state`][4])
treats a fully-Approved slot set as Approved at the aggregate level;
no fix-up is needed.

## Where it's used

- **M2/P4 vault writes** — every `add_secret` call builds a Template E
  AR for the new `secret:<slug>` resource ([`server/src/platform/secrets/add.rs`][5]).
- **M2/P5 model providers** (landing) — every `register_provider` call
  does the same for `provider:<id>`.
- **M2/P6 MCP servers** (landing) — for `mcp:<id>`.
- **M2/P7 platform defaults** (landing) — for the singleton
  `platform-defaults:root` row.

## The replay caveat

M7b's audit-chain replay walks each `AuthRequest`'s slot-by-slot
transition history and re-verifies the aggregation at each step.
Template-E ARs have **no intermediate transitions** — they enter the
terminal state at construction — so the replay must special-case
`provenance_template: TemplateKind::E` ARs. See [Open question Q4
in the archived plan](../../../../plan/build/a6005e06-m2-platform-setup.md)
for the M7b tracking note.

## References

- [ADR-0016 — Template E, self-interested auto-approve](../decisions/0016-template-e-self-interested-auto-approve.md)
- [`../../m1/architecture/auth-request-state-machine.md`](../../m1/architecture/auth-request-state-machine.md)
  — the 9-state machine Template E bypasses by construction.
- [phi-core-reuse-map](phi-core-reuse-map.md) — Template E is baby-phi-only
  (no phi-core counterpart; permission-system concern).

[1]: ../../../../../../modules/crates/domain/src/auth_requests/transitions.rs
[2]: ../../../../../../modules/crates/domain/src/templates/e.rs
[3]: ../../../../../../modules/crates/domain/tests/template_e_props.rs
[4]: ../../../../../../modules/crates/domain/src/auth_requests/state.rs
[5]: ../../../../../../modules/crates/server/src/platform/secrets/add.rs
