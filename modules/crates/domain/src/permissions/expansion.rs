//! Composite → fundamental expansion + `resolve_grant`.
//!
//! Two loading functions live here:
//!
//! 1. [`expand_resource_to_fundamentals`] — turns a string like
//!    `external_service_object` or `filesystem_object` into its constituent
//!    [`Fundamental`]s. Unknown names expand to the empty set so the engine
//!    surfaces a clean Step-1 failure rather than panicking.
//! 2. [`resolve_grant`] — lifts a persisted [`Grant`] into a
//!    [`ResolvedGrant`] that carries its expanded fundamentals, a parsed
//!    [`Selector`], and (when the grant targets a composite) the implicit
//!    `#kind:` refinement added per
//!    `concepts/permissions/04-manifest-and-resolution.md` §Refinement.

use std::collections::HashSet;

use crate::model::nodes::Grant;
use crate::model::{Composite, Fundamental};

use super::selector::Selector;

/// Expand a resource-class string (either a fundamental name or a composite
/// name) into the set of fundamentals it contributes to a Permission Check.
///
/// Unknown strings return `Ok(HashSet::new())` — the caller (Step 1) treats
/// a manifest that expands to nothing as `ManifestEmpty`.
pub fn expand_resource_to_fundamentals(name: &str) -> HashSet<Fundamental> {
    if let Some(f) = parse_fundamental(name) {
        return HashSet::from([f]);
    }
    if let Some(c) = parse_composite(name) {
        return c.constituents().iter().copied().collect();
    }
    HashSet::new()
}

/// Map a fundamental's canonical string form back to the enum.
fn parse_fundamental(name: &str) -> Option<Fundamental> {
    Fundamental::ALL
        .iter()
        .copied()
        .find(|f| f.as_str() == name)
}

/// Map a composite's canonical string form (e.g. `memory_object`) back to
/// the enum.
fn parse_composite(name: &str) -> Option<Composite> {
    Composite::ALL.iter().copied().find(|c| c.as_str() == name)
}

/// Engine-internal view of a grant: its fundamentals, its effective
/// selector (explicit AND any implicit `#kind:` refinement), and the original
/// grant reference for audit purposes.
#[derive(Debug, Clone)]
pub struct ResolvedGrant {
    /// The set of fundamentals this grant covers. Always non-empty unless
    /// the grant's resource URI is malformed — such grants are filtered out
    /// of the candidate set before Step 3.
    pub fundamentals: HashSet<Fundamental>,
    /// The grant's parsed selector. Used by Step 3 to match against the
    /// call target.
    pub selector: Selector,
    /// If the grant targeted a composite, the implicit `#kind:{name}`
    /// refinement the engine appends to the effective selector.
    pub kind_refinement: Option<Selector>,
    /// Pointer back into the original grant for Step 4 (constraint lookup),
    /// Step 5 (scope cascade), and Step 6 (consent gating).
    pub grant: Grant,
}

impl ResolvedGrant {
    /// Does the effective selector (explicit AND `#kind:` refinement) match
    /// the call target?
    pub fn effective_matches(&self, target_uri: &str, target_tags: &[String]) -> bool {
        let base = self.selector.matches(target_uri, target_tags);
        match &self.kind_refinement {
            Some(r) => base && r.matches(target_uri, target_tags),
            None => base,
        }
    }

    /// Does this grant cover the given `(fundamental, action)` reach?
    ///
    /// - Fundamental must be in this grant's `fundamentals` set.
    /// - Action must be in `grant.action` OR the grant must list `"*"` (a
    ///   bootstrap/root grant).
    pub fn covers(&self, fundamental: Fundamental, action: &str) -> bool {
        if !self.fundamentals.contains(&fundamental) {
            return false;
        }
        self.grant.action.iter().any(|a| a == action || a == "*")
    }
}

