<!-- Last verified: 2026-04-19 by Claude Code -->

# Operations — security scanning

Three scanners run on every push and pull request. Each covers a different surface: `cargo audit` for Rust runtime advisories, `cargo deny` for supply-chain + licence policy, and `npm audit` for the web tree. All three are CI gates — a failure blocks merge.

## `cargo audit`

- **Scope:** Known Rust advisories from the [RustSec Advisory Database](https://rustsec.org/).
- **CI step:** `audit` job in [`.github/workflows/rust.yml`](../../../../../../.github/workflows/rust.yml) — `cargo install cargo-audit --locked && cargo audit`.
- **Data source:** Pulled fresh on every CI run; no caching. New advisories surface within hours of publication.

### Local equivalent

```bash
cd baby-phi
/root/rust-env/cargo/bin/cargo install cargo-audit --locked   # one-time
/root/rust-env/cargo/bin/cargo audit
```

### Handling an advisory

1. Identify whether the affected crate is a direct dep (in one of our Cargo.toml files) or transitive (pulled in by another crate).
2. **Direct dep:** bump the version in the workspace `Cargo.toml`'s `[workspace.dependencies]` and re-run.
3. **Transitive dep, patched version exists:** bump the parent crate that pulls it in, or add an explicit override in `Cargo.toml` via `[patch.crates-io]`.
4. **No patched version exists:** open an issue, evaluate whether we're exposed to the vuln (some advisories only affect specific usage patterns), and, if appropriate, add `cargo audit --ignore RUSTSEC-YYYY-NNNN` as a temporary bypass with a dated TODO to remove the ignore.

Never land an `--ignore` without a sentence explaining *why* — either on the command line comment or in the commit.

## `cargo deny`

- **Scope:** Licence compliance, advisory cross-check, supply-chain sources, banned crates.
- **CI step:** `deny` job in [`.github/workflows/rust.yml`](../../../../../../.github/workflows/rust.yml) — runs `EmbarkStudios/cargo-deny-action@v2` against [`deny.toml`](../../../../../../deny.toml).

### Policy overview ([`deny.toml`](../../../../../../deny.toml))

```toml
[advisories]
yanked = "deny"
ignore = []

[bans]
multiple-versions = "warn"
wildcards = "deny"

[licenses]
allow = [
  "MIT", "Apache-2.0", "Apache-2.0 WITH LLVM-exception",
  "BSD-2-Clause", "BSD-3-Clause", "ISC",
  "Unicode-DFS-2016", "Unicode-3.0", "Zlib",
  "CC0-1.0", "MPL-2.0", "OpenSSL",
]
confidence-threshold = 0.8

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
```

- **Yanked crates** are rejected outright — a yanked version usually means the author withdrew it for a reason.
- **Multiple versions** of the same crate in the graph are **warned**, not denied, because dep conflicts are common. Worth periodic cleanup but not a merge blocker.
- **Wildcard versions** (`"*"`) are denied — pins must be explicit.
- **Licences** — permissive + weakly-copyleft. Anything else (GPL, AGPL, proprietary) fails. Adding a licence to the allow-list requires a policy discussion, not a silent commit.
- **Sources** — only `crates.io`. Git deps from random forks are rejected. Whitelist them explicitly if there's a genuine need.

### Local equivalent

```bash
cd baby-phi
/root/rust-env/cargo/bin/cargo install cargo-deny --locked   # one-time
/root/rust-env/cargo/bin/cargo deny check
```

### Handling a deny failure

| Class | Response |
|---|---|
| Licence not on allow-list | Read the licence. If acceptable, add it to `deny.toml` with a commit message explaining why. If not, find a different crate. |
| Multiple-version warning | Often a transitive-dep-hell situation. Run `cargo tree -d` to see why. Usually resolved by bumping the outer dep. |
| Wildcard version | Some crate pinned `"*"` in its Cargo.toml. Either upstream fixes it or we `[patch.crates-io]` a pin. |
| Unknown source (git fork) | Probably an accidental `path` change. Fix it. If intentional, add to `[sources].allow-git`. |

## `npm audit`

- **Scope:** Known JavaScript advisories from the [npm advisory DB](https://github.com/advisories).
- **CI step:** `audit` job in [`.github/workflows/web.yml`](../../../../../../.github/workflows/web.yml) — `npm audit --production --audit-level=high`.

### Key flags

- `--production` — dev deps (`eslint`, `typescript`, `autoprefixer`, etc.) are excluded. They don't ship to prod; a dev-only advisory doesn't risk production.
- `--audit-level=high` — only HIGH and CRITICAL advisories block. MODERATE and LOW are surfaced in the CI log but don't fail the build. This threshold can tighten in M7b if the web tree stabilises and we can commit to a lower baseline.

### Local equivalent

```bash
cd baby-phi/modules/web
npm audit --production --audit-level=high
```

### Handling an npm advisory

1. `npm audit fix` — if the advisory has an automatic patch path in the dep graph.
2. `npm audit fix --force` — if the automatic patch requires a major-version bump. **Read the breaking changes first**; don't blindly take these.
3. Override a transitive version via `overrides` in `package.json` — last resort.

npm advisories are often transitive and are often announced before a patch exists. For production blockers with no patch, document the exposure, file an issue upstream, and (if applicable) add a one-line comment in `package.json` noting the known issue. Avoid `--force` overrides that unpin critical deps without review.

## Dependabot — `[PLANNED M7b]`

The build plan commits Dependabot in M7b for ongoing dep refresh. Not in M0. Until then, we run the three scanners manually on a weekly cadence is the informal plan — formalised in the M7b runbook section.

## SAST — `[PLANNED M7b]`

`cargo clippy -W clippy::pedantic` on critical crates (`domain`, `store`, `server`) is a static-analysis tightening planned for M7b. M0 runs clippy on default lints — still strict (`-Dwarnings`) but not pedantic.

## Image scanning — `[PLANNED M7b]`

Trivy scan on the Docker image is an M7b deliverable alongside the rest of the deployment hardening:

- Base-image CVE check.
- Binary-dep scan (catches C-dep vulnerabilities that `cargo audit` misses).
- Misconfig check against Docker best practices.

Not in M0.

## Philosophy

Three overlapping scanners is deliberate redundancy. `cargo audit` catches RustSec advisories; `cargo deny` cross-checks the same DB *and* adds licence + supply-chain policy; `npm audit` covers the web tree. No single tool is the source of truth. The false-positive rate is low at v0 scale, and the cost of a missed advisory is too high to rely on only one.

For every new HIGH/CRITICAL advisory, the default posture is: **fix it within a week**, or write down (in an issue + a follow-up in the runbook) why we can't and what our exposure is.
