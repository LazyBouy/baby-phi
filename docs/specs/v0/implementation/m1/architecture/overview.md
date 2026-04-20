<!-- Last verified: 2026-04-20 by Claude Code -->

# M1 architecture — overview

M1 turns the M0 scaffolding into a functional Permission Check spine. This
page is the system map; depth pages cover each subsystem.

## System map (P1–P7 landed)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          web (Next.js 14 / SSR)                         │
│                       [P8]  /bootstrap page                             │
├─────────────────────────────────────────────────────────────────────────┤
│                       cli (clap; baby-phi binary)                       │
│     ✓ bootstrap — `baby-phi bootstrap {status,claim}` (HTTP client)     │
│     ✓ agent     — `baby-phi agent demo` (legacy phi-core demo loop)     │
├─────────────────────────────────────────────────────────────────────────┤
│                 server (axum; baby-phi-server binary)                   │
│   existing: /healthz/live /healthz/ready /metrics                       │
│     ✓ bootstrap — `bootstrap-init` subcommand + atomic claim logic      │
│     ✓ handlers  — GET /api/v0/bootstrap/status + POST /bootstrap/claim  │
│     ✓ session   — HS256 signed `baby_phi_session` cookie + verify       │
│     ✓ metric    — baby_phi_bootstrap_claims_total{result}               │
├─────────────────────────────────────────────────────────────────────────┤
│                              domain                                     │
│   ✓ model/   — 9 fundamentals, 8 composites, 37 nodes, 66 edges         │
│   ✓ model/principal_resource — sealed Principal/Resource marker traits  │
│   ✓ audit    — event shape, class tiers, hash-chain helper              │
│   ✓ repository — 35-method trait + 3 typed free-function wrappers       │
│   ✓ in_memory — HashMap-backed Repository fake (feature-gated)          │
│   ✓ permissions — 6-step (+2a) engine, pure fn, metric-instrumented     │
│   ✓ auth_requests — 9-state machine + aggregation + retention + revoke  │
├─────────────────────────────────────────────────────────────────────────┤
│                               store                                     │
│   ✓ SurrealStore::open_embedded runs migrations on startup              │
│   ✓ migrations — forward-only runner with startup-gate fail-safe        │
│   ✓ crypto — AES-GCM envelope for secrets_vault                         │
│   ✓ repo_impl — full SurrealDB impl of all 35 Repository methods        │
├─────────────────────────────────────────────────────────────────────────┤
│                  SurrealDB (embedded RocksDB backend)                   │
│   ✓ schema 0001_initial: 37 node tables + 66 edge relations             │
│                          + bootstrap_credentials + secrets_vault        │
│                          + audit_events + resources_catalogue           │
└─────────────────────────────────────────────────────────────────────────┘
```

**Dependency flow** (strict, downward):

```
cli              ┐
server          ─┼─▶ domain ─▶ store ─▶ SurrealDB
web (Next.js)  ──┘             (plus phi-core for agent/session types)
```

The `domain` crate does **not** depend on `store`; `Repository` is a trait
defined in `domain` and implemented in `store`. This keeps the crate DAG
downward-only and lets domain tests use an in-memory fake.

## What P1 + P2 + P3 + P4 + P5 + P6 + P7 delivered

**P1 foundation:**

1. **Graph model** (`modules/crates/domain/src/model/`). See
   [graph-model.md](graph-model.md) for the full inventory.
2. **Audit-event skeleton** (`modules/crates/domain/src/audit.rs`). See
   [audit-events.md](audit-events.md).
3. **Schema + migration runner** (`modules/crates/store/migrations/` +
   `modules/crates/store/src/migrations.rs`). See
   [schema-migrations.md](schema-migrations.md).
4. **At-rest encryption layer** (`modules/crates/store/src/crypto.rs`). See
   [at-rest-encryption.md](at-rest-encryption.md).
5. **Startup gate**: `SurrealStore::open_embedded` runs every embedded
   migration automatically; a failed migration surfaces as
   [`StoreError::Migration`](../../../../../../modules/crates/store/src/lib.rs)
   and the server refuses to start — fail-safe.

**P2 repository + type safety:**

6. **Type-safe ownership edges** (`modules/crates/domain/src/model/principal_resource.rs`):
   sealed `Principal` / `Resource` marker traits + typed
   `Edge::new_owned_by` / `new_created` / `new_allocated_to` constructors
   + 6 `trybuild` compile-fail fixtures. See
   [ADR-0015](../decisions/0015-type-safe-ownership-edges.md).
7. **Repository trait expansion** (`modules/crates/domain/src/repository.rs`):
   35 object-safe methods + 3 typed free-function wrappers. See
   [storage-and-repository.md](storage-and-repository.md).
8. **SurrealStore implementation** (`modules/crates/store/src/repo_impl.rs`):
   full CRUD against embedded SurrealDB via `type::thing(...)` record ids
   + `CONTENT $body` pattern + per-type row translators for rich types
   (Grant, AuthRequest).
9. **In-memory Repository fake**
   (`modules/crates/domain/src/in_memory.rs`): feature-gated HashMap
   impl. Used by the M0 server health tests and the P3/P4 proptests to
   follow.

**P3 Permission Check engine:**

10. **Permission Check engine** (`modules/crates/domain/src/permissions/`):
    8-file module implementing the full 6-step (+Step 2a) pipeline as a
    **pure** `check()` free function. Decision shape is a three-valued
    enum (`Allowed` / `Denied { failed_step }` / `Pending`); the engine
    records latency + result label via a caller-supplied
    `PermissionCheckMetrics` trait (no `prometheus` dependency in
    domain). See [permission-check-engine.md](permission-check-engine.md)
    + [ADR-0008](../decisions/0008-permission-check-as-pipeline.md).
11. **Engine proptest coverage**: 6 files under
    `modules/crates/domain/tests/permission_check_*_props.rs` +
    `permission_check_worked_trace.rs` — 14 invariants × 256 cases
    default ≈ 3,584 random-input branches per CI run. Canonical
    coverage: Step-0 catalogue precondition, Step-2 empty-grants,
    Step-3 no-matching-grant, Step-4 constraint satisfaction, Step-6
    consent gating, pipeline monotonicity (adding unrelated grants /
    ceilings / revoked grants never widens Allowed), and the
    concept-doc worked trace (`bash cargo build` → Allowed;
    `bash rm -rf /` without filesystem grant → Step 3 denial).

**P4 Auth Request state machine:**

12. **9-state lifecycle** (`modules/crates/domain/src/auth_requests/`):
    4-file module implementing the full
    `Draft → Pending → InProgress → {Approved | Denied | Partial | Expired | Revoked | Cancelled}`
    lifecycle as pure `Result`-returning transition functions.
    Aggregation flows upward (approver-slot state → resource state →
    request state); stored state refreshes from slot truth on every
    mutation. See
    [auth-request-state-machine.md](auth-request-state-machine.md) +
    [ADR-0010](../decisions/0010-per-slot-aggregation.md).
13. **Revocation is forward-only** with audit-event emission
    ([`revocation.rs`](../../../../../../modules/crates/domain/src/auth_requests/revocation.rs)):
    `revoke` returns the `Alerted` `auth_request.revoked` event
    pre-built for the `AuditEmitter` to persist. Cascading grant
    revocation is deferred to P5 bootstrap (a repository concern).
14. **Two-tier retention math**
    ([`retention.rs`](../../../../../../modules/crates/domain/src/auth_requests/retention.rs)):
    `active_until`, `is_archive_eligible`, `days_remaining` pure fns
    implement the 90-day active window + archived tier.
15. **Proptest coverage**: 4 files under
    `modules/crates/domain/tests/auth_request_*_props.rs` — 15
    invariants covering aggregation correctness, illegal-transition
    rejection, slot independence, revocation forward-monotonicity,
    and active-window monotonic non-increase.

**P5 System Bootstrap flow (s01):**

16. **`server/src/bootstrap/`** — 4-file module:
    - [`init.rs`](../../../../../../modules/crates/server/src/bootstrap/init.rs):
      generates 32 CSPRNG bytes → base64url → `bphi-bootstrap-` prefix,
      stores argon2id hash via `Repository::put_bootstrap_credential`.
    - [`credential.rs`](../../../../../../modules/crates/server/src/bootstrap/credential.rs):
      argon2id PHC-encoded hashing + constant-time verification.
    - [`claim.rs`](../../../../../../modules/crates/server/src/bootstrap/claim.rs):
      handler-free `execute_claim(repo, ClaimInput) -> Result<ClaimOutcome, ClaimError>`
      implementing R-SYS-s01-1 … R-SYS-s01-6 + R-ADMIN-01-W1 … W4.
    - `--bootstrap-init` subcommand on the `baby-phi-server` binary
      (clap-based); prints the plaintext once to stdout.
17. **Atomic `Repository::apply_bootstrap_claim`**: new trait method
    wrapping the seven writes (Human Agent + Channel + Inbox + Outbox +
    Auth Request + Grant + Audit Event) + N catalogue seeds + credential
    consumption in a single `BEGIN/COMMIT TRANSACTION` envelope
    (SurrealStore) / single write-lock region (in-memory fake). Rollback
    on any inner-query error; credential stays unconsumed for retry.
    See [bootstrap-flow.md](bootstrap-flow.md) +
    [ADR-0011](../decisions/0011-bootstrap-credential-single-use.md).
18. **Test coverage**: 13 server unit tests (hashing round-trip,
    credential generation entropy, claim happy-path, invalid / consumed
    / already-claimed / empty-input rejections) + 3 store integration
    tests (atomic commit; transaction rollback on agent-id collision;
    `list_bootstrap_credentials` filter contract). 16 tests added at
    P5; workspace was at 260 passing / 0 failed after this phase.
    (Post-P7 total is 288 — see the P7 row of the Testing posture
    table below.)

**P6 HTTP endpoints + session cookie:**

19. **`server/src/handlers/bootstrap.rs`** — thin axum shim for the
    business logic: `GET /api/v0/bootstrap/status` returns 200 with
    `{claimed, admin_agent_id?}`; `POST /api/v0/bootstrap/claim` runs
    [`execute_claim`](../../../../../../modules/crates/server/src/bootstrap/claim.rs)
    and maps [`ClaimRejection`] → HTTP status (400 / 403 / 409) with a
    stable `{code, message}` error envelope. See
    [server-topology.md](server-topology.md) +
    [http-api-reference.md](../user-guide/http-api-reference.md).
20. **`server/src/session.rs`** — HS256 JWT signed with
    `session.secret`, packaged into a `baby_phi_session` cookie
    (`HttpOnly`, `SameSite=Lax`, `Secure` per config). `sub` is the
    admin's `agent_id` UUID; `exp` defaults to 12h. Set on a 201
    response; verified via
    [`verify_from_cookies`](../../../../../../modules/crates/server/src/session.rs)
    on every authenticated route (routes that consume the cookie land
    in M2+). Server-side session rows + revocation are M3.
21. **Prometheus counter** `baby_phi_bootstrap_claims_total{result}`
    via the `metrics` facade crate; labels cover
    `success | invalid | already_consumed | already_claimed |
    validation | internal`. Registered at claim time; exposed on
    `/metrics` by the existing `with_prometheus` wrapper.
22. **Router + config extensions**: `CookieManagerLayer` wired in
    [`build_router`](../../../../../../modules/crates/server/src/router.rs);
    new `[session]` config section loaded via
    [`SessionConfig`](../../../../../../modules/crates/server/src/config.rs)
    with a length-gated `SessionKey::from_config` constructor.
23. **Test coverage**: 6 session unit tests (sign/verify roundtrip;
    wrong-sig rejection; garbage-token rejection; expired-token
    rejection; short-secret config guard; 32-byte-secret accepted)
    + 9 handler integration tests (status unclaimed / claimed; claim
    201 with cookie / 400 / 403-invalid / 403-consumed / 409;
    malformed JSON; missing channel) + 3 session-cookie integration
    tests (cookie signs + verifies; wrong-secret rejects; status
    endpoint sets no cookie). Total 18 new tests; workspace now at
    278 passing / 0 failed.

**P7 CLI subcommands:**

24. **`baby-phi` binary migration** (`modules/crates/cli/src/`): replaced
    the legacy single-use phi-core demo loop with a clap subcommand
    tree.
    - [`main.rs`](../../../../../../modules/crates/cli/src/main.rs):
      top-level `baby-phi bootstrap {status,claim}` + `baby-phi agent
      demo` commands + `--server-url` global flag (bound to the
      `BABY_PHI_API_URL` env var).
    - [`commands/bootstrap.rs`](../../../../../../modules/crates/cli/src/commands/bootstrap.rs):
      `reqwest`-based clients for both HTTP endpoints. Maps HTTP
      outcomes to stable CLI exit codes (0 success / 1 transport / 2
      rejected / 3 internal) so shell scripts can distinguish
      "retry later" from "fix input" from "escalate".
    - [`commands/agent.rs`](../../../../../../modules/crates/cli/src/commands/agent.rs):
      preserves the phi-core agent-loop demo verbatim behind the new
      subcommand. Still reads `baby-phi/config.toml` for now;
      retiring that reader is an M2+ cleanup item.
25. **URL resolution precedence**: `--server-url` (flag or
    `BABY_PHI_API_URL` env) wins over the layered
    [`ServerConfig::load()`](../../../../../../modules/crates/server/src/config.rs)
    fallback. A bind-all `0.0.0.0` config host is rewritten to
    `127.0.0.1` so the CLI never dials a bind-only address.
26. **Test coverage**: 3 unit tests (URL trailing-slash normalisation)
    + 7 CLI integration tests that boot the axum router on a random
    local port against `InMemoryRepository`, shell out to the built
    `baby-phi` binary via `CARGO_BIN_EXE_baby-phi`, and assert exit
    code + stdout shape for status (unclaimed/claimed), claim (happy
    201 shape, 403-invalid rejected, 409 already-claimed rejected),
    the transport-error path, and the `agent demo` subcommand's
    graceful failure when `config.toml` is absent. Workspace now at
    288 passing / 0 failed. (Post-P7 re-audit added 4 hash-chain
    proptest invariants in `audit_hash_chain_props.rs` for row C6 of
    the verification matrix, bringing the workspace to 292 — see
    §Post-P7 audit remediation.)

## Post-P7 audit remediation

A post-P7 independent re-audit flagged five doc / test issues before
P8 opened. All were resolved in a single remediation pass that made
no breaking code changes:

1. **Repository method count**: overview + storage-and-repository
   said "33-method trait" throughout, but P5 and P6 had added
   `apply_bootstrap_claim` + `list_bootstrap_credentials` bringing
   the total to 35. Every occurrence updated; the
   [storage-and-repository](storage-and-repository.md) trait-surface
   table now groups the two new methods explicitly.
2. **P6 session-test breakdown**: the P6 note said "5 session unit
   tests" but the actual count was 6 (an accept-32-byte-secret test
   was missed in the narrative). Fixed in both overview and
   [server-topology](server-topology.md).
3. **Stale P5 sentence**: P5's narrative ended with "workspace now at
   260 passing"; refreshed to make clear that 260 was the closing
   count at P5 time and the current total is 292 (see Testing
   posture).
4. **Testing-posture table arithmetic**: the trybuild row quoted `6`
   compile-fail fixtures but cargo reports the runner as `1` test;
   worked-trace + doctests (4 tests) weren't in any row. Reworked to
   make the row sum equal the cargo total, with a Gains column
   preserving the more-meaningful fixture count.
5. **C6 verification-matrix coverage**: the plan's Part-8 matrix
   calls for "unit + proptest" on the per-org audit hash-chain. Only
   unit-level tests existed in `audit.rs`. Added
   [`domain/tests/audit_hash_chain_props.rs`](../../../../../../modules/crates/domain/tests/audit_hash_chain_props.rs)
   with 4 invariants:
   - hash is independent of `prev_event_hash` (chain self-reference guard)
   - tampering any captured field changes the hash
   - `org_scope` is captured in the hash (tenant-chain isolation)
   - a two-event chain detects a tampered predecessor

After remediation: **292 tests pass / 0 fail**; fmt, clippy, doc-links,
spec-drift all green.

## Crate DAG

| Crate | Purpose | P1 additions |
|---|---|---|
| [`domain`](../../../../../../modules/crates/domain/) | Graph model + Permission Check engine + Auth Request state machine | `model/` submodule (5 files), `audit.rs`, blake3 dep |
| [`store`](../../../../../../modules/crates/store/) | SurrealDB (RocksDB) adapter | `migrations.rs`, `crypto.rs`, `migrations/0001_initial.surql`, aes-gcm + base64 + rand deps |
| [`server`](../../../../../../modules/crates/server/) | axum HTTP surface + bootstrap handlers + session cookie | `bootstrap/` (M1/P5), `handlers/bootstrap.rs` + `session.rs` (M1/P6) |
| [`cli`](../../../../../../modules/crates/cli/) | clap CLI — `baby-phi bootstrap {status,claim}` + `agent demo` | `commands/bootstrap.rs` + `commands/agent.rs` + `reqwest`/`server`/`serde` deps (M1/P7) |

## Testing posture (after P7)

Values are **`cargo test` pass counts** — the number you get by running
`cargo test --workspace` at the close of each phase. The trybuild row
is `1` because the runner invokes 6 compile-fail fixtures through a
single `#[test]`; the fixture count (6) is quoted in the Gains column
so the more meaningful figure is still visible.

