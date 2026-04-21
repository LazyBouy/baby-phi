//! Template E — **self-interested auto-approve**.
//!
//! The platform admin (or any principal that holds both the write
//! authority and the approval authority for a given resource) uses
//! this template to mint an Auth Request that is already in
//! [`AuthRequestState::Approved`] at construction time. The requestor
//! fills their own approver slot in the same struct; no transition is
//! required afterwards.
//!
//! Every M2 admin write (pages 02–05) uses this shape:
//! "add a secret", "register a model provider", "archive an MCP
//! server", "update platform defaults" — they are all Template-E
//! events under the hood.
//!
//! ## Why pre-built-Approved and not a transition?
//!
//! The 9-state machine's legal-transition table has no
//! `Unfilled → Approved` edge for approver slots (per
//! `auth_requests/transitions.rs`). Driving the normal transition
//! pipeline to satisfy it would add complexity for a shape where the
//! requestor and approver are the same principal. The pre-built
//! construction matches what `server::bootstrap::claim` already does
//! for the `SystemBootstrap` template — this module generalises that
//! pattern.
//!
//! ADR: `docs/specs/v0/implementation/m2/decisions/0016-template-e-self-interested-auto-approve.md`.

use chrono::{DateTime, Utc};

use crate::audit::AuditClass;
use crate::model::ids::AuthRequestId;
use crate::model::nodes::{
    ApproverSlot, ApproverSlotState, AuthRequest, AuthRequestState, PrincipalRef, ResourceRef,
    ResourceSlot, ResourceSlotState,
};

/// Default active-window for Template-E Auth Requests (matches the M1
/// bootstrap Auth Request). 30 days gives the audit chain a finite
/// retention window without forcing per-write tuning.
pub const DEFAULT_ACTIVE_WINDOW_DAYS: u32 = 30;

/// Inputs the caller provides to mint a Template-E Auth Request.
///
/// `kinds` is the list of `#kind:{composite}` filters the request
/// applies to (e.g. `#kind:secret_credential` for a vault write).
/// `scope` is the concrete scope strings the resulting grants will
/// carry (e.g. `["secret:anthropic-api-key"]`).
///
/// `justification` is free-form human text explaining the write —
/// surfaced in the audit log. For platform-admin writes "self-approved
/// platform-admin action" is the canonical phrasing.
#[derive(Debug, Clone)]
pub struct BuildArgs {
    /// The principal that is both requestor AND approver.
    pub requestor_and_approver: PrincipalRef,
    /// The single resource the Auth Request covers.
    pub resource: ResourceRef,
    /// Composite-kind filters the resulting grants should carry.
    pub kinds: Vec<String>,
    /// Concrete scope strings for the resulting grants.
    pub scope: Vec<String>,
    /// Free-form justification surfaced in the audit event.
    pub justification: Option<String>,
    /// Audit class for the resulting event (typically `Alerted` for
    /// sensitive writes like vault ops, `Logged` for routine ones).
    pub audit_class: AuditClass,
    /// Wall-clock time for `submitted_at` / approver `responded_at`.
    pub now: DateTime<Utc>,
}

