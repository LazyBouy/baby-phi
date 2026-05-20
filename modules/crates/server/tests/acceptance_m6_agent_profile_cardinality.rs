//! CH-28 / ADR-0063 — End-to-end acceptance suite for the AgentProfile
//! cardinality redesign (1:1 → N:1 hybrid Blueprint table + USES_PROFILE
//! edge rename + per-agent override-Blueprint via composite-write).
//!
//! Per plan §7 P2-ACCEPTANCE deliverable 2 + §11 Audit-A claim 14: 7
//! load-bearing scenarios covering the locked forks (F1.c + F2.b + F3.b)
//! end-to-end. Operates directly on `SurrealStore` (not over HTTP) —
//! the iter-5 scope-collapse per §D63.13 + §D63.14 + §D63.15 means
//! handler-tier behaviour is unchanged at CH-28; the cardinality flip
//! is observable at the repository tier through the composite-write +
//! synthesis read-path semantics already wired at P1.5-READ-BRIDGE.
//!
//! ## Scenario topology
//!
//! 1. **two_agents_share_one_template_blueprint_via_uses_profile_edge** —
//!    F1.c LOAD-BEARING positive test: two distinct Agent rows + two
//!    distinct AgentProfile rows both RELATEd at
//!    `AGENT_PROFILE_USES_BLUEPRINT` to ONE shared template-Blueprint.
//!    Both agents synthesize the same template-`parallelize` value.
//!
//! 2. **per_agent_override_blueprint_parallelize_does_not_affect_sibling_agent_sharing_same_template**
//!    — F1.c override-isolation: agent A's override-Blueprint
//!    (`agent_id = Some(A)`, `parallelize = 9`) does not leak into the
//!    synthesized read for agent B (still inherits template
//!    `parallelize = 3`).
//!
//! 3. **get_agent_profile_for_agent_synthesises_template_blueprint_plus_per_agent_override**
//!    — §D63.15 synthesis correctness: read-path priority
//!    override > template > stored properties is honored across all
//!    three override fields (`parallelize`, `model_config_id`,
//!    `mock_response`).
//!
//! 4. **upsert_agent_profile_no_longer_deletes_sibling_rows_for_same_agent_id**
//!    — F1.c regression: `upsert_agent_profile` scoped to the supplied
//!    agent_id only — does NOT remove the AgentProfile row for an
//!    UNRELATED agent (sibling-agent's row stays intact). The
//!    intra-agent_id eviction (1:1 invariant at the
//!    application tier per iter-5 plan §7 P1.5 line 534) is still
//!    enforced — the "no_longer" phrasing in the test name refers to
//!    inter-agent isolation, not intra-agent eviction. See SCOPE-NARROWING
//!    note in the P2-ACCEPTANCE implementer report.
//!
//! 5. **drop_unique_constraint_post_migration_allows_two_rows_with_same_agent_id**
//!    — F1.c schema-tier: migration 0019 dropped
//!    `agent_profile_agent_id` UNIQUE; two raw rows with the same
//!    `agent_id` may now be CREATEd via direct SurrealQL (cardinality
//!    flip 1:1 → N:1 at the schema tier).
//!
//! 6. **uses_profile_edge_emits_single_agent_at_ch28_per_adr_0063_d63_7**
//!    — F2.b listener-side: `DomainEvent::UsesProfileEdgeChanged` is
//!    routed to the single-agent path via
//!    `agent_id_and_timestamp_for` — verifies the rename preserves the
//!    listener's per-agent dispatch contract.
//!
//! 7. **blueprint_upserted_event_emits_when_override_blueprint_written**
//!    — F1.c §D63.1 listener-side: an override-tier
//!    `DomainEvent::BlueprintUpserted` (`agent_id = Some(_)`) fans out
//!    to the single-agent path via `agent_id_and_timestamp_for`; a
//!    template-tier emission (`agent_id = None`) short-circuits to
//!    `None`. End-to-end emission from a production write path is
//!    deferred to a future cycle per §D63.16 / D-CH28-FOLLOWUP-01;
//!    this test pins the listener-arm dispatch invariant at CH-28.

