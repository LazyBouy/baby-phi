//! Audit-event builders for page 03 (MCP / external services).
//!
//! Four event types ship with M2/P6:
//! - `platform.mcp_server.registered` (Alerted) — a new external service
//!   was bound.
//! - `platform.mcp_server.archived` (Alerted) — a server was soft-deleted.
//! - `platform.mcp_server.tenant_access_revoked` (Alerted) — the server's
//!   `tenants_allowed` was narrowed and the cascade revoked `N` grants
//!   across `M` auth requests. One summary event per PATCH, carrying the
//!   list of dropped org ids + the counts.
//! - `platform.mcp_server.health_degraded` (Alerted) — the (M7b) health
//!   probe saw a failure. M2 ships the builder only; real probe wiring
//!   is M7b per plan §G5.
//!
//! Plus a narrow helper for cascade-side-effects:
//! - `auth_request.revoked` (Alerted) — emitted once per Auth Request that
//!   lost its grants through the tenant-narrow cascade. Scoped to the
//!   `tenant_access_revoked` flow for M2; M3 generalises it when
//!   delegated revocation lands.
//!
//! phi-core leverage: MCP endpoints are transport-only strings (stdio /
//! HTTP URL); phi-core's `McpClient` is constructed on demand at probe
//! time (no phi-core data types in the persisted diff shape).

use chrono::{DateTime, Utc};

use crate::audit::{AuditClass, AuditEvent};
use crate::model::ids::{AgentId, AuditEventId, AuthRequestId, McpServerId, NodeId, OrgId};
use crate::model::ExternalService;
use crate::repository::TenantRevocation;

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn scaffold_mcp(
    event_type: &str,
    actor: AgentId,
    target: McpServerId,
    timestamp: DateTime<Utc>,
    diff: serde_json::Value,
    audit_class: AuditClass,
    provenance_auth_request_id: Option<AuthRequestId>,
) -> AuditEvent {
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: event_type.to_string(),
        actor_agent_id: Some(actor),
        target_entity_id: Some(NodeId::from_uuid(*target.as_uuid())),
        timestamp,
        diff,
        audit_class,
        provenance_auth_request_id,
        // Platform-scope writes chain under `None` (the root chain).
        org_scope: Option::<OrgId>::None,
        prev_event_hash: None,
    }
}

/// Baseline "after" snapshot shared across registered / archived. The
/// endpoint string is stored as-is — M2 persists the phi-core transport
/// argument (stdio command or HTTP URL) verbatim.
fn after_snapshot(svc: &ExternalService) -> serde_json::Value {
    serde_json::json!({
        "mcp_server_id":   svc.id.to_string(),
        "display_name":    svc.display_name,
        "kind":            svc.kind,
        "endpoint":        svc.endpoint,
        "secret_ref":      svc.secret_ref.as_ref().map(|s| s.as_str()),
        "tenants_allowed": svc.tenants_allowed,
        "status":          svc.status,
        "archived_at":     svc.archived_at,
        "created_at":      svc.created_at,
    })
}

// ---------------------------------------------------------------------------
// Builders — page-scoped events
// ---------------------------------------------------------------------------

/// `platform.mcp_server.registered` — a new external service was bound.
///
/// Emitted by the P6 `register_mcp_server` handler after Template E AR +
/// catalogue seed + per-instance grant are durable. Alerted.
pub fn mcp_server_registered(
    actor: AgentId,
    service: &ExternalService,
    provenance_auth_request_id: Option<AuthRequestId>,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    let diff = serde_json::json!({
        "before": serde_json::Value::Null,
        "after":  after_snapshot(service),
    });
    scaffold_mcp(
        "platform.mcp_server.registered",
        actor,
        service.id,
        timestamp,
        diff,
        AuditClass::Alerted,
        provenance_auth_request_id,
    )
}

/// `platform.mcp_server.archived` — MCP server soft-deleted.
///
/// Mirrors the provider archive flow: grants descending from the
/// original registration AR are **not** cascade-revoked here (M2 has a
/// single admin holding them; M3 wires full cascade once delegation
/// lands — plan Part 11 Q8).
pub fn mcp_server_archived(
    actor: AgentId,
    service: &ExternalService,
    provenance_auth_request_id: Option<AuthRequestId>,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    let diff = serde_json::json!({
        "before": after_snapshot(service),
        "after": {
            "mcp_server_id": service.id.to_string(),
            "archived_at":   timestamp,
        },
    });
    scaffold_mcp(
        "platform.mcp_server.archived",
        actor,
        service.id,
        timestamp,
        diff,
        AuditClass::Alerted,
        provenance_auth_request_id,
    )
}

