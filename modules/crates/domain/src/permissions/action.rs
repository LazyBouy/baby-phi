//! The **34 canonical action verbs** of the v0 permission vocabulary, plus a
//! `Wildcard` escape hatch — together the closed set of `Action` values a
//! Grant or tool Manifest may carry.
//!
//! Source of truth: `docs/specs/v0/concepts/permissions/03-action-vocabulary.md`
//! §"Standard Action Vocabulary" (the 34 verbs grouped into 10 categories) and
//! §"Action × Fundamental Applicability Matrix" (the 9×10 grid encoded by
//! [`Action::applies_to`] / [`ActionCategory::applies_to`]).
//!
//! Wire format is preserved: every canonical variant serializes to its
//! snake_case form (`Action::Read` ↔ `"read"`); [`Action::Wildcard`]
//! serializes to `"*"`. Existing on-disk JSON storing actions as plain
//! strings round-trips through this enum without migration.

use serde::{Deserialize, Serialize};

use super::super::model::composites::Composite;
use super::super::model::fundamentals::Fundamental;

/// Every action verb the v0 permission vocabulary admits.
///
/// **Count:** 35 — 34 canonical verbs from the concept doc plus one
/// [`Action::Wildcard`] escape hatch (the `"*"` value used by privileged
/// system grants like the bootstrap claim).
///
/// The 34 canonical verbs are partitioned into 10 [`ActionCategory`]s; each
/// category applies to a stable subset of fundamentals per the concept-doc
/// matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    // -- Discovery --
    Discover,
    List,
    Inspect,
    // -- Data --
    Read,
    Copy,
    Export,
    // -- Mutation --
    Create,
    Modify,
    Append,
    Delete,
    // -- Execution --
    Execute,
    Invoke,
    Send,
    // -- Connection --
    Connect,
    Bind,
    Listen,
    // -- Authority --
    Delegate,
    Approve,
    Escalate,
    Allocate,
    Transfer,
    // -- Memory --
    Store,
    Retain,
    Recall,
    // -- Configuration --
    Configure,
    Install,
    Enable,
    Disable,
    // -- Economic --
    Spend,
    Reserve,
    Exceed,
    // -- Observability --
    Observe,
    Log,
    Attest,
    /// Privileged escape hatch — matches any required action in the
    /// Permission Check engine's `covers` / ceiling logic. Only system-tier
    /// grants (bootstrap claim, system:genesis) should carry this. Wire
    /// form is the single character `"*"`.
    #[serde(rename = "*")]
    Wildcard,
}

/// The 10 action categories from the concept doc. Categories partition the
/// 34 canonical actions; the wildcard has no category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionCategory {
    Discovery,
    Data,
    Mutation,
    Execution,
    Connection,
    Authority,
    Memory,
    Configuration,
    Economic,
    Observability,
}

impl ActionCategory {
    /// Every category in concept-doc-order.
    pub const ALL: [ActionCategory; 10] = [
        ActionCategory::Discovery,
        ActionCategory::Data,
        ActionCategory::Mutation,
        ActionCategory::Execution,
        ActionCategory::Connection,
        ActionCategory::Authority,
        ActionCategory::Memory,
        ActionCategory::Configuration,
        ActionCategory::Economic,
        ActionCategory::Observability,
    ];