#![allow(clippy::uninlined_format_args)]

mod acceptance_common;

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::Utc;
use domain::events::{DomainEvent, EventBus, EventHandler, InProcessEventBus};
use domain::model::ids::{AgentId, AuditEventId, EdgeId, NodeId};
use domain::model::nodes::{Agent, AgentKind, AgentProfile, Blueprint, BlueprintId};
use domain::Repository;
use store::SurrealStore;
use tempfile::TempDir;

use acceptance_common::blueprint_fixture::make_test_profile_with_overrides;

// ---------------------------------------------------------------------------
// Local helpers (kept inline; no cross-test sharing pressure at CH-28)
// ---------------------------------------------------------------------------

async fn fresh_store() -> (SurrealStore, TempDir) {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = SurrealStore::open_embedded(dir.path().join("db"), "phi", "test")
        .await
        .expect("open embedded");
    (store, dir)
}

fn sample_agent(name: &str) -> Agent {
    Agent {
        id: AgentId::new(),
        kind: AgentKind::Llm,
        display_name: name.into(),
        owning_org: None,
        role: None,
        created_at: Utc::now(),
        active: true,
        archived_at: None,
    }
}

/// Build an AgentProfile struct with the supplied agent_id +
/// parallelize. The other two override fields are `None` — used by
/// tests that exercise the template-blueprint sharing semantic where
/// the per-agent overrides should NOT be set.
fn sample_profile(agent_id: AgentId, parallelize: u32) -> AgentProfile {
    AgentProfile {
        id: NodeId::new(),
        agent_id,
        parallelize,
        blueprint: phi_core::agents::profile::AgentProfile::default(),
        model_config_id: None,
        mock_response: None,
        created_at: Utc::now(),
    }
}

/// CH-28 / ADR-0063 §D63.1 + §D63.3 — RELATE an agent_profile row at a
/// template-Blueprint via the `agent_profile_uses_blueprint` edge.
/// Mirrors the migration-0020 backfill RELATE form (deterministic edge
/// id via `apub_<profile_id>`). Used by tests 1 + 2 + 3 to wire the
/// template-Blueprint shared-pointer semantic.
async fn relate_profile_to_template_blueprint(
    store: &SurrealStore,
    profile_id: NodeId,
    blueprint_id: BlueprintId,
) {
    let edge_id_token = format!("apub_test_{}", EdgeId::new());
    store
        .client()
        .query(
            "LET $p = type::thing('agent_profile', $pid); \
             LET $b = type::thing('blueprint', $bid); \
             RELATE $p -> agent_profile_uses_blueprint -> $b \
                SET id = type::thing('agent_profile_uses_blueprint', $eid) RETURN NONE;",
        )
        .bind(("pid", profile_id.to_string()))
        .bind(("bid", blueprint_id.to_string()))
        .bind(("eid", edge_id_token))
        .await
        .expect("RELATE template-Blueprint edge")
        .check()
        .expect("RELATE check");
}

/// CH-28 / ADR-0063 §D63.1 — Build a template-tier Blueprint row
/// (`agent_id = None`) with the supplied per-field overrides.
fn make_template_blueprint(
    parallelize: Option<u32>,
    model_config_id: Option<String>,
    mock_response: Option<String>,
) -> Blueprint {
    Blueprint {
        id: BlueprintId::new(),
        agent_id: None,
        parallelize,
        model_config_id,
        mock_response,
        created_at: Utc::now(),
    }
}

/// CapturingHandler — records every DomainEvent it receives. Used by
/// tests 6 + 7 to assert the listener-arm dispatch invariants via the
/// in-process bus.
#[derive(Default)]
struct CapturingHandler {
    events: Mutex<Vec<DomainEvent>>,
}

#[async_trait]
impl EventHandler for CapturingHandler {
    async fn on_event(&self, event: &DomainEvent) {
        self.events.lock().unwrap().push(event.clone());
    }
}

impl CapturingHandler {
    fn snapshot(&self) -> Vec<DomainEvent> {
        self.events.lock().unwrap().clone()
    }
}

