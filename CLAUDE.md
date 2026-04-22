# CLAUDE.md — phi

phi is the agent management platform that consumes `phi-core` as a library. As of M0 it is a **Cargo workspace** with three parallel surfaces (CLI, HTTP API, Next.js web UI) sharing one domain layer.

## Workspace layout

```
phi/                       (workspace root — virtual Cargo.toml)
├── modules/
│   ├── crates/
│   │   ├── cli/                CLI binary (clap). Package `cli`, binary `phi`.
│   │   ├── domain/             Graph model + Permission Check + state machines.
│   │   ├── store/              SurrealDB (embedded, RocksDB) adapter.
│   │   └── server/             axum HTTP API. Package `server`, binary `phi-server`.
│   └── web/                    Next.js 14 (App Router + SSR).
├── config/                     Layered TOML configs (default + dev/staging/prod).
├── docs/specs/                 Concepts + requirements (source of truth for v0).
├── docs/specs/plan/            Plan archives (git-ignored by convention).
├── scripts/                    Ops helpers (spec-drift check, …).
├── Dockerfile                  Multi-stage build for phi-server.
├── docker-compose.yml          Local dev stack (server + web).
└── deny.toml                   cargo-deny policy.
```

Dependency flow (strict, downward):
```
cli              ┐
server          ─┼─▶ domain ─▶ store ─▶ SurrealDB
web (Next.js)  ──┘             (plus phi-core for agent/session types)
```

Package names are deliberately terse (`cli`, `domain`, `store`, `server`); the shipped binary names keep the product prefix (`phi`, `phi-server`) via explicit `[[bin]] name`.

## Build & Run

All cargo commands must use `/root/rust-env/cargo/bin/cargo`. CI enforces `RUSTFLAGS="-Dwarnings"`.

```bash
# From phi/ (the workspace root):
/root/rust-env/cargo/bin/cargo build --workspace
/root/rust-env/cargo/bin/cargo fmt --all -- --check
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy --workspace --all-targets
/root/rust-env/cargo/bin/cargo test --workspace

# Run the HTTP server (reads config/default.toml + config/<profile>.toml +
# PHI_* env vars; see modules/crates/server/src/config.rs):
PHI_PROFILE=dev /root/rust-env/cargo/bin/cargo run -p server

# Run the existing CLI demo (still reads phi/config.toml):
set -a && source .env && set +a
/root/rust-env/cargo/bin/cargo run -p cli

# Web UI (from phi/modules/web/):
npm install && npm run dev
```

## Scope

Every platform-level feature sits in this workspace — phi-core stays a pure agent-loop library. Features tracked in the v0.1 build plan (`docs/specs/plan/build/`):

- Permission Check engine + Auth Request state machine + graph model (M1).
- Admin pages 01–14 as CLI + API + Web UI vertical slices (M2–M5).
- Agent self-service surfaces a01–a05 (M6).
- System flows s02–s06 (M7).
- Production hardening — OAuth 2.0, TLS, at-rest encryption, backup/restore, OpenTelemetry, rate limiting, GDPR erasure, runbook (M7b).

## phi-core Leverage (first-class mandate)

phi is a **consumer** of phi-core, not a parallel implementation. Every surface that overlaps with an existing `phi_core::` type MUST reuse it directly or wrap it — **never re-implement**. This is not a style preference; it is a two-source-of-truth problem that compounds per milestone.

**Rules of engagement:**

1. **Before introducing any struct/enum/trait** whose shape overlaps with something in phi-core, check `phi-core/src/` first. If phi-core ships it, import it.
   - Direct reuse (`use phi_core::X`) — use phi-core's type as-is.
   - Wrap (`pub struct Y { inner: phi_core::X, ... }`) — extend with phi-only governance fields.
   - Build from scratch — only if phi-core has **no** counterpart (e.g., permission-check engine, credentials vault, tenant sets, audit hash-chain).

