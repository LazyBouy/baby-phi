//! CH-06 proptest: AST → canonical-string → AST round-trip.
//!
//! Generates random selector ASTs from a bounded grammar, renders them via
//! the [`std::fmt::Display`] impls, re-parses with [`parse_selector`], and
//! asserts equality. Exercises the parser and the printer as inverse
//! functions over the predicates concept-09 §"Primary Predicates" + the
//! AND/OR/NOT combinators.

use domain::permissions::{parse_selector, BoolExpr, Predicate, Selector, SetRef, Tag};
use proptest::prelude::*;

// ---- Strategies ------------------------------------------------------------

fn ident_strategy() -> impl Strategy<Value = String> {
    // identifier = (alpha | _) (alphanum | _ | -)*
    "[a-z][a-z0-9_-]{0,4}".prop_filter("non-empty", |s| !s.is_empty())
}

fn tag_strategy() -> impl Strategy<Value = Tag> {
    prop_oneof![
        // Reserved with no value: e.g. `#archived`
        ident_strategy().prop_map(|ns| Tag::Reserved(ns, None)),
        // Reserved with value: e.g. `#kind:session`
        (ident_strategy(), ident_strategy()).prop_map(|(ns, v)| Tag::Reserved(ns, Some(v))),
        // Namespace: e.g. `org:acme/eng`
        (
            ident_strategy(),
            prop::collection::vec(ident_strategy(), 1..=3)
        )
            .prop_map(|(ns, parts)| Tag::Namespace(ns, parts)),
    ]
}

fn predicate_strategy() -> impl Strategy<Value = Predicate> {
    prop_oneof![
        Just(Predicate::Empty),
        Just(Predicate::NonEmpty),
        tag_strategy().prop_map(Predicate::Contains),
        prop::collection::vec(tag_strategy(), 1..=3).prop_map(Predicate::Intersects),
        // Glob with at least one wildcard so the P-004 check passes.
        ident_strategy().prop_map(|n| Predicate::AnyMatch(format!("{}/**", n))),
        (
            ident_strategy(),
            prop::collection::vec(ident_strategy(), 1..=2),
        )
            .prop_map(|(name, args)| Predicate::SubsetOf(SetRef { name, args })),
    ]
}

fn bool_expr_inner_strategy() -> impl Strategy<Value = BoolExpr> {
    let leaf = predicate_strategy().prop_map(BoolExpr::Pred);
    leaf.prop_recursive(
        3,  // depth
        16, // total nodes
        4,  // max children per node
        |inner| {
            prop_oneof![
                inner.clone().prop_map(|i| BoolExpr::Not(Box::new(i))),
                prop::collection::vec(inner.clone(), 2..=3).prop_map(BoolExpr::And),
                prop::collection::vec(inner, 2..=3).prop_map(BoolExpr::Or),
            ]
        },
    )
}

/// Top-level `BoolExpr` must be genuinely composite (And/Or/Not). A bare
/// `BoolExpr::Pred(_)` would collapse to `Selector::Pred` after parsing
/// (see `simplify_to_selector` in the parser), breaking round-trip.
fn bool_expr_composite_strategy() -> impl Strategy<Value = BoolExpr> {
    prop_oneof![
        bool_expr_inner_strategy().prop_map(|i| BoolExpr::Not(Box::new(i))),
        prop::collection::vec(bool_expr_inner_strategy(), 2..=3).prop_map(BoolExpr::And),
        prop::collection::vec(bool_expr_inner_strategy(), 2..=3).prop_map(BoolExpr::Or),
    ]
}

fn selector_strategy() -> impl Strategy<Value = Selector> {
    prop_oneof![
        predicate_strategy().prop_map(Selector::Pred),
        bool_expr_composite_strategy().prop_map(|b| Selector::Bool(Box::new(b))),
    ]
}

// ---- The proptest -----------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// AST → Display → parse_selector → AST round-trips. The Display impls
    /// emit canonical, fully-parenthesised expressions (BoolExpr) so the
    /// grammar's precedence rules cannot reshape the parse tree.
    #[test]
    fn ast_round_trip_via_display(s in selector_strategy()) {
        let rendered = s.to_string();
        let parsed = parse_selector(&rendered)
            .unwrap_or_else(|e| panic!("re-parse failed for {:?}: {} (rendered: {})", s, e, rendered));
        prop_assert_eq!(s, parsed, "round-trip failed; rendered: {}", rendered);
    }
}
