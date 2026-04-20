//! Shared fixtures + proptest strategies for the Permission Check engine.
//!
//! Each `permission_check_*_props.rs` integration file declares
//! `mod common;` to pull these in. The builders keep the test bodies short
//! so the invariant being checked stays readable.

#![allow(dead_code)]

use std::collections::{HashMap, HashSet};

use chrono::{Duration, TimeZone, Utc};
use domain::model::ids::{AgentId, AuthRequestId, GrantId, OrgId, ProjectId};
use domain::model::nodes::{Grant, PrincipalRef, ResourceRef};
use domain::model::Fundamental;
use domain::permissions::manifest::ConsentIndex;
use domain::permissions::{CatalogueLookup, CheckContext, Manifest, StaticCatalogue, ToolCall};

use proptest::collection::vec;
use proptest::prelude::*;

// ----------------------------------------------------------------------------
// Owned context — owns the Vec<Grant>s so CheckContext can borrow them.
// ----------------------------------------------------------------------------

pub struct OwnedCtx {
    pub agent: AgentId,
    pub current_org: Option<OrgId>,
    pub current_project: Option<ProjectId>,
    pub agent_grants: Vec<Grant>,
    pub project_grants: Vec<Grant>,
    pub org_grants: Vec<Grant>,
    pub ceiling_grants: Vec<Grant>,
    pub catalogue: StaticCatalogue,
    pub consents: ConsentIndex,
    pub template_gated: HashSet<AuthRequestId>,
}

impl OwnedCtx {
    pub fn borrow<'a>(&'a self, call: ToolCall) -> CheckContext<'a> {
        CheckContext {
            agent: self.agent,
            current_org: self.current_org,
            current_project: self.current_project,
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

/// Build an [`OwnedCtx`] with a fresh agent, no project/org, and the given
/// agent-tier grant set. Handy when the test only cares about the agent
/// tier.
pub fn ctx_with_agent_grants(grants: Vec<Grant>) -> OwnedCtx {
    OwnedCtx {
        agent: AgentId::new(),
        current_org: None,
        current_project: None,
        agent_grants: grants,
        project_grants: vec![],
        org_grants: vec![],
        ceiling_grants: vec![],
        catalogue: StaticCatalogue::empty(),
        consents: ConsentIndex::empty(),
        template_gated: HashSet::new(),
    }
}

// ----------------------------------------------------------------------------
// Builders
// ----------------------------------------------------------------------------

/// One-shot deterministic timestamp builder — keeps `issued_at` ordering
/// predictable in tests.
pub fn ts(offset_secs: i64) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap() + Duration::seconds(offset_secs)
}

pub fn grant_on(holder: PrincipalRef, actions: &[&str], resource_uri: &str) -> Grant {
    Grant {
        id: GrantId::new(),
        holder,
        action: actions.iter().map(|s| s.to_string()).collect(),
        resource: ResourceRef {
            uri: resource_uri.into(),
        },
        descends_from: None,
        delegable: false,
        issued_at: ts(0),
        revoked_at: None,
    }
}

pub fn manifest_of(actions: &[&str], resource: &[&str]) -> Manifest {
    Manifest {
        actions: actions.iter().map(|s| s.to_string()).collect(),
        resource: resource.iter().map(|s| s.to_string()).collect(),
        ..Default::default()
    }
}

// ----------------------------------------------------------------------------
// Strategies
// ----------------------------------------------------------------------------

pub fn any_fundamental() -> impl Strategy<Value = Fundamental> {
    prop_oneof![
        Just(Fundamental::FilesystemObject),
        Just(Fundamental::ProcessExecObject),
        Just(Fundamental::NetworkEndpoint),
        Just(Fundamental::SecretCredential),
        Just(Fundamental::EconomicResource),
        Just(Fundamental::TimeComputeResource),
        Just(Fundamental::DataObject),
        Just(Fundamental::Tag),
        Just(Fundamental::IdentityPrincipal),
    ]
}

pub fn any_action() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("read".to_string()),
        Just("list".to_string()),
        Just("modify".to_string()),
        Just("execute".to_string()),
        Just("allocate".to_string()),
        Just("connect".to_string()),
    ]
}

/// A "bare fundamental" grant — resource URI is a fundamental name so
/// `resolve_grant` recovers the fundamental from the URI directly.
pub fn any_agent_grant(agent: AgentId) -> impl Strategy<Value = Grant> {
    (any_fundamental(), vec(any_action(), 1..=3)).prop_map(move |(f, actions)| Grant {
        id: GrantId::new(),
        holder: PrincipalRef::Agent(agent),
        action: actions,
        resource: ResourceRef {
            uri: f.as_str().to_string(),
        },
        descends_from: None,
        delegable: false,
        issued_at: ts(0),
        revoked_at: None,
    })
}

pub fn any_manifest() -> impl Strategy<Value = Manifest> {
    (vec(any_action(), 1..=3), vec(any_fundamental(), 1..=3)).prop_map(|(actions, resource_fs)| {
        Manifest {
            actions,
            resource: resource_fs
                .into_iter()
                .map(|f| f.as_str().to_string())
                .collect(),
            ..Default::default()
        }
    })
}

/// Helper for `effective_matches` ergonomics — converts a simple
/// `(fundamental, action)` set into a concrete manifest reaching exactly
/// those fundamentals with those actions.
pub fn manifest_reaching(required: &[(Fundamental, &str)]) -> Manifest {
    let mut actions: Vec<String> = Vec::new();
    let mut fundamentals: Vec<String> = Vec::new();
    let mut seen_actions: HashSet<String> = HashSet::new();
    let mut seen_fs: HashSet<Fundamental> = HashSet::new();
    for (f, a) in required {
        if seen_actions.insert((*a).to_string()) {
            actions.push((*a).to_string());
        }
        if seen_fs.insert(*f) {
            fundamentals.push(f.as_str().to_string());
        }
    }
    Manifest {
        actions,
        resource: fundamentals,
        ..Default::default()
    }
}

/// Catalogue that contains every `(org, uri)` pair in `entries`.
pub fn cat_with(entries: &[(Option<OrgId>, &str)]) -> StaticCatalogue {
    StaticCatalogue::with_entries(
        entries
            .iter()
            .map(|(o, u)| (*o, (*u).to_string()))
            .collect::<Vec<_>>(),
    )
}

/// Accumulator-style: group reaches → action list per fundamental.
pub fn reaches_by_fundamental(
    reaches: &[(Fundamental, String)],
) -> HashMap<Fundamental, Vec<String>> {
    let mut out: HashMap<Fundamental, Vec<String>> = HashMap::new();
    for (f, a) in reaches {
        out.entry(*f).or_default().push(a.clone());
    }
    out
}

// ----------------------------------------------------------------------------
// Tiny marker so the module is accepted without triggering unused-import
// warnings when a given test binary doesn't use every helper.
// ----------------------------------------------------------------------------

pub const fn _assert_module_used() {}

// Silence `dead_code` on the catalogue impl when a test binary doesn't use
// the trait object directly (the helpers above are generic over
// `impl CatalogueLookup`).
#[allow(dead_code)]
fn _use_catalogue_trait(_: &dyn CatalogueLookup) {}
