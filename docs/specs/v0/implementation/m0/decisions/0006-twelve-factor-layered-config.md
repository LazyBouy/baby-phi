<!-- Last verified: 2026-04-19 by Claude Code -->

# ADR-0006: 12-factor layered config — TOML layers + `PHI_*` env overrides

## Status
Accepted — 2026-04-19 (M0).

## Context

phi ships as a self-contained server that must run identically from a developer laptop through staging to production. Config needs are:

- **Non-secret defaults** that can be read from the code review (ports, data paths, log levels).
- **Environment-specific tweaks** (different data dirs, log formats, bind addresses).
- **Secrets** (database encryption keys, OAuth client secrets) that must NEVER live in a file.
- **Ad-hoc overrides** for local debugging without editing committed files.

The industry-standard answer is the [12-factor app](https://12factor.net/config) pattern: config from environment variables, loaded at startup, with a small amount of baseline config in committed files.

## Decision

**Three-layer composition with increasing precedence:**

1. `config/default.toml` — committed, baseline values, never secrets.
2. `config/{profile}.toml` — committed, profile-specific overrides. Profile selected by `PHI_PROFILE` (default `dev`; other values `staging`, `prod`).
3. Environment variables — `PHI_<SECTION>__<FIELD>` overrides any file value. Secrets **only** here.

Implemented in [`modules/crates/server/src/config.rs`](../../../../../../modules/crates/server/src/config.rs) using the `config` crate.

## Consequences

### Positive

- **Defaults are reviewable.** Anyone reading the repo sees exactly what baseline config ships.
- **Profile-specific overlays are small.** `dev.toml` only contains what differs from `default.toml`, so the diff is the intent.
- **Secrets never enter git.** The baseline files contain no secrets, and there's no "`config/secrets.toml`" path — secrets travel exclusively through the deployment's secret-injection mechanism (Kubernetes Secrets, AWS Secrets Manager, Docker secrets, `.env` sourced at boot).
- **Env-only overrides for debugging.** Any field can be flipped with a one-off env var (`PHI_TELEMETRY__LOG_FILTER=trace cargo run -p server`) without editing a committed file.
- **12-factor compliant.** Matches expectations of orchestrators, CI systems, and operators coming from other modern services.

### Negative

- **Env-var names are verbose.** `PHI_SERVER__TLS__CERT_PATH` is long. Mitigated by: the verbosity reflects real nesting; you only type them when setting overrides.
- **Silent typos.** A misspelled `PHI_SERVR__PORT` is ignored, not flagged. This is the `config` crate's default behaviour. M7b adds a strict-mode validator that warns on unknown `PHI_*` env vars — see [`../architecture/configuration.md`](../architecture/configuration.md) §"Failure modes".
- **Double-underscore separator is non-obvious.** `__` separates nested keys (because single `_` is valid inside field names like `cert_path`). Fine when you know it; surprising if you don't. Documented here and in `.env.example`.

## Alternatives considered

- **Single TOML file per environment.** Rejected — the profile overlay keeps the diff small and makes defaults reviewable. Monolithic files drift.
- **YAML instead of TOML.** Rejected — TOML is Rust's canonical config format (Cargo uses it), no whitespace-sensitivity surprises, and the `config` crate handles both anyway.
- **JSON.** Rejected — no comments, awkward for humans to edit.
- **Env-only, no files.** Rejected — defaults and per-profile overlays are easier to reason about as files; env becomes a spaghetti of `export` lines.
- **`dotenvy` / `.env` auto-loading at startup.** Rejected for the server — injecting env vars is the deployment platform's job. (The *CLI* does load `.env` for the OpenRouter demo flow, as a convenience for local dev.)
- **Ship a `config-secrets.toml` alongside committed configs** in a gitignored path. Rejected because the anti-pattern risk is too high — someone will accidentally `git add` it. Env-only for secrets is a bright line.

## How this appears

In [`config.rs`](../../../../../../modules/crates/server/src/config.rs):
```rust
impl ServerConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        let profile = std::env::var("PHI_PROFILE").unwrap_or_else(|_| "dev".to_string());
        config::Config::builder()
            .add_source(config::File::with_name("config/default").required(true))
            .add_source(config::File::with_name(&format!("config/{profile}")).required(false))
            .add_source(
                config::Environment::with_prefix("PHI")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?
            .try_deserialize()
    }
}
```

Layered files at [`config/`](../../../../../../config/).

Env examples:
- `PHI_PROFILE=prod`
- `PHI_SERVER__PORT=9090`
- `PHI_STORAGE__DATA_DIR=/mnt/ssd/phi`
- `PHI_TELEMETRY__LOG_FILTER="info,server=debug"`

Secret placeholders (M1+) in [`.env.example`](../../../../../../.env.example) — the production secret-injection mechanism writes these into the process environment before launch.
