use std::sync::Arc;

use domain::Repository;

use crate::session::SessionKey;

/// Shared application state injected into every axum handler via
/// `State<AppState>`.
///
/// - `repo` is held behind a trait object so acceptance tests can swap in
///   in-memory fakes without touching handler code.
/// - `session` carries the HS256 signing key + cookie-shape settings for
///   [`crate::session::sign_and_build_cookie`] / [`crate::session::verify_from_cookies`].
#[derive(Clone)]
pub struct AppState {
    pub repo: Arc<dyn Repository>,
    pub session: SessionKey,
}
