<!-- Last verified: 2026-04-20 by Claude Code -->

# CLI usage — `phi bootstrap {status,claim}` + `agent demo`

M1/P7 migrates the `phi` binary to a `clap`-based subcommand tree.
Two top-level groups ship in this phase:

1. `phi bootstrap {status,claim}` — the HTTP client for the P6
   endpoints (operator-facing).
2. `phi agent demo` — the pre-M1 phi-core agent-loop demo,
   preserved behind an explicit subcommand so the prototype doesn't
   regress while M2+ adds real agent-management subcommands.

## Resolving the server URL

For any subcommand that hits the HTTP API, the CLI resolves the base
URL via this precedence (highest wins):

1. `--server-url <URL>` flag.
2. `PHI_API_URL` environment variable.
3. `{scheme}://{server.host}:{server.port}` from
   [`ServerConfig::load()`](../../../../../../modules/crates/server/src/config.rs)
   — the same layered TOML stack the server reads. Scheme is `https` if
   `[server.tls]` is set, otherwise `http`. A literal `0.0.0.0` host is
   rewritten to `127.0.0.1` so the CLI never tries to dial a bind-all
   address.

Trailing slashes on the URL are stripped.

## `phi bootstrap status`

```console
$ phi bootstrap status
platform admin NOT yet claimed
  next step: run `phi bootstrap claim --credential bphi-bootstrap-… \
    --display-name '…' --channel-kind slack --channel-handle @you`
```

After a successful claim:

```console
$ phi bootstrap status
platform admin already claimed
  admin_agent_id: a7b01e03-…
```

Exit codes:

| Code | Meaning |
|---|---|
| 0 | status retrieved successfully (claimed or not) |
| 1 | server unreachable / transport error |
| 2 | server returned 4xx with a known `code` (shouldn't fire on `status`) |
| 3 | server returned 5xx or an unexpected shape |

## `phi bootstrap claim`

Consumes the single-use credential printed at install time
(`phi-server bootstrap-init`) and creates the platform admin.

```console
$ phi bootstrap claim \
    --credential bphi-bootstrap-abc123… \
    --display-name 'Alex Chen' \
    --channel-kind slack \
    --channel-handle @alex
platform admin claimed successfully
  human_agent_id:            9d71…
  inbox_id:                  bb90…
  outbox_id:                 2c1b…
  grant_id:                  ff07…
  bootstrap_auth_request_id: 7e81…
  audit_event_id:            1a0d…

Next step: continue to the M2 platform-admin journey (model-provider registration).
```

`--channel-kind` accepts `slack | email | web`.

Exit codes:

| Code | Meaning | Typical cause |
|---|---|---|
| 0 | claim succeeded | fresh install, credential correct |
| 1 | transport error | server unreachable, TLS mismatch |
| 2 | claim rejected | `BOOTSTRAP_INVALID`, `BOOTSTRAP_ALREADY_CONSUMED`, `PLATFORM_ADMIN_CLAIMED`, `VALIDATION_FAILED` |
| 3 | internal error | 5xx from server, unexpected response shape |

The specific server-side rejection (`BOOTSTRAP_INVALID` etc.) is echoed
to stderr so shell scripts can parse it with `grep`. The response
envelope is stable — see
[http-api-reference.md](http-api-reference.md).

## `phi agent demo`

Runs the phi-core agent loop against the legacy `config.toml` at the
current working directory. Preserved from pre-M1 for prototype
continuity; retiring the loop in favour of real agent-management
subcommands is an M2+ cleanup item.

```console
$ phi agent demo
Agent: email-writer
=== phi agent demo ===

Prompt: Write a marketing email …
---

<streaming tokens…>

--- Done ---
Tokens: 42 input, 312 output, 354 total
Session saved to: workspace/session/<id>.json
```

Takes an optional prompt override:

```console
$ phi agent demo "Summarize the phi-core README in 3 bullets."
```

Exits non-zero if `config.toml` is missing, can't be parsed, or if no
agent can be instantiated from it.

## Environment variables

| Var | Effect |
|---|---|
| `PHI_API_URL` | Override the bootstrap server URL (step 2 of resolution) |
| `PHI_PROFILE` | Which `config/{profile}.toml` layer `ServerConfig::load()` reads (step 3) |
| `PHI__*` | Per-key overrides for the layered config (see [ServerConfig](../../../../../../modules/crates/server/src/config.rs)) |

## Test coverage

| Layer | File | Tests |
|---|---|---|
| Unit | [`commands/bootstrap.rs`](../../../../../../modules/crates/cli/src/commands/bootstrap.rs) | 3 (URL trailing-slash normalisation) |
| Integration | [`tests/bootstrap_cli.rs`](../../../../../../modules/crates/cli/tests/bootstrap_cli.rs) | 7 (status unclaimed / claimed, claim 201 shape, 403 rejected, 409 rejected, transport error, demo without config) |

Integration tests spin up the axum router in-process against
`InMemoryRepository`, shell out to the built `phi` binary, and
assert exit code + stdout.

## Cross-references

- [http-api-reference.md](http-api-reference.md) — the HTTP contract
  the CLI wraps.
- [first-bootstrap.md](first-bootstrap.md) — where this CLI fits in
  the admin's install-to-claim walkthrough.
- [architecture/server-topology.md](../architecture/server-topology.md) —
  the server side of the URL the CLI dials.