/// `platform.mcp_server.tenant_access_revoked` — narrowing cascade ran.
///
/// One event per PATCH that shrank the `tenants_allowed` set. Carries:
/// - the summary counts (`revoked_org_count`, `revoked_auth_request_count`,
///   `revoked_grant_count`) so reviewers can gauge blast radius at a
///   glance;
/// - the full list of dropped org ids + affected AR ids for forensics.
///
/// Per-AR `auth_request.revoked` events (one per AR) are emitted
/// alongside this summary via [`auth_request_revoked_by_mcp_cascade`].
pub fn mcp_server_tenant_access_revoked(
    actor: AgentId,
    service: &ExternalService,
    previous_tenants: &crate::model::TenantSet,
    revocations: &[TenantRevocation],
    provenance_auth_request_id: Option<AuthRequestId>,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    let revoked_orgs: Vec<String> = revocations.iter().map(|r| r.org.to_string()).collect();
    let affected_ars: Vec<String> = revocations
        .iter()
        .map(|r| r.auth_request.to_string())
        .collect();
    let revoked_grant_count: usize = revocations.iter().map(|r| r.revoked_grants.len()).sum();
    let diff = serde_json::json!({
        "before": {
            "mcp_server_id":   service.id.to_string(),
            "tenants_allowed": previous_tenants,
        },
        "after": {
            "mcp_server_id":               service.id.to_string(),
            "tenants_allowed":             service.tenants_allowed,
            "revoked_org_count":           revoked_orgs.len(),
            "revoked_auth_request_count":  affected_ars.len(),
            "revoked_grant_count":         revoked_grant_count,
            "revoked_orgs":                revoked_orgs,
            "affected_auth_requests":      affected_ars,
        },
    });
    scaffold_mcp(
        "platform.mcp_server.tenant_access_revoked",
        actor,
        service.id,
        timestamp,
        diff,
        AuditClass::Alerted,
        provenance_auth_request_id,
    )
}

/// `platform.mcp_server.health_degraded` — probe saw a failure.
///
/// Shipped shape-only in M2; the M7b scheduled probe wires the emit
/// call. Builder carries the old status + new status + reason so the
/// reviewer can correlate the incident.
pub fn mcp_server_health_degraded(
    actor: AgentId,
    service: &ExternalService,
    previous_status: crate::model::RuntimeStatus,
    reason: &str,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    let diff = serde_json::json!({
        "before": { "status": previous_status },
        "after": {
            "mcp_server_id": service.id.to_string(),
            "status":        service.status,
            "reason":        reason,
        },
    });
    scaffold_mcp(
        "platform.mcp_server.health_degraded",
        actor,
        service.id,
        timestamp,
        diff,
        AuditClass::Alerted,
        None,
    )
}

// ---------------------------------------------------------------------------
// Cascade side-effect builder — `auth_request.revoked`
// ---------------------------------------------------------------------------

