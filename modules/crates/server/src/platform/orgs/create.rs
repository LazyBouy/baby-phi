#![allow(clippy::result_large_err)]

//! Org-creation orchestrator — the M3/P4 business logic behind
//! `POST /api/v0/orgs`.
//!
//! ## phi-core leverage (per-deliverable)
//!
//! This file is **the most phi-core-heavy surface in M3**. Q1/Q2/Q3:
//!
//! - **Q1 — Direct imports**: `use phi_core::agents::profile::AgentProfile;`
//!   for the system-agent blueprint clone + per-role tweak.
//! - **Q2 — Transitive payload**: `OrgCreationPayload` (domain) ships
//!   the full `OrganizationDefaultsSnapshot` which wraps
//!   `phi_core::ExecutionLimits` / `ContextConfig` / `RetryConfig` /
//!   `AgentProfile`. These transit through serde end-to-end
//!   (wire → orchestrator → compound tx → SurrealDB
//!   `FLEXIBLE TYPE object` columns).
//! - **Q3 — Inherit-not-duplicate (ADR-0023)**: `ExecutionLimits` /
//!   `ContextConfig` / `RetryConfig` live **only** on the org's
//!   snapshot. System agents read from the snapshot at invoke time —
//!   no per-agent `ExecutionLimits` / `RetryPolicy` /
//!   `CompactionPolicy` / `CachePolicy` nodes are created here.
//!
//! ## Flow
//!
//! 1. Shape-validate the submitted wizard payload.
//! 2. Resolve the org's `defaults_snapshot` — wizard payload may
//!    supply it verbatim (advanced mode) OR the orchestrator
//!    snapshot-copies current platform defaults (default path; ADR-0019).
//! 3. Build the CEO Agent/Channel/Inbox/Outbox/Grant. The CEO is a
//!    Human agent with `owning_org = Some(org_id)`. The grant is
//!    `[allocate]` on `org:<id>` with fundamentals
//!    `{IdentityPrincipal, Tag}`.
//! 4. For each of the **two system agents** (memory-extractor +
//!    agent-catalog): clone the snapshot's
//!    `default_agent_profile: phi_core::AgentProfile` and override
//!    `name` + `system_prompt` for the role. This is the one place
//!    phi takes an explicit `use phi_core::...` at M3.
//! 5. Build the `TokenBudgetPool` from wizard `initial_allocation`.
//! 6. For each enabled template kind (subset of A/B/C/D), mint an
//!    adoption AR via [`domain::templates::adoption::build_adoption_request`]
//!    plus persist a `Template` graph node so the adoption AR's
//!    `provenance_template` can be wired post-persist.
//! 7. Assemble + commit via [`Repository::apply_org_creation`] — one
//!    atomic tx.
//! 8. Emit `platform.organization.created` + N
//!    `authority_template.adopted` audit events in one
//!    [`emit_audit_batch`] (fail-fast, preserves per-org chain
//!    continuity).
//!
//! Audit emission happens **outside** the repo tx — a successful
//! commit is durable before the first audit event is written; if the
//! batch emit fails afterwards, the operator sees a 500
//! `AUDIT_EMIT_FAILED` and the chain is re-verifiable (entries
//! before the failure are still consistent). This is the M1/M2
//! pattern unchanged.

use std::sync::Arc;

use chrono::{DateTime, Utc};

use domain::audit::events::m3::orgs as org_events;
use domain::audit::{AuditClass, AuditEmitter};
use domain::model::composites_m3::{ConsentPolicy, OrganizationDefaultsSnapshot, TokenBudgetPool};
use domain::model::ids::{AgentId, GrantId, NodeId, OrgId};
use domain::model::nodes::{
    Agent, AgentKind, AgentProfile, Channel, ChannelKind, Grant, InboxObject, Organization,
    OutboxObject, PrincipalRef, ResourceRef, TemplateKind,
};
use domain::model::Fundamental;
use domain::repository::{OrgCreationPayload, Repository};
use domain::templates::adoption::{build_adoption_request, AdoptionArgs};
// --- phi-core direct import ------------------------------------------------
// The *only* new `use phi_core::...` line introduced in M3/P4. Used
// exclusively for the per-system-agent blueprint clone + tweak below.
// `check-phi-core-reuse.sh` confirms no phi redeclaration of
// this type exists anywhere under `modules/crates/`.
use phi_core::agents::profile::AgentProfile as PhiCoreAgentProfile;
// ---------------------------------------------------------------------------

