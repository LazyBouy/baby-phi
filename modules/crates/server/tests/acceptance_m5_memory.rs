//! CH-24 / F1.B s02-extraction flow + memory-availability surface —
//! milestone-rollup acceptance for the memory-extraction pipeline.
//!
//! Per plan §7 P-NEW-TESTS (v2 user-locked F1.B 5-per-page split):
//!
//! - `m5_memory_extraction_to_memory_query_surface` — drive the
//!   full session golden path → session finalises → the
//!   `MemoryExtractionListener` fires on the `SessionEnded`
//!   `DomainEvent` → a `Memory` row materialises in the agent's
//!   memory pool → assert via the repository's
//!   `list_memories_for_agent` reader (the public memory-availability
//!   surface at M5 close per `domain/src/repository.rs:635`).
//!
//!   At M5 close there is no HTTP endpoint exposing the memory pool
//!   (it's an internal query surface used by future memory-replay
//!   wiring at M6+); the test asserts via the repository trait. This
//!   matches the existing `acceptance_memory_extraction.rs` pattern
//!   that wires the listener body directly + asserts via the repo.
//!
//!   The bootstrap goes through the full HTTP launch (not a direct
//!   bus emit) so this is a true end-to-end exercise: HTTP launch →
//!   recorder writes session → finalise emits SessionEnded → listener
//!   mints Memory → repo reader returns Memory.

mod acceptance_common;

use std::time::Duration;

use acceptance_common::m5_bootstrap::{bootstrap_to_session_endable, client_as};
use domain::audit::AuditEmitter;
use domain::events::{CatalogAuditMode, EventBus};
use domain::Repository;
use serde_json::{json, Value};
use server::state::build_event_bus_with_m5_listeners;
use std::sync::Arc;

