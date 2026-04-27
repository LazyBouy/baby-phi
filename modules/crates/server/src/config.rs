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
    /// Graceful-shutdown configuration (CH-K8S-PREP P-3). Defaults
    /// preserve M5/CH-02 behaviour for legacy configs that omit the
    /// `[shutdown]` block.
    #[serde(default)]
    pub shutdown: ShutdownConfig,
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
    /// Path to the SurrealDB RocksDB data directory (used when
    /// [`StorageConfig::mode`] is `Embedded` — the default).
    pub data_dir: PathBuf,
    pub namespace: String,
    pub database: String,
    /// Backend selection (CH-K8S-PREP P-2 / ADR-0033). `Embedded` (the
    /// default) opens RocksDB at `data_dir`. `Remote` uses
    /// [`StorageRemoteConfig::uri`] to connect to a standalone SurrealDB
    /// server — the typical M7b path for K8s deployments where storage
    /// is externalised. Override via `PHI_STORAGE__MODE=remote` +
    /// `PHI_STORAGE__REMOTE__URI=ws://...`.
    #[serde(default = "default_storage_mode")]
    pub mode: StorageMode,
    /// Remote-mode configuration. Only consulted when [`mode`] is
    /// `Remote`; defaults work for the embedded path so legacy configs
    /// keep round-tripping.
    #[serde(default)]
    pub remote: StorageRemoteConfig,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum StorageMode {
    #[default]
    Embedded,
    Remote,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct StorageRemoteConfig {
    /// SurrealDB connection URI (e.g. `ws://surreal.svc:8000`,
    /// `wss://surreal.example.com`, `memory://` for in-memory tests).
    /// Empty by default; required when `mode = "remote"` (boot fails
    /// fast otherwise).
    #[serde(default)]
    pub uri: String,
}

fn default_storage_mode() -> StorageMode {
    StorageMode::Embedded
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
    /// Platform-wide concurrency ceiling on active agent-loop sessions
    /// (M5 / ADR-0031). Launches beyond this cap return HTTP 503
    /// `SESSION_WORKER_SATURATED`. Default `16` covers a single-machine
    /// dev box; operators with larger machines tune up via
    /// `config/<profile>.toml` override or `PHI_SESSION__MAX_CONCURRENT`.
    /// Distinct from the per-agent `parallelize` cap (which surfaces as
    /// 409 `PARALLELIZE_CAP_REACHED` from the agent's blueprint).
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent: u32,
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

fn default_max_concurrent() -> u32 {
    16
}

/// Graceful-shutdown configuration (CH-K8S-PREP P-3 / ADR-0031 §D31.5
/// / ADR-0033). Tunable via `PHI_SHUTDOWN__TIMEOUT_SECS`.
#[derive(Debug, Clone, Deserialize)]
pub struct ShutdownConfig {
    /// Maximum seconds the SIGTERM handler waits for live agent-loop
    /// tasks to drain before reporting `DrainTimeout`. Defaults to 30
    /// per ADR-0031 §D31.2's grace-period baseline (matches K8s'
    /// default `terminationGracePeriodSeconds`).
    #[serde(default = "default_shutdown_timeout_secs")]
    pub timeout_secs: u64,
}

impl Default for ShutdownConfig {
    fn default() -> Self {
        Self {
            timeout_secs: default_shutdown_timeout_secs(),
        }
    }
}

fn default_shutdown_timeout_secs() -> u64 {
    crate::shutdown::DEFAULT_SHUTDOWN_TIMEOUT_SECS
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
