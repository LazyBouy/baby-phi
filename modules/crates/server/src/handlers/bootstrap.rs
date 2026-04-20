//! HTTP handlers for the System Bootstrap flow (s01).
//!
//! - `GET  /api/v0/bootstrap/status` — probe whether a platform admin
//!   exists. Unauthenticated.
//! - `POST /api/v0/bootstrap/claim`  — consume the single-use bootstrap
//!   credential and materialise the platform admin. Unauthenticated on
//!   the request, but sets the signed `baby_phi_session` cookie on success.
//!
//! Business logic lives in [`crate::bootstrap::execute_claim`]; this
//! module is the thin HTTP shim that does content-type negotiation,
//! maps [`ClaimRejection`] → HTTP status, attaches the session cookie,
//! and records the `baby_phi_bootstrap_claims_total{result}` counter.

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use domain::model::ids::{AgentId, AuditEventId, AuthRequestId, GrantId, NodeId};
use domain::model::nodes::ChannelKind;
use metrics::counter;
use serde::{Deserialize, Serialize};
use tower_cookies::Cookies;
use tracing::{error, info};

use crate::bootstrap::{execute_claim, ClaimError, ClaimInput, ClaimRejection};
use crate::session::sign_and_build_cookie;
use crate::state::AppState;

/// Counter name. Labelled by `result` ∈ {success, invalid, already_consumed,
/// already_claimed, validation, internal}. Wired via the `metrics` facade
/// crate so it shows up on `/metrics` via `axum-prometheus`.
pub const CLAIMS_COUNTER: &str = "baby_phi_bootstrap_claims_total";

// ---- GET /api/v0/bootstrap/status -----------------------------------------

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum StatusResponse {
    Unclaimed {
        claimed: bool,
        awaiting_credential: bool,
    },
    Claimed {
        claimed: bool,
        admin_agent_id: String,
    },
}

pub async fn status(State(state): State<AppState>) -> Response {
    match state.repo.get_admin_agent().await {
        Ok(Some(agent)) => (
            StatusCode::OK,
            Json(StatusResponse::Claimed {
                claimed: true,
                admin_agent_id: agent.id.to_string(),
            }),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::OK,
            Json(StatusResponse::Unclaimed {
                claimed: false,
                awaiting_credential: true,
            }),
        )
            .into_response(),
        Err(e) => {
            error!(error = %e, "bootstrap status: repository error");
            internal_error()
        }
    }
}

// ---- POST /api/v0/bootstrap/claim -----------------------------------------

#[derive(Debug, Deserialize)]
pub struct ClaimRequest {
    pub bootstrap_credential: String,
    pub display_name: String,
    pub channel: ChannelRequest,
}

#[derive(Debug, Deserialize)]
pub struct ChannelRequest {
    pub kind: ChannelKindInput,
    pub handle: String,
}

/// Mirrors `admin/01` §10 — the body names `slack|email|web`, lower-case.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelKindInput {
    Slack,
    Email,
    Web,
}

impl From<ChannelKindInput> for ChannelKind {
    fn from(input: ChannelKindInput) -> Self {
        match input {
            ChannelKindInput::Slack => ChannelKind::Slack,
            ChannelKindInput::Email => ChannelKind::Email,
            ChannelKindInput::Web => ChannelKind::Web,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ClaimSuccess {
    pub human_agent_id: AgentId,
    pub inbox_id: NodeId,
    pub outbox_id: NodeId,
    pub grant_id: GrantId,
    pub bootstrap_auth_request_id: AuthRequestId,
    pub audit_event_id: AuditEventId,
}

/// Error envelope shared by all 4xx responses. `code` is stable (machine
/// readable); `message` is a human explanation. For `BOOTSTRAP_*` codes
/// the HTTP status is 403 per `admin/01` §10.
#[derive(Debug, Serialize)]
pub struct ApiError {
    pub code: &'static str,
    pub message: String,
}

pub async fn claim(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(body): Json<ClaimRequest>,
) -> Response {
    let input = ClaimInput {
        bootstrap_credential: body.bootstrap_credential,
        display_name: body.display_name,
        channel_kind: body.channel.kind.into(),
        channel_handle: body.channel.handle,
    };

    match execute_claim(state.repo.as_ref(), input).await {
        Ok(outcome) => {
            // Mint + attach the signed session cookie.
            match sign_and_build_cookie(&state.session, &outcome.human_agent_id.to_string()) {
                Ok((_token, cookie)) => {
                    cookies.add(cookie);
                }
                Err(e) => {
                    error!(error = %e, "bootstrap claim: failed to mint session cookie");
                    counter!(CLAIMS_COUNTER, "result" => "internal").increment(1);
                    return internal_error();
                }
            }
            info!(
                agent_id = %outcome.human_agent_id,
                grant_id = %outcome.grant_id,
                audit_event_id = %outcome.audit_event_id,
                "platform admin claimed",
            );
            counter!(CLAIMS_COUNTER, "result" => "success").increment(1);
            (
                StatusCode::CREATED,
                Json(ClaimSuccess {
                    human_agent_id: outcome.human_agent_id,
                    inbox_id: outcome.inbox_id,
                    outbox_id: outcome.outbox_id,
                    grant_id: outcome.grant_id,
                    bootstrap_auth_request_id: outcome.bootstrap_auth_request_id,
                    audit_event_id: outcome.audit_event_id,
                }),
            )
                .into_response()
        }
        Err(ClaimError::Rejected(rej)) => reject(rej),
        Err(e) => {
            error!(error = %e, "bootstrap claim: internal error");
            counter!(CLAIMS_COUNTER, "result" => "internal").increment(1);
            internal_error()
        }
    }
}

fn reject(rej: ClaimRejection) -> Response {
    match rej {
        ClaimRejection::Invalid(reason) => {
            counter!(CLAIMS_COUNTER, "result" => "validation").increment(1);
            (
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    code: "VALIDATION_FAILED",
                    message: reason.to_string(),
                }),
            )
                .into_response()
        }
        ClaimRejection::CredentialInvalid => {
            counter!(CLAIMS_COUNTER, "result" => "invalid").increment(1);
            (
                StatusCode::FORBIDDEN,
                Json(ApiError {
                    code: "BOOTSTRAP_INVALID",
                    message: "bootstrap credential is not recognised".into(),
                }),
            )
                .into_response()
        }
        ClaimRejection::CredentialAlreadyConsumed => {
            counter!(CLAIMS_COUNTER, "result" => "already_consumed").increment(1);
            (
                StatusCode::FORBIDDEN,
                Json(ApiError {
                    code: "BOOTSTRAP_ALREADY_CONSUMED",
                    message: "bootstrap credential has already been consumed".into(),
                }),
            )
                .into_response()
        }
        ClaimRejection::AlreadyClaimed => {
            counter!(CLAIMS_COUNTER, "result" => "already_claimed").increment(1);
            (
                StatusCode::CONFLICT,
                Json(ApiError {
                    code: "PLATFORM_ADMIN_CLAIMED",
                    message: "a platform admin has already been claimed".into(),
                }),
            )
                .into_response()
        }
    }
}

fn internal_error() -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiError {
            code: "INTERNAL_ERROR",
            message: "an internal error occurred".into(),
        }),
    )
        .into_response()
}
