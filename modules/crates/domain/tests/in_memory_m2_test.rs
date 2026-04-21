//! Smoke tests for the M2 additions to `InMemoryRepository`.
//!
//! Covers the happy-path CRUD for each new surface so the trait impl
//! stays in lock-step with the SurrealDB-backed impl (C3 of the M2
//! plan's commitment ledger). Cascade + value-match deep tests live
//! in their own files.

// `in_memory` is gated behind `#[cfg(any(test, feature = "in-memory-repo"))]`;
// integration tests compile domain as an external crate so the `test`
// cfg doesn't apply. Gate this whole file behind the same feature and
// enable it in `domain/Cargo.toml`'s dev-deps.
#![cfg(feature = "in-memory-repo")]

use chrono::Utc;

use domain::in_memory::InMemoryRepository;
use domain::model::ids::{AgentId, McpServerId, ModelProviderId, OrgId, SecretId};
use domain::model::{
    Composite, ExternalService, ExternalServiceKind, ModelRuntime, PlatformDefaults, RuntimeStatus,
    SecretCredential, SecretRef, TenantSet,
};
use domain::repository::{Repository, RepositoryError, SealedBlob};

fn sample_sealed() -> SealedBlob {
    SealedBlob {
        ciphertext_b64: "Y2lwaGVydGV4dA".into(),
        nonce_b64: "bm9uY2U".into(),
    }
}

fn sample_credential() -> SecretCredential {
    SecretCredential {
        id: SecretId::new(),
        slug: SecretRef::new("anthropic-api-key"),
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

#[tokio::test]
async fn secret_put_get_list_round_trip() {
    let repo = InMemoryRepository::new();
    let cred = sample_credential();
    let sealed = sample_sealed();
    repo.put_secret(&cred, &sealed).await.unwrap();

    let back = repo.get_secret_by_slug(&cred.slug).await.unwrap().unwrap();
    assert_eq!(back.0.id, cred.id);
    assert_eq!(back.1, sealed);

    let list = repo.list_secrets().await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].slug, cred.slug);
}

#[tokio::test]
async fn secret_put_rejects_duplicate_slug() {
    let repo = InMemoryRepository::new();
    let cred = sample_credential();
    let sealed = sample_sealed();
    repo.put_secret(&cred, &sealed).await.unwrap();
    // Second row with the same slug (different id) must fail Conflict.
    let mut cred2 = sample_credential();
    cred2.slug = cred.slug.clone();
    let err = repo.put_secret(&cred2, &sealed).await.unwrap_err();
    matches!(err, RepositoryError::Conflict(_));
}

#[tokio::test]
async fn secret_rotate_updates_sealed_and_timestamp() {
    let repo = InMemoryRepository::new();
    let cred = sample_credential();
    repo.put_secret(&cred, &sample_sealed()).await.unwrap();
    let new_sealed = SealedBlob {
        ciphertext_b64: "cm90YXRlZA".into(),
        nonce_b64: "bmV3bm9uY2U".into(),
    };
    let at = Utc::now();
    repo.rotate_secret(cred.id, &new_sealed, at).await.unwrap();
    let back = repo.get_secret_by_slug(&cred.slug).await.unwrap().unwrap();
    assert_eq!(back.1, new_sealed);
    assert_eq!(back.0.last_rotated_at, Some(at));
}

#[tokio::test]
async fn secret_reassign_updates_custodian() {
    let repo = InMemoryRepository::new();
    let cred = sample_credential();
    repo.put_secret(&cred, &sample_sealed()).await.unwrap();
    let new_cust = AgentId::new();
    repo.reassign_secret_custodian(cred.id, new_cust)
        .await
        .unwrap();
    let back = repo.get_secret_by_slug(&cred.slug).await.unwrap().unwrap();
    assert_eq!(back.0.custodian, new_cust);
}

#[tokio::test]
async fn model_provider_put_list_archive() {
    let repo = InMemoryRepository::new();
    let rt = sample_model_runtime();
    repo.put_model_provider(&rt).await.unwrap();
    let list = repo.list_model_providers(false).await.unwrap();
    assert_eq!(list.len(), 1);

    repo.archive_model_provider(rt.id, Utc::now())
        .await
        .unwrap();
    let listed = repo.list_model_providers(false).await.unwrap();
    assert!(listed.is_empty(), "archived rows are hidden by default");
    let all = repo.list_model_providers(true).await.unwrap();
    assert_eq!(all.len(), 1);
    assert!(all[0].archived_at.is_some());
    assert_eq!(all[0].status, RuntimeStatus::Archived);
}

#[tokio::test]
async fn mcp_server_put_patch_archive() {
    let repo = InMemoryRepository::new();
    let s = sample_mcp_server();
    repo.put_mcp_server(&s).await.unwrap();

    let org = OrgId::new();
    repo.patch_mcp_tenants(s.id, &TenantSet::Only(vec![org]))
        .await
        .unwrap();
    let listed = repo.list_mcp_servers(false).await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].tenants_allowed, TenantSet::Only(vec![org]));

    repo.archive_mcp_server(s.id, Utc::now()).await.unwrap();
    assert!(repo.list_mcp_servers(false).await.unwrap().is_empty());
}

#[tokio::test]
async fn platform_defaults_singleton_round_trip() {
    let repo = InMemoryRepository::new();
    assert!(repo.get_platform_defaults().await.unwrap().is_none());
    let pd = sample_platform_defaults();
    repo.put_platform_defaults(&pd).await.unwrap();
    let back = repo.get_platform_defaults().await.unwrap().unwrap();
    assert_eq!(back.singleton, 1);
    assert_eq!(back.default_retention_days, 30);
    // Second PUT overwrites.
    let mut pd2 = pd;
    pd2.default_retention_days = 90;
    pd2.version = 1;
    repo.put_platform_defaults(&pd2).await.unwrap();
    let back2 = repo.get_platform_defaults().await.unwrap().unwrap();
    assert_eq!(back2.default_retention_days, 90);
    assert_eq!(back2.version, 1);
}

#[tokio::test]
async fn catalogue_entry_seeded_via_composite_helper() {
    let repo = InMemoryRepository::new();
    repo.seed_catalogue_entry_for_composite(
        None,
        "secret:anthropic-api-key",
        Composite::ExternalServiceObject,
    )
    .await
    .unwrap();
    // Confirm the underlying entry landed by querying via the base
    // method.
    assert!(repo
        .catalogue_contains(None, "secret:anthropic-api-key")
        .await
        .unwrap());
}
