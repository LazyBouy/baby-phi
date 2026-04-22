//! The three-valued [`Decision`] returned by the Permission Check engine.
//!
//! The shape mirrors the concept doc's pseudocode in
//! `permissions/04-manifest-and-resolution.md` §Formal Algorithm:
//!
//! ```text
//!   Decision = Allowed { resolved_grants }
//!            | Denied  { failed_step, reason }
//!            | Pending { awaiting_consent }
//! ```
//!
//! `failed_step` is surfaced both in the payload and in the Prometheus
//! histogram label — see `metrics.rs`.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::model::ids::{AgentId, GrantId, OrgId};
use crate::model::Fundamental;

/// Enumerates the eight explicit stages of the pipeline (0, 1, 2, 2a, 3, 4,
/// 5, 6). The integer representation is what the Prometheus histogram label
/// `failed_step` reports; 2a is encoded as `22` so labels stay numeric and
/// sortable without clashing with step 2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailedStep {
    /// Resource missing from owning org's catalogue.
    Catalogue = 0,
    /// Manifest expansion produced no reaches at all (malformed manifest).
    Expansion = 1,
    /// Candidate grant set was empty (subject holds no grants anywhere).
    Resolution = 2,
    /// Ceiling clamped every candidate to empty.
    Ceiling = 22,
    /// Some required reach had no matching grant.
    Match = 3,
    /// Winning grant violates a manifest-declared constraint.
    Constraint = 4,
    /// Scope-resolution cascade failed to pick a winner (no reader claim).
    Scope = 5,
    /// Template A–D grant lacks subordinate consent (surfaces as `Pending`).
    Consent = 6,
}

impl FailedStep {
    /// The metric label form used in `phi_permission_check_duration_seconds`.
    pub fn as_metric_label(self) -> &'static str {
        match self {
            FailedStep::Catalogue => "0",
            FailedStep::Expansion => "1",
            FailedStep::Resolution => "2",
            FailedStep::Ceiling => "2a",
            FailedStep::Match => "3",
            FailedStep::Constraint => "4",
            FailedStep::Scope => "5",
            FailedStep::Consent => "6",
        }
    }
}

/// Human-readable reason attached to a `Denied` outcome. The variants encode
/// the algorithm's explicit failure modes; `detail` fields capture the
/// specific triggering value so operators and audit logs can diagnose a
/// denial without re-running the check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DeniedReason {
    /// A reach touched a resource the owning org's catalogue does not list.
    CatalogueMiss { resource_uri: String },
    /// Manifest declared no resources/actions — nothing to match against.
    ManifestEmpty,
    /// Agent holds no grants at any scope.
    NoGrantsHeld,
    /// Every candidate grant was clamped to empty by the ceiling.
    CeilingEmptied,
    /// No grant in the candidate set matches this `(fundamental, action)`.
    NoMatchingGrant {
        fundamental: Fundamental,
        action: String,
    },
    /// A manifest-declared constraint was not satisfied by the winning grant.
    ConstraintViolation {
        constraint: String,
        grant_id: GrantId,
    },
    /// The scope-resolution cascade couldn't pick a winner (outsider case
    /// where no candidate shares scope with the reader).
    ScopeUnresolvable {
        fundamental: Fundamental,
        action: String,
    },
}

/// The three outcomes the Permission Check engine can return.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum Decision {
    /// All reaches matched; every constraint satisfied; consent present
    /// where required.
    Allowed {
        /// Per-reach winning grant. The key is the `(fundamental, action)`
        /// pair; the value is the grant id that covered it.
        resolved_grants: Vec<ResolvedReach>,
    },
    /// Denied at a specific step; `reason` names the triggering detail.
    Denied {
        failed_step: FailedStep,
        reason: DeniedReason,
    },
    /// The winning grant requires subordinate consent (Templates A–D) and
    /// consent has not been recorded. Callers can wait / request / retry.
    Pending { awaiting_consent: AwaitingConsent },
}

/// One `(fundamental, action)` reach and the grant that covered it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedReach {
    pub fundamental: Fundamental,
    pub action: String,
    pub grant_id: GrantId,
}

/// The principal whose consent is missing, plus the org under whose policy
/// the consent must be recorded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AwaitingConsent {
    pub subordinate: AgentId,
    pub org: OrgId,
}

impl Decision {
    /// Convenience: `true` for [`Decision::Allowed`].
    pub fn is_allowed(&self) -> bool {
        matches!(self, Decision::Allowed { .. })
    }

    /// Convenience: the failed-step label, if this is a denial; `None`
    /// otherwise.
    pub fn failed_step(&self) -> Option<FailedStep> {
        match self {
            Decision::Denied { failed_step, .. } => Some(*failed_step),
            _ => None,
        }
    }

    /// The value used for the Prometheus `result` label:
    /// `allowed` / `denied` / `pending`.
    pub fn metric_result_label(&self) -> &'static str {
        match self {
            Decision::Allowed { .. } => "allowed",
            Decision::Denied { .. } => "denied",
            Decision::Pending { .. } => "pending",
        }
    }

    /// For tests and audit summaries: a lookup from `(fundamental, action)`
    /// to grant id. Empty for non-Allowed outcomes.
    pub fn resolved_grants_map(&self) -> HashMap<(Fundamental, String), GrantId> {
        match self {
            Decision::Allowed { resolved_grants } => resolved_grants
                .iter()
                .map(|r| ((r.fundamental, r.action.clone()), r.grant_id))
                .collect(),
            _ => HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_step_metric_labels_are_distinct() {
        let labels: std::collections::HashSet<_> = [
            FailedStep::Catalogue,
            FailedStep::Expansion,
            FailedStep::Resolution,
            FailedStep::Ceiling,
            FailedStep::Match,
            FailedStep::Constraint,
            FailedStep::Scope,
            FailedStep::Consent,
        ]
        .iter()
        .map(|s| s.as_metric_label())
        .collect();
        assert_eq!(labels.len(), 8);
    }

    #[test]
    fn allowed_decision_metric_label_is_allowed() {
        let d = Decision::Allowed {
            resolved_grants: vec![],
        };
        assert_eq!(d.metric_result_label(), "allowed");
        assert!(d.is_allowed());
        assert_eq!(d.failed_step(), None);
    }

    #[test]
    fn denied_decision_surfaces_failed_step() {
        let d = Decision::Denied {
            failed_step: FailedStep::Catalogue,
            reason: DeniedReason::CatalogueMiss {
                resource_uri: "system:ghost".into(),
            },
        };
        assert_eq!(d.metric_result_label(), "denied");
        assert_eq!(d.failed_step(), Some(FailedStep::Catalogue));
        assert!(!d.is_allowed());
    }
}