| Layer | P1 | P2 | P3 | P3+ (widening) | P4 | P5 | P6 | P7 | P7+C6 | Gains |
|---|---|---|---|---|---|---|---|---|---|---|
| Domain unit (model + audit + marker-trait impls + typed constructors + permissions helpers + auth-request state/transitions/revocation/retention) | 27 | 36 | 91 | 91 | 134 | 134 | 134 | 134 | 134 | — |
| Domain proptest (permission-check + auth-request + audit-hash-chain invariants) | 0 | 0 | 14 | 14 | 29 | 29 | 29 | 29 | 33 | 11 files |
| Domain compile-fail runner (`trybuild`) | 0 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | drives 6 compile-fail fixtures |
| Domain worked-trace + doctests (`permission_check_worked_trace` + `permissions::selector`) | 0 | 0 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | — |
| Store unit (crypto + migrations) | 13 | 13 | 13 | 13 | 13 | 13 | 13 | 13 | 13 | — |
| Store integration (migrations + crypto vault + repository) | 2 | 29 | 29 | 59 | 59 | 62 | 62 | 62 | 62 | — |
| Server unit (bootstrap credential + init + claim business logic + session sign/verify) | 0 | 0 | 0 | 0 | 0 | 13 | 19 | 19 | 19 | — |
| Server integration (M0 health + TLS + P6 bootstrap handlers + session cookie) | 4 | 4 | 4 | 4 | 4 | 4 | 16 | 16 | 16 | — |
| CLI unit (URL normalisation) | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 3 | — |
| CLI integration (end-to-end: spawn server in-process + shell out to `baby-phi`) | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 7 | 7 | — |
| **Runnable total** | **46** | **82** | **151** | **186** | **244** | **260** | **278** | **288** | **292** | — |

