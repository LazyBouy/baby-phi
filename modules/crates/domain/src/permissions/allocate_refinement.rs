//! CH-08 / ADR-0052 §D52.2 — typed `AllocateRefinement` constraint per concept-doc 02
//! line 197 + concept-doc 03 line 54.
//!
//! Concept-doc 02 line 197: *"Specific refinements can be expressed as constraints on the
//! Grant. For example, `allocate: no_further_delegation` as a constraint removes the
//! allocate-with-delegation sub-capability, producing a 'one-level' shareholder who can
//! issue operational sub-grants but cannot create further shareholders."*
//!
//! v0 field set (F2.A user-locked at plan approval): minimal closure of fields the
//! forward-scope row + concept doc 02 line 197 explicitly name. Other fields stay
//! deferred to drift-list expansion when concrete use-cases land.

use serde::{Deserialize, Serialize};

/// Typed refinement constraints for a `[allocate]`-scope grant.
///
/// **Default**: `{ no_further_delegation: false, max_depth: None }` — unbounded
/// allocation authority, matching pre-CH-08 behaviour.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AllocateRefinement {
    /// When `true`, removes the allocate-with-delegation sub-capability —
    /// a "one-level" shareholder who can issue operational sub-grants but
    /// cannot create further shareholders. Concept-doc 02 line 197 verbatim.
    #[serde(default)]
    pub no_further_delegation: bool,

    /// When `Some(n)`, limits the depth of the allocation chain. `Some(1)` =
    /// "one-level shareholder" per concept-doc framing; `None` = unbounded.
    #[serde(default)]
    pub max_depth: Option<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_unbounded() {
        let r = AllocateRefinement::default();
        assert!(!r.no_further_delegation);
        assert_eq!(r.max_depth, None);
    }

    #[test]
    fn serde_round_trip() {
        let r = AllocateRefinement {
            no_further_delegation: true,
            max_depth: Some(2),
        };
        let json = serde_json::to_string(&r).unwrap();
        let r2: AllocateRefinement = serde_json::from_str(&json).unwrap();
        assert_eq!(r, r2);
    }

    #[test]
    fn legacy_grant_decodes_with_none() {
        // Pre-CH-08 grants don't carry `allocate_refinement`; verify a Grant
        // missing the field decodes with `Grant.allocate_refinement = None`.
        // Synthesise a JSON shape mirroring a pre-CH-08 stored grant (omits
        // `allocate_refinement` entirely; relies on the field's
        // `#[serde(default)]` shielding on Grant per ADR-0052 §D52.3).
        use crate::model::nodes::Grant;
        let legacy_json = serde_json::json!({
            "id": "00000000-0000-0000-0000-000000000001",
            "holder": { "system": "system:genesis" },
            "action": ["read"],
            "resource": { "uri": "system:root" },
            "fundamentals": [],
            "descends_from": null,
            "delegable": false,
            "issued_at": "2026-05-07T00:00:00Z",
            "revoked_at": null,
            "approval_mode": { "kind": "implicit" }
            // Note: `audit_class` and `allocate_refinement` both omitted —
            // both rely on serde-default shielding (CH-13 D50.5 + CH-08 D52.3).
        });
        let grant: Grant = serde_json::from_value(legacy_json).unwrap();
        assert!(grant.allocate_refinement.is_none());
    }
}
