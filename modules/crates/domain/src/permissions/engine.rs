//! The Permission Check engine — [`check`] plus the six explicit step
//! helpers.
//!
//! The engine is **pure** — no I/O, no async, no panics. Every input comes
//! from [`CheckContext`]; every output is a [`Decision`]. This makes the
//! engine unit-testable with simple in-memory fixtures and
//! proptest-friendly for the ≥5 proptest invariant files that gate M1/P3.
//!
//! Step-by-step the pipeline mirrors
//! `docs/specs/v0/concepts/permissions/04-manifest-and-resolution.md`
//! §Formal Algorithm. Source ordering below defers Step 4 until after
//! Step 5 per the concept doc ("Constraints are checked AFTER scope
//! resolution picks the winning grant in Step 5").

use std::collections::HashMap;
use std::time::Instant;

use tracing::instrument;

use crate::model::ids::{AgentId, GrantId, OrgId, ProjectId};
use crate::model::nodes::{Grant, PrincipalRef};
use crate::model::Fundamental;

use super::decision::{AwaitingConsent, Decision, DeniedReason, FailedStep, ResolvedReach};
use super::expansion::{resolve_grant, ResolvedGrant};
use super::manifest::{CheckContext, Manifest};
use super::metrics::PermissionCheckMetrics;

// ----------------------------------------------------------------------------
// Public entry point
// ----------------------------------------------------------------------------

/// Run the 6-step (+2a) Permission Check over `ctx` / `manifest`.
///
/// Records one sample to `metrics` before returning — see
/// `PermissionCheckMetrics`.
#[instrument(level = "debug", skip_all, fields(agent = ?ctx.agent))]
pub fn check(
    ctx: &CheckContext<'_>,
    manifest: &Manifest,
    metrics: &dyn PermissionCheckMetrics,
) -> Decision {
    let start = Instant::now();
    let decision = check_inner(ctx, manifest);
    let elapsed = start.elapsed();
    let result_label = decision.metric_result_label();
    let failed_step_owned = decision.failed_step().map(|s| s.as_metric_label());
    metrics.record(elapsed, result_label, failed_step_owned);
    decision
}

fn check_inner(ctx: &CheckContext<'_>, manifest: &Manifest) -> Decision {
    // --- Step 0 ----------------------------------------------------------
    if let Some(reason) = step_0_catalogue(ctx) {
        return Decision::Denied {
            failed_step: FailedStep::Catalogue,
            reason,
        };
    }

    // --- Step 1 ----------------------------------------------------------
    let required_reaches = step_1_expand_manifest(manifest);
    if required_reaches.is_empty() {
        return Decision::Denied {
            failed_step: FailedStep::Expansion,
            reason: DeniedReason::ManifestEmpty,
        };
    }

    // --- Step 2 ----------------------------------------------------------
    let candidates = step_2_resolve_grants(ctx);
    if candidates.is_empty() {
        return Decision::Denied {
            failed_step: FailedStep::Resolution,
            reason: DeniedReason::NoGrantsHeld,
        };
    }

    // --- Step 2a ---------------------------------------------------------
    let effective = step_2a_ceiling(candidates, ctx.ceiling_grants);
    if effective.is_empty() {
        return Decision::Denied {
            failed_step: FailedStep::Ceiling,
            reason: DeniedReason::CeilingEmptied,
        };
    }

    // --- Step 3 ----------------------------------------------------------
    let matches = match step_3_match_reaches(&required_reaches, &effective, ctx) {
        Ok(m) => m,
        Err(d) => return d,
    };

    // --- Step 5 (picks winner per reach) ---------------------------------
    let resolved = match step_5_scope_resolution(matches, ctx) {
        Ok(r) => r,
        Err(d) => return d,
    };

    // --- Step 4 (constraints vs winner) ----------------------------------
    if let Some(d) = step_4_constraints(&resolved, manifest, ctx) {
        return d;
    }

    // --- Step 6 ----------------------------------------------------------
    if let Some(d) = step_6_consent_gating(&resolved, ctx) {
        return d;
    }

    // All green.
    Decision::Allowed {
        resolved_grants: resolved
            .into_iter()
            .map(|(key, grant)| ResolvedReach {
                fundamental: key.0,
                action: key.1,
                grant_id: grant.grant.id,
            })
            .collect(),
    }
}