The P7+C6 column sums row-wise to **292**, matching the `cargo test
--workspace` grand total exactly. The "+C6" tranche is the
post-audit `audit_hash_chain_props.rs` file (4 proptest invariants)
added to close commitment-ledger row C6. Historical columns (P2 / P3)
predate the accounting convention used here and carry a ≤6-test
narrative drift from row-wise addition; their stated totals **are**
the cargo counts that were observed at the close of each phase.

P4 adds 43 domain-unit tests (aggregation, transitions, revocation,
retention) and 15 proptest invariants across 4 new files. Total domain
proptest invariants now **29** across 10 files — ≈ 7,424 random-input
branches per CI run at the default `PROPTEST_CASES=256`, or ≈ 29,000
branches at `PROPTEST_CASES=1000`. Specifically:

| Phase | Proptest files | Invariants |
|---|---|---|
| P3 | 6 `permission_check_*` | 14 |
| P4 | 4 `auth_request_*` | 15 |

P2 shipped 29 store integration tests; a post-P3 independent re-audit
identified the P2 repository surface as under-tested relative to the
plan's ≈92-test budget. The **P3+ widening pass** progressed in two
stages:

1. **Breadth pass** — 23 new integration tests covering: error paths
   (duplicate-id rejection, missing-id no-op semantics, idempotent
   revocation), multi-field round-trips (every `PrincipalRef` variant
   as a grant holder, multi-action grants, grants with `descends_from`
   provenance, multi-slot / multi-approver Auth Requests, full-field
   audit events), cross-cutting semantics (case-sensitive catalogue
   lookup, kind-metadata persistence, archive-flip drops from active
   listings, edge upserts produce distinct ids).
