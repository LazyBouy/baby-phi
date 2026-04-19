//! 12-factor configuration: layered TOML + env-var overrides.
//!
//! Precedence (lowest → highest):
//! 1. `config/default.toml`        — committed, non-secret defaults
//! 2. `config/{profile}.toml`      — per-environment tweaks (dev/staging/prod)
//! 3. Environment variables prefixed `BABY_PHI_` (double-underscore splits
//!    nested keys, e.g. `BABY_PHI_SERVER__PORT=8080`).
//!
//! Secrets **never** live in files. They must be env-injected in production.

use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub server: HttpConfig,
    pub storage: StorageConfig,
    pub telemetry: TelemetryConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HttpConfig {
    pub host: String,
    pub port: u16,
    /// Enable native TLS termination. In production the recommended pattern
    /// is a reverse proxy (nginx/Caddy) in front; native TLS is for simple
    /// deploys. See M7b hardening milestone.
    #[serde(default)]
    pub tls: Option<TlsConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TlsConfig {
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    /// Path to the SurrealDB RocksDB data directory.
    pub data_dir: PathBuf,
    pub namespace: String,
    pub database: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TelemetryConfig {
    /// `tracing` env-filter directive (e.g. "info,baby_phi=debug").
    pub log_filter: String,
    /// Emit structured JSON logs (production) vs pretty logs (dev).
    pub json_logs: bool,
}

impl ServerConfig {
    /// Load config from `config/default.toml` + `config/{profile}.toml` +
    /// `BABY_PHI_*` env vars. Profile defaults to "dev" if unset.
    pub fn load() -> Result<Self, config::ConfigError> {
        let profile = std::env::var("BABY_PHI_PROFILE").unwrap_or_else(|_| "dev".to_string());

        config::Config::builder()
            .add_source(config::File::with_name("config/default").required(true))
            .add_source(config::File::with_name(&format!("config/{profile}")).required(false))
            .add_source(
                config::Environment::with_prefix("BABY_PHI")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?
            .try_deserialize()
    }
}