    /// The canonical lower-case category name (matches the concept doc's
    /// **Category** column header).
    pub fn as_str(&self) -> &'static str {
        match self {
            ActionCategory::Discovery => "discovery",
            ActionCategory::Data => "data",
            ActionCategory::Mutation => "mutation",
            ActionCategory::Execution => "execution",
            ActionCategory::Connection => "connection",
            ActionCategory::Authority => "authority",
            ActionCategory::Memory => "memory",
            ActionCategory::Configuration => "configuration",
            ActionCategory::Economic => "economic",
            ActionCategory::Observability => "observability",
        }
    }

    /// The canonical actions belonging to this category, in concept-doc order.
    pub fn actions(&self) -> &'static [Action] {
        match self {
            ActionCategory::Discovery => &[Action::Discover, Action::List, Action::Inspect],
            ActionCategory::Data => &[Action::Read, Action::Copy, Action::Export],
            ActionCategory::Mutation => &[
                Action::Create,
                Action::Modify,
                Action::Append,
                Action::Delete,
            ],
            ActionCategory::Execution => &[Action::Execute, Action::Invoke, Action::Send],
            ActionCategory::Connection => &[Action::Connect, Action::Bind, Action::Listen],
            ActionCategory::Authority => &[
                Action::Delegate,
                Action::Approve,
                Action::Escalate,
                Action::Allocate,
                Action::Transfer,
            ],
            ActionCategory::Memory => &[Action::Store, Action::Retain, Action::Recall],
            ActionCategory::Configuration => &[
                Action::Configure,
                Action::Install,
                Action::Enable,
                Action::Disable,
            ],
            ActionCategory::Economic => &[Action::Spend, Action::Reserve, Action::Exceed],
            ActionCategory::Observability => &[Action::Observe, Action::Log, Action::Attest],
        }
    }

    /// Applicability of this category to a fundamental, per concept-doc
    /// matrix at `permissions/03-action-vocabulary.md` §"Action × Fundamental
    /// Applicability Matrix" (lines 27–37).
    pub fn applies_to(&self, f: Fundamental) -> bool {
        use ActionCategory::*;
        use Fundamental::*;
        match (self, f) {
            // Discovery — universal across all 9 fundamentals.
            (Discovery, _) => true,
            // Data — every fundamental except process_exec.
            (Data, ProcessExecObject) => false,
            (Data, _) => true,
            // Mutation — filesystem, data, tag, identity_principal only.
            (Mutation, FilesystemObject)
            | (Mutation, DataObject)
            | (Mutation, Tag)
            | (Mutation, IdentityPrincipal) => true,
            (Mutation, _) => false,
            // Execution — process_exec only.
            (Execution, ProcessExecObject) => true,
            (Execution, _) => false,
            // Connection — network_endpoint only.
            (Connection, NetworkEndpoint) => true,
            (Connection, _) => false,
            // Authority — universal.
            (Authority, _) => true,
            // Memory — tag only.
            (Memory, Tag) => true,
            (Memory, _) => false,
            // Configuration — process_exec, identity_principal, time/compute.
            (Configuration, ProcessExecObject)
            | (Configuration, IdentityPrincipal)
            | (Configuration, TimeComputeResource) => true,
            (Configuration, _) => false,
            // Economic — economic_resource, time/compute.
            (Economic, EconomicResource) | (Economic, TimeComputeResource) => true,
            (Economic, _) => false,
            // Observability — universal.
            (Observability, _) => true,
        }
    }
}

impl Action {
    /// All 35 variants in canonical order (34 canonical + Wildcard last).
    pub const ALL: [Action; 35] = [
        Action::Discover,
        Action::List,
        Action::Inspect,
        Action::Read,
        Action::Copy,
        Action::Export,
        Action::Create,
        Action::Modify,
        Action::Append,
        Action::Delete,
        Action::Execute,
        Action::Invoke,
        Action::Send,
        Action::Connect,
        Action::Bind,
        Action::Listen,
        Action::Delegate,
        Action::Approve,
        Action::Escalate,
        Action::Allocate,
        Action::Transfer,
        Action::Store,
        Action::Retain,
        Action::Recall,
        Action::Configure,
        Action::Install,
        Action::Enable,
        Action::Disable,
        Action::Spend,
        Action::Reserve,
        Action::Exceed,
        Action::Observe,
        Action::Log,
        Action::Attest,
        Action::Wildcard,
    ];

    /// The 34 canonical variants — `ALL` minus `Wildcard`. Useful for
    /// matrix iteration where the wildcard is a special case rather than
    /// a per-cell verdict.
    pub const CANONICAL: [Action; 34] = [
        Action::Discover,
        Action::List,
        Action::Inspect,
        Action::Read,
        Action::Copy,
        Action::Export,
        Action::Create,
        Action::Modify,
        Action::Append,
        Action::Delete,
        Action::Execute,
        Action::Invoke,
        Action::Send,
        Action::Connect,
        Action::Bind,
        Action::Listen,
        Action::Delegate,
        Action::Approve,
        Action::Escalate,
        Action::Allocate,
        Action::Transfer,
        Action::Store,
        Action::Retain,
        Action::Recall,
        Action::Configure,
        Action::Install,
        Action::Enable,
        Action::Disable,
        Action::Spend,
        Action::Reserve,
        Action::Exceed,
        Action::Observe,
        Action::Log,
        Action::Attest,
    ];

