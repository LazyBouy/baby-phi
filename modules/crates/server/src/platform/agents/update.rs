//! Agent-profile update orchestrator (page 09 edit mode, M4/P5).
//!
//! ## phi-core leverage
//!
//! - **Q1 direct imports**:
//!   - [`phi_core::agents::profile::AgentProfile`] — the patch body
//!     supplies the new blueprint; we compute a diff and persist.
//!   - [`phi_core::context::execution::ExecutionLimits`] — the three
//!     ExecutionLimits paths (inherit / set-override / revert).
//!   - [`phi_core::types::ThinkingLevel`] — captured in the patch diff.
//! - **Q2 transitive**: the `AgentProfilePatchDiff` audit struct
//!   carries phi-core types (`ThinkingLevel`) via serde.
//! - **Q3 rejections**: `ContextConfig` + `RetryConfig` + `ModelConfig`
//!   (full body) stay inherit-from-snapshot; at M4 only the
//!   `model_config.id` handle is surfaced for editing (the selected
//!   provider-config in the org catalogue).
//!
//! ## Immutability rules at M4
//!
//! Per D3 (M4 plan), the following Agent fields are **immutable**
//! post-creation; a patch that attempts to change them returns
//! `400 AGENT_IMMUTABLE_FIELD_CHANGED`:
//!
//! - `id`, `kind`, `owning_org`, `role`
//!
//! Role transitions (Intern → Contract, Member → Admin) are out of
//! M4 scope; a dedicated role-transition flow lands later.
//!
//! ## ModelConfig change gating (D-M4-3)
//!
//! Changing `blueprint.model_config.id` returns `409
//! ACTIVE_SESSIONS_BLOCK_MODEL_CHANGE` if the agent has any active
//! sessions. At M4 the count is always zero (session persistence is
//! M5 work — see [`domain::Repository::count_active_sessions_for_agent`]);
//! the error-code wiring is present so M5 can flip the count to a
//! real query with no handler edit.

use std::sync::Arc;

use chrono::{DateTime, Utc};

use phi_core::agents::profile::AgentProfile as PhiCoreAgentProfile;
use phi_core::context::execution::ExecutionLimits;
use phi_core::types::ThinkingLevel;

use domain::audit::events::m4::agents::{
    agent_profile_updated, AgentProfilePatchDiff, ExecutionLimitsSource,
};
use domain::audit::AuditEmitter;
use domain::model::composites_m4::AgentExecutionLimitsOverride;
use domain::model::ids::{AgentId, AuditEventId, NodeId};
use domain::model::nodes::{Agent, AgentKind, AgentProfile, AgentRole};
use domain::repository::Repository;

use super::execution_limits::{apply_override, clear_override, org_ceiling_for_agent};
use super::AgentError;

/// Compile-time coercion checks — pin the phi-core types the
/// orchestrator binds to.
#[allow(dead_code)]
fn _is_phi_core_agent_profile(_: &PhiCoreAgentProfile) {}
#[allow(dead_code)]
fn _is_phi_core_execution_limits(_: &ExecutionLimits) {}
#[allow(dead_code)]
fn _is_phi_core_thinking_level(_: &ThinkingLevel) {}

/// Three ways the patch can shape the override row.
#[derive(Debug, Clone, Default)]
pub enum ExecutionLimitsPatch {
    /// Leave the current state alone — whatever it was (override or
    /// inherit) stays.
    #[default]
    Unchanged,
    /// Delete the override row (idempotent). Future reads fall back
    /// to the org snapshot per ADR-0023.
    Revert,
    /// Upsert the override row with these limits. Validated against
    /// the org ceiling before persist.
    Set(ExecutionLimits),
}

/// Patch body. Only non-`None` fields are applied.
///
/// Immutable fields (`id`, `kind`, `role`, `owning_org`) are rejected
/// if the client supplies them, because every non-`None` value is a
/// change request — the spec explicitly forbids these.
#[derive(Debug, Clone, Default)]
pub struct UpdateAgentPatch {
    // Identity / immutables — if any are `Some`, we reject early.
    pub new_kind: Option<AgentKind>,
    pub new_role: Option<AgentRole>,
    pub new_owning_org: Option<domain::model::ids::OrgId>,

