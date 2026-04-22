//! Integration tests for the full `SurrealStore: Repository` surface that
//! P2 lands. Covers node CRUD, grants, auth requests, ownership edges
//! (including the typed free-function wrappers from ADR-0015), bootstrap
//! credentials, resources catalogue, and audit events.
//!
//! Every test boots a fresh embedded SurrealDB in a tempdir and drops it
//! at scope end — the tests are independent and parallel-safe.

use chrono::{Duration, Utc};
use domain::audit::{AuditClass, AuditEvent};
use domain::model::ids::{
    AgentId, AuditEventId, AuthRequestId, ConsentId, GrantId, MemoryId, NodeId, OrgId, ProjectId,
    TemplateId, UserId,
};
use domain::model::nodes::{
    Agent, AgentKind, AgentProfile, ApproverSlot, ApproverSlotState, AuthRequest, AuthRequestState,
    Channel, ChannelKind, Consent, Grant, InboxObject, Memory, Organization, OutboxObject,
    PrincipalRef, ResourceRef, ResourceSlot, ResourceSlotState, Template, ToolAuthorityManifest,
    User,
};
use domain::repository::{self, Repository};
use store::SurrealStore;
use tempfile::TempDir;
use uuid::Uuid;

async fn fresh_store() -> (SurrealStore, TempDir) {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = SurrealStore::open_embedded(dir.path().join("db"), "phi", "test")
        .await
        .expect("open embedded");
    (store, dir)
}

// ---------------------------------------------------------------------------
// Node CRUD
// ---------------------------------------------------------------------------

fn sample_agent() -> Agent {
    Agent {
        id: AgentId::new(),
        kind: AgentKind::Human,
        display_name: "Alice".into(),
        owning_org: None,
        role: None,
        created_at: Utc::now(),
    }
}

#[tokio::test]
async fn agent_create_and_get_round_trip() {
    let (store, _dir) = fresh_store().await;
    let agent = sample_agent();
    store.create_agent(&agent).await.expect("create");

    let got = store.get_agent(agent.id).await.expect("get").expect("row");
    assert_eq!(got.id, agent.id);
    assert_eq!(got.display_name, "Alice");
    assert_eq!(got.kind, AgentKind::Human);
}

#[tokio::test]
async fn get_agent_returns_none_when_absent() {
    let (store, _dir) = fresh_store().await;
    let missing = store.get_agent(AgentId::new()).await.expect("get");
    assert!(missing.is_none());
}

#[tokio::test]
async fn organization_create_and_get_round_trip() {
    let (store, _dir) = fresh_store().await;
    let org = Organization {
        id: OrgId::new(),
        display_name: "Acme".into(),
        vision: None,
        mission: None,
        consent_policy: domain::model::ConsentPolicy::Implicit,
        audit_class_default: domain::audit::AuditClass::Logged,
        authority_templates_enabled: vec![],
        defaults_snapshot: None,
        default_model_provider: None,
        system_agents: vec![],
        created_at: Utc::now(),
    };
    store.create_organization(&org).await.expect("create");

    let got = store
        .get_organization(org.id)
        .await
        .expect("get")
        .expect("row");
    assert_eq!(got.display_name, "Acme");
}

// ---- Create-only node surfaces — one focused test per surface, each
// verifying persistence via a direct SurrealDB `count()` query (since no
// `get_*` method exists on the trait for these types yet).

/// Verify that a row with the given primary key exists in the named table.
async fn row_exists(store: &SurrealStore, table: &str, id: &str) -> bool {
    let q = format!("SELECT count() FROM type::thing('{table}', $id) GROUP ALL");
    let counts: Vec<i64> = store
        .client()
        .query(&q)
        .bind(("id", id.to_string()))
        .await
        .unwrap()
        .take((0, "count"))
        .unwrap();
    counts.into_iter().next().unwrap_or(0) > 0
}

#[tokio::test]
async fn create_agent_profile_persists_row() {
    let (store, _dir) = fresh_store().await;
    let id = NodeId::new();
    let blueprint = phi_core::agents::profile::AgentProfile {
        profile_id: "alice-profile".into(),
        name: Some("Alice-profile".into()),
        system_prompt: Some("You are Alice.".into()),
        temperature: Some(0.7),
        ..phi_core::agents::profile::AgentProfile::default()
    };
    store
        .create_agent_profile(&AgentProfile {
            id,
            agent_id: AgentId::new(),
            parallelize: 4,
            blueprint,
            created_at: Utc::now(),
        })
        .await
        .unwrap();
    assert!(row_exists(&store, "agent_profile", &id.to_string()).await);

    // Confirm the `parallelize` field landed (not just the row).
    let p: Vec<u32> = store
        .client()
        .query("SELECT parallelize FROM type::thing('agent_profile', $id)")
        .bind(("id", id.to_string()))
        .await
        .unwrap()
        .take((0, "parallelize"))
        .unwrap();
    assert_eq!(p.first().copied(), Some(4));

    // Confirm the phi-core blueprint round-trips (system_prompt preserved).
    let prompts: Vec<Option<String>> = store
        .client()
        .query("SELECT blueprint.system_prompt AS system_prompt FROM type::thing('agent_profile', $id)")
        .bind(("id", id.to_string()))
        .await
        .unwrap()
        .take((0, "system_prompt"))
        .unwrap();
    assert_eq!(
        prompts.first().and_then(|p| p.clone()).as_deref(),
        Some("You are Alice."),
        "phi-core blueprint fields must round-trip through SurrealDB"
    );
}

#[tokio::test]
async fn create_user_persists_row() {
    let (store, _dir) = fresh_store().await;
    let id = UserId::new();
    store
        .create_user(&User {
            id,
            display_name: "alice@example.com".into(),
            created_at: Utc::now(),
        })
        .await
        .unwrap();
    assert!(row_exists(&store, "user", &id.to_string()).await);
}

#[tokio::test]
async fn create_template_persists_row_with_name() {
    let (store, _dir) = fresh_store().await;
    let id = TemplateId::new();
    store
        .create_template(&Template {
            id,
            name: "template:system_bootstrap".into(),
            kind: domain::model::nodes::TemplateKind::SystemBootstrap,
            created_at: Utc::now(),
        })
        .await
        .unwrap();
    let names: Vec<String> = store
        .client()
        .query("SELECT name FROM type::thing('template', $id)")
        .bind(("id", id.to_string()))
        .await
        .unwrap()
        .take((0, "name"))
        .unwrap();
    assert_eq!(
        names.first().map(String::as_str),
        Some("template:system_bootstrap")
    );
}

#[tokio::test]
async fn create_inbox_persists_row_with_agent_link() {
    let (store, _dir) = fresh_store().await;
    let id = NodeId::new();
    let agent_id = AgentId::new();
    store
        .create_inbox(&InboxObject {
            id,
            agent_id,
            created_at: Utc::now(),
        })
        .await
        .unwrap();
    let linked: Vec<String> = store
        .client()
        .query("SELECT agent_id FROM type::thing('inbox_object', $id)")
        .bind(("id", id.to_string()))
        .await
        .unwrap()
        .take((0, "agent_id"))
        .unwrap();
    assert_eq!(
        linked.first().map(String::as_str),
        Some(agent_id.to_string().as_str())
    );
}