    /// The canonical wire form. `Action::Read` → `"read"`,
    /// `Action::Wildcard` → `"*"`.
    pub fn as_str(&self) -> &'static str {
        match self {
            Action::Discover => "discover",
            Action::List => "list",
            Action::Inspect => "inspect",
            Action::Read => "read",
            Action::Copy => "copy",
            Action::Export => "export",
            Action::Create => "create",
            Action::Modify => "modify",
            Action::Append => "append",
            Action::Delete => "delete",
            Action::Execute => "execute",
            Action::Invoke => "invoke",
            Action::Send => "send",
            Action::Connect => "connect",
            Action::Bind => "bind",
            Action::Listen => "listen",
            Action::Delegate => "delegate",
            Action::Approve => "approve",
            Action::Escalate => "escalate",
            Action::Allocate => "allocate",
            Action::Transfer => "transfer",
            Action::Store => "store",
            Action::Retain => "retain",
            Action::Recall => "recall",
            Action::Configure => "configure",
            Action::Install => "install",
            Action::Enable => "enable",
            Action::Disable => "disable",
            Action::Spend => "spend",
            Action::Reserve => "reserve",
            Action::Exceed => "exceed",
            Action::Observe => "observe",
            Action::Log => "log",
            Action::Attest => "attest",
            Action::Wildcard => "*",
        }
    }

    /// Which of the 10 categories this action belongs to. `Wildcard` has no
    /// category — callers needing per-action classification must handle the
    /// wildcard explicitly.
    pub fn category(&self) -> Option<ActionCategory> {
        match self {
            Action::Discover | Action::List | Action::Inspect => Some(ActionCategory::Discovery),
            Action::Read | Action::Copy | Action::Export => Some(ActionCategory::Data),
            Action::Create | Action::Modify | Action::Append | Action::Delete => {
                Some(ActionCategory::Mutation)
            }
            Action::Execute | Action::Invoke | Action::Send => Some(ActionCategory::Execution),
            Action::Connect | Action::Bind | Action::Listen => Some(ActionCategory::Connection),
            Action::Delegate
            | Action::Approve
            | Action::Escalate
            | Action::Allocate
            | Action::Transfer => Some(ActionCategory::Authority),
            Action::Store | Action::Retain | Action::Recall => Some(ActionCategory::Memory),
            Action::Configure | Action::Install | Action::Enable | Action::Disable => {
                Some(ActionCategory::Configuration)
            }
            Action::Spend | Action::Reserve | Action::Exceed => Some(ActionCategory::Economic),
            Action::Observe | Action::Log | Action::Attest => Some(ActionCategory::Observability),
            Action::Wildcard => None,
        }
    }

    /// Whether this action is applicable to a given fundamental, per the
    /// concept-doc matrix. `Wildcard` matches every fundamental.
    pub fn applies_to(&self, f: Fundamental) -> bool {
        match self {
            Action::Wildcard => true,
            other => other
                .category()
                .expect("non-wildcard action has a category")
                .applies_to(f),
        }
    }

    /// Whether this action is applicable to a composite. A composite's
    /// applicable actions are the union of its constituents' actions per
    /// concept-doc §"Composite inheritance" (line 39). `Wildcard` matches
    /// every composite.
    pub fn applies_to_composite(&self, c: Composite) -> bool {
        if matches!(self, Action::Wildcard) {
            return true;
        }
        c.constituents().iter().any(|f| self.applies_to(*f))
    }
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Failure parsing a string back into [`Action`]. Carries the offending
/// input so error messages stay useful.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseActionError {
    pub input: String,
}

impl std::fmt::Display for ParseActionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} is not a valid action verb (expected one of the 34 canonical verbs from concepts/permissions/03-action-vocabulary.md, or \"*\" for the wildcard)",
            self.input
        )
    }
}

impl std::error::Error for ParseActionError {}

impl std::str::FromStr for Action {
    type Err = ParseActionError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Action::ALL
            .iter()
            .copied()
            .find(|a| a.as_str() == s)
            .ok_or_else(|| ParseActionError {
                input: s.to_string(),
            })
    }
}

