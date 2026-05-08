//! `build_session_launch_manifest` — the synthetic launch manifest
//! the engine reads when gating session launch.
//!
//! ## Shape (CH-15 / ADR-0054 §D54.1 + §D54.2 — F3.A user-locked)
//!
//! - `actions = [Action::Read, Action::Inspect, Action::List]` —
//!   matches Template A's `[Read, Inspect, List]` issuance shape
//!   exactly (per `templates/a.rs:fire_grant_on_lead_assignment`),
//!   giving the launch handler a clean grant-shape match against
//!   the paired Template A session-object grant.
//! - `resource = ["session_object"]` — the composite the lead's
//!   paired grant covers; expands to `[DataObject, Tag]` per concept
//!   doc 01.
//! - `transitive`, `constraints`, `constraint_requirements`, `kinds`
//!   all empty — launch is not a tool-invocation reach.
//!
//! ## Forward-scope re-interpretation (ADR-0054 §D54.2 / §D54.8)
//!
//! The forward-scope row literal text *"actions `session.start` /
//! `session.tool_invoke` / `session.read_memory`"* is a scoping-gloss,
//! NOT a literal Action variant set. Concept doc 03's closed 34-verb
//! vocabulary takes precedence; the launch boundary gates only the
//! `session.start` semantics → `[Read, Inspect, List]` on
//! `session_object`. `tool_invoke` + `read_memory` are runtime
//! per-tool manifest reaches that ship at M6+.
//!
//! ## Call sites (CH-15 / ADR-0054 §D54.7)
//!
//! Both `preview.rs::preview_session` and
//! `launch.rs::gate_session_launch_consent` call this builder so the
//! preview-launch parity invariant holds — divergence at the manifest
//! layer would re-open D4.1's "advisory layer" pattern at the
//! consent boundary.

use std::collections::HashMap;

use crate::model::ids::ProjectId;
use crate::permissions::action::Action;
use crate::permissions::manifest::Manifest;

/// Build the synthetic launch-time manifest for a session running
/// against project `project_id`.
///
/// Pure — no I/O, no Repository. See module docstring for the shape
/// + forward-scope re-interpretation rationale.
///
/// The `project_id` parameter is preserved on the signature even
/// though today's body does not reference it — the builder's
/// signature is part of the forward-scope row literal
/// `build_session_launch_manifest(project_id, tools)` and ADR-0054
/// §D54.1. Future M6+ extensions will use it to vary the manifest
/// per-project (e.g., per-project allowed-tool sets); preserving the
/// signature now avoids a callsite cascade later.
pub fn build_session_launch_manifest(_project_id: ProjectId) -> Manifest {
    Manifest {
        actions: vec![Action::Read, Action::Inspect, Action::List],
        resource: vec!["session_object".to_string()],
        transitive: vec![],
        constraints: vec![],
        constraint_requirements: HashMap::new(),
        kinds: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// CH-15 / ADR-0054 §D54.1 + §D54.2 — the builder's `actions`
    /// match Template A's `[Read, Inspect, List]` issuance shape
    /// exactly, giving the launch-time engine call a clean grant
    /// match.
    #[test]
    fn builder_produces_action_set_matching_template_a_grant_action_set() {
        let m = build_session_launch_manifest(ProjectId::new());
        assert_eq!(m.actions, vec![Action::Read, Action::Inspect, Action::List]);
    }

    /// CH-15 / ADR-0054 §D54.1 — the resource is the `session_object`
    /// composite (expands to `[DataObject, Tag]` per concept doc 01).
    #[test]
    fn builder_resource_is_session_object_composite() {
        let m = build_session_launch_manifest(ProjectId::new());
        assert_eq!(m.resource, vec!["session_object".to_string()]);
        assert!(m.transitive.is_empty());
    }

    /// CH-15: `constraints`, `constraint_requirements`, and `kinds`
    /// are empty — launch is not a tool-invocation reach.
    #[test]
    fn builder_carries_no_constraints_or_kind_filters() {
        let m = build_session_launch_manifest(ProjectId::new());
        assert!(m.constraints.is_empty());
        assert!(m.constraint_requirements.is_empty());
        assert!(m.kinds.is_empty());
    }

    /// CH-15: the manifest is non-empty in the engine's `is_empty`
    /// sense — both `actions` AND `resource` carry at least one
    /// entry.
    #[test]
    fn builder_produces_non_empty_manifest() {
        let m = build_session_launch_manifest(ProjectId::new());
        assert!(!m.is_empty());
    }
}
