<!-- Last verified: 2026-04-19 by Claude Code -->

# User guide — troubleshooting

Common error signatures encountered during M0 onboarding and iteration, each with the fix we took when it hit us.

## Cargo / rustc

### `error: rustc 1.94.0 is not supported by the following package: constant_time_eq@0.4.3 requires rustc 1.95.0`

The `blake3` crate (transitively pulled in by `surrealdb-core`) now requires rustc 1.95. Upgrade:

```bash
/root/rust-env/cargo/bin/rustup update stable
/root/rust-env/cargo/bin/rustc --version   # should print 1.95.0 or later
```

If `rustup update` partially corrupts the toolchain (leaves `rustc` binary missing), fully re-install:

```bash
/root/rust-env/cargo/bin/rustup toolchain uninstall stable
rm -rf /root/.rustup/downloads /root/.rustup/tmp
/root/rust-env/cargo/bin/rustup toolchain install stable --profile minimal
/root/rust-env/cargo/bin/rustup component add rustfmt clippy
```

### `Unable to find libclang: "couldn't find any valid shared libraries matching: ['libclang.so', …]"`

`surrealdb-librocksdb-sys` uses `bindgen` at build time, which needs libclang. Install:

```bash
sudo apt-get install -y clang libclang-dev
```

If the build script still can't find it (e.g. because clang was installed from a non-default location):

```bash
export LIBCLANG_PATH=/usr/lib/llvm-18/lib
cargo build --workspace
```

Add the export to your shell profile (`.bashrc` / `.zshrc`) if you hit this repeatedly.

### `error: failed to run custom build command for surrealdb-librocksdb-sys v0.18.1+…`

Usually the above libclang issue. Occasionally a missing `cmake`:

```bash
sudo apt-get install -y cmake pkg-config
```

### `could not find 'Cargo.toml' in '…' or any parent directory`

You're running `cargo` outside the workspace root. `cd /root/projects/phi/baby-phi/` and retry.

### Clippy passes locally but fails in CI

CI sets `RUSTFLAGS="-Dwarnings"`. Reproduce locally:

```bash
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy --workspace --all-targets
```

### Tests pass locally but fail in CI

Often `--locked` catching a drift. Reproduce:

```bash
/root/rust-env/cargo/bin/cargo test --workspace --locked
```

If drift is real, commit the updated `Cargo.lock`.

## Server runtime

### Server binds but `/healthz/ready` returns 503

Storage failed. Check logs for a `StoreError::Connect` from [`modules/crates/store/src/lib.rs`](../../../../../../modules/crates/store/src/lib.rs). Common causes:

- `data_dir` path doesn't exist and the parent isn't writable.
- Another process is holding the RocksDB lock (you forgot to stop a previous `cargo run -p server`).
- Disk full.

Fix:

```bash
# Check lockfile
ls -la data/baby-phi-dev.db/LOCK 2>/dev/null

# Nuclear option: wipe dev data
rm -rf data/baby-phi-dev.db
```

### `Address already in use (os error 98)` on server boot

Another service is bound to port 8080. Change ports:

```bash
BABY_PHI_SERVER__PORT=8081 /root/rust-env/cargo/bin/cargo run -p server
```

Or find the offender:

```bash
sudo lsof -i :8080
```

### `axum-prometheus` panic: `FailedToCreateHTTPListener` or `SetRecorderError`

The metrics recorder is process-global. You hit this if:

- You're running a test that calls `with_prometheus` instead of `build_router`. Use `build_router` in tests — see [`../decisions/0005-metrics-layer-separation.md`](../decisions/0005-metrics-layer-separation.md).
- Another baby-phi-server is running in the same process (shouldn't be possible in practice — `#[tokio::main]` owns the process).

### `Failed to build metrics recorder` during `cargo test`

Same root cause. `cargo test` default-runs in parallel; if two test crates each call `with_prometheus`, the second one panics. Use `build_router` in tests.

## TLS

### `Could not automatically determine the process-level CryptoProvider from Rustls crate features`

You called rustls-using code without installing a crypto provider. For the `ring` backend:

```rust
rustls::crypto::ring::default_provider().install_default();
```

This is done once per process. The TLS integration test at [`modules/crates/server/tests/tls_test.rs`](../../../../../../modules/crates/server/tests/tls_test.rs) does it at the top of the test function. Production `main.rs` doesn't need it — `axum-server`'s `tls-rustls` feature wires its own provider.

### `failed to load cert: …`

`BABY_PHI_SERVER__TLS__CERT_PATH` or `…KEY_PATH` points to a file that doesn't exist, isn't PEM-encoded, or is unreadable by the server process. Check permissions:

```bash
ls -la $BABY_PHI_SERVER__TLS__CERT_PATH $BABY_PHI_SERVER__TLS__KEY_PATH
head -1 $BABY_PHI_SERVER__TLS__CERT_PATH   # should start with "-----BEGIN CERTIFICATE-----"
head -1 $BABY_PHI_SERVER__TLS__KEY_PATH    # should start with "-----BEGIN PRIVATE KEY-----" or similar
```

## Web / Next.js

### `npm ERR! Missing: package-lock.json`

Regenerate:

```bash
cd modules/web
npm install
```

Commit `package-lock.json` afterwards.

### `Error: Cannot find module 'next'` at runtime

`node_modules` is missing or partial. `rm -rf node_modules .next && npm ci`.

### Next.js hot-reload not working

You're editing files inside a Docker bind mount and the inotify events aren't propagating. Either:

- Run the dev server natively instead of under docker-compose.
- Set `CHOKIDAR_USEPOLLING=true npm run dev` (polls; slower but works across Docker on any platform).

## Docker

### `Dockerfile`-build takes forever / runs out of memory

Release build with LTO is CPU + RAM-intensive. For local iterations:

- Build without LTO by temporarily editing `[profile.release]` in `Cargo.toml` to comment out `lto = true`.
- Or use `cargo run -p server` natively instead of Dockerizing.

### `docker compose up` fails with "permission denied" on volume

Named volumes inherit the uid the container runs as — `babyphi` uid 10001 in our case. If you previously ran the container as root and left state behind, `docker compose down -v` to reset.

### `wget: command not found` during healthcheck

The Dockerfile healthcheck assumes `wget` is available in `debian:bookworm-slim`. It is by default. If you override the base image, adjust the HEALTHCHECK command.

## Git submodules

### `git status` shows `modified: baby-phi (new commits)`

You committed inside the baby-phi submodule but haven't updated the outer `phi` repo's pointer. From the outer repo:

```bash
git add baby-phi
git commit -m "Update baby-phi submodule to <short sha>"
```

### `fatal: No url found for submodule path 'baby-phi' in .gitmodules`

`.gitmodules` is missing or the checkout is incomplete. From the outer repo:

```bash
git submodule update --init --recursive
```

## Still stuck?

- Re-run [getting-started.md](getting-started.md) from a clean clone to isolate whether it's environment-specific.
- Check the server's `tracing` output at `BABY_PHI_TELEMETRY__LOG_FILTER=trace`.
- Open an issue in the baby-phi repo; attach the failing command + log tail.