async fn wait_for_finalised(client: &reqwest::Client, show_url: &str, deadline_ms: u64) -> Value {
    let start = std::time::Instant::now();
    loop {
        let res = client
            .get(show_url)
            .send()
            .await
            .expect("GET session detail");
        if res.status().as_u16() == 200 {
            let body: Value = res.json().await.expect("decode session detail JSON");
            let state = body["session"]["governance_state"].as_str().unwrap_or("");
            if state != "running" {
                return body;
            }
        }
        if start.elapsed() > Duration::from_millis(deadline_ms) {
            panic!("session did not finalise within {deadline_ms} ms");
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

#[tokio::test]
async fn m5_memory_extraction_to_memory_query_surface() {
    let fx = bootstrap_to_session_endable().await;
    let project = &fx.project;
    let ceo_client = client_as(&project.claimed_org.admin, project.claimed_org.ceo_agent_id);

    // The acceptance harness's default event bus is
    // `InProcessEventBus::new()` with NO subscribers
    // (`acceptance_common/mod.rs::spawn` builds an empty bus).
    // For this end-to-end memory-extraction test we need the M5
    // listener set wired (production wires this via
    // `build_event_bus_with_m5_listeners`). At M5 close the bus on
    // the live AppState is the same as `acceptance_common`'s
    // (production wiring is in `phi-server` `main.rs`; the acceptance
    // harness wires the production listeners explicitly here to
    // exercise the real path). The launch handler emits SessionEnded
    // via `state.event_bus` so we need to subscribe the listener to
    // that exact bus instance.
    //
    // Subscribe the MemoryExtractionListener (CH-21) by hand against
    // the harness's existing event_bus. Mirrors
    // `acceptance_system_flows_s05.rs::subscribe_template_*` pattern.
    let repo: Arc<dyn Repository> = project.claimed_org.admin.acc.store.clone();

    // Pre-fix: the `spawn_claimed_with_org` fixture seeds the
    // memory-extractor system agent with display_name="memory-extractor",
    // but the CH-21 `MemoryExtractionListener::resolve_memory_extraction_system_agent`
    // resolves by matching display_name against the
    // `MEMORY_EXTRACTION_SYSTEM_AGENT_DISPLAY_NAME` constant
    // (= "memory-extraction-agent"). Rename the fixture's system
    // agent here so the listener resolves it. The fixture mismatch is
    // a pre-existing CH-21 / fixture drift that doesn't affect
    // production (production wires the system agent via the
    // org-creation wizard with the correct slug); the legacy fixture
    // helper predates CH-21's renaming.
    let extractor_sys_agent_id = project.claimed_org.system_agents[0];
    let mut extractor_agent = repo
        .get_agent(extractor_sys_agent_id)
        .await
        .expect("repo ok")
        .expect("extractor system agent present");
    extractor_agent.display_name =
        domain::events::MEMORY_EXTRACTION_SYSTEM_AGENT_DISPLAY_NAME.to_string();
    repo.upsert_agent(&extractor_agent)
        .await
        .expect("rename extractor agent to MEMORY_EXTRACTION_SYSTEM_AGENT_DISPLAY_NAME");

    let audit: Arc<dyn AuditEmitter> = Arc::new(store::SurrealAuditEmitter::new(repo.clone()));
    let production_bus =
        build_event_bus_with_m5_listeners(repo.clone(), audit, CatalogAuditMode::default());
    // Subscribe the production listener bundle to the harness bus
    // so SessionEnded emits from the launch handler reach the
    // MemoryExtractionListener. Bridge via a forwarding handler
    // because `InProcessEventBus` only takes `Arc<dyn EventHandler>`.
    let bridge = Arc::new(BridgeHandler {
        downstream: production_bus,
    });
    project.claimed_org.admin.acc.event_bus.subscribe(bridge);

    // -------- Launch via HTTP -----------------------------------------------
    let launch_url = project.url(&format!(
        "/api/v0/orgs/{}/projects/{}/sessions",
        project.org_id(),
        project.project_id
    ));
    let res = ceo_client
        .post(&launch_url)
        .json(&json!({
            "agent_id": project.project_lead.to_string(),
            "prompt": "CH-24 memory extraction end-to-end"
        }))
        .send()
        .await
        .expect("POST launch");
    assert_eq!(res.status().as_u16(), 201);
    let launch_body: Value = res.json().await.expect("decode launch body");
    let session_id = launch_body["session_id"]
        .as_str()
        .expect("session_id present")
        .to_string();

    // -------- Wait for finalisation ----------------------------------------
    let show_url = project.url(&format!("/api/v0/sessions/{}", session_id));
    let _ = wait_for_finalised(&ceo_client, &show_url, 2_000).await;

    // The SessionEnded event is emitted from the launch task's
    // finalisation path; the listener fires async on its tokio
    // task. Poll for the Memory row to land in the lead's memory
    // pool — the bus dispatches in the same process so we expect a
    // handful of ms.
    let lead = project.project_lead;
    let deadline = std::time::Instant::now() + Duration::from_millis(2_000);
    let memories = loop {
        let mems = repo
            .list_memories_for_agent(lead)
            .await
            .expect("list_memories_for_agent");
        if !mems.is_empty() {
            break mems;
        }
        if std::time::Instant::now() >= deadline {
            panic!(
                "memory-extraction listener did not mint a Memory row \
                 within 2 s after session finalisation; the bridge handler \
                 was subscribed to the harness bus + the production listener \
                 bundle was wired against the repo, so this means SessionEnded \
                 did not propagate (CH-21 regression suspected)"
            );
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    };

    // The CH-21 contract is exactly-one-memory-per-session at v0
    // (mirrors `acceptance_memory_extraction.rs` scenario 1).
    assert_eq!(memories.len(), 1, "exactly one Memory per session at v0");
    let mem = &memories[0];

    // Required tag set: agent: + session: + project: + org:
    // (CH-21 / ADR-0041 P2 invariant).
    assert!(
        mem.tags.iter().any(|t| t == &format!("agent:{}", lead)),
        "Memory.tags must include agent:<id>: {:?}",
        mem.tags
    );
    assert!(
        mem.tags
            .iter()
            .any(|t| t == &format!("session:{}", session_id)),
        "Memory.tags must include session:<id>: {:?}",
        mem.tags
    );
    assert!(
        mem.tags
            .iter()
            .any(|t| t == &format!("project:{}", project.project_id)),
        "Memory.tags must include project:<id>: {:?}",
        mem.tags
    );
    assert!(
        mem.tags
            .iter()
            .any(|t| t == &format!("org:{}", project.claimed_org.org_id)),
        "Memory.tags must include org:<id>: {:?}",
        mem.tags
    );

    // Identity counter advanced (CH-21 §D40.3 + D41 contract).
    let identity = repo
        .get_identity(lead)
        .await
        .expect("repo ok")
        .expect("identity row present for LLM agent");
    assert_eq!(
        identity.witnessed.memories_extracted, 1,
        "Identity counter must bump exactly once per memory"
    );
}

// ---------------------------------------------------------------------------
// Bus bridge — fans an inbound DomainEvent on the harness bus into the
// production listener bundle. Required because `build_event_bus_with_m5_listeners`
// returns its own bus (not a subscriber set); we forward each event so
// the production listeners observe the harness bus's emits.
// ---------------------------------------------------------------------------

struct BridgeHandler {
    downstream: Arc<dyn EventBus>,
}

#[async_trait::async_trait]
impl domain::events::EventHandler for BridgeHandler {
    async fn on_event(&self, event: &domain::events::DomainEvent) {
        self.downstream.emit(event.clone()).await;
    }
}
