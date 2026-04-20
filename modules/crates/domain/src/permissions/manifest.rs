//! Engine-facing [`Manifest`] + [`CheckContext`] + [`ToolCall`].
//!
//! These types are the **input contract** to [`crate::permissions::check`].
//! Callers (server handlers, bootstrap flow) project the relevant graph nodes
//! onto these shapes before calling the engine. The engine never touches
//! storage.
//!
//! ## Where the fields come from
//!
//! | Field | Concept-doc reference |
//! |---|---|
//! | `Manifest.actions` / `resource` / `transitive` / `constraints` / `kinds` | `permissions/04-manifest-and-resolution.md` §Tool Authority Manifest |
//! | `CheckContext.agent_grants` | `HOLDS_GRANT` from the agent |
//! | `CheckContext.project_grants` / `org_grants` | `HOLDS_GRANT` from the agent's project / org |
//! | `CheckContext.ceiling_grants` | org caps project, project caps agent (see §Ceiling Enforcement) |
//! | `CheckContext.catalogue` | `resources_catalogue` per owning org (§Resource Catalogue) |
//! | `ToolCall.target_uri` / `target_tags` | the entity the call is about (used for selector matching) |
//! | `ToolCall.constraint_context` | run-time values the manifest's constraints check against |

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use std::collections::HashSet;

use crate::model::ids::{AgentId, AuthRequestId, OrgId, ProjectId};
use crate::model::nodes::{Grant, ToolAuthorityManifest};

use super::catalogue::CatalogueLookup;

/// The engine's view of a tool authority manifest. This is a projection of
/// the persisted [`ToolAuthorityManifest`] graph node onto the fields the
/// engine actually reads.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Manifest {
    /// Actions the tool performs (`execute`, `read`, `modify`, …).
    pub actions: Vec<String>,
    /// Primary resource classes the tool touches. Each value is either a
    /// fundamental name (e.g. `filesystem_object`) or a composite name
    /// (e.g. `external_service_object`). Composites expand to fundamentals
    /// at Step 1.
    pub resource: Vec<String>,
    /// Transitive classes the tool can reach (per `bash`'s canonical
    /// over-declaration pattern). Same naming rules as `resource`.
    pub transitive: Vec<String>,
    /// Constraint names the manifest requires to be populated (e.g.
    /// `command_pattern`, `path_prefix`, `sandbox`). Step 4 checks each
    /// against the winning grant.
    pub constraints: Vec<String>,
    /// `#kind:` filters that must be carried on the target's tags.
    pub kinds: Vec<String>,
}

impl Manifest {
    /// Lift the persisted graph node into the engine's input shape.
    pub fn from_node(m: &ToolAuthorityManifest) -> Self {
        Self {
            actions: m.actions.clone(),
            resource: m.resource.clone(),
            transitive: m.transitive.clone(),
            constraints: m.constraints.clone(),
            kinds: m.kinds.clone(),
        }
    }

    /// True when the manifest declares neither resources nor actions; the
    /// engine treats this as a Step 1 failure.
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty() || (self.resource.is_empty() && self.transitive.is_empty())
    }
}

/// One runtime invocation the engine is being asked about. Reused across the
/// pipeline: `target_uri` + `target_tags` feed selector matching (Step 3);
/// `constraint_context` feeds Step 4; `target_agent` feeds Step 6 consent
/// lookup.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolCall {
    /// The URI of the entity the call is about (matched against grant
    /// `resource.uri`). Empty string when the call operates on no specific
    /// entity (e.g. `allocate` on a class rather than an instance).
    pub target_uri: String,
    /// Tags carried by the target entity. Includes the `#kind:{type}` type
    /// tag and the `{kind}:{instance_id}` self-identity tag per composite
    /// convention.
    pub target_tags: Vec<String>,
    /// The agent the call targets, when applicable (Template A/B/C/D
    /// consent gating uses this).
    pub target_agent: Option<AgentId>,
    /// Values the manifest's constraints check against (e.g. the actual
    /// `command_pattern`, `timeout_secs`, `sandbox`).
    pub constraint_context: HashMap<String, serde_json::Value>,
}

