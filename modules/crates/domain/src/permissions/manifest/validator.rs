//! Publish-time **tool authority manifest validator** — the gate every
//! [`ToolAuthorityManifest`] passes through before it can be persisted.
//!
//! Source of truth: `docs/specs/v0/concepts/permissions/04-manifest-and-resolution.md`
//! §"The Transitive-Grant Match Rule" (Rule A) +
//! `permissions/07-templates-and-tools.md` §"What v0 Validates vs Future
//! Enhancements" (the v0 rule set) +
//! `permissions/09-selector-grammar.md` §"Reserved Namespace Enforcement"
//! (Rule C) +
//! `permissions/03-action-vocabulary.md` §"Constraint × Fundamental
//! Applicability Matrix" (Rule D, lines 78–88).
//!
//! ## Rule set
//!
//! - **Rule A** — Composite/`#kind:` consistency. Every declared composite
//!   needs a matching `kinds` entry; every declared kind needs the
//!   composite's constituents in `resource ∪ transitive`. Blanket
//!   `kinds == ["*"]` accepted with [`ValidationWarning::BlanketKindWildcard`].
//! - **Rule B** — Action × Fundamental matrix. CH-04's
//!   [`Action::applies_to`] is the source of truth; the validator never
//!   re-encodes the matrix.
//! - **Rule C** — Reserved-namespace write rejection. `[Modify]` + bare
//!   `"tag"` resource is rejected (the `Tag` fundamental's namespaces are
//!   runtime-owned per `01-resource-ontology.md` §"Reserved tag
//!   namespaces"). Composites that internally include `Tag` (memory,
//!   session, …) remain legitimate — tools modify their composite data,
//!   not the composite's runtime-assigned identity tags.
//! - **Rule D** — Constraint × Fundamental matrix. Every declared
//!   constraint must apply to at least one expanded fundamental, OR be a
//!   universal constraint (`time_window`, `approval_requirement`,
//!   `non_delegability`, `purpose`).
//!
//! ## Errors and warnings
//!
//! [`validate_published_manifest`] returns `Err(ValidationError)` on the
//! first hard rejection (operator can fix and resubmit; subsequent rules
//! are not yet visible). Warnings accumulate alongside an `Ok(...)`
//! return so all are surfaced at once.

use std::collections::HashSet;

use thiserror::Error;

use super::super::action::Action;
use crate::model::composites::Composite;
use crate::model::fundamentals::Fundamental;
use crate::model::nodes::ToolAuthorityManifest;

// ----------------------------------------------------------------------------
// Reserved-namespace constant + generator
// ----------------------------------------------------------------------------

/// The three concept-doc-named reserved-namespace literals
/// (`01-resource-ontology.md` §"Reserved tag namespaces" lines 254–260).
///
/// `{kind}:*` for the 8 registered composites is generated dynamically
/// via [`reserved_namespace_prefixes`] so adding a new composite
/// auto-grows the list without an edit here.
pub const RESERVED_NAMESPACE_LITERALS: &[&str] = &["kind", "delegated_from", "derived_from"];

/// The full reserved-namespace prefix list at runtime: the three literals
/// (each with a `:` suffix and the `#kind` literal carrying the `#`
/// sigil) plus the 8 composite kind-prefixes (`session:`, `memory:`,
/// `auth_request:`, `external_service:`, `model_runtime:`,
/// `control_plane:`, `inbox:`, `outbox:`).
///
/// Used by CH-12's frozen-session-tag enforcement and by Rule C's
/// neighbouring use sites that need a complete denylist.
pub fn reserved_namespace_prefixes() -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(3 + Composite::ALL.len());
    out.push("#kind:".to_string());
    out.push("delegated_from:".to_string());
    out.push("derived_from:".to_string());
    for c in Composite::ALL {
        out.push(format!("{}:", c.kind_name()));
    }
    out
}

// ----------------------------------------------------------------------------
// Constraint × Fundamental applicability matrix
// ----------------------------------------------------------------------------

/// The four universal constraints from `03-action-vocabulary.md` lines
/// 92–97. Universal constraints apply to every fundamental.
pub const UNIVERSAL_CONSTRAINTS: &[&str] = &[
    "time_window",
    "approval_requirement",
    "non_delegability",
    "purpose",
];