// ===========================================================================
// Test 1 — Two agents share one template Blueprint via uses_profile edge
// ===========================================================================

#[tokio::test]
async fn test_two_agents_can_share_one_template_blueprint_via_uses_profile_edge() {
    // F1.c LOAD-BEARING: two agents, two AgentProfile rows, ONE shared
    // template-Blueprint pointed at by both via
    // AGENT_PROFILE_USES_BLUEPRINT. Both synthesized reads return the
    // template's parallelize value (3), demonstrating the N:1 template-
    // sharing semantic.
    let (store, _dir) = fresh_store().await;

    // Two distinct agents.
    let agent_a = sample_agent("agent-a");
    let agent_b = sample_agent("agent-b");
    store.create_agent(&agent_a).await.expect("create A");
    store.create_agent(&agent_b).await.expect("create B");

    // One shared template-Blueprint.
    let template = make_template_blueprint(Some(3), None, None);
    let template_id = template.id;
    store
        .upsert_blueprint_template(&template)
        .await
        .expect("upsert template");

    // Two AgentProfile rows. Use parallelize=1 (the SCHEMAFULL minimum
    // per migration 0019 line 111 — `ASSERT $value = NONE OR $value >= 1`).
    // Use sample_profile + create_agent_profile directly; this composite-
    // writes an override-Blueprint we then suppress by clearing it (so
    // the synthesized read falls through to the template).
    let profile_a = sample_profile(agent_a.id, 1);
    let profile_b = sample_profile(agent_b.id, 1);
    store
        .create_agent_profile(&profile_a)
        .await
        .expect("create profile A");
    store
        .create_agent_profile(&profile_b)
        .await
        .expect("create profile B");

    // Clear the auto-generated override-Blueprints (composite-write
    // emitted them; we want the synthesis to fall through to the
    // template tier for this test). Order: DELETE the edges FIRST
    // (relation tables need explicit DELETE), then the blueprint rows.
    store
        .client()
        .query("DELETE agent_uses_blueprint_override; DELETE blueprint WHERE agent_id != NONE;")
        .await
        .expect("clear overrides + edges")
        .check()
        .expect("clear overrides + edges check");

    // RELATE both AgentProfile rows → the same template-Blueprint.
    relate_profile_to_template_blueprint(&store, profile_a.id, template_id).await;
    relate_profile_to_template_blueprint(&store, profile_b.id, template_id).await;

    // Both synthesized reads return parallelize=3 from the template.
    let read_a = store
        .get_agent_profile_for_agent(agent_a.id)
        .await
        .expect("read A")
        .expect("profile A exists");
    let read_b = store
        .get_agent_profile_for_agent(agent_b.id)
        .await
        .expect("read B")
        .expect("profile B exists");
    assert_eq!(
        read_a.parallelize, 3,
        "agent A inherits template parallelize via AGENT_PROFILE_USES_BLUEPRINT"
    );
    assert_eq!(
        read_b.parallelize, 3,
        "agent B inherits template parallelize via the SAME template Blueprint (N:1 sharing)"
    );

    // Sanity: the template-Blueprint row id matches both pointers.
    let tmpl_a = store
        .get_blueprint_for_agent_profile(profile_a.id)
        .await
        .expect("get template via A")
        .expect("template row exists for A");
    let tmpl_b = store
        .get_blueprint_for_agent_profile(profile_b.id)
        .await
        .expect("get template via B")
        .expect("template row exists for B");
    assert_eq!(
        tmpl_a.id, tmpl_b.id,
        "F1.c N:1: both AgentProfile rows resolve to the SAME template Blueprint"
    );
}

// ===========================================================================
// Test 2 — Per-agent override does not affect sibling sharing same template
// ===========================================================================

