//! The s01 bootstrap-claim flow.
//!
//! The HTTP handler (P6) will validate input, call [`execute_claim`], and
//! map the result to an HTTP response. This module is handler-free — it
//! takes a typed [`ClaimInput`] + `&dyn Repository` and returns a
//! [`ClaimOutcome`] (on success) or a [`ClaimRejection`] (on expected
//! error shapes like `BOOTSTRAP_INVALID`). Low-level repository / hashing
//! failures surface as [`ClaimError`].
//!
//! Flow (per `requirements/system/s01-bootstrap-template-adoption.md` +
//! `requirements/admin/01-platform-bootstrap-claim.md` §6 W1):
//!
//! 1. Reject if a platform admin already exists (409).
//! 2. Find one **unconsumed** credential whose argon2id hash verifies the
//!    supplied plaintext. If none matches → reject 403 `BOOTSTRAP_INVALID`.
//!    If one matches but is already consumed → reject 403
//!    `BOOTSTRAP_ALREADY_CONSUMED`.
//! 3. Build the seven entities of the s01 flow (Human Agent, Channel,
//!    Inbox, Outbox, Bootstrap Auth Request, Grant, Audit Event) + the
//!    catalogue seeds.
//! 4. Commit atomically via [`Repository::apply_bootstrap_claim`].

use chrono::Utc;
use domain::audit::{AuditClass, AuditEvent};
use domain::model::ids::{AgentId, AuditEventId, AuthRequestId, GrantId, NodeId, TemplateId};
use domain::model::nodes::{
    Agent, AgentKind, ApproverSlot, ApproverSlotState, AuthRequest, AuthRequestState, Channel,
    ChannelKind, Grant, InboxObject, OutboxObject, PrincipalRef, ResourceRef, ResourceSlot,
    ResourceSlotState,
};
use domain::repository::{BootstrapClaim, Repository, RepositoryError};

use super::credential::verify_credential;

/// Validated input to [`execute_claim`] — what the HTTP handler builds
/// from the incoming POST body.
#[derive(Debug, Clone)]
pub struct ClaimInput {
    /// Full `bphi-bootstrap-...` credential string from the admin.
    pub bootstrap_credential: String,
    /// Non-empty human-readable name.
    pub display_name: String,
    /// Slack / email / web.
    pub channel_kind: ChannelKind,
    /// Non-empty channel handle.
    pub channel_handle: String,
}

/// Happy-path outcome. Fields mirror the API contract in
/// `requirements/admin/01` §10.
#[derive(Debug, Clone)]
pub struct ClaimOutcome {
    pub human_agent_id: AgentId,
    pub inbox_id: NodeId,
    pub outbox_id: NodeId,
    pub grant_id: GrantId,
    pub bootstrap_auth_request_id: AuthRequestId,
    pub audit_event_id: AuditEventId,
}

/// The specific rejections the admin contract names — every one of these
/// maps to a well-known HTTP response code. Internal errors surface as
/// [`ClaimError::Internal`] instead.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ClaimRejection {
    /// 400 — validation failure (empty display_name / empty handle).
    #[error("validation failed: {0}")]
    Invalid(&'static str),
    /// 403 `BOOTSTRAP_INVALID` — the supplied credential does not match
    /// any stored hash (including the case where no credential exists at
    /// all).
    #[error("bootstrap credential invalid")]
    CredentialInvalid,
    /// 403 `BOOTSTRAP_ALREADY_CONSUMED` — we matched the credential but
    /// it has already been consumed.
    #[error("bootstrap credential already consumed")]
    CredentialAlreadyConsumed,
    /// 409 — another claim has already succeeded; no new claims allowed.
    #[error("a platform admin has already been claimed")]
    AlreadyClaimed,
}