use crate::handler_support::{emit_audit_batch, ApiError};

use super::{CreatedOrg, OrgError};

/// Validated wizard payload — the shape `POST /api/v0/orgs` accepts.
#[derive(Debug, Clone)]
pub struct CreateInput {
    /// Display name for the new org; non-empty.
    pub display_name: String,
    pub vision: Option<String>,
    pub mission: Option<String>,
    /// Consent policy the org enforces at session-start time.
    pub consent_policy: ConsentPolicy,
    /// Default audit class for events without an explicit class.
    pub audit_class_default: AuditClass,
    /// Authority templates the org enables at creation — subset of
    /// `[A, B, C, D]`. E is always available (auto-approve); F is
    /// reserved for M6; SystemBootstrap is a platform-level template.
    pub authority_templates_enabled: Vec<TemplateKind>,
    /// Optional override of the frozen snapshot. `None` means "snapshot
    /// the current platform defaults" (the default wizard path).
    pub defaults_snapshot_override: Option<OrganizationDefaultsSnapshot>,
    /// Optional default model provider. Referenced by system agents
    /// at M5 invoke time via `UsesModel` edge (deferred).
    pub default_model_provider: Option<domain::model::ids::ModelProviderId>,
    /// CEO nomination — an existing User or an Agent id. At M3 the
    /// orchestrator creates the corresponding `Agent` node (kind
    /// Human, owning_org = new org).
    pub ceo_display_name: String,
    pub ceo_channel_kind: ChannelKind,
    pub ceo_channel_handle: String,
    /// Ceiling for the org's token budget pool.
    pub token_budget: u64,
    /// Actor (platform admin) recorded on every audit event.
    pub actor: AgentId,
    pub now: DateTime<Utc>,
}

