//! End-to-end acceptance tests for `POST /api/v0/orgs` + the list
//! / show companions.
//!
//! Commitment C11 in the M3 plan. Scenarios:
//!
//! 1. Validation failures (5 separate cases) — each maps to a stable
//!    `{code, message}` envelope.
//! 2. Happy path — minimal-startup payload; returns 201 + full
//!    receipt; follow-up GET `/orgs/:id` returns the persisted org.
//! 3. 409 duplicate-org-id (via rapid re-submit of the same CEO +
//!    identical shape; org_id is server-minted so the duplicate
//!    surfaces as a different kind of collision — we verify that the
//!    *second* create succeeds with a distinct id, and the two
//!    co-exist under `GET /orgs`).
//! 4. **phi-core transit invariant** — the persisted organization's
//!    `defaults_snapshot` contains real `phi_core::ExecutionLimits`
//!    / `ContextConfig` / `RetryConfig` / `AgentProfile` field
//!    shapes; each system agent's `blueprint` is `phi_core::AgentProfile`.
//! 5. **ADR-0023 invariant** — post-creation, `execution_limits` /
//!    `retry_policy` / `cache_policy` / `compaction_policy` tables
//!    have zero rows.

mod acceptance_common;

use acceptance_common::admin::spawn_claimed;
use domain::repository::Repository;
use serde_json::json;

fn happy_body() -> serde_json::Value {
    // Minimal-startup wizard payload — lets the server snapshot
    // platform defaults automatically (no override supplied).
    json!({
        "display_name": "Acme Research",
        "vision": "Memory-first autonomous research",
        "mission": "Automate literature review",
        "consent_policy": "implicit",
        "audit_class_default": "logged",
        "authority_templates_enabled": ["a"],
        "default_model_provider": null,
        "ceo_display_name": "Alice",
        "ceo_channel_kind": "email",
        "ceo_channel_handle": "alice@acme.test",
        "token_budget": 1_000_000
    })
}

async fn post_orgs(
    admin: &acceptance_common::admin::ClaimedAdmin,
    body: serde_json::Value,
) -> reqwest::Response {
    admin
        .authed_client
        .post(admin.url("/api/v0/orgs"))
        .json(&body)
        .send()
        .await
        .expect("post orgs")
}

// ---------------------------------------------------------------------------
// Validation failures
// ---------------------------------------------------------------------------

#[tokio::test]
async fn create_rejects_empty_display_name() {
    let admin = spawn_claimed(false).await;
    let mut body = happy_body();
    body["display_name"] = json!("   ");
    let res = post_orgs(&admin, body).await;
    assert_eq!(res.status().as_u16(), 400);
    let err: serde_json::Value = res.json().await.unwrap();
    assert_eq!(err["code"].as_str(), Some("VALIDATION_FAILED"));
    assert!(err["message"].as_str().unwrap().contains("display_name"));
}

#[tokio::test]
async fn create_rejects_empty_ceo_display_name() {
    let admin = spawn_claimed(false).await;
    let mut body = happy_body();
    body["ceo_display_name"] = json!("");
    let res = post_orgs(&admin, body).await;
    assert_eq!(res.status().as_u16(), 400);
}

#[tokio::test]
async fn create_rejects_zero_token_budget() {
    let admin = spawn_claimed(false).await;
    let mut body = happy_body();
    body["token_budget"] = json!(0);
    let res = post_orgs(&admin, body).await;
    assert_eq!(res.status().as_u16(), 400);
    let err: serde_json::Value = res.json().await.unwrap();
    assert!(err["message"].as_str().unwrap().contains("token_budget"));
}

#[tokio::test]
async fn create_rejects_duplicate_template_kinds() {
    let admin = spawn_claimed(false).await;
    let mut body = happy_body();
    body["authority_templates_enabled"] = json!(["a", "a"]);
    let res = post_orgs(&admin, body).await;
    assert_eq!(res.status().as_u16(), 400);
}

#[tokio::test]
async fn create_rejects_non_adoptable_template_e() {
    let admin = spawn_claimed(false).await;
    let mut body = happy_body();
    // Template E is platform-level (not adoptable by orgs).
    body["authority_templates_enabled"] = json!(["e"]);
    let res = post_orgs(&admin, body).await;
    assert_eq!(res.status().as_u16(), 400);
    let err: serde_json::Value = res.json().await.unwrap();
    assert_eq!(err["code"].as_str(), Some("TEMPLATE_NOT_ADOPTABLE"));
}