/// Internal error the HTTP handler should return as 500. Covers storage
/// and hashing failures that don't fit a semantic contract shape.
#[derive(Debug, thiserror::Error)]
pub enum ClaimError {
    #[error("rejected by contract: {0}")]
    Rejected(#[from] ClaimRejection),
    #[error("repository error: {0}")]
    Repository(#[from] RepositoryError),
    #[error("hash verification failed: {0}")]
    Hash(#[from] argon2::password_hash::Error),
}

/// Run the atomic s01 bootstrap-claim flow.
pub async fn execute_claim(
    repo: &dyn Repository,
    input: ClaimInput,
) -> Result<ClaimOutcome, ClaimError> {
    // --- Validate input (fast fail before any IO) -------------------------
    if input.display_name.trim().is_empty() {
        return Err(ClaimRejection::Invalid("display_name must not be empty").into());
    }
    if input.channel_handle.trim().is_empty() {
        return Err(ClaimRejection::Invalid("channel_handle must not be empty").into());
    }
    let supplied = input.bootstrap_credential.trim();
    if supplied.is_empty() {
        return Err(ClaimRejection::Invalid("bootstrap_credential must not be empty").into());
    }

    // --- Refuse if admin already exists (409) -----------------------------
    if repo.get_admin_agent().await?.is_some() {
        return Err(ClaimRejection::AlreadyClaimed.into());
    }

    // --- Locate + verify credential ---------------------------------------
    // The stored "digest" is the PHC-encoded argon2id hash, which embeds
    // its own salt. We can't query by exact hash equality (salts differ
    // per credential), so we scan bootstrap_credentials and verify each
    // against the supplied plaintext. Platform-admin bootstrap creates at
    // most a handful of credentials — O(n) is fine.
    let all = repo
        .list_bootstrap_credentials(/* unconsumed_only = */ false)
        .await?;
    let mut matched_unconsumed = None;
    let mut matched_consumed = false;
    for row in all {
        if verify_credential(supplied, &row.digest)? {
            if row.consumed_at.is_some() {
                matched_consumed = true;
            } else {
                matched_unconsumed = Some(row);
                break;
            }
        }
    }
    let credential = match matched_unconsumed {
        Some(row) => row,
        None => {
            if matched_consumed {
                return Err(ClaimRejection::CredentialAlreadyConsumed.into());
            }
            return Err(ClaimRejection::CredentialInvalid.into());
        }
    };

    // --- Build the seven entities + catalogue seeds -----------------------
    let now = Utc::now();
    let agent_id = AgentId::new();
    let channel_id = NodeId::new();
    let inbox_id = NodeId::new();
    let outbox_id = NodeId::new();
    let auth_request_id = AuthRequestId::new();
    let grant_id = GrantId::new();
    let audit_event_id = AuditEventId::new();

    let human_agent = Agent {
        id: agent_id,
        kind: AgentKind::Human,
        display_name: input.display_name.trim().to_string(),
        owning_org: None,
        created_at: now,
    };
    let channel = Channel {
        id: channel_id,
        agent_id,
        kind: input.channel_kind,
        handle: input.channel_handle.trim().to_string(),
        created_at: now,
    };
    let inbox = InboxObject {
        id: inbox_id,
        agent_id,
        created_at: now,
    };
    let outbox = OutboxObject {
        id: outbox_id,
        agent_id,
        created_at: now,
    };

    // Bootstrap Auth Request in Approved state — system:genesis both
    // requests and approves (R-SYS-s01-1).
    let auth_request = AuthRequest {
        id: auth_request_id,
        requestor: PrincipalRef::System("system:genesis".into()),
        kinds: vec!["control_plane_object".into()],
        scope: vec!["allocate".into()],
        state: AuthRequestState::Approved,
        valid_until: None,
        submitted_at: now,
        resource_slots: vec![ResourceSlot {
            resource: ResourceRef {
                uri: "system:root".into(),
            },
            approvers: vec![ApproverSlot {
                approver: PrincipalRef::System("system:genesis".into()),
                state: ApproverSlotState::Approved,
                responded_at: Some(now),
                reconsidered_at: None,
            }],
            state: ResourceSlotState::Approved,
        }],
        justification: Some("system bootstrap template adoption".into()),
        audit_class: AuditClass::Alerted,
        terminal_state_entered_at: Some(now),
        archived: false,
        active_window_days: 3650, // Retain the genesis record for ~10 yrs.
        // A stable, hardcoded Template id so future audit traversal can
        // terminate cleanly at the axiom. Uses the all-zero UUID to mark
        // "the bootstrap template" unambiguously.
        provenance_template: Some(TemplateId::from_uuid(uuid::Uuid::nil())),
    };

    // `[allocate]`-on-`system:root` Grant — underpins every delegation
    // downstream. `fundamentals` intentionally left empty — the
    // `system:root` URI is special-cased in `resolve_grant` (it admits
    // every fundamental), so the new P4.5 field is unnecessary here.
    let grant = Grant {
        id: grant_id,
        holder: PrincipalRef::Agent(agent_id),
        action: vec!["allocate".into()],
        resource: ResourceRef {
            uri: "system:root".into(),
        },
        fundamentals: vec![],
        descends_from: Some(auth_request_id),
        delegable: true,
        issued_at: now,
        revoked_at: None,
    };

    // Alerted audit event — `PlatformAdminClaimed`.
    let audit_event = AuditEvent {
        event_id: audit_event_id,
        event_type: "platform_admin.claimed".into(),
        actor_agent_id: Some(agent_id),
        target_entity_id: Some(NodeId::from_uuid(*agent_id.as_uuid())),
        timestamp: now,
        diff: serde_json::json!({
            "before": null,
            "after": {
                "human_agent_id": agent_id.to_string(),
                "display_name": human_agent.display_name,
                "channel_kind": format!("{:?}", input.channel_kind).to_lowercase(),
                "channel_handle": channel.handle,
                "bootstrap_auth_request_id": auth_request_id.to_string(),
                "grant_id": grant_id.to_string(),
                "bootstrap_credential_digest_fragment": digest_fragment(&credential.digest),
            }
        }),
        audit_class: AuditClass::Alerted,
        provenance_auth_request_id: Some(auth_request_id),
        org_scope: None,
        prev_event_hash: None, // Platform-scope root chain — first event has no prev.
    };

    // Platform-level catalogue seeds (R-SYS-s01-3).
    let catalogue_entries = vec![
        (
            "system:root".to_string(),
            "control_plane_object".to_string(),
        ),
        (format!("inbox:{}", inbox_id), "inbox_object".to_string()),
        (format!("outbox:{}", outbox_id), "outbox_object".to_string()),
    ];

    let claim = BootstrapClaim {
        credential_record_id: credential.record_id,
        human_agent,
        channel,
        inbox,
        outbox,
        auth_request,
        grant,
        catalogue_entries,
        audit_event,
    };

    // --- Commit atomically ------------------------------------------------
    repo.apply_bootstrap_claim(&claim).await?;

    Ok(ClaimOutcome {
        human_agent_id: agent_id,
        inbox_id,
        outbox_id,
        grant_id,
        bootstrap_auth_request_id: auth_request_id,
        audit_event_id,
    })
}

/// The leading characters of the stored hash — enough to correlate the
/// install-time event and the claim event without exposing the full hash
/// (per admin/01 §N3 guidance). Intentionally short.
fn digest_fragment(digest: &str) -> String {
    digest.chars().take(24).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bootstrap::credential::hash_credential;
    use domain::in_memory::InMemoryRepository;

    async fn seed_credential(repo: &InMemoryRepository, plaintext: &str) -> String {
        let hash = hash_credential(plaintext).unwrap();
        let row = repo.put_bootstrap_credential(hash).await.unwrap();
        row.record_id
    }

    fn sample_input(credential: &str) -> ClaimInput {
        ClaimInput {
            bootstrap_credential: credential.to_string(),
            display_name: "Alex Chen".into(),
            channel_kind: ChannelKind::Slack,
            channel_handle: "@alex".into(),
        }
    }

    #[tokio::test]
    async fn happy_path_returns_ids_and_marks_admin_claimed() {
        let repo = InMemoryRepository::new();
        let plaintext = "bphi-bootstrap-happy";
        seed_credential(&repo, plaintext).await;

        let out = execute_claim(&repo, sample_input(plaintext)).await.unwrap();

        // Admin exists.
        let admin = repo.get_admin_agent().await.unwrap().unwrap();
        assert_eq!(admin.id, out.human_agent_id);
        assert_eq!(admin.kind, AgentKind::Human);
        assert_eq!(admin.display_name, "Alex Chen");

        // Grant exists with [allocate] on system:root.
        let grant = repo.get_grant(out.grant_id).await.unwrap().unwrap();
        assert_eq!(grant.action, vec!["allocate".to_string()]);
        assert_eq!(grant.resource.uri, "system:root");
        assert_eq!(grant.descends_from, Some(out.bootstrap_auth_request_id));
        assert!(grant.delegable);

        // Auth Request is Approved.
        let ar = repo
            .get_auth_request(out.bootstrap_auth_request_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(ar.state, AuthRequestState::Approved);
        assert_eq!(ar.resource_slots[0].resource.uri, "system:root");

        // Catalogue contains system:root.
        assert!(repo.catalogue_contains(None, "system:root").await.unwrap());
    }

    #[tokio::test]
    async fn invalid_credential_returns_credential_invalid() {
        let repo = InMemoryRepository::new();
        seed_credential(&repo, "bphi-bootstrap-real").await;

        let err = execute_claim(&repo, sample_input("bphi-bootstrap-wrong"))
            .await
            .unwrap_err();
        match err {
            ClaimError::Rejected(ClaimRejection::CredentialInvalid) => {}
            other => panic!("expected CredentialInvalid, got {:?}", other),
        }
        // And no admin was created.
        assert!(repo.get_admin_agent().await.unwrap().is_none());
    }

    #[tokio::test]
    async fn reused_credential_returns_already_consumed() {
        let repo = InMemoryRepository::new();
        let plaintext = "bphi-bootstrap-reuse";
        seed_credential(&repo, plaintext).await;
        // First claim consumes the credential.
        execute_claim(&repo, sample_input(plaintext)).await.unwrap();

        // Second attempt: admin already exists → AlreadyClaimed takes
        // precedence over AlreadyConsumed (per admin/01 §10 the 409 is
        // the first-line check).
        let err = execute_claim(&repo, sample_input(plaintext))
            .await
            .unwrap_err();
        match err {
            ClaimError::Rejected(ClaimRejection::AlreadyClaimed) => {}
            other => panic!("expected AlreadyClaimed, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn reused_credential_without_admin_returns_already_consumed() {
        // Directly construct a scenario where the credential was consumed
        // but no admin exists (edge case — shouldn't arise in practice,
        // but pinning the contract).
        let repo = InMemoryRepository::new();
        let plaintext = "bphi-bootstrap-orphan";
        let record_id = seed_credential(&repo, plaintext).await;
        // Manually consume without running the full claim.
        repo.consume_bootstrap_credential(&record_id).await.unwrap();

        let err = execute_claim(&repo, sample_input(plaintext))
            .await
            .unwrap_err();
        match err {
            ClaimError::Rejected(ClaimRejection::CredentialAlreadyConsumed) => {}
            other => panic!("expected CredentialAlreadyConsumed, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn empty_display_name_rejects_400() {
        let repo = InMemoryRepository::new();
        seed_credential(&repo, "bphi-bootstrap-valid").await;
        let mut input = sample_input("bphi-bootstrap-valid");
        input.display_name = "   ".into();
        let err = execute_claim(&repo, input).await.unwrap_err();
        match err {
            ClaimError::Rejected(ClaimRejection::Invalid(_)) => {}
            other => panic!("expected Invalid, got {:?}", other),
        }
    }
}
