<!-- Last verified: 2026-04-19 by Claude Code -->

# User guide — getting started

This page takes you from a clean Linux / macOS workstation to a green `cargo test --workspace`. Expect 30–45 minutes if the network is fast; most of the time is Rust's first compile.

## Prerequisites

| Tool | Minimum | Why |
|---|---|---|
| `rustc` + `cargo` | 1.95 | `surrealdb-core` → `blake3` requires rustc 1.95+. |
| `clang` + `libclang-dev` (Linux) | 14+ | `surrealdb-librocksdb-sys` uses `bindgen` at build time. |
| `cmake` | 3.20+ | Transitive build-script dep. |
| `pkg-config` | any | Transitive. |
| Node | 22 | Matches `engines` in `modules/web/package.json`. |
| `npm` | 10+ | Ships with Node 22. |
| `git` | 2.20+ | Submodule support. |
| Docker + docker compose v2 (optional) | 24+ | For `docker compose up`. |

### Rust toolchain

If you don't have rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
source "$HOME/.cargo/env"
rustup toolchain install stable --profile minimal
```

On the workstation we use, Cargo lives at `/root/rust-env/cargo/bin/cargo` — every cargo invocation in this repo uses that absolute path (see [`CLAUDE.md`](../../../../../../CLAUDE.md)).

### libclang (Linux)

```bash
sudo apt-get update
sudo apt-get install -y clang libclang-dev cmake pkg-config
```

Verify:
```bash
clang --version           # Ubuntu clang version 18.x expected
ls /usr/lib/llvm-*/lib/libclang.so*
```

If the build fails with `Unable to find libclang`, set `LIBCLANG_PATH=/usr/lib/llvm-18/lib` (adjust version).

### Node

```bash
# Via nvm (recommended for dev boxes)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
. ~/.nvm/nvm.sh
nvm install 22
nvm use 22
```

Verify:
```bash
node --version   # v22.x.x
npm --version    # 10.x or 11.x
```

## Clone the repository

The phi workspace is a git submodule inside `phi`. Clone with submodules:

```bash
git clone --recurse-submodules https://github.com/LazyBouy/phi.git
cd phi/phi    # the workspace root for all cargo commands
```

If you already cloned without submodules:
```bash
git submodule update --init --recursive
```

## First build

```bash
/root/rust-env/cargo/bin/cargo build --workspace
```

First run takes ~5–6 minutes (downloads + compiles ~500 crates). Subsequent builds are incremental — ~10 s for a no-op, ~30–90 s after a code change.

Expected output tail:
```
Compiling server v0.1.0 (/root/projects/phi/phi/modules/crates/server)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5m 59s
```

## Verify tests pass

```bash
/root/rust-env/cargo/bin/cargo test --workspace
```

Expected:
```
test live_is_always_ok ... ok
test ready_reports_storage_up ... ok
test ready_reports_storage_down ... ok
test result: ok. 3 passed; 0 failed; 0 ignored
test native_tls_listener_serves_https_and_rejects_plaintext ... ok
test result: ok. 1 passed; 0 failed; 0 ignored
```

Four tests total: 3 health + 1 TLS.

## Verify lint + format

```bash
/root/rust-env/cargo/bin/cargo fmt --all -- --check
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy --workspace --all-targets
```

Both should produce no output beyond the "Checking … Finished" status lines.

## First Next.js build

```bash
cd modules/web
npm ci              # reads package-lock.json; deterministic install
npm run lint        # ESLint
npm run typecheck   # tsc --noEmit
npm run build       # next build (standalone output)
```

Expected `build` tail:
```
Route (app)                              Size     First Load JS
┌ ƒ /                                    138 B          87.2 kB
└ ○ /_not-found                          873 B            88 kB
```

## Copy `.env.example` to `.env`

```bash
cd /root/projects/phi/phi    # back to workspace root
cp .env.example .env
```

Edit `.env` if you want to run the CLI demo (set `OPENROUTER_API_KEY`). For the HTTP server + web UI, the defaults work out of the box.

## You're ready

Next stop: [running-locally.md](running-locally.md) — boot the server, CLI, and web UI and hit the health endpoints.

If any of the above failed, head to [troubleshooting.md](troubleshooting.md) for the common error signatures.
