//! Shared error envelope for every M2+ handler.
//!
//! Promoted from `handlers::bootstrap::ApiError` in M2/P3 so every
//! surface serialises the same `{code, message}` shape. The code is a
//! stable, machine-readable string; the message is human-readable and
//! may change across releases.
//!
//! The web tier's stable-code registry (`modules/web/lib/api/errors.ts`)
//! must stay in sync with the codes defined here — P1 seeded the
//! client-side table with Permission-Check + auth + internal codes.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

/// Stable `{code, message}` envelope emitted on every 4xx/5xx response.
///
/// `code` is kept `&'static str` so handler callsites pin the code at
/// compile time rather than allocating short-lived `String`s on the
/// error path.
#[derive(Debug, Clone, Serialize)]
pub struct ApiError {
    pub code: &'static str,
    pub message: String,
    /// HTTP status to emit. Not serialised — used only by the
    /// [`IntoResponse`] impl.
    #[serde(skip)]
    pub status: StatusCode,
}

impl ApiError {
    pub fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
        }
    }

    // ---- Canonical constructors (one per M2 stable code) ---------------

    pub fn unauthenticated() -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            "UNAUTHENTICATED",
            "session cookie missing or expired",
        )
    }

    pub fn validation_failed(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "VALIDATION_FAILED", message)
    }

    pub fn internal() -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL_ERROR",
            "an internal error occurred",
        )
    }

    pub fn audit_emit_failed(message: impl Into<String>) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "AUDIT_EMIT_FAILED",
            message,
        )
    }
}

/// Lets handlers return `Result<_, ApiError>` directly — axum's
/// `IntoResponse` handles the rest. Serialises as
/// `{ "code": "...", "message": "..." }` with the paired HTTP status.
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status;
        (status, Json(self)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unauthenticated_builds_a_401() {
        let err = ApiError::unauthenticated();
        assert_eq!(err.status, StatusCode::UNAUTHORIZED);
        assert_eq!(err.code, "UNAUTHENTICATED");
    }

    #[test]
    fn serialises_as_code_and_message_without_status() {
        let err = ApiError::validation_failed("bad slug");
        let json = serde_json::to_string(&err).unwrap();
        // `status` is `#[serde(skip)]` — wire format excludes it.
        assert!(!json.contains("status"));
        assert!(json.contains("VALIDATION_FAILED"));
        assert!(json.contains("bad slug"));
    }

    #[test]
    fn internal_error_code_matches_web_registry() {
        // The web tier's lib/api/errors.ts registers "INTERNAL_ERROR" —
        // these must stay paired.
        let err = ApiError::internal();
        assert_eq!(err.code, "INTERNAL_ERROR");
    }
}
