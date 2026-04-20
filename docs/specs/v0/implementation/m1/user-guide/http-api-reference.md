<!-- Last verified: 2026-04-20 by Claude Code -->

# HTTP API reference — `/api/v0/bootstrap/*`

Two endpoints land in M1/P6. Both are unauthenticated on the request
side; `POST /api/v0/bootstrap/claim` sets a signed session cookie on
success. The handler maps business-logic rejections from
[`execute_claim`](../../../../../../modules/crates/server/src/bootstrap/claim.rs)
to stable machine-readable codes (see
[architecture/server-topology.md §Error envelope](../architecture/server-topology.md#error-envelope)).

## `GET /api/v0/bootstrap/status`

Probes whether a platform admin has been claimed. Always 200 OK; the
body carries the boolean.

### Response — unclaimed

```http
HTTP/1.1 200 OK
Content-Type: application/json

{ "claimed": false, "awaiting_credential": true }
```

### Response — claimed

```http
HTTP/1.1 200 OK
Content-Type: application/json

{ "claimed": true, "admin_agent_id": "a7b01e03-…" }
```

### Failure modes

- **500** with `{"code": "INTERNAL_ERROR"}` if the repository is
  unreachable. The web `/bootstrap` page treats this as "try again in a
  moment"; the CLI surfaces the error verbatim.

## `POST /api/v0/bootstrap/claim`

Consumes the single-use bootstrap credential and materialises the
platform admin via the seven-writes atomic s01 flow
(see [architecture/bootstrap-flow.md](../architecture/bootstrap-flow.md)).

### Request

```http
POST /api/v0/bootstrap/claim HTTP/1.1
Content-Type: application/json

{
  "bootstrap_credential": "bphi-bootstrap-…",
  "display_name": "Alex Chen",
  "channel": { "kind": "slack", "handle": "@alex" }
}
```

`channel.kind` is one of `"slack" | "email" | "web"`. The body is
deserialised by
[`ClaimRequest`](../../../../../../modules/crates/server/src/handlers/bootstrap.rs);
unknown fields are rejected by Serde's default behaviour.

### Response — 201 Created

```http
HTTP/1.1 201 Created
Content-Type: application/json
Set-Cookie: baby_phi_session=<jwt>; HttpOnly; SameSite=Lax; Path=/; Expires=…; Secure

{
  "human_agent_id": "…",
  "inbox_id": "…",
  "outbox_id": "…",
  "grant_id": "…",
  "bootstrap_auth_request_id": "…",
  "audit_event_id": "…"
}
```

The `Secure` attribute is driven by `session.secure` in config
(default `true`; `config/dev.toml` flips it off for plaintext
localhost). `Expires` lands at `iat + session.ttl_seconds` (default
12h).

### Response — 400 Bad Request

```json
{ "code": "VALIDATION_FAILED", "message": "display_name must not be empty" }
```

Emitted when `display_name`, `channel.handle`, or `bootstrap_credential`
is empty after trimming whitespace. The Serde extractor also returns
4xx (usually 400 or 422) for malformed JSON or missing fields — those
responses come from axum and do not carry the `code` envelope.

### Response — 403 Forbidden — `BOOTSTRAP_INVALID`

```json
{ "code": "BOOTSTRAP_INVALID", "message": "bootstrap credential is not recognised" }
```

Emitted when the supplied credential does not verify any stored
argon2id hash. Includes the case where no bootstrap credential has been
persisted at all.

### Response — 403 Forbidden — `BOOTSTRAP_ALREADY_CONSUMED`

```json
{ "code": "BOOTSTRAP_ALREADY_CONSUMED", "message": "bootstrap credential has already been consumed" }
```

The plaintext matches a stored hash, but the row carries a non-null
`consumed_at`. Usually the admin is retrying a claim after the first
one succeeded; see the 409 path below.

### Response — 409 Conflict — `PLATFORM_ADMIN_CLAIMED`

```json
{ "code": "PLATFORM_ADMIN_CLAIMED", "message": "a platform admin has already been claimed" }
```

A Human-kind agent already exists. This is the first-line check inside
[`execute_claim`](../../../../../../modules/crates/server/src/bootstrap/claim.rs);
it fires before the credential scan so presenting any plaintext after
the first success yields the 409 rather than a 403. (The 403 path
remains reachable via the edge case where the credential is consumed
without an admin — e.g. a rollback that succeeded at the application
layer but left the `consumed_at` stamped.)

### Response — 500 Internal Server Error

```json
{ "code": "INTERNAL_ERROR", "message": "an internal error occurred" }
```

Covers repository failures, argon2 verification errors, and JWT
signing failures. The structured error is logged server-side; the HTTP
body deliberately omits details (no leaking implementation state to
unauthenticated callers).

## Session cookie

Every `201 Created` response carries a `Set-Cookie: baby_phi_session=…`
header. The cookie's value is an HS256 JWT signed with
`session.secret`:

```json
{
  "sub": "<human_agent_id UUID>",
  "iat": 1713600000,
  "exp": 1713643200
}
```

Subsequent requests (once M2 adds authenticated routes) attach the
cookie via the `Cookie` header; the server validates it with
[`verify_from_cookies`](../../../../../../modules/crates/server/src/session.rs).

## Rate limiting

Not implemented in M1 (row R-NFR-observability-4 schedules it for
M7b). An unauthenticated caller can hammer `POST /bootstrap/claim`
with arbitrary credentials; the 403 response path runs one argon2id
verification per call, which is a natural throttle (~100 ms/verify on
commodity hardware). M7b adds a hard cap.

## Cross-references

- [requirements/admin/01 §10](../../../requirements/admin/01-platform-bootstrap-claim.md#10-api-contract-sketch) —
  the contract this page lands.
- [architecture/server-topology.md](../architecture/server-topology.md) —
  middleware stack + metric surface.
- [architecture/bootstrap-flow.md](../architecture/bootstrap-flow.md) —
  the seven atomic writes behind a 201.
- [ADR-0011](../decisions/0011-bootstrap-credential-single-use.md) —
  why the credential is single-use and never echoed on wire.
- [first-bootstrap.md](first-bootstrap.md) — end-to-end walkthrough the
  admin follows.
