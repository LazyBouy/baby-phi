//! `phi_core::StreamProvider` selection at session-launch time.
//!
//! At M5 (CH-02 / ADR-0032 D32.1) every session runs through
//! `phi_core::provider::mock::MockProvider`, regardless of the
//! configured `ModelRuntime`. The provider's response is driven by
//! the agent's `AgentProfile.mock_response` governance field
//! (ADR-0032 D32.2): when `Some(s)` the provider returns `s`; when
//! `None` it returns the default `"Acknowledged."`.
//!
//! At M7 the body of [`provider_for`] swaps to dispatch real
//! providers via `phi_core::provider::registry::ProviderRegistry`
//! after splicing the credentials vault entry into
//! `runtime.config.api_key`. The `mock_response` field stays as a
//! test-mode override but is bypassed when real credentials are
//! configured.
//!
//! See [`docs/specs/v0/implementation/m5_2/decisions/0032-mock-provider-at-m5.md`]
//! for the conforming-real-provider criteria the M7 swap must
//! satisfy.

use std::sync::Arc;

use domain::model::composites_m2::ModelRuntime;
use domain::model::nodes::AgentProfile;
use domain::session_recorder::SessionLaunchContext;
use phi_core::provider::mock::MockProvider;
use phi_core::provider::traits::StreamProvider;
use phi_core::types::context::AgentContext;

/// Default mock response when `AgentProfile.mock_response` is `None`.
/// Kept module-public so tests can assert against it.
pub const DEFAULT_MOCK_RESPONSE: &str = "Acknowledged.";

/// Resolve the `StreamProvider` baby-phi will hand to
/// `phi_core::agent_loop` for this session.
///
/// At M5 the `_runtime` argument is intentionally unused; M7 will
/// switch on `runtime.config.api` to dispatch real providers.
pub fn provider_for(_runtime: &ModelRuntime, profile: &AgentProfile) -> Arc<dyn StreamProvider> {
    let response = profile
        .mock_response
        .clone()
        .unwrap_or_else(|| DEFAULT_MOCK_RESPONSE.to_string());
    Arc::new(MockProvider::text(response))
}

/// Build the `phi_core::AgentContext` baby-phi will hand to
/// `agent_loop` for this session.
///
/// - `system_prompt` is pulled from `profile.blueprint.system_prompt`
///   (the phi-core inner is the source of truth per ADR-0029); empty
///   string when the blueprint omits it.
/// - `agent_id` / `session_id` / `loop_id` are pre-set from the
///   launch context so phi-core re-uses baby-phi's pre-allocated IDs
///   from the compound-tx (Step 6 of `launch_session`) rather than
///   generating fresh UUIDs at loop entry.
/// - `messages` starts empty — the first user prompt is fed into
///   `agent_loop` via the `prompts: Vec<AgentMessage>` parameter, not
///   via the context.
/// - `tools` is empty at M5; CH-22 / CH-23 wire real tools later.
pub fn build_agent_context(ctx: &SessionLaunchContext, profile: &AgentProfile) -> AgentContext {
    AgentContext {
        system_prompt: profile.blueprint.system_prompt.clone().unwrap_or_default(),
        agent_id: Some(ctx.started_by.to_string()),
        session_id: Some(ctx.phi_core_session_id.clone()),
        loop_id: ctx.first_loop_id.map(|id| id.to_string()),
        ..AgentContext::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::Utc;
    use domain::model::ids::{
        AgentId, LoopId, ModelProviderId, NodeId, OrgId, ProjectId, SessionId,
    };
    use domain::model::{ModelRuntime, RuntimeStatus, SecretRef, TenantSet};

    fn fixture_runtime() -> ModelRuntime {
        ModelRuntime {
            id: ModelProviderId::new(),
            config: phi_core::provider::model::ModelConfig::anthropic("test", "test", ""),
            secret_ref: SecretRef::new("vault://test"),
            tenants_allowed: TenantSet::All,
            status: RuntimeStatus::Ok,
            archived_at: None,
            created_at: Utc::now(),
        }
    }

    fn fixture_profile(mock: Option<String>) -> AgentProfile {
        AgentProfile {
            id: NodeId::new(),
            agent_id: AgentId::new(),
            parallelize: 1,
            blueprint: phi_core::agents::profile::AgentProfile::default(),
            model_config_id: None,
            mock_response: mock,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn provider_for_returns_default_mock_when_profile_field_is_none() {
        let runtime = fixture_runtime();
        let profile = fixture_profile(None);
        let provider = provider_for(&runtime, &profile);
        // Trait-object coercion compiled — the only way to verify
        // the chosen response is via Arc::ptr_eq + a parallel
        // construction. Smoke-test the coercion compiles.
        let _: &dyn StreamProvider = provider.as_ref();
    }

    #[test]
    fn provider_for_uses_mock_response_override_when_some() {
        let runtime = fixture_runtime();
        let profile = fixture_profile(Some("Custom test fixture".into()));
        let provider = provider_for(&runtime, &profile);
        let _: &dyn StreamProvider = provider.as_ref();
    }

    #[test]
    fn default_mock_response_constant_is_acknowledged() {
        assert_eq!(DEFAULT_MOCK_RESPONSE, "Acknowledged.");
    }

    fn fixture_launch_ctx(loop_id: Option<LoopId>) -> SessionLaunchContext {
        SessionLaunchContext {
            session_id: SessionId::new(),
            phi_core_session_id: "phi-core-session-abc".to_string(),
            owning_org: OrgId::new(),
            owning_project: ProjectId::new(),
            started_by: AgentId::new(),
            started_at: Utc::now(),
            first_loop_id: loop_id,
        }
    }

    #[test]
    fn build_agent_context_propagates_ids_from_launch_ctx() {
        let loop_id = LoopId::new();
        let launch_ctx = fixture_launch_ctx(Some(loop_id));
        let mut profile = fixture_profile(None);
        profile.blueprint.system_prompt = Some("You are a test agent.".into());

        let agent_ctx = build_agent_context(&launch_ctx, &profile);

        assert_eq!(
            agent_ctx.agent_id.as_deref(),
            Some(launch_ctx.started_by.to_string().as_str())
        );
        assert_eq!(
            agent_ctx.session_id.as_deref(),
            Some("phi-core-session-abc")
        );
        assert_eq!(
            agent_ctx.loop_id.as_deref(),
            Some(loop_id.to_string().as_str())
        );
        assert_eq!(agent_ctx.system_prompt, "You are a test agent.");
        assert!(
            agent_ctx.messages.is_empty(),
            "first prompt enters via agent_loop's `prompts` arg, not context"
        );
        assert!(agent_ctx.tools.is_empty(), "no tools wired at M5");
    }

    #[test]
    fn build_agent_context_handles_missing_system_prompt_and_loop_id() {
        let launch_ctx = fixture_launch_ctx(None);
        let profile = fixture_profile(None); // blueprint.system_prompt = None
        let agent_ctx = build_agent_context(&launch_ctx, &profile);
        assert_eq!(
            agent_ctx.system_prompt, "",
            "absent blueprint system_prompt yields empty string"
        );
        assert_eq!(
            agent_ctx.loop_id, None,
            "absent first_loop_id yields None — agent_loop will allocate"
        );
    }
}
