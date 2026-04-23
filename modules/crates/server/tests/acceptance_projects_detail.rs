//! End-to-end acceptance tests for the M4/P7 project-detail surface —
//! `GET /api/v0/projects/:id` + `PATCH /api/v0/projects/:id/okrs`
//! (commitment C16).
//!
//! Scenarios (7):
//!
//! Reader path:
//!  1. Happy read — bootstrap-admin viewer fetches a fixture project
//!     and sees the header + roster (lead only at M4) +
//!     `recent_sessions: []` (C-M5-3 placeholder).
//!  2. 404 — unknown project id.
//!  3. 403 — viewer with no relation to any owning org is denied.
//!
//! OKR patch:
//!  4. Create objective — post-image + audit_event_ids returned.
//!  5. Update key_result — value change rides through.
//!  6. Delete objective — fails if a dependent KR still references it.
//!  7. Invalid patch (duplicate objective_id) rejected with
//!     `OKR_VALIDATION_FAILED`.

mod acceptance_common;

use acceptance_common::admin::{spawn_claimed_with_org_and_project, ClaimedProject};

use chrono::Utc;
use domain::model::ids::{AgentId, OrgId};
use domain::model::nodes::{Agent, AgentKind, Organization};
use domain::Repository;
use serde_json::{json, Value};
use std::sync::Arc;

/// Reader + writer endpoints require an authed session belonging to a
/// member of an owning org. The fixture's bootstrap admin is NOT a
/// member of the fixture org (it's a platform-level agent), so every
/// positive-path test mints a session for the CEO of the owning org.
fn ceo_client(project: &ClaimedProject) -> reqwest::Client {
    acceptance_common::admin::authed_client_for(
        &project.claimed_org.admin,
        project.claimed_org.ceo_agent_id,
    )
    .expect("mint CEO session")
}

fn get_show(project: &ClaimedProject) -> reqwest::RequestBuilder {
    let url = project.url(&format!("/api/v0/projects/{}", project.project_id));
    ceo_client(project).get(url)
}

fn patch_okrs(project: &ClaimedProject, body: Value) -> reqwest::RequestBuilder {
    let url = project.url(&format!("/api/v0/projects/{}/okrs", project.project_id));
    ceo_client(project).patch(url).json(&body)
}

// ---------------------------------------------------------------------------
// Reader path
// ---------------------------------------------------------------------------

#[tokio::test]
async fn show_happy_path() {
    let project = spawn_claimed_with_org_and_project(false).await;
    let res = get_show(&project).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 200);
    let body: Value = res.json().await.unwrap();
    // Wire shape — project header + owning_org_ids + roster + sessions.
    assert_eq!(
        body["project"]["id"].as_str().unwrap(),
        project.project_id.to_string()
    );
    assert_eq!(body["project"]["shape"], "shape_a");
    let owning_orgs = body["owning_org_ids"].as_array().unwrap();
    assert_eq!(owning_orgs.len(), 1);
    assert_eq!(
        owning_orgs[0].as_str().unwrap(),
        project.claimed_org.org_id.to_string()
    );
    // Lead surfaces; roster at M4 = lead only.
    assert_eq!(
        body["lead_agent_id"].as_str().unwrap(),
        project.project_lead.to_string()
    );
    let roster = body["roster"].as_array().unwrap();
    assert!(
        roster
            .iter()
            .any(|m| m["project_role"] == "lead"
                && m["agent_id"] == project.project_lead.to_string()),
        "roster must contain the lead agent"
    );
    // M4 placeholder: recent sessions always empty.
    let sessions = body["recent_sessions"].as_array().unwrap();
    assert!(
        sessions.is_empty(),
        "recent_sessions MUST be empty at M4 — C-M5-3 flips this at M5"
    );
    // phi-core strip invariant — no blueprint / execution_limits leaked.
    let raw = serde_json::to_string(&body).unwrap();
    for forbidden in ["blueprint", "execution_limits", "defaults_snapshot"] {
        assert!(
            !raw.contains(forbidden),
            "ProjectDetail wire must strip `{forbidden}` at every depth"
        );
    }
}

#[tokio::test]
async fn show_unknown_project_returns_404() {
    let project = spawn_claimed_with_org_and_project(false).await;
    let bogus = domain::model::ids::ProjectId::new();
    let url = project.url(&format!("/api/v0/projects/{bogus}"));
    // Use the CEO client — the NotFound gate fires BEFORE the
    // access-denied gate, so either client would surface 404, but the
    // CEO client preserves the invariant that "well-authorised viewer
    // hitting a missing id gets 404".
    let res = ceo_client(&project).get(url).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 404);
    let err: Value = res.json().await.unwrap();
    assert_eq!(err["code"], "PROJECT_NOT_FOUND");
}

