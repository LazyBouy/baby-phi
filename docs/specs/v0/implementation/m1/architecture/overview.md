<!-- Last verified: 2026-04-20 by Claude Code -->

# M1 architecture — overview

M1 turns the M0 scaffolding into a functional Permission Check spine. This
page is the system map; depth pages cover each subsystem.

## System map (P1–P9 landed — M1 sealed)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          web (Next.js 14 / SSR)                         │
│     ✓ /bootstrap — SSR probe + claim Server Action + session cookie     │
├─────────────────────────────────────────────────────────────────────────┤
│                       cli (clap; phi binary)                       │
│     ✓ bootstrap — `phi bootstrap {status,claim}` (HTTP client)     │
│     ✓ agent     — `phi agent demo` (legacy phi-core demo loop)     │
├─────────────────────────────────────────────────────────────────────────┤
│                 server (axum; phi-server binary)                   │
│   existing: /healthz/live /healthz/ready /metrics                       │
│     ✓ bootstrap — `bootstrap-init` subcommand + atomic claim logic      │
│     ✓ handlers  — GET /api/v0/bootstrap/status + POST /bootstrap/claim  │
│     ✓ session   — HS256 signed `phi_kernel_session` cookie + verify       │
│     ✓ metric    — phi_bootstrap_claims_total{result}               │
├─────────────────────────────────────────────────────────────────────────┤
│                              domain                                     │
│   ✓ model/   — 9 fundamentals, 8 composites, 37 nodes, 66 edges         │
│   ✓ model/principal_resource — sealed Principal/Resource marker traits  │
│   ✓ audit    — event shape, class tiers, hash-chain helper              │
│   ✓ repository — 36-method trait + 3 typed free-function wrappers       │
│   ✓ in_memory — HashMap-backed Repository fake (feature-gated)          │
│   ✓ permissions — 6-step (+2a) engine, pure fn, metric-instrumented     │
│   ✓ auth_requests — 9-state machine + aggregation + retention + revoke  │
├─────────────────────────────────────────────────────────────────────────┤
│                               store                                     │
│   ✓ SurrealStore::open_embedded runs migrations on startup              │
│   ✓ migrations — forward-only runner with startup-gate fail-safe        │
│   ✓ crypto — AES-GCM envelope for secrets_vault                         │
│   ✓ repo_impl — full SurrealDB impl of all 36 Repository methods        │
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

## What P1 + P2 + P3 + P4 + P5 + P6 + P7 + P8 + P9 delivered

**P1 foundation:**

1. **Graph model** (`modules/crates/domain/src/model/`). See
   [graph-model.md](graph-model.md) for the full inventory.
2. **Audit-event skeleton** (`modules/crates/domain/src/audit/mod.rs`). See
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
   36 object-safe methods + 3 typed free-function wrappers. See
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
    - `--bootstrap-init` subcommand on the `phi-server` binary
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
    `session.secret`, packaged into a `phi_kernel_session` cookie
    (`HttpOnly`, `SameSite=Lax`, `Secure` per config). `sub` is the
    admin's `agent_id` UUID; `exp` defaults to 12h. Set on a 201
    response; verified via
    [`verify_from_cookies`](../../../../../../modules/crates/server/src/session.rs)
    on every authenticated route (routes that consume the cookie land
    in M2+). Server-side session rows + revocation are M3.
21. **Prometheus counter** `phi_bootstrap_claims_total{result}`
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

24. **`phi` binary migration** (`modules/crates/cli/src/`): replaced
    the legacy single-use phi-core demo loop with a clap subcommand
    tree.
    - [`main.rs`](../../../../../../modules/crates/cli/src/main.rs):
      top-level `phi bootstrap {status,claim}` + `phi agent
      demo` commands + `--server-url` global flag (bound to the
      `PHI_API_URL` env var).
    - [`commands/bootstrap.rs`](../../../../../../modules/crates/cli/src/commands/bootstrap.rs):
      `reqwest`-based clients for both HTTP endpoints. Maps HTTP
      outcomes to stable CLI exit codes (0 success / 1 transport / 2
      rejected / 3 internal) so shell scripts can distinguish
      "retry later" from "fix input" from "escalate".
    - [`commands/agent.rs`](../../../../../../modules/crates/cli/src/commands/agent.rs):
      preserves the phi-core agent-loop demo verbatim behind the new
      subcommand. Still reads `phi/config.toml` for now;
      retiring that reader is an M2+ cleanup item.