// ----------------------------------------------------------------------------
// Step 0 — Catalogue precondition
// ----------------------------------------------------------------------------

/// If the call names a specific target, it must be declared in the owning
/// org's catalogue. Class-level invocations (empty `target_uri`) skip Step
/// 0 — they reference no specific instance.
pub fn step_0_catalogue(ctx: &CheckContext<'_>) -> Option<DeniedReason> {
    if ctx.call.target_uri.is_empty() {
        return None;
    }
    if ctx
        .catalogue
        .contains(ctx.current_org, &ctx.call.target_uri)
    {
        None
    } else {
        Some(DeniedReason::CatalogueMiss {
            resource_uri: ctx.call.target_uri.clone(),
        })
    }
}

// ----------------------------------------------------------------------------
// Step 1 — Manifest expansion
// ----------------------------------------------------------------------------

/// Expand the manifest into `(fundamental, action)` reaches. Composite
/// names are expanded to their constituent fundamentals; unknown names are
/// dropped (they contribute no reaches).
pub fn step_1_expand_manifest(manifest: &Manifest) -> Vec<(Fundamental, String)> {
    let mut reaches: Vec<(Fundamental, String)> = Vec::new();

    let fundamentals_union: std::collections::HashSet<Fundamental> = manifest
        .resource
        .iter()
        .chain(manifest.transitive.iter())
        .flat_map(|name| super::expansion::expand_resource_to_fundamentals(name))
        .collect();

    for fundamental in fundamentals_union {
        for action in &manifest.actions {
            reaches.push((fundamental, action.clone()));
        }
    }
    reaches.sort_by(|a, b| (a.0 as u8, &a.1).cmp(&(b.0 as u8, &b.1)));
    reaches.dedup();
    reaches
}

// ----------------------------------------------------------------------------
// Step 2 — Resolve candidate grants
// ----------------------------------------------------------------------------

/// Where a candidate grant was picked up. Drives Step 5's most-specific-
/// first cascade (Agent → Project → Org).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScopeTier {
    /// Grant held directly by the agent. Most specific.
    Agent,
    /// Grant propagated from the agent's current project.
    Project,
    /// Grant propagated from the agent's current org.
    Organization,
}

/// A candidate grant + where it came from.
#[derive(Debug, Clone)]
pub struct Candidate {
    pub resolved: ResolvedGrant,
    pub tier: ScopeTier,
}

/// Collect all non-revoked grants from agent/project/org, resolve each, and
/// tag it with its scope tier.
pub fn step_2_resolve_grants(ctx: &CheckContext<'_>) -> Vec<Candidate> {
    fn collect(grants: &[Grant], tier: ScopeTier, out: &mut Vec<Candidate>) {
        for g in grants {
            if g.revoked_at.is_some() {
                continue;
            }
            out.push(Candidate {
                resolved: resolve_grant(g),
                tier,
            });
        }
    }
    let mut out = Vec::with_capacity(
        ctx.agent_grants.len() + ctx.project_grants.len() + ctx.org_grants.len(),
    );
    collect(ctx.agent_grants, ScopeTier::Agent, &mut out);
    collect(ctx.project_grants, ScopeTier::Project, &mut out);
    collect(ctx.org_grants, ScopeTier::Organization, &mut out);
    out
}

// ----------------------------------------------------------------------------
// Step 2a — Ceiling enforcement
// ----------------------------------------------------------------------------

/// Clamp each candidate against the set of ceiling grants. For M1 we
/// implement "clamp" as "keep iff some ceiling admits this candidate" —
/// strictly shrinking. Empty ceilings → no clamping (infinite ceiling).
///
/// A ceiling admits a candidate iff:
///
/// 1. `candidate.fundamentals ⊆ ceiling.fundamentals`,
/// 2. every action on the candidate grant is listed on the ceiling grant
///    (or the ceiling lists `*`).
///
/// Selector subset is deferred to M2+ (requires lattice machinery we don't
/// need yet).
pub fn step_2a_ceiling(candidates: Vec<Candidate>, ceiling_grants: &[Grant]) -> Vec<Candidate> {
    if ceiling_grants.is_empty() {
        return candidates;
    }
    let ceilings: Vec<ResolvedGrant> = ceiling_grants
        .iter()
        .filter(|c| c.revoked_at.is_none())
        .map(resolve_grant)
        .collect();
    candidates
        .into_iter()
        .filter(|cand| ceilings.iter().any(|c| ceiling_admits(c, &cand.resolved)))
        .collect()
}

