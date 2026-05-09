//! `build_session_observe_manifest` — the synthetic manifest the
//! engine reads when gating an SSE live-event tail subscription.
//!
//! ## Shape (CH-17 / ADR-0055 §D55.5 — F5.B user-locked)
//!
//! - `actions = [Action::Observe]` — single-action manifest covering
//!   the SSE live-event observability surface. Concept doc 03
//!   §"Standard Action Vocabulary" line 22 lists Observability's
//!   three verbs (`observe, log, attest`); concept doc 03
//!   §"Action × Fundamental Applicability Matrix" line 44 makes
//!   Observability universal across all 9 fundamentals (every
//!   fundamental admits `observe`). The SSE handler at
//!   `server::platform::sessions::events::open_live_stream` builds
//!   this manifest + invokes the same `check()` engine surface as
//!   session launch.
//! - `resource = ["session_object"]` — the same composite the
//!   launch builder uses; expands to `[DataObject, Tag]` per
//!   concept doc 01.
//! - `transitive`, `constraints`, `constraint_requirements`, `kinds`
//!   all empty — observability is a class-level reach (no per-tool
//!   constraint surface).
//!
//! ## Sibling-builder design (ADR-0055 §D55.5)
//!
//! Launch (preview + actual launch) reuses
//! [`super::session_launch::build_session_launch_manifest`] which
//! returns `[Read, Inspect, List]` on `session_object`. SSE uses
//! THIS builder which returns `[Observe]` on `session_object`. Both
//! flow through the engine's `check()` Step 3 (match each manifest
//! reach to ≥1 covering grant). Template A grants mint
//! `[Read, Inspect, List, Observe]` on `session_object` (extended at
//! CH-17 / ADR-0055 §D55.9 + migration 0016 backfills legacy grants)
//! so a project lead's grants cover both the launch reach AND the
//! SSE reach.
//!
//! ## Why `Observe` and not `Read`
//!
//! Pre-iter-2 (planner iter-1's F5.A recommendation), CH-17 reused
//! `[Read, Inspect, List]` for SSE-parity-with-launch. iter-2's
//! user-lock at F5.B selects `[Observe]` instead because:
//!   * `Action::Observe` is canonical pre-CH-17 at
//!     `domain/src/permissions/action.rs:73,282,322` (introduced at
//!     CH-04 / ADR-0043 alongside the closed 34-verb vocabulary).
//!   * Concept doc 03 line 44 makes Observability universal; using
//!     it on `session_object` exercises an already-honored verb,
//!     not an extension.
//!   * Live-event tailing IS an observability operation per concept
//!     nomenclature; using `[Read]` (a Data-category verb) blurs the
//!     read-the-terminal-state-row vs watch-events-stream
//!     distinction.

use std::collections::HashMap;

use crate::model::ids::ProjectId;
use crate::permissions::action::Action;
use crate::permissions::manifest::Manifest;

/// Build the synthetic SSE-observe manifest for a session running
/// against project `project_id`.
///
/// Pure — no I/O, no Repository. See module docstring for the shape
/// + F5.B rationale.
///
/// The `project_id` parameter is preserved on the signature for
/// symmetry with [`super::session_launch::build_session_launch_manifest`]
/// even though today's body does not reference it. Future M6+
/// extensions may vary the manifest per-project (e.g., per-project
/// observability filters); preserving the signature now avoids a
/// callsite cascade later.
pub fn build_session_observe_manifest(_project_id: ProjectId) -> Manifest {
    Manifest {
        actions: vec![Action::Observe],
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

    /// CH-17 / ADR-0055 §D55.5 — the builder's `actions` is the
    /// single-element vector `[Observe]`. Pinned so a future M6+
    /// extension that changes the action set (e.g., adding `Log`)
    /// surfaces here.
    #[test]
    fn builder_produces_action_set_with_observe_only() {
        let m = build_session_observe_manifest(ProjectId::new());
        assert_eq!(m.actions, vec![Action::Observe]);
    }

    /// CH-17 / ADR-0055 §D55.5 — the resource is the `session_object`
    /// composite (expands to `[DataObject, Tag]` per concept doc 01).
    /// Same shape the launch builder uses so the lead's session-object
    /// grant covers both reaches.
    #[test]
    fn builder_resource_is_session_object_composite() {
        let m = build_session_observe_manifest(ProjectId::new());
        assert_eq!(m.resource, vec!["session_object".to_string()]);
        assert!(m.transitive.is_empty());
    }

    /// CH-17: `constraints`, `constraint_requirements`, and `kinds`
    /// are empty — observability is a class-level reach (no per-tool
    /// constraint surface).
    #[test]
    fn builder_carries_no_constraints_or_kind_filters() {
        let m = build_session_observe_manifest(ProjectId::new());
        assert!(m.constraints.is_empty());
        assert!(m.constraint_requirements.is_empty());
        assert!(m.kinds.is_empty());
    }

    /// CH-17: the manifest is non-empty in the engine's `is_empty`
    /// sense — both `actions` AND `resource` carry at least one
    /// entry. Step 1 of the engine refuses an empty manifest.
    #[test]
    fn builder_produces_non_empty_manifest() {
        let m = build_session_observe_manifest(ProjectId::new());
        assert!(!m.is_empty());
    }
}