#[tokio::test]
async fn create_outbox_persists_row() {
    let (store, _dir) = fresh_store().await;
    let id = NodeId::new();
    store
        .create_outbox(&OutboxObject {
            id,
            agent_id: AgentId::new(),
            created_at: Utc::now(),
        })
        .await
        .unwrap();
    assert!(row_exists(&store, "outbox_object", &id.to_string()).await);
}

#[tokio::test]
async fn create_channel_persists_row_with_kind_and_handle() {
    let (store, _dir) = fresh_store().await;
    let id = NodeId::new();
    store
        .create_channel(&Channel {
            id,
            agent_id: AgentId::new(),
            kind: ChannelKind::Slack,
            handle: "@alice".into(),
            created_at: Utc::now(),
        })
        .await
        .unwrap();
    let kinds: Vec<String> = store
        .client()
        .query("SELECT kind FROM type::thing('channel', $id)")
        .bind(("id", id.to_string()))
        .await
        .unwrap()
        .take((0, "kind"))
        .unwrap();
    assert_eq!(kinds.first().map(String::as_str), Some("slack"));

    let handles: Vec<String> = store
        .client()
        .query("SELECT handle FROM type::thing('channel', $id)")
        .bind(("id", id.to_string()))
        .await
        .unwrap()
        .take((0, "handle"))
        .unwrap();
    assert_eq!(handles.first().map(String::as_str), Some("@alice"));
}

#[tokio::test]
async fn create_memory_persists_row_preserving_tags() {
    let (store, _dir) = fresh_store().await;
    let id = MemoryId::new();
    store
        .create_memory(&Memory {
            id,
            owning_agent: AgentId::new(),
            tags: vec![
                "#kind:memory".into(),
                "memory:m-1".into(),
                "project:alpha".into(),
            ],
            created_at: Utc::now(),
        })
        .await
        .unwrap();
    let tags: Vec<Vec<String>> = store
        .client()
        .query("SELECT tags FROM type::thing('memory', $id)")
        .bind(("id", id.to_string()))
        .await
        .unwrap()
        .take((0, "tags"))
        .unwrap();
    let got = tags.into_iter().next().expect("row").clone();
    assert_eq!(got.len(), 3);
    assert!(got.contains(&"#kind:memory".to_string()));
    assert!(got.contains(&"memory:m-1".to_string()));
    assert!(got.contains(&"project:alpha".to_string()));
}

#[tokio::test]
async fn create_consent_persists_row_with_subordinate_and_org() {
    let (store, _dir) = fresh_store().await;
    let id = ConsentId::new();
    let subordinate = AgentId::new();
    let org = OrgId::new();
    store
        .create_consent(&Consent {
            id,
            subordinate,
            scoped_to: org,
            granted_at: Utc::now(),
            revoked_at: None,
        })
        .await
        .unwrap();
    assert!(row_exists(&store, "consent", &id.to_string()).await);

    let subs: Vec<String> = store
        .client()
        .query("SELECT subordinate FROM type::thing('consent', $id)")
        .bind(("id", id.to_string()))
        .await
        .unwrap()
        .take((0, "subordinate"))
        .unwrap();
    assert_eq!(
        subs.first().map(String::as_str),
        Some(subordinate.to_string().as_str())
    );
}

#[tokio::test]
async fn create_tool_authority_manifest_persists_full_shape() {
    let (store, _dir) = fresh_store().await;
    let id = NodeId::new();
    store
        .create_tool_authority_manifest(&ToolAuthorityManifest {
            id,
            tool_name: "read_memory".into(),
            resource: vec!["memory_object".into()],
            transitive: vec!["tag".into()],
            actions: vec!["read".into(), "list".into()],
            constraints: vec!["tag_predicate".into()],
            kinds: vec!["memory".into()],
            target_kinds: vec!["memory".into()],
            delegable: false,
            approval: "auto".into(),
        })
        .await
        .unwrap();
    let names: Vec<String> = store
        .client()
        .query("SELECT tool_name FROM type::thing('tool_authority_manifest', $id)")
        .bind(("id", id.to_string()))
        .await
        .unwrap()
        .take((0, "tool_name"))
        .unwrap();
    assert_eq!(names.first().map(String::as_str), Some("read_memory"));

    let actions: Vec<Vec<String>> = store
        .client()
        .query("SELECT actions FROM type::thing('tool_authority_manifest', $id)")
        .bind(("id", id.to_string()))
        .await
        .unwrap()
        .take((0, "actions"))
        .unwrap();
    let got = actions.into_iter().next().expect("actions row");
    assert_eq!(got.len(), 2);
    assert!(got.contains(&"read".to_string()));
    assert!(got.contains(&"list".to_string()));
}

#[tokio::test]
async fn ping_returns_ok_on_fresh_store() {
    let (store, _dir) = fresh_store().await;
    store.ping().await.expect("ping");
}

#[tokio::test]
async fn get_admin_agent_returns_none_before_bootstrap() {
    let (store, _dir) = fresh_store().await;
    let admin = store.get_admin_agent().await.expect("get");
    assert!(admin.is_none());
}

#[tokio::test]
async fn get_admin_agent_finds_human_agent_after_create() {
    let (store, _dir) = fresh_store().await;

    // LLM agents must NOT satisfy the admin query.
    let llm = Agent {
        id: AgentId::new(),
        kind: AgentKind::Llm,
        display_name: "background-bot".into(),
        owning_org: None,
        role: None,
        created_at: Utc::now(),
    };
    store.create_agent(&llm).await.expect("llm");
    assert!(store.get_admin_agent().await.expect("get").is_none());

    // Human agent satisfies the query.
    let human = sample_agent();
    store.create_agent(&human).await.expect("human");
    let admin = store.get_admin_agent().await.expect("get").expect("row");
    assert_eq!(admin.id, human.id);
    assert_eq!(admin.kind, AgentKind::Human);
}

// ---------------------------------------------------------------------------
// Grants
// ---------------------------------------------------------------------------

fn sample_grant(holder: PrincipalRef, action: &str, resource_uri: &str) -> Grant {
    Grant {
        id: GrantId::new(),
        holder,
        action: vec![action.into()],
        resource: ResourceRef {
            uri: resource_uri.into(),
        },
        fundamentals: vec![],
        descends_from: None,
        delegable: false,
        issued_at: Utc::now(),
        revoked_at: None,
    }
}