#[tokio::test]
async fn show_unrelated_viewer_returns_403() {
    let project = spawn_claimed_with_org_and_project(false).await;
    let store: Arc<dyn Repository> = project.claimed_org.admin.acc.store.clone();

    // Stand up an unrelated org + an agent in it that will log in as
    // the viewer.
    let other_org_id = OrgId::new();
    store
        .create_organization(&Organization {
            id: other_org_id,
            display_name: "Outsider".into(),
            vision: None,
            mission: None,
            consent_policy: domain::model::composites_m3::ConsentPolicy::Implicit,
            audit_class_default: domain::audit::AuditClass::Logged,
            authority_templates_enabled: vec![],
            defaults_snapshot: None,
            default_model_provider: None,
            system_agents: vec![],
            created_at: Utc::now(),
        })
        .await
        .unwrap();
    let outsider_id = AgentId::new();
    store
        .create_agent(&Agent {
            id: outsider_id,
            kind: AgentKind::Human,
            display_name: "Outsider CEO".into(),
            owning_org: Some(other_org_id),
            role: None,
            created_at: Utc::now(),
        })
        .await
        .unwrap();

    // Mint a session cookie for the outsider and hit the endpoint.
    let outsider_client =
        acceptance_common::admin::authed_client_for(&project.claimed_org.admin, outsider_id)
            .expect("mint session for outsider");
    let url = project.url(&format!("/api/v0/projects/{}", project.project_id));
    let res = outsider_client.get(url).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 403);
    let err: Value = res.json().await.unwrap();
    assert_eq!(err["code"], "PROJECT_ACCESS_DENIED");
}

// ---------------------------------------------------------------------------
// OKR patch path
// ---------------------------------------------------------------------------

fn sample_objective(id: &str, owner: AgentId) -> Value {
    json!({
        "objective_id": id,
        "name": format!("Objective {id}"),
        "description": "",
        "status": "draft",
        "owner": owner.to_string(),
        "key_result_ids": [],
    })
}

fn sample_kr(id: &str, obj_id: &str, owner: AgentId, target_int: i64) -> Value {
    json!({
        "kr_id": id,
        "objective_id": obj_id,
        "name": format!("KR {id}"),
        "description": "",
        "measurement_type": "count",
        "target_value": { "kind": "integer", "value": target_int },
        "owner": owner.to_string(),
        "status": "not_started",
    })
}

#[tokio::test]
async fn okr_patch_create_objective_and_kr() {
    let project = spawn_claimed_with_org_and_project(false).await;
    let owner = project.project_lead;
    let body = json!({
        "patches": [
            { "kind": "objective", "op": "create", "payload": sample_objective("obj-1", owner) },
            { "kind": "key_result", "op": "create",
              "payload": sample_kr("kr-1", "obj-1", owner, 100) }
        ]
    });
    let res = patch_okrs(&project, body).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 200);
    let body: Value = res.json().await.unwrap();
    let audits = body["audit_event_ids"].as_array().unwrap();
    assert_eq!(audits.len(), 2, "one audit event per applied mutation");
    let objectives = body["objectives"].as_array().unwrap();
    assert_eq!(objectives.len(), 1);
    let key_results = body["key_results"].as_array().unwrap();
    assert_eq!(key_results.len(), 1);
}

#[tokio::test]
async fn okr_patch_update_keyresult_progress() {
    let project = spawn_claimed_with_org_and_project(false).await;
    let owner = project.project_lead;
    // Seed one KR so we can update it.
    let seed = json!({
        "patches": [
            { "kind": "objective", "op": "create", "payload": sample_objective("obj-1", owner) },
            { "kind": "key_result", "op": "create",
              "payload": sample_kr("kr-1", "obj-1", owner, 100) }
        ]
    });
    patch_okrs(&project, seed).send().await.unwrap();

    // Update the KR's current_value.
    let mut updated_kr = sample_kr("kr-1", "obj-1", owner, 100);
    updated_kr["current_value"] = json!({ "kind": "integer", "value": 40 });
    updated_kr["status"] = json!("in_progress");
    let patch = json!({
        "patches": [
            { "kind": "key_result", "op": "update", "payload": updated_kr }
        ]
    });
    let res = patch_okrs(&project, patch).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 200);
    let body: Value = res.json().await.unwrap();
    let kr = &body["key_results"][0];
    assert_eq!(kr["current_value"]["value"], 40);
    assert_eq!(kr["status"], "in_progress");
}

#[tokio::test]
async fn okr_patch_delete_objective_with_dependent_kr_rejected() {
    let project = spawn_claimed_with_org_and_project(false).await;
    let owner = project.project_lead;
    let seed = json!({
        "patches": [
            { "kind": "objective", "op": "create", "payload": sample_objective("obj-1", owner) },
            { "kind": "key_result", "op": "create",
              "payload": sample_kr("kr-1", "obj-1", owner, 100) }
        ]
    });
    patch_okrs(&project, seed).send().await.unwrap();

    // Try to delete the parent objective while the KR still references it.
    let bad = json!({
        "patches": [
            { "kind": "objective", "op": "delete", "objective_id": "obj-1" }
        ]
    });
    let res = patch_okrs(&project, bad).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 400);
    let err: Value = res.json().await.unwrap();
    assert_eq!(err["code"], "OKR_VALIDATION_FAILED");
}

#[tokio::test]
async fn okr_patch_duplicate_objective_id_rejected() {
    let project = spawn_claimed_with_org_and_project(false).await;
    let owner = project.project_lead;
    let body = json!({
        "patches": [
            { "kind": "objective", "op": "create", "payload": sample_objective("obj-1", owner) },
            { "kind": "objective", "op": "create", "payload": sample_objective("obj-1", owner) }
        ]
    });
    let res = patch_okrs(&project, body).send().await.unwrap();
    assert_eq!(res.status().as_u16(), 400);
    let err: Value = res.json().await.unwrap();
    assert_eq!(err["code"], "OKR_VALIDATION_FAILED");
}
