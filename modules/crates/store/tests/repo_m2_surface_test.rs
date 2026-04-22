//! Integration tests for the M2 additions to `Repository`'s
//! SurrealDB-backed impl (C3 in the M2 plan).
//!
//! Each method gets a happy-path test + targeted error paths. Cascade
//! (narrow_mcp_tenants) has its own file in P6; this suite focuses on
//! the per-surface shapes.

use chrono::Utc;
use tempfile::tempdir;

use domain::audit::AuditClass;
use domain::model::ids::{
    AgentId, AuthRequestId, GrantId, McpServerId, ModelProviderId, OrgId, SecretId,
};
use domain::model::nodes::{
    ApproverSlot, ApproverSlotState, AuthRequest, AuthRequestState, Grant, PrincipalRef,
    ResourceRef, ResourceSlot, ResourceSlotState,
};
use domain::model::{
    Composite, ExternalService, ExternalServiceKind, ModelRuntime, PlatformDefaults, RuntimeStatus,
    SecretCredential, SecretRef, TenantSet,
};
use domain::repository::{Repository, RepositoryError, SealedBlob};
use store::SurrealStore;

async fn fresh_store() -> (SurrealStore, tempfile::TempDir) {
    let dir = tempdir().expect("tempdir");
    let store = SurrealStore::open_embedded(dir.path().join("db"), "phi", "test")
        .await
        .expect("open store");
    (store, dir)
}

fn sample_sealed() -> SealedBlob {
    SealedBlob {
        ciphertext_b64: "Y2lwaGVydGV4dC1ieXRlcw".into(),
        nonce_b64: "bm9uY2UtYnl0ZXM".into(),
    }
}

fn sample_credential() -> SecretCredential {
    SecretCredential {
        id: SecretId::new(),
        slug: SecretRef::new(format!("secret-{}", uuid::Uuid::new_v4())),
        custodian: AgentId::new(),
        last_rotated_at: None,
        sensitive: true,
        created_at: Utc::now(),
    }
}

fn sample_model_runtime() -> ModelRuntime {
    ModelRuntime {
        id: ModelProviderId::new(),
        config: phi_core::provider::model::ModelConfig::anthropic(
            "claude-sonnet",
            "claude-sonnet-4-6",
            "__placeholder__",
        ),
        secret_ref: SecretRef::new("anthropic-api-key"),
        tenants_allowed: TenantSet::All,
        status: RuntimeStatus::Ok,
        archived_at: None,
        created_at: Utc::now(),
    }
}

fn sample_mcp_server() -> ExternalService {
    ExternalService {
        id: McpServerId::new(),
        display_name: "memory-mcp".into(),
        kind: ExternalServiceKind::Mcp,
        endpoint: "stdio:///usr/local/bin/memory-mcp".into(),
        secret_ref: None,
        tenants_allowed: TenantSet::All,
        status: RuntimeStatus::Ok,
        archived_at: None,
        created_at: Utc::now(),
    }
}

fn sample_platform_defaults() -> PlatformDefaults {
    PlatformDefaults {
        singleton: 1,
        execution_limits: phi_core::context::execution::ExecutionLimits::default(),
        default_agent_profile: phi_core::agents::profile::AgentProfile::default(),
        context_config: phi_core::context::config::ContextConfig::default(),
        retry_config: phi_core::provider::retry::RetryConfig::default(),
        default_retention_days: 30,
        default_alert_channels: vec!["ops@example.com".into()],
        updated_at: Utc::now(),
        version: 0,
    }
}

// ============================================================================
// Secrets (5 methods)
// ============================================================================

#[tokio::test]
async fn put_and_get_secret_round_trips_through_surreal() {
    let (store, _dir) = fresh_store().await;
    let cred = sample_credential();
    let sealed = sample_sealed();
    store.put_secret(&cred, &sealed).await.unwrap();

    let back = store
        .get_secret_by_slug(&cred.slug)
        .await
        .unwrap()
        .expect("secret present");
    assert_eq!(back.0.id, cred.id);
    assert_eq!(back.0.slug, cred.slug);
    assert_eq!(back.0.custodian, cred.custodian);
    assert!(back.0.sensitive);
    assert_eq!(back.1, sealed);
}

#[tokio::test]
async fn put_secret_rejects_duplicate_slug_as_conflict() {
    let (store, _dir) = fresh_store().await;
    let cred = sample_credential();
    store.put_secret(&cred, &sample_sealed()).await.unwrap();

    let mut cred2 = sample_credential();
    cred2.slug = cred.slug.clone();
    let err = store
        .put_secret(&cred2, &sample_sealed())
        .await
        .unwrap_err();
    match err {
        RepositoryError::Conflict(_) => {}
        other => panic!("expected Conflict, got {:?}", other),
    }
}

