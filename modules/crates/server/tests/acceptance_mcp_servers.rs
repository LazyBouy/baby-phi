//! Acceptance tests for the M2/P6 MCP-servers HTTP surface.
//!
//! Real axum app + real embedded SurrealDB + real HTTP over the
//! loopback interface. Covers the 4 routes plus the cascade contract
//! (tenant-narrowing PATCHes emit a summary event + per-AR
//! `auth_request.revoked` events, and live grants drop monotonically).
//!
//! Scenarios (plan §P6 + C10 verification):
//!
//! 1. Unauth — list returns 401.
//! 2. Register without a seeded `secret_ref` (when supplied) → 400
//!    SECRET_REF_NOT_FOUND.
//! 3. Fresh install register → list flow: row present, per-instance
//!    grant issued with ExternalServiceObject fundamentals, catalogue
//!    seeded at `external_service:<id>`.
//! 4. Archive → list filter + Alerted audit event.
//! 5. Archive on unknown id — 404.
//! 6. Register with no `secret_ref` succeeds (unauth MCP services).
//! 7. PATCH `tenants_allowed` with no-op (same set) → empty cascade,
//!    no audit event.
//! 8. PATCH `tenants_allowed` that widens → no cascade; Template-E AR
//!    is created.
//! 9. End-to-end cascade: register with `Only([a, b])` + seed an AR/grant
//!    requested by `a`, then PATCH to `Only([b])` → cascade revokes
//!    org `a`'s grant and emits one `tenant_access_revoked` summary +
//!    one `auth_request.revoked` per affected AR.

mod acceptance_common;

use acceptance_common::admin::{spawn_claimed, ClaimedAdmin};
use base64::engine::general_purpose::STANDARD_NO_PAD as BASE64_NOPAD;
use base64::Engine as _;
use chrono::Utc;
use domain::audit::AuditClass;
use domain::model::ids::{AuthRequestId, GrantId, OrgId};
use domain::model::nodes::{
    AuthRequest, AuthRequestState, Grant, PrincipalRef, ResourceRef, ResourceSlot,
    ResourceSlotState,
};
use domain::model::Fundamental;
use domain::Repository;
use serde_json::{json, Value};
use uuid::Uuid;

fn mcp_url(admin: &ClaimedAdmin) -> String {
    admin.url("/api/v0/platform/mcp-servers")
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

// ---- 1. Unauth ------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn unauthenticated_list_returns_401() {
    let admin = spawn_claimed(false).await;
    let r = admin
        .acc
        .client()
        .get(mcp_url(&admin))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 401);
}