2. **Known reuse surfaces** (non-exhaustive; see `docs/specs/v0/concepts/phi-core-mapping.md` for the full list):
   - Config / agent blueprint → `phi_core::agents::profile::AgentProfile`, `phi_core::config::{parser, schema}`.
   - Providers → `phi_core::provider::{model::ModelConfig, model::ApiProtocol, registry::ProviderRegistry, traits::{StreamProvider, StreamConfig, StreamEvent}, retry::RetryConfig}`.
   - Tools → `phi_core::types::tool::{AgentTool, ToolResult}`, `phi_core::mcp::{client::McpClient, types::*, tool_adapter::McpToolAdapter}`.
   - Execution / context → `phi_core::context::{execution::ExecutionLimits, ContextConfig, CompactionStrategy}`, `phi_core::types::usage::{CacheConfig, ThinkingLevel}`.
   - Sessions / events → `phi_core::types::event::AgentEvent`, `phi_core::session::model::{Session, LoopRecord, Turn, LoopStatus}`, `phi_core::session::recorder::SessionRecorder`.

3. **Enforcement.** `scripts/check-phi-core-reuse.sh` runs in CI and fails on forbidden duplications (e.g., any `struct ExecutionLimits`, `struct ModelConfig`, `struct McpClient`, `struct AgentProfile`, `struct AgentEvent`, `struct Session`, etc. defined anywhere under `modules/crates/`). The lint is advisory during a milestone's foundation phase and flips to hard-gate at the next re-audit.

4. **Reviewer checklist.** In code review, reject any PR that introduces a new type whose field set matches a phi-core type; require the phi-core import instead. When in doubt, consult the phi-core-reuse-map doc for the milestone.

5. **`thiserror` must track phi-core's version** (currently `"2"`). Version drift breaks `#[from]` conversions at runtime with cryptic "implementations not found" errors.

**Orthogonal surfaces that are NOT phi-core duplicates** (these are intentionally phi-only — do not conflate):
- `domain::audit::AuditEvent` (governance write log, hash-chain, retention tier) vs `phi_core::types::event::AgentEvent` (agent-loop telemetry stream) — see `implementation/m1/architecture/audit-events.md`.
- `server::session::SessionClaims` (HTTP cookie JWT) vs `phi_core::session::Session` (persisted execution trace) — see `implementation/m1/architecture/server-topology.md`.
- `domain::model::ToolDefinition` (permission metadata node) vs `phi_core::types::tool::AgentTool` (runtime trait) — see `implementation/m1/architecture/graph-model.md`.
- `server::config::ServerConfig` (HTTP infrastructure TOML) vs `phi_core::config::schema::AgentConfig` (agent blueprint YAML/TOML/JSON with `${VAR}`) — see `implementation/m1/architecture/overview.md`.

When the line is unclear, err toward reuse and ask in review.

## Documentation Alignment

Documentation in `docs/` must accurately reflect the current codebase at all times. Code is always the source of truth.

- **Update docs with code changes**: When modifying code, update all affected documentation in the same commit. This includes status tags, API signatures, config examples, and pseudocode.
- **Status tags**: `[EXISTS]` = implemented in code, `[PLANNED]` = designed but not yet implemented, `[CONCEPTUAL]` = idea stage. Review and update these tags whenever the referenced code changes.
- **Verification header**: Every doc file carries `<!-- Last verified: YYYY-MM-DD by Claude Code -->` at the top, updated on each review pass.
- **No forward references**: Do not document features as existing unless the code is merged. Use `[PLANNED]` or `[CONCEPTUAL]` for future work.
- **Spec-drift guard**: `scripts/check-spec-drift.sh` runs in CI. If a requirement id (`R-ADMIN-*`, `R-AGENT-*`, `R-SYS-*`, `R-NFR-*`) referenced in code disappears from `docs/specs/v0/requirements/`, CI fails.

## Working Discipline

- **Thoroughness over speed.** When a choice exists between "faster" and "more thorough," always pick thorough. Applies to audits, test coverage, documentation, refactors, and milestone execution. Speed is cheap to regain; shortcuts compound into debt.
- **Phase-by-phase review.** For multi-phase milestones (M1+), pause at each phase boundary for a thorough self-review against the milestone's verification matrix before opening the next phase. Don't chain phases autonomously.
- **Pre-implementation audits.** Before starting a milestone, run a gap-audit against every concept doc, requirement, and production-readiness commitment it touches. Surface deltas (stale counts, implicit assumptions, missing pieces) in the plan rather than discovering them during implementation.