fn ceiling_admits(ceiling: &ResolvedGrant, candidate: &ResolvedGrant) -> bool {
    // Every action on the candidate must appear on the ceiling (or ceiling
    // has a `*` wildcard).
    let actions_ok = candidate
        .grant
        .action
        .iter()
        .all(|a| ceiling.grant.action.iter().any(|ca| ca == a || ca == "*"));
    if !actions_ok {
        return false;
    }
    // Candidate fundamentals must be a subset of ceiling fundamentals.
    candidate.fundamentals.is_subset(&ceiling.fundamentals)
}

// ----------------------------------------------------------------------------
// Step 3 — Match each required reach against ≥1 grant
// ----------------------------------------------------------------------------

type ReachKey = (Fundamental, String);

/// For each required reach, filter `effective` down to the grants whose
/// fundamentals + actions + selector cover the reach. Empty match set for
/// any reach → Denied.
pub fn step_3_match_reaches(
    required: &[ReachKey],
    effective: &[Candidate],
    ctx: &CheckContext<'_>,
) -> Result<HashMap<ReachKey, Vec<Candidate>>, Decision> {
    let mut out: HashMap<ReachKey, Vec<Candidate>> = HashMap::new();
    for (f, action) in required {
        let matching: Vec<Candidate> = effective
            .iter()
            .filter(|c| c.resolved.covers(*f, action))
            .filter(|c| {
                c.resolved
                    .effective_matches(&ctx.call.target_uri, &ctx.call.target_tags)
            })
            .cloned()
            .collect();
        if matching.is_empty() {
            return Err(Decision::Denied {
                failed_step: FailedStep::Match,
                reason: DeniedReason::NoMatchingGrant {
                    fundamental: *f,
                    action: action.clone(),
                },
            });
        }
        out.insert((*f, action.clone()), matching);
    }
    Ok(out)
}

// ----------------------------------------------------------------------------
// Step 5 — Scope resolution cascade
// ----------------------------------------------------------------------------

/// For each reach, pick the winning grant via the
/// most-specific-first cascade (Agent → Project → Org). Ties within a tier
/// are broken by [`tie_break_within_tier`].
pub fn step_5_scope_resolution(
    matches: HashMap<ReachKey, Vec<Candidate>>,
    _ctx: &CheckContext<'_>,
) -> Result<HashMap<ReachKey, ResolvedGrant>, Decision> {
    let mut winners = HashMap::with_capacity(matches.len());
    for (key, mut candidates) in matches {
        // Sort by tier ascending (Agent first).
        candidates.sort_by_key(|c| c.tier);
        let top_tier = candidates[0].tier;
        candidates.retain(|c| c.tier == top_tier);
        let winner = tie_break_within_tier(candidates).ok_or_else(|| Decision::Denied {
            failed_step: FailedStep::Scope,
            reason: DeniedReason::ScopeUnresolvable {
                fundamental: key.0,
                action: key.1.clone(),
            },
        })?;
        winners.insert(key, winner);
    }
    Ok(winners)
}

/// Within a tier, pick the most recently issued grant. Deterministic
/// tiebreak is required by the concept doc; `issued_at` + grant id bytes
/// gives a total order.
fn tie_break_within_tier(mut cands: Vec<Candidate>) -> Option<ResolvedGrant> {
    if cands.is_empty() {
        return None;
    }
    cands.sort_by(|a, b| {
        b.resolved
            .grant
            .issued_at
            .cmp(&a.resolved.grant.issued_at)
            .then_with(|| {
                b.resolved
                    .grant
                    .id
                    .as_uuid()
                    .as_bytes()
                    .cmp(a.resolved.grant.id.as_uuid().as_bytes())
            })
    });
    Some(cands.into_iter().next().unwrap().resolved)
}

// ----------------------------------------------------------------------------
// Step 4 — Constraint satisfaction
// ----------------------------------------------------------------------------

