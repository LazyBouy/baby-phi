//! 12-factor configuration: layered TOML + env-var overrides.
//!
//! Precedence (lowest → highest):
//! 1. `config/default.toml`        — committed, non-secret defaults
//! 2. `config/{profile}.toml`      — per-environment tweaks (dev/staging/prod)
//! 3. Environment variables prefixed `PHI_` (double-underscore splits
//!    nested keys, e.g. `PHI_SERVER__PORT=8080`).
//!
//! Secrets **never** live in files. They must be env-injected in production.

use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub server: HttpConfig,
    pub storage: StorageConfig,
    pub telemetry: TelemetryConfig,
    pub session: SessionConfig,
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
    /// `tracing` env-filter directive (e.g. "info,phi=debug").
    pub log_filter: String,
    /// Emit structured JSON logs (production) vs pretty logs (dev).
    pub json_logs: bool,
}

/// Session-cookie configuration.
///
/// M1 ships a signed session cookie that is set on a successful bootstrap
/// claim so the browser knows *who* just finished bootstrapping. OAuth
/// wiring (which replaces this signed-cookie scheme for the general case)
/// lands in M3 — see [ADR-0015] (placeholder) and the admin journey plan.
///
/// The secret must be at least 32 bytes. In production it MUST come from
/// `PHI_SESSION__SECRET`; committed `config/*.toml` holds only a dev
/// default (see `config/dev.toml`).
#[derive(Debug, Clone, Deserialize)]
pub struct SessionConfig {
    /// HS256 signing key for the `phi_kernel_session` cookie.
    pub secret: String,
    /// Cookie name. Defaults to `phi_kernel_session`.
    #[serde(default = "default_cookie_name")]
    pub cookie_name: String,
    /// Token lifetime in seconds. Defaults to 12 hours.
    #[serde(default = "default_ttl_seconds")]
    pub ttl_seconds: u64,
    /// Set the `Secure` cookie attribute. Defaults to `true`; set to
    /// `false` in `config/dev.toml` so the cookie survives plaintext
    /// localhost HTTP.
    #[serde(default = "default_secure")]
    pub secure: bool,
}

fn default_cookie_name() -> String {
    "phi_kernel_session".to_string()
}

fn default_ttl_seconds() -> u64 {
    12 * 60 * 60
}

fn default_secure() -> bool {
    true
}

impl ServerConfig {
    /// Load config from `config/default.toml` + `config/{profile}.toml` +
    /// `PHI_*` env vars. Profile defaults to "dev" if unset.
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