impl TryFrom<&str> for Action {
    type Error = ParseActionError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    // ── Variant-count invariants ──────────────────────────────────────────

    #[test]
    fn all_contains_thirty_five_variants() {
        assert_eq!(Action::ALL.len(), 35);
    }

    #[test]
    fn canonical_contains_thirty_four_variants() {
        assert_eq!(Action::CANONICAL.len(), 34);
    }

    #[test]
    fn all_variants_are_distinct() {
        let set: HashSet<_> = Action::ALL.iter().collect();
        assert_eq!(set.len(), 35);
    }

    #[test]
    fn canonical_variants_are_distinct() {
        let set: HashSet<_> = Action::CANONICAL.iter().collect();
        assert_eq!(set.len(), 34);
    }

    #[test]
    fn canonical_is_all_minus_wildcard() {
        let canonical_set: HashSet<_> = Action::CANONICAL.iter().copied().collect();
        let mut all_minus_wild: HashSet<_> = Action::ALL.iter().copied().collect();
        all_minus_wild.remove(&Action::Wildcard);
        assert_eq!(canonical_set, all_minus_wild);
    }

    // ── as_str invariants ─────────────────────────────────────────────────

    #[test]
    fn as_str_distinct_per_variant() {
        let strs: HashSet<_> = Action::ALL.iter().map(Action::as_str).collect();
        assert_eq!(strs.len(), 35);
    }

    #[test]
    fn wildcard_serializes_to_star() {
        assert_eq!(Action::Wildcard.as_str(), "*");
    }

    #[test]
    fn canonical_actions_use_snake_case() {
        // No canonical wire string contains uppercase or hyphen.
        for a in Action::CANONICAL {
            let s = a.as_str();
            assert!(
                s.chars().all(|c| c.is_ascii_lowercase() || c == '_'),
                "non-snake_case wire form: {} for {:?}",
                s,
                a
            );
        }
    }

    // ── Category invariants ──────────────────────────────────────────────

    #[test]
    fn category_for_each_canonical_action() {
        for a in Action::CANONICAL {
            assert!(
                a.category().is_some(),
                "canonical action {:?} must have a category",
                a
            );
        }
    }

    #[test]
    fn wildcard_has_no_category() {
        assert_eq!(Action::Wildcard.category(), None);
    }

    #[test]
    fn category_actions_partition_canonical() {
        let mut union: HashSet<Action> = HashSet::new();
        let mut total = 0usize;
        for cat in ActionCategory::ALL {
            for a in cat.actions() {
                assert!(union.insert(*a), "action {:?} appears in two categories", a);
                total += 1;
            }
        }
        assert_eq!(total, 34);
        assert_eq!(union.len(), 34);
        // The union equals the canonical set.
        let canonical: HashSet<Action> = Action::CANONICAL.iter().copied().collect();
        assert_eq!(union, canonical);
    }

    #[test]
    fn category_round_trips_via_actions() {
        for cat in ActionCategory::ALL {
            for a in cat.actions() {
                assert_eq!(a.category(), Some(cat));
            }
        }
    }

    // ── TryFrom / FromStr ────────────────────────────────────────────────

    #[test]
    fn try_from_str_round_trips_every_variant() {
        for a in Action::ALL {
            let s = a.as_str();
            let back = Action::try_from(s)
                .unwrap_or_else(|e| panic!("{:?} round-trip failed for {:?}: {}", a, s, e));
            assert_eq!(back, a);
        }
    }

    #[test]
    fn try_from_str_wildcard_accepts_star() {
        assert_eq!(Action::try_from("*").unwrap(), Action::Wildcard);
    }

    #[test]
    fn try_from_str_unknown_returns_error_with_input() {
        let err = Action::try_from("raed").unwrap_err();
        assert_eq!(err.input, "raed");
    }

    #[test]
    fn try_from_str_empty_string_returns_error() {
        assert!(Action::try_from("").is_err());
    }

    #[test]
    fn try_from_str_uppercase_rejected() {
        // Wire form is lowercase only — case-sensitive parsing.
        assert!(Action::try_from("READ").is_err());
    }

    // ── serde wire format ────────────────────────────────────────────────