/// For each manifest-required constraint, the winning grant must be
/// callable with a value for that constraint.
///
/// v0 rule (M2): the invocation's `constraint_context` must carry a
/// value under each required constraint name (**presence check**).
/// Additionally, when `manifest.constraint_requirements` names a
/// required value for that constraint, the context value must be
/// **equal** to that required value (**value-match check**).
///
/// Examples:
/// - `constraints = ["purpose"]`, no requirement — passes if the
///   context has ANY `"purpose"` entry.
/// - `constraints = ["purpose"]` + `constraint_requirements["purpose"]
///   = json!("reveal")` — passes only if the context's `"purpose"`
///   equals `"reveal"` exactly.
///
/// Pattern matching / lattice ordering land in M3 when the constraint
/// lattice machinery arrives. String / number / boolean equality is
/// sufficient for M2's page-04 `purpose=reveal` contract (G3 in the
/// archived M2 plan).
pub fn step_4_constraints(
    resolved: &HashMap<ReachKey, ResolvedGrant>,
    manifest: &Manifest,
    ctx: &CheckContext<'_>,
) -> Option<Decision> {
    for required in &manifest.constraints {
        let Some(provided) = ctx.call.constraint_context.get(required) else {
            return Some(constraint_violation(resolved, required));
        };
        // When the manifest names a required *value*, check equality.
        // Missing from `constraint_requirements` means presence-only
        // (matching M1 behaviour).
        if let Some(expected) = manifest.constraint_requirements.get(required) {
            if provided != expected {
                return Some(constraint_violation(resolved, required));
            }
        }
    }
    None
}

/// Build the denial decision for a constraint violation, attributing it
/// to a deterministic winner (smallest sorted reach key) so tests and
/// logs stay stable across `HashMap` iteration-order runs.
fn constraint_violation(resolved: &HashMap<ReachKey, ResolvedGrant>, constraint: &str) -> Decision {
    let mut keys: Vec<_> = resolved.keys().cloned().collect();
    keys.sort_by(|a, b| (a.0 as u8, &a.1).cmp(&(b.0 as u8, &b.1)));
    // `resolved` is non-empty here: callers invoke this only after Step
    // 5 fills `resolved`, which itself is non-empty because Step 1
    // would otherwise have returned at ManifestEmpty.
    let k = keys
        .first()
        .expect("step_4 called with non-empty resolved map");
    let winner = &resolved[k];
    Decision::Denied {
        failed_step: FailedStep::Constraint,
        reason: DeniedReason::ConstraintViolation {
            constraint: constraint.to_string(),
            grant_id: winner.grant.id,
        },
    }
}

// ----------------------------------------------------------------------------
// Step 6 — Consent gating (Templates A–D)
// ----------------------------------------------------------------------------

/// If any winning grant was issued by a Template-A/B/C/D Auth Request, the
/// target subordinate must have an `Acknowledged` consent under the
/// issuing org. Missing consent → `Pending`, not `Denied`.
///
/// For M1, the caller injects the "template-gated" Auth Request ids via
/// [`CheckContext::template_gated_auth_requests`] — an empty set
/// effectively skips Step 6, which is the correct M1 behaviour because
/// bootstrap grants are not template-sourced. P4/M3 wires Template
/// provenance through.
pub fn step_6_consent_gating(
    resolved: &HashMap<ReachKey, ResolvedGrant>,
    ctx: &CheckContext<'_>,
) -> Option<Decision> {
    let target = ctx.call.target_agent?;
    let org = consent_org_for_ctx(ctx)?;
    for winner in resolved.values() {
        if !winner_requires_consent(winner, ctx) {
            continue;
        }
        if ctx.consents.is_acknowledged(target, org) {
            continue;
        }
        return Some(Decision::Pending {
            awaiting_consent: AwaitingConsent {
                subordinate: target,
                org,
            },
        });
    }
    None
}

/// Which org's policy governs consent for this check? For M1 the agent's
/// owning org.
fn consent_org_for_ctx(ctx: &CheckContext<'_>) -> Option<OrgId> {
    ctx.current_org
}

/// Is this winning grant template-gated for consent? The caller tells the
/// engine via `CheckContext::template_gated_auth_requests` — a hook that
/// stays empty in M1 and is populated in M3 when Templates A–D wire in.
fn winner_requires_consent(winner: &ResolvedGrant, ctx: &CheckContext<'_>) -> bool {
    match winner.grant.descends_from {
        Some(ar_id) => ctx.template_gated_auth_requests.contains(&ar_id),
        None => false,
    }
}

