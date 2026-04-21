//! `AuthenticatedSession` axum extractor — the gate every M2+ write
//! handler goes through before running business logic.
//!
//! Drops into any handler signature as the first extractor after
//! `State<AppState>`:
//!
//! ```ignore
//! pub async fn add_secret(
//!     State(state): State<AppState>,
//!     session: AuthenticatedSession,
//!     Json(body): Json<AddSecretRequest>,
//! ) -> Result<Response, ApiError> { ... }
//! ```
//!
//! If the cookie is missing, malformed, or expired, the extractor
//! rejects with a canned `ApiError::unauthenticated()` (HTTP 401 +
//! `code: "UNAUTHENTICATED"`). Handlers never need to branch on a
//! missing-cookie case.
//!
//! The extractor reads `SessionKey` off `AppState` and `Cookies` off
//! `tower_cookies::Cookies` (installed by `CookieManagerLayer` in
//! `router::build_router`), so both must be in scope — the axum
//! `FromRequestParts` machinery arranges this.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use domain::model::ids::AgentId;
use tower_cookies::Cookies;
use uuid::Uuid;

use crate::session::{verify_from_cookies, SessionClaims};
use crate::state::AppState;

use super::errors::ApiError;

/// A verified session cookie + its decoded `sub` (the admin's
/// `AgentId`). Handlers take this by value — it's a couple of small
/// fields, not worth borrowing through axum.
#[derive(Debug, Clone)]
pub struct AuthenticatedSession {
    /// The admin's Agent id, parsed from the cookie's `sub` claim.
    pub agent_id: AgentId,
    /// The raw decoded claims — kept so handlers that care about
    /// `iat`/`exp` (e.g. diagnostics endpoints) can read them without
    /// verifying the cookie a second time.
    pub claims: SessionClaims,
}

#[axum::async_trait]
impl FromRequestParts<AppState> for AuthenticatedSession {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // `Cookies` is installed as an axum Extension by
        // `CookieManagerLayer` in `router::build_router`. If the
        // layer isn't in the stack (misconfigured router), fail loudly
        // with INTERNAL rather than silently emitting UNAUTHENTICATED.
        let cookies = Cookies::from_request_parts(parts, state)
            .await
            .map_err(|_| ApiError::internal())?;

        let claims = verify_from_cookies(&state.session, &cookies)
            .map_err(|_| ApiError::unauthenticated())?;

        let agent_id = Uuid::parse_str(&claims.sub)
            .map(AgentId::from_uuid)
            .map_err(|_| ApiError::unauthenticated())?;

        Ok(Self { agent_id, claims })
    }
}