/// Whether a named constraint applies to a given fundamental, per the
/// concept-doc Constraint × Fundamental matrix at
/// `permissions/03-action-vocabulary.md` lines 78–88.
///
/// Universal constraints (`time_window`, `approval_requirement`,
/// `non_delegability`, `purpose`) return `true` for every fundamental.
/// Unknown constraint names return `false` for every fundamental — the
/// validator surfaces them as [`ValidationError::ConstraintFundamentalMismatch`]
/// with an empty fundamentals set.
pub fn constraint_applies_to(name: &str, fundamental: Fundamental) -> bool {
    if UNIVERSAL_CONSTRAINTS.contains(&name) {
        return true;
    }
    use Fundamental::*;
    match (name, fundamental) {
        // path_prefix — filesystem only
        ("path_prefix", FilesystemObject) => true,
        ("path_prefix", _) => false,
        // command_pattern — process_exec only
        ("command_pattern", ProcessExecObject) => true,
        ("command_pattern", _) => false,
        // domain_allowlist — network only
        ("domain_allowlist", NetworkEndpoint) => true,
        ("domain_allowlist", _) => false,
        // tag_predicate — tag only
        ("tag_predicate", Tag) => true,
        ("tag_predicate", _) => false,
        // max_size_bytes — filesystem, network, data
        ("max_size_bytes", FilesystemObject)
        | ("max_size_bytes", NetworkEndpoint)
        | ("max_size_bytes", DataObject) => true,
        ("max_size_bytes", _) => false,
        // max_spend — economic, time/compute
        ("max_spend", EconomicResource) | ("max_spend", TimeComputeResource) => true,
        ("max_spend", _) => false,
        // sandbox_requirement — process_exec only
        ("sandbox_requirement", ProcessExecObject) => true,
        ("sandbox_requirement", _) => false,
        // output_channel — network only
        ("output_channel", NetworkEndpoint) => true,
        ("output_channel", _) => false,
        // timeout_secs — process_exec, network, time/compute
        ("timeout_secs", ProcessExecObject)
        | ("timeout_secs", NetworkEndpoint)
        | ("timeout_secs", TimeComputeResource) => true,
        ("timeout_secs", _) => false,
        // Unknown constraint name → not applicable to any fundamental.
        _ => false,
    }
}

/// The 9 specific (non-universal) constraint names the matrix knows
/// about. Used by exhaustive tests to enumerate without hand-listing.
pub const SPECIFIC_CONSTRAINT_NAMES: &[&str] = &[
    "path_prefix",
    "command_pattern",
    "domain_allowlist",
    "tag_predicate",
    "max_size_bytes",
    "max_spend",
    "sandbox_requirement",
    "output_channel",
    "timeout_secs",
];

// ----------------------------------------------------------------------------
// Resource classification
// ----------------------------------------------------------------------------

/// Classification of a resource-name string as it appears in
/// `manifest.resource` or `manifest.transitive`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceClass {
    /// Names a fundamental (e.g. `"filesystem_object"`).
    Fundamental(Fundamental),
    /// Names a composite (e.g. `"memory_object"`).
    Composite(Composite),
    /// String matches neither a fundamental nor a composite.
    Unknown(String),
}

/// Classify a resource-name string.
pub fn classify_resource_name(name: &str) -> ResourceClass {
    if let Some(f) = Fundamental::ALL
        .iter()
        .copied()
        .find(|f| f.as_str() == name)
    {
        return ResourceClass::Fundamental(f);
    }
    if let Some(c) = Composite::ALL.iter().copied().find(|c| c.as_str() == name) {
        return ResourceClass::Composite(c);
    }
    ResourceClass::Unknown(name.to_string())
}

// ----------------------------------------------------------------------------
// Validation outcomes
// ----------------------------------------------------------------------------

