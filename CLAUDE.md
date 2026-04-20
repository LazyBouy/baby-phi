# CLAUDE.md — baby-phi

baby-phi is the agent management platform that consumes `phi-core` as a library. As of M0 it is a **Cargo workspace** with three parallel surfaces (CLI, HTTP API, Next.js web UI) sharing one domain layer.

## Workspace layout

```
baby-phi/                       (workspace root — virtual Cargo.toml)
├── modules/
│   ├── crates/
│   │   ├── cli/                CLI binary (clap). Package `cli`, binary `baby-phi`.
│   │   ├── domain/             Graph model + Permission Check + state machines.
│   │   ├── store/              SurrealDB (embedded, RocksDB) adapter.
│   │   └── server/             axum HTTP API. Package `server`, binary `baby-phi-server`.
│   └── web/                    Next.js 14 (App Router + SSR).
├── config/                     Layered TOML configs (default + dev/staging/prod).
├── docs/specs/                 Concepts + requirements (source of truth for v0).
├── docs/specs/plan/            Plan archives (git-ignored by convention).
├── scripts/                    Ops helpers (spec-drift check, …).
├── Dockerfile                  Multi-stage build for baby-phi-server.
├── docker-compose.yml          Local dev stack (server + web).
└── deny.toml                   cargo-deny policy.
```

Dependency flow (strict, downward):
```
cli              ┐
server          ─┼─▶ domain ─▶ store ─▶ SurrealDB
web (Next.js)  ──┘             (plus phi-core for agent/session types)
```

Package names are deliberately terse (`cli`, `domain`, `store`, `server`); the shipped binary names keep the product prefix (`baby-phi`, `baby-phi-server`) via explicit `[[bin]] name`.

## Build & Run

All cargo commands must use `/root/rust-env/cargo/bin/cargo`. CI enforces `RUSTFLAGS="-Dwarnings"`.

```bash
# From baby-phi/ (the workspace root):
/root/rust-env/cargo/bin/cargo build --workspace
/root/rust-env/cargo/bin/cargo fmt --all -- --check
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy --workspace --all-targets
/root/rust-env/cargo/bin/cargo test --workspace

# Run the HTTP server (reads config/default.toml + config/<profile>.toml +
# BABY_PHI_* env vars; see modules/crates/server/src/config.rs):
BABY_PHI_PROFILE=dev /root/rust-env/cargo/bin/cargo run -p server

# Run the existing CLI demo (still reads baby-phi/config.toml):
set -a && source .env && set +a
/root/rust-env/cargo/bin/cargo run -p cli

# Web UI (from baby-phi/modules/web/):
npm install && npm run dev
```

## Scope

Every platform-level feature sits in this workspace — phi-core stays a pure agent-loop library. Features tracked in the v0.1 build plan (`docs/specs/plan/build/`):

- Permission Check engine + Auth Request state machine + graph model (M1).
- Admin pages 01–14 as CLI + API + Web UI vertical slices (M2–M5).
- Agent self-service surfaces a01–a05 (M6).
- System flows s02–s06 (M7).
- Production hardening — OAuth 2.0, TLS, at-rest encryption, backup/restore, OpenTelemetry, rate limiting, GDPR erasure, runbook (M7b).

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