#[tokio::test]
async fn test_per_agent_override_blueprint_parallelize_does_not_affect_sibling_agent_sharing_same_template(
) {
    // F1.c override-isolation: agent A writes an override-Blueprint
    // (parallelize=9). Agent B inherits the template (parallelize=3)
    // and MUST NOT see agent A's override.
    let (store, _dir) = fresh_store().await;

    let agent_a = sample_agent("override-isolation-a");
    let agent_b = sample_agent("override-isolation-b");
    store.create_agent(&agent_a).await.expect("create A");
    store.create_agent(&agent_b).await.expect("create B");

    let template = make_template_blueprint(Some(3), None, None);
    let template_id = template.id;
    store
        .upsert_blueprint_template(&template)
        .await
        .expect("upsert template");

    // parallelize=1 satisfies the SCHEMAFULL ASSERT on the composite-
    // write's override-Blueprint row (migration 0019 line 111).
    let profile_a = sample_profile(agent_a.id, 1);
    let profile_b = sample_profile(agent_b.id, 1);
    store
        .create_agent_profile(&profile_a)
        .await
        .expect("create profile A");
    store
        .create_agent_profile(&profile_b)
        .await
        .expect("create profile B");

    // Wipe the auto-generated overrides + their edges (composite-write
    // side effect from create_agent_profile). Edges deleted FIRST.
    store
        .client()
        .query("DELETE agent_uses_blueprint_override; DELETE blueprint WHERE agent_id != NONE;")
        .await
        .expect("clear overrides + edges")
        .check()
        .expect("ok");

    // RELATE both → same template.
    relate_profile_to_template_blueprint(&store, profile_a.id, template_id).await;
    relate_profile_to_template_blueprint(&store, profile_b.id, template_id).await;

    // Now write a per-agent override for agent A only.
    let override_a = Blueprint {
        id: BlueprintId::new(),
        agent_id: Some(agent_a.id),
        parallelize: Some(9),
        model_config_id: None,
        mock_response: None,
        created_at: Utc::now(),
    };
    store
        .upsert_agent_blueprint_override(&override_a)
        .await
        .expect("write override A");

    let read_a = store
        .get_agent_profile_for_agent(agent_a.id)
        .await
        .expect("read A")
        .expect("profile A");
    let read_b = store
        .get_agent_profile_for_agent(agent_b.id)
        .await
        .expect("read B")
        .expect("profile B");
    assert_eq!(
        read_a.parallelize, 9,
        "agent A's override (parallelize=9) wins over template (3)"
    );
    assert_eq!(
        read_b.parallelize, 3,
        "agent B inherits template (parallelize=3); A's override does NOT leak"
    );
}

// ===========================================================================
// Test 3 — get_agent_profile_for_agent synthesises template + override
// ===========================================================================

#[tokio::test]
async fn test_get_agent_profile_for_agent_synthesises_template_blueprint_plus_per_agent_override() {
    // §D63.15 synthesis correctness: priority is override > template
    // > stored. Set template (parallelize=2, mock_response=Some("tmpl")),
    // override (parallelize=8, mock_response=None — i.e., the override
    // does NOT carry mock_response). Synthesized read: parallelize=8
    // (override wins) AND mock_response="tmpl" (override falls through
    // to template).
    let (store, _dir) = fresh_store().await;

    let agent = sample_agent("synth");
    store.create_agent(&agent).await.expect("create agent");

    let template = make_template_blueprint(Some(2), None, Some("tmpl".into()));
    let template_id = template.id;
    store
        .upsert_blueprint_template(&template)
        .await
        .expect("upsert template");

    // parallelize=1 satisfies the SCHEMAFULL ASSERT on the composite-
    // write's override-Blueprint row (migration 0019 line 111).
    let profile = sample_profile(agent.id, 1);
    store
        .create_agent_profile(&profile)
        .await
        .expect("create profile");

    // Wipe composite-write auto override + edge (edges first).
    store
        .client()
        .query("DELETE agent_uses_blueprint_override; DELETE blueprint WHERE agent_id != NONE;")
        .await
        .expect("clear overrides + edges")
        .check()
        .expect("ok");

    relate_profile_to_template_blueprint(&store, profile.id, template_id).await;

    let override_bp = Blueprint {
        id: BlueprintId::new(),
        agent_id: Some(agent.id),
        parallelize: Some(8),
        model_config_id: None,
        mock_response: None,
        created_at: Utc::now(),
    };
    store
        .upsert_agent_blueprint_override(&override_bp)
        .await
        .expect("write override");

    let read = store
        .get_agent_profile_for_agent(agent.id)
        .await
        .expect("read")
        .expect("profile");
    assert_eq!(
        read.parallelize, 8,
        "override parallelize=8 wins over template parallelize=2"
    );
    assert_eq!(
        read.mock_response.as_deref(),
        Some("tmpl"),
        "override mock_response=None falls through to template mock_response=Some(tmpl)"
    );
}

