<!-- Last verified: 2026-04-21 by Claude Code -->

# Architecture — Platform Defaults (page 05)

**Status: [EXISTS]** — shipped with M2/P7.

`PlatformDefaults` is the singleton row of platform-wide governance
baselines. Every M2 admin surface bar this one manages a collection
of rows (providers, MCP servers, secrets); page 05 manages exactly
one row — the platform's "starting point" for every new org.

## Anatomy

The struct (composed in [`composites_m2.rs`](../../../../../../modules/crates/domain/src/model/composites_m2.rs)) has 6 substantive fields + 3 metadata fields:

| Field | Type | Source |
|---|---|---|
| `execution_limits` | `phi_core::context::execution::ExecutionLimits` | **phi-core** — wrapped directly |
| `default_agent_profile` | `phi_core::agents::profile::AgentProfile` | **phi-core** — wrapped directly |
| `context_config` | `phi_core::context::config::ContextConfig` | **phi-core** — wrapped directly |
| `retry_config` | `phi_core::provider::retry::RetryConfig` | **phi-core** — wrapped directly |
| `default_retention_days` | `u32` | baby-phi-only (audit retention policy) |
| `default_alert_channels` | `Vec<String>` | baby-phi-only (alert delivery) |
| `singleton` | `u8` always `1` | baby-phi — enforces table uniqueness |
| `updated_at` | `DateTime<Utc>` | server-stamped on every PUT |
| `version` | `u64` | server-bumped on every PUT (OCC) |

Four of the six substantive fields are **direct phi-core wraps**, so
a phi-core bump flows through baby-phi without a migration. The
`platform_defaults` SurrealDB table declares those columns as
`FLEXIBLE TYPE object` — added phi-core fields are absorbed
transparently.

The two baby-phi-native fields carry concerns phi-core has no
counterpart for: audit-log retention tiers and alert channels are
platform-governance mechanisms, not agent-loop primitives.

## Storage

A single `platform_defaults` SurrealDB row with a `UNIQUE INDEX` on
the constant `singleton = 1` column (plan §G11 / §D5). At most one
row ever exists — the UNIQUE INDEX enforces this at the DB layer,
not via application-level locking.

```
                                  ┌─────────────────────────────┐
                                  │  platform_defaults          │
                                  │    (UNIQUE INDEX singleton) │
                                  │                             │
  PUT /api/v0/platform/defaults ─▶│  singleton=1                │
                                  │  execution_limits: <phi-core> │
                                  │  default_agent_profile: … │
                                  │  context_config: …        │
                                  │  retry_config: …          │
                                  │  default_retention_days: …│
                                  │  default_alert_channels: …│
                                  │  updated_at: …              │
                                  │  version: monotonic         │
                                  └─────────────────────────────┘
```

On a fresh install the table is empty. The GET handler synthesises a
response from `PlatformDefaults::factory(now)` with
`persisted = false`, so the web UI / CLI always have a concrete
starting point.

## Optimistic concurrency

The PUT handler enforces optimistic concurrency (plan §D5):

1. Read the current row (or treat as `version = 0` when empty).
2. Reject with `409 PLATFORM_DEFAULTS_STALE_WRITE` if
   `input.if_version != current.version`. The error message carries
   the current version so the client can re-read and retry.
3. On match: bump `version + 1`, stamp `updated_at = now`, persist.

Two admins editing in parallel are serialised safely — the second
PUT sees a version mismatch, re-reads, and resubmits on top of the
first's changes. No lost updates.

## Non-retroactive invariant (ADR-0019)

**PUT affects only the singleton.** Existing orgs keep their snapshot
untouched. M3's org-creation wizard snapshots the current
`PlatformDefaults` into an `OrganizationDefaults` node at provisioning
time.

Enforced structurally (the handler writes only to `platform_defaults`)
and verified by [`platform_defaults_non_retroactive_props`](../../../../../../modules/crates/domain/tests/platform_defaults_non_retroactive_props.rs) (32 cases
per run).

## Catalogue + Permission Check

The PUT handler seeds a catalogue entry at `platform_defaults:singleton`
under `Composite::ControlPlaneObject` (idempotent — every PUT re-seeds).
This lets Permission-Check Step 0 resolve URI filters that target the
singleton, even though M2's PUT doesn't itself invoke Permission Check
(self-approved Template-E pattern like every other M2 admin write).

## Template E audit chain

Every PUT creates a Template-E Auth Request (self-approved
platform-admin write) and emits one `platform.defaults.updated`
(Alerted) event. The event diff carries the full before + after
snapshot — embedded phi-core serde is the single source of truth,
so a reviewer sees exactly which phi-core field changed across the
revision.

## phi-core leverage notes

- **Wrapped fields.** `ExecutionLimits`, `AgentProfile`,
  `ContextConfig`, `RetryConfig` — all imported directly; no parallel
  baby-phi struct.
- **Factory baseline.** `PlatformDefaults::factory()` uses each
  phi-core type's `Default::default()`. Bumping phi-core
  automatically bumps baby-phi's factory.
- **Multi-format config parser.** phi-core's
  `phi_core::config::parser::parse_config` is scoped to the
  `AgentConfig` schema — a different envelope. P7 deliberately does
  **not** reach through that parser for YAML/TOML support on the
  server side. The CLI uses `serde_yaml` / `toml` directly on the
  same `PlatformDefaults` struct, which gives us multi-format
  ergonomics without duplicating phi-core's parser logic. The reuse
  boundary is the embedded phi-core types, not the wrapper parser.

## See also

- [`../operations/platform-defaults-operations.md`](../operations/platform-defaults-operations.md) — day-to-day runbook + stale-write recovery.
- [`../user-guide/platform-defaults-usage.md`](../user-guide/platform-defaults-usage.md) — operator-facing walkthrough (CLI + web).
- [`../decisions/0019-platform-defaults-non-retroactive.md`](../decisions/0019-platform-defaults-non-retroactive.md) — the ADR.
- Handler: [`modules/crates/server/src/handlers/platform_defaults.rs`](../../../../../../modules/crates/server/src/handlers/platform_defaults.rs).
- Business logic: [`modules/crates/server/src/platform/defaults/`](../../../../../../modules/crates/server/src/platform/defaults/).
- Audit builder: [`modules/crates/domain/src/audit/events/m2/defaults.rs`](../../../../../../modules/crates/domain/src/audit/events/m2/defaults.rs).