25. **URL resolution precedence**: `--server-url` (flag or
    `PHI_API_URL` env) wins over the layered
    [`ServerConfig::load()`](../../../../../../modules/crates/server/src/config.rs)
    fallback. A bind-all `0.0.0.0` config host is rewritten to
    `127.0.0.1` so the CLI never dials a bind-only address.
26. **Test coverage**: 3 unit tests (URL trailing-slash normalisation)
    + 7 CLI integration tests that boot the axum router on a random
    local port against `InMemoryRepository`, shell out to the built
    `phi` binary via `CARGO_BIN_EXE_phi`, and assert exit
    code + stdout shape for status (unclaimed/claimed), claim (happy
    201 shape, 403-invalid rejected, 409 already-claimed rejected),
    the transport-error path, and the `agent demo` subcommand's
    graceful failure when `config.toml` is absent. Workspace now at
    288 passing / 0 failed. (Post-P7 re-audit added 4 hash-chain
    proptest invariants in `audit_hash_chain_props.rs` for row C6 of
    the verification matrix, bringing the workspace to 292 — see
    §Post-P7 audit remediation.)

**P8 Web `/bootstrap` page:**

27. **`modules/web/app/bootstrap/`** — the first real human-facing UI
    route. Composed of three files:
    - [`page.tsx`](../../../../../../modules/web/app/bootstrap/page.tsx) —
      Server Component; SSR-probes `GET /api/v0/bootstrap/status` and
      branches to a claim form (unclaimed), a terminal view (claimed),
      or a server-unreachable view. `export const dynamic =
      "force-dynamic"` to avoid caching across admin-state flips.
    - [`actions.ts`](../../../../../../modules/web/app/bootstrap/actions.ts) —
      Server Action `submitClaim(prev, formData)` that validates
      input, calls `postBootstrapClaim`, and — on a 201 response —
      forwards the `Set-Cookie` the Rust server issued by parsing it
      with `extractSessionJwt` and re-emitting via
      `next/headers → cookies().set(...)` so the browser carries the
      session cookie on every subsequent request.
    - [`ClaimForm.tsx`](../../../../../../modules/web/app/bootstrap/ClaimForm.tsx) —
      Client component using React 18's `useFormState` +
      `useFormStatus` (the Next 14.2 idiom) for inline rerender of
      the Server Action result.
28. **`lib/` helpers extended**:
    - [`api.ts`](../../../../../../modules/web/lib/api.ts) now exports
      `getBootstrapStatus`, `postBootstrapClaim`, plus three pure
      translators (`parseStatusBody`, `parseClaimSuccess`,
      `extractSessionJwt`) — split out so unit tests can exercise
      them without a live HTTP server.
    - [`session.ts`](../../../../../../modules/web/lib/session.ts) reads
      the signed cookie via `next/headers` and delegates verification
      to [`session-verify.ts`](../../../../../../modules/web/lib/session-verify.ts),
      the pure HS256 JWT verifier (jose-based). The split lets
      Node's built-in test runner exercise `verifySessionToken`
      without standing up a Next.js runtime.
29. **Session-cookie roundtrip** between Rust (P6) and Web (P8):
    the Rust server signs the JWT; the browser carries it; the Next
    web process verifies it with the shared
    `PHI_SESSION_SECRET` (≥ 32 bytes enforced on both sides).
    Dev-placeholder secrets match so `npm run dev` + `phi-server`
    play nicely out of the box.
30. **Test coverage**: 9 `api.test.ts` invariants (status parsing for
    claimed/unclaimed/defensive; claim-success snake→camel map;
    `extractSessionJwt` across four Set-Cookie shapes) + 5
    `session.test.ts` invariants (valid token; wrong-secret;
    garbage token; expired token; empty token). Zero-dep: Node 22's
    built-in `--test` runner + `--experimental-strip-types`.
