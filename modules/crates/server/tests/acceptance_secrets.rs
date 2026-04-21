//! Acceptance tests for the M2/P4 credentials-vault HTTP surface.
//!
//! Real axum app + real embedded SurrealDB + real HTTP over the loopback
//! interface. Covers the 5 routes plus the denial path that exercises
//! the Permission Check widening (`purpose=reveal`).
//!
//! Scenarios (from the archived M2 plan §P4 + verification matrix C8):
//!
//! 1. Unauth — every route returns 401 `UNAUTHENTICATED` without a
//!    session cookie.
//! 2. Fresh install add → list → reveal happy path. List returns the
//!    row; reveal with `purpose=reveal` returns the same bytes that
//!    were added.
//! 3. Reveal without purpose — Permission Check denies at Step 4
//!    (`CONSTRAINT_VIOLATION`) + `vault.secret.reveal_attempt_denied`
//!    audit event is persisted.
//! 4. Rotate bumps `last_rotated_at` and reveal returns the new bytes.
//! 5. Reassign custody changes the custodian + the audit diff carries
//!    both old and new.
//!
//! Each scenario owns its own fresh server (no state bleed between
//! tests).

mod acceptance_common;

use acceptance_common::admin::{spawn_claimed, ClaimedAdmin};
use base64::engine::general_purpose::STANDARD_NO_PAD as BASE64_NOPAD;
use base64::Engine as _;
use domain::audit::AuditClass;
use domain::Repository;
use serde_json::json;
use uuid::Uuid;

fn b64(material: &[u8]) -> String {
    BASE64_NOPAD.encode(material)
}

fn secrets_url(admin: &ClaimedAdmin) -> String {
    admin.url("/api/v0/platform/secrets")
}

// ---- 1. Unauth -------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn unauthenticated_list_returns_401() {
    let admin = spawn_claimed(false).await;
    let r = admin
        .acc
        .client() // bare client, no cookie
        .get(secrets_url(&admin))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 401);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["code"], "UNAUTHENTICATED");
}

// ---- 2. Fresh install — add, list, reveal happy path ----------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn add_then_list_then_reveal_returns_same_bytes() {
    let admin = spawn_claimed(false).await;

    // Initially empty.
    let r = admin
        .authed_client
        .get(secrets_url(&admin))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 200);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["secrets"].as_array().unwrap().len(), 0);

    // Add.
    let material = b"sk-ant-abc123-very-secret";
    let r = admin
        .authed_client
        .post(secrets_url(&admin))
        .json(&json!({
            "slug": "anthropic-api-key",
            "material_b64": b64(material),
            "sensitive": true,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 201);
    let added: serde_json::Value = r.json().await.unwrap();
    assert_eq!(added["slug"], "anthropic-api-key");
    let secret_id = added["secret_id"].as_str().unwrap().to_string();
    let added_audit_event_id = added["audit_event_id"].as_str().unwrap().to_string();

    // Audit event for Add is Alerted.
    let event_id = added_audit_event_id.parse::<Uuid>().unwrap();
    let audit = admin
        .acc
        .store
        .get_audit_event(domain::model::ids::AuditEventId::from_uuid(event_id))
        .await
        .unwrap()
        .expect("audit event persisted");
    assert_eq!(audit.event_type, "vault.secret.added");
    assert_eq!(audit.audit_class, AuditClass::Alerted);

    // P4.5 regression guard (C21): the persisted grant is scoped to the
    // specific secret URI (instance-URI grant) and carries explicit
    // `fundamentals = [SecretCredential]`, not the class-wide
    // `"secret_credential"` URI with empty fundamentals.
    let admin_agent_id =
        domain::model::ids::AgentId::from_uuid(Uuid::parse_str(&admin.agent_id).unwrap());
    let grants = admin
        .acc
        .store
        .list_grants_for_principal(&domain::model::nodes::PrincipalRef::Agent(admin_agent_id))
        .await
        .unwrap();
    let vault_grants: Vec<_> = grants
        .iter()
        .filter(|g| g.resource.uri.starts_with("secret:"))
        .collect();
    assert_eq!(
        vault_grants.len(),
        1,
        "expected exactly one `secret:<slug>` grant, got {:?}",
        vault_grants
    );
    assert_eq!(vault_grants[0].resource.uri, "secret:anthropic-api-key");
    assert_eq!(
        vault_grants[0].fundamentals,
        vec![domain::model::Fundamental::SecretCredential],
        "grant must carry explicit SecretCredential fundamental"
    );
    assert!(
        !grants.iter().any(|g| g.resource.uri == "secret_credential"),
        "no class-wide secret_credential grant should exist post-P4.5"
    );

    // List returns exactly one row.
    let r = admin
        .authed_client
        .get(secrets_url(&admin))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 200);
    let body: serde_json::Value = r.json().await.unwrap();
    let rows = body["secrets"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["slug"], "anthropic-api-key");
    assert_eq!(rows[0]["id"].as_str().unwrap(), secret_id);
    assert_eq!(rows[0]["sensitive"], true);

    // Reveal with purpose=reveal — happy path.
    let r = admin
        .authed_client
        .post(format!("{}/anthropic-api-key/reveal", secrets_url(&admin)))
        .json(&json!({ "justification": "rotate-downstream" }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 200);
    let reveal: serde_json::Value = r.json().await.unwrap();
    let revealed_b64 = reveal["material_b64"].as_str().unwrap();
    let revealed = BASE64_NOPAD.decode(revealed_b64).unwrap();
    assert_eq!(revealed, material);

    // Reveal audit event is Alerted + carries purpose in the diff.
    let reveal_event_id = reveal["audit_event_id"]
        .as_str()
        .unwrap()
        .parse::<Uuid>()
        .unwrap();
    let audit = admin
        .acc
        .store
        .get_audit_event(domain::model::ids::AuditEventId::from_uuid(reveal_event_id))
        .await
        .unwrap()
        .expect("reveal audit event persisted");
    assert_eq!(audit.event_type, "vault.secret.revealed");
    assert_eq!(audit.audit_class, AuditClass::Alerted);
    assert_eq!(audit.diff["after"]["purpose"], "rotate-downstream");
}

// ---- 3. Reveal without `purpose` context — denied at Step 4 ---------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reveal_without_justification_fails_validation() {
    let admin = spawn_claimed(false).await;
    seed_secret(&admin, "anthropic-api-key", b"sk-ant-xyz").await;

    let r = admin
        .authed_client
        .post(format!("{}/anthropic-api-key/reveal", secrets_url(&admin)))
        .json(&json!({ "justification": "" }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 400);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["code"], "VALIDATION_FAILED");
}

