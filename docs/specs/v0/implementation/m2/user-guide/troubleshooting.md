<!-- Last verified: 2026-04-21 by Claude Code -->

# Troubleshooting — M2 error codes & recovery

**Status: [EXISTS]** — M2/P8 seal doc; catalogues every M2 surface's
stable error codes + the recovery path.

This page is the grepable operator reference for M2's four admin
pages plus the `handler_support` shared surface. For M1 codes see
[`../../m1/user-guide/troubleshooting.md`](../../m1/user-guide/troubleshooting.md).

## HTTP error envelope

Every M2 4xx / 5xx response carries the same shape as M1:

```json
{ "code": "<STABLE_CODE>", "message": "<human explanation>" }
```

Codes are STABLE — shell scripts and CI jobs can match on them.

## Shared codes (every M2 handler can emit)

Defined in [`handler_support::errors::ApiError`](../../../../../../modules/crates/server/src/handler_support/errors.rs).
These ride in through any M2 write or read.

| Code | HTTP | Meaning | Recovery |
|---|---|---|---|
| `UNAUTHENTICATED` | 401 | Session cookie missing or expired | Re-run `baby-phi bootstrap claim` (or the future M3 `baby-phi login`) to mint a fresh cookie |
| `VALIDATION_FAILED` | 400 | Request body failed shape / bounds check | Fix the input per the accompanying `message` |
| `AUDIT_EMIT_FAILED` | 500 | Audit emitter returned an error; the underlying write may have succeeded | Check server logs (`audit_events` table or hash-chain break); re-verify state via a GET before retrying |
| `INTERNAL_ERROR` | 500 | Repository / crypto / session-signing error | Check the structured `error=` field in server logs |

## Permission-Check denials (shared shim)

Emitted by [`handler_support::check_permission`](../../../../../../modules/crates/server/src/handler_support/permission.rs) when the engine returns `Decision::Denied`. M2 admin handlers take the Template-E self-approved bypass, so these codes surface primarily in M3+; listing them here keeps the registry complete.

| Code | HTTP | `FailedStep` | Meaning |
|---|---|---|---|
| `AWAITING_CONSENT` | 202 | Consent | Engine is waiting on a downstream Auth Request; the UI should show a "work continues" toast |
| `CATALOGUE_MISS` | 403 | Catalogue | Target URI not seeded in `resources_catalogue` |
| `MANIFEST_EMPTY` | 400 | Expansion | Manifest has no reaches after fundamental-expansion |
| `NO_GRANTS_HELD` | 403 | Resolution | No grants matched the principal / resource selector |
| `CEILING_EMPTIED` | 403 | Ceiling | Scope-ceiling intersection eliminated every candidate |
| `NO_MATCHING_GRANT` | 403 | Match | No single grant covered the full reach set |
| `CONSTRAINT_VIOLATION` | 403 | Constraint | Constraint-context missing a required key or value mismatch (e.g., `purpose=reveal` required but absent) |
| `SCOPE_UNRESOLVABLE` | 403 | Scope | Scope template referenced a non-resolvable variable |

## Page 04 — Credentials Vault

Emitted by [`handlers/platform_secrets.rs`](../../../../../../modules/crates/server/src/handlers/platform_secrets.rs) / [`platform/secrets/`](../../../../../../modules/crates/server/src/platform/secrets/).

| Code | HTTP | Meaning | Recovery |
|---|---|---|---|
| `SECRET_SLUG_IN_USE` | 409 | `slug` already exists in the vault | Pick a different slug or rotate the existing entry with `baby-phi secret rotate` |
| `SECRET_NOT_FOUND` | 404 | No vault entry for that slug | Confirm via `baby-phi secret list`; slugs are case-sensitive |
| `AWAITING_CONSENT` | 202 | Reveal waiting on Permission-Check consent | Complete the consent flow surfaced in the UI; retry after approval |
| `VAULT_CRYPTO_FAILED` | 500 | Seal/unseal failure (master-key drift, tampered ciphertext) | Check server logs for `crypto` errors; verify `BABY_PHI_SESSION_SECRET` and master-key provisioning |

## Page 02 — Model Providers

Emitted by [`handlers/platform_model_providers.rs`](../../../../../../modules/crates/server/src/handlers/platform_model_providers.rs) / [`platform/model_providers/`](../../../../../../modules/crates/server/src/platform/model_providers/).

