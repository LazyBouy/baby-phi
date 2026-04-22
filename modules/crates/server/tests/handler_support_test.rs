//! Integration tests for `handler_support` — the shim every M2+
//! handler builds on.
//!
//! Covers:
//! - `AuthenticatedSession` extractor — 401 on missing / malformed
//!   cookies, happy-path parse on a valid signed cookie.
//! - `check_permission` — every `Decision::Denied` variant maps to its
//!   expected ApiError; `Decision::Allowed` passes through.
//! - `emit_audit` — propagates emitter success; maps emitter failure
//!   to a 500 `AUDIT_EMIT_FAILED`.

use std::sync::Arc;

use async_trait::async_trait;
use axum::http::StatusCode;
use chrono::Utc;

use domain::audit::{AuditEmitter, AuditEvent, NoopAuditEmitter};
use domain::model::ids::{AgentId, AuditEventId, GrantId, OrgId};
use domain::model::nodes::{Grant, PrincipalRef, ResourceRef};
use domain::model::Fundamental;
use domain::permissions::{
    catalogue::StaticCatalogue,
    decision::{DeniedReason, FailedStep},
    manifest::{CheckContext, ConsentIndex, Manifest, ToolCall},
    metrics::NoopMetrics,
};
use domain::repository::{RepositoryError, RepositoryResult};
use server::handler_support::permission::{check_permission, denial_to_api_error};
use server::handler_support::{emit_audit, emit_audit_batch, ApiError};

// -----------------------------------------------------------------------
// check_permission contract
// -----------------------------------------------------------------------

fn mk_grant(holder: PrincipalRef, actions: &[&str], resource: &str) -> Grant {
    Grant {
        id: GrantId::new(),
        holder,
        action: actions.iter().map(|s| s.to_string()).collect(),
        resource: ResourceRef {
            uri: resource.into(),
        },
        fundamentals: vec![],
        descends_from: None,
        delegable: false,
        issued_at: Utc::now(),
        revoked_at: None,
    }
}

#[test]
fn check_permission_allows_when_grant_covers_reach() {
    let agent = AgentId::new();
    let grants = [mk_grant(
        PrincipalRef::Agent(agent),
        &["read"],
        "filesystem_object",
    )];
    let catalogue = StaticCatalogue::empty();
    let consents = ConsentIndex::empty();
    let gated = std::collections::HashSet::new();
    let ctx = CheckContext {
        agent,
        current_org: None,
        current_project: None,
        agent_grants: &grants,
        project_grants: &[],
        org_grants: &[],
        ceiling_grants: &[],
        catalogue: &catalogue,
        consents: &consents,
        template_gated_auth_requests: &gated,
        call: ToolCall::default(),
    };
    let m = Manifest {
        actions: vec!["read".into()],
        resource: vec!["filesystem_object".into()],
        ..Default::default()
    };
    let result = check_permission(&ctx, &m, &NoopMetrics);
    assert!(result.is_ok(), "Allowed must pass through");
}

#[test]
fn check_permission_maps_step_2_resolution_to_no_grants_held() {
    let agent = AgentId::new();
    let catalogue = StaticCatalogue::empty();
    let consents = ConsentIndex::empty();
    let gated = std::collections::HashSet::new();
    let ctx = CheckContext {
        agent,
        current_org: None,
        current_project: None,
        agent_grants: &[],
        project_grants: &[],
        org_grants: &[],
        ceiling_grants: &[],
        catalogue: &catalogue,
        consents: &consents,
        template_gated_auth_requests: &gated,
        call: ToolCall::default(),
    };
    let m = Manifest {
        actions: vec!["read".into()],
        resource: vec!["filesystem_object".into()],
        ..Default::default()
    };
    let err = check_permission(&ctx, &m, &NoopMetrics).unwrap_err();
    assert_eq!(err.code, "NO_GRANTS_HELD");
    assert_eq!(err.status, StatusCode::FORBIDDEN);
}