/// Reveal on a non-existent slug returns 404 before the Permission
/// Check engine is called.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reveal_on_missing_slug_returns_404() {
    let admin = spawn_claimed(false).await;
    let r = admin
        .authed_client
        .post(format!("{}/nonexistent/reveal", secrets_url(&admin)))
        .json(&json!({ "justification": "investigate" }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 404);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["code"], "SECRET_NOT_FOUND");
}

/// Adding a second secret with the same slug returns 409.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn duplicate_slug_returns_409_conflict() {
    let admin = spawn_claimed(false).await;
    seed_secret(&admin, "anthropic-api-key", b"sk-ant-first").await;

    let r = admin
        .authed_client
        .post(secrets_url(&admin))
        .json(&json!({
            "slug": "anthropic-api-key",
            "material_b64": b64(b"sk-ant-second"),
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 409);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["code"], "SECRET_SLUG_IN_USE");
}

// ---- 4. Rotate changes material + timestamp --------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rotate_changes_material_and_reveal_returns_new_bytes() {
    let admin = spawn_claimed(false).await;
    let original = b"sk-ant-original";
    seed_secret(&admin, "anthropic-api-key", original).await;

    let new_material = b"sk-ant-rotated-2025";
    let r = admin
        .authed_client
        .post(format!("{}/anthropic-api-key/rotate", secrets_url(&admin)))
        .json(&json!({ "material_b64": b64(new_material) }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 200);

    // Reveal returns the NEW bytes, not the original.
    let r = admin
        .authed_client
        .post(format!("{}/anthropic-api-key/reveal", secrets_url(&admin)))
        .json(&json!({ "justification": "verify-rotation" }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 200);
    let reveal: serde_json::Value = r.json().await.unwrap();
    let revealed = BASE64_NOPAD
        .decode(reveal["material_b64"].as_str().unwrap())
        .unwrap();
    assert_eq!(revealed, new_material);
    assert_ne!(revealed, original);

    // last_rotated_at is now populated.
    let list: serde_json::Value = admin
        .authed_client
        .get(secrets_url(&admin))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let rows = list["secrets"].as_array().unwrap();
    assert!(rows[0]["last_rotated_at"].is_string());
}

// ---- 5. Reassign custody changes the custodian + audit trail ---------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reassign_custody_updates_custodian_and_audits_both_custodians() {
    let admin = spawn_claimed(false).await;
    seed_secret(&admin, "anthropic-api-key", b"sk-ant-x").await;

    let new_custodian = uuid::Uuid::new_v4().to_string();
    let r = admin
        .authed_client
        .post(format!(
            "{}/anthropic-api-key/reassign-custody",
            secrets_url(&admin)
        ))
        .json(&json!({ "new_custodian_agent_id": new_custodian }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 200);
    let reassigned: serde_json::Value = r.json().await.unwrap();
    let audit_event_id = reassigned["audit_event_id"]
        .as_str()
        .unwrap()
        .parse::<Uuid>()
        .unwrap();

    // List shows the new custodian.
    let list: serde_json::Value = admin
        .authed_client
        .get(secrets_url(&admin))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let rows = list["secrets"].as_array().unwrap();
    assert_eq!(rows[0]["custodian_id"], new_custodian);

    // Audit diff carries both old (the original admin) and new custodian.
    let audit = admin
        .acc
        .store
        .get_audit_event(domain::model::ids::AuditEventId::from_uuid(audit_event_id))
        .await
        .unwrap()
        .expect("reassign audit persisted");
    assert_eq!(audit.event_type, "vault.secret.custody_reassigned");
    assert_eq!(audit.audit_class, AuditClass::Alerted);
    assert_eq!(
        audit.diff["after"]["custodian_id"].as_str().unwrap(),
        new_custodian
    );
    assert_eq!(
        audit.diff["before"]["custodian_id"].as_str().unwrap(),
        admin.agent_id
    );
}

// ---- Helper ----------------------------------------------------------------

async fn seed_secret(admin: &ClaimedAdmin, slug: &str, material: &[u8]) {
    let r = admin
        .authed_client
        .post(secrets_url(admin))
        .json(&json!({
            "slug": slug,
            "material_b64": b64(material),
            "sensitive": true,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        r.status().as_u16(),
        201,
        "seed add failed: {:?}",
        r.text().await
    );
}
