use std::sync::Arc;

use domain::Repository;

/// Shared application state injected into every axum handler via
/// `State<AppState>`. Repository is held behind a trait object so acceptance
/// tests can swap in in-memory fakes without touching handler code.
#[derive(Clone)]
pub struct AppState {
    pub repo: Arc<dyn Repository>,
}