#[tokio::test]
async fn grant_create_and_get_round_trip() {
    let (store, _dir) = fresh_store().await;
    let holder = PrincipalRef::Agent(AgentId::new());
    let grant = sample_grant(holder.clone(), "read", "memory:m-1");
    store.create_grant(&grant).await.expect("create");

    let got = store.get_grant(grant.id).await.expect("get").expect("row");
    assert_eq!(got.id, grant.id);
    assert_eq!(got.action, vec!["read".to_string()]);
    assert_eq!(got.resource.uri, "memory:m-1");
    match got.holder {
        PrincipalRef::Agent(a) => match holder {
            PrincipalRef::Agent(expected) => assert_eq!(a, expected),
            _ => panic!("wrong holder kind"),
        },
        _ => panic!("wrong holder kind"),
    }
}

#[tokio::test]
async fn grant_revoke_sets_revoked_at() {
    let (store, _dir) = fresh_store().await;
    let grant = sample_grant(PrincipalRef::Agent(AgentId::new()), "read", "memory:m-1");
    store.create_grant(&grant).await.expect("create");
    assert!(store
        .get_grant(grant.id)
        .await
        .expect("get")
        .unwrap()
        .revoked_at
        .is_none());

    let when = Utc::now();
    store.revoke_grant(grant.id, when).await.expect("revoke");

    let after = store.get_grant(grant.id).await.expect("get").unwrap();
    assert!(after.revoked_at.is_some());
}

