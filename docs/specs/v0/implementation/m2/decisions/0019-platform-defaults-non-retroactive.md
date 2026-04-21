<!-- Last verified: 2026-04-21 by Claude Code -->

# ADR-0019 — Platform defaults are non-retroactive

**Status: [PLANNED M2/P7]** — proposed, pending P7 implementation.

## Context

`PlatformDefaults` is a singleton; edits could in principle apply
retroactively to every existing org. That would make
"defaults-at-creation-time" a pseudo-contract — any edit silently
rewrites every org's effective config.

## Decision

PUTs on `platform_defaults` affect only orgs created after the write.
Existing orgs carry a snapshot taken at their own creation time
(`OrganizationDefaults` — M3 will wire it).

## Consequences

- The invariant is verified by a proptest
  (`platform_defaults_non_retroactive_props`) that generates random
  PUTs and confirms pre-existing org rows are byte-identical after.
- M3's org-creation wizard must snapshot the current
  `PlatformDefaults` before the first agent in the org is provisioned.

Full ADR lands with P7.