/// Mint a fresh org. All writes commit inside one `apply_org_creation`
/// tx; audit events emit in one `emit_audit_batch` outside the tx.
pub async fn create_organization(
    repo: Arc<dyn Repository>,
    audit: Arc<dyn AuditEmitter>,
    input: CreateInput,
) -> Result<CreatedOrg, OrgError> {
    // 1. Shape validation.
    validate_input(&input)?;

    // 2. Resolve the snapshot. Default path: snapshot-copy current
    //    platform defaults (or platform factory if none persisted)
    //    per ADR-0019. Advanced path: operator supplied an override
    //    (typically only to tighten `max_retries` or shorten
    //    retention — the wizard UI will expose a subset of the fields
    //    in the future).
    let snapshot = match input.defaults_snapshot_override {
        Some(s) => s,
        None => snapshot_from_platform_defaults(&*repo, input.now).await?,
    };

    let org_id = OrgId::new();

    // 3. CEO + membership envelope.
    let ceo_agent = Agent {
        id: AgentId::new(),
        kind: AgentKind::Human,
        display_name: input.ceo_display_name.clone(),
        owning_org: Some(org_id),
        // M4 will set `Some(AgentRole::Executive)` from the wizard.
        role: None,
        created_at: input.now,
    };
    let ceo_channel = Channel {
        id: NodeId::new(),
        agent_id: ceo_agent.id,
        kind: input.ceo_channel_kind,
        handle: input.ceo_channel_handle.clone(),
        created_at: input.now,
    };
    let ceo_inbox = InboxObject {
        id: NodeId::new(),
        agent_id: ceo_agent.id,
        created_at: input.now,
    };
    let ceo_outbox = OutboxObject {
        id: NodeId::new(),
        agent_id: ceo_agent.id,
        created_at: input.now,
    };
    let ceo_grant = Grant {
        id: GrantId::new(),
        holder: PrincipalRef::Agent(ceo_agent.id),
        action: vec!["allocate".into()],
        resource: ResourceRef {
            uri: format!("org:{}", org_id),
        },
        fundamentals: vec![Fundamental::IdentityPrincipal, Fundamental::Tag],
        descends_from: None,
        delegable: true,
        issued_at: input.now,
        revoked_at: None,
    };

    // 4. Two system agents with phi-core blueprints cloned from the
    //    snapshot. This is THE phi-core-direct-reuse step in M3.
    let system_agents = build_system_agents(&snapshot, org_id, input.now);
    let system_agent_ids = [system_agents[0].0.id, system_agents[1].0.id];

    // 5. Token budget pool (phi-native — no phi-core counterpart).
    let token_budget_pool = TokenBudgetPool::new(org_id, input.token_budget, input.now);

    // 6. Adoption ARs. Template *node* persistence is deferred to
    //    M5 when trigger-fire needs to reference a shared pattern
    //    node — at M3 the adoption AR IS the provenance (the
    //    `authority_template.adopted` audit event carries
    //    `provenance_auth_request_id = ar.id`, and the dashboard
    //    resolves the template kind from the adoption AR's URI
    //    prefix). Avoiding per-org Template nodes also sidesteps a
    //    UNIQUE INDEX collision on `template.name` when two orgs
    //    adopt the same kind.
    let mut adoption_ars = Vec::with_capacity(input.authority_templates_enabled.len());
    for kind in &input.authority_templates_enabled {
        reject_non_adoptable_template(*kind)?;
        let ar = build_adoption_request(
            *kind,
            AdoptionArgs {
                org_id,
                ceo: PrincipalRef::Agent(ceo_agent.id),
                now: input.now,
            },
        );
        adoption_ars.push(ar);
    }

    // 7. Assemble Organization node + persist via compound tx.
    let organization = Organization {
        id: org_id,
        display_name: input.display_name.clone(),
        vision: input.vision.clone(),
        mission: input.mission.clone(),
        consent_policy: input.consent_policy,
        audit_class_default: input.audit_class_default,
        authority_templates_enabled: input.authority_templates_enabled.clone(),
        defaults_snapshot: Some(snapshot.clone()),
        default_model_provider: input.default_model_provider,
        system_agents: system_agent_ids.to_vec(),
        created_at: input.now,
    };

    // Catalogue seeds: the org's control-plane root + each adopted
    // template's adoption URI. The dashboard's AdoptedTemplates panel
    // reads these.
    let mut catalogue_entries = vec![(format!("org:{}", org_id), "control_plane".to_string())];
    for kind in &input.authority_templates_enabled {
        catalogue_entries.push((
            format!("org:{}/template:{}", org_id, kind.as_str()),
            "control_plane".to_string(),
        ));
    }

    let payload = OrgCreationPayload {
        organization: organization.clone(),
        ceo_agent: ceo_agent.clone(),
        ceo_channel,
        ceo_inbox,
        ceo_outbox,
        ceo_grant,
        system_agents: system_agents.clone(),
        token_budget_pool,
        adoption_auth_requests: adoption_ars.clone(),
        catalogue_entries,
    };

    let receipt = repo.apply_org_creation(&payload).await?;

    // 8. Audit batch — OrganizationCreated + N AuthorityTemplateAdopted.
    let mut events = Vec::with_capacity(1 + adoption_ars.len());
    events.push(org_events::organization_created(
        input.actor,
        &organization,
        receipt.ceo_agent_id,
        adoption_ars.first().map(|ar| ar.id),
        input.now,
    ));
    for (i, kind) in input.authority_templates_enabled.iter().enumerate() {
        events.push(org_events::authority_template_adopted(
            input.actor,
            org_id,
            *kind,
            receipt.adoption_auth_request_ids[i],
            input.now,
        ));
    }
    let audit_event_ids = emit_audit_batch(&*audit, events)
        .await
        .map_err(|e: ApiError| OrgError::AuditEmit(e.message))?;

    Ok(CreatedOrg {
        org_id: receipt.org_id,
        ceo_agent_id: receipt.ceo_agent_id,
        ceo_channel_id: receipt.ceo_channel_id,
        ceo_inbox_id: receipt.ceo_inbox_id,
        ceo_outbox_id: receipt.ceo_outbox_id,
        ceo_grant_id: receipt.ceo_grant_id,
        system_agent_ids: receipt.system_agent_ids,
        token_budget_pool_id: receipt.token_budget_pool_id,
        adoption_auth_request_ids: receipt.adoption_auth_request_ids,
        audit_event_ids,
        template_ids: Vec::new(),
    })
}