#[tokio::test]
async fn list_grants_for_principal_returns_only_matching_grants() {
    let (store, _dir) = fresh_store().await;
    let alice = PrincipalRef::Agent(AgentId::new());
    let bob = PrincipalRef::Agent(AgentId::new());
    let org = PrincipalRef::Organization(OrgId::new());

    store
        .create_grant(&sample_grant(alice.clone(), "read", "r1"))
        .await
        .unwrap();
    store
        .create_grant(&sample_grant(alice.clone(), "write", "r2"))
        .await
        .unwrap();
    store
        .create_grant(&sample_grant(bob.clone(), "read", "r3"))
        .await
        .unwrap();
    store
        .create_grant(&sample_grant(org.clone(), "allocate", "r4"))
        .await
        .unwrap();

    let alice_grants = store.list_grants_for_principal(&alice).await.unwrap();
    assert_eq!(alice_grants.len(), 2);

    let bob_grants = store.list_grants_for_principal(&bob).await.unwrap();
    assert_eq!(bob_grants.len(), 1);

    let org_grants = store.list_grants_for_principal(&org).await.unwrap();
    assert_eq!(org_grants.len(), 1);

    let nobody = PrincipalRef::User(UserId::new());
    assert!(store
        .list_grants_for_principal(&nobody)
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn get_grant_returns_none_when_absent() {
    let (store, _dir) = fresh_store().await;
    assert!(store.get_grant(GrantId::new()).await.unwrap().is_none());
}

// ---------------------------------------------------------------------------
// Auth Requests
// ---------------------------------------------------------------------------

fn sample_auth_request(resource_uri: &str) -> AuthRequest {
    AuthRequest {
        id: AuthRequestId::new(),
        requestor: PrincipalRef::System("system:genesis".into()),
        kinds: vec!["auth_request_object".into()],
        scope: vec!["allocate".into()],
        state: AuthRequestState::Pending,
        valid_until: Some(Utc::now() + Duration::days(14)),
        submitted_at: Utc::now(),
        resource_slots: vec![ResourceSlot {
            resource: ResourceRef {
                uri: resource_uri.into(),
            },
            approvers: vec![ApproverSlot {
                approver: PrincipalRef::System("system:genesis".into()),
                state: ApproverSlotState::Unfilled,
                responded_at: None,
                reconsidered_at: None,
            }],
            state: ResourceSlotState::InProgress,
        }],
        justification: Some("unit test".into()),
        audit_class: AuditClass::Alerted,
        terminal_state_entered_at: None,
        archived: false,
        active_window_days: 90,
        provenance_template: None,
    }
}

#[tokio::test]
async fn auth_request_create_and_get_round_trip() {
    let (store, _dir) = fresh_store().await;
    let req = sample_auth_request("system:root");
    store.create_auth_request(&req).await.expect("create");

    let got = store
        .get_auth_request(req.id)
        .await
        .expect("get")
        .expect("row");
    assert_eq!(got.id, req.id);
    assert_eq!(got.state, AuthRequestState::Pending);
    assert_eq!(got.resource_slots.len(), 1);
    assert_eq!(got.resource_slots[0].resource.uri, "system:root");
    assert_eq!(got.justification.as_deref(), Some("unit test"));
}

#[tokio::test]
async fn auth_request_update_replaces_state_and_slots() {
    let (store, _dir) = fresh_store().await;
    let mut req = sample_auth_request("system:root");
    store.create_auth_request(&req).await.unwrap();

    req.state = AuthRequestState::Approved;
    req.resource_slots[0].state = ResourceSlotState::Approved;
    req.resource_slots[0].approvers[0].state = ApproverSlotState::Approved;
    req.resource_slots[0].approvers[0].responded_at = Some(Utc::now());
    req.terminal_state_entered_at = Some(Utc::now());

    store.update_auth_request(&req).await.unwrap();

    let got = store.get_auth_request(req.id).await.unwrap().unwrap();
    assert_eq!(got.state, AuthRequestState::Approved);
    assert_eq!(
        got.resource_slots[0].approvers[0].state,
        ApproverSlotState::Approved
    );
    assert!(got.resource_slots[0].approvers[0].responded_at.is_some());
    assert!(got.terminal_state_entered_at.is_some());
}

#[tokio::test]
async fn list_active_auth_requests_for_resource_matches_uri_and_excludes_archived() {
    let (store, _dir) = fresh_store().await;

    // Two active requests for the target resource, one for a different
    // resource, one archived on the target.
    let target = "system:root";
    let other = "memory:m-1";

    let a1 = sample_auth_request(target);
    let a2 = sample_auth_request(target);
    let a3 = sample_auth_request(other);
    let mut archived = sample_auth_request(target);
    archived.archived = true;

    for r in [&a1, &a2, &a3, &archived] {
        store.create_auth_request(r).await.unwrap();
    }

    let found = store
        .list_active_auth_requests_for_resource(&ResourceRef { uri: target.into() })
        .await
        .unwrap();
    let ids: Vec<_> = found.iter().map(|r| r.id).collect();
    assert_eq!(ids.len(), 2, "expected 2 matches, got {:?}", ids);
    assert!(ids.contains(&a1.id));
    assert!(ids.contains(&a2.id));
}

// ---------------------------------------------------------------------------
// Ownership edges — raw methods + typed free-function wrappers (ADR-0015)
// ---------------------------------------------------------------------------

/// Tiny helper: did a record with the given id exist in the given edge
/// table? Uses `count()` so we don't have to decode the `id: Thing`
/// back through `serde_json::Value` (which can't represent SurrealDB's
/// native record-id type).
async fn edge_exists(store: &SurrealStore, table: &str, id: &str) -> bool {
    let q = format!("SELECT count() FROM type::thing('{table}', $id) GROUP ALL");
    let counts: Vec<i64> = store
        .client()
        .query(&q)
        .bind(("id", id.to_string()))
        .await
        .unwrap()
        .take((0, "count"))
        .unwrap();
    counts.into_iter().next().unwrap_or(0) > 0
}

#[tokio::test]
async fn typed_upsert_ownership_via_wrapper_records_edge() {
    let (store, _dir) = fresh_store().await;
    let memory = MemoryId::new();
    let user = UserId::new();

    // Typed free-function wrapper — compile-time safe.
    let edge_id = repository::upsert_ownership(&store, &memory, &user, None)
        .await
        .expect("ownership");
    assert!(
        edge_exists(&store, "owned_by", &edge_id.to_string()).await,
        "ownership edge must be persisted"
    );
}

#[tokio::test]
async fn typed_upsert_creation_records_edge() {
    let (store, _dir) = fresh_store().await;
    let creator = AgentId::new();
    let memory = MemoryId::new();
    let edge_id = repository::upsert_creation(&store, &creator, &memory)
        .await
        .expect("creation");
    assert!(edge_exists(&store, "created", &edge_id.to_string()).await);
}

#[tokio::test]
async fn typed_upsert_allocation_records_edge_with_provenance() {
    let (store, _dir) = fresh_store().await;
    let from = OrgId::new();
    let to = ProjectId::new();
    let auth_id = AuthRequestId::new();
    let resource = ResourceRef {
        uri: "filesystem:/workspace/project-a/**".into(),
    };

    let edge_id = repository::upsert_allocation(&store, &from, &to, &resource, auth_id)
        .await
        .expect("allocation");

    assert!(edge_exists(&store, "allocated_to", &edge_id.to_string()).await);

    // Check the provenance field landed on the edge row. We can safely
    // project string columns (unlike the `id: Thing` field).
    let uris: Vec<String> = store
        .client()
        .query("SELECT resource_uri FROM type::thing('allocated_to', $id)")
        .bind(("id", edge_id.to_string()))
        .await
        .unwrap()
        .take((0, "resource_uri"))
        .unwrap();
    assert_eq!(
        uris.first().map(String::as_str),
        Some("filesystem:/workspace/project-a/**")
    );

    let auth_refs: Vec<String> = store
        .client()
        .query("SELECT auth_request FROM type::thing('allocated_to', $id)")
        .bind(("id", edge_id.to_string()))
        .await
        .unwrap()
        .take((0, "auth_request"))
        .unwrap();
    assert_eq!(
        auth_refs.first().map(String::as_str),
        Some(auth_id.to_string().as_str())
    );
}

// ---------------------------------------------------------------------------
// Bootstrap credentials
// ---------------------------------------------------------------------------

#[tokio::test]
async fn bootstrap_put_find_consume_full_lifecycle() {
    let (store, _dir) = fresh_store().await;
    let digest = "argon2id$hash-goes-here".to_string();

    let row = store
        .put_bootstrap_credential(digest.clone())
        .await
        .expect("put");
    assert_eq!(row.digest, digest);
    assert!(row.consumed_at.is_none());

    let found = store
        .find_unconsumed_credential(&digest)
        .await
        .expect("find")
        .expect("row");
    assert_eq!(found.record_id, row.record_id);
    assert!(found.consumed_at.is_none());

    store
        .consume_bootstrap_credential(&row.record_id)
        .await
        .expect("consume");

    let after = store
        .find_unconsumed_credential(&digest)
        .await
        .expect("find-after-consume");
    assert!(
        after.is_none(),
        "consumed credentials must not appear in find_unconsumed_credential"
    );
}

#[tokio::test]
async fn bootstrap_find_unknown_digest_returns_none() {
    let (store, _dir) = fresh_store().await;
    let found = store.find_unconsumed_credential("nope").await.unwrap();
    assert!(found.is_none());
}

// ---------------------------------------------------------------------------
// Resources Catalogue
// ---------------------------------------------------------------------------

#[tokio::test]
async fn catalogue_seed_then_contains_hits() {
    let (store, _dir) = fresh_store().await;
    store
        .seed_catalogue_entry(None, "system:root", "control_plane_object")
        .await
        .unwrap();

    assert!(store.catalogue_contains(None, "system:root").await.unwrap());
}

#[tokio::test]
async fn catalogue_contains_misses_cleanly() {
    let (store, _dir) = fresh_store().await;
    assert!(!store.catalogue_contains(None, "never-added").await.unwrap());
}

#[tokio::test]
async fn catalogue_isolates_per_org_scope() {
    let (store, _dir) = fresh_store().await;
    let org_a = OrgId::new();
    let org_b = OrgId::new();

    store
        .seed_catalogue_entry(Some(org_a), "memory_object:all", "composite")
        .await
        .unwrap();

    assert!(store
        .catalogue_contains(Some(org_a), "memory_object:all")
        .await
        .unwrap());
    assert!(
        !store
            .catalogue_contains(Some(org_b), "memory_object:all")
            .await
            .unwrap(),
        "entries seeded under org_a must not be visible under org_b"
    );
    assert!(
        !store
            .catalogue_contains(None, "memory_object:all")
            .await
            .unwrap(),
        "entries seeded under org_a must not be visible at platform scope"
    );
}

// ---------------------------------------------------------------------------
// Audit events
// ---------------------------------------------------------------------------

fn sample_event(org: Option<OrgId>, prev: Option<[u8; 32]>, at_offset_secs: i64) -> AuditEvent {
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: "test.event".into(),
        actor_agent_id: None,
        target_entity_id: None,
        timestamp: Utc::now() + Duration::seconds(at_offset_secs),
        diff: serde_json::json!({}),
        audit_class: AuditClass::Logged,
        provenance_auth_request_id: None,
        org_scope: org,
        prev_event_hash: prev,
    }
}

#[tokio::test]
async fn audit_write_persists_event() {
    let (store, _dir) = fresh_store().await;
    store
        .write_audit_event(&sample_event(None, None, 0))
        .await
        .expect("write");

    // Count direct from SurrealDB.
    let counts: Vec<i64> = store
        .client()
        .query("SELECT count() FROM audit_events GROUP ALL")
        .await
        .unwrap()
        .take((0, "count"))
        .unwrap();
    assert_eq!(counts.first().copied(), Some(1));
}

#[tokio::test]
async fn get_audit_event_roundtrips_every_field() {
    let (store, _dir) = fresh_store().await;
    let org = OrgId::new();
    let actor = AgentId::new();
    let ar = AuthRequestId::new();
    let target = NodeId::new();
    let mut ev = sample_event(Some(org), Some([7u8; 32]), 0);
    ev.event_type = "platform_admin.claimed".into();
    ev.audit_class = AuditClass::Alerted;
    ev.actor_agent_id = Some(actor);
    ev.target_entity_id = Some(target);
    ev.provenance_auth_request_id = Some(ar);
    ev.diff = serde_json::json!({"after": {"id": "x"}});

    store.write_audit_event(&ev).await.unwrap();
    let fetched = store
        .get_audit_event(ev.event_id)
        .await
        .unwrap()
        .expect("event row must exist after write");

    assert_eq!(fetched.event_id, ev.event_id);
    assert_eq!(fetched.event_type, "platform_admin.claimed");
    assert_eq!(fetched.audit_class, AuditClass::Alerted);
    assert_eq!(fetched.actor_agent_id, Some(actor));
    assert_eq!(fetched.target_entity_id, Some(target));
    assert_eq!(fetched.provenance_auth_request_id, Some(ar));
    assert_eq!(fetched.org_scope, Some(org));
    assert_eq!(fetched.prev_event_hash, Some([7u8; 32]));
    assert_eq!(fetched.diff, serde_json::json!({"after": {"id": "x"}}));
    // Timestamps round-trip at RFC3339 precision.
    assert_eq!(fetched.timestamp.to_rfc3339(), ev.timestamp.to_rfc3339());
}

#[tokio::test]
async fn get_audit_event_returns_none_for_missing_id() {
    let (store, _dir) = fresh_store().await;
    assert!(store
        .get_audit_event(AuditEventId::new())
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn last_event_hash_for_empty_org_returns_none() {
    let (store, _dir) = fresh_store().await;
    let out = store.last_event_hash_for_org(None).await.unwrap();
    assert!(out.is_none());
}

#[tokio::test]
async fn last_event_hash_returns_most_recent_hash_per_org() {
    // Contract: `last_event_hash_for_org(org)` returns
    // `hash_event(most_recent_event_in_org)` — the digest the NEXT
    // emit in that org will copy into its `prev_event_hash` to form
    // the chain. Returning the stored `prev_event_hash` would
    // propagate the SECOND-to-last event's hash (i.e. not a chain).
    use domain::audit::hash_event;
    let (store, _dir) = fresh_store().await;
    let org = OrgId::new();
    let other = OrgId::new();

    // Two events for `org`; we re-read the latest one to compute its
    // deterministic hash against the same canonical bytes the lookup
    // path uses.
    store
        .write_audit_event(&sample_event(Some(org), None, 0))
        .await
        .unwrap();
    let mut with_hash = sample_event(Some(org), None, 1);
    with_hash.prev_event_hash = Some([3u8; 32]);
    store.write_audit_event(&with_hash).await.unwrap();
    let stored_latest_org = store
        .get_audit_event(with_hash.event_id)
        .await
        .unwrap()
        .unwrap();

    // Event for a DIFFERENT org; must not bleed into the `org` lookup.
    let mut other_event = sample_event(Some(other), None, 2);
    other_event.prev_event_hash = Some([99u8; 32]);
    store.write_audit_event(&other_event).await.unwrap();
    let stored_other = store
        .get_audit_event(other_event.event_id)
        .await
        .unwrap()
        .unwrap();

    let last_for_org = store.last_event_hash_for_org(Some(org)).await.unwrap();
    assert_eq!(last_for_org, Some(hash_event(&stored_latest_org)));

    let last_for_other = store.last_event_hash_for_org(Some(other)).await.unwrap();
    assert_eq!(last_for_other, Some(hash_event(&stored_other)));

    // Platform scope (org = None) has no events.
    let last_for_platform = store.last_event_hash_for_org(None).await.unwrap();
    assert!(last_for_platform.is_none());
}

// ---------------------------------------------------------------------------
// Guard: Uuid parse round-trip via the domain-level free-function wrappers
// still works even for adversarially-crafted UUIDs (all-zero).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn typed_wrappers_tolerate_all_zero_uuid() {
    let (store, _dir) = fresh_store().await;
    let mem = MemoryId::from_uuid(Uuid::nil());
    let user = UserId::new();
    let edge_id = repository::upsert_ownership(&store, &mem, &user, None)
        .await
        .expect("nil uuid resource");
    assert_ne!(edge_id.to_string(), "");
}

// ---------------------------------------------------------------------------
// Additional coverage — P2 widening (error paths + multi-field round-trips +
// cross-cutting semantics). Added after the P1-P3 independent re-audit
// flagged test breadth as the only outstanding M1 gap.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn create_agent_rejects_duplicate_id() {
    let (store, _dir) = fresh_store().await;
    let agent = sample_agent();
    store.create_agent(&agent).await.expect("first create");

    // Second CREATE with the same id must fail — SurrealDB's implicit
    // record-id uniqueness catches this.
    let err = store
        .create_agent(&agent)
        .await
        .expect_err("duplicate agent id must be rejected");
    // We don't pin the exact error variant — Backend(String) is the
    // expected catch-all for SurrealDB DB errors.
    let msg = format!("{err}");
    assert!(
        msg.contains("backend") || msg.contains("record"),
        "unexpected error shape: {msg}"
    );
}

#[tokio::test]
async fn agent_llm_kind_and_owning_org_roundtrips() {
    let (store, _dir) = fresh_store().await;
    let org_id = OrgId::new();
    let agent = Agent {
        id: AgentId::new(),
        kind: AgentKind::Llm,
        display_name: "claude-coder-7".into(),
        owning_org: Some(org_id),
        role: None,
        created_at: Utc::now(),
    };
    store.create_agent(&agent).await.unwrap();
    let got = store.get_agent(agent.id).await.unwrap().expect("row");
    assert_eq!(got.kind, AgentKind::Llm);
    assert_eq!(got.owning_org, Some(org_id));
    assert_eq!(got.display_name, "claude-coder-7");
}

#[tokio::test]
async fn get_organization_returns_none_when_absent() {
    let (store, _dir) = fresh_store().await;
    let got = store.get_organization(OrgId::new()).await.unwrap();
    assert!(got.is_none());
}

// ---- Grants: holder variants + multi-field shapes ------------------------

#[tokio::test]
async fn grant_with_system_principal_holder_roundtrips() {
    let (store, _dir) = fresh_store().await;
    let holder = PrincipalRef::System("system:genesis".into());
    let grant = sample_grant(holder.clone(), "allocate", "system:root");
    store.create_grant(&grant).await.unwrap();

    let got = store.get_grant(grant.id).await.unwrap().expect("row");
    match got.holder {
        PrincipalRef::System(s) => assert_eq!(s, "system:genesis"),
        other => panic!("expected System holder, got {:?}", other),
    }
    // list_grants_for_principal also accepts System variant.
    let listed = store.list_grants_for_principal(&holder).await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, grant.id);
}

