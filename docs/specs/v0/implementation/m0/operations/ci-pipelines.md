<!-- Last verified: 2026-04-19 by Claude Code -->

# Operations — CI pipelines

Three GitHub Actions workflows gate every push and pull request to `main` / `dev`. All three must be green before merge.

## Workflows

| File | Purpose | Triggers |
|---|---|---|
| [`.github/workflows/rust.yml`](../../../../../../.github/workflows/rust.yml) | fmt, clippy, test, cargo audit, cargo deny | push + PR on `main`, `dev` |
| [`.github/workflows/web.yml`](../../../../../../.github/workflows/web.yml) | lint, typecheck, test, build, npm audit | push + PR on `main`, `dev` — paths-scoped to `baby-phi/modules/web/**` |
| [`.github/workflows/spec-drift.yml`](../../../../../../.github/workflows/spec-drift.yml) | Spec-drift guard (requirement ids referenced in code must exist in `docs/specs/v0/requirements/`) | push + PR on `main`, `dev` |

## `rust.yml` jobs

### `fmt`

`cargo fmt --all -- --check` with the `rustfmt` component. Fails if any file needs reformatting. Fix locally with `cargo fmt --all`.

### `clippy`

`RUSTFLAGS="-Dwarnings" cargo clippy --workspace --all-targets` — all targets includes tests, examples, benchmarks. Any warning becomes a hard error.

Pre-step installs `clang` + `libclang-dev` — required because `surrealdb-librocksdb-sys`'s build script uses `bindgen`, which needs libclang at compile time.

The `Swatinem/rust-cache@v2` step caches `~/.cargo/registry`, `~/.cargo/git`, and `target/` keyed on `Cargo.lock`. First CI run is slow (~5 min); subsequent runs are ~1–2 min.

### `test`

`cargo test --workspace --locked`. The `--locked` flag makes Cargo fail if `Cargo.lock` would change — prevents silent dep-version drift on CI. Same libclang + cache setup as `clippy`.

Current M0 test count: 4 (3 health + 1 TLS). Growth tracked per-milestone.

### `audit`

`cargo audit` against the RustSec advisory DB. Installs `cargo-audit` via `cargo install --locked` each run (cheap; ~30 s cold).

The advisory DB is downloaded fresh on every run so new advisories surface immediately. If an advisory is unavoidable (e.g. a transitive dep with no patched version), the workflow can be amended with `cargo audit --ignore RUSTSEC-YYYY-NNNN` after a code-review discussion.

### `deny`

`cargo deny` via the [`EmbarkStudios/cargo-deny-action@v2`](https://github.com/EmbarkStudios/cargo-deny-action) action, which reads our policy from [`deny.toml`](../../../../../../deny.toml):

- `[advisories]` — same RustSec DB as `cargo audit`. Duplication is intentional; deny's check adds license/ban/source enforcement on top.
- `[bans]` — `multiple-versions = "warn"`, `wildcards = "deny"`. Catches duplicate versions of the same crate in the graph and any dep pinned as `"*"`.
- `[licenses]` — allow-list of permissive + weakly-copyleft licences (MIT, Apache-2.0, BSD variants, ISC, MPL-2.0, Unicode, Zlib, CC0). Anything else fails CI; must be added to the allow-list explicitly.
- `[sources]` — only `crates.io` allowed. Git deps from random forks are rejected unless whitelisted.

## `web.yml` jobs

### `build`

1. `npm ci` — requires `package-lock.json` (committed at [`modules/web/package-lock.json`](../../../../../../modules/web/package-lock.json)).
2. `npm run lint` — `next lint` with the `next/core-web-vitals` config.
3. `npm run typecheck` — `tsc --noEmit`.
4. `npm run test` — placeholder for M0; real tests arrive in M3+.
5. `npm run build` — `next build` (produces `.next/standalone`).

Each step is atomic; any failure aborts. Build output is not published in M0 (no registry push yet).

The workflow is **path-scoped**: it only runs when files under `baby-phi/modules/web/**` or the workflow file itself change. This keeps the main CI loop fast.

### `audit`

`npm audit --production --audit-level=high`. Fails if any production dep has a HIGH or CRITICAL advisory. Dev deps (eslint, typescript, etc.) are not gated — they don't ship to production.

## `spec-drift.yml`

Runs [`scripts/check-spec-drift.sh`](../../../../../../scripts/check-spec-drift.sh). The script:

1. Greps `modules/` and `tests/` for requirement IDs matching `R-(ADMIN|AGENT|SYS|NFR)-[A-Z0-9]+-[A-Z0-9]+`.
2. For each id found, asserts it also appears under `docs/specs/v0/requirements/`.
3. Exits 0 if no mismatches, 1 otherwise.

Pre-M1 there are no requirement ids in code, so the script exits 0 with an "expected" message. Once M1 adds the first `R-ADMIN-01-*` id in a handler comment or test, the gate activates.

## Local reproducibility

Running the same commands locally reproduces CI:

```bash
# Rust workspace
/root/rust-env/cargo/bin/cargo fmt --all -- --check
RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy --workspace --all-targets
/root/rust-env/cargo/bin/cargo test --workspace --locked
/root/rust-env/cargo/bin/cargo install cargo-audit --locked   # one-time
/root/rust-env/cargo/bin/cargo audit
# (cargo-deny needs a one-time install or Docker run)

# Web
cd modules/web
npm ci
npm run lint
npm run typecheck
npm run test
npm run build
npm audit --production --audit-level=high
```

The `spec-drift.sh` script runs from any shell with `grep` and `bash`.

## Failure triage

| Failing job | First thing to check |
|---|---|
| `fmt` | Run `cargo fmt --all` locally, commit. |
| `clippy` | Read the diff; fix underlying issue, don't `#[allow]` without discussion. |
| `test` (existing test fails) | Did logic change? Update test or fix logic. |
| `test` (compile error) | Did a Cargo.toml dep version drift? Run `cargo update --dry-run` locally. |
| `test` (Cargo.lock changed) | Commit the updated `Cargo.lock`. The `--locked` flag is catching a real drift. |
| `audit` | Check RustSec advisory; plan an upgrade or `--ignore` after review. |
| `deny` | Check `deny.toml`; a new license/source/ban needs explicit approval. |
| `web/build` | Run `npm ci && npm run build` locally — often a `tsc` error. |
| `web/audit` | New npm advisory; check if you can bump the dep or if it's transitive. |
| `spec-drift` | A requirement id has been referenced in code but removed from `docs/specs/v0/requirements/`. Either restore the id or update the code. |

## Things `[PLANNED]` beyond M0

- **M7b:** Docker image build + Trivy scan; SBOM generation via `cosign`; multi-arch matrix.
- **M7:** Performance benchmarks as a CI gate (criterion-backed).
- **M1+:** Acceptance-test workflow that spins up a real SurrealDB per test, runs the fresh-install scenarios.
