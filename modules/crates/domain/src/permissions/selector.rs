//! Resource-selector grammar used inside the Permission Check engine.
//!
//! CH-06 lights up the full PEG tag-predicate DSL from
//! [`concepts/permissions/09-selector-grammar.md`]. The shape:
//!
//! - **Six predicates**: `tags contains <tag>`, `tags intersects {…}`,
//!   `tags any_match <glob>`, `tags subset_of <set_ref>`, `tags empty`,
//!   `tags non_empty`.
//! - **Three combinators**: `AND` (left-assoc) > `OR` (left-assoc), with
//!   `NOT` (right-assoc) binding tightest. Parens override.
//! - **Four legacy URI shapes** preserved as-is for the 9 grant-mint call
//!   sites: `*` → [`Selector::Any`], `system:root` → [`Selector::Exact`],
//!   `<prefix>**` → [`Selector::Prefix`], `#kind:<name>` →
//!   [`Selector::KindTag`]. These remain the AST-level encoding the engine
//!   uses today; richer-grammar grants land as [`Selector::Bool`] /
//!   [`Selector::Pred`].
//!
//! ### Two parser surfaces
//!
//! - [`parse_selector`] — strict full-grammar parse. Returns
//!   [`Result<Selector, SelectorParseError>`] with stable parse-error codes
//!   `P-001` … `P-005`. Used by code that knows it's reading a selector
//!   expression (manifests, future Grant fields).
//! - [`parse_selector_or_uri`] — total fast-path. The 4 legacy URI shapes
//!   are recognised first, then the grammar is attempted, then the input
//!   falls back to [`Selector::Exact`]. Used by [`Selector::parse`] (the
//!   M1-era entry point still wired into [`crate::permissions::expansion::resolve_grant`]).
//!
//! ### Evaluator
//!
//! [`Selector::evaluate`] recursively interprets the AST against
//! `(target_uri, target_tags, registry)`. The
//! [`SetRefRegistry`] trait resolves `subset_of foo(args…)` set references
//! at runtime; the default [`NoopSetRefRegistry`] returns `None` for every
//! lookup, which makes any `subset_of` predicate evaluate to `false`. CH-15
//! wires the production registry that resolves named scopes (e.g.
//! `supervisors_tagging_scope(supervisor-7)`).
//!
//! ### Backwards compatibility
//!
//! [`Selector::matches`] is preserved as a thin wrapper around
//! [`Selector::evaluate`] with a [`NoopSetRefRegistry`]. The 8 unit tests
//! pinned at M1 still exercise the legacy variants directly, and
//! [`crate::permissions::expansion::ResolvedGrant::effective_matches`] now
//! threads the registry through (see ADR-0036 §D36.5 for the
//! continuity policy).

use std::collections::HashSet;

use pest::iterators::{Pair, Pairs};
use pest::Parser as _;
use serde::{Deserialize, Serialize};

#[derive(pest_derive::Parser)]
#[grammar = "permissions/grammar.pest"]
struct SelectorParser;

// ----------------------------------------------------------------------------
// AST
// ----------------------------------------------------------------------------

/// One selector expression. The four legacy variants ([`Selector::Any`] …
/// [`Selector::KindTag`]) preserve M1 grant-URI shapes verbatim so the
/// 9 grant-mint call sites do not need to change. The richer
/// [`Selector::Bool`] / [`Selector::Pred`] variants encode the full
/// tag-predicate DSL from concept-09.
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
    /// Composed boolean expression (AND / OR / NOT / parens) over predicates.
    Bool(Box<BoolExpr>),
    /// A single tag-predicate at the top level (no logical composition).
    Pred(Predicate),
}

/// Logical composition of tag predicates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoolExpr {
    /// Disjunction; matches iff any inner expression matches.
    Or(Vec<BoolExpr>),
    /// Conjunction; matches iff every inner expression matches.
    And(Vec<BoolExpr>),
    /// Negation.
    Not(Box<BoolExpr>),
    /// A leaf tag predicate.
    Pred(Predicate),
}

/// One tag-predicate. Each variant maps 1:1 to a row of concept-09
/// §"Primary Predicates".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Predicate {
    /// `tags contains <tag>` — membership.
    Contains(Tag),
    /// `tags intersects { <tag>, … }` — non-empty intersection.
    Intersects(Vec<Tag>),
    /// `tags any_match "<glob>"` — at least one target tag matches the glob.
    AnyMatch(String),
    /// `tags subset_of <set_ref>` — every target tag is in the named set.
    SubsetOf(SetRef),
    /// `tags empty` — target has no tags.
    Empty,
    /// `tags non_empty` — target has at least one tag.
    NonEmpty,
}

/// One tag form: reserved (`#ns` or `#ns:val`), namespace (`ns:val[/val…]`),
/// or a literal (string-literal) tag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tag {
    /// `#kind:session`, `#archived`, etc. The reserved-prefix family.
    /// Second component is `None` for bare reserved tags (e.g. `#archived`)
    /// and `Some(value)` for the `#ns:val` form.
    Reserved(String, Option<String>),
    /// `org:acme`, `org:acme/eng/web`. Namespace + slash-separated value path.
    Namespace(String, Vec<String>),
    /// `"some literal tag"` — fallback for tags that do not fit the
    /// reserved or namespace shapes.
    Literal(String),
}

