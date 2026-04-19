<!-- Last verified: 2026-04-19 by Claude Code -->

# ADR-0003: `modules/crates/` + `modules/web/` directory split

## Status
Accepted — 2026-04-19 (M0; restructured mid-M0 after initial flat layout).

## Context

M0's initial scaffolding placed each workspace member at the baby-phi repo root:

```
baby-phi/
├── baby-phi-cli/
├── baby-phi-domain/
├── baby-phi-store/
├── baby-phi-server/
├── web/
├── Cargo.toml
└── …
```

This is the Cargo-idiomatic flat layout for small workspaces, and it works. But it mixes concerns at the repo root: buildable code sits next to ops artefacts (Dockerfile, docker-compose.yml, deny.toml), config (config/), docs (docs/), and scripts. As the project grows past v0.1, more ops and docs content will accumulate at the root — visually the code becomes harder to find.

The user asked for a two-level group:

1. A single top-level folder holding **every buildable module** (all Rust crates + the Next.js app).
2. Inside that, a subfolder for the Rust crates specifically, separate from the web tree.

## Decision

**Group buildable modules under `modules/`, with Rust crates under `modules/crates/` and the Next.js app under `modules/web/`.**

```
baby-phi/
├── modules/
│   ├── crates/
│   │   ├── cli/
│   │   ├── domain/
│   │   ├── store/
│   │   └── server/
│   └── web/
├── config/
├── docs/
├── scripts/
├── Cargo.toml   ← workspace root stays here
└── …
```

The workspace manifest at [`Cargo.toml`](../../../../../../Cargo.toml) declares members as `"modules/crates/cli"` etc. The `phi-core` path dep remains `../phi-core` (relative to the workspace root).

## Consequences

### Positive

- **Root stays scannable.** Code lives in `modules/`; ops + config + docs are visually separate at the root.
- **Extensible.** Future additions (a Python analysis tool, a mobile app, a WASM plugin tree) slot under `modules/` as siblings. No repo-root churn needed.
- **Matches monorepo conventions.** `packages/` (Nx, Lerna), `apps/` and `libs/` (Nx) — common shapes. `modules/` captures both apps and libs under one roof, which matches our mix of CLI/server binaries and library crates.
- **Clear Rust-vs-other split.** `crates/` is the Rust-community-standard name for workspace members. `web/` naming calls out that it's not a Rust crate.

### Negative

- **Slightly deeper paths.** `modules/crates/server/src/router.rs` instead of `server/src/router.rs`. File navigation adds two hops; editor fuzzy-find offsets this cost almost entirely.
- **Migration cost.** Rearranging mid-M0 required updating Cargo.toml member paths, every crate's internal deps, Rust import names (from `baby_phi_X` to `X`), Dockerfile, docker-compose, CI workflows, spec-drift script, CLAUDE.md, and the web rewrite path. Cost paid; not recurring.
- **Cargo path strings** in the workspace manifest are longer (`"modules/crates/cli"` vs `"baby-phi-cli"`).

## Alternatives considered

- **Flat at repo root.** Initial shape. Rejected because it visually crowds the root as the project grows.
- **`modules/cli/`, `modules/domain/`, …, `modules/web/` flat** (no `crates/` subfolder). Rejected because it erases the Rust-vs-not boundary and makes it harder to iterate over just the Rust members (e.g. the spec-drift script would need to whitelist each Rust dir).
- **`crates/`, `apps/`, `libs/` — Nx-style multi-folder.** Rejected as premature differentiation. baby-phi has one kind of Rust code (workspace crate) and one web app; a three-folder split would be cargo-culting. Revisit if the mix grows.
- **Per-surface git submodules** (CLI, server, web as separate repos). Rejected because the domain types are shared and cross-repo refactors are painful. A single repo with clear boundaries beats per-surface repos at this size.

## Follow-up work

- The web tree is inside the Rust repo for now. If the web grows to have its own non-trivial ops surface, it may move to a sibling git repo. The user has acknowledged this as a "move out later if needed" direction; everything web-related is intentionally self-contained under `modules/web/` so that move is a `git mv` + path-update exercise.
