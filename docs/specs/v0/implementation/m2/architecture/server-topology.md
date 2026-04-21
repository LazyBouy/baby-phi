<!-- Last verified: 2026-04-21 by Claude Code -->

# Architecture — server topology (M2 extension)

**Status: [PLANNED M2/P3–P7]**

Extends the M1 route table with the `/api/v0/platform/*` nested routers
(secrets, model-providers, mcp-servers, platform-defaults). Each page's
routes land alongside the page's vertical slice.

See also:
- [`../../m1/architecture/server-topology.md`](../../m1/architecture/server-topology.md)
  — the M1 skeleton this extends.
- [handler-support.md](handler-support.md) — the shared shim every
  route handler uses.
