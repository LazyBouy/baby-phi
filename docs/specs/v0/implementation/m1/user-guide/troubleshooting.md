<!-- Last verified: 2026-04-20 by Claude Code -->

# Troubleshooting — M1 error codes & recovery

This page is the grepable reference for operator-facing error
messages produced by the M1 surfaces (HTTP API, CLI, Web). Each
entry links to the handler that emits the message and the recovery
path.

## HTTP error envelope

Every M1 4xx response carries:

```json
{ "code": "<STABLE_CODE>", "message": "<human explanation>" }
```

Codes are STABLE — shell scripts and CI jobs can match on them.

| Code | HTTP | Emitted by | Meaning | Recovery |
|---|---|---|---|---|
| `VALIDATION_FAILED` | 400 | [`handlers/bootstrap.rs`](../../../../../../modules/crates/server/src/handlers/bootstrap.rs) | `display_name` / `channel_handle` / `bootstrap_credential` empty after trim | Resubmit with all three populated |
| `BOOTSTRAP_INVALID` | 403 | `handlers/bootstrap.rs` | Supplied credential doesn't verify any stored hash | Re-check the plaintext; if truly lost, reinstall (see below) |
| `BOOTSTRAP_ALREADY_CONSUMED` | 403 | `handlers/bootstrap.rs` | Credential hash matched but `consumed_at` is set | See "Credential consumed but no admin exists" below |
| `PLATFORM_ADMIN_CLAIMED` | 409 | `handlers/bootstrap.rs` | A platform admin already exists | Use the existing admin's credentials; additional admins are an M2+ feature |
| `INTERNAL_ERROR` | 500 | `handlers/bootstrap.rs` | Repository / hashing / session-signing error | Check server logs for the structured `error=` field |

## CLI exit codes

The `baby-phi` binary maps the envelope to stable exit codes so
shell scripts can distinguish failure classes
([`cli/src/commands/bootstrap.rs`](../../../../../../modules/crates/cli/src/commands/bootstrap.rs)):

| Code | Meaning | Typical causes |
|---|---|---|
| `0` | success | — |
| `1` | transport error | server unreachable, DNS resolve failure, TLS handshake mismatch |
| `2` | server rejected with a known 4xx code | `BOOTSTRAP_INVALID`, `BOOTSTRAP_ALREADY_CONSUMED`, `PLATFORM_ADMIN_CLAIMED`, `VALIDATION_FAILED` |
| `3` | internal / unexpected | 5xx from server, malformed response body |

## Server won't start

### `StoreError::Migration(...)` at startup

A migration failed to apply. Recovery is in
[operations/schema-migrations-operations.md §Recovering from a
broken migration](../operations/schema-migrations-operations.md#recovering-from-a-broken-migration).

### `BABY_PHI_SESSION_SECRET must be at least 32 bytes`

The session secret loaded from config (`session.secret` or the
`BABY_PHI_SESSION__SECRET` env var) is shorter than 32 bytes.
[`SessionKey::from_config`](../../../../../../modules/crates/server/src/session.rs)
rejects it at startup. Regenerate a ≥ 32-byte secret:

```bash
head -c 32 /dev/urandom | base64 -w0
```

### Port already in use

```
Error: Address already in use (os error 98)
```

Another process is bound to the configured `server.port` (default
8080). Stop it or change `server.port` in `config/dev.toml` (or
`BABY_PHI__SERVER__PORT=9090 baby-phi-server`).

### `No such file or directory` for the data dir

Configured `storage.data_dir` doesn't exist and its parent isn't
writable by the server's UID. Create the parent directory and
chown it; the server creates the RocksDB subtree itself.

## Web server won't render `/bootstrap`

### "Cannot reach the server"

The SSR `/api/v0/bootstrap/status` probe failed. Check:

1. `baby-phi-server` is up (`curl http://127.0.0.1:8080/healthz/live`).
2. `BABY_PHI_API_URL` (web-side) matches `server.host:server.port`
   (Rust-side).
3. No firewall between the Next.js process and the Rust server.
4. TLS: if `config/dev.toml` enabled TLS on the Rust server, the
   Next process must point at `https://…`, not `http://…`.

### Session cookie doesn't carry across requests

After a successful claim, subsequent page loads should render the
claimed state. If they don't:

1. Check the browser's DevTools → Storage → Cookies for
   `baby_phi_session`. Absent? The claim response's `Set-Cookie`
   wasn't forwarded. Verify `BABY_PHI_SESSION_COOKIE_NAME`
   matches on both sides.
2. Check `BABY_PHI_SESSION_SECRET` is identical on the Rust
   server and the Next process. A mismatch makes every verify
   fail; the web treats it as unauthenticated.
3. `Secure` flag: in production the cookie has `Secure=true` and
   won't travel over plaintext HTTP. For localhost dev, the
   default `config/dev.toml` flips `session.secure = false`.

## Credential consumed but no admin exists

Symptom: `/api/v0/bootstrap/status` reports `{ claimed: false }`
but `/api/v0/bootstrap/claim` with the correct plaintext returns
403 `BOOTSTRAP_ALREADY_CONSUMED`.

Cause: a previous claim's atomic transaction rolled back at the
application layer (e.g. the server crashed mid-write) but the
credential consumption was committed. Shouldn't happen in M1 — the
`apply_bootstrap_claim` transaction is all-or-nothing — but
defensive code paths allow it.

Recovery: **reinstall** is the supported M1 path. Wipe
`data_dir`, `baby-phi-server bootstrap-init`, retry. M7b adds
`baby-phi admin regenerate-bootstrap-credential` for in-band
recovery.

## Lost the bootstrap plaintext before claiming

The plaintext is stdout-only, hashed with argon2id at-rest
([ADR-0011](../decisions/0011-bootstrap-credential-single-use.md)).
Brute-forcing it from the hash is infeasible. Recovery: reinstall
(wipe `data_dir`, rerun `bootstrap-init`, save the new plaintext
immediately).

## Permission Check denies something that "should" allow

M1 ships the Permission Check engine but doesn't wire it to any
user-facing flow yet (the bootstrap claim bypasses it; admin pages
start calling it in M2). So a "Permission Check denies my request"
symptom in M1 means you're either:

1. Running your own embedding of the engine — check the
   `Decision::Denied { failed_step, reason }` shape to see which
   step kicked it out. Map per
   [architecture/permission-check-engine.md](../architecture/permission-check-engine.md).
2. Hitting an acceptance / proptest failure — the worked trace in
   [`permission_check_worked_trace.rs`](../../../../../../modules/crates/domain/tests/permission_check_worked_trace.rs)
   gives a known-good reference.

## Cross-references

- [first-bootstrap.md](first-bootstrap.md) — the admin-facing
  install-to-claim walkthrough (failure cases table is
  complementary to this page).
- [http-api-reference.md](http-api-reference.md) — every response
  shape, including error envelopes.
- [cli-usage.md](cli-usage.md) — exit-code ladder for the CLI.
- [web-usage.md](web-usage.md) — the `/bootstrap` page's error
  rendering + recovery suggestions.
- [operations/](../operations/) — deeper operator runbooks for
  schema migrations, at-rest encryption, credential lifecycle,
  and audit-log retention.
