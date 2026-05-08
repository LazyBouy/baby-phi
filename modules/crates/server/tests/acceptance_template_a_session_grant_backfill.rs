//! Acceptance test for migration `0015_template_a_session_object_grant.surql`
//! (CH-15 / ADR-0054 §D54.4).
//!
//! Verifies the backfill walks every active legacy Template A grant
//! and inserts the paired `session_object/project:<uuid>` grant.
//! Idempotency is exercised via the second-run guard (re-applying the
//! migration through the runner is a no-op on the ledger; the
//! migration body's `descends_from NOT IN (...)` clause is the
//! belt-and-braces check against ledger drift).
//!
//! Coverage:
//!   1. `migration_0015_seeds_paired_session_object_grant_for_legacy_template_a_grant`
//!      — happy path: pre-migration legacy grant + Template A AR;
//!      post-migration paired grant on `session_object/project:<id>`
//!      exists with shared `descends_from` and the correct fundamentals.
//!   2. `migration_0015_skips_revoked_legacy_grants` — revoked legacy
//!      grants are NOT backfilled.
//!   3. `migration_0015_skips_non_template_a_grants` — grants whose
//!      `descends_from` points at non-Template-A ARs are NOT
//!      backfilled (template B/C/D ARs ignored).

mod acceptance_common;

use acceptance_common::spawn;
use chrono::Utc;
use domain::audit::AuditClass;
use domain::model::ids::{AgentId, AuthRequestId, GrantId, OrgId};
use domain::model::nodes::{
    ApprovalMode, AuthRequest, AuthRequestState, Grant, PrincipalRef, ResourceRef,
};
use domain::model::Fundamental;
use domain::permissions::Action;
use domain::Repository;
use store::{run_migrations, EMBEDDED_MIGRATIONS};

/// Seed a Template A adoption AR + a legacy single-grant lead.
/// Returns `(ar_id, legacy_grant_id, lead_id, project_uuid)`.
async fn seed_legacy_template_a(
    store: &dyn Repository,
    template_kind_tag: &str,
) -> (AuthRequestId, GrantId, AgentId, uuid::Uuid) {
    let now = Utc::now();
    let org = OrgId::new();
    let lead = AgentId::new();
    let ar_id = AuthRequestId::new();
    // AR stub matching the `auth_request` schema; only the kinds tag
    // matters for the migration's WHERE clause.
    let ar = AuthRequest {
        id: ar_id,
        requestor: PrincipalRef::Agent(lead),
        kinds: vec![template_kind_tag.to_string()],
        scope: vec!["read".to_string()],
        state: AuthRequestState::Approved,
        valid_until: None,
        submitted_at: now,
        resource_slots: vec![],
        justification: None,
        audit_class: AuditClass::Silent,
        terminal_state_entered_at: Some(now),
        archived: false,
        active_window_days: 90,
        provenance_template: None,
        tags: vec![],
        descends_from_grant: None,
    };
    store.create_auth_request(&ar).await.unwrap();

    let project_uuid = uuid::Uuid::new_v4();
    let grant_id = GrantId::new();
    let legacy_grant = Grant {
        id: grant_id,
        holder: PrincipalRef::Agent(lead),
        action: vec![Action::Read, Action::Inspect, Action::List],
        resource: ResourceRef {
            uri: format!("project:{project_uuid}"),
        },
        fundamentals: vec![Fundamental::Tag],
        descends_from: Some(ar_id),
        delegable: false,
        issued_at: now,
        revoked_at: None,
        approval_mode: ApprovalMode::Implicit,
        audit_class: AuditClass::Silent,
        allocate_refinement: None,
    };
    store.create_grant(&legacy_grant).await.unwrap();
    let _ = org;
    (ar_id, grant_id, lead, project_uuid)
}

