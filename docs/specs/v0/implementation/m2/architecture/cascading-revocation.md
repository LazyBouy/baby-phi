<!-- Last verified: 2026-04-21 by Claude Code -->

# Architecture — cascading tenant revocation (page 03)

**Status: [PLANNED M2/P6]**

MCP tenant-narrowing semantics: when `TenantSet::Only` shrinks, every
grant descending from an AR against a dropped org is forward-only
revoked in a single transaction. Fleshed out in P6.

See also:
- [overview.md](overview.md)
- [phi-core-reuse-map.md](phi-core-reuse-map.md) — `TenantSet` is
  baby-phi-only; `McpClient` is reused from phi-core for the probe path.