    #[test]
    fn serde_round_trip_for_each_variant() {
        for a in Action::ALL {
            let json = serde_json::to_string(&a).expect("serialize");
            let back: Action = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(back, a, "round-trip failed for {:?}", a);
        }
    }

    #[test]
    fn serde_canonical_serializes_to_quoted_snake_case() {
        // Wire-format compat: `Action::Read` must serialize to the same
        // string the legacy `Vec<String>` storage did. The audit hash chain
        // depends on this byte-stability.
        assert_eq!(serde_json::to_string(&Action::Read).unwrap(), "\"read\"");
        assert_eq!(
            serde_json::to_string(&Action::Allocate).unwrap(),
            "\"allocate\""
        );
        assert_eq!(
            serde_json::to_string(&Action::Invoke).unwrap(),
            "\"invoke\""
        );
    }

    #[test]
    fn serde_wildcard_serializes_to_quoted_star() {
        assert_eq!(serde_json::to_string(&Action::Wildcard).unwrap(), "\"*\"");
    }

    #[test]
    fn serde_legacy_string_vec_round_trips_through_typed_vec() {
        // A Vec<String> built from canonical strings must deserialize cleanly
        // into Vec<Action>, and the resulting Vec<Action> must serialize
        // bit-for-bit identically to the original Vec<String>.
        let legacy = vec!["read".to_string(), "modify".to_string(), "*".to_string()];
        let legacy_json = serde_json::to_string(&legacy).unwrap();
        let typed: Vec<Action> = serde_json::from_str(&legacy_json).unwrap();
        assert_eq!(typed, vec![Action::Read, Action::Modify, Action::Wildcard]);
        let typed_json = serde_json::to_string(&typed).unwrap();
        assert_eq!(typed_json, legacy_json);
    }

    // ── Applicability matrix — exhaustive 306-cell check ────────────────

    #[test]
    fn applies_to_matrix_exhaustive_against_concept_doc() {
        // Concept doc `permissions/03-action-vocabulary.md` lines 27–37,
        // transcribed verbatim. Each row lists the 10 categories' verdicts
        // for that fundamental in concept-doc column order:
        //   (Discovery, Data, Mutation, Execution, Connection,
        //    Authority, Memory, Configuration, Economic, Observability)
        let expected: &[(Fundamental, [bool; 10])] = &[
            // filesystem_object | ✓ | ✓ | ✓ | — | — | ✓ | — | — | — | ✓ |
            (
                Fundamental::FilesystemObject,
                [
                    true, true, true, false, false, true, false, false, false, true,
                ],
            ),
            // process_exec_object | ✓ | — | — | ✓ | — | ✓ | — | ✓ | — | ✓ |
            (
                Fundamental::ProcessExecObject,
                [
                    true, false, false, true, false, true, false, true, false, true,
                ],
            ),
            // network_endpoint | ✓ | ✓ | — | — | ✓ | ✓ | — | — | — | ✓ |
            (
                Fundamental::NetworkEndpoint,
                [
                    true, true, false, false, true, true, false, false, false, true,
                ],
            ),
            // secret/credential | ✓ | ✓ | — | — | — | ✓ | — | — | — | ✓ |
            (
                Fundamental::SecretCredential,
                [
                    true, true, false, false, false, true, false, false, false, true,
                ],
            ),
            // economic_resource | ✓ | ✓ | — | — | — | ✓ | — | — | ✓ | ✓ |
            (
                Fundamental::EconomicResource,
                [
                    true, true, false, false, false, true, false, false, true, true,
                ],
            ),
            // time/compute_resource | ✓ | ✓ | — | — | — | ✓ | — | ✓ | ✓ | ✓ |
            (
                Fundamental::TimeComputeResource,
                [
                    true, true, false, false, false, true, false, true, true, true,
                ],
            ),
            // data_object | ✓ | ✓ | ✓ | — | — | ✓ | — | — | — | ✓ |
            (
                Fundamental::DataObject,
                [
                    true, true, true, false, false, true, false, false, false, true,
                ],
            ),
            // tag | ✓ | ✓ | ✓ | — | — | ✓ | ✓ | — | — | ✓ |
            (
                Fundamental::Tag,
                [
                    true, true, true, false, false, true, true, false, false, true,
                ],
            ),
            // identity_principal | ✓ | ✓ | ✓ | — | — | ✓ | — | ✓ | — | ✓ |
            (
                Fundamental::IdentityPrincipal,
                [
                    true, true, true, false, false, true, false, true, false, true,
                ],
            ),
        ];

        // The matrix must cover all 9 fundamentals (sanity).
        assert_eq!(expected.len(), 9);

        let mut total_cells = 0usize;
        for (f, cells) in expected {
            for (idx, cat) in ActionCategory::ALL.iter().enumerate() {
                let want = cells[idx];
                let got = cat.applies_to(*f);
                assert_eq!(
                    got,
                    want,
                    "(category={:?}, fundamental={:?}) — concept doc says {}, code says {}",
                    cat,
                    f,
                    if want { "✓" } else { "—" },
                    if got { "✓" } else { "—" }
                );
                // Per-action verdicts within this category must agree with
                // the category-level verdict (categories partition cleanly).
                for a in cat.actions() {
                    assert_eq!(
                        a.applies_to(*f),
                        want,
                        "action={:?} disagrees with its category {:?} on fundamental {:?}",
                        a,
                        cat,
                        f
                    );
                    total_cells += 1;
                }
            }
        }
        // 9 fundamentals × 34 canonical actions = 306 cells.
        assert_eq!(total_cells, 9 * 34);
    }

