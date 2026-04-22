//! `register_mcp_server` — bind a new MCP / external service.
//!
//! Flow (plan §P6 + §P4.5 grant shape):
//! 1. Validate input — display_name / endpoint non-empty; secret_ref
//!    slug shape (if present).
//! 2. Verify `secret_ref` exists in the vault (referential integrity)
//!    — only when one was supplied; MCP services without auth are
//!    valid.
//! 3. Mint a Template E Auth Request.
//! 4. Persist in order: AR → MCP server row → catalogue seed
//!    (`external_service:<id>`) → per-instance `[invoke]` grant with
//!    `fundamentals = [NetworkEndpoint, SecretCredential, Tag]`
//!    (matches `Composite::ExternalServiceObject`).
//! 5. Emit `platform.mcp_server.registered` (Alerted).
//!
//! phi-core leverage: the stored `endpoint` string is the verbatim
//! argument phi-core's [`McpClient::connect_stdio`] / `connect_http`
//! accepts — at invoke time, handlers parse it and hand it to the
//! phi-core constructor without any phi-local reinterpretation.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use domain::audit::events::m2::mcp as mcp_events;
use domain::audit::{AuditClass, AuditEmitter};
use domain::model::ids::{AgentId, GrantId, McpServerId};
use domain::model::nodes::{Grant, PrincipalRef, ResourceRef};
use domain::model::{
    Composite, ExternalService, ExternalServiceKind, Fundamental, RuntimeStatus, SecretRef,
    TenantSet,
};
use domain::repository::Repository;
use domain::templates::e::{build_auto_approved_request, BuildArgs};

use super::{mcp_server_uri, McpError, RegisterOutcome, KIND_TAG};

/// Inputs the HTTP handler hands over after decoding the JSON body.
pub struct RegisterInput {
    pub display_name: String,
    pub kind: ExternalServiceKind,
    pub endpoint: String,
    pub secret_ref: Option<SecretRef>,
    pub tenants_allowed: TenantSet,
    pub actor: AgentId,
    pub now: DateTime<Utc>,
}

pub async fn register_mcp_server(
    repo: Arc<dyn Repository>,
    audit: Arc<dyn AuditEmitter>,
    input: RegisterInput,
) -> Result<RegisterOutcome, McpError> {
    // 1. Validate.
    if input.display_name.trim().is_empty() {
        return Err(McpError::Validation(
            "display_name must be non-empty".into(),
        ));
    }
    if input.endpoint.trim().is_empty() {
        return Err(McpError::Validation("endpoint must be non-empty".into()));
    }
    if let Some(sr) = input.secret_ref.as_ref() {
        crate::platform::secrets::validate_slug(sr.as_str())
            .map_err(|e| McpError::Validation(format!("secret_ref: {e}")))?;
    }

    // 2. Referential integrity on secret_ref (when supplied).
    if let Some(sr) = input.secret_ref.as_ref() {
        if repo.get_secret_by_slug(sr).await?.is_none() {
            return Err(McpError::SecretRefNotFound(sr.as_str().to_string()));
        }
    }

    // 3. Template E AR — self-approved platform-admin write.
    let mcp_id = McpServerId::new();
    let uri = mcp_server_uri(mcp_id);
    let ar = build_auto_approved_request(BuildArgs {
        requestor_and_approver: PrincipalRef::Agent(input.actor),
        resource: ResourceRef { uri: uri.clone() },
        kinds: vec![KIND_TAG.to_string()],
        scope: vec![uri.clone()],
        justification: Some(format!(
            "self-approved platform-admin write: register MCP server `{}` ({:?})",
            input.display_name, input.kind
        )),
        audit_class: AuditClass::Alerted,
        now: input.now,
    });
    let auth_request_id = ar.id;

    // 4. Persist sequentially.
    repo.create_auth_request(&ar).await?;

    let service = ExternalService {
        id: mcp_id,
        display_name: input.display_name,
        kind: input.kind,
        endpoint: input.endpoint,
        secret_ref: input.secret_ref,
        tenants_allowed: input.tenants_allowed,
        status: RuntimeStatus::Ok,
        archived_at: None,
        created_at: input.now,
    };
    repo.put_mcp_server(&service).await?;

    // Catalogue seed — `Composite::ExternalServiceObject` is a real
    // composite.
    repo.seed_catalogue_entry_for_composite(None, &uri, Composite::ExternalServiceObject)
        .await?;

    // Per-instance `[invoke]` grant on `external_service:<id>` with the
    // ExternalServiceObject's constituent fundamentals. The engine's
    // Case D (P4.5) picks this up with the URI-scoped selector.
    let grant = Grant {
        id: GrantId::new(),
        holder: PrincipalRef::Agent(input.actor),
        action: vec!["invoke".to_string()],
        resource: ResourceRef { uri: uri.clone() },
        fundamentals: vec![
            Fundamental::NetworkEndpoint,
            Fundamental::SecretCredential,
            Fundamental::Tag,
        ],
        descends_from: Some(auth_request_id),
        delegable: true,
        issued_at: input.now,
        revoked_at: None,
    };
    repo.create_grant(&grant).await?;

    // 5. Audit.
    let event =
        mcp_events::mcp_server_registered(input.actor, &service, Some(auth_request_id), input.now);
    let audit_event_id = event.event_id;
    audit
        .emit(event)
        .await
        .map_err(|e| McpError::AuditEmit(e.to_string()))?;

    Ok(RegisterOutcome {
        service,
        auth_request_id,
        audit_event_id,
    })
}
