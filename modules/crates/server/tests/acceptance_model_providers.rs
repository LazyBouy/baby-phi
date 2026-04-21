//! Acceptance tests for the M2/P5 model-providers HTTP surface.
//!
//! Real axum app + real embedded SurrealDB + real HTTP over the
//! loopback interface. Covers the 4 routes plus the phi-core
//! leverage contract (`provider-kinds` surface is driven by
//! phi-core's `ProviderRegistry::default()`, and `register` stores
//! the persisted config verbatim via phi-core's `ModelConfig` serde).
//!
//! Scenarios (plan §P5 + C9 verification):
//!
//! 1. Unauth — list returns 401.
//! 2. Provider-kinds enumerates phi-core's `ApiProtocol` variants.
//! 3. Register without a vault `secret_ref` → 400 SECRET_REF_NOT_FOUND.
//! 4. Fresh install register → list flow: row present, per-instance
//!    grant issued with ModelRuntimeObject fundamentals, catalogue
//!    seeded at `provider:<id>`, api_key scrubbed.
//! 5. Duplicate `(provider, model_id)` → 409 MODEL_PROVIDER_DUPLICATE.
//! 6. Archive → list with include_archived=false hides the row;
//!    include_archived=true shows it; audit event is Alerted.

mod acceptance_common;

use acceptance_common::admin::{spawn_claimed, ClaimedAdmin};
use base64::engine::general_purpose::STANDARD_NO_PAD as BASE64_NOPAD;
use base64::Engine as _;
use domain::audit::AuditClass;
use domain::Repository;
use serde_json::{json, Value};
use uuid::Uuid;

fn providers_url(admin: &ClaimedAdmin) -> String {
    admin.url("/api/v0/platform/model-providers")
}

async fn seed_vault_secret(admin: &ClaimedAdmin, slug: &str, material: &[u8]) {
    let url = admin.url("/api/v0/platform/secrets");
    let r = admin
        .authed_client
        .post(&url)
        .json(&json!({
            "slug": slug,
            "material_b64": BASE64_NOPAD.encode(material),
            "sensitive": true,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        r.status().as_u16(),
        201,
        "seed secret failed: {:?}",
        r.text().await
    );
}

fn sample_config(provider: &str, model_id: &str) -> Value {
    json!({
        "id": model_id,
        "name": "Test Model",
        "api": "anthropic_messages",
        "provider": provider,
        "base_url": "https://api.anthropic.com",
        "reasoning": false,
        "context_window": 200_000,
        "max_tokens": 8192,
    })
}

// ---- 1. Unauth -------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn unauthenticated_list_returns_401() {
    let admin = spawn_claimed(false).await;
    let r = admin
        .acc
        .client()
        .get(providers_url(&admin))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 401);
}

// ---- 2. provider-kinds is phi-core-driven ----------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn provider_kinds_enumerates_phi_core_protocols() {
    let admin = spawn_claimed(false).await;
    let r = admin
        .authed_client
        .get(admin.url("/api/v0/platform/provider-kinds"))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 200);
    let body: Value = r.json().await.unwrap();
    let kinds = body["kinds"].as_array().expect("kinds is array");
    assert!(!kinds.is_empty(), "phi-core must expose at least one kind");
    // Anthropic is shipped by phi-core's default registry — if this
    // drops, something upstream changed.
    assert!(
        kinds
            .iter()
            .any(|k| k.as_str() == Some("anthropic_messages")),
        "expected anthropic_messages in kinds: {kinds:?}"
    );
}