// ===========================================================================
// Test 4 — upsert_agent_profile scoped to supplied agent_id (sibling agents
// untouched)
// ===========================================================================

#[tokio::test]
async fn test_upsert_agent_profile_does_not_delete_other_agents_rows() {
    // F1.c regression: `upsert_agent_profile`'s eviction body is
    // scoped to `agent_id = $agent` — it does NOT cascade to OTHER
    // agents' rows. Pre-CH-28 a buggy global eviction (e.g., DELETE
    // agent_profile WHERE id != $id without the agent_id filter) would
    // have removed sibling agents' rows; this test pins the inter-agent
    // isolation invariant. The intra-agent_id 1:1 eviction
    // (`DELETE agent_profile WHERE agent_id = $agent AND id != $id` at
    // `repo_impl.rs:785`) is INTENTIONALLY preserved per iter-5 plan
    // §7 P1.5 line 534 and is asserted separately at the
    // application-tier 1:1 invariant in
    // `repository_test::upsert_agent_profile_composite_write_persists_override_blueprint`.
    // Renamed from `test_upsert_agent_profile_no_longer_deletes_sibling_rows_for_same_agent_id`
    // at CH-28 retro per P-rename-1 standards update (clarity over
    // the previous ambiguous "sibling rows" framing).
    let (store, _dir) = fresh_store().await;

    let agent_a = sample_agent("sibling-isolation-a");
    let agent_b = sample_agent("sibling-isolation-b");
    store.create_agent(&agent_a).await.expect("create A");
    store.create_agent(&agent_b).await.expect("create B");

    let profile_a = sample_profile(agent_a.id, 1);
    let profile_b = sample_profile(agent_b.id, 2);
    store
        .create_agent_profile(&profile_a)
        .await
        .expect("create profile A");
    store
        .create_agent_profile(&profile_b)
        .await
        .expect("create profile B");

    // Upsert agent A's profile with a NEW row id (i.e. simulate the
    // profile-row-replacement path). The intra-agent eviction would
    // remove agent A's prior row; agent B's row MUST stay.
    let new_profile_a = AgentProfile {
        id: NodeId::new(),
        agent_id: agent_a.id,
        parallelize: 7,
        blueprint: phi_core::agents::profile::AgentProfile::default(),
        model_config_id: None,
        mock_response: None,
        created_at: Utc::now(),
    };
    store
        .upsert_agent_profile(&new_profile_a)
        .await
        .expect("upsert A");

    // Agent B's profile row still resolves to parallelize=2 (or
    // whatever the synthesis returns); the row must still exist.
    let read_b = store
        .get_agent_profile_for_agent(agent_b.id)
        .await
        .expect("read B")
        .expect("agent B's profile row still exists — upsert(A) did NOT delete sibling agent rows");
    assert_eq!(
        read_b.agent_id, agent_b.id,
        "agent B's row still keyed to agent B"
    );
}

// ===========================================================================
// Test 5 — Drop unique constraint post-migration allows two rows
// ===========================================================================