#[tokio::test]
async fn grant_with_project_principal_holder_roundtrips() {
    let (store, _dir) = fresh_store().await;
    let project = PrincipalRef::Project(ProjectId::new());
    let grant = sample_grant(project.clone(), "read", "filesystem_object");
    store.create_grant(&grant).await.unwrap();

    let listed = store.list_grants_for_principal(&project).await.unwrap();
    assert_eq!(listed.len(), 1);
    match &listed[0].holder {
        PrincipalRef::Project(_) => {}
        other => panic!("expected Project holder, got {:?}", other),
    }
}

#[tokio::test]
async fn grant_with_multiple_actions_roundtrips() {
    let (store, _dir) = fresh_store().await;
    let holder = PrincipalRef::Agent(AgentId::new());
    let mut grant = sample_grant(holder, "read", "filesystem_object");
    grant.action = vec![
        "read".into(),
        "list".into(),
        "inspect".into(),
        "observe".into(),
    ];

    store.create_grant(&grant).await.unwrap();
    let got = store.get_grant(grant.id).await.unwrap().expect("row");
    assert_eq!(got.action.len(), 4);
    assert_eq!(got.action[0], "read");
    assert_eq!(got.action[3], "observe");
}

#[tokio::test]
async fn grant_with_descends_from_auth_request_roundtrips() {
    let (store, _dir) = fresh_store().await;
    let holder = PrincipalRef::Agent(AgentId::new());
    let ar = AuthRequestId::new();
    let mut grant = sample_grant(holder, "read", "memory_object");
    grant.descends_from = Some(ar);
    grant.delegable = true;

    store.create_grant(&grant).await.unwrap();
    let got = store.get_grant(grant.id).await.unwrap().expect("row");
    assert_eq!(got.descends_from, Some(ar));
    assert!(got.delegable);
}

