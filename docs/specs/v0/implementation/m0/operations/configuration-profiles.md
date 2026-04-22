<!-- Last verified: 2026-04-19 by Claude Code -->

# Operations — configuration profiles

phi-server selects a config profile at startup via `PHI_PROFILE`. The profile's TOML file overlays onto `config/default.toml`, and then environment variables override both. This page describes the three shipped profiles and how to add a new one.

See also: [`../architecture/configuration.md`](../architecture/configuration.md) for the schema and the loader behaviour, and [ADR-0006](../decisions/0006-twelve-factor-layered-config.md) for the rationale.

## Shipped profiles

### `dev` (default)

[`config/dev.toml`](../../../../../../config/dev.toml). Selected when `PHI_PROFILE` is unset or `dev`.

Differences from `default.toml`:

- `server.host = "127.0.0.1"` — loopback only, so a dev instance doesn't accidentally listen on an external interface.
- `storage.data_dir = "data/phi-dev.db"` — keeps dev data out of the production default location.
- `telemetry.log_filter = "info,server=debug,domain=debug,store=debug"` — deep logging inside our crates for easy tracing.
- `telemetry.json_logs = false` — pretty, colourised output for a terminal.

### `staging`

[`config/staging.toml`](../../../../../../config/staging.toml). Selected when `PHI_PROFILE=staging`.

- `server.host = "0.0.0.0"` — bind all interfaces (typically one, inside a container network).
- `storage.data_dir = "/var/lib/phi/data"` — persistent volume path.
- `telemetry.log_filter = "info"` — less noise.
- `telemetry.json_logs = true` — ingest-ready for Loki / Elasticsearch / Cloud Logging.

### `prod`

[`config/prod.toml`](../../../../../../config/prod.toml). Selected when `PHI_PROFILE=prod`. Set automatically in the Dockerfile.

Same shape as `staging`, with an extended comment documenting the TLS policy:

> The recommended production pattern is to terminate TLS at a reverse proxy in front of phi-server, and run phi-server itself over plaintext HTTP on an internal network. Native TLS via `[server.tls]` is supported for simple single-node deploys.

See [tls-and-transport-security.md](tls-and-transport-security.md) for the TLS story.

## Layering in practice

Imagine we're running in production with a bespoke port override:

```bash
PHI_PROFILE=prod PHI_SERVER__PORT=9090 phi-server
```

Layering order:

| Layer | `server.port` | `server.host` | `storage.data_dir` | `telemetry.json_logs` |
|---|---|---|---|---|
| `default.toml` | `8080` | `"0.0.0.0"` | `"data/phi.db"` | `true` |
| `prod.toml` | *(inherits)* | *(inherits)* | `"/var/lib/phi/data"` | `true` |
| env | **`9090`** | — | — | — |
| **Effective** | **9090** | `"0.0.0.0"` | `"/var/lib/phi/data"` | `true` |

## Adding a new profile

If you need, say, a `canary` profile for pre-production traffic:

1. Create `config/canary.toml` with only the fields that differ from `default.toml`. Example:
   ```toml
   [server]
   host = "0.0.0.0"
   port = 8080

   [storage]
   data_dir = "/var/lib/phi/data"

   [telemetry]
   log_filter = "debug"   # chattier than prod so issues surface fast
   json_logs = true
   ```
2. Set `PHI_PROFILE=canary` in your deployment.
3. Done — no code changes. The loader at [`config.rs:58-71`](../../../../../../modules/crates/server/src/config.rs) loads `config/{profile}.toml` with `required(false)`, so any profile name you use Just Works as long as the file exists.

If you mistype the profile name, the overlay file is silently absent and only `default.toml` applies. This is either a feature (permissive) or a footgun (silent). M7b adds strict-mode validation; until then, operators should `grep -l` their profile file before rolling out.

## Secrets per profile

**Secrets never live in `config/*.toml`.** The profile files are committed to git and reviewed publicly; anything sensitive must flow through env vars (and typically through a secret manager — Kubernetes Secrets, AWS Secrets Manager, Docker secrets).

The placeholders in [`.env.example`](../../../../../../.env.example) show what per-profile secrets will be needed in later milestones:

- `[M3]` `PHI_AUTH__OIDC__CLIENT_SECRET` — OAuth client secret.
- `[M1/M7b]` `PHI_STORAGE__ENCRYPTION_KEY` — 32-byte hex AES key for SurrealDB at-rest encryption.
- `[M7]` `PHI_AUDIT__STREAM__ACCESS_KEY` / `…SECRET_KEY` — write-only creds for the audit log off-site bucket.

Each secret is loaded as an `Option<…>` in the relevant config struct; production deploys must set it explicitly, and failure to do so surfaces at boot.

## What `.env` files are and aren't

The phi server **does not** auto-load a `.env` file. Environment variables must be set in the process environment before `phi-server` starts (e.g. by systemd, Docker `--env-file`, Kubernetes env, or a shell wrapper).

The phi **CLI** does load `.env` (via the OpenRouter demo flow inherited from the phi-core scaffold). This is a developer convenience for running the CLI against a local API key; it's not a production pattern.

See [`../user-guide/running-locally.md`](../user-guide/running-locally.md) for the dev-side view.

## Troubleshooting

| Symptom | Likely cause |
|---|---|
| Server listens on port 8080 despite `PHI_SERVER__PORT=9090` | Typo — single underscore between `SERVER` and `PORT`. Must be `__`. |
| "Profile X not applied" | `config/X.toml` doesn't exist; only `default.toml` loaded. |
| `Err(ConfigError::Type)` on startup | Env value can't be parsed as the field's type (e.g. non-numeric port, non-bool json_logs). |
| Secret showing in process list / logs | Secret was passed on the command line or logged. Always inject through the platform's secret mechanism and use `tracing::debug!(secret=?…)` sparingly (never on prod). |
