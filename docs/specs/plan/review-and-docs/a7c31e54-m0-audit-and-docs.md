# Plan: M0 Accuracy Check + M0 Documentation

> **Legend:**
> - `[STATUS: ⏳ pending]` — not yet done
> - `[STATUS: ✓ done]` — already complete
> - `[STATUS: n/a]` — reference / meta section

## Context  `[STATUS: n/a]`

M0 (project scaffolding) is implementation-complete and CI-green: workspace restructured under `baby-phi/modules/{crates,web}`, SurrealDB embedded (RocksDB) adapter wired, health + metrics endpoints live, Next.js 14 scaffold in place, three CI workflows set up. Before starting M1 (Permission Check spine) we want two things:

1. **An independent accuracy check** of what M0 actually shipped vs what the archived build plan committed to (`baby-phi/docs/specs/plan/build/36d0c6c5-build-plan-v01.md`).
2. **Detailed M0 documentation** under a new tree `baby-phi/docs/specs/v0/implementation/m0/`, organised into `architecture/`, `user-guide/`, `operations/`, and `decisions/` (ADRs) subfolders.

The audit result (below) says M0 is structurally solid but has one HIGH-severity production-readiness gap that should be closed *before* docs are written — so the docs reflect a correct M0, not one with TODO markers throughout.

The *overall v0.1 build plan* (M0→M8) still lives in its archive at `baby-phi/docs/specs/plan/build/36d0c6c5-build-plan-v01.md`. This plan file supersedes that one only for the next block of work (audit + docs); M1+ planning will be picked up separately.

**Archive location for this plan:** `baby-phi/docs/specs/plan/review-and-docs/a7c31e54-m0-audit-and-docs.md` (new subfolder; matches the sibling convention used by `plan/build/` and `plan/requirements/`). The first execution step (Part 7 step 0) is to copy this plan file verbatim to that path so the audit + doc plan is preserved alongside the other archived plans.

---

## Part 1 — Accuracy audit of M0  `[STATUS: ✓ done]`

An independent Explore-agent audit compared the current state of `baby-phi/` against the archived build plan's §M0 bullets + the M0-tagged rows in §Production-readiness commitments.

### Section scores

| Area | Verdict | Evidence (file path) |
|---|---|---|
| Workspace layout (`modules/crates/{cli,domain,store,server}` + `modules/web`) | ✓ | `Cargo.toml` members array; member Cargo.tomls |
| Package names (terse) + binary names (prefixed) | ✓ | `modules/crates/cli/Cargo.toml` (`name="cli"`, `[[bin]] name="baby-phi"`); `modules/crates/server/Cargo.toml` |
| Rust imports updated to short names | ✓ | `modules/crates/server/src/{main,state}.rs`, `modules/crates/store/src/lib.rs` |
| Health endpoints (`/healthz/{live,ready}`) split + storage probe | ✓ | `modules/crates/server/src/health.rs` |
| `/metrics` via axum-prometheus, production-only wiring | ✓ | `modules/crates/server/src/router.rs` (`build_router` vs `with_prometheus`) |
| SurrealDB embedded RocksDB adapter + `Repository::ping` | ✓ | `modules/crates/store/src/lib.rs` |
| 12-factor layered config (default + dev/staging/prod) + `BABY_PHI_*` env overrides | ✓ | `config/*.toml`, `modules/crates/server/src/config.rs` |
| Next.js 14 App Router + SSR + Tailwind + `/api/v0/*` proxy + auth stub | ✓ | `modules/web/**` |
| Dockerfile multi-stage + non-root user (uid 10001) + tini + HEALTHCHECK | ✓ | `Dockerfile` |
| docker-compose.yml referencing `modules/web` | ✓ | `docker-compose.yml` |
| CI: rust.yml (fmt, clippy, test, cargo-audit, cargo-deny) | ✓ | `.github/workflows/rust.yml` |
| CI: web.yml (lint, typecheck, test, build, npm audit) | ✓ | `.github/workflows/web.yml` |
| CI: spec-drift.yml + `scripts/check-spec-drift.sh` (executable, grepping `modules/`) | ✓ | `.github/workflows/spec-drift.yml`, `scripts/check-spec-drift.sh` |
| `deny.toml` policy (advisories, bans, licences, sources) | ✓ | `deny.toml` |
| `RUSTFLAGS=-Dwarnings` enforced in CI | ✓ | `.github/workflows/rust.yml` |
| Build plan archived verbatim | ✓ | `docs/specs/plan/build/36d0c6c5-build-plan-v01.md` |
| `CLAUDE.md` reflects new layout | ✓ | `baby-phi/CLAUDE.md` |
| **TLS support surface** (axum-rustls crate + listener path) | **✗** | `TlsConfig` struct exists in `modules/crates/server/src/config.rs` but no `axum-rustls` dep in Cargo.toml and no listener path in `main.rs` |
| `modules/web/package-lock.json` committed | ✗ | File not present; CI `npm ci` reproducibility at risk |
| `.env.example` for developer onboarding | ✗ | Absent |
| `docs/ops/runbook.md` stub | ✗ | Absent (plan officially defers full runbook to M7b, but a stub anchor was expected in M0) |