#[tokio::test]
async fn revoked_grant_still_returned_by_get_grant() {
    // Revoked grants are audit-visible — they don't disappear. Pinning
    // the current contract; Step 2 of the Permission Check is what
    // excludes them, not the repository.
    let (store, _dir) = fresh_store().await;
    let grant = sample_grant(PrincipalRef::Agent(AgentId::new()), "read", "memory:m-1");
    store.create_grant(&grant).await.unwrap();
    let when = Utc::now();
    store.revoke_grant(grant.id, when).await.unwrap();

    let got = store.get_grant(grant.id).await.unwrap().expect("row");
    assert!(got.revoked_at.is_some());
    assert_eq!(got.id, grant.id);
}

#[tokio::test]
async fn revoke_grant_on_missing_id_is_noop() {
    // Pinning SurrealDB UPDATE semantics: updating a record that doesn't
    // exist is a no-op (zero rows affected, no error). Bootstrap flow (P5)
    // assumes this so idempotent cleanup doesn't panic.
    let (store, _dir) = fresh_store().await;
    let when = Utc::now();
    let result = store.revoke_grant(GrantId::new(), when).await;
    assert!(
        result.is_ok(),
        "revoke of missing id must be silent: {:?}",
        result
    );
}

#[tokio::test]
async fn revoke_grant_is_idempotent_with_later_timestamp_winning() {
    let (store, _dir) = fresh_store().await;
    let grant = sample_grant(PrincipalRef::Agent(AgentId::new()), "read", "memory:m-1");
    store.create_grant(&grant).await.unwrap();

    let t1 = Utc::now();
    store.revoke_grant(grant.id, t1).await.unwrap();
    let t2 = t1 + Duration::seconds(10);
    store.revoke_grant(grant.id, t2).await.unwrap();

    let got = store.get_grant(grant.id).await.unwrap().expect("row");
    let revoked = got.revoked_at.expect("revoked_at set");
    assert!(revoked >= t1);
}

// ---- Auth Requests: shape variations + error paths -----------------------

#[tokio::test]
async fn get_auth_request_returns_none_when_absent() {
    let (store, _dir) = fresh_store().await;
    let got = store.get_auth_request(AuthRequestId::new()).await.unwrap();
    assert!(got.is_none());
}

#[tokio::test]
async fn auth_request_with_multiple_slots_and_approvers_roundtrips() {
    let (store, _dir) = fresh_store().await;
    let mut req = sample_auth_request("r1");
    // Add a second resource slot with two approvers.
    req.resource_slots.push(ResourceSlot {
        resource: ResourceRef { uri: "r2".into() },
        approvers: vec![
            ApproverSlot {
                approver: PrincipalRef::Agent(AgentId::new()),
                state: ApproverSlotState::Approved,
                responded_at: Some(Utc::now()),
                reconsidered_at: None,
            },
            ApproverSlot {
                approver: PrincipalRef::Organization(OrgId::new()),
                state: ApproverSlotState::Denied,
                responded_at: Some(Utc::now()),
                reconsidered_at: Some(Utc::now()),
            },
        ],
        state: ResourceSlotState::Partial,
    });

    store.create_auth_request(&req).await.unwrap();
    let got = store.get_auth_request(req.id).await.unwrap().expect("row");
    assert_eq!(got.resource_slots.len(), 2);
    let slot2 = &got.resource_slots[1];
    assert_eq!(slot2.resource.uri, "r2");
    assert_eq!(slot2.state, ResourceSlotState::Partial);
    assert_eq!(slot2.approvers.len(), 2);
    assert_eq!(slot2.approvers[0].state, ApproverSlotState::Approved);
    assert_eq!(slot2.approvers[1].state, ApproverSlotState::Denied);
    assert!(slot2.approvers[1].reconsidered_at.is_some());
}

#[tokio::test]
async fn update_auth_request_on_missing_row_is_noop() {
    // Pinning UPDATE semantics — consistent with revoke_grant and
    // consume_bootstrap_credential; P4's state machine depends on it.
    let (store, _dir) = fresh_store().await;
    let req = sample_auth_request("system:root");
    // No prior CREATE.
    let result = store.update_auth_request(&req).await;
    assert!(
        result.is_ok(),
        "update of missing row must be silent: {:?}",
        result
    );
    // And no row was created.
    let got = store.get_auth_request(req.id).await.unwrap();
    assert!(got.is_none());
}

#[tokio::test]
async fn list_active_auth_requests_returns_empty_when_no_match() {
    let (store, _dir) = fresh_store().await;
    store
        .create_auth_request(&sample_auth_request("memory:m-1"))
        .await
        .unwrap();
    let got = store
        .list_active_auth_requests_for_resource(&ResourceRef {
            uri: "nonexistent".into(),
        })
        .await
        .unwrap();
    assert!(got.is_empty());
}

#[tokio::test]
async fn list_active_auth_requests_drops_request_after_archive_flip() {
    let (store, _dir) = fresh_store().await;
    let mut req = sample_auth_request("system:root");
    store.create_auth_request(&req).await.unwrap();
    // Starts active.
    let active = store
        .list_active_auth_requests_for_resource(&ResourceRef {
            uri: "system:root".into(),
        })
        .await
        .unwrap();
    assert_eq!(active.len(), 1);

    // Flip to archived via update.
    req.archived = true;
    store.update_auth_request(&req).await.unwrap();
    let after = store
        .list_active_auth_requests_for_resource(&ResourceRef {
            uri: "system:root".into(),
        })
        .await
        .unwrap();
    assert!(
        after.is_empty(),
        "archived=true must remove row from active listing"
    );
}

// ---- Ownership edges: provenance + idempotency ---------------------------

