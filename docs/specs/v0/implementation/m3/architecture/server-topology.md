<!-- Last verified: 2026-04-22 by Claude Code -->

# Architecture — Server topology (M3 extension)

**Status: [PLANNED M3/P4-P5]** — flipped to [EXISTS] at P5 close.

Extends the M2 route table with the `/api/v0/orgs/*` surface. The
canonical route list lives in the docstring on
[`server::router::build_router`](../../../../../../modules/crates/server/src/router.rs).

Expected M3 additions:

| Method | Path | Phase |
|---|---|---|
| `POST` | `/api/v0/orgs` | M3/P4 (wizard submit) |
| `GET`  | `/api/v0/orgs` | M3/P4 (list) |
| `GET`  | `/api/v0/orgs/:id` | M3/P4 (show) |
| `GET`  | `/api/v0/orgs/:id/dashboard` | M3/P5 (dashboard) |

See:
- [`../../m2/architecture/server-topology.md`](../../m2/architecture/server-topology.md) — M2 routes this extends.
- [`handler-support.md`](../../m2/architecture/handler-support.md) — the shared handler shim every new route uses.