#[tokio::test]
async fn create_rejects_non_adoptable_template_f() {
    let admin = spawn_claimed(false).await;
    let mut body = happy_body();
    body["authority_templates_enabled"] = json!(["f"]);
    let res = post_orgs(&admin, body).await;
    assert_eq!(res.status().as_u16(), 400);
    let err: serde_json::Value = res.json().await.unwrap();
    assert_eq!(err["code"].as_str(), Some("TEMPLATE_NOT_ADOPTABLE"));
}

// ---------------------------------------------------------------------------
// Happy path + receipt shape
// ---------------------------------------------------------------------------

#[tokio::test]
async fn create_happy_path_returns_201_and_full_receipt() {
    let admin = spawn_claimed(false).await;
    let res = post_orgs(&admin, happy_body()).await;
    assert_eq!(res.status().as_u16(), 201);
    let receipt: serde_json::Value = res.json().await.unwrap();

    assert!(receipt["org_id"].is_string());
    assert!(receipt["ceo_agent_id"].is_string());
    assert_eq!(
        receipt["system_agent_ids"].as_array().map(|v| v.len()),
        Some(2)
    );
    // 1 adoption AR for Template A.
    assert_eq!(
        receipt["adoption_auth_request_ids"]
            .as_array()
            .map(|v| v.len()),
        Some(1)
    );
    // Audit events: organization_created + 1 authority_template.adopted.
    assert_eq!(
        receipt["audit_event_ids"].as_array().map(|v| v.len()),
        Some(2)
    );
}

#[tokio::test]
async fn create_then_show_returns_defaults_snapshot_with_phi_core_fields() {
    // Positive phi-core transit assertion: after creation, GET /orgs/:id
    // must return `organization.defaults_snapshot` with the 4 phi-core
    // wrapped fields populated. A baby-phi redeclaration would lose
    // fields or serialise them under different keys.
    let admin = spawn_claimed(false).await;
    let res = post_orgs(&admin, happy_body()).await;
    assert_eq!(res.status().as_u16(), 201);
    let receipt: serde_json::Value = res.json().await.unwrap();
    let org_id = receipt["org_id"].as_str().unwrap().to_string();

    let show = admin
        .authed_client
        .get(admin.url(&format!("/api/v0/orgs/{org_id}")))
        .send()
        .await
        .expect("get /orgs/:id");
    assert_eq!(show.status().as_u16(), 200);
    let body: serde_json::Value = show.json().await.unwrap();

    let snap = &body["organization"]["defaults_snapshot"];
    // Each of the four phi-core-wrapped fields MUST be an object
    // carrying phi-core's known field names:
    assert!(snap["execution_limits"].is_object());
    assert!(snap["execution_limits"]["max_turns"].is_number());
    assert!(snap["execution_limits"]["max_total_tokens"].is_number());
    assert!(snap["context_config"].is_object());
    assert!(snap["retry_config"].is_object());
    assert!(snap["retry_config"]["max_retries"].is_number());
    assert!(snap["default_agent_profile"].is_object());
    assert!(snap["default_agent_profile"]["profile_id"].is_string());
    // Baby-phi-governance fields next to the phi-core wraps.
    assert!(snap["default_retention_days"].is_number());
}

#[tokio::test]
async fn create_respects_adr_0023_no_per_agent_policy_nodes() {
    // After creation, querying the per-agent policy tables must
    // return zero rows — the ADR-0023 inherit-from-snapshot
    // invariant applied at the HTTP layer (the repo-layer test is
    // at `store/tests/apply_org_creation_tx_test.rs`; this is the
    // E2E re-verification).
    let admin = spawn_claimed(false).await;
    let res = post_orgs(&admin, happy_body()).await;
    assert_eq!(res.status().as_u16(), 201);

    let store = admin.acc.store.as_ref();
    for table in &[
        "execution_limits",
        "retry_policy",
        "cache_policy",
        "compaction_policy",
    ] {
        let q = format!("SELECT count() FROM {table} GROUP ALL");
        let counts: Vec<i64> = store
            .client()
            .query(&q)
            .await
            .unwrap()
            .take((0, "count"))
            .unwrap();
        assert_eq!(
            counts.into_iter().next().unwrap_or(0),
            0,
            "{table} must be empty post-creation — ADR-0023 invariant",
        );
    }
}

