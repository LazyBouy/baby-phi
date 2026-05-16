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

use crate::model::ids::{AgentId, AuthRequestId, OrgId, ProjectId, SessionId};
use crate::model::nodes::{ConsentState, Grant, TimeoutResponse, ToolAuthorityManifest};

use super::action::Action;
use super::catalogue::CatalogueLookup;
use super::selector::SetRefRegistry;

pub mod validator;

/// The engine's view of a tool authority manifest. This is a projection of
/// the persisted [`ToolAuthorityManifest`] graph node onto the fields the
/// engine actually reads.
///
/// **CH-15 / ADR-0054 §D54.1** — `PartialEq` derived (Eq omitted
/// because `serde_json::Value` may carry `f64` via Number; `Eq` is
/// unsound there). The builder's tests use `assert_eq!` against
/// hand-built reference manifests — every other field is
/// well-behaved under `PartialEq`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    /// Actions the tool performs (`execute`, `read`, `modify`, …).
    pub actions: Vec<Action>,
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
    /// Per-constraint **value requirements**. When a constraint name is
    /// present in `constraints` AND in `constraint_requirements`, Step
    /// 4 checks BOTH the key's presence in the invocation's
    /// `constraint_context` AND strict equality of the context value
    /// against this required value. When a constraint is only in
    /// `constraints` (not in `constraint_requirements`), Step 4 checks
    /// only presence, matching M1 behaviour.
    ///
    /// Value-match is the M2 widening that unlocks the
    /// `purpose=reveal` contract for the credentials vault (page 04):
    /// the manifest declares `constraint_requirements["purpose"] =
    /// "reveal"` so invocations that forget to assert the purpose
    /// land at `FailedStep::Constraint` instead of silently exposing
    /// plaintext.
    ///
    /// Richer comparators (pattern match, lattice, numeric ranges)
    /// land in M3 when the constraint lattice machinery arrives.
    #[serde(default)]
    pub constraint_requirements: std::collections::HashMap<String, serde_json::Value>,
    /// `#kind:` filters that must be carried on the target's tags.
    pub kinds: Vec<String>,
}