### Gaps — ranked

| # | Gap | Severity | Why it matters |
|---|---|---|---|
| 1 | **axum-rustls dep + listener path not wired.** The build plan's §Production-readiness row `Transport security (TLS)` explicitly tags M0 for the server-skeleton TLS surface. `TlsConfig` is declared in `config.rs` but the Cargo.toml lacks `axum-rustls` and `main.rs` never consumes the config's `tls` field. | **HIGH** | Stated production-readiness commitment unfulfilled. Silently shipping a config surface that doesn't do anything is worse than not shipping the surface. |
| 2 | **No `modules/web/package-lock.json`.** web.yml runs `npm ci` — without a lockfile, first CI run will npm-install and resolve fresh, so dependency versions drift. | MEDIUM | CI reproducibility. Does not block M0 green but will bite on the first PR that touches web deps. |
| 3 | **No `.env.example`.** Onboarding a new developer requires reading code to discover env-var names. | LOW | Onboarding friction. |
| 4 | **No runbook stub.** Plan puts the full runbook in M7b; an M0 stub (even a one-liner pointing forward) was implied but not landed. | LOW | Ops anchor. |

### Confidence: **75 %**

**Rubric applied:** 1 HIGH gap (one unfulfilled production-readiness commitment) + 1 MEDIUM + 2 LOW. Structure-only confidence would be ~92 %. TLS commitment unfulfilment drops it to 75 %. After Part 2 remediation below, expected confidence is **~92 %**.

---

## Part 2 — Pre-doc remediation  `[STATUS: ⏳ pending]`

Close every gap identified in Part 1 **before** writing any documentation. Sequencing chosen so the docs describe a correct M0, not a partial one.

### R1 — Wire native TLS via `axum-rustls`  `[HIGH]`
1. Add `axum-server = { version = "0.7", features = ["tls-rustls"] }` to `baby-phi/Cargo.toml` `[workspace.dependencies]` and to `modules/crates/server/Cargo.toml`. (`axum-server` is the canonical crate; `axum-rustls` in the original plan was shorthand.)
2. In `modules/crates/server/src/main.rs`:
   - If `cfg.server.tls` is `Some(tls)`: bind via `axum_server::tls_rustls::RustlsConfig::from_pem_file(tls.cert_path, tls.key_path)` and serve with `axum_server::bind_rustls`.
   - Otherwise: existing `axum::serve(TcpListener, app)` path (HTTP).
3. Add a comment to `config/prod.toml` explaining that the reverse-proxy-with-TLS-termination pattern is the recommended production default; the inline-TLS path is for simple deploys.
4. Add a unit/integration smoke test: with a self-signed cert fixture under `tests/fixtures/tls/` (generated by `openssl req -x509 -nodes -subj '/CN=localhost' -keyout key.pem -out cert.pem -newkey rsa:2048 -days 3650` on first setup), assert that a `TlsConfig`-enabled binary refuses a plain-HTTP client and accepts an HTTPS one.
5. Update workspace build + clippy + test verification (R5 below).

