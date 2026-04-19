<!-- Last verified: 2026-04-19 by Claude Code -->

# ADR-0004: Package names drop the `baby-phi-` prefix; binary names keep it

## Status
Accepted — 2026-04-19 (M0).

## Context

Rust workspace members initially used fully-qualified names: `baby-phi-cli`, `baby-phi-domain`, `baby-phi-store`, `baby-phi-server`. This is conventional for crates that might someday be published independently and need globally-unique names on crates.io.

Neither condition applies here: these crates are internal to baby-phi and will never be published separately. The prefix is visual noise in every import line and every `cargo run -p …` invocation.

At the same time, the **shipped binary** name matters to operators. `baby-phi-server` is an identifiable process name for `ps`, `systemctl`, Docker image naming, and runbook references. Renaming it to `server` would create instant confusion with the hundreds of other generic "server" processes on any box.

## Decision

**Package names are terse (`cli`, `domain`, `store`, `server`). Binary names keep the product prefix (`baby-phi`, `baby-phi-server`) via explicit `[[bin]] name = "…"` overrides.**

| Crate directory | Package name | Binary name | How |
|---|---|---|---|
| `modules/crates/cli` | `cli` | `baby-phi` | `[[bin]] name = "baby-phi"` in [`cli/Cargo.toml`](../../../../../../modules/crates/cli/Cargo.toml) |
| `modules/crates/domain` | `domain` | — (library only) | — |
| `modules/crates/store` | `store` | — (library only) | — |
| `modules/crates/server` | `server` | `baby-phi-server` | `[[bin]] name = "baby-phi-server"` in [`server/Cargo.toml`](../../../../../../modules/crates/server/Cargo.toml) |

## Consequences

### Positive

- **Rust code reads cleanly.** `use domain::Repository;` beats `use baby_phi_domain::Repository;`. Imports are dense across the codebase; any noise reduction compounds.
- **`cargo run -p server`** is shorter than `cargo run -p baby-phi-server`.
- **Binary name carries product identity** where it matters: process lists, log prefixes, Docker image tags, service registry entries.
- **Operators see `baby-phi-server` in `ps`** — unambiguous for on-call triage.

### Negative

- **Generic package names could clash** with other crates on crates.io if we ever publish (`domain`, `server` are almost certainly already taken). Mitigated by: we won't publish these; they're internal to the baby-phi workspace. If we ever need to publish, renaming is a single find-and-replace.
- **Ambiguity risk when navigating.** "Which `server` crate?" isn't a real question inside baby-phi, but a newcomer glancing at the Cargo.lock might wonder. Mitigated by the workspace layout — all `path` deps point to `modules/crates/…`, and the workspace is local-only.

## Alternatives considered

- **Keep `baby-phi-*` everywhere.** The verbose-but-universal option. Rejected because the noise outweighs the (very small) future-publication safety margin.
- **Mix: `bp-cli`, `bp-domain`, …** (shortened prefix). Rejected as the worst of both worlds — still prefixed, but with a less descriptive prefix.
- **Terse everywhere, including binaries** (`baby-phi-server` becomes `server`). Rejected because operator tooling relies on a distinctive process name. `ps aux | grep server` is a disaster.
- **Different prefix on the library vs the binary** (`baby-phi-server` crate → `baby-phi-server` binary, but `bp-server` or similar for imports). Rejected as confusing — having two names for the same crate invites drift.

## How this appears

Workspace manifest at [`Cargo.toml`](../../../../../../Cargo.toml) members list:
```toml
members = [
    "modules/crates/cli",
    "modules/crates/domain",
    "modules/crates/server",
    "modules/crates/store",
]
```

Workspace dep entries use the terse names:
```toml
[workspace.dependencies]
domain = { path = "modules/crates/domain" }
store  = { path = "modules/crates/store" }
server = { path = "modules/crates/server" }
```

Source imports:
```rust
use domain::Repository;
use store::SurrealStore;
use server::{build_router, AppState};
```

Binary invocation (unchanged from what operators expect):
```
$ baby-phi-server --help
$ baby-phi bootstrap claim   # M1+
```