impl Manifest {
    /// Lift the persisted graph node into the engine's input shape.
    ///
    /// `constraint_requirements` starts empty — `ToolAuthorityManifest`
    /// (the graph node) doesn't persist per-constraint value
    /// requirements in M2. Callers that need value-match (e.g. the
    /// credentials-vault `reveal` path in page 04) construct `Manifest`
    /// directly rather than going through the node. M3 extends
    /// `ToolAuthorityManifest` when the constraint lattice lands.
    pub fn from_node(m: &ToolAuthorityManifest) -> Self {
        Self {
            actions: m.actions.clone(),
            resource: m.resource.clone(),
            transitive: m.transitive.clone(),
            constraints: m.constraints.clone(),
            constraint_requirements: std::collections::HashMap::new(),
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
    /// CH-11 / ADR-0048 §D48.3 — Per-Session ambient context. The session
    /// the call runs under, when applicable. `None` for class-level
    /// invocations. Populated by the launch handler from `session.id`;
    /// preview / handler_support test fixtures pass `None` and rely on
    /// `approval_mode == Implicit` short-circuiting Step 6.
    ///
    /// Mirrors `current_org` / `current_project` ambient-context
    /// pattern. Step 6's `PerSession` policy branch reads this; if the
    /// engine encounters a `PerSession`-policy grant with
    /// `current_session = None`, it returns
    /// `Decision::Denied(NoSessionContext)` — the caller is responsible
    /// for populating the field at the call site.
    pub current_session: Option<SessionId>,
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
    /// Consent records keyed by (subordinate, org, session_id) — Step 6
    /// looks up here. Missing entries count as "no consent recorded yet"
    /// and yield `Pending`.
    pub consents: &'a ConsentIndex,
    /// CH-11 / ADR-0048 §D48.4 — Default response when a `Requested`
    /// consent has crossed its deadline and the sweeper auto-flipped it
    /// to `TimedOut`. Step 6 consults this for the `TimedOut` arm of the
    /// response table: `Deny` → `Denied(ConsentTimedOutDeny)`, `Allow`
    /// → proceed to Allow. Defaults to `Deny` per concept doc 06 line
    /// 349 ("absence of consent is not consent"). The launch handler
    /// populates this from `org.approval_timeout_default_response`.
    pub timeout_default_response: TimeoutResponse,
    /// Which Auth Request ids are known to be template-gated (Templates
    /// A/B/C/D). Step 6 only runs when a winning grant's `descends_from`
    /// is in this set. Empty in M1 — P4 populates it once the Auth Request
    /// state machine tracks template provenance.
    ///
    /// CH-11 soft-deprecation: the engine prefers `winner.grant.approval_mode`
    /// for routing Step 6 logic. This field stays for back-compat with
    /// pre-CH-11 callers that haven't been migrated; new code populates
    /// `approval_mode` instead. Removal lands at a future chunk.
    pub template_gated_auth_requests: &'a HashSet<AuthRequestId>,
    /// Resolves `subset_of foo(args…)` set references during selector
    /// evaluation. Default at every call site is
    /// [`crate::permissions::selector::NoopSetRefRegistry`]; CH-15 wires
    /// the production registry. Trait-shaped per CH-K8S-PREP D33
    /// conforming criteria so a future remote-backed registry slots in
    /// without touching the engine.
    pub set_ref_registry: &'a dyn SetRefRegistry,
    /// CH-07 / ADR-0051 §D51.4 — Session's owning-organization tags
    /// parsed from `Session.tags` `org:<uuid>` prefixes.
    ///
    /// **Empty slice** = single-org (Shape A/D) path; the cascade
    /// reduces to today's behaviour (top-tier-then-tie-break) so M1
    /// callsite-count test invariants are preserved exactly. Concept
    /// 06 lines 14–53 reads scopes from `session.tags` directly; the
    /// launch handler computes this slice via
    /// [`parse_session_scope_tags`] before constructing the context.
    ///
    /// **Non-empty** = multi-scope (Shape B/C) — the cascade applies
    /// the 2-tier branching with intersection fallback per ADR-0051
    /// §D51.4 + §D51.5.
    pub session_org_tags: &'a [OrgId],
    /// CH-07 / ADR-0051 §D51.4 — Session's owning-project tags parsed
    /// from `Session.tags` `project:<uuid>` prefixes.
    ///
    /// **Empty slice** = no project (Shape D) OR single-project (Shape
    /// A); the cascade falls through to the org tier or stays at
    /// today's behaviour. Concept 06 lines 14–53 reads scopes from
    /// `session.tags` directly.
    pub session_project_tags: &'a [ProjectId],
    /// CH-25 / ADR-0060 §D60.3 — Orgs the calling [`agent`] owns via
    /// `Edge::Owns` graph edges (from-side `Agent`, to-side
    /// `OwnedResourceId::Org`). Pre-loaded by call-sites from
    /// [`crate::repository::Repository::list_agent_owned_orgs`] so the
    /// engine's [`crate::permissions::engine::step_2_resolve_grants`]
    /// can synth `[Allocate, Transfer]` candidate grants at the most-
    /// specific (`ScopeTier::Agent`) tier without an I/O call inside
    /// the pure engine.
    ///
    /// **Empty slice** = agent owns no orgs → no synth grants produced
    /// (the engine's synth loop is a no-op). This matches the M1-baseline
    /// `agent_grants` / `project_grants` / `org_grants` slice discipline:
    /// callers pass `&[]` when the tier is irrelevant.
    pub agent_owned_orgs: &'a [OrgId],
    /// CH-25 / ADR-0060 §D60.3 — Projects the calling [`agent`] owns via
    /// `Edge::Owns` graph edges (from-side `Agent`, to-side
    /// `OwnedResourceId::Project`). Symmetric peer to
    /// [`agent_owned_orgs`]; see its docs for the rationale and
    /// pre-load discipline.
    pub agent_owned_projects: &'a [ProjectId],
    /// The invocation being checked.
    pub call: ToolCall,
}

/// CH-07 / ADR-0051 §D51.4 — parse `org:<uuid>` and `project:<uuid>`
/// prefixes from a session's tag set into typed `OrgId` / `ProjectId`
/// vectors.
///
/// Returns deduplicated, deterministic-order vectors (insertion order
/// preserving first-occurrence; subsequent duplicates dropped). Tags
/// whose suffix is not a parseable UUID are silently skipped — the
/// cascade treats them as no-ops, matching CH-06's
/// `format!("org:{}", session.owning_org)` emission shape at
/// `events/listeners.rs:703-704`.
///
/// `session.tags` is the canonical multi-scope encoding per concept
/// doc 06 lines 14–53. The launch handler at
/// `server::platform::sessions::launch.rs` calls this helper before
/// constructing [`CheckContext`].
pub fn parse_session_scope_tags(session_tags: &[String]) -> (Vec<OrgId>, Vec<ProjectId>) {
    let mut orgs: Vec<OrgId> = Vec::new();
    let mut projects: Vec<ProjectId> = Vec::new();
    for tag in session_tags {
        if let Some(rest) = tag.strip_prefix("org:") {
            if let Ok(uuid) = uuid::Uuid::parse_str(rest) {
                let id = OrgId::from_uuid(uuid);
                if !orgs.contains(&id) {
                    orgs.push(id);
                }
            }
        } else if let Some(rest) = tag.strip_prefix("project:") {
            if let Ok(uuid) = uuid::Uuid::parse_str(rest) {
                let id = ProjectId::from_uuid(uuid);
                if !projects.contains(&id) {
                    projects.push(id);
                }
            }
        }
    }
    (orgs, projects)
}

/// Lookup from `(subordinate, org, Option<session_id>)` → [`ConsentState`].
///
/// CH-11 / ADR-0048 §D48.7 — extended from a `(subordinate, org)` →
/// `bool(acknowledged?)` projection to a full state-carrying map keyed
/// by the per-session axis. The launch handler builds this projection
/// from the persisted `Consent` rows for the calling agent's
/// subordinates before constructing [`CheckContext`].
///
/// **Back-compat**: [`ConsentIndex::is_acknowledged`] preserves M1
/// semantics — it returns `true` iff `(subordinate, org, None)` maps to
/// [`ConsentState::Acknowledged`]. Existing callers that don't carry a
/// session-id continue to work; the OneTime-policy lookup path uses
/// `None` as the session key.
///
/// **phi-core leverage**: none — phi-core has no consent concept.
#[derive(Debug, Clone, Default)]
pub struct ConsentIndex {
    /// All recorded consents keyed by `(subordinate, org, session_id)`.
    /// `session_id == None` rows cover Implicit / OneTime policies;
    /// `session_id == Some(id)` rows cover Per-Session policy.
    states: HashMap<(AgentId, OrgId, Option<SessionId>), ConsentState>,
}

impl ConsentIndex {
    /// Empty index — every consent lookup returns `None`.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Build an index from an iterator of `(subordinate, org)` pairs
    /// that have recorded an `Acknowledged` consent at the
    /// `session_id = None` axis. Back-compat constructor; new callers
    /// should prefer [`ConsentIndex::from_state_map`] which lets them
    /// distinguish per-session rows + non-Acknowledged states.
    pub fn from_pairs<I: IntoIterator<Item = (AgentId, OrgId)>>(pairs: I) -> Self {
        let states = pairs
            .into_iter()
            .map(|(a, o)| ((a, o, None), ConsentState::Acknowledged))
            .collect();
        Self { states }
    }

    /// CH-11 / ADR-0048 §D48.7 — build from a fully-typed state map.
    /// The launch handler's projection step uses this constructor: it
    /// fetches every `Consent` row for the calling subordinate and
    /// folds each into `((subordinate, scope.org, scope.session_id),
    /// state)`.
    pub fn from_state_map<I>(entries: I) -> Self
    where
        I: IntoIterator<Item = ((AgentId, OrgId, Option<SessionId>), ConsentState)>,
    {
        Self {
            states: entries.into_iter().collect(),
        }
    }

    /// CH-11 / ADR-0048 §D48.7 — full-state lookup. Returns `None` when
    /// no consent row exists for the triple. Step 6's response table
    /// maps `Some(state)` and `None` separately (the latter triggers
    /// the launch-handler mint path; the former routes per the response
    /// table per state).
    pub fn lookup(
        &self,
        subordinate: AgentId,
        org: OrgId,
        session_id: Option<SessionId>,
    ) -> Option<ConsentState> {
        self.states.get(&(subordinate, org, session_id)).copied()
    }

    /// Back-compat: does this subordinate have an `Acknowledged`
    /// consent at the `session_id = None` axis under this org?
    /// Preserved verbatim from CH-09's M1 semantics: callers without
    /// session info still work, and the OneTime-policy lookup path
    /// reads through this method via `lookup(subordinate, org, None)`.
    pub fn is_acknowledged(&self, subordinate: AgentId, org: OrgId) -> bool {
        self.lookup(subordinate, org, None) == Some(ConsentState::Acknowledged)
    }

    /// CH-11 / ADR-0048 §D48.7 — Per-Session-axis acknowledged check.
    /// Returns `true` iff `(subordinate, org, Some(session_id))` maps
    /// to [`ConsentState::Acknowledged`].
    pub fn is_acknowledged_for_session(
        &self,
        subordinate: AgentId,
        org: OrgId,
        session_id: SessionId,
    ) -> bool {
        self.lookup(subordinate, org, Some(session_id)) == Some(ConsentState::Acknowledged)
    }

    /// CH-11 / ADR-0048 §D48.9 — projection helper. Fetches every
    /// [`crate::model::nodes::Consent`] row for `subordinate` via the
    /// repository, then folds each `(scope.org, scope.session_id)` pair
    /// into the typed-state map keyed by
    /// `(subordinate, scope.org, scope.session_id)`.
    ///
    /// The launch handler calls this BEFORE constructing
    /// [`CheckContext`] so Step 6's `(target, org, session?)` lookups
    /// hit a fresh projection. Multiple rows for the same key collapse
    /// onto the most-recently-seen state (deterministic in tests but
    /// arbitrary across SurrealDB insertion order — production rarely
    /// has more than one row per key, since the engine only mints when
    /// the lookup is empty).
    ///
    /// The argument is the subordinate the engine is gating reads
    /// against — typically the `target_agent` on the [`ToolCall`].
    pub async fn project_from_repo(
        repo: &dyn crate::repository::Repository,
        subordinate: AgentId,
    ) -> crate::repository::RepositoryResult<Self> {
        let rows = repo.list_consents_for_subordinate(subordinate).await?;
        let mut states: HashMap<(AgentId, OrgId, Option<SessionId>), ConsentState> =
            HashMap::with_capacity(rows.len());
        for row in rows {
            states.insert(
                (subordinate, row.scope.org, row.scope.session_id),
                row.state,
            );
        }
        Ok(Self { states })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_manifest_detected() {
        assert!(Manifest::default().is_empty());
        let only_actions = Manifest {
            actions: vec![Action::Read],
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
            actions: vec![Action::Execute],
            resource: vec!["process_exec_object".into()],
            ..Default::default()
        };
        assert!(!m.is_empty());
    }

    /// CH-15 / ADR-0054 §D54.1 — `PartialEq` derive lets the builder
    /// + downstream tests assert byte-identical manifest construction.
    #[test]
    fn manifest_partialeq_derive_compiles_and_round_trips() {
        let a = Manifest {
            actions: vec![Action::Read, Action::Inspect, Action::List],
            resource: vec!["session_object".to_string()],
            transitive: vec![],
            constraints: vec![],
            constraint_requirements: HashMap::new(),
            kinds: vec![],
        };
        let b = a.clone();
        assert_eq!(a, b);
        let c = Manifest {
            actions: vec![Action::Read],
            ..a.clone()
        };
        assert_ne!(a, c);
    }

    #[test]
    fn consent_index_empty_returns_false_for_every_pair() {
        let idx = ConsentIndex::empty();
        let a = AgentId::new();
        let o = OrgId::new();
        assert!(!idx.is_acknowledged(a, o));
        assert!(idx.lookup(a, o, None).is_none());
    }

    #[test]
    fn consent_index_roundtrips_pairs() {
        let a = AgentId::new();
        let o = OrgId::new();
        let idx = ConsentIndex::from_pairs([(a, o)]);
        assert!(idx.is_acknowledged(a, o));
        assert!(!idx.is_acknowledged(AgentId::new(), o));
    }

    #[test]
    fn consent_index_from_pairs_back_compat_uses_none_session_axis() {
        // CH-11 back-compat invariant: the existing
        // `is_acknowledged(a, o)` semantics translate to a HashMap
        // entry `((a, o, None) -> Acknowledged)`.
        let a = AgentId::new();
        let o = OrgId::new();
        let idx = ConsentIndex::from_pairs([(a, o)]);
        assert_eq!(idx.lookup(a, o, None), Some(ConsentState::Acknowledged));
        // A different session-id MUST NOT match a `from_pairs`-built
        // index — that's the per-session discrimination boundary.
        assert!(idx.lookup(a, o, Some(SessionId::new())).is_none());
    }

    #[test]
    fn consent_index_from_state_map_preserves_session_axis() {
        let a = AgentId::new();
        let o = OrgId::new();
        let s = SessionId::new();
        let idx = ConsentIndex::from_state_map([
            ((a, o, None), ConsentState::Requested),
            ((a, o, Some(s)), ConsentState::Acknowledged),
        ]);
        assert_eq!(idx.lookup(a, o, None), Some(ConsentState::Requested));
        assert_eq!(idx.lookup(a, o, Some(s)), Some(ConsentState::Acknowledged));
        assert!(idx.is_acknowledged_for_session(a, o, s));
        // Per-Session row does NOT satisfy the `is_acknowledged`
        // back-compat predicate (which reads the `None` axis).
        assert!(!idx.is_acknowledged(a, o));
    }
}
