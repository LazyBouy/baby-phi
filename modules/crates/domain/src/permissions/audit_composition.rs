//! `audit_class` strictest-wins composition (CH-13 / ADR-0050).
//!
//! Concept-doc anchor:
//! `docs/specs/v0/concepts/permissions/07-templates-and-tools.md`
//! §"audit_class Composition Through Templates" lines 63–71.
//!
//! When a template fires a Grant, the resolved [`AuditClass`] is the
//! **strictest** of three candidate inputs:
//!
//! - **(a) org-level default** — [`Organization::audit_class_default`] at
//!   the firing org.
//! - **(b) template adoption Auth Request's `audit_class`** — the
//!   adoption AR that authorised the template's enablement.
//! - **(c) optional per-Grant override** — supplied by the template at
//!   fire time (currently `None` for production Templates A/C/D; reserved
//!   for forward-scope templates).
//!
//! The ordering loosest → strictest is `Silent < Logged < Alerted` per
//! [`AuditClass`]'s `PartialOrd, Ord` derive (concept-doc-07 line 67's
//! `none < logged < alerted`; `none ↔ Silent` per ADR-0050 §D50.1). Per
//! concept-doc-07 line 68, **per-Grant overrides may only escalate** —
//! the `max`-fold makes any override below the (a)+(b) maximum a no-op
//! structurally.
//!
//! Per concept-doc-07 line 69, the **winning input** is also recorded so
//! operators can see what was applied; [`compose_audit_class_with_source`]
//! returns the `(resolved, source)` tuple. Tie-breaker per ADR-0050
//! §D50.3: when 2+ inputs tie at the strictest, more-specific source
//! wins — `Override > TemplateAr > OrgDefault`.

use serde::{Deserialize, Serialize};

use crate::audit::AuditClass;

/// Which of the three candidate inputs supplied the resolved
/// [`AuditClass`].
///
/// Serialised in the audit-event diff (per concept-doc-07 line 69's
/// "along with which of (a)/(b)/(c) supplied the winning value"); the
/// wire format is the snake-case string `"org_default"` /
/// `"template_ar"` / `"override"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditClassSource {
    /// The org's [`Organization::audit_class_default`] supplied the
    /// winning value.
    OrgDefault,
    /// The template adoption Auth Request's `audit_class` supplied the
    /// winning value.
    TemplateAr,
    /// The template's per-Grant override supplied the winning value.
    Override,
}

