<!-- Last verified: 2026-04-19 by Claude Code -->

# ADR-0002: Ship CLI + HTTP API + Web UI in parallel

## Status
Accepted — 2026-04-19 (M0).

## Context

baby-phi needs to be operable by humans (administrators, agent leads) and scriptable by automation (CI, fleet management, SDK users). Early platform projects often punt this choice: build one surface first, add others later. That leaves whichever audience wasn't chosen as the first-class target with a degraded experience for months.

The audit journey through the 14 admin pages + 5 agent self-service surfaces + 6 system flows includes operations that are natural for humans (approving an Auth Request, reviewing an org dashboard) and operations that are natural for scripts (reconciling a fleet of agents, CI-provisioning a tenant).

## Decision

**Ship three surfaces from M0 onward, all consuming the same REST API:**

1. **Rust CLI** (`cli` crate, binary `baby-phi`) — the scriptable surface.
2. **Rust HTTP API** (`server` crate, binary `baby-phi-server`) — the single source of truth for platform state; all reads and writes flow through it.
3. **Next.js 14 web UI** (`modules/web/`) — the human surface.

The API is the **only** source of truth. The CLI and web UI are both clients of the API; neither reaches into the database directly.

## Consequences

### Positive

- **Parity by construction.** Anything the web UI can do, the CLI can do — because both speak the same REST contract. Automation never sees a "web-only" feature that blocks it.
- **Single security boundary.** Authn/authz (arriving in M3) lives in the API. CLI users, web users, and LLM agents all authenticate against the same surface and are subject to the same Permission Check.
- **Testability.** Acceptance tests drive the CLI (trivial to script) and assert on API-surface state. No browser automation required for the fresh-install journey's acceptance suite (M1–M5).
- **Deployment flexibility.** Single-tenant deploys can ship the server only; web UI is optional for headless installs.

### Negative

- **Three things to keep in sync.** Every new API endpoint needs corresponding CLI subcommand(s) and web page(s). The vertical-slice milestone structure (M2–M5) enforces this by tying all three to the admin-page boundaries of the fresh-install journey.
- **More code surface.** Three codebases instead of one. Mitigated by: the CLI is tiny (each subcommand is a thin shell over a single API call), and the web UI is a typical Next.js app. The volume is a small multiple, not a linear tripling.
- **Auth on three surfaces.** M3 must handle session cookies (web), machine tokens (CLI + LLM agents), and the OAuth PKCE flow (human login). All three share the server's session store — the cost is in M3's implementation effort, not in ongoing complexity.

## Alternatives considered

- **Web UI first, CLI later.** Standard for consumer SaaS. Rejected because baby-phi's early adopters are platform teams who expect `cli` + scripts to be first-class. A web-only MVP would block the audience we're building for.
- **CLI first, web UI later.** Common for dev-tools startups. Rejected because the fresh-install journey includes human-facing approvals (Auth Requests, consent ceremonies) that are painful in a CLI. We need the web UI ready when those ceremonies appear (M3+).
- **gRPC-first with optional REST gateway.** Attractive for internal-service workloads. Rejected for v0.1: browser clients need REST (or GraphQL), the public-API contract needs to be inspectable with `curl`, and we have no near-term need for bidirectional streaming.
- **One Rust binary that serves both API + CLI in-process.** The CLI would short-circuit the HTTP layer and call domain functions directly. Rejected because it creates a "special" code path the web UI can't use, and integration tests can't easily exercise both paths symmetrically.

## How this appears in the code

- `server` crate owns every handler. Its Cargo.toml has `axum` + route definitions in [`server/src/router.rs`](../../../../../../modules/crates/server/src/router.rs).
- `cli` crate (M1+) will use `clap` for arg parsing and `reqwest` for HTTP calls to the local server. The M0 CLI is the legacy phi-core demo; rebuilding it as a clap-structured command tree is M1 work.
- `modules/web/` proxies `/api/v0/*` to the server via `next.config.mjs` — see [`../architecture/web-topology.md`](../architecture/web-topology.md).

No crate or page ever bypasses the API.
