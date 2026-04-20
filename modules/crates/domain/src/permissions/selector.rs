//! Resource-selector grammar used inside the Permission Check engine.
//!
//! For v0.1 we stay deliberately small: three explicit match modes plus an
//! `Any` fallback. Richer predicates (regex, tag-intersection, numeric
//! ranges) are tracked for M2+ but would explode the surface here without
//! buying anything the M1 spine needs. The four modes together cover:
//!
//! - The bootstrap `system:root` grant (exact-match).
//! - Filesystem-prefix grants like `filesystem:/workspace/**` (prefix).
//! - Composite-kind filters like `#kind:memory` from `resolve_grant`
//!   (kind-tag).
//! - Catch-all `*` fallbacks (any).
//!
//! The parser is total — every string maps to a [`Selector`].
//!
//! ```
//! use domain::permissions::Selector;
//! assert!(matches!(Selector::parse("system:root"),
//!     Selector::Exact(ref s) if s == "system:root"));
//! ```
//!
//! Source cross-ref:
//! `docs/specs/v0/concepts/permissions/04-manifest-and-resolution.md`
//! §Grant as a Graph Node (for why selector doubles as the URI shape in
//! v0.1).

use serde::{Deserialize, Serialize};

/// One selector expression parsed from a grant's `resource.uri`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum Selector {
    /// Matches anything. Written as the literal `"*"` in a grant URI.
    Any,
    /// Exact string equality against the target URI. E.g. `system:root`.
    Exact(String),
    /// The target URI must start with this prefix. Written as
    /// `"<prefix>**"`; the trailing `**` is stripped by the parser.
    Prefix(String),
    /// The target's tags must contain the given `#kind:<name>` tag.
    /// Written as the literal `"#kind:<name>"`.
    KindTag(String),
}

impl Selector {
    /// Parse a selector from a grant `resource.uri`. Total: every input
    /// produces some [`Selector`].
    pub fn parse(uri: &str) -> Self {
        if uri == "*" {
            Selector::Any
        } else if let Some(kind) = uri.strip_prefix("#kind:") {
            Selector::KindTag(kind.to_string())
        } else if let Some(prefix) = uri.strip_suffix("**") {
            Selector::Prefix(prefix.to_string())
        } else {
            Selector::Exact(uri.to_string())
        }
    }

    /// Does this selector match the call's target?
    ///
    /// - `Any` → always.
    /// - `Exact(s)` → `target_uri == s`.
    /// - `Prefix(p)` → `target_uri.starts_with(p)`.
    /// - `KindTag(k)` → target tags contain `#kind:<k>`.
    pub fn matches(&self, target_uri: &str, target_tags: &[String]) -> bool {
        match self {
            Selector::Any => true,
            Selector::Exact(s) => target_uri == s,
            Selector::Prefix(p) => target_uri.starts_with(p),
            Selector::KindTag(k) => {
                let needle = format!("#kind:{}", k);
                target_tags.iter().any(|t| t == &needle)
            }
        }
    }

    /// Compose two selectors with logical AND. Used by `resolve_grant` to
    /// narrow a composite grant's explicit selector with the implicit
    /// `#kind:` refinement.
    pub fn and(self, other: Selector) -> AndSelector {
        AndSelector {
            left: self,
            right: other,
        }
    }
}

/// Conjunction of two selectors — both must match.
///
/// The engine uses this to combine a composite grant's explicit selector
/// with the implicit `#kind:` refinement per `resolve_grant` in
/// `permissions/04-manifest-and-resolution.md` §Refinement.
#[derive(Debug, Clone)]
pub struct AndSelector {
    pub left: Selector,
    pub right: Selector,
}

impl AndSelector {
    pub fn matches(&self, target_uri: &str, target_tags: &[String]) -> bool {
        self.left.matches(target_uri, target_tags) && self.right.matches(target_uri, target_tags)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn star_parses_as_any() {
        assert!(matches!(Selector::parse("*"), Selector::Any));
    }

    #[test]
    fn kind_tag_literal_parses_as_kind_tag() {
        let s = Selector::parse("#kind:memory");
        assert_eq!(s, Selector::KindTag("memory".into()));
    }

    #[test]
    fn double_star_suffix_parses_as_prefix() {
        let s = Selector::parse("filesystem:/workspace/**");
        assert_eq!(s, Selector::Prefix("filesystem:/workspace/".into()));
    }

    #[test]
    fn plain_literal_parses_as_exact() {
        assert_eq!(
            Selector::parse("system:root"),
            Selector::Exact("system:root".into())
        );
    }

    #[test]
    fn any_matches_everything() {
        let s = Selector::Any;
        assert!(s.matches("", &[]));
        assert!(s.matches("system:root", &["#kind:any".to_string()]));
    }

    #[test]
    fn exact_matches_only_identical_uri() {
        let s = Selector::Exact("system:root".into());
        assert!(s.matches("system:root", &[]));
        assert!(!s.matches("system:rootx", &[]));
        assert!(!s.matches("system:roo", &[]));
    }

    #[test]
    fn prefix_matches_uri_beginning_with_prefix() {
        let s = Selector::Prefix("filesystem:/workspace/".into());
        assert!(s.matches("filesystem:/workspace/main.rs", &[]));
        assert!(s.matches("filesystem:/workspace/sub/dir/file", &[]));
        assert!(!s.matches("filesystem:/other/file", &[]));
    }

    #[test]
    fn kind_tag_matches_when_tag_present() {
        let s = Selector::KindTag("memory".into());
        assert!(s.matches("memory:m-1", &["#kind:memory".into()]));
        assert!(!s.matches("memory:m-1", &["#kind:session".into()]));
        assert!(!s.matches("memory:m-1", &[]));
    }

    #[test]
    fn and_selector_requires_both_to_match() {
        let a = Selector::Prefix("memory:".into()).and(Selector::KindTag("memory".into()));
        assert!(a.matches("memory:m-1", &["#kind:memory".into()]));
        assert!(!a.matches("session:s-1", &["#kind:memory".into()]));
        assert!(!a.matches("memory:m-1", &["#kind:session".into()]));
    }
}