#[tokio::test]
async fn upsert_ownership_with_auth_request_stores_provenance() {
    let (store, _dir) = fresh_store().await;
    let mem = MemoryId::new();
    let user = UserId::new();
    let ar = AuthRequestId::new();
    let edge_id = repository::upsert_ownership(&store, &mem, &user, Some(ar))
        .await
        .unwrap();

    let provenance: Vec<String> = store
        .client()
        .query("SELECT auth_request FROM type::thing('owned_by', $id)")
        .bind(("id", edge_id.to_string()))
        .await
        .unwrap()
        .take((0, "auth_request"))
        .unwrap();
    assert_eq!(
        provenance.first().map(String::as_str),
        Some(ar.to_string().as_str())
    );
}

#[tokio::test]
async fn upsert_edges_produce_distinct_ids_on_repeated_calls() {
    // Pinning current semantics: "upsert" is additive — each call inserts
    // a new edge row. De-duplication (true upsert) is an M2+ concern per
    // ADR-0015 §Alternatives. Callers that need dedup check existence
    // first.
    let (store, _dir) = fresh_store().await;
    let mem = MemoryId::new();
    let user = UserId::new();
    let e1 = repository::upsert_ownership(&store, &mem, &user, None)
        .await
        .unwrap();
    let e2 = repository::upsert_ownership(&store, &mem, &user, None)
        .await
        .unwrap();
    assert_ne!(e1, e2, "each upsert creates a fresh edge row");
    assert!(edge_exists(&store, "owned_by", &e1.to_string()).await);
    assert!(edge_exists(&store, "owned_by", &e2.to_string()).await);
}

// ---- Bootstrap credentials: missing + duplicate handling -----------------

#[tokio::test]
async fn consume_missing_credential_is_noop() {
    let (store, _dir) = fresh_store().await;
    // Using an ID that does not exist.
    let result = store
        .consume_bootstrap_credential("bootstrap_credentials:nonexistent")
        .await;
    assert!(
        result.is_ok(),
        "consume of missing id must be silent: {:?}",
        result
    );
}

#[tokio::test]
async fn put_multiple_credentials_are_independent() {
    let (store, _dir) = fresh_store().await;
    let a = store
        .put_bootstrap_credential("digest-a".into())
        .await
        .unwrap();
    let b = store
        .put_bootstrap_credential("digest-b".into())
        .await
        .unwrap();
    assert_ne!(a.record_id, b.record_id);

    // Consuming one must not affect the other.
    store
        .consume_bootstrap_credential(&a.record_id)
        .await
        .unwrap();
    assert!(store
        .find_unconsumed_credential("digest-a")
        .await
        .unwrap()
        .is_none());
    let still_b = store.find_unconsumed_credential("digest-b").await.unwrap();
    assert!(still_b.is_some());
    assert_eq!(still_b.unwrap().record_id, b.record_id);
}

// ---- Resources catalogue: semantic edge cases ----------------------------

#[tokio::test]
async fn catalogue_contains_is_case_sensitive() {
    let (store, _dir) = fresh_store().await;
    store
        .seed_catalogue_entry(None, "System:Root", "control_plane_object")
        .await
        .unwrap();
    assert!(store.catalogue_contains(None, "System:Root").await.unwrap());
    assert!(!store.catalogue_contains(None, "system:root").await.unwrap());
}

#[tokio::test]
async fn catalogue_records_kind_metadata_column() {
    let (store, _dir) = fresh_store().await;
    let org = OrgId::new();
    store
        .seed_catalogue_entry(Some(org), "memory:m-42", "memory_object")
        .await
        .unwrap();

    // Project the `kind` column directly to prove it landed.
    let kinds: Vec<String> = store
        .client()
        .query(
            "SELECT kind FROM resources_catalogue \
             WHERE owning_org = $org AND resource_uri = $uri",
        )
        .bind(("org", org.to_string()))
        .bind(("uri", "memory:m-42".to_string()))
        .await
        .unwrap()
        .take((0, "kind"))
        .unwrap();
    assert_eq!(kinds.first().map(String::as_str), Some("memory_object"));
}

// ---- Audit events: full-field round-trip + chain reset across orgs -------

#[tokio::test]
async fn audit_event_full_field_round_trip() {
    // Round-trip every optional field on the audit_events schema to prove
    // the serializer's shape matches what the hash-chain proptest will
    // eventually read back.
    let (store, _dir) = fresh_store().await;
    let org = OrgId::new();
    let actor = AgentId::new();
    let target_node = NodeId::new();
    let ar = AuthRequestId::new();
    let mut ev = sample_event(Some(org), Some([9u8; 32]), 0);
    ev.actor_agent_id = Some(actor);
    ev.target_entity_id = Some(target_node);
    ev.provenance_auth_request_id = Some(ar);
    ev.event_type = "grant.issued".into();
    ev.diff = serde_json::json!({"before": null, "after": {"grant_id": "g-1"}});
    ev.audit_class = AuditClass::Alerted;

    store.write_audit_event(&ev).await.unwrap();

    // `last_event_hash_for_org` returns `hash_event(latest)` — the
    // digest the next event would copy as its `prev_event_hash` to
    // form a chain. Re-read from the store so we compute against the
    // same canonical bytes the lookup path sees.
    use domain::audit::hash_event;
    let stored = store.get_audit_event(ev.event_id).await.unwrap().unwrap();
    let last = store.last_event_hash_for_org(Some(org)).await.unwrap();
    assert_eq!(last, Some(hash_event(&stored)));
}

