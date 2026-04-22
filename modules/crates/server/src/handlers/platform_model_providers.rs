//! HTTP handlers for the model-providers surface (page 02).
//!
//! Five routes, all gated by `AuthenticatedSession` except
//! `provider-kinds` which is a read-only enumeration safe for any
//! authenticated caller:
//!
//! | Method | Path | Op |
//! |---|---|---|
//! | `GET`  | `/api/v0/platform/model-providers` | list registered runtimes |
//! | `POST` | `/api/v0/platform/model-providers` | register a new runtime |
//! | `POST` | `/api/v0/platform/model-providers/{id}/archive` | soft-delete |
//! | `GET`  | `/api/v0/platform/provider-kinds` | phi-core-driven kind enumeration |
//!
//! Wire body for POST mirrors [`phi_core::provider::model::ModelConfig`]
//! directly — clients construct the config object exactly as phi-core's
//! own factory methods produce it. phi adds three wrapper fields
//! (`secret_ref`, `tenants_allowed`, `config`); no other transcription.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::{DateTime, Utc};
use domain::model::ids::{AuditEventId, AuthRequestId, ModelProviderId};
use domain::model::{RuntimeStatus, SecretRef, TenantSet};
use phi_core::provider::model::{ApiProtocol, ModelConfig};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use uuid::Uuid;

use crate::handler_support::{ApiError, AuthenticatedSession};
use crate::platform::model_providers::{
    archive::{archive_provider, ArchiveInput},
    list::{list_providers, ListInput},
    provider_kinds::list_provider_kinds,
    register::{register_provider, RegisterInput},
    ProviderError,
};
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Wire shapes — requests
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct RegisterProviderRequest {
    /// The phi-core `ModelConfig` — accepted verbatim. The `api_key`
    /// field is scrubbed server-side before persistence.
    pub config: ModelConfig,
    /// Vault slug that holds the real API key at invocation time.
    pub secret_ref: String,
    /// Which orgs may invoke this runtime. Defaults to `All`.
    #[serde(default = "default_tenants")]
    pub tenants_allowed: TenantSet,
}

fn default_tenants() -> TenantSet {
    TenantSet::All
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    pub include_archived: bool,
}

// ---------------------------------------------------------------------------
// Wire shapes — responses
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct ProviderSummary {
    pub id: ModelProviderId,
    /// phi-core `ModelConfig`, serialised verbatim. `api_key` is
    /// always the empty sentinel — vault reveal is the only path to
    /// plaintext.
    pub config: ModelConfig,
    pub secret_ref: String,
    pub tenants_allowed: TenantSet,
    pub status: RuntimeStatus,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ListProvidersResponse {
    pub providers: Vec<ProviderSummary>,
}

#[derive(Debug, Serialize)]
pub struct RegisterProviderResponse {
    pub provider_id: ModelProviderId,
    pub auth_request_id: AuthRequestId,
    pub audit_event_id: AuditEventId,
}

#[derive(Debug, Serialize)]
pub struct ArchiveProviderResponse {
    pub provider_id: ModelProviderId,
    pub audit_event_id: AuditEventId,
}

#[derive(Debug, Serialize)]
pub struct ProviderKindsResponse {
    /// phi-core's `ApiProtocol` variants, serialised in snake_case.
    pub kinds: Vec<ApiProtocol>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

pub async fn list(
    State(state): State<AppState>,
    _session: AuthenticatedSession,
    Query(params): Query<ListQuery>,
) -> Result<Response, ApiError> {
    let outcome = list_providers(
        state.repo.clone(),
        ListInput {
            include_archived: params.include_archived,
        },
    )
    .await
    .map_err(error_to_api_error)?;
    let providers = outcome
        .runtimes
        .into_iter()
        .map(|rt| ProviderSummary {
            id: rt.id,
            config: rt.config,
            secret_ref: rt.secret_ref.as_str().to_string(),
            tenants_allowed: rt.tenants_allowed,
            status: rt.status,
            archived_at: rt.archived_at,
            created_at: rt.created_at,
        })
        .collect();
    Ok((StatusCode::OK, Json(ListProvidersResponse { providers })).into_response())
}

pub async fn register(
    State(state): State<AppState>,
    session: AuthenticatedSession,
    Json(body): Json<RegisterProviderRequest>,
) -> Result<Response, ApiError> {
    let outcome = register_provider(
        state.repo.clone(),
        state.audit.clone(),
        RegisterInput {
            config: body.config,
            secret_ref: SecretRef::new(body.secret_ref),
            tenants_allowed: body.tenants_allowed,
            actor: session.agent_id,
            now: Utc::now(),
        },
    )
    .await
    .map_err(error_to_api_error)?;

    info!(
        provider_id = %outcome.runtime.id,
        provider = %outcome.runtime.config.provider,
        model = %outcome.runtime.config.id,
        auth_request_id = %outcome.auth_request_id,
        audit_event_id = %outcome.audit_event_id,
        "model_provider: registered",
    );

    Ok((
        StatusCode::CREATED,
        Json(RegisterProviderResponse {
            provider_id: outcome.runtime.id,
            auth_request_id: outcome.auth_request_id,
            audit_event_id: outcome.audit_event_id,
        }),
    )
        .into_response())
}

pub async fn archive(
    State(state): State<AppState>,
    session: AuthenticatedSession,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    let provider_id = Uuid::parse_str(&id)
        .map(ModelProviderId::from_uuid)
        .map_err(|_| ApiError::validation_failed("provider id must be a UUID"))?;

    let outcome = archive_provider(
        state.repo.clone(),
        state.audit.clone(),
        ArchiveInput {
            provider_id,
            actor: session.agent_id,
            now: Utc::now(),
        },
    )
    .await
    .map_err(error_to_api_error)?;

    info!(
        provider_id = %outcome.provider_id,
        audit_event_id = %outcome.audit_event_id,
        "model_provider: archived",
    );

    Ok((
        StatusCode::OK,
        Json(ArchiveProviderResponse {
            provider_id: outcome.provider_id,
            audit_event_id: outcome.audit_event_id,
        }),
    )
        .into_response())
}

pub async fn provider_kinds(_session: AuthenticatedSession) -> Result<Response, ApiError> {
    let kinds = list_provider_kinds();
    Ok((StatusCode::OK, Json(ProviderKindsResponse { kinds })).into_response())
}

// ---------------------------------------------------------------------------
// Error mapping
// ---------------------------------------------------------------------------

fn error_to_api_error(err: ProviderError) -> ApiError {
    match err {
        ProviderError::Validation(m) => ApiError::validation_failed(m),
        ProviderError::DuplicateProvider { provider, model_id } => ApiError::new(
            StatusCode::CONFLICT,
            "MODEL_PROVIDER_DUPLICATE",
            format!("provider `{provider}` + model `{model_id}` already registered"),
        ),
        ProviderError::SecretRefNotFound(slug) => ApiError::new(
            StatusCode::BAD_REQUEST,
            "SECRET_REF_NOT_FOUND",
            format!("secret_ref `{slug}` not found in vault"),
        ),
        ProviderError::NotFound(id) => ApiError::new(
            StatusCode::NOT_FOUND,
            "MODEL_PROVIDER_NOT_FOUND",
            format!("no model provider with id `{id}`"),
        ),
        ProviderError::Repository(m) => {
            error!(error = %m, "model_provider: repository error");
            ApiError::internal()
        }
        ProviderError::AuditEmit(m) => ApiError::audit_emit_failed(m),
    }
}
