<!-- Last verified: 2026-04-21 by Claude Code -->

# Architecture — server topology (M2 extension)

**Status: [EXISTS]** — shipped incrementally across M2/P3–P7.

Extends the M1 route table with the `/api/v0/platform/*` routes. All
four admin surfaces (secrets, model-providers, mcp-servers,
platform-defaults) landed in their vertical-slice phases (P4–P7) on
top of the shared `handler_support` shim introduced in P3. The
canonical route list is the docstring on
[`server::router::build_router`](../../../../../../modules/crates/server/src/router.rs).

See also:
- [`../../m1/architecture/server-topology.md`](../../m1/architecture/server-topology.md)
  — the M1 skeleton this extends.
- [handler-support.md](handler-support.md) — the shared shim every
  route handler uses.