/// `auth_request.revoked` — one per Auth Request affected by the M2/P6
/// tenant-narrow cascade.
///
/// `target_entity_id` is the AR's NodeId so the audit reader can link
/// the event straight to the revoked AR row. The diff carries the list
/// of revoked grant ids + the MCP server id that triggered the cascade
/// for cross-navigation.
///
/// Event naming is deliberately generic (`auth_request.revoked`, not
/// `…_by_mcp_cascade`) because M3 will reuse the same event for
/// delegated-grant cascades; the `reason` field differentiates.
pub fn auth_request_revoked_by_mcp_cascade(
    actor: AgentId,
    mcp_server_id: McpServerId,
    org: OrgId,
    revocation: &TenantRevocation,
    provenance_auth_request_id: Option<AuthRequestId>,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    let grant_ids: Vec<String> = revocation
        .revoked_grants
        .iter()
        .map(|g| g.to_string())
        .collect();
    let diff = serde_json::json!({
        "before": serde_json::Value::Null,
        "after": {
            "auth_request_id": revocation.auth_request.to_string(),
            "org":             org.to_string(),
            "revoked_grants":  grant_ids,
            "reason":          "mcp_tenant_narrow",
            "mcp_server_id":   mcp_server_id.to_string(),
            "revoked_at":      timestamp,
        },
    });
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: "auth_request.revoked".to_string(),
        actor_agent_id: Some(actor),
        target_entity_id: Some(NodeId::from_uuid(*revocation.auth_request.as_uuid())),
        timestamp,
        diff,
        audit_class: AuditClass::Alerted,
        provenance_auth_request_id,
        org_scope: Option::<OrgId>::None,
        prev_event_hash: None,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::composites_m2::{ExternalServiceKind, RuntimeStatus, TenantSet};
    use crate::model::ids::{McpServerId, OrgId};
    use crate::model::{ExternalService, SecretRef};
    use crate::repository::TenantRevocation;

    fn sample_service() -> ExternalService {
        ExternalService {
            id: McpServerId::new(),
            display_name: "memory-mcp".to_string(),
            kind: ExternalServiceKind::Mcp,
            endpoint: "stdio:///usr/local/bin/memory-mcp".to_string(),
            secret_ref: Some(SecretRef::new("mcp-memory-key")),
            tenants_allowed: TenantSet::All,
            status: RuntimeStatus::Ok,
            archived_at: None,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn registered_event_shape_and_class() {
        let svc = sample_service();
        let ev = mcp_server_registered(AgentId::new(), &svc, None, Utc::now());
        assert_eq!(ev.event_type, "platform.mcp_server.registered");
        assert_eq!(ev.audit_class, AuditClass::Alerted);
        assert_eq!(ev.diff["before"], serde_json::Value::Null);
        assert_eq!(ev.diff["after"]["kind"], "mcp");
        assert_eq!(
            ev.diff["after"]["endpoint"],
            "stdio:///usr/local/bin/memory-mcp"
        );
        assert_eq!(ev.diff["after"]["secret_ref"], "mcp-memory-key");
    }

    #[test]
    fn archived_event_captures_before_snapshot() {
        let svc = sample_service();
        let ev = mcp_server_archived(AgentId::new(), &svc, None, Utc::now());
        assert_eq!(ev.event_type, "platform.mcp_server.archived");
        assert_eq!(ev.audit_class, AuditClass::Alerted);
        assert_eq!(ev.diff["before"]["kind"], "mcp");
        assert!(ev.diff["after"]["archived_at"].is_string());
    }

    #[test]
    fn tenant_access_revoked_reports_counts_and_lists() {
        let svc = sample_service();
        let org1 = OrgId::new();
        let org2 = OrgId::new();
        let ar1 = AuthRequestId::new();
        let ar2 = AuthRequestId::new();
        let revs = vec![
            TenantRevocation {
                org: org1,
                auth_request: ar1,
                revoked_grants: vec![crate::model::ids::GrantId::new()],
            },
            TenantRevocation {
                org: org2,
                auth_request: ar2,
                revoked_grants: vec![
                    crate::model::ids::GrantId::new(),
                    crate::model::ids::GrantId::new(),
                ],
            },
        ];
        let ev = mcp_server_tenant_access_revoked(
            AgentId::new(),
            &svc,
            &TenantSet::All,
            &revs,
            None,
            Utc::now(),
        );
        assert_eq!(ev.event_type, "platform.mcp_server.tenant_access_revoked");
        assert_eq!(ev.audit_class, AuditClass::Alerted);
        assert_eq!(ev.diff["after"]["revoked_org_count"], 2);
        assert_eq!(ev.diff["after"]["revoked_auth_request_count"], 2);
        assert_eq!(ev.diff["after"]["revoked_grant_count"], 3);
    }

    #[test]
    fn health_degraded_records_status_transition() {
        let svc = sample_service();
        let ev = mcp_server_health_degraded(
            AgentId::new(),
            &svc,
            RuntimeStatus::Ok,
            "timeout contacting MCP server",
            Utc::now(),
        );
        assert_eq!(ev.event_type, "platform.mcp_server.health_degraded");
        assert!(ev.diff["after"]["reason"]
            .as_str()
            .unwrap()
            .contains("timeout"));
    }

    #[test]
    fn auth_request_revoked_cascade_event_shape() {
        let mcp_id = McpServerId::new();
        let org = OrgId::new();
        let ar = AuthRequestId::new();
        let rev = TenantRevocation {
            org,
            auth_request: ar,
            revoked_grants: vec![crate::model::ids::GrantId::new()],
        };
        let ev = auth_request_revoked_by_mcp_cascade(
            AgentId::new(),
            mcp_id,
            org,
            &rev,
            None,
            Utc::now(),
        );
        assert_eq!(ev.event_type, "auth_request.revoked");
        assert_eq!(ev.audit_class, AuditClass::Alerted);
        assert_eq!(ev.diff["after"]["reason"], "mcp_tenant_narrow");
        assert_eq!(ev.diff["after"]["mcp_server_id"], mcp_id.to_string());
    }

    #[test]
    fn no_builder_sets_prev_event_hash() {
        let svc = sample_service();
        let now = Utc::now();
        for ev in [
            mcp_server_registered(AgentId::new(), &svc, None, now),
            mcp_server_archived(AgentId::new(), &svc, None, now),
            mcp_server_health_degraded(AgentId::new(), &svc, RuntimeStatus::Ok, "t", now),
        ] {
            assert!(
                ev.prev_event_hash.is_none(),
                "emitter fills prev_event_hash; builder leaves it `None`"
            );
        }
    }
}