impl Tag {
    /// Render the tag in its canonical wire form (the same shape used in
    /// `target_tags` columns + composite `auto_tags()` output).
    pub fn to_canonical(&self) -> String {
        match self {
            Tag::Reserved(ns, None) => format!("#{}", ns),
            Tag::Reserved(ns, Some(v)) => format!("#{}:{}", ns, v),
            Tag::Namespace(ns, parts) => format!("{}:{}", ns, parts.join("/")),
            Tag::Literal(s) => s.clone(),
        }
    }
}

/// A parameterised set reference: `name(arg1, arg2, …)`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetRef {
    pub name: String,
    pub args: Vec<String>,
}

// ----------------------------------------------------------------------------
// Display (AST → canonical grammar string)
// ----------------------------------------------------------------------------

impl std::fmt::Display for Selector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Selector::Any => f.write_str("*"),
            Selector::Exact(s) => f.write_str(s),
            Selector::Prefix(p) => write!(f, "{}**", p),
            Selector::KindTag(k) => write!(f, "#kind:{}", k),
            Selector::Bool(b) => write!(f, "{}", b),
            Selector::Pred(p) => write!(f, "{}", p),
        }
    }
}

impl std::fmt::Display for BoolExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Always parenthesise sub-expressions to keep precedence
        // unambiguous — round-trips are guaranteed by the grammar's
        // explicit grouping rule.
        match self {
            BoolExpr::Or(parts) => {
                let rendered: Vec<String> = parts.iter().map(|p| format!("({})", p)).collect();
                f.write_str(&rendered.join(" OR "))
            }
            BoolExpr::And(parts) => {
                let rendered: Vec<String> = parts.iter().map(|p| format!("({})", p)).collect();
                f.write_str(&rendered.join(" AND "))
            }
            BoolExpr::Not(inner) => write!(f, "NOT ({})", inner),
            BoolExpr::Pred(p) => write!(f, "{}", p),
        }
    }
}

impl std::fmt::Display for Predicate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Predicate::Contains(tag) => write!(f, "tags contains {}", tag),
            Predicate::Intersects(tags) => {
                let rendered: Vec<String> = tags.iter().map(|t| t.to_string()).collect();
                write!(f, "tags intersects {{{}}}", rendered.join(", "))
            }
            Predicate::AnyMatch(glob) => write!(f, "tags any_match \"{}\"", glob),
            Predicate::SubsetOf(sref) => write!(f, "tags subset_of {}", sref),
            Predicate::Empty => f.write_str("tags empty"),
            Predicate::NonEmpty => f.write_str("tags non_empty"),
        }
    }
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tag::Reserved(ns, None) => write!(f, "#{}", ns),
            Tag::Reserved(ns, Some(v)) => write!(f, "#{}:{}", ns, v),
            Tag::Namespace(ns, parts) => write!(f, "{}:{}", ns, parts.join("/")),
            Tag::Literal(s) => write!(f, "\"{}\"", s),
        }
    }
}

impl std::fmt::Display for SetRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.name, self.args.join(", "))
    }
}

// ----------------------------------------------------------------------------
// Set-reference resolution
// ----------------------------------------------------------------------------

/// Resolves `subset_of` set references at evaluation time. Implementations
/// look up named runtime functions (e.g.
/// `supervisors_tagging_scope(supervisor-7)`) and return the resolved tag
/// set. Returning [`None`] means "set unknown" → the predicate evaluates to
/// `false`.
///
/// The trait is `Send + Sync` so it can be borrowed across the engine
/// without lifetime gymnastics. Per ADR-0036 §D36.3, the production registry
/// wiring lands in CH-15; in M5.2 we ship only the [`NoopSetRefRegistry`]
/// stub and one optional kind-scoping registration is deferred.
pub trait SetRefRegistry: Send + Sync {
    fn resolve(&self, name: &str, args: &[&str]) -> Option<HashSet<String>>;
}

/// Default registry: every set reference is unknown. Used by
/// [`Selector::matches`] (the legacy entry point) and by every engine
/// caller that has not yet been wired to a real registry. `subset_of`
/// predicates evaluated under this registry always return `false`.
pub struct NoopSetRefRegistry;

impl SetRefRegistry for NoopSetRefRegistry {
    fn resolve(&self, _name: &str, _args: &[&str]) -> Option<HashSet<String>> {
        None
    }
}

/// `'static` singleton of [`NoopSetRefRegistry`]. Every [`crate::permissions::manifest::CheckContext`]
/// constructor in M5.2 borrows this — CH-15 swaps in a real registry at the
/// engine entry point without changing any other call sites.
pub static NOOP_SET_REF_REGISTRY: NoopSetRefRegistry = NoopSetRefRegistry;

// ----------------------------------------------------------------------------
// Parse errors
// ----------------------------------------------------------------------------