// ---- 2. Register with missing secret_ref — 400 -----------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn register_with_missing_secret_ref_returns_400() {
    let admin = spawn_claimed(false).await;
    let r = admin
        .authed_client
        .post(mcp_url(&admin))
        .json(&json!({
            "display_name": "memory-mcp",
            "kind": "mcp",
            "endpoint": "stdio:///usr/local/bin/memory-mcp",
            "secret_ref": "missing-key",
            "tenants_allowed": { "mode": "all" }
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 400);
    let body: Value = r.json().await.unwrap();
    assert_eq!(body["code"], "SECRET_REF_NOT_FOUND");
}

// ---- 3. Happy register + grant / catalogue / audit check ------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn register_then_list_creates_grant_with_external_service_fundamentals() {
    let admin = spawn_claimed(false).await;
    seed_vault_secret(&admin, "mcp-memory-key", b"secret-material").await;

    let r = admin
        .authed_client
        .post(mcp_url(&admin))
        .json(&json!({
            "display_name": "memory-mcp",
            "kind": "mcp",
            "endpoint": "stdio:///usr/local/bin/memory-mcp",
            "secret_ref": "mcp-memory-key",
            "tenants_allowed": { "mode": "all" }
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 201);
    let reg: Value = r.json().await.unwrap();
    let mcp_server_id = reg["mcp_server_id"].as_str().unwrap().to_string();
    let audit_event_id = reg["audit_event_id"].as_str().unwrap().to_string();

    // Audit event is Alerted + named `platform.mcp_server.registered`.
    let event_id = audit_event_id.parse::<Uuid>().unwrap();
    let audit = admin
        .acc
        .store
        .get_audit_event(domain::model::ids::AuditEventId::from_uuid(event_id))
        .await
        .unwrap()
        .expect("registered audit event persisted");
    assert_eq!(audit.event_type, "platform.mcp_server.registered");
    assert_eq!(audit.audit_class, AuditClass::Alerted);
    assert_eq!(audit.diff["after"]["kind"], "mcp");
    assert_eq!(audit.diff["after"]["secret_ref"], "mcp-memory-key");

    // GET /mcp-servers — exactly one row.
    let r = admin
        .authed_client
        .get(mcp_url(&admin))
        .send()
        .await
        .unwrap();
    let body: Value = r.json().await.unwrap();
    let rows = body["servers"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["id"].as_str().unwrap(), mcp_server_id);
    assert_eq!(rows[0]["display_name"], "memory-mcp");
    assert_eq!(rows[0]["kind"], "mcp");
    assert_eq!(rows[0]["endpoint"], "stdio:///usr/local/bin/memory-mcp");

    // P4.5 grant shape: per-instance URI `external_service:<id>` with
    // ExternalServiceObject fundamentals.
    let admin_agent_id =
        domain::model::ids::AgentId::from_uuid(Uuid::parse_str(&admin.agent_id).unwrap());
    let grants = admin
        .acc
        .store
        .list_grants_for_principal(&PrincipalRef::Agent(admin_agent_id))
        .await
        .unwrap();
    let mcp_grants: Vec<_> = grants
        .iter()
        .filter(|g| g.resource.uri.starts_with("external_service:"))
        .collect();
    assert_eq!(
        mcp_grants.len(),
        1,
        "expected exactly one `external_service:<id>` grant"
    );
    assert_eq!(
        mcp_grants[0].resource.uri,
        format!("external_service:{mcp_server_id}")
    );
    let fs: std::collections::HashSet<_> = mcp_grants[0].fundamentals.iter().copied().collect();
    assert!(fs.contains(&Fundamental::NetworkEndpoint));
    assert!(fs.contains(&Fundamental::SecretCredential));
    assert!(fs.contains(&Fundamental::Tag));
}

// ---- 4. Archive hides row + Alerted event ---------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn archive_hides_row_unless_include_archived() {
    let admin = spawn_claimed(false).await;

    let r = admin
        .authed_client
        .post(mcp_url(&admin))
        .json(&json!({
            "display_name": "mem",
            "kind": "mcp",
            "endpoint": "stdio:///usr/local/bin/mem",
            "tenants_allowed": { "mode": "all" }
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 201);
    let reg: Value = r.json().await.unwrap();
    let mcp_server_id = reg["mcp_server_id"].as_str().unwrap().to_string();

    // Archive.
    let r = admin
        .authed_client
        .post(format!("{}/{mcp_server_id}/archive", mcp_url(&admin)))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 200);
    let arc: Value = r.json().await.unwrap();
    let event_id = arc["audit_event_id"]
        .as_str()
        .unwrap()
        .parse::<Uuid>()
        .unwrap();
    let audit = admin
        .acc
        .store
        .get_audit_event(domain::model::ids::AuditEventId::from_uuid(event_id))
        .await
        .unwrap()
        .expect("archived audit event persisted");
    assert_eq!(audit.event_type, "platform.mcp_server.archived");
    assert_eq!(audit.audit_class, AuditClass::Alerted);

    // Default list filters archived rows.
    let r = admin
        .authed_client
        .get(mcp_url(&admin))
        .send()
        .await
        .unwrap();
    let body: Value = r.json().await.unwrap();
    assert_eq!(body["servers"].as_array().unwrap().len(), 0);

    // include_archived=true shows it.
    let r = admin
        .authed_client
        .get(format!("{}?include_archived=true", mcp_url(&admin)))
        .send()
        .await
        .unwrap();
    let body: Value = r.json().await.unwrap();
    let rows = body["servers"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert!(rows[0]["archived_at"].is_string());
}

// ---- 5. Archive on unknown id — 404 ---------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn archive_unknown_id_returns_404() {
    let admin = spawn_claimed(false).await;
    let fake = Uuid::new_v4();
    let r = admin
        .authed_client
        .post(format!("{}/{fake}/archive", mcp_url(&admin)))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 404);
    let err: Value = r.json().await.unwrap();
    assert_eq!(err["code"], "MCP_SERVER_NOT_FOUND");
}

// ---- 6. Register without secret_ref succeeds ------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn register_without_secret_ref_succeeds_for_anon_service() {
    let admin = spawn_claimed(false).await;
    let r = admin
        .authed_client
        .post(mcp_url(&admin))
        .json(&json!({
            "display_name": "anon-mcp",
            "kind": "mcp",
            "endpoint": "http://localhost:9999/mcp",
            "tenants_allowed": { "mode": "all" }
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 201);
}

// ---- 7. PATCH no-op returns empty cascade ---------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn patch_noop_returns_empty_cascade_with_no_audit_event() {
    let admin = spawn_claimed(false).await;

    let r = admin
        .authed_client
        .post(mcp_url(&admin))
        .json(&json!({
            "display_name": "mem",
            "kind": "mcp",
            "endpoint": "stdio:///usr/local/bin/mem",
            "tenants_allowed": { "mode": "all" }
        }))
        .send()
        .await
        .unwrap();
    let mcp_server_id = r.json::<Value>().await.unwrap()["mcp_server_id"]
        .as_str()
        .unwrap()
        .to_string();

    let r = admin
        .authed_client
        .patch(format!("{}/{mcp_server_id}/tenants", mcp_url(&admin)))
        .json(&json!({ "tenants_allowed": { "mode": "all" } }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 200);
    let body: Value = r.json().await.unwrap();
    assert!(body["cascade"].as_array().unwrap().is_empty());
    assert!(body["audit_event_id"].is_null());
}

// ---- 8. End-to-end cascade — register + seed AR/grant + narrow ------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn narrow_tenants_cascades_and_emits_summary_plus_auth_request_revoked() {
    let admin = spawn_claimed(false).await;

    let org_a = OrgId::new();
    let org_b = OrgId::new();

    // Register the MCP server with tenants = Only([a, b]).
    let r = admin
        .authed_client
        .post(mcp_url(&admin))
        .json(&json!({
            "display_name": "shared-mcp",
            "kind": "mcp",
            "endpoint": "stdio:///bin/shared",
            "tenants_allowed": {
                "mode": "only",
                "orgs": [org_a.to_string(), org_b.to_string()]
            }
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 201);
    let mcp_server_id = r.json::<Value>().await.unwrap()["mcp_server_id"]
        .as_str()
        .unwrap()
        .to_string();

    // Seed an AR requested by org_a with a descending grant.
    let ar_id = AuthRequestId::new();
    let ar = AuthRequest {
        id: ar_id,
        requestor: PrincipalRef::Organization(org_a),
        kinds: vec!["#kind:external_service".into()],
        scope: vec![format!("external_service:{mcp_server_id}")],
        state: AuthRequestState::Approved,
        valid_until: None,
        submitted_at: Utc::now(),
        resource_slots: vec![ResourceSlot {
            resource: ResourceRef {
                uri: format!("external_service:{mcp_server_id}"),
            },
            approvers: vec![],
            state: ResourceSlotState::Approved,
        }],
        justification: Some("acceptance-fixture".into()),
        audit_class: AuditClass::Alerted,
        terminal_state_entered_at: None,
        archived: false,
        active_window_days: 90,
        provenance_template: None,
    };
    admin.acc.store.create_auth_request(&ar).await.unwrap();

    let holder = domain::model::ids::AgentId::from_uuid(Uuid::parse_str(&admin.agent_id).unwrap());
    let grant = Grant {
        id: GrantId::new(),
        holder: PrincipalRef::Agent(holder),
        action: vec!["invoke".into()],
        resource: ResourceRef {
            uri: format!("external_service:{mcp_server_id}"),
        },
        fundamentals: vec![Fundamental::NetworkEndpoint],
        descends_from: Some(ar_id),
        delegable: false,
        issued_at: Utc::now(),
        revoked_at: None,
    };
    admin.acc.store.create_grant(&grant).await.unwrap();

    // PATCH tenants to Only([b]) — drops org_a.
    let r = admin
        .authed_client
        .patch(format!("{}/{mcp_server_id}/tenants", mcp_url(&admin)))
        .json(&json!({
            "tenants_allowed": {
                "mode": "only",
                "orgs": [org_b.to_string()]
            }
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 200);
    let body: Value = r.json().await.unwrap();
    let cascade = body["cascade"].as_array().unwrap();
    assert_eq!(cascade.len(), 1, "exactly one org was dropped");
    assert_eq!(cascade[0]["org"].as_str().unwrap(), org_a.to_string());
    assert_eq!(
        cascade[0]["revoked_grants"].as_array().unwrap().len(),
        1,
        "exactly one grant was revoked"
    );
    let summary_id = body["audit_event_id"].as_str().unwrap().to_string();

    // Verify the summary audit event.
    let event_id = summary_id.parse::<Uuid>().unwrap();
    let summary = admin
        .acc
        .store
        .get_audit_event(domain::model::ids::AuditEventId::from_uuid(event_id))
        .await
        .unwrap()
        .expect("summary event persisted");
    assert_eq!(
        summary.event_type,
        "platform.mcp_server.tenant_access_revoked"
    );
    assert_eq!(summary.audit_class, AuditClass::Alerted);
    assert_eq!(summary.diff["after"]["revoked_org_count"], 1);
    assert_eq!(summary.diff["after"]["revoked_auth_request_count"], 1);
    assert_eq!(summary.diff["after"]["revoked_grant_count"], 1);

    // The revoked grant should no longer be live.
    let grants_after = admin
        .acc
        .store
        .list_grants_for_principal(&PrincipalRef::Agent(holder))
        .await
        .unwrap();
    let target = grants_after.iter().find(|g| g.id == grant.id).unwrap();
    assert!(target.revoked_at.is_some(), "grant was not revoked");
}