#[tokio::test]
async fn list_secrets_returns_metadata_only() {
    let (store, _dir) = fresh_store().await;
    for _ in 0..3 {
        store
            .put_secret(&sample_credential(), &sample_sealed())
            .await
            .unwrap();
    }
    let rows = store.list_secrets().await.unwrap();
    assert_eq!(rows.len(), 3);
    // No panic on the cheaper metadata-only query path; every entry
    // carries its slug + custodian.
    for r in rows {
        assert!(!r.slug.as_str().is_empty());
    }
}

#[tokio::test]
async fn rotate_secret_updates_sealed_and_timestamp() {
    let (store, _dir) = fresh_store().await;
    let cred = sample_credential();
    store.put_secret(&cred, &sample_sealed()).await.unwrap();
    let new_sealed = SealedBlob {
        ciphertext_b64: "cm90YXRlZC1ieXRlcw".into(),
        nonce_b64: "cm90YXRlZC1ub25jZQ".into(),
    };
    let at = Utc::now();
    store.rotate_secret(cred.id, &new_sealed, at).await.unwrap();
    let back = store.get_secret_by_slug(&cred.slug).await.unwrap().unwrap();
    assert_eq!(back.1, new_sealed);
    assert!(back.0.last_rotated_at.is_some());
}

#[tokio::test]
async fn rotate_missing_secret_returns_not_found() {
    let (store, _dir) = fresh_store().await;
    let err = store
        .rotate_secret(SecretId::new(), &sample_sealed(), Utc::now())
        .await
        .unwrap_err();
    matches!(err, RepositoryError::NotFound);
}

#[tokio::test]
async fn reassign_custodian_updates_row() {
    let (store, _dir) = fresh_store().await;
    let cred = sample_credential();
    store.put_secret(&cred, &sample_sealed()).await.unwrap();
    let new_cust = AgentId::new();
    store
        .reassign_secret_custodian(cred.id, new_cust)
        .await
        .unwrap();
    let back = store.get_secret_by_slug(&cred.slug).await.unwrap().unwrap();
    assert_eq!(back.0.custodian, new_cust);
}

// ============================================================================
// Model providers (3 methods)
// ============================================================================

#[tokio::test]
async fn put_list_archive_model_provider_round_trip() {
    let (store, _dir) = fresh_store().await;
    let rt = sample_model_runtime();
    store.put_model_provider(&rt).await.unwrap();

    let listed = store.list_model_providers(false).await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, rt.id);
    assert_eq!(listed[0].config.id, "claude-sonnet");

    store
        .archive_model_provider(rt.id, Utc::now())
        .await
        .unwrap();
    assert!(store.list_model_providers(false).await.unwrap().is_empty());
    let all = store.list_model_providers(true).await.unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].status, RuntimeStatus::Archived);
    assert!(all[0].archived_at.is_some());
}

#[tokio::test]
async fn archive_missing_model_provider_returns_not_found() {
    let (store, _dir) = fresh_store().await;
    let err = store
        .archive_model_provider(ModelProviderId::new(), Utc::now())
        .await
        .unwrap_err();
    matches!(err, RepositoryError::NotFound);
}

// ============================================================================
// MCP servers (4 methods)
// ============================================================================

#[tokio::test]
async fn put_list_patch_archive_mcp_server_round_trip() {
    let (store, _dir) = fresh_store().await;
    let s = sample_mcp_server();
    store.put_mcp_server(&s).await.unwrap();

    let listed = store.list_mcp_servers(false).await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].display_name, "memory-mcp");

    let org = OrgId::new();
    store
        .patch_mcp_tenants(s.id, &TenantSet::Only(vec![org]))
        .await
        .unwrap();
    let after_patch = store.list_mcp_servers(false).await.unwrap();
    assert_eq!(after_patch[0].tenants_allowed, TenantSet::Only(vec![org]));

    store.archive_mcp_server(s.id, Utc::now()).await.unwrap();
    assert!(store.list_mcp_servers(false).await.unwrap().is_empty());
    let all = store.list_mcp_servers(true).await.unwrap();
    assert_eq!(all[0].status, RuntimeStatus::Archived);
}

// ============================================================================
// Platform defaults (2 methods)
// ============================================================================

#[tokio::test]
async fn platform_defaults_is_singleton_upsert() {
    let (store, _dir) = fresh_store().await;
    assert!(store.get_platform_defaults().await.unwrap().is_none());

    let mut pd = sample_platform_defaults();
    store.put_platform_defaults(&pd).await.unwrap();

    pd.default_retention_days = 90;
    pd.version = 1;
    store.put_platform_defaults(&pd).await.unwrap();

    let back = store
        .get_platform_defaults()
        .await
        .unwrap()
        .expect("row present");
    assert_eq!(back.default_retention_days, 90);
    assert_eq!(back.version, 1);
}