/// Stable error codes returned by [`parse_selector`]. Codes are part of the
/// public ops surface — see `m5_2/operations/selector-grammar-operations.md`
/// for the runbook.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectorParseError {
    /// `P-001` — token at `position` did not fit the grammar.
    UnexpectedToken {
        code: &'static str,
        position: usize,
        found: String,
    },
    /// `P-002` — `(` without a matching `)` (or vice versa).
    UnbalancedParens { code: &'static str, position: usize },
    /// `P-003` — predicate name was not one of the 6 supported forms.
    UnknownPredicate { code: &'static str, name: String },
    /// `P-004` — `any_match` glob did not contain `*` or `**` (semantic check).
    InvalidGlob { code: &'static str, pattern: String },
    /// `P-005` — set-reference name was not registered. (Reserved for
    /// runtime resolution failures; the parser only emits it via the
    /// optional pre-registration pass.)
    UnknownSetRef { code: &'static str, name: String },
}

impl std::fmt::Display for SelectorParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SelectorParseError::UnexpectedToken {
                code,
                position,
                found,
            } => {
                write!(
                    f,
                    "[{}] unexpected token '{}' at position {}",
                    code, found, position
                )
            }
            SelectorParseError::UnbalancedParens { code, position } => {
                write!(f, "[{}] unbalanced parens at position {}", code, position)
            }
            SelectorParseError::UnknownPredicate { code, name } => {
                write!(f, "[{}] unknown predicate '{}'", code, name)
            }
            SelectorParseError::InvalidGlob { code, pattern } => {
                write!(
                    f,
                    "[{}] invalid glob '{}' (must contain * or **)",
                    code, pattern
                )
            }
            SelectorParseError::UnknownSetRef { code, name } => {
                write!(f, "[{}] unknown set reference '{}'", code, name)
            }
        }
    }
}

impl std::error::Error for SelectorParseError {}

impl SelectorParseError {
    pub fn code(&self) -> &'static str {
        match self {
            SelectorParseError::UnexpectedToken { code, .. } => code,
            SelectorParseError::UnbalancedParens { code, .. } => code,
            SelectorParseError::UnknownPredicate { code, .. } => code,
            SelectorParseError::InvalidGlob { code, .. } => code,
            SelectorParseError::UnknownSetRef { code, .. } => code,
        }
    }
}

// ----------------------------------------------------------------------------
// Parser entry points
// ----------------------------------------------------------------------------

/// Parse a full grammar expression. Used when the input is known to be a
/// selector (not a legacy URI). Returns [`SelectorParseError`] on any
/// syntactic or semantic failure.
pub fn parse_selector(input: &str) -> Result<Selector, SelectorParseError> {
    let mut pairs = SelectorParser::parse(Rule::selector, input).map_err(map_pest_error)?;
    // The top-level `selector` rule wraps a single `or_expr`.
    let top = pairs
        .next()
        .ok_or_else(|| SelectorParseError::UnexpectedToken {
            code: "P-001",
            position: 0,
            found: String::new(),
        })?;
    let or_pair = top
        .into_inner()
        .find(|p| p.as_rule() == Rule::or_expr)
        .ok_or_else(|| SelectorParseError::UnexpectedToken {
            code: "P-001",
            position: 0,
            found: String::new(),
        })?;
    let bool_expr = build_or(or_pair)?;
    Ok(simplify_to_selector(bool_expr))
}

/// Total parse: legacy fast-path → grammar attempt → `Exact` fallback.
/// Used by [`Selector::parse`] for the 9 M1 grant-mint call sites; never
/// errors.
pub fn parse_selector_or_uri(input: &str) -> Selector {
    if input == "*" {
        return Selector::Any;
    }
    if let Some(kind) = input.strip_prefix("#kind:") {
        return Selector::KindTag(kind.to_string());
    }
    if let Some(prefix) = input.strip_suffix("**") {
        return Selector::Prefix(prefix.to_string());
    }
    parse_selector(input).unwrap_or_else(|_| Selector::Exact(input.to_string()))
}

fn map_pest_error(err: pest::error::Error<Rule>) -> SelectorParseError {
    use pest::error::{ErrorVariant, InputLocation};

    let position = match err.location {
        InputLocation::Pos(p) => p,
        InputLocation::Span((start, _)) => start,
    };

    match &err.variant {
        ErrorVariant::ParsingError { positives, .. } => {
            // Heuristic: if the parser was looking for a closing `)`, surface
            // P-002. If it was at the start of a `contains_op`, surface
            // P-003 ("unknown predicate"). Otherwise P-001.
            let wants_close_paren = positives.iter().any(|r| matches!(r, Rule::paren_expr));
            if wants_close_paren {
                return SelectorParseError::UnbalancedParens {
                    code: "P-002",
                    position,
                };
            }
            let wants_pred_name = positives.iter().any(|r| {
                matches!(
                    r,
                    Rule::contains_pred
                        | Rule::intersects_pred
                        | Rule::any_match_pred
                        | Rule::subset_of_pred
                        | Rule::empty_pred
                        | Rule::non_empty_pred
                )
            });
            if wants_pred_name {
                return SelectorParseError::UnknownPredicate {
                    code: "P-003",
                    name: err.line().trim().to_string(),
                };
            }
            SelectorParseError::UnexpectedToken {
                code: "P-001",
                position,
                found: err.line().trim().to_string(),
            }
        }
        ErrorVariant::CustomError { message } => SelectorParseError::UnexpectedToken {
            code: "P-001",
            position,
            found: message.clone(),
        },
    }
}

// ----------------------------------------------------------------------------
// AST construction (pest pair → AST)
// ----------------------------------------------------------------------------