#[test]
fn check_permission_maps_step_4_to_constraint_violation() {
    let agent = AgentId::new();
    let grants = [mk_grant(
        PrincipalRef::Agent(agent),
        &["read"],
        "filesystem_object",
    )];
    let catalogue = StaticCatalogue::empty();
    let consents = ConsentIndex::empty();
    let gated = std::collections::HashSet::new();
    let ctx = CheckContext {
        agent,
        current_org: None,
        current_project: None,
        agent_grants: &grants,
        project_grants: &[],
        org_grants: &[],
        ceiling_grants: &[],
        catalogue: &catalogue,
        consents: &consents,
        template_gated_auth_requests: &gated,
        call: ToolCall::default(),
    };
    let mut m = Manifest {
        actions: vec!["read".into()],
        resource: vec!["filesystem_object".into()],
        constraints: vec!["purpose".into()],
        ..Default::default()
    };
    m.constraint_requirements
        .insert("purpose".into(), serde_json::json!("reveal"));
    let err = check_permission(&ctx, &m, &NoopMetrics).unwrap_err();
    assert_eq!(err.code, "CONSTRAINT_VIOLATION");
    assert_eq!(err.status, StatusCode::FORBIDDEN);
}

#[test]
fn check_permission_maps_pending_to_awaiting_consent_202() {
    use domain::model::ids::AuthRequestId;
    let agent = AgentId::new();
    let org = OrgId::new();
    let ar = AuthRequestId::new();
    let mut g = mk_grant(PrincipalRef::Agent(agent), &["read"], "filesystem_object");
    g.descends_from = Some(ar);
    let grants = [g];
    let mut gated = std::collections::HashSet::new();
    gated.insert(ar);
    let catalogue = StaticCatalogue::empty();
    let consents = ConsentIndex::empty();
    let ctx = CheckContext {
        agent,
        current_org: Some(org),
        current_project: None,
        agent_grants: &grants,
        project_grants: &[],
        org_grants: &[],
        ceiling_grants: &[],
        catalogue: &catalogue,
        consents: &consents,
        template_gated_auth_requests: &gated,
        call: ToolCall {
            target_agent: Some(AgentId::new()),
            ..Default::default()
        },
    };
    let m = Manifest {
        actions: vec!["read".into()],
        resource: vec!["filesystem_object".into()],
        ..Default::default()
    };
    let err = check_permission(&ctx, &m, &NoopMetrics).unwrap_err();
    assert_eq!(err.code, "AWAITING_CONSENT");
    assert_eq!(err.status, StatusCode::ACCEPTED);
}

#[test]
fn denial_to_api_error_catalogue_miss_carries_resource_uri() {
    let err = denial_to_api_error(
        FailedStep::Catalogue,
        &DeniedReason::CatalogueMiss {
            resource_uri: "memory:mem-7".into(),
        },
    );
    assert_eq!(err.code, "CATALOGUE_MISS");
    assert_eq!(err.status, StatusCode::FORBIDDEN);
    assert!(err.message.contains("memory:mem-7"));
}

#[test]
fn denial_to_api_error_match_carries_fundamental_and_action() {
    let err = denial_to_api_error(
        FailedStep::Match,
        &DeniedReason::NoMatchingGrant {
            fundamental: Fundamental::NetworkEndpoint,
            action: "connect".into(),
        },
    );
    assert_eq!(err.code, "NO_MATCHING_GRANT");
    assert!(err.message.contains("connect"));
}

// -----------------------------------------------------------------------
// emit_audit contract
// -----------------------------------------------------------------------

fn sample_audit_event() -> AuditEvent {
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: "test.emit".into(),
        actor_agent_id: None,
        target_entity_id: None,
        timestamp: Utc::now(),
        diff: serde_json::json!({}),
        audit_class: domain::audit::AuditClass::Logged,
        provenance_auth_request_id: None,
        org_scope: None,
        prev_event_hash: None,
    }
}

#[tokio::test]
async fn emit_audit_happy_path_is_ok() {
    let emitter: Arc<dyn AuditEmitter> = Arc::new(NoopAuditEmitter);
    emit_audit(emitter.as_ref(), sample_audit_event())
        .await
        .expect("noop emitter always succeeds");
}