fn validate_input(input: &CreateInput) -> Result<(), OrgError> {
    if input.display_name.trim().is_empty() {
        return Err(OrgError::Validation(
            "display_name must not be empty".into(),
        ));
    }
    if input.ceo_display_name.trim().is_empty() {
        return Err(OrgError::Validation(
            "ceo_display_name must not be empty".into(),
        ));
    }
    if input.ceo_channel_handle.trim().is_empty() {
        return Err(OrgError::Validation(
            "ceo_channel_handle must not be empty".into(),
        ));
    }
    if input.token_budget == 0 {
        return Err(OrgError::Validation("token_budget must be > 0".into()));
    }
    // Duplicate templates would cause a UNIQUE-index collision on
    // (org_id, kind). Reject up-front with a friendlier error.
    let mut seen = std::collections::HashSet::new();
    for k in &input.authority_templates_enabled {
        if !seen.insert(*k) {
            return Err(OrgError::Validation(format!(
                "duplicate authority template: {:?}",
                k
            )));
        }
    }
    Ok(())
}

fn reject_non_adoptable_template(kind: TemplateKind) -> Result<(), OrgError> {
    match kind {
        TemplateKind::A | TemplateKind::B | TemplateKind::C | TemplateKind::D => Ok(()),
        TemplateKind::E => Err(OrgError::TemplateNotAdoptable(
            "Template E is platform-level (Template-E-shaped self-approve); not adopted by orgs"
                .into(),
        )),
        TemplateKind::F => Err(OrgError::TemplateNotAdoptable(
            "Template F is reserved for M6 break-glass; not adoptable at org creation".into(),
        )),
        TemplateKind::SystemBootstrap => Err(OrgError::TemplateNotAdoptable(
            "SystemBootstrap is the platform-genesis template; not org-scoped".into(),
        )),
    }
}

/// Resolve the org's frozen defaults snapshot. When no override is
/// supplied, snapshot the current platform defaults (or factory if
/// none persisted). The result is *always* complete — the non-
/// retroactive invariant holds because the snapshot is captured
/// before the compound tx starts.
async fn snapshot_from_platform_defaults(
    repo: &dyn Repository,
    now: DateTime<Utc>,
) -> Result<OrganizationDefaultsSnapshot, OrgError> {
    let platform = repo
        .get_platform_defaults()
        .await?
        .unwrap_or_else(|| domain::model::PlatformDefaults::factory(now));
    Ok(OrganizationDefaultsSnapshot::from_platform_defaults(
        &platform,
    ))
}