// ----------------------------------------------------------------------------
// Helpers re-exported for tests
// ----------------------------------------------------------------------------

/// Given a `PrincipalRef`, does the pair `(ctx.agent, ctx.current_org,
/// ctx.current_project)` legitimately hold a grant keyed on that principal?
/// Used by `step_2` (indirectly) and by the in-memory fake when it simulates
/// `list_grants_for_principal`.
pub fn holds_grant_for(
    principal: &PrincipalRef,
    agent: AgentId,
    org: Option<OrgId>,
    project: Option<ProjectId>,
) -> bool {
    match principal {
        PrincipalRef::Agent(a) => *a == agent,
        PrincipalRef::Organization(o) => org == Some(*o),
        PrincipalRef::Project(p) => project == Some(*p),
        _ => false,
    }
}

/// For tests: an `Allowed` decision's `resolved_grants` map as
/// `HashMap<(fundamental, action), GrantId>`.
pub fn allowed_map(d: &Decision) -> HashMap<(Fundamental, String), GrantId> {
    d.resolved_grants_map()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ids::{AgentId, GrantId};
    use crate::model::nodes::{PrincipalRef, ResourceRef};
    use crate::permissions::catalogue::StaticCatalogue;
    use crate::permissions::manifest::{ConsentIndex, ToolCall};
    use crate::permissions::metrics::NoopMetrics;
    use chrono::Utc;
    use std::collections::HashSet;

    // ---- Fixture helpers --------------------------------------------------

    fn grant(holder: PrincipalRef, actions: &[&str], resource_uri: &str) -> Grant {
        Grant {
            id: GrantId::new(),
            holder,
            action: actions.iter().map(|s| s.to_string()).collect(),
            resource: ResourceRef {
                uri: resource_uri.into(),
            },
            fundamentals: vec![],
            descends_from: None,
            delegable: false,
            issued_at: Utc::now(),
            revoked_at: None,
        }
    }

    struct Fixture {
        agent: AgentId,
        org: Option<OrgId>,
        project: Option<ProjectId>,
        agent_grants: Vec<Grant>,
        project_grants: Vec<Grant>,
        org_grants: Vec<Grant>,
        ceiling_grants: Vec<Grant>,
        catalogue: StaticCatalogue,
        consents: ConsentIndex,
        template_gated: HashSet<crate::model::ids::AuthRequestId>,
    }

    impl Fixture {
        fn new() -> Self {
            Self {
                agent: AgentId::new(),
                org: None,
                project: None,
                agent_grants: vec![],
                project_grants: vec![],
                org_grants: vec![],
                ceiling_grants: vec![],
                catalogue: StaticCatalogue::empty(),
                consents: ConsentIndex::empty(),
                template_gated: HashSet::new(),
            }
        }

        fn ctx<'a>(&'a self, call: ToolCall) -> CheckContext<'a> {
            CheckContext {
                agent: self.agent,
                current_org: self.org,
                current_project: self.project,
                agent_grants: &self.agent_grants,
                project_grants: &self.project_grants,
                org_grants: &self.org_grants,
                ceiling_grants: &self.ceiling_grants,
                catalogue: &self.catalogue,
                consents: &self.consents,
                template_gated_auth_requests: &self.template_gated,
                call,
            }
        }
    }

    fn manifest(actions: &[&str], resource: &[&str]) -> Manifest {
        Manifest {
            actions: actions.iter().map(|s| s.to_string()).collect(),
            resource: resource.iter().map(|s| s.to_string()).collect(),
            ..Default::default()
        }
    }

    // ---- Tests ------------------------------------------------------------

    #[test]
    fn step_0_skipped_for_class_level_call() {
        let f = Fixture::new();
        let ctx = f.ctx(ToolCall::default());
        assert!(step_0_catalogue(&ctx).is_none());
    }

    #[test]
    fn step_0_denies_when_target_not_in_catalogue() {
        let f = Fixture::new();
        let ctx = f.ctx(ToolCall {
            target_uri: "filesystem:/nope".into(),
            ..Default::default()
        });
        assert!(matches!(
            step_0_catalogue(&ctx),
            Some(DeniedReason::CatalogueMiss { .. })
        ));
    }

    #[test]
    fn step_0_passes_when_target_catalogued() {
        let mut f = Fixture::new();
        f.catalogue.seed(None, "system:root");
        let ctx = f.ctx(ToolCall {
            target_uri: "system:root".into(),
            ..Default::default()
        });
        assert!(step_0_catalogue(&ctx).is_none());
    }

    #[test]
    fn step_1_empty_for_empty_manifest() {
        assert!(step_1_expand_manifest(&Manifest::default()).is_empty());
    }

    #[test]
    fn step_1_expands_fundamental_and_composite() {
        let m = manifest(&["read"], &["filesystem_object", "memory_object"]);
        let reaches = step_1_expand_manifest(&m);
        // filesystem_object + memory_object (data_object + tag) = 3 fundamentals × 1 action.
        assert_eq!(reaches.len(), 3);
        assert!(reaches
            .iter()
            .any(|(f, a)| *f == Fundamental::FilesystemObject && a == "read"));
        assert!(reaches
            .iter()
            .any(|(f, a)| *f == Fundamental::DataObject && a == "read"));
        assert!(reaches
            .iter()
            .any(|(f, a)| *f == Fundamental::Tag && a == "read"));
    }

    #[test]
    fn engine_denies_at_step_0_when_catalogue_missing() {
        let f = Fixture::new();
        let ctx = f.ctx(ToolCall {
            target_uri: "filesystem:/missing".into(),
            ..Default::default()
        });
        let m = manifest(&["read"], &["filesystem_object"]);
        let d = check(&ctx, &m, &NoopMetrics);
        assert_eq!(d.failed_step(), Some(FailedStep::Catalogue));
    }

    #[test]
    fn engine_denies_at_step_1_when_manifest_empty() {
        let f = Fixture::new();
        let ctx = f.ctx(ToolCall::default());
        let d = check(&ctx, &Manifest::default(), &NoopMetrics);
        assert_eq!(d.failed_step(), Some(FailedStep::Expansion));
    }

    #[test]
    fn engine_denies_at_step_2_when_subject_holds_nothing() {
        let f = Fixture::new();
        let ctx = f.ctx(ToolCall::default());
        let m = manifest(&["read"], &["filesystem_object"]);
        let d = check(&ctx, &m, &NoopMetrics);
        assert_eq!(d.failed_step(), Some(FailedStep::Resolution));
    }

    #[test]
    fn engine_denies_at_step_3_when_no_grant_matches_reach() {
        let mut f = Fixture::new();
        f.agent_grants.push(grant(
            PrincipalRef::Agent(f.agent),
            &["read"],
            "filesystem_object",
        ));
        let ctx = f.ctx(ToolCall::default());
        // Manifest needs network, not filesystem.
        let m = manifest(&["connect"], &["network_endpoint"]);
        let d = check(&ctx, &m, &NoopMetrics);
        assert_eq!(d.failed_step(), Some(FailedStep::Match));
    }

    #[test]
    fn engine_allows_when_single_grant_covers_single_reach() {
        let mut f = Fixture::new();
        f.agent_grants.push(grant(
            PrincipalRef::Agent(f.agent),
            &["read"],
            "filesystem_object",
        ));
        let ctx = f.ctx(ToolCall::default());
        let m = manifest(&["read"], &["filesystem_object"]);
        let d = check(&ctx, &m, &NoopMetrics);
        assert!(d.is_allowed(), "expected Allowed, got {:?}", d);
    }

    #[test]
    fn engine_denies_at_step_4_when_constraint_not_supplied() {
        let mut f = Fixture::new();
        f.agent_grants.push(grant(
            PrincipalRef::Agent(f.agent),
            &["read"],
            "filesystem_object",
        ));
        let ctx = f.ctx(ToolCall::default());
        let mut m = manifest(&["read"], &["filesystem_object"]);
        m.constraints = vec!["path_prefix".into()];
        let d = check(&ctx, &m, &NoopMetrics);
        assert_eq!(d.failed_step(), Some(FailedStep::Constraint));
    }

    #[test]
    fn engine_denies_at_step_4_when_constraint_value_differs_from_requirement() {
        // G3 in the archived M2 plan — the invocation carries SOME
        // value under `purpose`, but not the one the manifest demands.
        let mut f = Fixture::new();
        f.agent_grants.push(grant(
            PrincipalRef::Agent(f.agent),
            &["read"],
            "filesystem_object",
        ));
        let mut call = ToolCall::default();
        call.constraint_context
            .insert("purpose".into(), serde_json::json!("list"));
        let ctx = f.ctx(call);
        let mut m = manifest(&["read"], &["filesystem_object"]);
        m.constraints = vec!["purpose".into()];
        m.constraint_requirements
            .insert("purpose".into(), serde_json::json!("reveal"));
        let d = check(&ctx, &m, &NoopMetrics);
        assert_eq!(
            d.failed_step(),
            Some(FailedStep::Constraint),
            "value-match mismatch must fail at Step 4"
        );
    }

    #[test]
    fn engine_allows_when_constraint_value_matches_requirement() {
        let mut f = Fixture::new();
        f.agent_grants.push(grant(
            PrincipalRef::Agent(f.agent),
            &["read"],
            "filesystem_object",
        ));
        let mut call = ToolCall::default();
        call.constraint_context
            .insert("purpose".into(), serde_json::json!("reveal"));
        let ctx = f.ctx(call);
        let mut m = manifest(&["read"], &["filesystem_object"]);
        m.constraints = vec!["purpose".into()];
        m.constraint_requirements
            .insert("purpose".into(), serde_json::json!("reveal"));
        let d = check(&ctx, &m, &NoopMetrics);
        assert!(d.is_allowed(), "exact value-match must pass Step 4");
    }

    #[test]
    fn engine_presence_only_still_works_when_no_requirement_set() {
        // No constraint_requirements → Step 4 only checks presence
        // (M1 behaviour preserved).
        let mut f = Fixture::new();
        f.agent_grants.push(grant(
            PrincipalRef::Agent(f.agent),
            &["read"],
            "filesystem_object",
        ));
        let mut call = ToolCall::default();
        call.constraint_context
            .insert("purpose".into(), serde_json::json!("anything"));
        let ctx = f.ctx(call);
        let mut m = manifest(&["read"], &["filesystem_object"]);
        m.constraints = vec!["purpose".into()];
        // No constraint_requirements entry.
        let d = check(&ctx, &m, &NoopMetrics);
        assert!(d.is_allowed(), "presence-only must stay green");
    }

    #[test]
    fn engine_allows_when_constraint_supplied_in_context() {
        let mut f = Fixture::new();
        f.agent_grants.push(grant(
            PrincipalRef::Agent(f.agent),
            &["read"],
            "filesystem_object",
        ));
        let mut call = ToolCall::default();
        call.constraint_context
            .insert("path_prefix".into(), serde_json::json!("/workspace/"));
        let ctx = f.ctx(call);
        let mut m = manifest(&["read"], &["filesystem_object"]);
        m.constraints = vec!["path_prefix".into()];
        let d = check(&ctx, &m, &NoopMetrics);
        assert!(d.is_allowed(), "expected Allowed, got {:?}", d);
    }

    #[test]
    fn system_root_grant_allows_any_target_under_that_fundamental() {
        let mut f = Fixture::new();
        // Bootstrap-style grant: allocate on system:root, covers all
        // fundamentals via the special case in resolve_grant.
        f.agent_grants.push(grant(
            PrincipalRef::Agent(f.agent),
            &["allocate"],
            "system:root",
        ));
        f.catalogue.seed(None, "system:root");
        let ctx = f.ctx(ToolCall {
            target_uri: "system:root".into(),
            ..Default::default()
        });
        let m = manifest(&["allocate"], &["identity_principal"]);
        let d = check(&ctx, &m, &NoopMetrics);
        assert!(d.is_allowed(), "expected Allowed, got {:?}", d);
    }

    #[test]
    fn engine_denies_at_step_2a_when_ceiling_empties_candidates() {
        let mut f = Fixture::new();
        // Candidate: read filesystem_object.
        f.agent_grants.push(grant(
            PrincipalRef::Agent(f.agent),
            &["read"],
            "filesystem_object",
        ));
        // Ceiling: only allows network_endpoint — filesystem reach is not
        // under the ceiling's fundamentals.
        f.ceiling_grants.push(grant(
            PrincipalRef::Agent(f.agent),
            &["read"],
            "network_endpoint",
        ));
        let ctx = f.ctx(ToolCall::default());
        let m = manifest(&["read"], &["filesystem_object"]);
        let d = check(&ctx, &m, &NoopMetrics);
        assert_eq!(d.failed_step(), Some(FailedStep::Ceiling));
    }

    #[test]
    fn engine_returns_pending_when_template_gated_grant_lacks_consent() {
        use crate::model::ids::AuthRequestId;
        let mut f = Fixture::new();
        let org = OrgId::new();
        f.org = Some(org);
        // Template-sourced grant.
        let ar = AuthRequestId::new();
        let mut g = grant(PrincipalRef::Agent(f.agent), &["read"], "filesystem_object");
        g.descends_from = Some(ar);
        f.agent_grants.push(g);
        f.template_gated.insert(ar);
        let target = AgentId::new();
        let ctx = f.ctx(ToolCall {
            target_agent: Some(target),
            ..Default::default()
        });
        let m = manifest(&["read"], &["filesystem_object"]);
        let d = check(&ctx, &m, &NoopMetrics);
        assert!(matches!(d, Decision::Pending { .. }));
    }

    #[test]
    fn engine_allows_template_gated_grant_when_consent_present() {
        use crate::model::ids::AuthRequestId;
        let mut f = Fixture::new();
        let org = OrgId::new();
        f.org = Some(org);
        let ar = AuthRequestId::new();
        let mut g = grant(PrincipalRef::Agent(f.agent), &["read"], "filesystem_object");
        g.descends_from = Some(ar);
        f.agent_grants.push(g);
        f.template_gated.insert(ar);
        let target = AgentId::new();
        f.consents = ConsentIndex::from_pairs([(target, org)]);
        let ctx = f.ctx(ToolCall {
            target_agent: Some(target),
            ..Default::default()
        });
        let m = manifest(&["read"], &["filesystem_object"]);
        let d = check(&ctx, &m, &NoopMetrics);
        assert!(d.is_allowed(), "expected Allowed, got {:?}", d);
    }

    #[test]
    fn agent_tier_beats_project_and_org_in_scope_cascade() {
        let mut f = Fixture::new();
        let proj = ProjectId::new();
        let org = OrgId::new();
        f.project = Some(proj);
        f.org = Some(org);

        // All three scopes can read filesystem_object.
        f.agent_grants.push(grant(
            PrincipalRef::Agent(f.agent),
            &["read"],
            "filesystem_object",
        ));
        f.project_grants.push(grant(
            PrincipalRef::Project(proj),
            &["read"],
            "filesystem_object",
        ));
        f.org_grants.push(grant(
            PrincipalRef::Organization(org),
            &["read"],
            "filesystem_object",
        ));

        let ctx = f.ctx(ToolCall::default());
        let m = manifest(&["read"], &["filesystem_object"]);
        let d = check(&ctx, &m, &NoopMetrics);
        let allowed = match d {
            Decision::Allowed { resolved_grants } => resolved_grants,
            other => panic!("expected Allowed, got {:?}", other),
        };
        let winner_id = allowed[0].grant_id;
        // Winner must be the agent-tier grant (first one we pushed).
        assert_eq!(winner_id, f.agent_grants[0].id);
    }

    #[test]
    fn metrics_recorded_on_every_decision() {
        use std::sync::Mutex;
        #[derive(Default)]
        struct Cap(Mutex<Vec<(String, Option<String>)>>);
        impl PermissionCheckMetrics for Cap {
            fn record(&self, _d: std::time::Duration, r: &str, f: Option<&str>) {
                self.0
                    .lock()
                    .unwrap()
                    .push((r.to_string(), f.map(|s| s.to_string())));
            }
        }
        let cap = Cap::default();
        let f = Fixture::new();
        let ctx = f.ctx(ToolCall::default());
        let m = manifest(&["read"], &["filesystem_object"]);
        let _ = check(&ctx, &m, &cap);
        let recorded = cap.0.lock().unwrap().clone();
        assert_eq!(recorded.len(), 1);
        assert_eq!(recorded[0].0, "denied");
        assert_eq!(recorded[0].1.as_deref(), Some("2")); // NoGrantsHeld → Resolution.
    }
}
