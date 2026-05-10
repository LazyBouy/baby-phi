//! `archive_mcp_server` — soft-delete an external-service row.
//!
//! Sets `archived_at` + emits `platform.mcp_server.archived` (Alerted).
//! Grants descending from the original registration AR are **not**
//! revoked here — M2 has a single admin holding them; M3 wires the
//! cascade when delegated grants become common (plan Part 11 Q8).

use std::sync::Arc;

use chrono::{DateTime, Utc};
use domain::audit::events::m2::mcp as mcp_events;
use domain::audit::{AuditClass, AuditEmitter};
use domain::auth_requests::access::{check_auth_request_access, IntendedOp};
use domain::model::ids::{AgentId, McpServerId};
use domain::model::nodes::{AuthRequest, AuthRequestState, PrincipalRef, ResourceRef};
use domain::repository::Repository;
use domain::templates::e::{build_auto_approved_request, BuildArgs};

use super::{mcp_server_uri, ArchiveOutcome, McpError, KIND_TAG};

pub struct ArchiveInput {
    pub mcp_server_id: McpServerId,
    pub actor: AgentId,
    pub now: DateTime<Utc>,
}

pub async fn archive_mcp_server(
    repo: Arc<dyn Repository>,
    audit: Arc<dyn AuditEmitter>,
    input: ArchiveInput,
) -> Result<ArchiveOutcome, McpError> {
    // Look up the row (for the audit diff + a clean 404 on unknown id).
    let existing = repo
        .list_mcp_servers(true)
        .await?
        .into_iter()
        .find(|s| s.id == input.mcp_server_id)
        .ok_or(McpError::NotFound(input.mcp_server_id))?;

    // Template E AR for the archive write — self-approved.
    let uri = mcp_server_uri(input.mcp_server_id);
    let ar = build_auto_approved_request(BuildArgs {
        requestor_and_approver: PrincipalRef::Agent(input.actor),
        resource: ResourceRef { uri },
        kinds: vec![KIND_TAG.to_string()],
        scope: vec!["archive".to_string()],
        justification: Some(format!(
            "self-approved platform-admin write: archive MCP server `{}`",
            input.mcp_server_id
        )),
        audit_class: AuditClass::Alerted,
        now: input.now,
    });
    let auth_request_id = ar.id;

    // CH-18 / ADR-0056 §D56.5 + F3.B.create-side.a — defence-in-depth
    // Submit gate (synthetic-Draft probe — see `defaults/put.rs`).
    let probe = AuthRequest {
        state: AuthRequestState::Draft,
        ..ar.clone()
    };
    check_auth_request_access(
        &probe,
        &PrincipalRef::Agent(input.actor),
        IntendedOp::Submit,
    )?;

    repo.create_auth_request(&ar).await?;

    // Flip the row.
    repo.archive_mcp_server(input.mcp_server_id, input.now)
        .await?;

    // Build the archive event with the PRE-archive snapshot in the
    // "before" diff so a reviewer sees what the server looked like
    // before archival.
    let event =
        mcp_events::mcp_server_archived(input.actor, &existing, Some(auth_request_id), input.now);
    let audit_event_id = event.event_id;
    audit
        .emit(event)
        .await
        .map_err(|e| McpError::AuditEmit(e.to_string()))?;

    Ok(ArchiveOutcome {
        mcp_server_id: input.mcp_server_id,
        audit_event_id,
    })
}
