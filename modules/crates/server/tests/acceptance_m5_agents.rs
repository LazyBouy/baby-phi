//! CH-24 / F1.B page-5/page-6 vertical slice — milestone-rollup
//! acceptance for the agent-creation + roster surfaces.
//!
//! Per plan §7 P-NEW-TESTS (v2 user-locked F1.B 5-per-page split):
//!
//! - `m5_agents_create_ceo_and_intern_and_assert_page6_visibility` —
//!   bootstrap one org, seed a Human "CFO" agent + an LLM "intern"
//!   agent, and assert both surface on the page-6 roster endpoint
//!   `GET /api/v0/orgs/:org_id/agents` with correct `kind` + `role`
//!   fields.

mod acceptance_common;

use acceptance_common::m5_bootstrap::bootstrap_with_org_and_two_agents;
use serde_json::Value;

#[tokio::test]
async fn m5_agents_create_ceo_and_intern_and_assert_page6_visibility() {
    let fx = bootstrap_with_org_and_two_agents().await;
    let org = &fx.claimed_org;

    let res = org
        .admin
        .authed_client
        .get(org.url(&format!("/api/v0/orgs/{}/agents", org.org_id)))
        .send()
        .await
        .expect("GET roster");
    assert_eq!(res.status().as_u16(), 200);
    let body: Value = res.json().await.expect("decode roster body");
    let agents = body["agents"].as_array().expect("agents array");

    // Baseline fixture seeds 3 agents (CEO + memory-extractor +
    // agent-catalog); the helper adds 2 more (atlas-cfo + atlas-intern).
    assert_eq!(
        agents.len(),
        5,
        "expected 3 baseline + 2 helper-seeded agents: {body:?}"
    );

    // Index each seeded agent by id for direct field assertions
    // (`AgentRosterItemWire.id` is the wire-stable column).
    let by_id: std::collections::HashMap<String, &Value> = agents
        .iter()
        .map(|a| (a["id"].as_str().expect("id string").to_string(), a))
        .collect();

    let cfo = by_id
        .get(&fx.extra_ceo.to_string())
        .expect("seeded CFO present in roster");
    assert_eq!(
        cfo["kind"].as_str(),
        Some("human"),
        "CFO agent kind must be human"
    );
    assert_eq!(
        cfo["role"].as_str(),
        Some("member"),
        "CFO role is member at fixture-time"
    );
    assert_eq!(cfo["display_name"].as_str(), Some("atlas-cfo"));

    let intern = by_id
        .get(&fx.intern.to_string())
        .expect("seeded intern present in roster");
    assert_eq!(
        intern["kind"].as_str(),
        Some("llm"),
        "intern agent kind must be llm"
    );
    assert_eq!(
        intern["role"].as_str(),
        Some("intern"),
        "intern role must be intern"
    );
    assert_eq!(intern["display_name"].as_str(), Some("atlas-intern"));

    // Cross-check: filter the roster by role=intern; only the
    // helper-seeded intern surfaces (memory-extractor + agent-catalog
    // were seeded with role=None per `spawn_claimed_with_org`).
    let only_intern = org
        .admin
        .authed_client
        .get(org.url(&format!("/api/v0/orgs/{}/agents?role=intern", org.org_id)))
        .send()
        .await
        .expect("GET roster?role=intern");
    assert_eq!(only_intern.status().as_u16(), 200);
    let intern_body: Value = only_intern.json().await.expect("decode intern body");
    let intern_arr = intern_body["agents"].as_array().expect("agents array");
    assert_eq!(intern_arr.len(), 1, "exactly one intern in roster");
    assert_eq!(
        intern_arr[0]["id"].as_str(),
        Some(fx.intern.to_string().as_str())
    );
}
