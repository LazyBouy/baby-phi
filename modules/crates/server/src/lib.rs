//! baby-phi HTTP API.
//!
//! M0 ships health + metrics. Bootstrap/claim, org, agent, project, grant,
//! session, and auth-request endpoints land in M1–M5.

pub mod config;
pub mod health;
pub mod router;
pub mod state;
pub mod telemetry;

pub use config::ServerConfig;
pub use router::{build_router, with_prometheus};
pub use state::AppState;
