//! phi HTTP API.
//!
//! M0 shipped health + metrics. M1/P6 adds the `/api/v0/bootstrap/*`
//! endpoints + the signed-session-cookie layer. M2–M5 layer on the org,
//! agent, project, grant, session, and auth-request surfaces.

pub mod bootstrap;
pub mod config;
pub mod handler_support;
pub mod handlers;
pub mod health;
pub mod platform;
pub mod router;
pub mod session;
pub mod shutdown;
pub mod state;
pub mod telemetry;

pub use config::ServerConfig;
pub use handler_support::{
    check_permission, denial_to_api_error, emit_audit, ApiError, AuthenticatedSession,
};
pub use router::{build_router, with_prometheus};
pub use session::{SessionBuildError, SessionClaims, SessionError, SessionKey};
pub use state::AppState;
