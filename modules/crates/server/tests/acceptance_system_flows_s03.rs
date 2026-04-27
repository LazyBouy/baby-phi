//! M5/CH-22 — system flow s03 acceptance tests.
//!
//! Closes the loop on the agent-catalog listener body: the production
//! HTTP code paths emit the trigger events the listener subscribes
//! to, and a real `AgentCatalogListener` subscribed to the harness's
//! event bus mutates the durable `agent_catalog_entry` rows + the
//! catalog system agent's `system_agent_runtime_status` tile (drift
//! D6.1 second call site).
//!
//! Scenarios:
//!  1. **Agent creation populates catalog.** `POST /api/v0/orgs/:org/agents`
//!     creates a Human Member; the listener fires on `AgentCreated`
//!     and (a) seeds the catalog row with `active = true`,
//!     (b) advances the catalog system agent's runtime-status tile.
//!  2. **Archive flips catalog active.** A custom system agent is
//!     added then archived; the listener fires on `AgentArchived`
//!     and the catalog row's `active` flips to `false` (ADR-0034
//!     §D34.5 archive-wins-ties via the durable `archived_at` flip).

mod acceptance_common;

use std::sync::Arc;

use acceptance_common::admin::{spawn_claimed_with_org, ClaimedOrg};

use chrono::Utc;
use domain::audit::AuditEmitter;
use domain::events::{AgentCatalogListener, CatalogAuditMode};
use domain::Repository;
use serde_json::{json, Value};

fn ceo_client(org: &ClaimedOrg) -> reqwest::Client {
    acceptance_common::admin::authed_client_for(&org.admin, org.ceo_agent_id)
        .expect("mint CEO session")
}

/// Subscribe a real `AgentCatalogListener` to the harness's event
/// bus so the listener observes emits from production handlers. The
/// default harness has no subscribers; CH-22 wires one inline per
/// test so existing tests remain unaffected.
fn subscribe_catalog_listener(org: &ClaimedOrg) {
    let repo: Arc<dyn Repository> = org.admin.acc.store.clone();
    let audit: Arc<dyn AuditEmitter> = Arc::new(store::SurrealAuditEmitter::new(repo.clone()));
    let listener = Arc::new(AgentCatalogListener::new(
        repo,
        audit,
        CatalogAuditMode::Silent,
    ));
    org.admin.acc.event_bus.subscribe(listener);
}

// ---------------------------------------------------------------------------
// Scenario A — Agent creation populates catalog
// ---------------------------------------------------------------------------

#[tokio::test]
async fn agent_create_populates_catalog_and_advances_runtime_status_tile() {
    let org = spawn_claimed_with_org(false).await;
    subscribe_catalog_listener(&org);
    let test_start = Utc::now();

    // POST /api/v0/orgs/:org/agents — Human Member.
    let body = json!({
        "display_name": "iris-member",
        "kind": "human",
        "role": "member",
        "blueprint": {},
        "parallelize": 1,
        "initial_execution_limits_override": null,
    });
    let url = org.url(&format!("/api/v0/orgs/{}/agents", org.org_id));
    let res = ceo_client(&org)
        .post(url)
        .json(&body)
        .send()
        .await
        .expect("POST agents");
    assert_eq!(
        res.status().as_u16(),
        201,
        "create returned {:?}",
        res.text().await
    );
    let resp: Value = res.json().await.unwrap();
    let agent_id: domain::model::ids::AgentId =
        serde_json::from_value(resp["agent_id"].clone()).expect("agent_id");

    // (a) Catalog row exists + active = true.
    let entry = org
        .admin
        .acc
        .store
        .get_agent_catalog_entry(agent_id)
        .await
        .expect("get_agent_catalog_entry")
        .expect("catalog row was upserted by the listener");
    assert!(entry.active, "newly-created agent's catalog row is active");
    assert_eq!(entry.display_name, "iris-member");
    assert_eq!(entry.role.as_deref(), Some("member"));
    assert_eq!(entry.owning_org, org.org_id);

    // (b) Runtime-status tile advanced. The fixture provisions the
    //     catalog system agent at index [1] of `system_agents` (per
    //     `spawn_claimed_with_org` ordering — memory-extractor first,
    //     agent-catalog second).
    let catalog_sys_agent_id = org.system_agents[1];
    let tiles = org
        .admin
        .acc
        .store
        .fetch_system_agent_runtime_status_for_org(org.org_id)
        .await
        .expect("fetch runtime status");
    let tile = tiles
        .iter()
        .find(|t| t.agent_id == catalog_sys_agent_id)
        .expect("runtime-status tile present for catalog system agent");
    let last_fired = tile.last_fired_at.expect("last_fired_at populated");
    assert!(
        last_fired >= test_start,
        "last_fired_at advanced after listener fired (test_start={test_start:?}, \
         last_fired={last_fired:?})",
    );
}

// ---------------------------------------------------------------------------
// Scenario B — Archive flips catalog active
// ---------------------------------------------------------------------------

#[tokio::test]
async fn archive_flips_catalog_active_to_false() {
    let org = spawn_claimed_with_org(false).await;
    subscribe_catalog_listener(&org);

    // Add a custom system agent (standard agents can't be archived).
    let add_url = org.url(&format!("/api/v0/orgs/{}/system-agents", org.org_id));
    let add_resp: Value = ceo_client(&org)
        .post(&add_url)
        .json(&json!({
            "display_name": "to-archive-flow-s03",
            "profile_ref": "custom-archive-flow-s03",
            "parallelize": 1,
            "trigger": "session_end",
        }))
        .send()
        .await
        .expect("add system agent")
        .json()
        .await
        .expect("add response");
    let custom_agent_id: domain::model::ids::AgentId =
        serde_json::from_value(add_resp["agent_id"].clone()).expect("agent_id");

    // Add fired AgentCreated → catalog row created with active=true.
    let pre = org
        .admin
        .acc
        .store
        .get_agent_catalog_entry(custom_agent_id)
        .await
        .expect("get_agent_catalog_entry pre-archive")
        .expect("catalog row seeded by add");
    assert!(pre.active, "pre-archive catalog row is active");

    // Archive it.
    let archive_url = org.url(&format!(
        "/api/v0/orgs/{}/system-agents/{}/archive",
        org.org_id, custom_agent_id
    ));
    let archive_res = ceo_client(&org)
        .post(&archive_url)
        .send()
        .await
        .expect("archive request");
    assert_eq!(
        archive_res.status().as_u16(),
        200,
        "archive returned {:?}",
        archive_res.text().await
    );

    // Durable agent row now has archived_at = Some.
    let agent = org
        .admin
        .acc
        .store
        .get_agent(custom_agent_id)
        .await
        .expect("get_agent")
        .expect("agent row exists");
    assert!(
        agent.archived_at.is_some(),
        "archive handler flipped durable archived_at",
    );

    // Listener fired on AgentArchived → catalog row's active = false
    // (ADR-0034 D34.5 — archive wins ties even when agent.active is
    //  still true at the durable level).
    let post = org
        .admin
        .acc
        .store
        .get_agent_catalog_entry(custom_agent_id)
        .await
        .expect("get_agent_catalog_entry post-archive")
        .expect("catalog row still present after archive");
    assert!(
        !post.active,
        "post-archive catalog row's active flipped to false (ADR-0034 D34.5)",
    );
}
