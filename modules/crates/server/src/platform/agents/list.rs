//! `GET /api/v0/orgs/:org_id/agents` — page 08 agent roster list.
//!
//! ## phi-core leverage
//!
//! Q1 **none**, Q2 **none**, Q3 **none** — the roster payload carries
//! phi governance fields only. Per-agent `AgentProfile` (which
//! wraps `phi_core::AgentProfile`) is a **separate node** surfaced on
//! page 09's edit view, not here. Keeping blueprint off the list
//! response preserves the "index view is summary, detail view carries
//! phi-core shape" split established by M3/P4 (`orgs/list.rs` vs
//! `orgs/show.rs`).
//!
//! ## Scope at M4/P4
//!
//! - Optional role filter (`Some(AgentRole)` for chip-driven view,
//!   `None` for everybody).
//! - Optional free-text search over `display_name` (case-insensitive
//!   substring; trimmed; empty-after-trim is rejected with
//!   `Validation`).
//! - No pagination at M4 — orgs have tens of agents on page 08. P8's
//!   dashboard rewrite still operates on the same underlying repo
//!   method, so adding cursor pagination later is additive (no shape
//!   change to existing response items).

use std::sync::Arc;

use domain::model::ids::OrgId;
use domain::model::nodes::{Agent, AgentRole};
use domain::repository::Repository;

use super::AgentError;

/// A single row in the roster table. Deliberately thin: one row per
/// agent; all fields are phi governance. No phi-core types in the
/// struct.
#[derive(Debug, Clone)]
pub struct AgentRosterRow {
    pub agent: Agent,
}

/// Input to [`list_agents`].
#[derive(Debug, Clone, Default)]
pub struct ListAgentsInput {
    /// When `Some`, filter to agents whose `role == Some(role)`.
    pub role: Option<AgentRole>,
    /// When `Some`, filter `display_name` by case-insensitive substring.
    /// Empty-after-trim is a `Validation` error so the caller
    /// surfaces "enter a search term" rather than silently matching
    /// everybody.
    pub search: Option<String>,
}

pub async fn list_agents(
    repo: Arc<dyn Repository>,
    org: OrgId,
    input: ListAgentsInput,
) -> Result<Vec<AgentRosterRow>, AgentError> {
    // Precondition: the org must exist. Returning a 404 here (via the
    // handler layer) beats silently returning an empty list, which
    // would mask typos in operator URLs.
    let exists = repo
        .get_organization(org)
        .await
        .map_err(|e| AgentError::Repository(e.to_string()))?;
    if exists.is_none() {
        return Err(AgentError::OrgNotFound(org));
    }

    let search = match input.search.as_deref() {
        None => None,
        Some(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return Err(AgentError::Validation(
                    "search term must be non-empty after trimming whitespace".into(),
                ));
            }
            Some(trimmed.to_lowercase())
        }
    };

    let agents = repo
        .list_agents_in_org_by_role(org, input.role)
        .await
        .map_err(|e| AgentError::Repository(e.to_string()))?;

    let filtered: Vec<AgentRosterRow> = agents
        .into_iter()
        .filter(|a| match &search {
            None => true,
            Some(needle) => a.display_name.to_lowercase().contains(needle),
        })
        .map(|agent| AgentRosterRow { agent })
        .collect();

    Ok(filtered)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use domain::audit::AuditClass;
    use domain::in_memory::InMemoryRepository;
    use domain::model::composites_m3::ConsentPolicy;
    use domain::model::ids::{AgentId, OrgId};
    use domain::model::nodes::{Agent, AgentKind, AgentRole, Organization};

    fn org(id: OrgId) -> Organization {
        Organization {
            id,
            display_name: format!("org-{id}"),
            vision: None,
            mission: None,
            consent_policy: ConsentPolicy::Implicit,
            audit_class_default: AuditClass::Logged,
            authority_templates_enabled: vec![],
            defaults_snapshot: None,
            default_model_provider: None,
            system_agents: vec![],
            created_at: Utc::now(),
        }
    }

    fn agent(owning: OrgId, kind: AgentKind, role: Option<AgentRole>, name: &str) -> Agent {
        Agent {
            id: AgentId::new(),
            kind,
            display_name: name.into(),
            owning_org: Some(owning),
            role,
            created_at: Utc::now(),
        }
    }

    async fn setup() -> (Arc<dyn Repository>, OrgId) {
        let repo: Arc<dyn Repository> = Arc::new(InMemoryRepository::new());
        let id = OrgId::new();
        repo.create_organization(&org(id)).await.unwrap();
        repo.create_agent(&agent(
            id,
            AgentKind::Human,
            Some(AgentRole::Executive),
            "Alice",
        ))
        .await
        .unwrap();
        repo.create_agent(&agent(
            id,
            AgentKind::Llm,
            Some(AgentRole::Intern),
            "iris-bot",
        ))
        .await
        .unwrap();
        repo.create_agent(&agent(
            id,
            AgentKind::Llm,
            Some(AgentRole::Contract),
            "alpha-bot",
        ))
        .await
        .unwrap();
        (repo, id)
    }

    #[tokio::test]
    async fn list_all_returns_every_agent_in_org() {
        let (repo, id) = setup().await;
        let rows = list_agents(repo, id, ListAgentsInput::default())
            .await
            .unwrap();
        assert_eq!(rows.len(), 3);
    }

    #[tokio::test]
    async fn filter_by_role_returns_only_matching_rows() {
        let (repo, id) = setup().await;
        let rows = list_agents(
            repo,
            id,
            ListAgentsInput {
                role: Some(AgentRole::Intern),
                search: None,
            },
        )
        .await
        .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].agent.display_name, "iris-bot");
    }

    #[tokio::test]
    async fn search_is_case_insensitive_substring() {
        let (repo, id) = setup().await;
        let rows = list_agents(
            repo.clone(),
            id,
            ListAgentsInput {
                role: None,
                search: Some("BOT".into()),
            },
        )
        .await
        .unwrap();
        assert_eq!(rows.len(), 2);

        let rows = list_agents(
            repo,
            id,
            ListAgentsInput {
                role: None,
                search: Some("ali".into()),
            },
        )
        .await
        .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].agent.display_name, "Alice");
    }

    #[tokio::test]
    async fn empty_search_is_validation_error() {
        let (repo, id) = setup().await;
        let err = list_agents(
            repo,
            id,
            ListAgentsInput {
                role: None,
                search: Some("   ".into()),
            },
        )
        .await
        .unwrap_err();
        assert!(matches!(err, AgentError::Validation(_)));
    }

    #[tokio::test]
    async fn unknown_org_returns_org_not_found() {
        let repo: Arc<dyn Repository> = Arc::new(InMemoryRepository::new());
        let err = list_agents(repo, OrgId::new(), ListAgentsInput::default())
            .await
            .unwrap_err();
        assert!(matches!(err, AgentError::OrgNotFound(_)));
    }

    #[tokio::test]
    async fn combined_role_and_search_intersect() {
        let (repo, id) = setup().await;
        let rows = list_agents(
            repo,
            id,
            ListAgentsInput {
                role: Some(AgentRole::Contract),
                search: Some("bot".into()),
            },
        )
        .await
        .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].agent.display_name, "alpha-bot");
    }
}