/// Lift a persisted [`Grant`] into a [`ResolvedGrant`].
///
/// Extracts the fundamentals from the grant's `resource.uri` (treated as a
/// class name when it matches a fundamental/composite) and produces a
/// `#kind:{name}` refinement for composite grants. For URIs that are not
/// class names (e.g. `system:root`, `filesystem:/workspace/**`) the
/// fundamentals set is derived from the explicit `class` prefix where
/// possible; otherwise the caller should use [`ResolvedGrant::with_fundamentals`]
/// to inject them.
///
/// This keeps the three flavours of grant that M1 actually creates workable:
///
/// | Grant URI | Fundamentals | Selector | Refinement |
/// |---|---|---|---|
/// | `system:root` | (caller injects via `with_fundamentals`) | `Exact("system:root")` | None |
/// | `filesystem_object` | `{filesystem_object}` | `Any` | None |
/// | `memory_object` | `{data_object, tag}` | `Any` | `#kind:memory` |
/// | `filesystem:/workspace/**` | (injected) | `Prefix("filesystem:/workspace/")` | None |
///
/// When the caller knows better than the URI string, [`ResolvedGrant::
/// with_fundamentals`] lets them override.
pub fn resolve_grant(grant: &Grant) -> ResolvedGrant {
    let uri = &grant.resource.uri;
    // Special case: `system:root` is the axiomatic ceiling for the entire
    // resource tree (seeded by the System Bootstrap Template). A grant on
    // `system:root` implicitly covers every fundamental and matches every
    // target URI, so long as the grant's listed action is in scope.
    if uri == "system:root" {
        return ResolvedGrant {
            fundamentals: Fundamental::ALL.iter().copied().collect(),
            selector: Selector::Any,
            kind_refinement: None,
            grant: grant.clone(),
        };
    }
    // Case A: the URI names a fundamental.
    if let Some(f) = parse_fundamental(uri) {
        return ResolvedGrant {
            fundamentals: HashSet::from([f]),
            selector: Selector::Any,
            kind_refinement: None,
            grant: grant.clone(),
        };
    }
    // Case B: the URI names a composite — expand + add #kind: refinement.
    if let Some(c) = parse_composite(uri) {
        return ResolvedGrant {
            fundamentals: c.constituents().iter().copied().collect(),
            selector: Selector::Any,
            kind_refinement: Some(Selector::KindTag(c.kind_name().to_string())),
            grant: grant.clone(),
        };
    }
    // Case C: the URI is an opaque instance URI — parse as a selector; the
    // caller supplies the fundamentals via with_fundamentals when they need
    // the grant to cover specific classes (this is how the bootstrap
    // `system:root` grant becomes an [allocate]-on-identity-principal grant).
    ResolvedGrant {
        fundamentals: HashSet::new(),
        selector: Selector::parse(uri),
        kind_refinement: None,
        grant: grant.clone(),
    }
}