    // Mutable agent-row fields.
    pub display_name: Option<String>,

    // Mutable profile-row fields.
    pub parallelize: Option<u32>,
    pub blueprint: Option<PhiCoreAgentProfile>,
    /// Per-agent `ModelConfig` binding (M5 / C-M5-5). When `Some`,
    /// the update validates the supplied id against the owning
    /// org's `ModelRuntime` catalogue and rejects with 409
    /// `ACTIVE_SESSIONS_BLOCK_MODEL_CHANGE` if the agent has any
    /// `Running`-state sessions. Setting `Some(String::new())`
    /// explicitly is rejected — use `None` (unchanged) instead.
    pub model_config_id: Option<String>,

    // ExecutionLimits override path.
    pub execution_limits: ExecutionLimitsPatch,

    // Metadata.
    pub actor: AgentId,
    pub now: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct UpdatedAgent {
    pub agent_id: AgentId,
    /// Absent when the patch was a no-op (no fields changed, no
    /// audit emitted).
    pub audit_event_id: Option<AuditEventId>,
    /// Current source of truth for ExecutionLimits after the patch.
    pub execution_limits_source: ExecutionLimitsSource,
}

pub async fn update_agent_profile(
    repo: Arc<dyn Repository>,
    audit: Arc<dyn AuditEmitter>,
    agent_id: AgentId,
    patch: UpdateAgentPatch,
) -> Result<UpdatedAgent, AgentError> {
    // ---- Reject immutable-field changes up-front ---------------------------
    if patch.new_kind.is_some() {
        return Err(AgentError::ImmutableFieldChanged("kind"));
    }
    if patch.new_role.is_some() {
        return Err(AgentError::ImmutableFieldChanged("role"));
    }
    if patch.new_owning_org.is_some() {
        return Err(AgentError::ImmutableFieldChanged("owning_org"));
    }

    // ---- Load current state ----------------------------------------------
    let agent = repo
        .get_agent(agent_id)
        .await
        .map_err(|e| AgentError::Repository(e.to_string()))?
        .ok_or(AgentError::AgentNotFound(agent_id))?;

    // System agents are read-only per plan D3.
    if matches!(agent.role, Some(AgentRole::System)) {
        return Err(AgentError::SystemAgentReadOnly);
    }

    let current_profile = repo
        .get_agent_profile_for_agent(agent_id)
        .await
        .map_err(|e| AgentError::Repository(e.to_string()))?;
    let current_override = repo
        .get_agent_execution_limits_override(agent_id)
        .await
        .map_err(|e| AgentError::Repository(e.to_string()))?;

    // ---- Validate patch field-by-field ------------------------------------
    if let Some(p) = patch.parallelize {
        if p == 0 || p > super::create::PARALLELIZE_MAX_CAP {
            return Err(AgentError::ParallelizeCeilingExceeded {
                requested: p,
                ceiling: super::create::PARALLELIZE_MAX_CAP,
            });
        }
    }

    // ---- C-M5-5 flip: ModelConfig change + active-session gate ----------
    //
    // At M5/P4 the `model_config_id` field on `AgentProfile` is
    // live. Validate the new binding against the owning org's
    // `ModelRuntime` catalogue; reject with 409
    // `ACTIVE_SESSIONS_BLOCK_MODEL_CHANGE` if the agent is
    // currently running any sessions. The flip honours the P2
    // invariant that `count_active_sessions_for_agent` is no
    // longer a stub — real counts flow through.
    //
    // The `blueprint.config_id` field on phi-core's `AgentProfile`
    // is a stable identity for loop_id composition, NOT a
    // ModelConfig reference — it stays orthogonal to this gating
    // (ADR-0029 §D29.3).
    if let Some(new_model_config_id) = patch.model_config_id.as_ref() {
        if new_model_config_id.trim().is_empty() {
            return Err(AgentError::ImmutableFieldChanged("model_config_id"));
        }
        // Validate the id resolves to an active runtime row in the
        // catalogue.
        let runtimes = repo
            .list_model_providers(true)
            .await
            .map_err(|e| AgentError::Repository(e.to_string()))?;
        let rt = runtimes
            .iter()
            .find(|r| r.id.to_string() == *new_model_config_id)
            .ok_or(AgentError::ImmutableFieldChanged(
                "model_config_id (unknown model runtime)",
            ))?;
        if rt.archived_at.is_some() {
            return Err(AgentError::ImmutableFieldChanged(
                "model_config_id (model runtime archived)",
            ));
        }
        // Active-session gate.
        let active = repo
            .count_active_sessions_for_agent(agent_id)
            .await
            .map_err(|e| AgentError::Repository(e.to_string()))?;
        if active > 0 {
            return Err(AgentError::ActiveSessionsBlockModelChange);
        }
    }

    // ---- Apply ExecutionLimits patch --------------------------------------
    let (limits_source_after, limits_changed) = match &patch.execution_limits {
        ExecutionLimitsPatch::Unchanged => {
            let src = if current_override.is_some() {
                ExecutionLimitsSource::Override
            } else {
                ExecutionLimitsSource::Inherit
            };
            (src, false)
        }
        ExecutionLimitsPatch::Revert => {
            // Idempotent — safe whether or not a row exists.
            let changed = current_override.is_some();
            clear_override(repo.clone(), agent_id).await?;
            (ExecutionLimitsSource::Inherit, changed)
        }
        ExecutionLimitsPatch::Set(new_limits) => {
            let row = AgentExecutionLimitsOverride {
                id: current_override
                    .as_ref()
                    .map(|c| c.id)
                    .unwrap_or_else(NodeId::new),
                owning_agent: agent_id,
                limits: new_limits.clone(),
                created_at: current_override
                    .as_ref()
                    .map(|c| c.created_at)
                    .unwrap_or(patch.now),
            };
            apply_override(repo.clone(), &row).await?;
            let changed = match current_override.as_ref() {
                None => true,
                Some(cur) => !execution_limits_equal(&cur.limits, new_limits),
            };
            (ExecutionLimitsSource::Override, changed)
        }
    };

    // ---- Apply profile-row changes ----------------------------------------
    // We upsert the profile row if any profile field changed.
    let mut profile_changed = false;
    let mut diff = AgentProfilePatchDiff::default();
    let mut next_profile = current_profile.clone();

    if let Some(new_parallelize) = patch.parallelize {
        match next_profile.as_mut() {
            Some(p) if p.parallelize != new_parallelize => {
                diff.parallelize = Some((p.parallelize, new_parallelize));
                p.parallelize = new_parallelize;
                profile_changed = true;
            }
            None => {
                // Profile doesn't exist yet — create one.
                let bp = patch.blueprint.clone().unwrap_or_default();
                next_profile = Some(AgentProfile {
                    id: NodeId::new(),
                    agent_id,
                    parallelize: new_parallelize,
                    blueprint: bp,
                    model_config_id: None,
                    mock_response: None,
                    created_at: patch.now,
                });
                profile_changed = true;
                diff.parallelize = Some((1, new_parallelize));
            }
            _ => {}
        }
    }

    if let Some(new_blueprint) = patch.blueprint.as_ref() {
        match next_profile.as_mut() {
            Some(p) => {
                let b_before = &p.blueprint;
                let b_after = new_blueprint;
                if b_before.system_prompt != b_after.system_prompt {
                    diff.system_prompt = Some((
                        b_before.system_prompt.clone(),
                        b_after.system_prompt.clone(),
                    ));
                    profile_changed = true;
                }
                if b_before.temperature != b_after.temperature {
                    diff.temperature = Some((b_before.temperature, b_after.temperature));
                    profile_changed = true;
                }
                if b_before.thinking_level != b_after.thinking_level {
                    diff.thinking_level = Some((b_before.thinking_level, b_after.thinking_level));
                    profile_changed = true;
                }
                // Note: blueprint.max_tokens edits don't have a
                // dedicated diff slot at M4 — captured under the
                // `system_prompt` path when relevant or left as a
                // forward-compat field in `AgentProfilePatchDiff`.
                if profile_changed {
                    p.blueprint = new_blueprint.clone();
                }
            }
            None => {
                next_profile = Some(AgentProfile {
                    id: NodeId::new(),
                    agent_id,
                    parallelize: patch.parallelize.unwrap_or(1),
                    blueprint: new_blueprint.clone(),
                    model_config_id: None,
                    mock_response: None,
                    created_at: patch.now,
                });
                profile_changed = true;
            }
        }
    }

    // ---- Apply `model_config_id` (C-M5-5 change arm) --------------------
    if let Some(new_model_config_id) = patch.model_config_id.as_ref() {
        match next_profile.as_mut() {
            Some(p) => {
                if p.model_config_id.as_deref() != Some(new_model_config_id.as_str()) {
                    p.model_config_id = Some(new_model_config_id.clone());
                    profile_changed = true;
                }
            }
            None => {
                // Profile didn't exist; create one carrying the new
                // binding + a default blueprint.
                let bp = patch.blueprint.clone().unwrap_or_default();
                next_profile = Some(AgentProfile {
                    id: NodeId::new(),
                    agent_id,
                    parallelize: patch.parallelize.unwrap_or(1),
                    blueprint: bp,
                    model_config_id: Some(new_model_config_id.clone()),
                    mock_response: None,
                    created_at: patch.now,
                });
                profile_changed = true;
            }
        }
    }

    if profile_changed {
        if let Some(profile_row) = next_profile.as_ref() {
            // If the agent had NO prior profile row, the UPSERT
            // path's UPDATE arm is a no-op (M5/P2 drift D2.1
            // family). Use `create_agent_profile` for the
            // fresh-row case; `upsert_agent_profile` for
            // existing-row mutation.
            let write_result = if current_profile.is_some() {
                repo.upsert_agent_profile(profile_row).await
            } else {
                repo.create_agent_profile(profile_row).await
            };
            write_result.map_err(|e| AgentError::Repository(e.to_string()))?;
        }
    }

    // ---- Apply agent-row changes (display_name) ---------------------------
    let mut agent_changed = false;
    if let Some(new_name) = patch.display_name.as_ref() {
        let trimmed = new_name.trim();
        if trimmed.is_empty() {
            return Err(AgentError::Validation(
                "display_name must be non-empty".into(),
            ));
        }
        if trimmed != agent.display_name {
            diff.display_name = Some((agent.display_name.clone(), trimmed.to_string()));
            let next_agent = Agent {
                display_name: trimmed.to_string(),
                ..agent.clone()
            };
            repo.upsert_agent(&next_agent)
                .await
                .map_err(|e| AgentError::Repository(e.to_string()))?;
            agent_changed = true;
        }
    }

    // ---- Record limits-source flip in the diff ---------------------------
    let limits_source_before = if current_override.is_some() {
        ExecutionLimitsSource::Override
    } else {
        ExecutionLimitsSource::Inherit
    };
    if limits_source_before != limits_source_after || limits_changed {
        diff.execution_limits_source = Some((limits_source_before, limits_source_after));
    }

    // ---- Emit audit only if something changed ----------------------------
    let any_change = agent_changed || profile_changed || limits_changed;
    if !any_change || diff.is_empty() {
        return Ok(UpdatedAgent {
            agent_id,
            audit_event_id: None,
            execution_limits_source: limits_source_after,
        });
    }

    let org = agent
        .owning_org
        .ok_or_else(|| AgentError::Validation("agent has no owning_org".into()))?;
    let event = agent_profile_updated(
        patch.actor,
        agent_id,
        org,
        diff,
        None, // no provenance AR at M4
        patch.now,
    );
    let event_id = event.event_id;
    audit
        .emit(event)
        .await
        .map_err(|e| AgentError::AuditEmit(e.to_string()))?;

    Ok(UpdatedAgent {
        agent_id,
        audit_event_id: Some(event_id),
        execution_limits_source: limits_source_after,
    })
}

fn execution_limits_equal(a: &ExecutionLimits, b: &ExecutionLimits) -> bool {
    a.max_turns == b.max_turns
        && a.max_total_tokens == b.max_total_tokens
        && a.max_duration == b.max_duration
        && a.max_cost == b.max_cost
}

// Suppress unused-import warning when org_ceiling_for_agent is only
// used by the apply_override path (which is called from the Set arm).
#[allow(dead_code)]
fn _keep_org_ceiling_live(
    repo: Arc<dyn Repository>,
    aid: AgentId,
) -> impl std::future::Future<Output = Result<ExecutionLimits, AgentError>> {
    org_ceiling_for_agent(repo, aid)
}
