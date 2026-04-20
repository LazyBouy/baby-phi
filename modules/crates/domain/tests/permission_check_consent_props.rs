//! Property tests for Step 6 — Consent gating (Templates A/B/C/D).
//!
//! Invariants covered:
//!
//! 1. `template_gated_grant_without_consent_is_pending` — a grant whose
//!    `descends_from` is in the template-gated set, combined with a call
//!    that targets a specific agent + an org without a recorded consent,
//!    yields `Pending`.
//! 2. `template_gated_grant_with_consent_is_allowed` — adding the
//!    matching `(subordinate, org)` to the consent index flips the
//!    outcome to `Allowed`.
//! 3. `non_template_grant_never_becomes_pending` — grants whose
//!    `descends_from` is None OR whose Auth Request is not
//!    template-gated never yield `Pending`.

mod common;

use common::*;
use domain::model::ids::{AgentId, AuthRequestId, OrgId};
use domain::model::nodes::PrincipalRef;
use domain::permissions::manifest::ConsentIndex;
use domain::permissions::{check, Decision, NoopMetrics, ToolCall};

use proptest::prelude::*;

proptest! {
    #[test]
    fn template_gated_grant_without_consent_is_pending(
        fundamental in any_fundamental(),
        action in any_action(),
    ) {
        let agent = AgentId::new();
        let target = AgentId::new();
        let org = OrgId::new();
        let ar = AuthRequestId::new();
        let mut g = grant_on(PrincipalRef::Agent(agent), &[&action], fundamental.as_str());
        g.descends_from = Some(ar);

        let mut ctx_owned = ctx_with_agent_grants(vec![g]);
        ctx_owned.agent = agent;
        ctx_owned.current_org = Some(org);
        ctx_owned.template_gated.insert(ar);

        let call = ToolCall {
            target_agent: Some(target),
            ..Default::default()
        };
        let ctx = ctx_owned.borrow(call);
        let m = manifest_of(&[&action], &[fundamental.as_str()]);
        let d = check(&ctx, &m, &NoopMetrics);
        prop_assert!(
            matches!(d, Decision::Pending { .. }),
            "expected Pending, got {:?}",
            d
        );
    }

    #[test]
    fn template_gated_grant_with_consent_is_allowed(
        fundamental in any_fundamental(),
        action in any_action(),
    ) {
        let agent = AgentId::new();
        let target = AgentId::new();
        let org = OrgId::new();
        let ar = AuthRequestId::new();
        let mut g = grant_on(PrincipalRef::Agent(agent), &[&action], fundamental.as_str());
        g.descends_from = Some(ar);

        let mut ctx_owned = ctx_with_agent_grants(vec![g]);
        ctx_owned.agent = agent;
        ctx_owned.current_org = Some(org);
        ctx_owned.template_gated.insert(ar);
        ctx_owned.consents = ConsentIndex::from_pairs([(target, org)]);

        let call = ToolCall {
            target_agent: Some(target),
            ..Default::default()
        };
        let ctx = ctx_owned.borrow(call);
        let m = manifest_of(&[&action], &[fundamental.as_str()]);
        let d = check(&ctx, &m, &NoopMetrics);
        prop_assert!(d.is_allowed(), "expected Allowed, got {:?}", d);
    }

    #[test]
    fn grant_not_in_template_gated_set_never_yields_pending(
        fundamental in any_fundamental(),
        action in any_action(),
        attach_ar in proptest::bool::ANY,
    ) {
        let agent = AgentId::new();
        let target = AgentId::new();
        let org = OrgId::new();
        let mut g = grant_on(PrincipalRef::Agent(agent), &[&action], fundamental.as_str());
        if attach_ar {
            g.descends_from = Some(AuthRequestId::new());
        }

        let mut ctx_owned = ctx_with_agent_grants(vec![g]);
        ctx_owned.agent = agent;
        ctx_owned.current_org = Some(org);
        // template_gated stays empty — no Auth Request in it.

        let call = ToolCall {
            target_agent: Some(target),
            ..Default::default()
        };
        let ctx = ctx_owned.borrow(call);
        let m = manifest_of(&[&action], &[fundamental.as_str()]);
        let d = check(&ctx, &m, &NoopMetrics);
        prop_assert!(
            !matches!(d, Decision::Pending { .. }),
            "expected non-Pending, got {:?}",
            d
        );
    }
}