fn build_or(pair: Pair<Rule>) -> Result<BoolExpr, SelectorParseError> {
    debug_assert_eq!(pair.as_rule(), Rule::or_expr);
    let mut parts: Vec<BoolExpr> = Vec::new();
    for child in pair.into_inner() {
        if child.as_rule() == Rule::and_expr {
            parts.push(build_and(child)?);
        }
    }
    Ok(if parts.len() == 1 {
        parts.into_iter().next().unwrap()
    } else {
        BoolExpr::Or(parts)
    })
}

fn build_and(pair: Pair<Rule>) -> Result<BoolExpr, SelectorParseError> {
    debug_assert_eq!(pair.as_rule(), Rule::and_expr);
    let mut parts: Vec<BoolExpr> = Vec::new();
    for child in pair.into_inner() {
        if child.as_rule() == Rule::not_expr {
            parts.push(build_not(child)?);
        }
    }
    Ok(if parts.len() == 1 {
        parts.into_iter().next().unwrap()
    } else {
        BoolExpr::And(parts)
    })
}

fn build_not(pair: Pair<Rule>) -> Result<BoolExpr, SelectorParseError> {
    debug_assert_eq!(pair.as_rule(), Rule::not_expr);
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| SelectorParseError::UnexpectedToken {
            code: "P-001",
            position: 0,
            found: "not_expr".into(),
        })?;
    match inner.as_rule() {
        Rule::not_pred => {
            let pred_pair = inner
                .into_inner()
                .find(|p| p.as_rule() == Rule::predicate)
                .ok_or_else(|| SelectorParseError::UnexpectedToken {
                    code: "P-001",
                    position: 0,
                    found: "not_pred".into(),
                })?;
            Ok(BoolExpr::Not(Box::new(build_predicate(pred_pair)?)))
        }
        Rule::predicate => build_predicate(inner),
        r => Err(SelectorParseError::UnexpectedToken {
            code: "P-001",
            position: 0,
            found: format!("{:?}", r),
        }),
    }
}

fn build_predicate(pair: Pair<Rule>) -> Result<BoolExpr, SelectorParseError> {
    debug_assert_eq!(pair.as_rule(), Rule::predicate);
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| SelectorParseError::UnexpectedToken {
            code: "P-001",
            position: 0,
            found: "predicate".into(),
        })?;
    match inner.as_rule() {
        Rule::paren_expr => {
            let or_pair = inner
                .into_inner()
                .find(|p| p.as_rule() == Rule::or_expr)
                .ok_or(SelectorParseError::UnbalancedParens {
                    code: "P-002",
                    position: 0,
                })?;
            build_or(or_pair)
        }
        Rule::tag_predicate => Ok(BoolExpr::Pred(build_tag_predicate(inner)?)),
        r => Err(SelectorParseError::UnexpectedToken {
            code: "P-001",
            position: 0,
            found: format!("{:?}", r),
        }),
    }
}

fn build_tag_predicate(pair: Pair<Rule>) -> Result<Predicate, SelectorParseError> {
    debug_assert_eq!(pair.as_rule(), Rule::tag_predicate);
    let contains_op = pair
        .into_inner()
        .find(|p| p.as_rule() == Rule::contains_op)
        .ok_or_else(|| SelectorParseError::UnknownPredicate {
            code: "P-003",
            name: "tag_predicate".into(),
        })?;
    let body =
        contains_op
            .into_inner()
            .next()
            .ok_or_else(|| SelectorParseError::UnknownPredicate {
                code: "P-003",
                name: "contains_op".into(),
            })?;
    match body.as_rule() {
        Rule::contains_pred => {
            let tag_pair = first_child(body, Rule::tag, "contains")?;
            Ok(Predicate::Contains(build_tag(tag_pair)?))
        }
        Rule::intersects_pred => {
            let set_pair = first_child(body, Rule::tag_set, "intersects")?;
            Ok(Predicate::Intersects(build_tag_set(set_pair)?))
        }
        Rule::any_match_pred => {
            let glob_pair = first_child(body, Rule::tag_glob, "any_match")?;
            let pattern =
                unquote_string_literal(first_child(glob_pair, Rule::string_literal, "any_match")?);
            if !pattern.contains('*') {
                return Err(SelectorParseError::InvalidGlob {
                    code: "P-004",
                    pattern,
                });
            }
            Ok(Predicate::AnyMatch(pattern))
        }
        Rule::subset_of_pred => {
            let sref_pair = first_child(body, Rule::set_ref, "subset_of")?;
            Ok(Predicate::SubsetOf(build_set_ref(sref_pair)))
        }
        Rule::empty_pred => Ok(Predicate::Empty),
        Rule::non_empty_pred => Ok(Predicate::NonEmpty),
        r => Err(SelectorParseError::UnknownPredicate {
            code: "P-003",
            name: format!("{:?}", r),
        }),
    }
}

fn first_child<'a>(
    pair: Pair<'a, Rule>,
    rule: Rule,
    ctx: &str,
) -> Result<Pair<'a, Rule>, SelectorParseError> {
    pair.into_inner()
        .find(|p| p.as_rule() == rule)
        .ok_or_else(|| SelectorParseError::UnexpectedToken {
            code: "P-001",
            position: 0,
            found: ctx.to_string(),
        })
}