/// Migration 0015 seeds the paired session-object grant for a legacy
/// Template A holder.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn migration_0015_seeds_paired_session_object_grant_for_legacy_template_a_grant() {
    // The harness runs every migration during `spawn`; we re-run
    // 0015 by re-invoking `run_migrations` against the same client.
    // The ledger guards against re-application of already-recorded
    // versions, but the migration body's NOT-EXISTS clause means we
    // can seed PRE-existing legacy grants AFTER `spawn` and re-run
    // the migration to exercise the body.
    let acc = spawn(false).await;

    let (ar_id, _legacy_grant_id, lead, project_uuid) =
        seed_legacy_template_a(acc.store.as_ref(), "#template:a").await;

    // Re-running the migration set is a no-op on the ledger BUT we
    // need to actually run the body — invoke the inline SQL directly
    // by re-applying the SurrealQL script. Use the runner's idempotent
    // `run_migrations` (skips already-applied versions). To force the
    // body run on the freshly-seeded data, we delete the ledger row
    // for version 15 then re-run.
    acc.store
        .client()
        .query("DELETE FROM _migrations WHERE version = 15")
        .await
        .unwrap();
    let applied = run_migrations(acc.store.client(), EMBEDDED_MIGRATIONS)
        .await
        .expect("re-run migrations");
    assert!(applied.contains(&15), "version 15 should re-apply");

    // The paired grant must exist post-migration.
    let lead_grants = acc
        .store
        .list_grants_for_principal(&PrincipalRef::Agent(lead))
        .await
        .unwrap();
    let session_uri =
        format!(r#"tags contains "project:{project_uuid}" AND tags contains #kind:session"#);
    let paired = lead_grants
        .iter()
        .find(|g| g.resource.uri == session_uri)
        .expect("paired session_object grant must exist post-migration");
    assert_eq!(paired.descends_from, Some(ar_id));
    assert_eq!(
        paired.action,
        vec![Action::Read, Action::Inspect, Action::List]
    );
    // Migration writes the canonical SessionObject composite expansion
    // ([data_object, tag]).
    assert_eq!(
        paired.fundamentals,
        vec![Fundamental::DataObject, Fundamental::Tag]
    );
    assert!(!paired.delegable);
    assert!(paired.revoked_at.is_none());
}

/// Migration 0015 must NOT backfill revoked legacy grants.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn migration_0015_skips_revoked_legacy_grants() {
    let acc = spawn(false).await;

    let (_ar_id, legacy_grant_id, lead, project_uuid) =
        seed_legacy_template_a(acc.store.as_ref(), "#template:a").await;

    // Revoke the legacy grant.
    let now = Utc::now();
    acc.store.revoke_grant(legacy_grant_id, now).await.unwrap();

    // Re-run migration body.
    acc.store
        .client()
        .query("DELETE FROM _migrations WHERE version = 15")
        .await
        .unwrap();
    let _ = run_migrations(acc.store.client(), EMBEDDED_MIGRATIONS)
        .await
        .unwrap();

    // No paired grant on the lead.
    let lead_grants = acc
        .store
        .list_grants_for_principal(&PrincipalRef::Agent(lead))
        .await
        .unwrap();
    let session_uri =
        format!(r#"tags contains "project:{project_uuid}" AND tags contains #kind:session"#);
    let any_paired = lead_grants.iter().any(|g| g.resource.uri == session_uri);
    assert!(
        !any_paired,
        "revoked legacy grants must NOT be backfilled; got grants: {lead_grants:?}"
    );
}

/// Migration 0015 must NOT backfill grants whose `descends_from` points
/// at a non-Template-A AR (e.g. Template B/C/D).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn migration_0015_skips_non_template_a_grants() {
    let acc = spawn(false).await;

    // Seed a Template C grant (kinds tag = "#template:c").
    let (_c_ar, _c_grant, lead_c, project_c) =
        seed_legacy_template_a(acc.store.as_ref(), "#template:c").await;

    // Re-run migration body.
    acc.store
        .client()
        .query("DELETE FROM _migrations WHERE version = 15")
        .await
        .unwrap();
    let _ = run_migrations(acc.store.client(), EMBEDDED_MIGRATIONS)
        .await
        .unwrap();

    // Template-C lead must NOT have a paired session_object grant.
    let c_grants = acc
        .store
        .list_grants_for_principal(&PrincipalRef::Agent(lead_c))
        .await
        .unwrap();
    let session_uri =
        format!(r#"tags contains "project:{project_c}" AND tags contains #kind:session"#);
    let any_paired = c_grants.iter().any(|g| g.resource.uri == session_uri);
    assert!(
        !any_paired,
        "Template C grants must NOT be backfilled by Template A migration"
    );
}
