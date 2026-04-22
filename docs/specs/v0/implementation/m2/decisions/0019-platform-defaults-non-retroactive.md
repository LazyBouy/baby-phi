<!-- Last verified: 2026-04-21 by Claude Code -->

# ADR-0019 — Platform defaults are non-retroactive

**Status: Accepted** — shipped with M2/P7.

## Context

`PlatformDefaults` is a singleton row that holds platform-wide baselines
for four phi-core types (`ExecutionLimits`, `AgentProfile`,
`ContextConfig`, `RetryConfig`) plus two phi-native fields
(`default_retention_days`, `default_alert_channels`). Every newly
created org consumes these at creation time as the starting point
for its own governance config.

The question: when an admin edits `PlatformDefaults`, do existing
orgs adopt the new values?

Two options:

1. **Retroactive.** Every org's effective config is a live lookup
   against the current singleton. Edit propagates immediately.
2. **Non-retroactive (chosen).** Each org snapshots the defaults at
   creation time and stores its own copy. Edits affect only orgs
   created after the write.

## Decision

**Option 2 — non-retroactive.** `PUT /api/v0/platform/defaults` mutates
only the `platform_defaults` singleton row. It does **not** touch any
per-org state. M3's org-creation wizard (not yet shipped) will
snapshot the current `PlatformDefaults` into an `OrganizationDefaults`
node at the moment the org is provisioned; existing orgs keep their
snapshot untouched across future edits.

## Consequences

### Pros

- **Predictability.** An operator editing the defaults can reason
  locally — no hidden cascade across the platform. An org's effective
  config is a property of the org, not a property of the current
  platform state.
- **Audit clarity.** The `platform.defaults.updated` event carries the
  old + new platform-level snapshot. Each affected org's creation
  event independently carries the snapshot that org consumed.
  Two orthogonal trails, no tangling.
- **Rollback safety.** An operator can tighten a default (e.g.
  `execution_limits.max_turns = 20`) and be certain no live agent
  session suddenly trips the cap mid-flight.

### Cons

- **No "push to all orgs."** An operator who genuinely wants to
  upgrade every org's baseline has to iterate orgs explicitly (M3+
  surface). That's the right tradeoff — explicit is safer than
  implicit at the platform level.
- **Two sources of default truth.** `PlatformDefaults` (platform-
  wide) and each org's snapshot are independent after creation.
  Reviewers must know which one they're inspecting. The M3 wizard
  mitigates this with a "your org's snapshot taken from platform
  defaults vN on 2026-04-21" trail.

### Enforcement

The invariant is verified structurally + via proptest:

- **Structural:** the P7 `put_platform_defaults` handler writes
  exclusively to the `platform_defaults` singleton row. No other
  table is touched on the write path.
- **Proptest:** [`platform_defaults_non_retroactive_props`](../../../../../../modules/crates/domain/tests/platform_defaults_non_retroactive_props.rs)
  generates a random graph (N orgs + M agents), PUTs the defaults
  with random field mutations, and asserts every pre-existing org /
  agent row is byte-identical after. 32 cases per run.

### Alternatives considered

- **Retroactive with explicit "pin" flag.** Each org would opt into
  snapshot behaviour at creation. Rejected: opt-in semantics for
  safety invariants are bug-prone (the safe default must be safe).
- **Live override layering.** Each org's config is the singleton
  merged with org-specific overrides. Rejected: the merge semantics
  are subtle (which side wins for nested struct fields? Override
  everything or only listed keys?) and would tangle audit trails.

## See also

- [`../architecture/platform-defaults.md`](../architecture/platform-defaults.md) — how the singleton + snapshot model fits into M2.
- [`../operations/platform-defaults-operations.md`](../operations/platform-defaults-operations.md) — day-to-day operator runbook, including the stale-write 409 recovery flow.
- Business logic: [`modules/crates/server/src/platform/defaults/put.rs`](../../../../../../modules/crates/server/src/platform/defaults/put.rs).
- Proptest: [`modules/crates/domain/tests/platform_defaults_non_retroactive_props.rs`](../../../../../../modules/crates/domain/tests/platform_defaults_non_retroactive_props.rs).