31. **Manual smoke** at P8 close: minted a credential, booted server
    and `npm run dev`, curled `/bootstrap` on an unclaimed install
    (renders the claim form), POSTed directly to the HTTP claim
    endpoint to flip state, re-fetched `/bootstrap` (renders
    "Platform admin already claimed" with the correct UUID).
    Browser-based Playwright coverage is scheduled for M7b.

**P9 Acceptance harness + final seal:**

32. **Acceptance harness**
    ([`server/tests/acceptance_common/mod.rs`](../../../../../../modules/crates/server/tests/acceptance_common/mod.rs)):
    boots a **real** `axum` app against a **real** embedded SurrealDB
    in a fresh tempdir per test, binds to a random loopback port,
    returns a `reqwest` client. Tests are E2E — no in-memory fakes,
    no mocked HTTP. The Prometheus layer is installed once per
    process via a `OnceLock` so the `/metrics` test can coexist
    with the non-metric tests in the same binary.
33. **Acceptance scenarios**
    ([`server/tests/acceptance_bootstrap.rs`](../../../../../../modules/crates/server/tests/acceptance_bootstrap.rs)):
    5 E2E tests covering the Part-8 verification-matrix rows C4 / C5
    / C9 / C10 / C11 / C13:
    - **`fresh_install_happy_claim`** (C4 + C5 + C9 + C11) — status
      unclaimed → claim succeeds → status claimed; admin agent row
      persisted; grant row with `[allocate]`-on-`system:root`,
      delegable, correct `descends_from`; auth-request in `Approved`
      with `audit_class = Alerted`; catalogue contains `system:root`;
      credential consumed; session cookie emitted.
    - **`wrong_credential_rejects_403_bootstrap_invalid`** (C5) —
      403 with the stable code; no admin created; credential
      untouched; audit chain empty.
    - **`reused_credential_without_admin_rejects_403_already_consumed`** (C5)
      — the edge case where `consumed_at` is stamped but no admin
      exists; correct stable code.
    - **`second_claim_after_success_rejects_409_platform_admin_claimed`** (C5)
      — first-line admin-check beats credential scan.
    - **`metrics_endpoint_exposes_bootstrap_claims_counter`** (C10) —
      `/metrics` scrape exposes `phi_bootstrap_claims_total`
      with `result="success"` after a 201.
34. **Operations runbooks**:
    [schema-migrations](../operations/schema-migrations-operations.md),
    [at-rest-encryption](../operations/at-rest-encryption-operations.md),
    [bootstrap-credential-lifecycle](../operations/bootstrap-credential-lifecycle.md),
    [audit-log-retention](../operations/audit-log-retention.md) — each
    covers the operator-facing workflow, what M1 ships, what's
    deferred to M7b, and the recovery paths.
35. **Troubleshooting reference**
    ([user-guide/troubleshooting.md](../user-guide/troubleshooting.md))
    — grepable error-code table (HTTP codes + CLI exit codes +
    web-specific symptoms) with explicit recovery steps.
36. **CI workflow extension** (`.github/workflows/rust.yml`): two
    new jobs sit alongside the existing `fmt` / `clippy` / `test` /
    `audit` / `deny` pipeline:
    - `proptest` — runs domain tests with `PROPTEST_CASES=256` and
      `--test-threads 1` so proptest's failure-reduction is
      deterministic.
    - `acceptance` — runs the 5 acceptance scenarios in **release
      profile** so any optimisation-only regressions surface here
      before reaching prod.
37. **Workspace totals after P9**: 297 cargo tests pass (292 pre-P9
    + 5 acceptance) + 14 web tests = 311 total / 0 failed. The
    post-P9 100 %-audit pass further raised this to **299 + 14 =
    313 total / 0 failed** by adding `get_audit_event` + its two
    store integration tests — see §Post-P9 100 %-audit remediation.

## Post-P9 100%-audit remediation