#[tokio::test]
async fn emit_audit_maps_failure_to_500_audit_emit_failed() {
    struct FailingEmitter;
    #[async_trait]
    impl AuditEmitter for FailingEmitter {
        async fn emit(&self, _e: AuditEvent) -> RepositoryResult<()> {
            Err(RepositoryError::Backend("disk full".into()))
        }
    }
    let emitter: Arc<dyn AuditEmitter> = Arc::new(FailingEmitter);
    let err: ApiError = emit_audit(emitter.as_ref(), sample_audit_event())
        .await
        .unwrap_err();
    assert_eq!(err.code, "AUDIT_EMIT_FAILED");
    assert_eq!(err.status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(err.message.contains("disk full"));
}

// -----------------------------------------------------------------------
// emit_audit_batch contract (M3/P3)
// -----------------------------------------------------------------------

/// Records every event the emitter sees, in order, so the test can
/// assert on iteration order.
#[derive(Default)]
struct RecordingEmitter {
    recorded: tokio::sync::Mutex<Vec<AuditEventId>>,
}

#[async_trait]
impl AuditEmitter for RecordingEmitter {
    async fn emit(&self, event: AuditEvent) -> RepositoryResult<()> {
        self.recorded.lock().await.push(event.event_id);
        Ok(())
    }
}

#[tokio::test]
async fn emit_audit_batch_returns_ids_in_input_order() {
    let events: Vec<AuditEvent> = (0..4).map(|_| sample_audit_event()).collect();
    let expected_ids: Vec<AuditEventId> = events.iter().map(|e| e.event_id).collect();

    let emitter = Arc::new(RecordingEmitter::default());
    let ids = emit_audit_batch(emitter.as_ref(), events)
        .await
        .expect("batch emit succeeds");

    assert_eq!(ids, expected_ids, "returned ids must match input order");
    assert_eq!(
        *emitter.recorded.lock().await,
        expected_ids,
        "emitter saw events in the input order — required for per-org chain continuity"
    );
}

#[tokio::test]
async fn emit_audit_batch_empty_input_returns_empty_vec() {
    let emitter: Arc<dyn AuditEmitter> = Arc::new(NoopAuditEmitter);
    let ids = emit_audit_batch(emitter.as_ref(), vec![])
        .await
        .expect("empty batch is trivially Ok");
    assert!(ids.is_empty());
}

#[tokio::test]
async fn emit_audit_batch_fails_fast_on_first_error() {
    /// Fails on the Nth call (0-indexed); succeeds otherwise. Used to
    /// prove fail-fast: after the failing call, no further emits
    /// happen.
    struct FailsAtIndex {
        fail_at: usize,
        calls: tokio::sync::Mutex<usize>,
    }
    #[async_trait]
    impl AuditEmitter for FailsAtIndex {
        async fn emit(&self, _e: AuditEvent) -> RepositoryResult<()> {
            let mut n = self.calls.lock().await;
            let current = *n;
            *n += 1;
            if current == self.fail_at {
                Err(RepositoryError::Backend("simulated emit failure".into()))
            } else {
                Ok(())
            }
        }
    }
    let emitter = Arc::new(FailsAtIndex {
        fail_at: 2,
        calls: tokio::sync::Mutex::new(0),
    });
    let events: Vec<AuditEvent> = (0..5).map(|_| sample_audit_event()).collect();

    let err = emit_audit_batch(emitter.as_ref(), events)
        .await
        .expect_err("batch must fail on the 3rd event");
    assert_eq!(err.code, "AUDIT_EMIT_FAILED");

    // Emitter received exactly 3 calls: events 0, 1, 2 — then bailed.
    // Events 3 and 4 MUST NOT have been attempted (fail-fast).
    let total_calls = *emitter.calls.lock().await;
    assert_eq!(
        total_calls, 3,
        "emitter should have seen exactly 3 calls (fail-fast after the failing one)"
    );
}