/// Hard rejection from [`validate_published_manifest`]. Each variant
/// carries the offending detail so the operator can fix and resubmit.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ValidationError {
    /// Rule A.a — manifest declares a composite (in `resource` or
    /// `transitive`) but does not list its kind in `kinds`.
    #[error(
        "manifest declares composite {} but kinds is missing the matching {:?}",
        composite.as_str(),
        composite.kind_name()
    )]
    MissingKindForComposite { composite: Composite },

    /// Rule A.b — manifest's `kinds` declares a composite kind but the
    /// composite's constituent fundamentals are not all present in
    /// `resource ∪ transitive`.
    #[error(
        "manifest declares kinds containing {} but its constituents {:?} are not a subset of declared fundamentals {:?}",
        kind.kind_name(),
        expected,
        declared
    )]
    KindFundamentalsInconsistent {
        kind: Composite,
        declared: HashSet<Fundamental>,
        expected: HashSet<Fundamental>,
    },

    /// A `resource` or `transitive` entry names neither a fundamental
    /// nor a composite.
    #[error(
        "manifest declares unknown resource class {:?} (must match a Fundamental or Composite name)",
        name
    )]
    UnknownResource { name: String },

    /// Rule B — declared (action, fundamental) pair is rejected by the
    /// CH-04 applicability matrix.
    #[error(
        "manifest declares action {:?} on fundamental {:?} but the Action × Fundamental matrix does not admit this pair",
        action,
        fundamental
    )]
    ActionFundamentalMismatch {
        action: Action,
        fundamental: Fundamental,
    },

    /// Rule D — declared constraint applies to none of the manifest's
    /// expanded fundamentals (and is not a universal constraint).
    #[error(
        "manifest declares constraint {:?} but it applies to none of the expanded fundamentals {:?}",
        constraint,
        fundamentals
    )]
    ConstraintFundamentalMismatch {
        constraint: String,
        fundamentals: HashSet<Fundamental>,
    },

    /// Rule C — manifest declares `[Modify]` on the bare `tag`
    /// fundamental. Tag namespaces are runtime-owned (`#kind:*`,
    /// `{kind}:*`, `delegated_from:*`, `derived_from:*`). Tools that
    /// create composite instances should declare the composite as
    /// resource (e.g. `memory_object`), not the bare `tag` fundamental.
    #[error(
        "manifest declares action {:?} on bare tag fundamental — tag namespaces are reserved for the runtime ({}); tools that create composite instances should declare the composite as the resource",
        action,
        crate::permissions::manifest::validator::RESERVED_NAMESPACE_LITERALS.join(", ")
    )]
    ReservedNamespaceWrite { namespace: String, action: Action },
}

/// Non-blocking advisory from [`validate_published_manifest`]. Warnings
/// accumulate alongside an `Ok(...)` return; they do not prevent
/// persistence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationWarning {
    /// `kinds == ["*"]` — accepted but operator should consider
    /// narrowing to a specific kind (concept doc 04 line 75).
    BlanketKindWildcard,

    /// Manifest's fundamentals match a composite shape but the manifest
    /// declares only the bare fundamentals. Suggest the composite label
    /// for readability (concept doc 04 line 110).
    MissingCompositeShorthand { suggested: Composite },

    /// Manifest declares `[Create]` on a composite but `target_kinds`
    /// does not list the composite's kind. The runtime can't auto-tag
    /// the new instance correctly without this hint (concept doc 07
    /// line 814).
    TargetKindsMissingForCreate { composite: Composite },
}

// ----------------------------------------------------------------------------
// Public entry point
// ----------------------------------------------------------------------------