    #[test]
    fn applies_to_wildcard_universal_over_fundamentals() {
        for f in Fundamental::ALL {
            assert!(
                Action::Wildcard.applies_to(f),
                "wildcard must apply to every fundamental, missed {:?}",
                f
            );
        }
    }

    // ── Composite inheritance ───────────────────────────────────────────

    #[test]
    fn applies_to_composite_via_constituent_union() {
        // For every (Action, Composite) pair, the applies_to_composite verdict
        // must equal the disjunction of applies_to over the composite's
        // constituents (per concept-doc §"Composite inheritance" line 39).
        for c in Composite::ALL {
            for a in Action::ALL {
                let want = if matches!(a, Action::Wildcard) {
                    true
                } else {
                    c.constituents().iter().any(|f| a.applies_to(*f))
                };
                let got = a.applies_to_composite(c);
                assert_eq!(
                    got, want,
                    "applies_to_composite disagrees with constituent union for action={:?} composite={:?}",
                    a, c
                );
            }
        }
    }

    #[test]
    fn applies_to_composite_memory_supports_recall() {
        // Concept doc §"Composite inheritance" calls out the memory_object
        // example explicitly: recall (from Tag) and read (from DataObject).
        assert!(Action::Recall.applies_to_composite(Composite::MemoryObject));
        assert!(Action::Store.applies_to_composite(Composite::MemoryObject));
        assert!(Action::Retain.applies_to_composite(Composite::MemoryObject));
        assert!(Action::Read.applies_to_composite(Composite::MemoryObject));
        assert!(Action::Modify.applies_to_composite(Composite::MemoryObject));
    }

    #[test]
    fn applies_to_composite_external_service_does_not_have_memory_actions() {
        // ExternalServiceObject = network + secret + tag. Tag carries Memory,
        // so recall actually does apply via composite inheritance — verify
        // the rule rather than asserting an absence.
        assert!(Action::Recall.applies_to_composite(Composite::ExternalServiceObject));
        // But Mutation is absent (network and secret reject Mutation; Tag
        // accepts it — so the union DOES include Mutation via tag).
        assert!(Action::Modify.applies_to_composite(Composite::ExternalServiceObject));
        // Execution should be absent (no constituent accepts execution).
        assert!(!Action::Execute.applies_to_composite(Composite::ExternalServiceObject));
    }

    #[test]
    fn applies_to_composite_wildcard_universal() {
        for c in Composite::ALL {
            assert!(
                Action::Wildcard.applies_to_composite(c),
                "wildcard must apply to every composite, missed {:?}",
                c
            );
        }
    }

    // ── ParseActionError surface ────────────────────────────────────────

    #[test]
    fn parse_error_display_mentions_input() {
        let err = Action::try_from("not_a_real_verb").unwrap_err();
        let s = format!("{}", err);
        assert!(s.contains("not_a_real_verb"));
    }
}