impl ResolvedGrant {
    /// Override the fundamentals this grant covers. Used when the grant URI
    /// is an instance reference (e.g. `system:root`) and the caller
    /// supplies the class association from elsewhere (e.g. the bootstrap
    /// grant covers `IdentityPrincipal` with action `allocate`).
    pub fn with_fundamentals<I: IntoIterator<Item = Fundamental>>(mut self, fs: I) -> Self {
        self.fundamentals = fs.into_iter().collect();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ids::{AgentId, GrantId};
    use crate::model::nodes::{PrincipalRef, ResourceRef};
    use chrono::Utc;

    fn sample_grant(resource_uri: &str, actions: &[&str]) -> Grant {
        Grant {
            id: GrantId::new(),
            holder: PrincipalRef::Agent(AgentId::new()),
            action: actions.iter().map(|s| s.to_string()).collect(),
            resource: ResourceRef {
                uri: resource_uri.into(),
            },
            descends_from: None,
            delegable: false,
            issued_at: Utc::now(),
            revoked_at: None,
        }
    }

    #[test]
    fn fundamental_name_expands_to_single_fundamental() {
        let set = expand_resource_to_fundamentals("filesystem_object");
        assert_eq!(set.len(), 1);
        assert!(set.contains(&Fundamental::FilesystemObject));
    }

    #[test]
    fn composite_name_expands_to_constituent_fundamentals() {
        let set = expand_resource_to_fundamentals("memory_object");
        // memory_object = data_object + tag
        assert_eq!(set.len(), 2);
        assert!(set.contains(&Fundamental::DataObject));
        assert!(set.contains(&Fundamental::Tag));
    }

    #[test]
    fn unknown_name_expands_to_empty() {
        assert!(expand_resource_to_fundamentals("nonsense_class").is_empty());
    }

    #[test]
    fn every_composite_expansion_includes_tag() {
        for c in Composite::ALL {
            let set = expand_resource_to_fundamentals(c.as_str());
            assert!(
                set.contains(&Fundamental::Tag),
                "{:?} expansion must include Tag fundamental",
                c
            );
        }
    }

    #[test]
    fn resolve_grant_on_fundamental_uri_produces_any_selector() {
        let g = sample_grant("filesystem_object", &["read"]);
        let r = resolve_grant(&g);
        assert_eq!(r.selector, Selector::Any);
        assert!(r.kind_refinement.is_none());
        assert!(r.fundamentals.contains(&Fundamental::FilesystemObject));
    }

    #[test]
    fn resolve_grant_on_composite_adds_kind_refinement() {
        let g = sample_grant("memory_object", &["read"]);
        let r = resolve_grant(&g);
        assert_eq!(r.kind_refinement, Some(Selector::KindTag("memory".into())));
        assert!(r.fundamentals.contains(&Fundamental::DataObject));
        assert!(r.fundamentals.contains(&Fundamental::Tag));
    }

    #[test]
    fn resolve_grant_on_opaque_instance_uri_parses_as_exact_selector() {
        let g = sample_grant("filesystem:/workspace/main.rs", &["read"]);
        let r = resolve_grant(&g);
        assert_eq!(
            r.selector,
            Selector::Exact("filesystem:/workspace/main.rs".into())
        );
        assert!(r.fundamentals.is_empty());
    }

    #[test]
    fn resolve_grant_on_prefix_uri_parses_as_prefix_selector() {
        let g = sample_grant("filesystem:/workspace/**", &["read"]);
        let r = resolve_grant(&g);
        assert_eq!(
            r.selector,
            Selector::Prefix("filesystem:/workspace/".into())
        );
    }

    #[test]
    fn resolve_grant_on_system_root_covers_all_fundamentals() {
        let g = sample_grant("system:root", &["allocate"]);
        let r = resolve_grant(&g);
        assert_eq!(r.selector, Selector::Any);
        assert_eq!(r.fundamentals.len(), Fundamental::ALL.len());
        assert!(r.kind_refinement.is_none());
    }

    #[test]
    fn with_fundamentals_injects_explicit_classes() {
        let g = sample_grant("filesystem:/workspace/**", &["read"]);
        let r =
            resolve_grant(&g).with_fundamentals([Fundamental::FilesystemObject, Fundamental::Tag]);
        assert_eq!(r.fundamentals.len(), 2);
        assert!(r.fundamentals.contains(&Fundamental::FilesystemObject));
    }

    #[test]
    fn covers_matches_action_and_fundamental() {
        let g = sample_grant("filesystem_object", &["read", "list"]);
        let r = resolve_grant(&g);
        assert!(r.covers(Fundamental::FilesystemObject, "read"));
        assert!(r.covers(Fundamental::FilesystemObject, "list"));
        assert!(!r.covers(Fundamental::FilesystemObject, "delete"));
        assert!(!r.covers(Fundamental::NetworkEndpoint, "read"));
    }

    #[test]
    fn covers_respects_star_action_wildcard() {
        let g = sample_grant("filesystem_object", &["*"]);
        let r = resolve_grant(&g);
        assert!(r.covers(Fundamental::FilesystemObject, "anything"));
    }

    #[test]
    fn effective_matches_respects_kind_refinement() {
        let g = sample_grant("memory_object", &["read"]);
        let r = resolve_grant(&g);
        // Memory grant must not match session entities.
        assert!(!r.effective_matches("session:s-1", &["#kind:session".into()]));
        assert!(r.effective_matches("memory:m-1", &["#kind:memory".into()]));
    }
}