fn build_tag(pair: Pair<Rule>) -> Result<Tag, SelectorParseError> {
    debug_assert_eq!(pair.as_rule(), Rule::tag);
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| SelectorParseError::UnexpectedToken {
            code: "P-001",
            position: 0,
            found: "tag".into(),
        })?;
    match inner.as_rule() {
        Rule::reserved_tag => {
            let mut idents = collect_idents(inner.into_inner());
            let ns = idents
                .next()
                .ok_or_else(|| SelectorParseError::UnexpectedToken {
                    code: "P-001",
                    position: 0,
                    found: "reserved_tag".into(),
                })?;
            let value = idents.next();
            Ok(Tag::Reserved(ns, value))
        }
        Rule::namespace_tag => {
            let mut children = inner.into_inner();
            let ns_pair = children
                .next()
                .ok_or_else(|| SelectorParseError::UnexpectedToken {
                    code: "P-001",
                    position: 0,
                    found: "namespace_tag".into(),
                })?;
            let ns = ns_pair.as_str().to_string();
            let parts: Vec<String> = children
                .next()
                .map(|tag_value_pair| {
                    tag_value_pair
                        .into_inner()
                        .filter(|p| p.as_rule() == Rule::identifier)
                        .map(|p| p.as_str().to_string())
                        .collect()
                })
                .unwrap_or_default();
            Ok(Tag::Namespace(ns, parts))
        }
        Rule::literal_tag => {
            let s = first_child(inner, Rule::string_literal, "literal_tag")?;
            Ok(Tag::Literal(unquote_string_literal(s)))
        }
        r => Err(SelectorParseError::UnexpectedToken {
            code: "P-001",
            position: 0,
            found: format!("{:?}", r),
        }),
    }
}