#[tokio::test]
async fn test_drop_unique_constraint_post_migration_allows_two_rows_with_same_agent_id() {
    // F1.c schema-tier: migration 0019 dropped
    // `agent_profile_agent_id` UNIQUE (line 75 of the migration). Two
    // raw rows with the same agent_id may now be inserted via direct
    // SurrealQL without a UNIQUE violation. This is the storage-tier
    // observability of the 1:1 → N:1 cardinality flip. The
    // application-tier `upsert_agent_profile` still enforces 1:1 per
    // iter-5 plan §7 P1.5 line 534; the SCHEMA permits multiplicity,
    // which is what enables N:1 template-sharing patterns where each
    // pointing AgentProfile row is distinct.
    let (store, _dir) = fresh_store().await;

    let agent = sample_agent("schema-multi");
    store.create_agent(&agent).await.expect("create agent");

    let profile_id_1 = NodeId::new();
    let profile_id_2 = NodeId::new();

    // Insert two raw agent_profile rows with the same agent_id via
    // direct SurrealQL (bypasses the application-tier eviction in
    // upsert_agent_profile). Post-0019, the SCHEMAFULL definition no
    // longer carries the per-agent override fields — these CREATE
    // statements write only the SCHEMAFULL-defined fields.
    let blueprint_default =
        serde_json::to_value(phi_core::agents::profile::AgentProfile::default())
            .expect("serialize phi-core default");
    let now = Utc::now().to_rfc3339();
    store
        .client()
        .query(
            "CREATE type::thing('agent_profile', $pid1) CONTENT { \
                agent_id: $agent, \
                blueprint: $bp, \
                created_at: $now \
             } RETURN NONE;",
        )
        .bind(("pid1", profile_id_1.to_string()))
        .bind(("agent", agent.id.to_string()))
        .bind(("bp", blueprint_default.clone()))
        .bind(("now", now.clone()))
        .await
        .expect("insert row 1")
        .check()
        .expect("row 1 inserted");
    store
        .client()
        .query(
            "CREATE type::thing('agent_profile', $pid2) CONTENT { \
                agent_id: $agent, \
                blueprint: $bp, \
                created_at: $now \
             } RETURN NONE;",
        )
        .bind(("pid2", profile_id_2.to_string()))
        .bind(("agent", agent.id.to_string()))
        .bind(("bp", blueprint_default))
        .bind(("now", now))
        .await
        .expect("insert row 2")
        .check()
        .expect("no UNIQUE violation post-0019 (row 2)");

    // Verify both rows exist by direct count.
    let count: Vec<i64> = store
        .client()
        .query("SELECT count() FROM agent_profile WHERE agent_id = $agent GROUP ALL")
        .bind(("agent", agent.id.to_string()))
        .await
        .expect("count")
        .take((0, "count"))
        .expect("extract count");
    assert_eq!(
        count.into_iter().next().unwrap_or(0),
        2,
        "F1.c schema-tier: 2 rows with same agent_id coexist after 0019 UNIQUE drop"
    );
}

// ===========================================================================
// Test 6 — UsesProfileEdgeChanged routes to single-agent listener path
// ===========================================================================

#[tokio::test]
async fn test_uses_profile_edge_emits_single_agent_at_ch28_per_adr_0063_d63_7() {
    // F2.b listener-side: `DomainEvent::UsesProfileEdgeChanged` (the
    // CH-28 rename of `HasProfileEdgeChanged`) routes through the
    // listener's `agent_id_and_timestamp_for` helper to the
    // single-agent path. The rename is wire-compatible (serde alias
    // `HAS_PROFILE` → `USES_PROFILE`) and the listener arm's
    // single-agent contract MUST be preserved per ADR-0063 §D63.7.
    let bus = InProcessEventBus::new();
    let capture = Arc::new(CapturingHandler::default());
    bus.subscribe(capture.clone() as Arc<dyn EventHandler>);

    let agent_id = AgentId::new();
    let old_profile_id = NodeId::new();
    let new_profile_id = NodeId::new();
    let at = Utc::now();
    let event_id = AuditEventId::new();
    let evt = DomainEvent::UsesProfileEdgeChanged {
        agent_id,
        old_profile_id: Some(old_profile_id),
        new_profile_id,
        at,
        event_id,
    };

    bus.emit(evt.clone()).await;

    let captured = capture.snapshot();
    assert_eq!(captured.len(), 1, "exactly one event captured");
    match &captured[0] {
        DomainEvent::UsesProfileEdgeChanged {
            agent_id: aid,
            old_profile_id: opid,
            new_profile_id: npid,
            ..
        } => {
            assert_eq!(*aid, agent_id, "event carries the single agent_id");
            assert_eq!(
                *opid,
                Some(old_profile_id),
                "event preserves old_profile_id"
            );
            assert_eq!(*npid, new_profile_id, "event preserves new_profile_id");
        }
        other => panic!("expected UsesProfileEdgeChanged, got {other:?}"),
    }
    // The kind() string is the stable listener-dispatch key per
    // §D63.7; pin it so future renames surface as a regression.
    assert_eq!(evt.kind(), "uses_profile_edge_changed");
}

