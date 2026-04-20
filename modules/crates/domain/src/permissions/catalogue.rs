//! The `CatalogueLookup` trait used by Step 0 of the Permission Check.
//!
//! The catalogue is the `resources_catalogue` table scoped to an org (see
//! `concepts/permissions/01-resource-ontology.md` §Resource Catalogue). The
//! engine only needs a point lookup — "is this URI in the catalogue?" — so
//! the trait is deliberately minimal.
//!
//! Production code wires `Repository::catalogue_contains` through a tiny
//! adapter; unit tests and proptests use [`StaticCatalogue`] to feed a
//! fixed set.

use std::collections::HashSet;

use crate::model::ids::OrgId;

/// Step-0 precondition lookup. Returns `true` iff the resource is declared
/// in the catalogue for the given owning org.
///
/// Platform-level resources (`system:root`, the bootstrap `control_plane`
/// node) are keyed under `owning_org = None`; everything else lives under
/// its owning org.
pub trait CatalogueLookup: Send + Sync {
    fn contains(&self, owning_org: Option<OrgId>, resource_uri: &str) -> bool;
}

/// An in-memory [`CatalogueLookup`] backed by a `HashSet`. Construct with
/// [`StaticCatalogue::seed`] / [`StaticCatalogue::with_entries`] and pass
/// to [`crate::permissions::check`] via `CheckContext.catalogue`.
#[derive(Debug, Clone, Default)]
pub struct StaticCatalogue {
    entries: HashSet<(Option<OrgId>, String)>,
}

impl StaticCatalogue {
    /// Empty catalogue — every lookup returns `false`.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Seed a catalogue with the given `(owning_org, uri)` pairs.
    pub fn with_entries<I: IntoIterator<Item = (Option<OrgId>, String)>>(pairs: I) -> Self {
        Self {
            entries: pairs.into_iter().collect(),
        }
    }

    /// Add a single entry. Returns `&mut self` for chaining.
    pub fn seed(
        &mut self,
        owning_org: Option<OrgId>,
        resource_uri: impl Into<String>,
    ) -> &mut Self {
        self.entries.insert((owning_org, resource_uri.into()));
        self
    }
}

impl CatalogueLookup for StaticCatalogue {
    fn contains(&self, owning_org: Option<OrgId>, resource_uri: &str) -> bool {
        self.entries
            .contains(&(owning_org, resource_uri.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_catalogue_misses_everything() {
        let c = StaticCatalogue::empty();
        assert!(!c.contains(None, "system:root"));
        assert!(!c.contains(Some(OrgId::new()), "filesystem:/workspace/"));
    }

    #[test]
    fn seeded_entry_hits() {
        let mut c = StaticCatalogue::empty();
        c.seed(None, "system:root");
        assert!(c.contains(None, "system:root"));
    }

    #[test]
    fn entries_under_org_are_separate_from_platform() {
        let org = OrgId::new();
        let c = StaticCatalogue::with_entries([
            (None, "system:root".to_string()),
            (Some(org), "filesystem:/workspace/".to_string()),
        ]);
        assert!(c.contains(None, "system:root"));
        assert!(c.contains(Some(org), "filesystem:/workspace/"));
        // Cross-scope lookups miss.
        assert!(!c.contains(Some(org), "system:root"));
        assert!(!c.contains(None, "filesystem:/workspace/"));
    }
}
