use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use crate::config::TelemetryConfig;

/// Initialise the global `tracing` subscriber. Safe to call once; subsequent
/// calls are ignored.
pub fn init(cfg: &TelemetryConfig) {
    let filter = EnvFilter::try_new(&cfg.log_filter).unwrap_or_else(|_| EnvFilter::new("info"));

    let registry = tracing_subscriber::registry().with(filter);

    if cfg.json_logs {
        let _ = registry.with(fmt::layer().json()).try_init();
    } else {
        let _ = registry.with(fmt::layer().pretty()).try_init();
    }
}
