<!-- Last verified: 2026-04-21 by Claude Code -->

# ADR-0016 — Template E: self-interested auto-approve

**Status: Accepted** — shipped in M2/P2; first production use in M2/P4
(credentials vault). Every subsequent M2 page vertical (P5 model
providers, P6 MCP servers, P7 platform defaults) uses the same
pattern.

## Context

Every M2 admin write on pages 02–05 has the platform admin as both
the **requestor** and the **approver** of the write. Without a
dedicated pattern, each handler would hand-roll an Auth Request,
drive its lone approver slot through `Pending → Filled → Approved`,
and separately assert the aggregate state flipped to `Approved`.
With four pages × roughly five writes each, that's ~20 copies of
the same five-line dance, each a potential place to drift off-spec.

The underlying state-machine constraint: M1's legal-transition
table ([`auth_requests/transitions.rs`][1]) has no
`ApproverSlotState::Unset → Approved` edge. The intermediate
`Pending` / `Filled` states exist precisely so the aggregation
function observes each step; a direct jump to `Approved` would
skip them. When the requestor and approver are the same principal,
those steps provide no information.

Bootstrap (M1's `server::bootstrap::claim`) solved the same problem
for the `SystemBootstrap` template by **constructing** the Auth
Request already in `Approved` state rather than driving it. M2 has
four more templates with the same shape.

## Decision

Ship a named template kind + pure helper in the domain crate.

1. Extend [`TemplateKind`][2] with variants `A`, `B`, `C`, `D`, `E`,
   `F` alongside the pre-existing `SystemBootstrap`. `E` is the
   self-interested auto-approve pattern; the other letters reserve
   their numbering for M3+ templates per the concept doc.
2. Add [`domain::templates::e::build_auto_approved_request`][3], a
   pure function that returns an `AuthRequest` in state
   `Approved` with exactly one `ResourceSlot` holding exactly one
   already-Approved `ApproverSlot` whose `approver` equals the
   caller's `requestor_and_approver` argument.
3. Callers compose the atomic write on top: `create_auth_request →
   put_<entity> → seed_catalogue_entry → create_grant →
   emit_audit`.

The helper is pure — no I/O, no hidden clock (caller injects
`now`), no random state outside the newly-minted `AuthRequestId`.
This is what unlocks the proptest surface in
[`domain/tests/template_e_props.rs`][4] (invariants over random
requestors, resources, scopes, audit classes).

## Consequences

- **Single source of truth** for the self-interested pattern. Drift
  across the four M2 page handlers is structurally impossible — they
  all call the same helper.
- **Aggregation function stays honest.** The aggregate state is
  `Approved` at construction because the single slot is `Approved`
  at construction; the aggregation function never sees a partial
  state. Verified by P2's aggregation proptests continuing to pass
  under the new helper.
- **Replay special-case at M7b.** The hash-chain replay walks each
  AR's slot-transition history. Template-E ARs have no intermediate
  transitions, so M7b's replay must special-case
  `provenance_template: TemplateKind::E` — detect the absent
  history and skip transition-replay for those records. Tracked as
  Q4 in the archived plan's Part 11.
- **Provenance linkage is explicit.** Handlers that want the AR to
  carry a `provenance_template` pointer set it after
  `build_auto_approved_request` returns. The helper leaves the
  field `None` so pure tests don't need to materialize Template
  graph nodes.

## Alternatives considered

1. **Extend `transition_slot` with a template-gated `Unset →
   Approved` edge.** Rejected — invasive for a platform-admin-only
   case. Would require every caller of `transition_slot` to
   discriminate on template kind, polluting the state machine with
   an edge that's semantically a bypass rather than a transition.
2. **Construct inline at each handler site (as bootstrap does).**
   Rejected — bootstrap is one site, M2 is four. Four copies drift
   the moment someone forgets to copy a field.
3. **Make the helper async and write-through.** Rejected — the pure
   helper is trivially proptest-friendly; making it async would mean
   every proptest needs a test repo. The I/O composition is
   lightweight enough in handlers that the split is worth it.

## Implementation pointer

- Pure helper: [`domain::templates::e::build_auto_approved_request`][3].
- Template kind extension: [`TemplateKind::E`][2].
- First production use: [`server::platform::secrets::add::add_secret`][5].
- Proptest invariants: [`domain/tests/template_e_props.rs`][4].

[1]: ../../../../../../modules/crates/domain/src/auth_requests/transitions.rs
[2]: ../../../../../../modules/crates/domain/src/model/nodes.rs
[3]: ../../../../../../modules/crates/domain/src/templates/e.rs
[4]: ../../../../../../modules/crates/domain/tests/template_e_props.rs
[5]: ../../../../../../modules/crates/server/src/platform/secrets/add.rs