| Code | HTTP | Meaning | Recovery |
|---|---|---|---|
| `MODEL_PROVIDER_DUPLICATE` | 409 | `(provider, config.id)` pair already registered | Archive the existing row and register fresh, or pick a different `config.id` |
| `SECRET_REF_NOT_FOUND` | 400 | `secret_ref` slug does not exist in the vault | Add the secret first via `baby-phi secret add --slug <slug> --material-file <path>` |
| `MODEL_PROVIDER_NOT_FOUND` | 404 | Archive target id doesn't exist | List via `baby-phi model-provider list --include-archived` to confirm the id |

## Page 03 — MCP Servers

Emitted by [`handlers/platform_mcp_servers.rs`](../../../../../../modules/crates/server/src/handlers/platform_mcp_servers.rs) / [`platform/mcp_servers/`](../../../../../../modules/crates/server/src/platform/mcp_servers/).

| Code | HTTP | Meaning | Recovery |
|---|---|---|---|
| `SECRET_REF_NOT_FOUND` | 400 | `secret_ref` slug does not exist in the vault | Add the secret first; OR omit `secret_ref` for unauthenticated services |
| `MCP_SERVER_NOT_FOUND` | 404 | PATCH / archive target id doesn't exist | List via `baby-phi mcp-server list --include-archived` |

**Cascade-on-narrow side-effects:** PATCHing `tenants_allowed` to a strict subset emits one `platform.mcp_server.tenant_access_revoked` summary + N per-AR `auth_request.revoked` events. Forward-only — see [`../operations/mcp-server-operations.md`](../operations/mcp-server-operations.md) §5 for the emergency over-narrow playbook.

## Page 05 — Platform Defaults

Emitted by [`handlers/platform_defaults.rs`](../../../../../../modules/crates/server/src/handlers/platform_defaults.rs) / [`platform/defaults/`](../../../../../../modules/crates/server/src/platform/defaults/).

| Code | HTTP | Meaning | Recovery |
|---|---|---|---|
| `PLATFORM_DEFAULTS_STALE_WRITE` | 409 | `if_version` doesn't match current row | Re-read via `baby-phi platform-defaults get --format json`, merge onto the new version, retry with the current `--if-version` |

Validation bounds on PUT (all → `VALIDATION_FAILED`):

- `execution_limits.max_turns == 0`
- `execution_limits.max_total_tokens == 0`
- `retry_config.max_retries > 100`

## CLI exit codes

Stable contract mapped by [`cli/src/exit.rs`](../../../../../../modules/crates/cli/src/exit.rs):

| Code | Meaning | Typical causes |
|---|---|---|
| `0` | success | — |
| `1` | transport error | server unreachable, DNS resolve failure, TLS handshake mismatch |
| `2` | server rejected with a known 4xx code | `VALIDATION_FAILED`, `MCP_SERVER_NOT_FOUND`, `PLATFORM_DEFAULTS_STALE_WRITE`, etc. |
| `3` | internal / unexpected | 5xx, malformed response body, local I/O failure on `factory` output |
| `4` | precondition failed | no saved session at `$XDG_CONFIG_HOME/baby-phi/session` |
| `5` | cascade aborted | `mcp-server patch-tenants` without `--confirm-cascade` |

## Common recovery paths

### "No session; every subcommand exits 4"

The operator hasn't run `bootstrap claim` yet, or the session file was deleted.

Recovery: re-run `baby-phi bootstrap claim --credential <bphi-bootstrap-…> --display-name <NAME> --channel-kind web --channel-handle <URL>`. The plaintext credential is single-use; if it was already consumed, see the M1 runbook's "Credential consumed but no admin exists".

### "PATCH narrows more than expected"

Triggered by `PLATFORM_…` cascade-revocation that revoked grants the admin didn't intend to drop.

Recovery: see [`../operations/mcp-server-operations.md`](../operations/mcp-server-operations.md) §5. Widening back does NOT restore revoked grants — forward-only cascade.

### "Stale-write 409 on platform-defaults PUT"

Two admins edited concurrently; your `if_version` is behind.

Recovery:
1. `baby-phi platform-defaults get --format json` — the response includes the current `version`.
2. Merge your intended changes on top of the fetched row.
3. `baby-phi platform-defaults put --file <merged.json> --if-version <new-version>`.

### "I lost the bootstrap credential plaintext"

Documented in the M1 runbook — the credential is single-use and non-recoverable. Reinstall the server (fresh SurrealDB) is the only supported path in M2. Key-recovery tooling is M7b.

## See also

- [M1 troubleshooting](../../m1/user-guide/troubleshooting.md) — bootstrap + session + server-won't-start scenarios.
- [`../operations/`](../operations/) — per-page ops runbooks.
- [`../../../../../ops/runbook.md`](../../../../../ops/runbook.md) — cross-cutting runbook (full version is M7b).