// ---- 3. Register without a seeded vault secret — 400 ----------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn register_without_seeded_secret_ref_fails() {
    let admin = spawn_claimed(false).await;
    let r = admin
        .authed_client
        .post(providers_url(&admin))
        .json(&json!({
            "config": sample_config("anthropic", "claude-sonnet-4"),
            "secret_ref": "nonexistent-key",
            "tenants_allowed": { "mode": "all" }
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 400);
    let body: Value = r.json().await.unwrap();
    assert_eq!(body["code"], "SECRET_REF_NOT_FOUND");
}

// ---- 4. Happy register + list — row, grant, catalogue, api_key scrub ------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn register_then_list_persists_phi_core_config_verbatim() {
    let admin = spawn_claimed(false).await;
    seed_vault_secret(&admin, "anthropic-api-key", b"sk-ant-x").await;

    let r = admin
        .authed_client
        .post(providers_url(&admin))
        .json(&json!({
            "config": sample_config("anthropic", "claude-sonnet-4-20250514"),
            "secret_ref": "anthropic-api-key",
            "tenants_allowed": { "mode": "all" }
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 201);
    let reg: Value = r.json().await.unwrap();
    let provider_id = reg["provider_id"].as_str().unwrap().to_string();
    let audit_event_id = reg["audit_event_id"].as_str().unwrap().to_string();

    // Audit event is Alerted + named `platform.model_provider.registered`.
    let event_id = audit_event_id.parse::<Uuid>().unwrap();
    let audit = admin
        .acc
        .store
        .get_audit_event(domain::model::ids::AuditEventId::from_uuid(event_id))
        .await
        .unwrap()
        .expect("registered audit event persisted");
    assert_eq!(audit.event_type, "platform.model_provider.registered");
    assert_eq!(audit.audit_class, AuditClass::Alerted);
    assert_eq!(audit.diff["after"]["provider_kind"], "anthropic_messages");
    assert_eq!(audit.diff["after"]["secret_ref"], "anthropic-api-key");

    // GET /model-providers — exactly one row, config round-trips,
    // api_key is the empty sentinel (plaintext never leaks back).
    let r = admin
        .authed_client
        .get(providers_url(&admin))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 200);
    let body: Value = r.json().await.unwrap();
    let rows = body["providers"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["id"].as_str().unwrap(), provider_id);
    assert_eq!(rows[0]["config"]["id"], "claude-sonnet-4-20250514");
    assert_eq!(rows[0]["config"]["api"], "anthropic_messages");
    assert_eq!(
        rows[0]["config"]["api_key"].as_str().unwrap_or(""),
        "",
        "api_key must be scrubbed; plaintext never surfaces via list"
    );

    // P4.5 grant shape: per-instance URI `provider:<id>` + explicit
    // ModelRuntimeObject fundamentals.
    let admin_agent_id =
        domain::model::ids::AgentId::from_uuid(Uuid::parse_str(&admin.agent_id).unwrap());
    let grants = admin
        .acc
        .store
        .list_grants_for_principal(&domain::model::nodes::PrincipalRef::Agent(admin_agent_id))
        .await
        .unwrap();
    let provider_grants: Vec<_> = grants
        .iter()
        .filter(|g| g.resource.uri.starts_with("provider:"))
        .collect();
    assert_eq!(
        provider_grants.len(),
        1,
        "expected exactly one `provider:<id>` grant"
    );
    assert_eq!(
        provider_grants[0].resource.uri,
        format!("provider:{provider_id}")
    );
    use domain::model::Fundamental::*;
    let fs: std::collections::HashSet<_> =
        provider_grants[0].fundamentals.iter().copied().collect();
    assert!(fs.contains(&NetworkEndpoint));
    assert!(fs.contains(&SecretCredential));
    assert!(fs.contains(&EconomicResource));
    assert!(fs.contains(&Tag));
}

// ---- 5. Duplicate register returns 409 ------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn duplicate_provider_model_pair_returns_409() {
    let admin = spawn_claimed(false).await;
    seed_vault_secret(&admin, "anthropic-api-key", b"sk-ant-x").await;

    let body = json!({
        "config": sample_config("anthropic", "claude-sonnet-4"),
        "secret_ref": "anthropic-api-key",
        "tenants_allowed": { "mode": "all" }
    });
    let r = admin
        .authed_client
        .post(providers_url(&admin))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 201);

    let r = admin
        .authed_client
        .post(providers_url(&admin))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 409);
    let err: Value = r.json().await.unwrap();
    assert_eq!(err["code"], "MODEL_PROVIDER_DUPLICATE");
}

// ---- 6. Archive flow ------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn archive_hides_row_unless_include_archived() {
    let admin = spawn_claimed(false).await;
    seed_vault_secret(&admin, "anthropic-api-key", b"sk-ant-x").await;

    let r = admin
        .authed_client
        .post(providers_url(&admin))
        .json(&json!({
            "config": sample_config("anthropic", "claude-sonnet-4"),
            "secret_ref": "anthropic-api-key",
            "tenants_allowed": { "mode": "all" }
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 201);
    let reg: Value = r.json().await.unwrap();
    let provider_id = reg["provider_id"].as_str().unwrap().to_string();

    // Archive.
    let r = admin
        .authed_client
        .post(format!("{}/{provider_id}/archive", providers_url(&admin)))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 200);
    let arc: Value = r.json().await.unwrap();
    let archived_audit_event = arc["audit_event_id"].as_str().unwrap().to_string();
    let event_id = archived_audit_event.parse::<Uuid>().unwrap();
    let audit = admin
        .acc
        .store
        .get_audit_event(domain::model::ids::AuditEventId::from_uuid(event_id))
        .await
        .unwrap()
        .expect("archived audit event persisted");
    assert_eq!(audit.event_type, "platform.model_provider.archived");
    assert_eq!(audit.audit_class, AuditClass::Alerted);

    // Default list filters archived rows.
    let r = admin
        .authed_client
        .get(providers_url(&admin))
        .send()
        .await
        .unwrap();
    let body: Value = r.json().await.unwrap();
    assert_eq!(body["providers"].as_array().unwrap().len(), 0);

    // include_archived=true shows it.
    let r = admin
        .authed_client
        .get(format!("{}?include_archived=true", providers_url(&admin)))
        .send()
        .await
        .unwrap();
    let body: Value = r.json().await.unwrap();
    let rows = body["providers"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert!(rows[0]["archived_at"].is_string());
}

// ---- 7. Archive on unknown id — 404 ---------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn archive_unknown_id_returns_404() {
    let admin = spawn_claimed(false).await;
    let fake = Uuid::new_v4();
    let r = admin
        .authed_client
        .post(format!("{}/{fake}/archive", providers_url(&admin)))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 404);
    let err: Value = r.json().await.unwrap();
    assert_eq!(err["code"], "MODEL_PROVIDER_NOT_FOUND");
}
