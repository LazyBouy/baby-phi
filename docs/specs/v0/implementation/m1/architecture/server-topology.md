<!-- Last verified: 2026-04-20 by Claude Code -->

# Server topology — M1/P6 extension

M0 shipped the axum skeleton with health + metrics only (see
[`m0/architecture/server-topology.md`](../../m0/architecture/server-topology.md)).
P6 extends that skeleton with the first user-visible endpoints, a
signed-session-cookie layer, and a bootstrap-flow metric.

## Route table (current)

| Method | Path | Handler | Auth | Status |
|---|---|---|---|---|
| `GET` | `/healthz/live` | [`health::live`](../../../../../../modules/crates/server/src/health.rs) | none | M0 |
| `GET` | `/healthz/ready` | [`health::ready`](../../../../../../modules/crates/server/src/health.rs) | none | M0 |
| `GET` | `/metrics` | `with_prometheus` closure | none | M0 |
| `GET` | `/api/v0/bootstrap/status` | [`handlers::bootstrap::status`](../../../../../../modules/crates/server/src/handlers/bootstrap.rs) | none | **M1/P6** |
| `POST` | `/api/v0/bootstrap/claim` | [`handlers::bootstrap::claim`](../../../../../../modules/crates/server/src/handlers/bootstrap.rs) | none (sets cookie) | **M1/P6** |

All M2+ surfaces (orgs, agents, projects, grants, sessions, auth-requests)
nest under the same `/api/v0/*` prefix and require a valid session cookie
(enforced by a middleware that lands in M2).

## Middleware stack

```
request ──▶ CookieManagerLayer  (tower-cookies; present for every route)
        ──▶ <per-route handler>
        ◀── Set-Cookie (on successful claim only)
        ◀── Prometheus metric layer (in production binary only)
        ◀── response
```

`CookieManagerLayer` is attached in [`router::build_router`](../../../../../../modules/crates/server/src/router.rs),
not in the production-only `with_prometheus`, so every integration test
gets the same cookie-jar behaviour as the real binary.

## Session cookie

P6 ships a minimal signed-cookie session; OAuth lands in M3.

- **Cookie name** — `baby_phi_session` (configurable via `session.cookie_name`).
- **Content** — HS256 JWT with claims `{ sub, iat, exp }`. `sub` is the
  admin's `agent_id` (UUID string); `exp` is `iat + session.ttl_seconds`
  (default 12h).
- **Attributes** — `HttpOnly`, `SameSite=Lax`, `Path=/`. `Secure` defaults
  to `true`; `config/dev.toml` flips it to `false` so the cookie survives
  plaintext localhost HTTP.
- **Signing key** — `session.secret` from config. Must be ≥ 32 bytes —
  [`SessionKey::from_config`](../../../../../../modules/crates/server/src/session.rs)
  rejects anything shorter at startup. In production the secret comes from
  `BABY_PHI_SESSION__SECRET`; `config/default.toml` carries a dev-only
  placeholder that is **never** shipped.
- **Revocation** — none in M1; the TTL is the only defence. Server-side
  session rows + `POST /sessions/{id}/revoke` are the M3 follow-up.

## Error envelope

Every 4xx response from the bootstrap routes has the shape

```json
{ "code": "<STABLE_CODE>", "message": "<human explanation>" }
```

Codes currently in use:

| Code | HTTP | When | Counter label |
|---|---|---|---|
| `VALIDATION_FAILED` | 400 | Empty `display_name` / `channel_handle` / `bootstrap_credential` after trim | `validation` |
| `BOOTSTRAP_INVALID` | 403 | Supplied credential does not verify any stored hash | `invalid` |
| `BOOTSTRAP_ALREADY_CONSUMED` | 403 | Matched a stored hash, but the row is marked consumed | `already_consumed` |
| `PLATFORM_ADMIN_CLAIMED` | 409 | A platform admin already exists (first-line check) | `already_claimed` |
| `INTERNAL_ERROR` | 500 | Repository / hashing / session-signing error | `internal` |

The 409 code intentionally drops the `BOOTSTRAP_` prefix — the condition
is a platform-state observation, not a credential-state one.

## Prometheus metric

P6 registers one new counter via the `metrics` facade crate:

```
baby_phi_bootstrap_claims_total{result="success|invalid|already_consumed|already_claimed|validation|internal"}
```

Defined in [`handlers::bootstrap::CLAIMS_COUNTER`](../../../../../../modules/crates/server/src/handlers/bootstrap.rs).
`axum-prometheus` picks it up automatically once the recorder is
installed via `with_prometheus`; integration tests do not install the
recorder, so the counter is a no-op there.

The acceptance suite in P9 asserts the counter surfaces on `/metrics`
after a single successful claim (row C10 of the verification matrix).

## AppState extension

```rust
pub struct AppState {
    pub repo: Arc<dyn Repository>,       // M0
    pub session: SessionKey,             // M1/P6
}
```

`SessionKey` is cheap to clone (`EncodingKey`/`DecodingKey` wrap
`Arc<[u8]>` internally) so every handler that wants to mint a cookie can
take it from state directly.

## Test coverage

| Layer | File | Tests |
|---|---|---|
| Unit (session) | [`session.rs`](../../../../../../modules/crates/server/src/session.rs) | 6 (roundtrip; wrong-sig; garbage token; expired; short-secret guard; 32-byte-accept) |
| Integration (handler) | [`bootstrap_handler_test.rs`](../../../../../../modules/crates/server/tests/bootstrap_handler_test.rs) | 9 (status unclaimed/claimed; claim 201/400/403-invalid/403-consumed/409; malformed JSON; missing channel) |
| Integration (cookie) | [`session_cookie_test.rs`](../../../../../../modules/crates/server/tests/session_cookie_test.rs) | 3 (cookie signs + verifies; wrong-secret rejects; status endpoint sets no cookie) |

## Cross-references

- [bootstrap-flow.md](bootstrap-flow.md) — the atomic server-side flow
  [`execute_claim`](../../../../../../modules/crates/server/src/bootstrap/claim.rs)
  the P6 handler wraps.
- [ADR-0011](../decisions/0011-bootstrap-credential-single-use.md) —
  credential storage + delivery rationale.
- [requirements/admin/01 §10](../../../requirements/admin/01-platform-bootstrap-claim.md#10-api-contract-sketch) —
  the contract this page lands.
- [requirements/cross-cutting/nfr-observability.md](../../../requirements/cross-cutting/nfr-observability.md) —
  metric naming conventions.