A final independent re-audit after P9 flagged three cosmetic items
that were keeping confidence at 99 % rather than 100 %. The fixes:

1. **`actions.ts` regex DRY.** The Server Action that forwards the
   session cookie on a successful claim was re-implementing
   `extractSessionJwt`'s regex inline at
   [`app/bootstrap/actions.ts:91`](../../../../../../modules/web/app/bootstrap/actions.ts).
   Now imports and calls `extractSessionJwt` from `lib/api.ts` —
   single regex, single source of truth.
2. **Acceptance harness: bounded poll in place of fixed sleep.** The
   harness previously waited a hardcoded 50 ms after spawning the
   axum task before returning the `Acceptance` handle. Replaced with
   [`wait_until_serving`](../../../../../../modules/crates/server/tests/acceptance_common/mod.rs),
   which polls `/healthz/live` on a 10 ms cadence until a 2xx is
   observed (5 s deadline). Every acceptance test is now
   deterministically race-free on slow machines and wastes no time
   on fast ones.
3. **`get_audit_event` added to Repository.** The P9 happy-path
   acceptance test was asserting `audit_class = Alerted` via an
   `AuthRequest.audit_class` proxy because the trait had no audit-row
   lookup by id. Added
   [`Repository::get_audit_event`](../../../../../../modules/crates/domain/src/repository.rs)
   (bringing the trait to **36 methods**), implemented in both
   `SurrealStore` (via `type::thing('audit_events', $id)` — the same
   record-id pattern every other write uses) and
   `InMemoryRepository`. The acceptance test now reads the event
   directly by id and asserts `event_type`, `audit_class`, actor,
   and provenance. 2 new store integration tests (round-trip every
   field + miss-by-id returns None) pin the behaviour.

After remediation: **299 Rust tests pass (+2), 14 Web tests pass,
313 total / 0 failed**; fmt, clippy, doc-links, spec-drift all
green.

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
| [`cli`](../../../../../../modules/crates/cli/) | clap CLI — `phi bootstrap {status,claim}` + `agent demo` | `commands/bootstrap.rs` + `commands/agent.rs` + `reqwest`/`server`/`serde` deps (M1/P7) |

## Testing posture (after P9 — M1 sealed)

Rust rows carry **`cargo test --workspace` pass counts** at the close
of each phase. Web rows carry **`npm test` pass counts** (Node 22
built-in test runner + TypeScript type-stripping). The trybuild row is
`1` because the runner invokes 6 compile-fail fixtures through a
single `#[test]`; the fixture count (6) is quoted in the Gains column
so the more meaningful figure is still visible.

| Layer | P1 | P2 | P3 | P3+ (widening) | P4 | P5 | P6 | P7 | P7+C6 | P8 | P9 | Gains |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Domain unit (model + audit + marker-trait impls + typed constructors + permissions helpers + auth-request state/transitions/revocation/retention) | 27 | 36 | 91 | 91 | 134 | 134 | 134 | 134 | 134 | 134 | 134 | — |
| Domain proptest (permission-check + auth-request + audit-hash-chain invariants) | 0 | 0 | 14 | 14 | 29 | 29 | 29 | 29 | 33 | 33 | 33 | 10 files |
| Domain compile-fail runner (`trybuild`) | 0 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1 | drives 6 compile-fail fixtures |
| Domain worked-trace + doctests (`permission_check_worked_trace` + `permissions::selector`) | 0 | 0 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | — |
| Store unit (crypto + migrations) | 13 | 13 | 13 | 13 | 13 | 13 | 13 | 13 | 13 | 13 | 13 | — |
| Store integration (migrations + crypto vault + repository) | 2 | 29 | 29 | 59 | 59 | 62 | 62 | 62 | 62 | 62 | 64 | — |
| Server unit (bootstrap credential + init + claim business logic + session sign/verify) | 0 | 0 | 0 | 0 | 0 | 13 | 19 | 19 | 19 | 19 | 19 | — |
| Server integration (M0 health + TLS + P6 bootstrap handlers + session cookie) | 4 | 4 | 4 | 4 | 4 | 4 | 16 | 16 | 16 | 16 | 16 | — |
| **Acceptance E2E (real server + embedded SurrealDB + real HTTP)** | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | **5** | new in P9 |
| CLI unit (URL normalisation) | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 3 | 3 | 3 | — |
| CLI integration (end-to-end: spawn server in-process + shell out to `phi`) | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 7 | 7 | 7 | 7 | — |
| Web unit (api wire-translators + `verifySessionToken`) | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 14 | 14 | `npm test` |
| **Runnable total** | **46** | **82** | **151** | **186** | **244** | **260** | **278** | **288** | **292** | **306** | **313** | — |

