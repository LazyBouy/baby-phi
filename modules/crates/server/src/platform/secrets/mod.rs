// `SecretError::RevealDenied` carries an `ApiError` + several String
// fields — larger than clippy's default tolerance for the `Err`
// variant but acceptable here: denial is a cold path that never
// allocates on the happy flow.
#![allow(clippy::result_large_err)]

//! Page 04 — Credentials Vault business logic.
//!
//! Each submodule exposes one operation — [`add`], [`rotate`], [`reveal`],
//! [`reassign`], [`list`]. Handlers in [`crate::handlers::platform_secrets`]
//! call these functions, map the returned `Result` to HTTP, and wire up
//! the audit emitter.
//!
//! Design invariants (plan §P4):
//! - Every write is a **Template E** Auth Request: the platform admin
//!   both requests and approves. Pure helper lives in
//!   [`domain::templates::e::build_auto_approved_request`]; persistence
//!   is sequential in M2 (repo doesn't expose an atomic "AR + write"
//!   batch API for M2 writes — M3 consolidates).
//! - **Reveal is a real Permission Check** with `purpose=reveal`
//!   (decision D11). Callers that omit the constraint land at
//!   `FailedStep::Constraint` — the engine stays the single source of
//!   permission truth.
//! - **Plaintext never appears** in audit diffs. The `SealedBlob`
//!   envelope stays in the domain/store boundary; reveal returns the
//!   unsealed bytes in the HTTP response only.
//! - Audit events fire **before** the HTTP response. Reveal audits
//!   before streaming the plaintext so a crash between emit + send
//!   leaves a trail.

pub mod add;
pub mod list;
pub mod reassign;
pub mod reveal;
pub mod rotate;

use domain::model::ids::{AuditEventId, AuthRequestId, SecretId};
use domain::model::SecretCredential;

/// Stable resource-uri scheme for vault entries — matches the catalogue
/// seed done at [`add::add_secret`] time and the engine-facing
/// `target_uri` used at [`reveal::reveal_secret`] time.
pub fn secret_uri(slug: &str) -> String {
    format!("secret:{slug}")
}

/// Tag placed on every secret record so Permission Check
/// Step 3 can match `#kind:` filters on manifests that target the
/// vault.
pub const KIND_TAG: &str = "#kind:secret_credential";

/// Outcome returned by [`add::add_secret`] once the row is durable
/// and the audit event has been emitted.
#[derive(Debug, Clone)]
pub struct AddOutcome {
    pub credential: SecretCredential,
    pub auth_request_id: AuthRequestId,
    pub audit_event_id: AuditEventId,
}

/// Outcome returned by [`rotate::rotate_secret`].
#[derive(Debug, Clone)]
pub struct RotateOutcome {
    pub secret_id: SecretId,
    pub slug: String,
    pub audit_event_id: AuditEventId,
}

/// Outcome returned by [`reassign::reassign_custody`].
#[derive(Debug, Clone)]
pub struct ReassignOutcome {
    pub secret_id: SecretId,
    pub slug: String,
    pub audit_event_id: AuditEventId,
}

/// Outcome returned by [`reveal::reveal_secret`]. `plaintext` is the
/// Vec<u8> decrypted from the sealed blob — the handler renders it as
/// a base64 string in the JSON response.
#[derive(Debug)]
pub struct RevealOutcome {
    pub secret_id: SecretId,
    pub slug: String,
    /// Decrypted material. Never logged; scrubbed from Debug output.
    pub plaintext: Vec<u8>,
    pub audit_event_id: AuditEventId,
}