### R2 — Generate + commit `modules/web/package-lock.json`  `[MEDIUM]`
1. From `baby-phi/modules/web/` run `npm install` (produces `package-lock.json` and `node_modules/`).
2. Commit only `package-lock.json`; `node_modules/` is already gitignored.
3. Confirm `web.yml`'s `cache-dependency-path: baby-phi/modules/web/package-lock.json` resolves (it does, path is already correct).

### R3 — Add `.env.example`  `[LOW]`
1. Create `baby-phi/.env.example` with commented placeholders for:
   - `OPENROUTER_API_KEY=` (consumed by the CLI demo via existing `.env` flow).
   - `BABY_PHI_PROFILE=dev` (selects the overlay in `config/`).
   - `BABY_PHI_API_URL=http://127.0.0.1:8080` (consumed by the Next.js web app).
   - Placeholders for future secrets — labelled `[M3]`, `[M7b]` — so developers see the future shape early.
2. Confirm `.env` (the real file) is still git-ignored — existing `.gitignore` has the rule.

### R4 — Add `docs/ops/runbook.md` stub  `[LOW]`
1. Create `baby-phi/docs/ops/runbook.md` as an explicit placeholder: the full runbook is an M7b deliverable; M0 ships a title, a "not yet populated — see build plan M7b" note, and a table of contents of the sections that will fill in (deploy, upgrade, rollback, backup, restore, 5 common incidents, known issues).
2. This is a 20-line file. Its purpose is a future-anchor link so onboarding can link to it without a 404.

### R5 — Re-verify green after remediation
1. `cd baby-phi && /root/rust-env/cargo/bin/cargo fmt --all -- --check`
2. `RUSTFLAGS="-Dwarnings" /root/rust-env/cargo/bin/cargo clippy --workspace --all-targets`
3. `/root/rust-env/cargo/bin/cargo test --workspace`
4. `cd modules/web && npm ci && npm run lint && npm run typecheck && npm run build`

Pass criteria: all four green. Expected outcome: M0 confidence rises from 75 % to **~92 %** (structural ≤3 low-severity smells).

---

## Part 3 — Documentation structure  `[STATUS: ⏳ pending]`

Root: `baby-phi/docs/specs/v0/implementation/m0/`

```
implementation/m0/
├── README.md                        ← index + quick navigation
├── architecture/                    ← "how M0 is built"
│   ├── overview.md
│   ├── workspace-layout.md
│   ├── server-topology.md
│   ├── storage-and-repository.md
│   ├── configuration.md
│   ├── telemetry-and-metrics.md
│   └── web-topology.md
├── user-guide/                      ← "how to run / develop on M0"
│   ├── getting-started.md
│   ├── dev-workflow.md
│   ├── running-locally.md
│   ├── docker-compose.md
│   ├── health-and-metrics.md
│   └── troubleshooting.md
├── operations/                      ← "how to deploy / monitor / secure"
│   ├── deployment-docker.md
│   ├── configuration-profiles.md
│   ├── tls-and-transport-security.md
│   ├── ci-pipelines.md
│   └── security-scanning.md
└── decisions/                       ← ADRs; one per load-bearing choice
    ├── 0001-surrealdb-over-memgraph.md
    ├── 0002-three-parallel-surfaces.md
    ├── 0003-modules-crates-layout.md
    ├── 0004-terse-package-names.md
    ├── 0005-metrics-layer-separation.md
    ├── 0006-twelve-factor-layered-config.md
    └── 0007-embedded-vs-sidecar-database.md
```

### Per-file content outline

#### `README.md`
- One-paragraph purpose (this is the executed implementation for M0 — the archived plan is the intent, these docs are the actuality).
- Links into each subfolder with one-line summaries per file.
- Links back to the archived build plan (`../../../plan/build/36d0c6c5-build-plan-v01.md`).
- Stable URL structure for future `m1/`, `m2/`, … siblings.

#### `architecture/overview.md`
- System map diagram (three surfaces + shared domain + shared DB), in ASCII.
- Dependency-flow rule (downward-only).
- Cross-link to individual architecture sub-pages.

