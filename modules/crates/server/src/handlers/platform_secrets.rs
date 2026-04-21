//! HTTP handlers for the credentials-vault surface (page 04).
//!
//! Five routes, all gated by the `AuthenticatedSession` extractor:
//!
//! | Method | Path | Op |
//! |---|---|---|
//! | `GET`  | `/api/v0/platform/secrets` | list (metadata only) |
//! | `POST` | `/api/v0/platform/secrets` | add |
//! | `POST` | `/api/v0/platform/secrets/{slug}/rotate` | rotate material |
//! | `POST` | `/api/v0/platform/secrets/{slug}/reveal` | unseal + return plaintext |
//! | `POST` | `/api/v0/platform/secrets/{slug}/reassign-custody` | swap custodian agent |
//!
//! Plaintext material is exchanged as base64 (standard alphabet, **no**
//! padding — matches `store::crypto::SealedSecret::to_base64`). The
//! handler decodes on the way in; `reveal` encodes on the way out.
//!
//! Error mapping:
//! - Validation / slug shape → `400 VALIDATION_FAILED`.
//! - Slug conflict on add → `409 SECRET_SLUG_IN_USE`.
//! - Missing slug on rotate/reveal/reassign → `404 SECRET_NOT_FOUND`.
//! - Reveal denied → stable code from `denial_to_api_error` (usually
//!   `403 CONSTRAINT_VIOLATION` for a missing `purpose=reveal`).
//! - Crypto / repository / audit failure → `500` with the matching code.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use base64::engine::general_purpose::STANDARD_NO_PAD as BASE64_NOPAD;
use base64::Engine as _;
use chrono::Utc;
use domain::model::ids::{AgentId, AuditEventId, AuthRequestId, SecretId};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::handler_support::{ApiError, AuthenticatedSession};
use crate::platform::secrets::{
    add::{add_secret, AddInput},
    list::list_secrets,
    reassign::{reassign_custody, ReassignInput},
    reveal::{reveal_secret, RevealInput},
    rotate::{rotate_secret, RotateInput},
    SecretError,
};
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Wire shapes — requests
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct AddSecretRequest {
    pub slug: String,
    /// Base64 (no padding) plaintext material.
    pub material_b64: String,
    #[serde(default = "default_sensitive")]
    pub sensitive: bool,
}

fn default_sensitive() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct RotateSecretRequest {
    pub material_b64: String,
}

#[derive(Debug, Deserialize)]
pub struct RevealSecretRequest {
    /// Free-form human justification. Must be non-empty.
    pub justification: String,
}

#[derive(Debug, Deserialize)]
pub struct ReassignCustodyRequest {
    pub new_custodian_agent_id: String,
}