/// Business-logic error vocabulary. Handlers map each variant to a
/// stable `ApiError` code. Every variant carries enough context for a
/// useful user-facing message; callers do not re-query.
#[derive(Debug)]
pub enum SecretError {
    /// Input failed shape validation (empty / bad-character slug, etc.).
    Validation(String),
    /// A row with this slug already exists.
    SlugInUse(String),
    /// No row matches the supplied slug.
    NotFound(String),
    /// Permission Check denied the reveal. Handlers forward the paired
    /// `ApiError` directly (the mapping + stable code live in
    /// [`crate::handler_support::permission::denial_to_api_error`]);
    /// this variant lets business code route the
    /// `SecretRevealAttemptDenied` audit write before converting.
    RevealDenied {
        secret_id: SecretId,
        slug: String,
        failed_step: String,
        reason: String,
        api_error: crate::handler_support::ApiError,
    },
    /// Permission Check returned `Pending` — subordinate consent is
    /// required. Surfaces as 202 AWAITING_CONSENT per D10.
    RevealPending,
    /// Sealing / unsealing the material failed. Maps to 500.
    Crypto(String),
    /// Repository returned an error. Maps to 500.
    Repository(String),
    /// Audit emitter returned an error. Maps to 500 AUDIT_EMIT_FAILED.
    AuditEmit(String),
}

impl std::fmt::Display for SecretError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecretError::Validation(m) => write!(f, "validation failed: {m}"),
            SecretError::SlugInUse(s) => write!(f, "slug `{s}` is already in use"),
            SecretError::NotFound(s) => write!(f, "no secret with slug `{s}`"),
            SecretError::RevealDenied {
                slug, failed_step, ..
            } => write!(f, "reveal denied for `{slug}` at step {failed_step}"),
            SecretError::RevealPending => write!(f, "subordinate consent required"),
            SecretError::Crypto(m) => write!(f, "crypto: {m}"),
            SecretError::Repository(m) => write!(f, "repository: {m}"),
            SecretError::AuditEmit(m) => write!(f, "audit emit: {m}"),
        }
    }
}

impl std::error::Error for SecretError {}

impl From<domain::repository::RepositoryError> for SecretError {
    fn from(e: domain::repository::RepositoryError) -> Self {
        SecretError::Repository(e.to_string())
    }
}

impl From<store::crypto::CryptoError> for SecretError {
    fn from(e: store::crypto::CryptoError) -> Self {
        SecretError::Crypto(e.to_string())
    }
}

/// Lightweight regex for vault slugs: lowercase letters, digits, and
/// dashes. Reject everything else so catalogue URIs stay ASCII-clean
/// and the CLI doesn't need shell-escape rules.
pub fn validate_slug(slug: &str) -> Result<(), SecretError> {
    if slug.is_empty() {
        return Err(SecretError::Validation("slug must not be empty".into()));
    }
    if slug.len() > 64 {
        return Err(SecretError::Validation(format!(
            "slug `{slug}` exceeds 64 chars"
        )));
    }
    if !slug
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(SecretError::Validation(format!(
            "slug `{slug}` must match [a-z0-9-]+"
        )));
    }
    if slug.starts_with('-') || slug.ends_with('-') {
        return Err(SecretError::Validation(format!(
            "slug `{slug}` must not start/end with `-`"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_slug_accepts_canonical_shape() {
        validate_slug("anthropic-api-key").unwrap();
        validate_slug("openai").unwrap();
        validate_slug("key-42").unwrap();
    }

    #[test]
    fn validate_slug_rejects_empty_or_invalid_chars() {
        assert!(validate_slug("").is_err());
        assert!(validate_slug("UPPER").is_err());
        assert!(validate_slug("space key").is_err());
        assert!(validate_slug("under_score").is_err());
        assert!(validate_slug("-leading").is_err());
        assert!(validate_slug("trailing-").is_err());
    }

    #[test]
    fn validate_slug_rejects_overlong_input() {
        let long = "a".repeat(65);
        assert!(validate_slug(&long).is_err());
    }

    #[test]
    fn secret_uri_has_stable_prefix() {
        assert_eq!(secret_uri("anthropic-api-key"), "secret:anthropic-api-key");
    }
}
