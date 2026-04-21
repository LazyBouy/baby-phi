<!-- Last verified: 2026-04-21 by Claude Code -->

# Architecture — platform defaults (page 05)

**Status: [PLANNED M2/P7]**

The singleton `platform_defaults` table + non-retroactive inheritance
invariant (PUT never mutates existing orgs). Fleshed out in P7. ADR:
[0019-platform-defaults-non-retroactive.md](../decisions/0019-platform-defaults-non-retroactive.md).

See also:
- [phi-core-reuse-map.md](phi-core-reuse-map.md) — the `PlatformDefaults`
  container is baby-phi-only but every nested field wraps a phi-core type.