The P9 column sums row-wise to **313** — 299 Rust (`cargo test
--workspace`) + 14 Web (`npm test`). Includes the post-P9 100%-audit
pass that added `get_audit_event` (36th Repository method) + 2 store
integration tests + acceptance tightening. The "+C6" tranche is the
post-P7-audit `audit_hash_chain_props.rs` file (4 proptest invariants)
closing commitment-ledger row C6. Historical columns (P2 / P3)
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
isolation). P5 and P6 extended the trait to **35 methods** + 3
bootstrap-flow integration tests, and the final post-P9 100%-audit
pass added `get_audit_event` (bringing the trait to **36 methods**)
plus 2 more store integration tests, taking the current store
integration count to **64**.

## Configuration — `ServerConfig` vs phi-core `AgentConfig`

phi's [`ServerConfig::load()`](../../../../../../modules/crates/server/src/config.rs)
parses layered TOML (`config/default.toml` + `config/{profile}.toml`)
with per-key environment-variable overrides (`PHI__SERVER__PORT=8080`
sets `server.port`). It deserialises a fixed schema of server-infra
concerns: HTTP bind, storage directory, telemetry filter, session-cookie
secret.

phi-core's [`parse_config_file()` / `parse_config_auto()`](../../../../../../../phi-core/src/config/parser.rs)
parse TOML / YAML / JSON into phi-core's
[`AgentConfig`](../../../../../../../phi-core/src/config/schema.rs) —
an **agent blueprint** schema (provider sections, tools, execution
limits, cache policy) — with `${ENV_VAR}` substitution inside field
values.

The two parsers are **orthogonal**, not substitutable:

| Aspect | `ServerConfig::load()` | `phi_core::parse_config_file()` |
|---|---|---|
| Scope | Server infrastructure (host, port, DB path, session secret) | Agent blueprint (LLM provider, tools, profile, execution limits) |
| Override style | Per-key env-var overrides (`PHI__KEY__NESTED=value`) | `${VAR}` interpolation inside field values |
| Schema shape | phi's `ServerConfig` struct | phi-core's `AgentConfig` struct |
| Who reads it | `main.rs` on startup | `agents_from_config(&config)` when materialising agents |

**M2 page 05 (Platform Defaults) is the correct reuse point** — it
imports / exports platform defaults via `phi_core::parse_config` +
`parse_config_auto` so operators can paste a phi-core-shaped YAML into
the admin UI and have it materialise as a `PlatformDefaults` row.
`ServerConfig` stays as-is because its shape is genuinely different.

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
  `phi bootstrap {status,claim}` + `agent demo` reference.
- [web-topology.md](web-topology.md) — P8's `/bootstrap` SSR page +
  Server Action + session-cookie plumbing.
- [web-usage.md](../user-guide/web-usage.md) — end-user walkthrough
  of the `/bootstrap` page, including the error-case table.
- [troubleshooting.md](../user-guide/troubleshooting.md) — grepable
  reference for every error code M1 can emit.
- The M1 operations runbooks —
  [schema-migrations](../operations/schema-migrations-operations.md),
  [at-rest-encryption](../operations/at-rest-encryption-operations.md),
  [bootstrap-credential-lifecycle](../operations/bootstrap-credential-lifecycle.md),
  [audit-log-retention](../operations/audit-log-retention.md).
- The M0 companion pages (one folder up,
  [`../../m0/`](../../m0/README.md)) for everything P1 builds upon.