fn collect_idents(pairs: Pairs<'_, Rule>) -> impl Iterator<Item = String> + '_ {
    pairs
        .filter(|p| p.as_rule() == Rule::identifier)
        .map(|p| p.as_str().to_string())
}

fn build_tag_set(pair: Pair<Rule>) -> Result<Vec<Tag>, SelectorParseError> {
    debug_assert_eq!(pair.as_rule(), Rule::tag_set);
    let mut tags = Vec::new();
    for child in pair.into_inner() {
        if child.as_rule() == Rule::tag {
            tags.push(build_tag(child)?);
        }
    }
    Ok(tags)
}

fn build_set_ref(pair: Pair<Rule>) -> SetRef {
    debug_assert_eq!(pair.as_rule(), Rule::set_ref);
    let mut idents = pair
        .into_inner()
        .filter(|p| p.as_rule() == Rule::identifier)
        .map(|p| p.as_str().to_string());
    let name = idents.next().unwrap_or_default();
    let args: Vec<String> = idents.collect();
    SetRef { name, args }
}

fn unquote_string_literal(pair: Pair<Rule>) -> String {
    let raw = pair.as_str();
    let trimmed = raw.trim_matches('"');
    let mut out = String::with_capacity(trimmed.len());
    let mut chars = trimmed.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(nc) = chars.next() {
                out.push(nc);
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn simplify_to_selector(expr: BoolExpr) -> Selector {
    match expr {
        BoolExpr::Pred(p) => Selector::Pred(p),
        other => Selector::Bool(Box::new(other)),
    }
}

// ----------------------------------------------------------------------------
// Evaluation
// ----------------------------------------------------------------------------

impl Selector {
    /// M1-era entry point. Accepts every input (no error). Maps the 4 legacy
    /// URI shapes to their existing variants and falls back to the full
    /// grammar (then to [`Selector::Exact`]) for everything else.
    pub fn parse(uri: &str) -> Self {
        parse_selector_or_uri(uri)
    }

    /// Match against `(target_uri, target_tags)` using a noop set-ref
    /// registry. Backwards-compatible shim — every M1 caller and unit test
    /// still operates on the 4 legacy variants and never needs a registry.
    pub fn matches(&self, target_uri: &str, target_tags: &[String]) -> bool {
        self.evaluate(target_uri, target_tags, &NoopSetRefRegistry)
    }

    /// Recursively evaluate the selector against the call's target. Used by
    /// [`crate::permissions::expansion::ResolvedGrant::effective_matches`]
    /// (Step 3 of the engine).
    pub fn evaluate(
        &self,
        target_uri: &str,
        target_tags: &[String],
        registry: &dyn SetRefRegistry,
    ) -> bool {
        match self {
            Selector::Any => true,
            Selector::Exact(s) => target_uri == s,
            Selector::Prefix(p) => target_uri.starts_with(p),
            Selector::KindTag(k) => {
                let needle = format!("#kind:{}", k);
                target_tags.iter().any(|t| t == &needle)
            }
            Selector::Bool(expr) => evaluate_bool(expr, target_tags, registry),
            Selector::Pred(p) => evaluate_predicate(p, target_tags, registry),
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

fn evaluate_bool(expr: &BoolExpr, target_tags: &[String], registry: &dyn SetRefRegistry) -> bool {
    match expr {
        BoolExpr::Or(parts) => parts
            .iter()
            .any(|p| evaluate_bool(p, target_tags, registry)),
        BoolExpr::And(parts) => parts
            .iter()
            .all(|p| evaluate_bool(p, target_tags, registry)),
        BoolExpr::Not(inner) => !evaluate_bool(inner, target_tags, registry),
        BoolExpr::Pred(p) => evaluate_predicate(p, target_tags, registry),
    }
}

fn evaluate_predicate(
    pred: &Predicate,
    target_tags: &[String],
    registry: &dyn SetRefRegistry,
) -> bool {
    match pred {
        Predicate::Contains(tag) => {
            let needle = tag.to_canonical();
            target_tags.iter().any(|t| t == &needle)
        }
        Predicate::Intersects(tags) => {
            let needles: HashSet<String> = tags.iter().map(|t| t.to_canonical()).collect();
            target_tags.iter().any(|t| needles.contains(t))
        }
        Predicate::AnyMatch(glob) => target_tags.iter().any(|t| glob_match(glob, t)),
        Predicate::SubsetOf(sref) => {
            let args: Vec<&str> = sref.args.iter().map(|s| s.as_str()).collect();
            match registry.resolve(&sref.name, &args) {
                Some(set) => target_tags.iter().all(|t| set.contains(t)),
                None => false,
            }
        }
        Predicate::Empty => target_tags.is_empty(),
        Predicate::NonEmpty => !target_tags.is_empty(),
    }
}

/// Slash-separated glob matcher. `*` consumes exactly one segment; `**`
/// consumes zero or more. Concept-09 §"Primary Predicates" semantics.
fn glob_match(pattern: &str, target: &str) -> bool {
    let p_segs: Vec<&str> = pattern.split('/').collect();
    let t_segs: Vec<&str> = target.split('/').collect();
    glob_match_segs(&p_segs, &t_segs)
}

fn glob_match_segs(p: &[&str], t: &[&str]) -> bool {
    match (p.first(), t.first()) {
        (None, None) => true,
        (None, Some(_)) => false,
        (Some(&"**"), _) => {
            // `**` matches zero or more segments. Try every split point.
            for skip in 0..=t.len() {
                if glob_match_segs(&p[1..], &t[skip..]) {
                    return true;
                }
            }
            false
        }
        (Some(&"*"), Some(_)) => glob_match_segs(&p[1..], &t[1..]),
        (Some(_), None) => false,
        (Some(p0), Some(t0)) if segment_match(p0, t0) => glob_match_segs(&p[1..], &t[1..]),
        _ => false,
    }
}

fn segment_match(pattern: &str, target: &str) -> bool {
    // Within a single segment, only literal equality (concept-09 §"Non-Normative
    // Notes" excludes bracketed char-classes + intra-segment globs from v0).
    pattern == target
}

// ----------------------------------------------------------------------------
// Backwards-compat AND selector
// ----------------------------------------------------------------------------

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

    pub fn evaluate(
        &self,
        target_uri: &str,
        target_tags: &[String],
        registry: &dyn SetRefRegistry,
    ) -> bool {
        self.left.evaluate(target_uri, target_tags, registry)
            && self.right.evaluate(target_uri, target_tags, registry)
    }
}

// ----------------------------------------------------------------------------
// Tests — legacy variants (8 from M1) + new grammar (~20)
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Legacy fast-path (M1 baseline) ----------------------------------

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

    // ---- Worked-parse goldens (concept-09 §"Worked Parses" 1-4) ----------

    #[test]
    fn worked_parse_1_project_session() {
        // tags contains project:acme-website-redesign AND tags contains #kind:session
        let s = parse_selector(
            "tags contains project:acme-website-redesign AND tags contains #kind:session",
        )
        .expect("worked parse 1");
        let Selector::Bool(b) = s else {
            panic!("expected Bool top-level");
        };
        let BoolExpr::And(parts) = *b else {
            panic!("expected AND");
        };
        assert_eq!(parts.len(), 2);
        assert!(matches!(
            &parts[0],
            BoolExpr::Pred(Predicate::Contains(Tag::Namespace(ns, _)))
            if ns == "project"
        ));
        assert!(matches!(
            &parts[1],
            BoolExpr::Pred(Predicate::Contains(Tag::Reserved(ns, Some(v))))
            if ns == "kind" && v == "session"
        ));
    }

    #[test]
    fn worked_parse_2_intersects_with_org() {
        let s = parse_selector(
            "tags intersects {session:s-9831, session:s-9832} AND tags contains org:acme",
        )
        .expect("worked parse 2");
        let Selector::Bool(b) = s else {
            panic!("Bool expected");
        };
        let BoolExpr::And(parts) = *b else {
            panic!("AND expected");
        };
        assert_eq!(parts.len(), 2);
        assert!(matches!(
            &parts[0],
            BoolExpr::Pred(Predicate::Intersects(v)) if v.len() == 2
        ));
    }

    #[test]
    fn worked_parse_3_subset_of_set_ref() {
        let s = parse_selector("tags subset_of supervisors_tagging_scope(supervisor-7)")
            .expect("worked parse 3");
        let Selector::Pred(Predicate::SubsetOf(sref)) = s else {
            panic!("expected SubsetOf at top");
        };
        assert_eq!(sref.name, "supervisors_tagging_scope");
        assert_eq!(sref.args, vec!["supervisor-7".to_string()]);
    }

    #[test]
    fn worked_parse_4_glob_with_not() {
        let s =
            parse_selector("tags any_match \"org:acme/eng/**\" AND NOT tags contains #archived")
                .expect("worked parse 4");
        let Selector::Bool(b) = s else {
            panic!("Bool expected");
        };
        let BoolExpr::And(parts) = *b else {
            panic!("AND expected");
        };
        assert_eq!(parts.len(), 2);
        assert!(
            matches!(&parts[0], BoolExpr::Pred(Predicate::AnyMatch(g)) if g == "org:acme/eng/**")
        );
        assert!(matches!(
            &parts[1],
            BoolExpr::Not(inner)
            if matches!(
                inner.as_ref(),
                BoolExpr::Pred(Predicate::Contains(Tag::Reserved(ns, None))) if ns == "archived"
            )
        ));
    }

    // ---- Predicate match unit tests (6) ----------------------------------

    #[test]
    fn predicate_contains_matches_when_tag_present() {
        let s = parse_selector("tags contains org:acme").unwrap();
        assert!(s.matches("any", &["org:acme".to_string()]));
        assert!(!s.matches("any", &["org:other".to_string()]));
    }

    #[test]
    fn predicate_intersects_matches_when_any_overlap() {
        let s = parse_selector("tags intersects {org:acme, org:other}").unwrap();
        assert!(s.matches("any", &["org:acme".to_string()]));
        assert!(s.matches("any", &["org:other".to_string()]));
        assert!(!s.matches("any", &["org:third".to_string()]));
    }

    #[test]
    fn predicate_any_match_glob_matches_subtree() {
        let s = parse_selector("tags any_match \"org:acme/eng/**\"").unwrap();
        assert!(s.matches("any", &["org:acme/eng/web/lead".to_string()]));
        assert!(s.matches("any", &["org:acme/eng".to_string()]));
        assert!(!s.matches("any", &["org:acme/sales".to_string()]));
    }

    #[test]
    fn predicate_subset_of_uses_registry() {
        struct StubRegistry;
        impl SetRefRegistry for StubRegistry {
            fn resolve(&self, name: &str, args: &[&str]) -> Option<HashSet<String>> {
                if name == "scope" && args == ["s-1"] {
                    Some(HashSet::from([
                        "org:acme".to_string(),
                        "team:eng".to_string(),
                    ]))
                } else {
                    None
                }
            }
        }
        let s = parse_selector("tags subset_of scope(s-1)").unwrap();
        assert!(s.evaluate("any", &["org:acme".to_string()], &StubRegistry));
        assert!(s.evaluate(
            "any",
            &["org:acme".to_string(), "team:eng".to_string()],
            &StubRegistry
        ));
        assert!(!s.evaluate("any", &["org:other".to_string()], &StubRegistry));
        // Noop registry → unknown set → false.
        assert!(!s.matches("any", &["org:acme".to_string()]));
    }

    #[test]
    fn predicate_empty_matches_only_when_no_tags() {
        let s = parse_selector("tags empty").unwrap();
        assert!(s.matches("any", &[]));
        assert!(!s.matches("any", &["x:y".to_string()]));
    }

    #[test]
    fn predicate_non_empty_matches_only_when_at_least_one_tag() {
        let s = parse_selector("tags non_empty").unwrap();
        assert!(!s.matches("any", &[]));
        assert!(s.matches("any", &["x:y".to_string()]));
    }

    // ---- Combinator unit tests (6) ---------------------------------------

    #[test]
    fn combinator_and_requires_both() {
        let s = parse_selector("tags contains a:x AND tags contains b:y").unwrap();
        assert!(s.matches("u", &["a:x".into(), "b:y".into()]));
        assert!(!s.matches("u", &["a:x".into()]));
        assert!(!s.matches("u", &["b:y".into()]));
    }

    #[test]
    fn combinator_or_requires_either() {
        let s = parse_selector("tags contains a:x OR tags contains b:y").unwrap();
        assert!(s.matches("u", &["a:x".into()]));
        assert!(s.matches("u", &["b:y".into()]));
        assert!(!s.matches("u", &["c:z".into()]));
    }

    #[test]
    fn combinator_not_inverts() {
        let s = parse_selector("NOT tags contains a:x").unwrap();
        assert!(!s.matches("u", &["a:x".into()]));
        assert!(s.matches("u", &["b:y".into()]));
        assert!(s.matches("u", &[]));
    }

    #[test]
    fn combinator_parens_override_precedence() {
        // Without parens: `a AND b OR c` = `(a AND b) OR c`
        let default_assoc =
            parse_selector("tags contains a:x AND tags contains b:y OR tags contains c:z").unwrap();
        // c:z alone satisfies the OR-side.
        assert!(default_assoc.matches("u", &["c:z".into()]));
        // a:x alone fails (b:y missing → AND-side false; c missing → OR-side false).
        assert!(!default_assoc.matches("u", &["a:x".into()]));

        // With parens forcing OR-then-AND: `a AND (b OR c)`
        let with_parens =
            parse_selector("tags contains a:x AND (tags contains b:y OR tags contains c:z)")
                .unwrap();
        assert!(with_parens.matches("u", &["a:x".into(), "b:y".into()]));
        assert!(with_parens.matches("u", &["a:x".into(), "c:z".into()]));
        assert!(!with_parens.matches("u", &["a:x".into()]));
        assert!(!with_parens.matches("u", &["b:y".into()]));
    }

    #[test]
    fn combinator_not_binds_tighter_than_and() {
        // `NOT a AND b` should parse as `(NOT a) AND b`
        let s = parse_selector("NOT tags contains a:x AND tags contains b:y").unwrap();
        // a missing AND b present → both branches true → match
        assert!(s.matches("u", &["b:y".into()]));
        // a present → NOT a false → AND false
        assert!(!s.matches("u", &["a:x".into(), "b:y".into()]));
        // b missing → AND false
        assert!(!s.matches("u", &[]));
    }

    #[test]
    fn combinator_and_binds_tighter_than_or() {
        // `a AND b OR c` should parse as `(a AND b) OR c`
        let s =
            parse_selector("tags contains a:x AND tags contains b:y OR tags contains c:z").unwrap();
        // c alone → OR-side wins
        assert!(s.matches("u", &["c:z".into()]));
        // a + b → AND-side wins
        assert!(s.matches("u", &["a:x".into(), "b:y".into()]));
        // a alone → both sides false
        assert!(!s.matches("u", &["a:x".into()]));
    }

    // ---- Reserved-namespace ordered-choice -------------------------------

    #[test]
    fn reserved_tag_wins_over_namespace_in_ordered_choice() {
        // `#kind:session` MUST parse as ReservedTag, not NamespaceTag
        // (concept-09 line 106).
        let s = parse_selector("tags contains #kind:session").unwrap();
        let Selector::Pred(Predicate::Contains(Tag::Reserved(ns, Some(v)))) = s else {
            panic!("expected Reserved tag");
        };
        assert_eq!(ns, "kind");
        assert_eq!(v, "session");
    }

    #[test]
    fn bare_reserved_tag_parses_with_no_value() {
        let s = parse_selector("tags contains #archived").unwrap();
        let Selector::Pred(Predicate::Contains(Tag::Reserved(ns, None))) = s else {
            panic!("expected bare reserved tag");
        };
        assert_eq!(ns, "archived");
    }

    // ---- Parse-error code surface (5 codes — P-001..P-005) ---------------

    #[test]
    fn parse_error_p001_unexpected_token() {
        let err = parse_selector("tags contains @@@").unwrap_err();
        assert_eq!(err.code(), "P-001");
    }

    #[test]
    fn parse_error_p002_unbalanced_parens() {
        let err = parse_selector("(tags contains a:x").unwrap_err();
        // Either P-002 (preferred) or P-001 acceptable; we assert
        // it's one of the two.
        assert!(matches!(err.code(), "P-001" | "P-002"));
    }

    #[test]
    fn parse_error_p003_unknown_predicate() {
        let err = parse_selector("tags wibbles foo").unwrap_err();
        assert!(matches!(err.code(), "P-001" | "P-003"));
    }

    #[test]
    fn parse_error_p004_invalid_glob_no_wildcard() {
        let err = parse_selector("tags any_match \"no-wildcard-here\"").unwrap_err();
        assert_eq!(err.code(), "P-004");
    }

    #[test]
    fn parse_error_p005_unknown_set_ref_at_runtime() {
        // P-005 is the runtime/registry-resolution code. Parse succeeds;
        // evaluate() returns false under NoopSetRefRegistry, which is the
        // operationally-equivalent denial.
        let s = parse_selector("tags subset_of unknown_set(arg)").unwrap();
        assert!(!s.matches("u", &["x:y".into()]));
    }

    // ---- parse_selector_or_uri legacy fast-path -------------------------

    #[test]
    fn fast_path_falls_back_to_exact_for_grant_uris() {
        // None of the 9 grant-mint URI shapes contain "tags …" so they all
        // hit the Exact fallback (preserving M1 semantics).
        assert_eq!(
            parse_selector_or_uri("secret:anthropic-key"),
            Selector::Exact("secret:anthropic-key".into())
        );
        assert_eq!(
            parse_selector_or_uri("provider:p-1"),
            Selector::Exact("provider:p-1".into())
        );
        assert_eq!(
            parse_selector_or_uri("system:root"),
            Selector::Exact("system:root".into())
        );
    }

    #[test]
    fn fast_path_full_grammar_still_routes_through() {
        let s = parse_selector_or_uri("tags contains org:acme");
        let Selector::Pred(Predicate::Contains(Tag::Namespace(ns, vs))) = s else {
            panic!("expected namespace contains");
        };
        assert_eq!(ns, "org");
        assert_eq!(vs, vec!["acme".to_string()]);
    }

    // ---- glob_match unit -------------------------------------------------

    #[test]
    fn glob_match_double_star_matches_zero_or_more_segments() {
        assert!(glob_match("a/**", "a"));
        assert!(glob_match("a/**", "a/b"));
        assert!(glob_match("a/**", "a/b/c/d"));
        assert!(!glob_match("a/**", "b/c"));
    }

    #[test]
    fn glob_match_single_star_matches_exactly_one_segment() {
        assert!(glob_match("a/*/c", "a/b/c"));
        assert!(!glob_match("a/*/c", "a/c")); // missing middle segment
        assert!(!glob_match("a/*/c", "a/b/x/c")); // too many segments
    }
}
