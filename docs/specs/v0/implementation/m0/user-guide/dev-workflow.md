<!-- Last verified: 2026-04-19 by Claude Code -->

# User guide — dev workflow

Day-to-day commands for contributing to baby-phi. Assumes you've completed [getting-started.md](getting-started.md).

## Cargo conventions

- **Always** invoke `/root/rust-env/cargo/bin/cargo`. The workstation has no `cargo` on `$PATH` by default — direct invocation ensures the right toolchain. See [`CLAUDE.md`](../../../../../../CLAUDE.md).
- **Always** run from the workspace root `/root/projects/phi/baby-phi/`. Cargo finds the workspace manifest there; running from a subcrate works but is inconsistent.

## The four local gates

Matching what CI enforces — run locally before pushing.

### Format

```bash
/root/rust-env/cargo/bin/cargo fmt --all
# Check only (what CI does):
/root/rust-env/cargo/bin/cargo fmt --all -- --check
```

### Clippy

```bash
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy --workspace --all-targets
```

All targets means tests + examples + benches, not just lib + bin. `-Dwarnings` makes every warning a hard error — matching CI exactly.

### Tests

```bash
/root/rust-env/cargo/bin/cargo test --workspace
# With --locked (what CI does; catches accidental Cargo.lock drift):
/root/rust-env/cargo/bin/cargo test --workspace --locked
```

### Web checks

```bash
cd modules/web
npm run lint
npm run typecheck
npm run test      # placeholder in M0
npm run build
```

## Narrower invocations

### Test a single crate

```bash
/root/rust-env/cargo/bin/cargo test -p server
```

### Test a single file

```bash
/root/rust-env/cargo/bin/cargo test -p server --test health_test
```

### Test a single test function

```bash
/root/rust-env/cargo/bin/cargo test -p server ready_reports_storage_down
```

### Clippy on one crate

```bash
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy -p server --all-targets
```

## Adding a new crate to the workspace (M1+ shape)

This is the canonical template every new M1+ crate should follow. (M0 already has four; no new ones are expected for a while.)

1. Decide the crate name and role. Package names are terse — `cli`, `domain`, `store`, `server`. A hypothetical M1 addition for shared HTTP types might be named `api-types` (hyphens in package names are fine; Rust code will see `use api_types::…`).
2. Create the directory:
   ```
   modules/crates/<name>/
   ├── Cargo.toml
   └── src/
       └── lib.rs    # or main.rs for a binary
   ```
3. The Cargo.toml starts from:
   ```toml
   [package]
   name = "<name>"
   version.workspace = true
   edition.workspace = true
   license.workspace = true
   repository.workspace = true
   description = "Short line."

   [dependencies]
   # Only what this crate needs:
   serde = { workspace = true }
   # …
   ```
4. Add the member to the workspace manifest [`Cargo.toml`](../../../../../../Cargo.toml):
   ```toml
   [workspace]
   members = [
       "modules/crates/cli",
       "modules/crates/domain",
       "modules/crates/server",
       "modules/crates/store",
       "modules/crates/<name>",    # new
   ]
   ```
5. If downstream crates will depend on it, add a `[workspace.dependencies]` entry:
   ```toml
   <name> = { path = "modules/crates/<name>" }
   ```
6. Run `cargo build -p <name>` to prove it compiles; `cargo clippy -p <name> --all-targets -- -D warnings` to confirm no warnings.
7. Update [`CLAUDE.md`](../../../../../../CLAUDE.md) and [`../architecture/workspace-layout.md`](../architecture/workspace-layout.md) with the new crate's role.

Rule of thumb: add a new crate only when a logical boundary needs its own dependency set. Don't create crates for "feels like a module" — a new module (`pub mod foo`) in an existing crate is usually the right call.

## Running the server + CLI + web

See [running-locally.md](running-locally.md) for boot sequences.

## Editor integration

- **`rust-analyzer`** — the workstation ships it at `/root/rust-env/cargo/bin/rust-analyzer`. VS Code's rust-analyzer extension will auto-detect it.
- Set rust-analyzer's `rust-analyzer.cargo.allTargets = true` to match CI's `--all-targets` clippy (otherwise test code may show warnings that CI doesn't flag, or vice versa).
- `editor.formatOnSave: true` with the `rust-lang.rust-analyzer` formatter handles `rustfmt`.

## Debugging

- `tracing` logs from the server respect `RUST_LOG` when `BABY_PHI_TELEMETRY__LOG_FILTER` is set (they override each other; the env-var wins). For quick debugging, `BABY_PHI_TELEMETRY__LOG_FILTER=trace cargo run -p server` dumps everything.
- For breakpoint-style debugging, `rust-gdb` or `rust-lldb` both work on the `target/debug/baby-phi-server` binary.
- SurrealDB's embedded RocksDB files can be inspected with `rocksdb-tools` if you need to peek at storage (rarely needed — prefer `cargo run -p cli -- …` CLI commands once M1 lands).

## Submodule discipline

`baby-phi` and `phi-core` are submodules of the outer `phi` repo. When you commit inside `baby-phi/`, the outer repo sees the submodule SHA change. Commit workflow:

```bash
# inside baby-phi/
git add <files>
git commit -m "…"
git push origin <branch>

# back in the outer repo
cd /root/projects/phi
git add baby-phi
git commit -m "Update baby-phi submodule to <short sha>"
git push
```

If you forget the outer-repo commit, pull requests on the outer repo will show the submodule as dirty. Make it a habit.

## Common tasks cheatsheet

```bash
# Run server locally (loopback + pretty logs)
BABY_PHI_PROFILE=dev /root/rust-env/cargo/bin/cargo run -p server

# Run the CLI demo (uses .env + config.toml)
set -a && source .env && set +a
/root/rust-env/cargo/bin/cargo run -p cli

# Full lint + format cycle in one line
/root/rust-env/cargo/bin/cargo fmt --all && \
  RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy --workspace --all-targets && \
  /root/rust-env/cargo/bin/cargo test --workspace

# Web dev server
cd modules/web && npm run dev
```