/// Run all four rules over the supplied manifest. Returns
/// `Err(ValidationError)` on the first hard rejection; otherwise
/// returns `Ok(warnings)` (possibly empty).
///
/// Pure function — no I/O, no repository access.
pub fn validate_published_manifest(
    m: &ToolAuthorityManifest,
) -> Result<Vec<ValidationWarning>, ValidationError> {
    // Step 1 — classify every resource/transitive entry. Any unknown
    // entry is a hard rejection right away (Rule A precondition: the
    // validator can only reason about known classes).
    let mut declared_fundamentals: HashSet<Fundamental> = HashSet::new();
    let mut declared_composites: HashSet<Composite> = HashSet::new();
    for name in m.resource.iter().chain(m.transitive.iter()) {
        match classify_resource_name(name) {
            ResourceClass::Fundamental(f) => {
                declared_fundamentals.insert(f);
            }
            ResourceClass::Composite(c) => {
                declared_composites.insert(c);
                // Also union in the composite's constituents — the
                // expanded set is what Rules B and D run against.
                for f in c.constituents() {
                    declared_fundamentals.insert(*f);
                }
            }
            ResourceClass::Unknown(s) => {
                return Err(ValidationError::UnknownResource { name: s });
            }
        }
    }

    // Track warnings. Rules emit them via push; final return surfaces
    // all of them at once.
    let mut warnings: Vec<ValidationWarning> = Vec::new();

    // -----------------------------------------------------------------
    // Rule A — Composite/kinds consistency
    // -----------------------------------------------------------------
    // Special case: kinds == ["*"] is the blanket-kind escape hatch
    // (concept doc 04 line 75). Accepted with a warning and skips the
    // per-composite kind-name match.
    let blanket_kinds = m.kinds.len() == 1 && m.kinds[0] == "*";
    if blanket_kinds {
        warnings.push(ValidationWarning::BlanketKindWildcard);
    } else {
        // A.a — every declared composite needs a kinds entry.
        let kinds_set: HashSet<&str> = m.kinds.iter().map(|s| s.as_str()).collect();
        for c in &declared_composites {
            if !kinds_set.contains(c.kind_name()) {
                return Err(ValidationError::MissingKindForComposite { composite: *c });
            }
        }
        // A.b — every kinds entry naming a registered composite needs
        // its constituents present in declared_fundamentals.
        for kind_name in &m.kinds {
            if let Some(c) = Composite::ALL
                .iter()
                .copied()
                .find(|c| c.kind_name() == kind_name.as_str())
            {
                let expected: HashSet<Fundamental> = c.constituents().iter().copied().collect();
                if !expected.is_subset(&declared_fundamentals) {
                    return Err(ValidationError::KindFundamentalsInconsistent {
                        kind: c,
                        declared: declared_fundamentals.clone(),
                        expected,
                    });
                }
            }
            // Unknown kind names are silently ignored at this layer —
            // they may be org-defined kinds that the v0 ontology doesn't
            // yet enumerate. Future chunks may tighten this.
        }
    }

    // -----------------------------------------------------------------
    // Rule C — Reserved-namespace write rejection
    // -----------------------------------------------------------------
    // Trigger: actions ∋ Modify AND resource ∪ transitive contains the
    // bare "tag" string (i.e., the Tag fundamental as a top-level
    // resource — not as a constituent of a composite). Composites that
    // internally include Tag (memory, session, …) remain legitimate
    // because tools modify their composite data, not the composite's
    // runtime-assigned identity tags.
    if m.actions.contains(&Action::Modify) {
        let names_tag_directly = m
            .resource
            .iter()
            .chain(m.transitive.iter())
            .any(|s| s.as_str() == Fundamental::Tag.as_str());
        if names_tag_directly {
            return Err(ValidationError::ReservedNamespaceWrite {
                namespace: Fundamental::Tag.as_str().to_string(),
                action: Action::Modify,
            });
        }
    }

    // -----------------------------------------------------------------
    // Rule B — Action × Fundamental matrix
    // -----------------------------------------------------------------
    // For every (action, fundamental) pair derivable from
    // `actions × declared_fundamentals`, the CH-04 matrix must admit
    // the pair. Action::Wildcard always passes (Action::applies_to
    // returns true for it).
    for action in &m.actions {
        for f in &declared_fundamentals {
            if !action.applies_to(*f) {
                return Err(ValidationError::ActionFundamentalMismatch {
                    action: *action,
                    fundamental: *f,
                });
            }
        }
    }

    // -----------------------------------------------------------------
    // Rule D — Constraint × Fundamental matrix
    // -----------------------------------------------------------------
    // Every declared constraint must apply to at least one expanded
    // fundamental, OR be a universal constraint.
    for constraint in &m.constraints {
        if UNIVERSAL_CONSTRAINTS.contains(&constraint.as_str()) {
            continue;
        }
        let applies_to_any = declared_fundamentals
            .iter()
            .any(|f| constraint_applies_to(constraint, *f));
        if !applies_to_any {
            return Err(ValidationError::ConstraintFundamentalMismatch {
                constraint: constraint.clone(),
                fundamentals: declared_fundamentals.clone(),
            });
        }
    }

    // -----------------------------------------------------------------
    // Warnings — non-blocking advisories
    // -----------------------------------------------------------------
    // Composite shorthand suggestion: if declared_fundamentals exactly
    // matches a composite's constituents AND no composite was declared
    // explicitly, suggest the composite label (concept doc 04 line 110).
    if declared_composites.is_empty() && !declared_fundamentals.is_empty() {
        for c in Composite::ALL {
            let constituents: HashSet<Fundamental> = c.constituents().iter().copied().collect();
            if constituents == declared_fundamentals {
                warnings.push(ValidationWarning::MissingCompositeShorthand { suggested: c });
                break;
            }
        }
    }

    // target_kinds consistency warning: [Create] on a composite without
    // matching target_kinds entry (concept doc 07 line 814).
    if m.actions.contains(&Action::Create) {
        let target_kinds_set: HashSet<&str> = m.target_kinds.iter().map(|s| s.as_str()).collect();
        for c in &declared_composites {
            if !target_kinds_set.contains(c.kind_name()) {
                warnings.push(ValidationWarning::TargetKindsMissingForCreate { composite: *c });
            }
        }
    }

    Ok(warnings)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ids::NodeId;

    // ---- Fixtures --------------------------------------------------------

    /// Build a minimal manifest for the given (resource, actions, kinds)
    /// triple. Other fields default sensibly for the test.
    fn manifest(
        resource: Vec<&str>,
        actions: Vec<Action>,
        kinds: Vec<&str>,
    ) -> ToolAuthorityManifest {
        ToolAuthorityManifest {
            id: NodeId::new(),
            tool_name: "test_tool".to_string(),
            resource: resource.into_iter().map(String::from).collect(),
            transitive: vec![],
            actions,
            constraints: vec![],
            kinds: kinds.into_iter().map(String::from).collect(),
            target_kinds: vec![],
            delegable: false,
            approval: "auto".to_string(),
        }
    }

    // ---- Reserved-namespace constant + generator -------------------------

    #[test]
    fn reserved_namespace_literals_are_three() {
        assert_eq!(
            RESERVED_NAMESPACE_LITERALS,
            &["kind", "delegated_from", "derived_from"]
        );
    }

    #[test]
    fn reserved_namespace_prefixes_includes_three_literals() {
        let prefixes = reserved_namespace_prefixes();
        assert!(prefixes.contains(&"#kind:".to_string()));
        assert!(prefixes.contains(&"delegated_from:".to_string()));
        assert!(prefixes.contains(&"derived_from:".to_string()));
    }

    #[test]
    fn reserved_namespace_prefixes_includes_one_per_composite() {
        let prefixes = reserved_namespace_prefixes();
        for c in Composite::ALL {
            let expected = format!("{}:", c.kind_name());
            assert!(
                prefixes.contains(&expected),
                "missing reserved-namespace prefix for {:?}: expected {:?}",
                c,
                expected
            );
        }
    }

    #[test]
    fn reserved_namespace_prefixes_total_count_is_three_plus_composites() {
        let prefixes = reserved_namespace_prefixes();
        // 3 literals + 8 composites = 11. The composite count is asserted
        // by the existing composites::tests::all_contains_exactly_eight.
        assert_eq!(prefixes.len(), 3 + Composite::ALL.len());
    }

    // ---- Constraint matrix ----------------------------------------------

    #[test]
    fn universal_constraints_apply_to_every_fundamental() {
        for c in UNIVERSAL_CONSTRAINTS {
            for f in Fundamental::ALL {
                assert!(
                    constraint_applies_to(c, f),
                    "universal constraint {:?} must apply to {:?}",
                    c,
                    f
                );
            }
        }
    }

    #[test]
    fn constraint_matrix_exhaustive_against_concept_doc() {
        // Concept doc `permissions/03-action-vocabulary.md` lines 78–88,
        // transcribed verbatim. Each row is the 9 fundamentals' verdicts
        // for that constraint in concept-doc column order:
        //   (filesystem, process_exec, network, secret, data,
        //    tag, identity, economic, time/compute)
        let expected: &[(&str, [bool; 9])] = &[
            // path_prefix      | ✓ | — | — | — | — | — | — | — | — |
            (
                "path_prefix",
                [true, false, false, false, false, false, false, false, false],
            ),
            // command_pattern  | — | ✓ | — | — | — | — | — | — | — |
            (
                "command_pattern",
                [false, true, false, false, false, false, false, false, false],
            ),
            // domain_allowlist | — | — | ✓ | — | — | — | — | — | — |
            (
                "domain_allowlist",
                [false, false, true, false, false, false, false, false, false],
            ),
            // tag_predicate    | — | — | — | — | — | ✓ | — | — | — |
            (
                "tag_predicate",
                [false, false, false, false, false, true, false, false, false],
            ),
            // max_size_bytes   | ✓ | — | ✓ | — | ✓ | — | — | — | — |
            (
                "max_size_bytes",
                [true, false, true, false, true, false, false, false, false],
            ),
            // max_spend        | — | — | — | — | — | — | — | ✓ | ✓ |
            (
                "max_spend",
                [false, false, false, false, false, false, false, true, true],
            ),
            // sandbox_requirement | — | ✓ | — | — | — | — | — | — | — |
            (
                "sandbox_requirement",
                [false, true, false, false, false, false, false, false, false],
            ),
            // output_channel   | — | — | ✓ | — | — | — | — | — | — |
            (
                "output_channel",
                [false, false, true, false, false, false, false, false, false],
            ),
            // timeout_secs     | — | ✓ | ✓ | — | — | — | — | — | ✓ |
            (
                "timeout_secs",
                [false, true, true, false, false, false, false, false, true],
            ),
        ];
        // Concept doc row order maps to this fundamental order:
        // (filesystem, process_exec, network, secret, data, tag,
        //  identity, economic, time/compute). Note this is NOT the
        // Fundamental::ALL order (which puts economic + time/compute
        // before data + tag + identity); the row indexes below are
        // in concept-doc column order.
        let column_fundamentals = [
            Fundamental::FilesystemObject,
            Fundamental::ProcessExecObject,
            Fundamental::NetworkEndpoint,
            Fundamental::SecretCredential,
            Fundamental::DataObject,
            Fundamental::Tag,
            Fundamental::IdentityPrincipal,
            Fundamental::EconomicResource,
            Fundamental::TimeComputeResource,
        ];

        assert_eq!(expected.len(), SPECIFIC_CONSTRAINT_NAMES.len());
        let mut total_cells = 0usize;
        for (name, cells) in expected {
            for (idx, f) in column_fundamentals.iter().enumerate() {
                let want = cells[idx];
                let got = constraint_applies_to(name, *f);
                assert_eq!(
                    got,
                    want,
                    "(constraint={:?}, fundamental={:?}) — concept doc says {}, code says {}",
                    name,
                    f,
                    if want { "✓" } else { "—" },
                    if got { "✓" } else { "—" }
                );
                total_cells += 1;
            }
        }
        // 9 specific constraints × 9 fundamentals = 81 cells.
        assert_eq!(total_cells, 81);
    }

    #[test]
    fn unknown_constraint_returns_false_for_every_fundamental() {
        for f in Fundamental::ALL {
            assert!(!constraint_applies_to("not_a_real_constraint", f));
        }
    }

    // ---- Resource classification ----------------------------------------

    #[test]
    fn classify_resource_name_recognises_every_fundamental() {
        for f in Fundamental::ALL {
            match classify_resource_name(f.as_str()) {
                ResourceClass::Fundamental(got) => assert_eq!(got, f),
                other => panic!("expected Fundamental({:?}), got {:?}", f, other),
            }
        }
    }

    #[test]
    fn classify_resource_name_recognises_every_composite() {
        for c in Composite::ALL {
            match classify_resource_name(c.as_str()) {
                ResourceClass::Composite(got) => assert_eq!(got, c),
                other => panic!("expected Composite({:?}), got {:?}", c, other),
            }
        }
    }

    #[test]
    fn classify_resource_name_returns_unknown_for_garbage() {
        match classify_resource_name("not_a_thing") {
            ResourceClass::Unknown(s) => assert_eq!(s, "not_a_thing"),
            other => panic!("expected Unknown, got {:?}", other),
        }
    }

    // ---- Happy-path tests ------------------------------------------------

    #[test]
    fn happy_valid_composite_manifest() {
        // memory_object + [Read, List] + kinds=[memory] — fully
        // consistent.
        let m = manifest(
            vec!["memory_object"],
            vec![Action::Read, Action::List],
            vec!["memory"],
        );
        let warnings = validate_published_manifest(&m).expect("should pass");
        assert!(
            warnings.is_empty(),
            "no warnings expected, got {:?}",
            warnings
        );
    }

    #[test]
    fn happy_valid_fundamental_only_manifest() {
        // filesystem_object + [Read, Modify] + no kinds (no composites
        // declared) — Rule A doesn't fire; Rule B passes (Mutation
        // applies to filesystem); Rule C doesn't fire (no bare tag).
        let m = manifest(
            vec!["filesystem_object"],
            vec![Action::Read, Action::Modify],
            vec![],
        );
        let warnings = validate_published_manifest(&m).expect("should pass");
        assert!(warnings.is_empty());
    }

    #[test]
    fn happy_valid_wildcard_action_manifest() {
        // bootstrap-style manifest: [Wildcard] passes Rule B for every
        // fundamental.
        let m = manifest(vec!["filesystem_object"], vec![Action::Wildcard], vec![]);
        validate_published_manifest(&m).expect("wildcard action must pass");
    }

    #[test]
    fn happy_universal_constraint_with_unrelated_fundamental() {
        // purpose is universal, applies to every fundamental.
        let mut m = manifest(vec!["filesystem_object"], vec![Action::Read], vec![]);
        m.constraints = vec!["purpose".to_string()];
        validate_published_manifest(&m).expect("universal constraint must pass");
    }

    #[test]
    fn happy_specific_constraint_for_matching_fundamental() {
        // path_prefix on filesystem_object — applies.
        let mut m = manifest(vec!["filesystem_object"], vec![Action::Read], vec![]);
        m.constraints = vec!["path_prefix".to_string()];
        validate_published_manifest(&m).expect("constraint applies to filesystem");
    }

    #[test]
    fn happy_blanket_kinds_wildcard() {
        // memory_object + kinds=["*"] — accepted with warning.
        let m = manifest(vec!["memory_object"], vec![Action::Read], vec!["*"]);
        let warnings = validate_published_manifest(&m).expect("blanket kinds must pass");
        assert!(warnings.contains(&ValidationWarning::BlanketKindWildcard));
    }

    // ---- Hard-rejection tests (one per ValidationError variant) ---------

    #[test]
    fn rule_a_a_missing_kind_for_declared_composite() {
        // Declares memory_object but kinds is empty.
        let m = manifest(vec!["memory_object"], vec![Action::Read], vec![]);
        let err = validate_published_manifest(&m).expect_err("must reject");
        match err {
            ValidationError::MissingKindForComposite { composite } => {
                assert_eq!(composite, Composite::MemoryObject);
            }
            other => panic!("expected MissingKindForComposite, got {:?}", other),
        }
    }

    #[test]
    fn rule_a_b_kind_fundamentals_inconsistent() {
        // kinds names memory but resource declares only network_endpoint
        // (memory_object's constituents are data_object + tag, neither
        // of which is in the declared set).
        let m = manifest(vec!["network_endpoint"], vec![Action::Read], vec!["memory"]);
        let err = validate_published_manifest(&m).expect_err("must reject");
        match err {
            ValidationError::KindFundamentalsInconsistent { kind, expected, .. } => {
                assert_eq!(kind, Composite::MemoryObject);
                assert!(expected.contains(&Fundamental::DataObject));
                assert!(expected.contains(&Fundamental::Tag));
            }
            other => panic!("expected KindFundamentalsInconsistent, got {:?}", other),
        }
    }

    #[test]
    fn unknown_resource_rejected() {
        let m = manifest(vec!["nonsense_class"], vec![Action::Read], vec![]);
        let err = validate_published_manifest(&m).expect_err("must reject");
        match err {
            ValidationError::UnknownResource { name } => assert_eq!(name, "nonsense_class"),
            other => panic!("expected UnknownResource, got {:?}", other),
        }
    }

    #[test]
    fn rule_b_action_fundamental_mismatch() {
        // Recall on filesystem_object — Memory category only applies to
        // Tag.
        let m = manifest(vec!["filesystem_object"], vec![Action::Recall], vec![]);
        let err = validate_published_manifest(&m).expect_err("must reject");
        match err {
            ValidationError::ActionFundamentalMismatch {
                action,
                fundamental,
            } => {
                assert_eq!(action, Action::Recall);
                assert_eq!(fundamental, Fundamental::FilesystemObject);
            }
            other => panic!("expected ActionFundamentalMismatch, got {:?}", other),
        }
    }

    #[test]
    fn rule_d_constraint_fundamental_mismatch() {
        // path_prefix on network_endpoint — applies only to filesystem.
        let mut m = manifest(vec!["network_endpoint"], vec![Action::Read], vec![]);
        m.constraints = vec!["path_prefix".to_string()];
        let err = validate_published_manifest(&m).expect_err("must reject");
        match err {
            ValidationError::ConstraintFundamentalMismatch {
                constraint,
                fundamentals,
            } => {
                assert_eq!(constraint, "path_prefix");
                assert!(fundamentals.contains(&Fundamental::NetworkEndpoint));
            }
            other => panic!("expected ConstraintFundamentalMismatch, got {:?}", other),
        }
    }

    #[test]
    fn rule_c_reserved_namespace_write_rejected() {
        // [Modify] on bare "tag" resource — rejected.
        let m = manifest(vec!["tag"], vec![Action::Modify], vec![]);
        let err = validate_published_manifest(&m).expect_err("must reject");
        match err {
            ValidationError::ReservedNamespaceWrite { namespace, action } => {
                assert_eq!(namespace, "tag");
                assert_eq!(action, Action::Modify);
            }
            other => panic!("expected ReservedNamespaceWrite, got {:?}", other),
        }
    }

    #[test]
    fn rule_c_does_not_fire_for_composite_that_includes_tag() {
        // memory_object includes Tag in its constituents, but the
        // manifest names the composite, not the bare fundamental. The
        // tool modifies memory data, not tag values directly. ALLOWED.
        let m = manifest(vec!["memory_object"], vec![Action::Modify], vec!["memory"]);
        validate_published_manifest(&m).expect(
            "memory_object + Modify must pass; the tool modifies memory data, not the runtime-owned identity tags",
        );
    }

    #[test]
    fn rule_c_does_not_fire_for_read_actions_on_tag() {
        // [Read, List] on bare tag is fine — read access is allowed
        // per concept doc 09 line 194.
        let m = manifest(vec!["tag"], vec![Action::Read, Action::List], vec![]);
        validate_published_manifest(&m).expect("read on tag must pass");
    }

    // ---- Action × Fundamental round-trip ---------------------------------

    #[test]
    fn rule_b_rejects_every_negative_matrix_cell() {
        // For every (Action, Fundamental) cell where applies_to == false,
        // a manifest declaring that pair must be rejected.
        for action in Action::CANONICAL {
            for f in Fundamental::ALL {
                if action.applies_to(f) {
                    continue;
                }
                // Special handling: [Modify] on bare "tag" hits Rule C
                // first (which fires before Rule B). Skip those cells —
                // they're rejected, just under a different error variant.
                if action == Action::Modify && f == Fundamental::Tag {
                    continue;
                }
                let m = manifest(vec![f.as_str()], vec![action], vec![]);
                let err = validate_published_manifest(&m).expect_err(&format!(
                    "negative matrix cell {:?}/{:?} must be rejected",
                    action, f
                ));
                assert!(
                    matches!(err, ValidationError::ActionFundamentalMismatch { .. }),
                    "expected ActionFundamentalMismatch for {:?}/{:?}, got {:?}",
                    action,
                    f,
                    err
                );
            }
        }
    }

    // ---- Warning tests --------------------------------------------------

    #[test]
    fn warning_missing_composite_shorthand() {
        // Declare data_object + tag directly — matches memory_object,
        // session_object, auth_request_object, inbox_object, outbox_object
        // (all five have constituents = {DataObject, Tag}). The first
        // composite in Composite::ALL with this shape gets suggested.
        let m = manifest(vec!["data_object", "tag"], vec![Action::Read], vec![]);
        let warnings = validate_published_manifest(&m).expect("must pass");
        let suggested: Vec<_> = warnings
            .iter()
            .filter_map(|w| match w {
                ValidationWarning::MissingCompositeShorthand { suggested } => Some(*suggested),
                _ => None,
            })
            .collect();
        assert!(
            !suggested.is_empty(),
            "expected at least one composite-shorthand suggestion, got {:?}",
            warnings
        );
    }

    #[test]
    fn warning_target_kinds_missing_for_create() {
        // [Create] on memory_object without target_kinds=["memory"].
        let m = manifest(vec!["memory_object"], vec![Action::Create], vec!["memory"]);
        let warnings = validate_published_manifest(&m).expect("must pass");
        assert!(warnings.iter().any(|w| matches!(
            w,
            ValidationWarning::TargetKindsMissingForCreate {
                composite: Composite::MemoryObject
            }
        )));
    }

    #[test]
    fn warning_target_kinds_present_for_create_no_warning() {
        // [Create] on memory_object WITH target_kinds=["memory"]: no
        // warning.
        let mut m = manifest(vec!["memory_object"], vec![Action::Create], vec!["memory"]);
        m.target_kinds = vec!["memory".to_string()];
        let warnings = validate_published_manifest(&m).expect("must pass");
        assert!(!warnings
            .iter()
            .any(|w| matches!(w, ValidationWarning::TargetKindsMissingForCreate { .. })));
    }

    // ---- Composite × kind self-consistency ------------------------------

    #[test]
    fn every_composite_validates_with_matching_kind() {
        // For every composite, a minimal manifest naming the composite +
        // its kind_name() in kinds must pass Rule A.
        for c in Composite::ALL {
            let m = manifest(vec![c.as_str()], vec![Action::Read], vec![c.kind_name()]);
            validate_published_manifest(&m).unwrap_or_else(|e| {
                panic!("composite {:?} should validate cleanly, got {:?}", c, e)
            });
        }
    }
}