/// The engine's full input context for one check. Construct once per call
/// site; the engine borrows it for the duration of [`crate::permissions::check`].
pub struct CheckContext<'a> {
    /// The calling agent.
    pub agent: AgentId,
    /// The org whose scope applies (usually the agent's `owning_org`).
    pub current_org: Option<OrgId>,
    /// The project scope the call runs under, if any.
    pub current_project: Option<ProjectId>,
    /// Grants the agent holds directly.
    pub agent_grants: &'a [Grant],
    /// Grants the agent's current project holds (propagate to members).
    pub project_grants: &'a [Grant],
    /// Grants the agent's current org holds (org-level ceiling + grants).
    pub org_grants: &'a [Grant],
    /// Ceiling grants above the agent (org caps project caps agent).
    /// Step 2a clamps every candidate against these.
    pub ceiling_grants: &'a [Grant],
    /// The resources catalogue for Step 0.
    pub catalogue: &'a dyn CatalogueLookup,
    /// Consent records keyed by (subordinate, org) — Step 6 looks up here.
    /// Missing entries count as "no consent recorded yet" and yield `Pending`.
    pub consents: &'a ConsentIndex,
    /// Which Auth Request ids are known to be template-gated (Templates
    /// A/B/C/D). Step 6 only runs when a winning grant's `descends_from`
    /// is in this set. Empty in M1 — P4 populates it once the Auth Request
    /// state machine tracks template provenance.
    pub template_gated_auth_requests: &'a HashSet<AuthRequestId>,
    /// The invocation being checked.
    pub call: ToolCall,
}

/// Lookup from `(subordinate, org)` → `has_acknowledged_consent`. Concrete
/// implementations live in P4; P3 tests use [`ConsentIndex::empty`] /
/// [`ConsentIndex::from_pairs`].
#[derive(Debug, Clone, Default)]
pub struct ConsentIndex {
    pairs: std::collections::HashSet<(AgentId, OrgId)>,
}

impl ConsentIndex {
    /// Empty index — every consent lookup returns `false`.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Build an index from an iterator of `(subordinate, org)` pairs that
    /// have recorded an `Acknowledged` consent.
    pub fn from_pairs<I: IntoIterator<Item = (AgentId, OrgId)>>(pairs: I) -> Self {
        Self {
            pairs: pairs.into_iter().collect(),
        }
    }

    /// Does this subordinate have a recorded consent under this org?
    pub fn is_acknowledged(&self, subordinate: AgentId, org: OrgId) -> bool {
        self.pairs.contains(&(subordinate, org))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_manifest_detected() {
        assert!(Manifest::default().is_empty());
        let only_actions = Manifest {
            actions: vec!["read".into()],
            ..Default::default()
        };
        assert!(only_actions.is_empty());
        let only_resource = Manifest {
            resource: vec!["filesystem_object".into()],
            ..Default::default()
        };
        assert!(only_resource.is_empty());
    }

    #[test]
    fn manifest_with_action_and_resource_is_non_empty() {
        let m = Manifest {
            actions: vec!["execute".into()],
            resource: vec!["process_exec_object".into()],
            ..Default::default()
        };
        assert!(!m.is_empty());
    }

    #[test]
    fn consent_index_empty_returns_false_for_every_pair() {
        let idx = ConsentIndex::empty();
        let a = AgentId::new();
        let o = OrgId::new();
        assert!(!idx.is_acknowledged(a, o));
    }

    #[test]
    fn consent_index_roundtrips_pairs() {
        let a = AgentId::new();
        let o = OrgId::new();
        let idx = ConsentIndex::from_pairs([(a, o)]);
        assert!(idx.is_acknowledged(a, o));
        assert!(!idx.is_acknowledged(AgentId::new(), o));
    }
}