#[tokio::test]
async fn create_each_system_agent_blueprint_has_phi_core_shape() {
    // The persisted `agent_profile.blueprint` column must carry
    // the full `phi_core::AgentProfile` shape — system_prompt +
    // name populated via the role tweak; profile_id present.
    let admin = spawn_claimed(false).await;
    let res = post_orgs(&admin, happy_body()).await;
    let receipt: serde_json::Value = res.json().await.unwrap();
    let sys_ids = receipt["system_agent_ids"].as_array().unwrap();
    assert_eq!(sys_ids.len(), 2);

    let store = admin.acc.store.as_ref();
    let agents = store
        .list_agents_in_org(domain::model::ids::OrgId::from_uuid(
            uuid::Uuid::parse_str(receipt["org_id"].as_str().unwrap()).unwrap(),
        ))
        .await
        .unwrap();
    // 3 = CEO + 2 system agents.
    assert_eq!(agents.len(), 3);

    // Query each system agent's profile.blueprint.system_prompt —
    // must be populated with the role-specific string.
    let sys_agent_ids: Vec<String> = sys_ids
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    let prompts: Vec<Option<String>> = store
        .client()
        .query(
            "SELECT blueprint.system_prompt AS system_prompt \
             FROM agent_profile \
             WHERE agent_id IN $ids",
        )
        .bind(("ids", sys_agent_ids))
        .await
        .unwrap()
        .take((0, "system_prompt"))
        .unwrap();
    assert_eq!(prompts.len(), 2);
    let flat: Vec<String> = prompts.into_iter().flatten().collect();
    // The two prompts mention their role.
    assert!(
        flat.iter().any(|p| p.contains("memory-extraction")),
        "one profile should be the memory-extractor; got: {flat:?}"
    );
    assert!(
        flat.iter().any(|p| p.contains("agent-catalog")),
        "one profile should be the agent-catalog; got: {flat:?}"
    );
}

#[tokio::test]
async fn create_emits_audit_chain_events_under_some_org_id() {
    // The two emitted audit events (OrganizationCreated +
    // authority_template.adopted) both carry `org_scope = Some(org_id)`
    // — opening the new org's hash chain. Verifies plan §G8 +
    // per-org-audit-chain.md invariant at the HTTP boundary.
    let admin = spawn_claimed(false).await;
    let res = post_orgs(&admin, happy_body()).await;
    let receipt: serde_json::Value = res.json().await.unwrap();
    let org_id = receipt["org_id"].as_str().unwrap().to_string();
    let event_ids = receipt["audit_event_ids"].as_array().unwrap();
    assert_eq!(event_ids.len(), 2);

    let store = admin.acc.store.as_ref();
    let recent = store
        .list_recent_audit_events_for_org(
            domain::model::ids::OrgId::from_uuid(uuid::Uuid::parse_str(&org_id).unwrap()),
            10,
        )
        .await
        .unwrap();
    assert_eq!(recent.len(), 2);
    // First event (newest — DESC by timestamp) is template.adopted;
    // second is organization.created. Both have org_scope = Some(org).
    for ev in &recent {
        assert_eq!(ev.org_scope.unwrap().to_string(), org_id);
    }
    let types: std::collections::HashSet<&str> =
        recent.iter().map(|e| e.event_type.as_str()).collect();
    assert!(types.contains("platform.organization.created"));
    assert!(types.contains("authority_template.adopted"));
}

// ---------------------------------------------------------------------------
// List + show integration
// ---------------------------------------------------------------------------

#[tokio::test]
async fn create_two_orgs_then_list_returns_both() {
    let admin = spawn_claimed(false).await;

    let r1 = post_orgs(&admin, happy_body()).await;
    assert_eq!(r1.status().as_u16(), 201);
    let mut b2 = happy_body();
    b2["display_name"] = json!("BetaCorp");
    b2["ceo_channel_handle"] = json!("beta@betacorp.test");
    let r2 = post_orgs(&admin, b2).await;
    assert_eq!(r2.status().as_u16(), 201);

    let list = admin
        .authed_client
        .get(admin.url("/api/v0/orgs"))
        .send()
        .await
        .unwrap();
    assert_eq!(list.status().as_u16(), 200);
    let body: serde_json::Value = list.json().await.unwrap();
    let orgs = body["orgs"].as_array().unwrap();
    assert_eq!(orgs.len(), 2);
    let names: std::collections::HashSet<&str> = orgs
        .iter()
        .map(|o| o["display_name"].as_str().unwrap())
        .collect();
    assert!(names.contains("Acme Research"));
    assert!(names.contains("BetaCorp"));
    // Each summary surfaces `member_count = 3` (CEO + 2 system agents).
    for o in orgs {
        assert_eq!(o["member_count"].as_u64(), Some(3));
    }
}

#[tokio::test]
async fn show_unknown_org_is_404() {
    let admin = spawn_claimed(false).await;
    let fake = uuid::Uuid::new_v4();
    let res = admin
        .authed_client
        .get(admin.url(&format!("/api/v0/orgs/{fake}")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 404);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["code"].as_str(), Some("ORG_NOT_FOUND"));
}