// ---------------------------------------------------------------------------
// Wire shapes — responses
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct SecretSummary {
    pub id: SecretId,
    pub slug: String,
    pub custodian_id: AgentId,
    pub sensitive: bool,
    pub last_rotated_at: Option<chrono::DateTime<Utc>>,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ListSecretsResponse {
    pub secrets: Vec<SecretSummary>,
}

#[derive(Debug, Serialize)]
pub struct AddSecretResponse {
    pub secret_id: SecretId,
    pub slug: String,
    pub auth_request_id: AuthRequestId,
    pub audit_event_id: AuditEventId,
}

#[derive(Debug, Serialize)]
pub struct RotateSecretResponse {
    pub secret_id: SecretId,
    pub slug: String,
    pub audit_event_id: AuditEventId,
}

#[derive(Debug, Serialize)]
pub struct ReassignCustodyResponse {
    pub secret_id: SecretId,
    pub slug: String,
    pub audit_event_id: AuditEventId,
}

#[derive(Debug, Serialize)]
pub struct RevealSecretResponse {
    pub secret_id: SecretId,
    pub slug: String,
    pub material_b64: String,
    pub audit_event_id: AuditEventId,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

pub async fn list(
    State(state): State<AppState>,
    session: AuthenticatedSession,
) -> Result<Response, ApiError> {
    let outcome = list_secrets(
        state.repo.clone(),
        state.audit.clone(),
        session.agent_id,
        Utc::now(),
    )
    .await
    .map_err(error_to_api_error)?;
    let secrets = outcome
        .credentials
        .into_iter()
        .map(|c| SecretSummary {
            id: c.id,
            slug: c.slug.as_str().to_string(),
            custodian_id: c.custodian,
            sensitive: c.sensitive,
            last_rotated_at: c.last_rotated_at,
            created_at: c.created_at,
        })
        .collect();
    Ok((StatusCode::OK, Json(ListSecretsResponse { secrets })).into_response())
}

pub async fn add(
    State(state): State<AppState>,
    session: AuthenticatedSession,
    Json(body): Json<AddSecretRequest>,
) -> Result<Response, ApiError> {
    let material = BASE64_NOPAD.decode(body.material_b64.trim()).map_err(|e| {
        ApiError::validation_failed(format!("material_b64 is not valid base64: {e}"))
    })?;
    if material.is_empty() {
        return Err(ApiError::validation_failed("material must be non-empty"));
    }

    let outcome = add_secret(
        state.repo.clone(),
        state.audit.clone(),
        &state.master_key,
        AddInput {
            slug: &body.slug,
            plaintext: &material,
            sensitive: body.sensitive,
            actor: session.agent_id,
            now: Utc::now(),
        },
    )
    .await
    .map_err(error_to_api_error)?;

    info!(
        secret_id = %outcome.credential.id,
        slug = %body.slug,
        auth_request_id = %outcome.auth_request_id,
        audit_event_id = %outcome.audit_event_id,
        "vault: secret added",
    );

    Ok((
        StatusCode::CREATED,
        Json(AddSecretResponse {
            secret_id: outcome.credential.id,
            slug: body.slug,
            auth_request_id: outcome.auth_request_id,
            audit_event_id: outcome.audit_event_id,
        }),
    )
        .into_response())
}

pub async fn rotate(
    State(state): State<AppState>,
    session: AuthenticatedSession,
    Path(slug): Path<String>,
    Json(body): Json<RotateSecretRequest>,
) -> Result<Response, ApiError> {
    let material = BASE64_NOPAD.decode(body.material_b64.trim()).map_err(|e| {
        ApiError::validation_failed(format!("material_b64 is not valid base64: {e}"))
    })?;
    if material.is_empty() {
        return Err(ApiError::validation_failed("material must be non-empty"));
    }

    let outcome = rotate_secret(
        state.repo.clone(),
        state.audit.clone(),
        &state.master_key,
        RotateInput {
            slug: &slug,
            plaintext: &material,
            actor: session.agent_id,
            now: Utc::now(),
        },
    )
    .await
    .map_err(error_to_api_error)?;

    info!(
        secret_id = %outcome.secret_id,
        slug = %outcome.slug,
        audit_event_id = %outcome.audit_event_id,
        "vault: secret rotated",
    );

    Ok((
        StatusCode::OK,
        Json(RotateSecretResponse {
            secret_id: outcome.secret_id,
            slug: outcome.slug,
            audit_event_id: outcome.audit_event_id,
        }),
    )
        .into_response())
}

pub async fn reveal(
    State(state): State<AppState>,
    session: AuthenticatedSession,
    Path(slug): Path<String>,
    Json(body): Json<RevealSecretRequest>,
) -> Result<Response, ApiError> {
    let outcome = reveal_secret(
        state.repo.clone(),
        state.audit.clone(),
        &state.master_key,
        RevealInput {
            slug: &slug,
            justification: &body.justification,
            actor: session.agent_id,
            now: Utc::now(),
        },
    )
    .await
    .map_err(error_to_api_error)?;

    info!(
        secret_id = %outcome.secret_id,
        slug = %outcome.slug,
        audit_event_id = %outcome.audit_event_id,
        "vault: secret revealed",
    );

    let material_b64 = BASE64_NOPAD.encode(&outcome.plaintext);
    Ok((
        StatusCode::OK,
        Json(RevealSecretResponse {
            secret_id: outcome.secret_id,
            slug: outcome.slug,
            material_b64,
            audit_event_id: outcome.audit_event_id,
        }),
    )
        .into_response())
}

pub async fn reassign(
    State(state): State<AppState>,
    session: AuthenticatedSession,
    Path(slug): Path<String>,
    Json(body): Json<ReassignCustodyRequest>,
) -> Result<Response, ApiError> {
    let new_custodian: AgentId = body
        .new_custodian_agent_id
        .parse::<uuid::Uuid>()
        .map(AgentId::from_uuid)
        .map_err(|_| ApiError::validation_failed("new_custodian_agent_id must be a UUID"))?;

    let outcome = reassign_custody(
        state.repo.clone(),
        state.audit.clone(),
        ReassignInput {
            slug: &slug,
            new_custodian,
            actor: session.agent_id,
            now: Utc::now(),
        },
    )
    .await
    .map_err(error_to_api_error)?;

    info!(
        secret_id = %outcome.secret_id,
        slug = %outcome.slug,
        new_custodian = %new_custodian,
        audit_event_id = %outcome.audit_event_id,
        "vault: custody reassigned",
    );

    Ok((
        StatusCode::OK,
        Json(ReassignCustodyResponse {
            secret_id: outcome.secret_id,
            slug: outcome.slug,
            audit_event_id: outcome.audit_event_id,
        }),
    )
        .into_response())
}

// ---------------------------------------------------------------------------
// Error mapping
// ---------------------------------------------------------------------------

fn error_to_api_error(err: SecretError) -> ApiError {
    match err {
        SecretError::Validation(m) => ApiError::validation_failed(m),
        SecretError::SlugInUse(slug) => ApiError::new(
            StatusCode::CONFLICT,
            "SECRET_SLUG_IN_USE",
            format!("slug `{slug}` is already in use"),
        ),
        SecretError::NotFound(slug) => ApiError::new(
            StatusCode::NOT_FOUND,
            "SECRET_NOT_FOUND",
            format!("no secret with slug `{slug}`"),
        ),
        SecretError::RevealDenied { api_error, .. } => api_error,
        SecretError::RevealPending => ApiError::new(
            StatusCode::ACCEPTED,
            "AWAITING_CONSENT",
            "subordinate consent required before reveal",
        ),
        SecretError::Crypto(m) => {
            error!(error = %m, "vault: crypto error");
            ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "VAULT_CRYPTO_FAILED", m)
        }
        SecretError::Repository(m) => {
            error!(error = %m, "vault: repository error");
            ApiError::internal()
        }
        SecretError::AuditEmit(m) => ApiError::audit_emit_failed(m),
    }
}