/// Build the two system agents + their `AgentProfile` nodes. Each
/// profile's `blueprint: phi_core::AgentProfile` is cloned from the
/// snapshot's `default_agent_profile` with `name` + `system_prompt`
/// overridden for the role.
///
/// Returns a `[(Agent, AgentProfile); 2]` matching the shape
/// `OrgCreationPayload.system_agents` expects. Order: `[0]` is
/// memory-extractor; `[1]` is agent-catalog. The compound tx + audit
/// chain rely on this stable ordering — changing it breaks
/// `spawn_claimed_with_org_smoke` + acceptance fidelity tests.
fn build_system_agents(
    snapshot: &OrganizationDefaultsSnapshot,
    org_id: OrgId,
    now: DateTime<Utc>,
) -> [(Agent, AgentProfile); 2] {
    let base: &PhiCoreAgentProfile = &snapshot.default_agent_profile;

    let memory_extractor_agent = Agent {
        id: AgentId::new(),
        kind: AgentKind::Llm,
        display_name: "memory-extractor".into(),
        owning_org: Some(org_id),
        // M4 will set `Some(AgentRole::System)` once system-agent
        // provisioning is migrated; leaving None keeps the existing
        // dashboard behaviour (counted under `unclassified`).
        role: None,
        created_at: now,
    };
    let mut memory_blueprint: PhiCoreAgentProfile = base.clone();
    memory_blueprint.name = Some("memory-extractor".into());
    memory_blueprint.system_prompt = Some(
        "You are this org's memory-extraction system agent. Distill \
         durable facts from recent session transcripts into the \
         shared memory store."
            .into(),
    );
    let memory_extractor_profile = AgentProfile {
        id: NodeId::new(),
        agent_id: memory_extractor_agent.id,
        parallelize: 1,
        blueprint: memory_blueprint,
        model_config_id: None,
        created_at: now,
    };

    let agent_catalog_agent = Agent {
        id: AgentId::new(),
        kind: AgentKind::Llm,
        display_name: "agent-catalog".into(),
        owning_org: Some(org_id),
        role: None,
        created_at: now,
    };
    let mut catalog_blueprint: PhiCoreAgentProfile = base.clone();
    catalog_blueprint.name = Some("agent-catalog".into());
    catalog_blueprint.system_prompt = Some(
        "You are this org's agent-catalog system agent. Maintain the \
         roster of active agents and their current capabilities."
            .into(),
    );
    let agent_catalog_profile = AgentProfile {
        id: NodeId::new(),
        agent_id: agent_catalog_agent.id,
        parallelize: 1,
        blueprint: catalog_blueprint,
        model_config_id: None,
        created_at: now,
    };

    [
        (memory_extractor_agent, memory_extractor_profile),
        (agent_catalog_agent, agent_catalog_profile),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reject_non_adoptable_template_blocks_e_f_system_bootstrap() {
        assert!(matches!(
            reject_non_adoptable_template(TemplateKind::E),
            Err(OrgError::TemplateNotAdoptable(_))
        ));
        assert!(matches!(
            reject_non_adoptable_template(TemplateKind::F),
            Err(OrgError::TemplateNotAdoptable(_))
        ));
        assert!(matches!(
            reject_non_adoptable_template(TemplateKind::SystemBootstrap),
            Err(OrgError::TemplateNotAdoptable(_))
        ));
    }

    #[test]
    fn reject_non_adoptable_template_accepts_a_b_c_d() {
        for k in [
            TemplateKind::A,
            TemplateKind::B,
            TemplateKind::C,
            TemplateKind::D,
        ] {
            assert!(reject_non_adoptable_template(k).is_ok());
        }
    }

    #[test]
    fn validate_rejects_empty_display_name() {
        let input = sample_input();
        let mut bad = input.clone();
        bad.display_name = "  ".into();
        assert!(matches!(validate_input(&bad), Err(OrgError::Validation(_))));
    }

    #[test]
    fn validate_rejects_zero_token_budget() {
        let mut input = sample_input();
        input.token_budget = 0;
        assert!(matches!(
            validate_input(&input),
            Err(OrgError::Validation(_))
        ));
    }

    #[test]
    fn validate_rejects_duplicate_templates() {
        let mut input = sample_input();
        input.authority_templates_enabled = vec![TemplateKind::A, TemplateKind::A];
        assert!(matches!(
            validate_input(&input),
            Err(OrgError::Validation(_))
        ));
    }

    /// Positive phi-core invariant: the blueprint on each built system
    /// agent MUST be exactly `phi_core::AgentProfile` (not a phi
    /// redeclaration). Compile-time coercion test — if anyone swaps
    /// in a local struct, this test stops compiling.
    #[test]
    fn system_agent_blueprint_is_phi_core_agent_profile() {
        fn is_phi_core_agent_profile(_: &phi_core::agents::profile::AgentProfile) {}
        let snap = OrganizationDefaultsSnapshot::from_platform_defaults(
            &domain::model::PlatformDefaults::factory(Utc::now()),
        );
        let pair = build_system_agents(&snap, OrgId::new(), Utc::now());
        is_phi_core_agent_profile(&pair[0].1.blueprint);
        is_phi_core_agent_profile(&pair[1].1.blueprint);
        // The per-role tweak took effect.
        assert_eq!(
            pair[0].1.blueprint.name.as_deref(),
            Some("memory-extractor")
        );
        assert_eq!(pair[1].1.blueprint.name.as_deref(), Some("agent-catalog"));
    }

    fn sample_input() -> CreateInput {
        CreateInput {
            display_name: "Acme".into(),
            vision: None,
            mission: None,
            consent_policy: ConsentPolicy::Implicit,
            audit_class_default: AuditClass::Logged,
            authority_templates_enabled: vec![TemplateKind::A],
            defaults_snapshot_override: None,
            default_model_provider: None,
            ceo_display_name: "Alice".into(),
            ceo_channel_kind: ChannelKind::Email,
            ceo_channel_handle: "alice@acme.test".into(),
            token_budget: 1_000_000,
            actor: AgentId::new(),
            now: Utc::now(),
        }
    }
}