/// Compose the resolved [`AuditClass`] strictest-wins of (a) the
/// org-level default, (b) the template adoption Auth Request's
/// `audit_class`, and (c) the optional per-Grant override.
///
/// Concept-doc anchor: `permissions/07-templates-and-tools.md` line 67
/// (`none < logged < alerted`; loosest → strictest). Per-Grant overrides
/// may **only** escalate (line 68) — the `max`-fold rejects any override
/// below the (a)+(b) max structurally.
///
/// # Example
///
/// ```ignore
/// use crate::audit::AuditClass;
/// use crate::permissions::audit_composition::compose_audit_class;
///
/// // Org compliance posture is Alerted; template AR is Logged; no
/// // override. Strictest wins → Alerted (no silent downgrade).
/// assert_eq!(
///     compose_audit_class(AuditClass::Alerted, AuditClass::Logged, None),
///     AuditClass::Alerted,
/// );
/// ```
pub fn compose_audit_class(
    org_default: AuditClass,
    template_ar: AuditClass,
    r#override: Option<AuditClass>,
) -> AuditClass {
    compose_audit_class_with_source(org_default, template_ar, r#override).0
}

/// Compose the resolved [`AuditClass`] **and** name the winning source.
///
/// Concept-doc-07 line 69 — operators must see "which of (a)/(b)/(c)
/// supplied the winning value". The 3 fire listeners feed this tuple
/// into the audit-event diff via the `audit_class_source` string field.
///
/// Tie-breaker per ADR-0050 §D50.3: when 2+ inputs tie at the strictest,
/// more-specific source wins — `Override > TemplateAr > OrgDefault`. The
/// natural `max()` fold does NOT capture this (it returns the first hit
/// at the strictest tier in iteration order); we walk the candidates in
/// **reverse priority order** so the most-specific source overwrites
/// less-specific ties.
pub fn compose_audit_class_with_source(
    org_default: AuditClass,
    template_ar: AuditClass,
    r#override: Option<AuditClass>,
) -> (AuditClass, AuditClassSource) {
    // Walk in priority order from least-specific to most-specific.
    // The strict `>` comparison ensures ties leave the previous (more-
    // specific) winner in place when iterated the OTHER way; we instead
    // iterate least → most specific, using `>=` so a later (more-
    // specific) candidate at the same strictness tier wins the tie.
    let mut resolved = org_default;
    let mut source = AuditClassSource::OrgDefault;

    if template_ar >= resolved {
        resolved = template_ar;
        source = AuditClassSource::TemplateAr;
    }

    if let Some(o) = r#override {
        if o >= resolved {
            resolved = o;
            source = AuditClassSource::Override;
        }
    }

    (resolved, source)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    /// D50.2 — variant ordering encodes loosest → strictest.
    #[test]
    fn unit_silent_loosest() {
        assert!(AuditClass::Silent < AuditClass::Logged);
        assert!(AuditClass::Logged < AuditClass::Alerted);
        assert!(AuditClass::Silent < AuditClass::Alerted);
    }

    /// Concept-doc 07 line 67 — strictest of (a, b, c?) wins.
    /// Truth table over 3 × 3 × 4 = 36 combos (4th override slot = None).
    #[test]
    fn unit_compose_returns_strictest_of_three() {
        let classes = [AuditClass::Silent, AuditClass::Logged, AuditClass::Alerted];
        let overrides = [
            None,
            Some(AuditClass::Silent),
            Some(AuditClass::Logged),
            Some(AuditClass::Alerted),
        ];

        for &org in &classes {
            for &tpl in &classes {
                for &ov in &overrides {
                    let resolved = compose_audit_class(org, tpl, ov);
                    let candidates: Vec<AuditClass> =
                        [Some(org), Some(tpl), ov].into_iter().flatten().collect();
                    let expected = *candidates.iter().max().expect("at least 2 candidates");
                    assert_eq!(
                        resolved, expected,
                        "compose({:?}, {:?}, {:?}) = {:?} but max-of-candidates = {:?}",
                        org, tpl, ov, resolved, expected
                    );
                }
            }
        }
    }

    /// Concept-doc 07 line 68 — overrides may only escalate.
    /// For every (org, tpl) pair: if override < max(org, tpl) the
    /// override is silently rejected (max wins); if override ≥
    /// max(org, tpl) the override becomes the result.
    #[test]
    fn unit_override_can_only_escalate() {
        let classes = [AuditClass::Silent, AuditClass::Logged, AuditClass::Alerted];

        for &org in &classes {
            for &tpl in &classes {
                let pair_max = std::cmp::max(org, tpl);
                for &ov in &classes {
                    let resolved = compose_audit_class(org, tpl, Some(ov));
                    if ov < pair_max {
                        // Override below pair-max → silently rejected.
                        assert_eq!(
                            resolved, pair_max,
                            "override {:?} < max({:?}, {:?}) = {:?} should be no-op; got {:?}",
                            ov, org, tpl, pair_max, resolved,
                        );
                    } else {
                        // Override ≥ pair-max → escalates (or ties at
                        // pair-max with override winning the tie).
                        assert_eq!(
                            resolved, ov,
                            "override {:?} ≥ max({:?}, {:?}) = {:?} should be the result; got {:?}",
                            ov, org, tpl, pair_max, resolved,
                        );
                    }
                }
            }
        }
    }

    /// D50.6 / concept-doc-07 line 69 — the winning source is named.
    #[test]
    fn unit_compose_with_source_attributes_winning_input() {
        // Org default winning: highest is org_default, all others lower.
        let (resolved, source) =
            compose_audit_class_with_source(AuditClass::Alerted, AuditClass::Silent, None);
        assert_eq!(resolved, AuditClass::Alerted);
        assert_eq!(source, AuditClassSource::OrgDefault);

        // Template AR winning: tpl > org, no override.
        let (resolved, source) =
            compose_audit_class_with_source(AuditClass::Silent, AuditClass::Alerted, None);
        assert_eq!(resolved, AuditClass::Alerted);
        assert_eq!(source, AuditClassSource::TemplateAr);

        // Override winning: override > both.
        let (resolved, source) = compose_audit_class_with_source(
            AuditClass::Silent,
            AuditClass::Logged,
            Some(AuditClass::Alerted),
        );
        assert_eq!(resolved, AuditClass::Alerted);
        assert_eq!(source, AuditClassSource::Override);
    }

    /// D50.3 — tie-breaker: when 2+ inputs tie at the strictest,
    /// `Override > TemplateAr > OrgDefault`.
    #[test]
    fn unit_compose_with_source_tie_breaker() {
        // org == tpl, no override → TemplateAr wins (more specific
        // than OrgDefault).
        let (resolved, source) =
            compose_audit_class_with_source(AuditClass::Logged, AuditClass::Logged, None);
        assert_eq!(resolved, AuditClass::Logged);
        assert_eq!(source, AuditClassSource::TemplateAr);

        // org == tpl == override → Override wins (most specific).
        let (resolved, source) = compose_audit_class_with_source(
            AuditClass::Logged,
            AuditClass::Logged,
            Some(AuditClass::Logged),
        );
        assert_eq!(resolved, AuditClass::Logged);
        assert_eq!(source, AuditClassSource::Override);

        // tpl == override at the strictest, org lower → Override wins.
        let (resolved, source) = compose_audit_class_with_source(
            AuditClass::Silent,
            AuditClass::Alerted,
            Some(AuditClass::Alerted),
        );
        assert_eq!(resolved, AuditClass::Alerted);
        assert_eq!(source, AuditClassSource::Override);

        // org == override at the strictest, tpl lower → Override wins
        // (still more specific than OrgDefault).
        let (resolved, source) = compose_audit_class_with_source(
            AuditClass::Alerted,
            AuditClass::Silent,
            Some(AuditClass::Alerted),
        );
        assert_eq!(resolved, AuditClass::Alerted);
        assert_eq!(source, AuditClassSource::Override);
    }

    /// D50.5 — `Grant::default_audit_class()` must return `Silent`
    /// (loosest); pre-CH-13 grants decode safely.
    #[test]
    fn unit_grant_serde_default_audit_class_is_silent() {
        use crate::model::nodes::Grant;
        assert_eq!(Grant::default_audit_class(), AuditClass::Silent);
    }

    proptest! {
        /// D50.2 — `Ord` derive: any vec containing `Alerted` has it
        /// as max.
        #[test]
        fn unit_audit_class_ord_derive_property(
            classes in proptest::collection::vec(
                prop_oneof![
                    Just(AuditClass::Silent),
                    Just(AuditClass::Logged),
                    Just(AuditClass::Alerted),
                ],
                1..16,
            )
        ) {
            if classes.contains(&AuditClass::Alerted) {
                prop_assert_eq!(classes.iter().max(), Some(&AuditClass::Alerted));
            } else if classes.contains(&AuditClass::Logged) {
                prop_assert_eq!(classes.iter().max(), Some(&AuditClass::Logged));
            } else {
                prop_assert_eq!(classes.iter().max(), Some(&AuditClass::Silent));
            }
        }
    }
}