#### `architecture/workspace-layout.md`
- Enumerates every crate and its role, with file-path links.
- Package-name-vs-binary-name distinction.
- Why `modules/crates/` and `modules/web/` (reference ADR-0003).

#### `architecture/server-topology.md`
- `build_router` vs `with_prometheus` separation (reference ADR-0005).
- `AppState` injection model (trait-object Repository).
- Health endpoint semantics (liveness = always, readiness = storage ping).
- Handler surface table (endpoint → handler → input → output → status codes) — for M0 the surface is three routes; this sets the template for later milestones.

#### `architecture/storage-and-repository.md`
- `Repository` trait surface (M0: just `ping`; M1+ grows).
- `SurrealStore::open_embedded` mechanics.
- RocksDB backend choice + scaling tiers (reference ADR-0001, ADR-0007).
- Migration path / failover notes (copy-paste from plan's §Data migration table).

#### `architecture/configuration.md`
- Precedence (default → profile → env).
- Env-var naming: `BABY_PHI_<SECTION>__<FIELD>`.
- Secrets boundary (never in files, always env).
- Full schema reference with field-by-field description.

#### `architecture/telemetry-and-metrics.md`
- `tracing` subscriber setup (pretty for dev, JSON for prod).
- `EnvFilter` + `BABY_PHI_TELEMETRY__LOG_FILTER`.
- `axum-prometheus` wiring + the "one global recorder per process" caveat.
- What metrics exist today (HTTP request histogram from `axum-prometheus`) and what will be added in M7 (custom `permission_check_latency_seconds`, etc.) — clearly tagged `[PLANNED M7]`.

#### `architecture/web-topology.md`
- App Router layout (`app/layout.tsx`, `app/page.tsx`).
- SSR-first, `dynamic = "force-dynamic"` on the health page.
- API proxy via `next.config.mjs` rewrites (`/api/v0/*` → `BABY_PHI_API_URL`).
- Auth placeholder contract (`lib/session.ts`) — what will be filled in at M3/M7b.

#### `user-guide/getting-started.md`
- Toolchain requirements: rustc ≥ 1.95, cargo 1.95, Node 22, libclang-18, clang.
- Install sequence for a fresh box (apt install, rustup install).
- First-build verification: `cargo build --workspace` → succeeds; `cargo test --workspace` → 3 health tests pass.

#### `user-guide/dev-workflow.md`
- `/root/rust-env/cargo/bin/cargo` invocation convention.
- fmt / clippy / test commands (copied from `CLAUDE.md` but with deeper explanation).
- Where to add a new crate (only at M1+, but the shape is documented so M1 follows it).
- How to run a single crate's tests, a single test, etc.

#### `user-guide/running-locally.md`
- CLI demo: `.env` + `cargo run -p cli` + `config.toml`.
- Server: `BABY_PHI_PROFILE=dev cargo run -p server` → `curl http://127.0.0.1:8080/healthz/live`.
- Web: `cd modules/web && npm install && npm run dev` → http://localhost:3000.

#### `user-guide/docker-compose.md`
- `docker-compose up --build` → both services up.
- Volume persistence (`baby-phi-data`).
- Rebuild on code change (images vs bind mounts).

#### `user-guide/health-and-metrics.md`
- `curl /healthz/live` → always 200.
- `curl /healthz/ready` → 200 or 503 depending on DB; sample JSON.
- `curl /metrics` → `axum-prometheus` default metrics; sample output; how Prometheus scraping is configured downstream.

#### `user-guide/troubleshooting.md`
- `error: rustc X is not supported by … constant_time_eq` → rustup update stable.
- `libclang not found` → `apt install libclang-dev clang` + `LIBCLANG_PATH=/usr/lib/llvm-18/lib`.
- "Port 8080 in use" → change `BABY_PHI_SERVER__PORT`.
- "axum-prometheus panics on second test" → use `build_router` not `with_prometheus` in tests (reference ADR-0005).

#### `operations/deployment-docker.md`
- Dockerfile walk-through, stage-by-stage.
- Non-root user, tini init, healthcheck.
- Expected runtime env vars.
- Volume mount contract (`/var/lib/baby-phi/data`).

#### `operations/configuration-profiles.md`
- `dev` / `staging` / `prod` semantics.
- How to add a new profile.
- Layering example: a field overridden by each layer, ending with an env var.

#### `operations/tls-and-transport-security.md`
- Recommended pattern: reverse proxy (nginx/Caddy) terminates TLS.
- Native TLS path via `axum-server` + `axum_server::tls_rustls::RustlsConfig` (wired during R1).
- Cert rotation: swap files + restart (v0.1), hot-reload is [PLANNED M7b].
- Minimum TLS version (1.3) and cipher policy (rustls defaults).

#### `operations/ci-pipelines.md`
- Per-workflow: what it runs, on which triggers, how to interpret failures.
- Cache story (`Swatinem/rust-cache`).
- How to reproduce a CI run locally.

#### `operations/security-scanning.md`
- `cargo audit` (RustSec advisories) — how to bump the advisory DB.
- `cargo deny` (licences, bans, sources) — editing `deny.toml`.
- `npm audit` severity threshold (`high`) — handling alerts.

#### `decisions/000X-*.md` (one ADR per load-bearing choice)

Each ADR is a single page with this shape:
```
# ADR-000X: <title>
## Status
Accepted — 2026-04-19.
## Context
Why the question arose.
## Decision
The choice made.
## Consequences
Positive + negative trade-offs.
## Alternatives considered
Bulleted with a sentence each.
```

- **0001 SurrealDB over Memgraph** — adapt the comparison table + "why SurrealDB wins" from the build plan; reference `baby-phi/docs/specs/plan/build/36d0c6c5-build-plan-v01.md` for full rationale.
- **0002 Three parallel surfaces** — CLI + HTTP API + web. Why the API is the single source of truth both clients consume.
- **0003 `modules/crates/` + `modules/web/` layout** — why this split, why not flat crate dirs at workspace root.
- **0004 Terse package names** — `cli`, `domain`, `store`, `server`; binary names keep the product prefix.
- **0005 Metrics layer separation** — `build_router` vs `with_prometheus`; the axum-prometheus global-recorder caveat.
- **0006 12-factor layered config** — TOML for defaults, env for secrets, profile selection via `BABY_PHI_PROFILE`.
- **0007 Embedded vs sidecar database** — embedded RocksDB for v0.1; scaling path to standalone server / TiKV cluster documented.

---

## Part 4 — Writing conventions  `[STATUS: n/a]`

Applied to every page under `docs/specs/v0/implementation/m0/`:

- **Verification header** on every file: `<!-- Last verified: 2026-04-19 by Claude Code -->`. Updated on each review pass per project convention.
- **Status tags** where a feature is referenced: `[EXISTS]`, `[PLANNED M<n>]`, or `[CONCEPTUAL]`.
- **No forward references** — M0 docs describe what M0 shipped; M1+ features are explicitly tagged `[PLANNED]` with the milestone number.
- **File + line links** for every code-rooted claim, so docs stay discoverable as code evolves. Example: "See [`modules/crates/server/src/health.rs:23-39`](../../../../modules/crates/server/src/health.rs#L23-L39)."
- **Link back to the archived build plan** for rationale-heavy claims, rather than copying text. The plan is immutable ground truth for the v0.1 intent; the implementation docs describe actuality.
- **Diagrams in ASCII**, not image binaries — keeps review diff-able and free of dependencies.

---

## Part 5 — Critical files (reference)  `[STATUS: n/a]`

Will be modified:
- `baby-phi/Cargo.toml`  (R1 — add axum-server workspace dep)
- `baby-phi/modules/crates/server/Cargo.toml`  (R1 — pull axum-server into server crate)
- `baby-phi/modules/crates/server/src/main.rs`  (R1 — branch on `cfg.server.tls`)
- `baby-phi/config/prod.toml`  (R1 — reverse-proxy comment)

Will be created:
- `baby-phi/docs/specs/plan/review-and-docs/a7c31e54-m0-audit-and-docs.md`  (archive of this plan; new `review-and-docs/` subfolder)
- `baby-phi/modules/web/package-lock.json`  (R2)
- `baby-phi/.env.example`  (R3)
- `baby-phi/docs/ops/runbook.md`  (R4 — stub)
- `baby-phi/tests/fixtures/tls/{cert,key}.pem`  (R1 — self-signed fixtures for TLS smoke test)
- `baby-phi/modules/crates/server/tests/tls_test.rs`  (R1 — TLS integration smoke)
- `baby-phi/docs/specs/v0/implementation/m0/README.md`
- `baby-phi/docs/specs/v0/implementation/m0/architecture/*.md`  (7 files)
- `baby-phi/docs/specs/v0/implementation/m0/user-guide/*.md`  (6 files)
- `baby-phi/docs/specs/v0/implementation/m0/operations/*.md`  (5 files)
- `baby-phi/docs/specs/v0/implementation/m0/decisions/*.md`  (7 ADRs)

Total new files: ~30. Per-page length target: 60–200 lines. Total doc additions: ~4–5k lines.

---

## Part 6 — Verification  `[STATUS: ⏳ pending]`

Before declaring this plan's work complete, verify:

1. **Build green** after R1–R5: `cargo build --workspace`, `cargo fmt --check`, `cargo clippy --all-targets -Dwarnings`, `cargo test --workspace`, `npm ci && npm run lint && npm run typecheck && npm run build`.
2. **Post-remediation re-audit** — re-run the same Explore-agent audit prompt from Part 1 and confirm the score climbs from 75 % to ≥ 90 %.
3. **Every code reference in docs grep-verifies** against the code. A helper script `scripts/check-doc-links.sh` walks every `implementation/m0/**/*.md`, extracts file-line references, and asserts each file exists and the cited line range is within file length. Added as a new CI gate after docs land.
4. **Doc-alignment cross-check** — every paragraph that makes a factual claim about code has either (a) a file-line link, (b) a link to an ADR, or (c) a link to the archived build plan. No orphan claims.
5. **Status-tag audit** — every `[EXISTS]` tag is grep-able in code; every `[PLANNED M<n>]` tag points to a real future milestone.

---

## Part 7 — Execution order  `[STATUS: ⏳ pending]`

0. **Archive this plan** — create `baby-phi/docs/specs/plan/review-and-docs/` (new subfolder) and copy this plan verbatim to `baby-phi/docs/specs/plan/review-and-docs/a7c31e54-m0-audit-and-docs.md`. Mirrors the pattern used for the v0.1 build plan (`plan/build/36d0c6c5-…`) and the requirements plan (`plan/requirements/e2781622-…`). (~2 min.)
1. **R1** — axum-server + TLS wiring + smoke test (~45 min).
2. **R2** — `npm install` in modules/web, commit lockfile (~5 min).
3. **R3** — `.env.example` (~5 min).
4. **R4** — runbook stub (~10 min).
5. **R5** — re-verify fmt/clippy/test/web green (~10 min + long test runs).
6. **Re-audit** (Explore agent) — confirm ≥ 90 % (~5 min).
7. **Docs authoring** — in the order: README → architecture (start from overview, then layout, then depth pages) → decisions (ADRs, since they're referenced from architecture) → operations → user-guide. This order keeps cross-references resolvable as you go. (~3–5 hours for the 30-file set.)
8. **CI gate** — add `scripts/check-doc-links.sh` + `.github/workflows/doc-links.yml` mirroring `spec-drift.yml`'s pattern.
9. **Final verification** — Part 6 items 1–5.

---

## What stays unchanged  `[STATUS: n/a]`

- The overall v0.1 build plan at `baby-phi/docs/specs/plan/build/36d0c6c5-build-plan-v01.md` is not edited by this work — it remains the source of truth for M0–M8 intent.
- Concept + requirement specs under `docs/specs/v0/concepts/` + `docs/specs/v0/requirements/` are untouched.
- `baby-phi/CLAUDE.md` already reflects the M0-post-restructure layout; it will be cross-linked from the new `implementation/m0/README.md` but not rewritten.
- M1 planning is a separate conversation that picks up once this plan's work lands.
