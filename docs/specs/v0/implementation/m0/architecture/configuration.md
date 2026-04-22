<!-- Last verified: 2026-04-19 by Claude Code -->

# Architecture — configuration

Runtime configuration for `phi-server` follows a **12-factor** pattern: non-secret defaults live in committed TOML files, environment-specific overlays layer on top, and secrets are injected via environment variables. Secrets never land in a file.

## Source order

At [`modules/crates/server/src/config.rs:58-71`](../../../../../../modules/crates/server/src/config.rs), `ServerConfig::load()` composes three sources in order (later wins):

```
┌─────────────────────────────────────────────────────────────────────┐
│ 1. config/default.toml       — committed; non-secret defaults       │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│ 2. config/{profile}.toml     — profile = $PHI_PROFILE or "dev" │
│                                committed; non-secret env tweaks     │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│ 3. Environment variables      — prefix PHI_;                   │
│                                 double-underscore "__" splits       │
│                                 nested keys                         │
└─────────────────────────────────────────────────────────────────────┘
```

Profile selection:

- `PHI_PROFILE=dev` (default) → layers `config/dev.toml`.
- `PHI_PROFILE=staging` → layers `config/staging.toml`.
- `PHI_PROFILE=prod` → layers `config/prod.toml`.
- Unknown profile → the overlay is `required(false)`, so a missing file is silently skipped; only `config/default.toml` is strictly required.

## Env-var naming

Nested keys split with **double underscore** (`__`):

| Config field | Env override |
|---|---|
| `server.host` | `PHI_SERVER__HOST` |
| `server.port` | `PHI_SERVER__PORT` |
| `server.tls.cert_path` | `PHI_SERVER__TLS__CERT_PATH` |
| `server.tls.key_path` | `PHI_SERVER__TLS__KEY_PATH` |
| `storage.data_dir` | `PHI_STORAGE__DATA_DIR` |
| `storage.namespace` | `PHI_STORAGE__NAMESPACE` |
| `storage.database` | `PHI_STORAGE__DATABASE` |
| `telemetry.log_filter` | `PHI_TELEMETRY__LOG_FILTER` |
| `telemetry.json_logs` | `PHI_TELEMETRY__JSON_LOGS` |

The `config` crate at version 0.14 parses scalars (`bool`, integers, strings, `PathBuf`) out of env strings automatically because we pass `.try_parsing(true)` at [`config.rs:67`](../../../../../../modules/crates/server/src/config.rs).

## Schema reference

Defined at [`modules/crates/server/src/config.rs`](../../../../../../modules/crates/server/src/config.rs):

### `[server]`

| Field | Type | Required | Default | Meaning |
|---|---|---|---|---|
| `host` | string | yes | `"0.0.0.0"` (default.toml) | Bind address. |
| `port` | u16 | yes | `8080` | Bind port. |
| `tls` | `Option<TlsConfig>` | no | `None` | When set, serve HTTPS natively via `axum-server`. |

### `[server.tls]` (optional)

| Field | Type | Required | Meaning |
|---|---|---|---|
| `cert_path` | path | yes | PEM-encoded server certificate. |
| `key_path` | path | yes | PEM-encoded server private key. |

### `[storage]`

| Field | Type | Required | Default | Meaning |
|---|---|---|---|---|
| `data_dir` | path | yes | `"data/phi.db"` | SurrealDB RocksDB data directory. |
| `namespace` | string | yes | `"phi"` | SurrealDB namespace. |
| `database` | string | yes | `"v0"` | SurrealDB database name inside the namespace. |

### `[telemetry]`

| Field | Type | Required | Default | Meaning |
|---|---|---|---|---|
| `log_filter` | string | yes | `"info"` | `tracing-subscriber::EnvFilter` directive (e.g. `"info,server=debug"`). |
| `json_logs` | bool | yes | `true` (prod), `false` (dev) | JSON structured logs vs pretty. |

## Committed layers

At [`config/`](../../../../../../config/):

- [`default.toml`](../../../../../../config/default.toml) — baseline. All fields present.
- [`dev.toml`](../../../../../../config/dev.toml) — binds to `127.0.0.1`, uses `data/phi-dev.db`, pretty logs, debug filter.
- [`staging.toml`](../../../../../../config/staging.toml) — binds `0.0.0.0`, `/var/lib/phi/data`, JSON logs.
- [`prod.toml`](../../../../../../config/prod.toml) — same as staging; carries an extended comment explaining the reverse-proxy-first TLS pattern.

None of them contain secrets.

## Secrets policy

Secrets are **always env-injected**, never file-committed. Current secret surface in M0 is empty (the HTTP server has no user auth yet). Placeholders tagged in [`.env.example`](../../../../../../.env.example) show the expected shape of future secrets:

- `[M3]` OAuth 2.0 client credentials (`PHI_AUTH__OIDC__*`).
- `[M1/M7b]` SurrealDB at-rest encryption master key (`PHI_STORAGE__ENCRYPTION_KEY`).
- `[M7]` Audit-log off-site stream bucket/region (`PHI_AUDIT__STREAM__*`).

Every one of these will arrive as an `Option<…>` in the appropriate config struct so omission is explicit, and every one will be loaded **only** from env (no layered-TOML fallback), enforced at review time when those fields are added.

See [ADR-0006](../decisions/0006-twelve-factor-layered-config.md) for the full rationale.

## Failure modes

| Problem | Behaviour |
|---|---|
| `config/default.toml` missing | `ServerConfig::load()` returns `Err(config::ConfigError::Frozen)`; `main.rs` propagates and the process exits with non-zero. |
| Unknown profile name | Overlay silently absent; only `default.toml` applied. No error. |
| Typo in env-var name (e.g. `PHI_SERVER_PORT` missing one underscore) | Env var silently ignored; field keeps layered value. This is a sharp edge — M7b adds a strict-mode validation pass in `ServerConfig::load()` that warns on unknown `PHI_*` env vars. |
| Unparseable value (e.g. non-numeric `PHI_SERVER__PORT`) | `try_deserialize()` returns `Err(ConfigError::Type)`; process exits. |