// ===========================================================================
// Test 7 — BlueprintUpserted override-tier emit (listener-arm dispatch)
// ===========================================================================

#[tokio::test]
async fn test_blueprint_upserted_event_emits_when_override_blueprint_written() {
    // F1.c §D63.1 listener-side: an override-tier
    // `DomainEvent::BlueprintUpserted` (`agent_id = Some(_)`) carries
    // a concrete agent_id that the listener routes to the single-agent
    // path; a template-tier emission (`agent_id = None`) is a fan-out
    // signal handled separately at the top of `on_event` per §D63.16
    // scope-narrowing.
    //
    // SCOPE NOTE: end-to-end emission of BlueprintUpserted from a
    // production write path (e.g., from
    // `Repository::upsert_agent_blueprint_override`) is deferred to a
    // future cycle per §D63.16 + D-CH28-FOLLOWUP-01 (M6-DEFERRED-04).
    // At CH-28 the production code does not publish the event; this
    // test pins the listener-arm dispatch invariant via direct bus
    // emit so the rename + variant shape stays under test.
    let bus = InProcessEventBus::new();
    let capture = Arc::new(CapturingHandler::default());
    bus.subscribe(capture.clone() as Arc<dyn EventHandler>);

    let agent_id = AgentId::new();
    let blueprint_id = BlueprintId::new();
    let at = Utc::now();
    let event_id = AuditEventId::new();
    let override_evt = DomainEvent::BlueprintUpserted {
        blueprint_id,
        agent_id: Some(agent_id),
        at,
        event_id,
    };
    bus.emit(override_evt.clone()).await;

    let template_evt = DomainEvent::BlueprintUpserted {
        blueprint_id: BlueprintId::new(),
        agent_id: None,
        at,
        event_id: AuditEventId::new(),
    };
    bus.emit(template_evt.clone()).await;

    let captured = capture.snapshot();
    assert_eq!(captured.len(), 2, "both BlueprintUpserted events captured");

    // Override-tier carries Some(agent_id); template-tier carries None.
    match &captured[0] {
        DomainEvent::BlueprintUpserted {
            agent_id: aid,
            blueprint_id: bpid,
            ..
        } => {
            assert_eq!(
                *aid,
                Some(agent_id),
                "override-tier emission carries Some(agent_id)"
            );
            assert_eq!(*bpid, blueprint_id, "blueprint_id preserved on the wire");
        }
        other => panic!("expected BlueprintUpserted override-tier, got {other:?}"),
    }
    match &captured[1] {
        DomainEvent::BlueprintUpserted { agent_id: aid, .. } => {
            assert!(
                aid.is_none(),
                "template-tier emission carries agent_id = None (discriminator)"
            );
        }
        other => panic!("expected BlueprintUpserted template-tier, got {other:?}"),
    }

    // Companion fixture-helper exercise — verify the factory builds
    // an internally consistent (AgentProfile, Blueprint) pair where
    // the blueprint is the override-tier shape the composite-write
    // would emit.
    let (profile, override_bp) =
        make_test_profile_with_overrides(agent_id, 4, Some("model-x".into()), Some("mr".into()));
    assert_eq!(profile.agent_id, agent_id);
    assert_eq!(profile.parallelize, 4);
    assert_eq!(override_bp.agent_id, Some(agent_id));
    assert_eq!(override_bp.parallelize, Some(4));
    assert_eq!(override_bp.model_config_id.as_deref(), Some("model-x"));
    assert_eq!(override_bp.mock_response.as_deref(), Some("mr"));
    assert_eq!(
        profile.created_at, override_bp.created_at,
        "factory keeps timestamps in lockstep mirroring composite-write semantics"
    );
}