/// Build a fully-Approved Auth Request for Template E.
///
/// The returned `AuthRequest`:
/// - carries a fresh [`AuthRequestId`];
/// - is in state [`AuthRequestState::Approved`];
/// - has exactly one [`ResourceSlot`] covering `args.resource`, with
///   exactly one [`ApproverSlot`] already in
///   [`ApproverSlotState::Approved`] (filled by the requestor);
/// - has `provenance_template = None` — callers that want to link the
///   AR back to a specific `Template` graph node should set that
///   separately after persisting the template row.
/// - has `terminal_state_entered_at = Some(args.now)` — Approved is a
///   terminal state per the lifecycle; this timestamp drives retention.
///
/// This function is **pure**: no I/O, no random state outside the
/// generated `AuthRequestId`, no hidden clock access (`args.now` is
/// injected). Callers compose persistence (write the AR, issue the
/// grants, emit the audit event) on top.
pub fn build_auto_approved_request(args: BuildArgs) -> AuthRequest {
    let BuildArgs {
        requestor_and_approver,
        resource,
        kinds,
        scope,
        justification,
        audit_class,
        now,
    } = args;

    let approver_slot = ApproverSlot {
        approver: requestor_and_approver.clone(),
        state: ApproverSlotState::Approved,
        responded_at: Some(now),
        reconsidered_at: None,
    };
    let resource_slot = ResourceSlot {
        resource,
        approvers: vec![approver_slot],
        state: ResourceSlotState::Approved,
    };

    AuthRequest {
        id: AuthRequestId::new(),
        requestor: requestor_and_approver,
        kinds,
        scope,
        state: AuthRequestState::Approved,
        valid_until: None,
        submitted_at: now,
        resource_slots: vec![resource_slot],
        justification,
        audit_class,
        terminal_state_entered_at: Some(now),
        archived: false,
        active_window_days: DEFAULT_ACTIVE_WINDOW_DAYS,
        provenance_template: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ids::AgentId;

    fn sample_args() -> BuildArgs {
        BuildArgs {
            requestor_and_approver: PrincipalRef::Agent(AgentId::new()),
            resource: ResourceRef {
                uri: "secret:anthropic-api-key".to_string(),
            },
            kinds: vec!["#kind:secret_credential".to_string()],
            scope: vec!["secret:anthropic-api-key".to_string()],
            justification: Some("self-approved platform-admin action".to_string()),
            audit_class: AuditClass::Alerted,
            now: Utc::now(),
        }
    }

    #[test]
    fn built_request_is_already_approved() {
        let ar = build_auto_approved_request(sample_args());
        assert_eq!(ar.state, AuthRequestState::Approved);
    }

    #[test]
    fn built_request_has_exactly_one_slot_with_one_filled_approver() {
        let ar = build_auto_approved_request(sample_args());
        assert_eq!(ar.resource_slots.len(), 1);
        let slot = &ar.resource_slots[0];
        assert_eq!(slot.state, ResourceSlotState::Approved);
        assert_eq!(slot.approvers.len(), 1);
        let app = &slot.approvers[0];
        assert_eq!(app.state, ApproverSlotState::Approved);
        assert!(app.responded_at.is_some());
    }

    #[test]
    fn requestor_equals_approver() {
        let requestor = PrincipalRef::Agent(AgentId::new());
        let mut args = sample_args();
        args.requestor_and_approver = requestor.clone();
        let ar = build_auto_approved_request(args);
        match (&ar.requestor, &ar.resource_slots[0].approvers[0].approver) {
            (PrincipalRef::Agent(a), PrincipalRef::Agent(b)) => assert_eq!(a, b),
            _ => panic!("requestor/approver must match the PrincipalRef we provided"),
        }
    }

    #[test]
    fn terminal_state_timestamp_matches_now() {
        let args = sample_args();
        let expected = args.now;
        let ar = build_auto_approved_request(args);
        assert_eq!(ar.terminal_state_entered_at, Some(expected));
        assert_eq!(ar.submitted_at, expected);
    }

    #[test]
    fn fresh_id_each_call() {
        let a = build_auto_approved_request(sample_args());
        let b = build_auto_approved_request(sample_args());
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn audit_class_is_carried_through() {
        let mut args = sample_args();
        args.audit_class = AuditClass::Logged;
        let ar = build_auto_approved_request(args);
        assert_eq!(ar.audit_class, AuditClass::Logged);
    }

    #[test]
    fn provenance_template_starts_unset() {
        let ar = build_auto_approved_request(sample_args());
        assert!(ar.provenance_template.is_none());
    }

    #[test]
    fn active_window_defaults_to_30_days() {
        let ar = build_auto_approved_request(sample_args());
        assert_eq!(ar.active_window_days, DEFAULT_ACTIVE_WINDOW_DAYS);
        assert_eq!(ar.active_window_days, 30);
    }
}