2. **Audit follow-up** — an Explore-agent re-audit flagged two
   remaining weaknesses: (a) two bulk "all create green" tests that
   did not verify persistence via `get_*`, and (b) the `ping` health
   surface was untested. Both were addressed: the bulk tests were
   split into 9 focused tests that each assert row persistence and
   key field preservation via direct SurrealDB `count()` /
   field-projection queries, and `ping_returns_ok_on_fresh_store`
   was added.

Total store integration count after the widening pass was **59** (57
repository + 1 migrations + 1 crypto vault) — substantive coverage
across every one of the then-33 trait methods + 3 free-function
wrappers + the `ping` surface, with strong error-path parity on the
critical paths (revocation, update, list filters, cross-scope
isolation). P5 and P6 extended the trait to **35 methods** and added
3 bootstrap-flow integration tests, so the current store integration
count is **62**.

## What to read next

- [graph-model.md](graph-model.md) — the 9/8/37/66 inventory and how it's
  organised in Rust.
- [schema-migrations.md](schema-migrations.md) — the forward-only runner.
- [audit-events.md](audit-events.md) — the audit event shape and hash-chain
  seed.
- [at-rest-encryption.md](at-rest-encryption.md) — envelope encryption for
  the secrets vault.
- [permission-check-engine.md](permission-check-engine.md) — P3's
  6-step (+2a) pipeline with ASCII diagram, module layout, and
  proptest coverage map.
- [auth-request-state-machine.md](auth-request-state-machine.md) —
  P4's 9-state lifecycle, aggregation tables, transition API,
  revocation/retention semantics, and proptest coverage map.
- [bootstrap-flow.md](bootstrap-flow.md) — P5's `bootstrap-init`
  subcommand + atomic s01 claim flow with rollback contract.
- [server-topology.md](server-topology.md) — P6's HTTP surface + the
  signed session cookie + the bootstrap claim metric.
- [http-api-reference.md](../user-guide/http-api-reference.md) — the
  `/api/v0/bootstrap/*` request + response contract the admin sees.
- [cli-usage.md](../user-guide/cli-usage.md) — P7's
  `baby-phi bootstrap {status,claim}` + `agent demo` reference.
- The M0 companion pages (one folder up,
  [`../../m0/`](../../m0/README.md)) for everything P1 builds upon.