#[tokio::test]
async fn audit_last_event_hash_isolated_between_org_and_platform_scope() {
    use domain::audit::hash_event;
    let (store, _dir) = fresh_store().await;
    let org = OrgId::new();

    // Platform-scope event first.
    let mut platform = sample_event(None, Some([7u8; 32]), 0);
    platform.event_type = "platform.alert".into();
    store.write_audit_event(&platform).await.unwrap();
    let stored_platform = store
        .get_audit_event(platform.event_id)
        .await
        .unwrap()
        .unwrap();

    // Then an org-scope event with a different hash.
    let mut org_event = sample_event(Some(org), Some([11u8; 32]), 1);
    org_event.event_type = "org.created".into();
    store.write_audit_event(&org_event).await.unwrap();
    let stored_org = store
        .get_audit_event(org_event.event_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(
        store.last_event_hash_for_org(None).await.unwrap(),
        Some(hash_event(&stored_platform)),
        "platform scope must see its own last event's hash"
    );
    assert_eq!(
        store.last_event_hash_for_org(Some(org)).await.unwrap(),
        Some(hash_event(&stored_org)),
        "org scope must see its own last event's hash"
    );
}

// ---------------------------------------------------------------------------
// Bootstrap claim — atomic s01 flow (P5)
// ---------------------------------------------------------------------------

fn claim_entities() -> (AgentId, NodeId, NodeId, NodeId, AuthRequestId, GrantId) {
    (
        AgentId::new(),
        NodeId::new(),
        NodeId::new(),
        NodeId::new(),
        AuthRequestId::new(),
        GrantId::new(),
    )
}

fn bootstrap_claim_for(
    credential_record_id: &str,
    agent_id: AgentId,
    channel_id: NodeId,
    inbox_id: NodeId,
    outbox_id: NodeId,
    auth_request_id: AuthRequestId,
    grant_id: GrantId,
) -> domain::repository::BootstrapClaim {
    let now = Utc::now();
    domain::repository::BootstrapClaim {
        credential_record_id: credential_record_id.to_string(),
        human_agent: Agent {
            id: agent_id,
            kind: AgentKind::Human,
            display_name: "Alex Chen".into(),
            owning_org: None,
            role: None,
            created_at: now,
        },
        channel: Channel {
            id: channel_id,
            agent_id,
            kind: ChannelKind::Slack,
            handle: "@alex".into(),
            created_at: now,
        },
        inbox: InboxObject {
            id: inbox_id,
            agent_id,
            created_at: now,
        },
        outbox: OutboxObject {
            id: outbox_id,
            agent_id,
            created_at: now,
        },
        auth_request: AuthRequest {
            id: auth_request_id,
            requestor: PrincipalRef::System("system:genesis".into()),
            kinds: vec!["control_plane_object".into()],
            scope: vec!["allocate".into()],
            state: AuthRequestState::Approved,
            valid_until: None,
            submitted_at: now,
            resource_slots: vec![ResourceSlot {
                resource: ResourceRef {
                    uri: "system:root".into(),
                },
                approvers: vec![ApproverSlot {
                    approver: PrincipalRef::System("system:genesis".into()),
                    state: ApproverSlotState::Approved,
                    responded_at: Some(now),
                    reconsidered_at: None,
                }],
                state: ResourceSlotState::Approved,
            }],
            justification: Some("bootstrap".into()),
            audit_class: AuditClass::Alerted,
            terminal_state_entered_at: Some(now),
            archived: false,
            active_window_days: 3650,
            provenance_template: Some(TemplateId::from_uuid(Uuid::nil())),
        },
        grant: Grant {
            id: grant_id,
            holder: PrincipalRef::Agent(agent_id),
            action: vec!["allocate".into()],
            resource: ResourceRef {
                uri: "system:root".into(),
            },
            fundamentals: vec![],
            descends_from: Some(auth_request_id),
            delegable: true,
            issued_at: now,
            revoked_at: None,
        },
        catalogue_entries: vec![
            (
                "system:root".to_string(),
                "control_plane_object".to_string(),
            ),
            (format!("inbox:{}", inbox_id), "inbox_object".to_string()),
        ],
        audit_event: AuditEvent {
            event_id: AuditEventId::new(),
            event_type: "platform_admin.claimed".into(),
            actor_agent_id: Some(agent_id),
            target_entity_id: Some(NodeId::from_uuid(*agent_id.as_uuid())),
            timestamp: now,
            diff: serde_json::json!({"after": {"display_name": "Alex Chen"}}),
            audit_class: AuditClass::Alerted,
            provenance_auth_request_id: Some(auth_request_id),
            org_scope: None,
            prev_event_hash: None,
        },
    }
}

#[tokio::test]
async fn apply_bootstrap_claim_happy_path_commits_every_entity() {
    let (store, _dir) = fresh_store().await;
    let cred = store
        .put_bootstrap_credential("argon2-digest".into())
        .await
        .unwrap();
    let (agent_id, channel_id, inbox_id, outbox_id, ar_id, grant_id) = claim_entities();
    let claim = bootstrap_claim_for(
        &cred.record_id,
        agent_id,
        channel_id,
        inbox_id,
        outbox_id,
        ar_id,
        grant_id,
    );

    store.apply_bootstrap_claim(&claim).await.expect("commit");

    // Every entity is persisted.
    assert!(store.get_agent(agent_id).await.unwrap().is_some());
    assert!(store.get_grant(grant_id).await.unwrap().is_some());
    assert!(store.get_auth_request(ar_id).await.unwrap().is_some());
    assert!(store.catalogue_contains(None, "system:root").await.unwrap());
    // The credential has been marked consumed.
    let cred_after = store
        .find_unconsumed_credential("argon2-digest")
        .await
        .unwrap();
    assert!(
        cred_after.is_none(),
        "credential must be consumed after successful claim"
    );
    // Admin lookup finds the new human agent.
    let admin = store.get_admin_agent().await.unwrap().unwrap();
    assert_eq!(admin.id, agent_id);
}

#[tokio::test]
async fn apply_bootstrap_claim_is_idempotent_failure_when_agent_id_collides() {
    // Pinning atomicity: if a duplicate-id inside the transaction fires a
    // SurrealDB error, the whole batch rolls back — no partial state
    // survives, and the credential stays unconsumed for retry.
    let (store, _dir) = fresh_store().await;
    let cred = store
        .put_bootstrap_credential("argon2-collide".into())
        .await
        .unwrap();

    let (agent_id, channel_id, inbox_id, outbox_id, ar_id, grant_id) = claim_entities();

    // Pre-create the agent so the transaction's CREATE will collide.
    store
        .create_agent(&Agent {
            id: agent_id,
            kind: AgentKind::Human,
            display_name: "Pre-Existing".into(),
            owning_org: None,
            role: None,
            created_at: Utc::now(),
        })
        .await
        .unwrap();

    let claim = bootstrap_claim_for(
        &cred.record_id,
        agent_id,
        channel_id,
        inbox_id,
        outbox_id,
        ar_id,
        grant_id,
    );

    let err = store
        .apply_bootstrap_claim(&claim)
        .await
        .expect_err("must fail on agent-id collision");
    drop(err);

    // Rollback guarantees: the grant / auth request / new catalogue
    // entries / audit event did NOT land, and the credential is STILL
    // unconsumed so the admin can retry with a fresh agent id.
    assert!(store.get_grant(grant_id).await.unwrap().is_none());
    assert!(store.get_auth_request(ar_id).await.unwrap().is_none());
    let cred_after = store
        .find_unconsumed_credential("argon2-collide")
        .await
        .unwrap();
    assert!(
        cred_after.is_some(),
        "credential must remain unconsumed after failed claim"
    );
    // Catalogue was untouched.
    assert!(
        !store.catalogue_contains(None, "system:root").await.unwrap(),
        "catalogue must not gain entries when the transaction rolls back"
    );
}

#[tokio::test]
async fn list_bootstrap_credentials_returns_all_when_unconsumed_only_false() {
    let (store, _dir) = fresh_store().await;
    let _a = store
        .put_bootstrap_credential("digest-1".into())
        .await
        .unwrap();
    let b = store
        .put_bootstrap_credential("digest-2".into())
        .await
        .unwrap();
    store
        .consume_bootstrap_credential(&b.record_id)
        .await
        .unwrap();

    let all = store.list_bootstrap_credentials(false).await.unwrap();
    assert_eq!(all.len(), 2);
    let unconsumed = store.list_bootstrap_credentials(true).await.unwrap();
    assert_eq!(unconsumed.len(), 1);
    assert_eq!(unconsumed[0].digest, "digest-1");
}
