<!-- Last verified: 2026-04-22 by Claude Code -->

# ADR-0020 — OrganizationDefaults embed on the Organization node

**Status: Accepted** — landed in M3/P1.

## Context

M2/P7 shipped `PlatformDefaults` — a singleton row holding platform-
wide baselines for `ExecutionLimits`, `AgentProfile`, `ContextConfig`,
`RetryConfig` plus two baby-phi-native fields. ADR-0019 pinned the
non-retroactive invariant: a later PUT does **not** mutate existing
orgs; each org freezes its own copy at creation time.

M3 needs a place to store that per-org frozen snapshot. Two shapes
were considered:

1. **Sibling composite `OrganizationDefaults`** — a `control_plane_
   object` child with its own `HAS_DEFAULTS` edge to the Organization.
   Independent NodeId, independent lifecycle, independent audit-event
   stream.
2. **Embedded field on the `Organization` node** —
   `defaults_snapshot: Option<OrganizationDefaultsSnapshot>` where
   `OrganizationDefaultsSnapshot` is a plain struct (not a node) in
   `domain::model::composites_m3`.

## Decision

**Option 2 — embed on the Organization node.**
[`Organization`](../../../../../../modules/crates/domain/src/model/nodes.rs)
carries `defaults_snapshot: Option<OrganizationDefaultsSnapshot>` as
a first-class field. The snapshot struct wraps four phi-core types
directly (same pattern as
[`PlatformDefaults`](../../../../../../modules/crates/domain/src/model/composites_m2.rs))
and has no independent NodeId / edge / audit-event stream.

## Consequences

### Pros

- **Matches non-retroactive semantics.** The snapshot is logically
  part of the org's identity at creation time — not an
  independently-mutating subtree. Embedding makes "frozen"
  structural rather than policy-maintained.
- **No sibling-lifecycle ambiguity.** A sibling composite would
  raise questions the M3 scope can't answer: does archiving an
  Organization archive its defaults? Can an operator edit the
  snapshot without editing the org? Can two orgs share a
  snapshot? All three are `No` under ADR-0019, which is easier to
  enforce when the snapshot is a field than when it is a node.
- **Matches R-ADMIN-06-W2's wire shape.** The wizard's POST body
  puts defaults inline; the handler deserialises straight into the
  embedded field. A sibling composite would need a join at
  creation time + every subsequent read.
- **Zero new tables.** Migration 0003 extends `organization` with
  the field as `FLEXIBLE TYPE option<object>`. Sibling composite
  would need a full table + UNIQUE INDEX on `owning_org` +
  referential-integrity trigger.

### Cons

- **Slightly wider Organization row.** Every org read pays the
  (small) cost of loading the snapshot even when the caller doesn't
  need it. Mitigated: the snapshot is a few hundred bytes; the
  dashboard already reads the org row for other fields.
- **Audit-event granularity** — a future "defaults-only" edit
  surface (e.g. M7+ "refresh org from current platform defaults")
  would need a custom event type rather than a `CompositeUpdated`
  shape. Deferred: M3's non-retroactive semantics explicitly
  forbid that edit path, so the concern does not materialise
  until the invariant is loosened.

## phi-core leverage

The snapshot struct wraps four phi-core types directly:
`phi_core::context::execution::ExecutionLimits`,
`phi_core::agents::profile::AgentProfile`,
`phi_core::context::config::ContextConfig`, and
`phi_core::provider::retry::RetryConfig`. No parallel baby-phi
layer — same pattern M2/P7 established for `PlatformDefaults`.
`OrganizationDefaultsSnapshot::from_platform_defaults` is a
field-wise copy; phi-core sub-fields ride through verbatim. Baby-phi
adds only `default_retention_days` + `default_alert_channels`
(governance concerns with no phi-core counterpart).

## References

- [`../../../plan/build/563945fe-m3-organization-creation.md`](../../../../plan/build/563945fe-m3-organization-creation.md) §G2 / §D1.
- [`../../m2/decisions/0019-platform-defaults-non-retroactive.md`](../../m2/decisions/0019-platform-defaults-non-retroactive.md) — the invariant this ADR implements per-org.
- [`../../../../../../modules/crates/domain/src/model/composites_m3.rs`](../../../../../../modules/crates/domain/src/model/composites_m3.rs) — the struct definition.
- [`../../../../../../modules/crates/domain/src/model/nodes.rs`](../../../../../../modules/crates/domain/src/model/nodes.rs) — `Organization.defaults_snapshot` field.