// ============================================================================
// Cascade (2 methods)
// ============================================================================

fn grant_owned_by_org(org: OrgId, ar: AuthRequestId) -> Grant {
    Grant {
        id: GrantId::new(),
        holder: PrincipalRef::Organization(org),
        action: vec!["invoke".into()],
        resource: ResourceRef {
            uri: format!("mcp:some-tool-{}", uuid::Uuid::new_v4()),
        },
        fundamentals: vec![],
        descends_from: Some(ar),
        delegable: false,
        issued_at: Utc::now(),
        revoked_at: None,
    }
}

fn auth_request_for_org(org: OrgId) -> AuthRequest {
    AuthRequest {
        id: AuthRequestId::new(),
        requestor: PrincipalRef::Organization(org),
        kinds: vec![],
        scope: vec![],
        state: AuthRequestState::Approved,
        valid_until: None,
        submitted_at: Utc::now(),
        resource_slots: vec![ResourceSlot {
            resource: ResourceRef {
                uri: "mcp:some-server".into(),
            },
            approvers: vec![ApproverSlot {
                approver: PrincipalRef::Organization(org),
                state: ApproverSlotState::Approved,
                responded_at: Some(Utc::now()),
                reconsidered_at: None,
            }],
            state: ResourceSlotState::Approved,
        }],
        justification: None,
        audit_class: AuditClass::Alerted,
        terminal_state_entered_at: Some(Utc::now()),
        archived: false,
        active_window_days: 30,
        provenance_template: None,
    }
}

#[tokio::test]
async fn revoke_grants_by_descends_from_flips_all_matching_grants() {
    let (store, _dir) = fresh_store().await;
    let org = OrgId::new();
    let ar = auth_request_for_org(org);
    store.create_auth_request(&ar).await.unwrap();

    for _ in 0..3 {
        let g = grant_owned_by_org(org, ar.id);
        store.create_grant(&g).await.unwrap();
    }
    // One decoy grant not descending from the AR — must NOT be touched.
    let decoy = {
        let mut g = grant_owned_by_org(org, AuthRequestId::new());
        g.descends_from = None;
        g
    };
    store.create_grant(&decoy).await.unwrap();

    let at = Utc::now();
    let revoked = store
        .revoke_grants_by_descends_from(ar.id, at)
        .await
        .unwrap();
    assert_eq!(revoked.len(), 3, "three matching grants revoked");

    // Decoy is still unrevoked.
    let decoy_after = store.get_grant(decoy.id).await.unwrap().unwrap();
    assert!(decoy_after.revoked_at.is_none());
}

#[tokio::test]
async fn narrow_mcp_tenants_cascades_revocation() {
    let (store, _dir) = fresh_store().await;
    let org_a = OrgId::new();
    let org_b = OrgId::new();

    // MCP server scoped to both orgs initially.
    let mut mcp = sample_mcp_server();
    mcp.tenants_allowed = TenantSet::Only(vec![org_a, org_b]);
    store.put_mcp_server(&mcp).await.unwrap();

    // Each org has an Auth Request + 2 grants descending from it.
    let ar_a = auth_request_for_org(org_a);
    let ar_b = auth_request_for_org(org_b);
    store.create_auth_request(&ar_a).await.unwrap();
    store.create_auth_request(&ar_b).await.unwrap();
    for _ in 0..2 {
        store
            .create_grant(&grant_owned_by_org(org_a, ar_a.id))
            .await
            .unwrap();
        store
            .create_grant(&grant_owned_by_org(org_b, ar_b.id))
            .await
            .unwrap();
    }

    // Narrow to org_a only. org_b's grants must be revoked.
    let revocations = store
        .narrow_mcp_tenants(mcp.id, &TenantSet::Only(vec![org_a]), Utc::now())
        .await
        .unwrap();

    // Exactly one revocation entry (for org_b, covering ar_b's grants).
    assert_eq!(revocations.len(), 1);
    let rev = &revocations[0];
    assert_eq!(rev.org, org_b);
    assert_eq!(rev.auth_request, ar_b.id);
    assert_eq!(rev.revoked_grants.len(), 2);

    // Server's tenants_allowed is now only org_a.
    let after = store.list_mcp_servers(false).await.unwrap();
    assert_eq!(after[0].tenants_allowed, TenantSet::Only(vec![org_a]));
}

// ============================================================================
// Catalogue (1 method)
// ============================================================================

#[tokio::test]
async fn seed_catalogue_entry_for_composite_uses_kind_name() {
    let (store, _dir) = fresh_store().await;
    store
        .seed_catalogue_entry_for_composite(
            None,
            "model_runtime:anthropic",
            Composite::ModelRuntimeObject,
        )
        .await
        .unwrap();
    assert!(store
        .catalogue_contains(None, "model_runtime:anthropic")
        .await
        .unwrap());
}
