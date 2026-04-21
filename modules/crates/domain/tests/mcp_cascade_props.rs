//! Property tests for M2/P6: `Repository::narrow_mcp_tenants` cascade
//! semantics.
//!
//! Three invariants:
//!
//! 1. **Monotonic grant count.** Narrowing `tenants_allowed` cannot
//!    INCREASE the count of live (non-revoked) grants. The count either
//!    stays the same (no dropped org had grants descending from it) or
//!    strictly decreases.
//!
//! 2. **Narrow idempotence.** Calling `narrow_mcp_tenants` twice with
//!    the same `new_allowed` returns a non-empty cascade only the first
//!    time; the second call returns an empty `Vec<TenantRevocation>`
//!    because every affected grant is already revoked.
//!
//! 3. **No over-revocation.** Grants whose provenance `descends_from`
//!    an AR requested by an org still in `new_allowed` remain live
//!    after the cascade.

#![cfg(feature = "in-memory-repo")]

use chrono::Utc;
use proptest::collection::vec;
use proptest::prelude::*;

use domain::audit::AuditClass;
use domain::in_memory::InMemoryRepository;
use domain::model::ids::{AgentId, AuthRequestId, GrantId, McpServerId, OrgId};
use domain::model::nodes::{
    AuthRequest, AuthRequestState, Grant, PrincipalRef, ResourceRef, ResourceSlot,
    ResourceSlotState,
};
use domain::model::{ExternalService, ExternalServiceKind, Fundamental, RuntimeStatus, TenantSet};
use domain::repository::Repository;

// ----------------------------------------------------------------------------
// Fixture helpers
// ----------------------------------------------------------------------------

fn mcp_server(id: McpServerId, tenants: TenantSet) -> ExternalService {
    ExternalService {
        id,
        display_name: "test-mcp".into(),
        kind: ExternalServiceKind::Mcp,
        endpoint: "stdio:///cmd".into(),
        secret_ref: None,
        tenants_allowed: tenants,
        status: RuntimeStatus::Ok,
        archived_at: None,
        created_at: Utc::now(),
    }
}

/// Construct an approved AR with the org as requestor. Uses the
/// minimum fields required by the in-memory store.
fn ar_for_org(id: AuthRequestId, org: OrgId) -> AuthRequest {
    AuthRequest {
        id,
        requestor: PrincipalRef::Organization(org),
        kinds: vec!["#kind:external_service".into()],
        scope: vec!["external_service:stub".into()],
        state: AuthRequestState::Approved,
        valid_until: None,
        submitted_at: Utc::now(),
        resource_slots: vec![ResourceSlot {
            resource: ResourceRef {
                uri: "external_service:stub".into(),
            },
            approvers: vec![],
            state: ResourceSlotState::Approved,
        }],
        justification: Some("proptest fixture".into()),
        audit_class: AuditClass::Alerted,
        terminal_state_entered_at: None,
        archived: false,
        active_window_days: 90,
        provenance_template: None,
    }
}

fn grant_for_ar(holder: AgentId, ar_id: AuthRequestId) -> Grant {
    Grant {
        id: GrantId::new(),
        holder: PrincipalRef::Agent(holder),
        action: vec!["invoke".into()],
        resource: ResourceRef {
            uri: "external_service:stub".into(),
        },
        fundamentals: vec![Fundamental::NetworkEndpoint],
        descends_from: Some(ar_id),
        delegable: false,
        issued_at: Utc::now(),
        revoked_at: None,
    }
}

/// Generate a unique-org vector of size `n` by constructing fresh OrgIds.
fn fresh_orgs(n: usize) -> Vec<OrgId> {
    (0..n).map(|_| OrgId::new()).collect()
}

// ----------------------------------------------------------------------------
// Invariant 1 — monotonic grant count
// ----------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig { cases: 32, ..ProptestConfig::default() })]

    #[test]
    fn narrowing_never_increases_live_grant_count(
        org_count in 2usize..5,
        grants_per_org in 1usize..4,
        drop_count in 1usize..4,
    ) {
        prop_assume!(drop_count < org_count);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build runtime");

        rt.block_on(async move {
            let orgs = fresh_orgs(org_count);
            let repo = InMemoryRepository::new();
            let holder = AgentId::new();

            // Seed one AR + grants_per_org grants per org.
            for org in &orgs {
                let ar_id = AuthRequestId::new();
                let ar = ar_for_org(ar_id, *org);
                repo.create_auth_request(&ar).await.unwrap();
                for _ in 0..grants_per_org {
                    let g = grant_for_ar(holder, ar_id);
                    repo.create_grant(&g).await.unwrap();
                }
            }

            let mcp_id = McpServerId::new();
            let svc = mcp_server(mcp_id, TenantSet::Only(orgs.clone()));
            repo.put_mcp_server(&svc).await.unwrap();

            // Count live grants BEFORE.
            let before = repo
                .list_grants_for_principal(&PrincipalRef::Agent(holder))
                .await
                .unwrap();
            let before_live = before.iter().filter(|g| g.revoked_at.is_none()).count();

            // Drop the first `drop_count` orgs.
            let kept: Vec<OrgId> = orgs.iter().skip(drop_count).copied().collect();
            let cascade = repo
                .narrow_mcp_tenants(mcp_id, &TenantSet::Only(kept.clone()), Utc::now())
                .await
                .unwrap();

            // Count live grants AFTER.
            let after = repo
                .list_grants_for_principal(&PrincipalRef::Agent(holder))
                .await
                .unwrap();
            let after_live = after.iter().filter(|g| g.revoked_at.is_none()).count();

            prop_assert!(
                after_live <= before_live,
                "live grant count increased: before={before_live} after={after_live}"
            );

            // Each dropped org should have contributed `grants_per_org` revocations.
            let expected_drop = drop_count * grants_per_org;
            prop_assert_eq!(
                before_live - after_live,
                expected_drop,
                "unexpected revocation count: expected {} got {}",
                expected_drop,
                before_live - after_live
            );

            // And the cascade vec should have exactly `drop_count`
            // TenantRevocation entries (one per dropped org).
            prop_assert_eq!(cascade.len(), drop_count);
            Ok(())
        }).unwrap();
    }
}

// ----------------------------------------------------------------------------
// Invariant 2 — narrow idempotence
// ----------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig { cases: 24, ..ProptestConfig::default() })]

    #[test]
    fn narrow_twice_is_empty_on_second_call(
        org_count in 2usize..4,
        grants_per_org in 1usize..3,
    ) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build runtime");

        rt.block_on(async move {
            let orgs = fresh_orgs(org_count);
            let repo = InMemoryRepository::new();
            let holder = AgentId::new();
            for org in &orgs {
                let ar_id = AuthRequestId::new();
                repo.create_auth_request(&ar_for_org(ar_id, *org))
                    .await
                    .unwrap();
                for _ in 0..grants_per_org {
                    repo.create_grant(&grant_for_ar(holder, ar_id))
                        .await
                        .unwrap();
                }
            }

            let mcp_id = McpServerId::new();
            let svc = mcp_server(mcp_id, TenantSet::Only(orgs.clone()));
            repo.put_mcp_server(&svc).await.unwrap();

            // Drop the first org.
            let kept: Vec<OrgId> = orgs.iter().skip(1).copied().collect();
            let cascade1 = repo
                .narrow_mcp_tenants(mcp_id, &TenantSet::Only(kept.clone()), Utc::now())
                .await
                .unwrap();
            let cascade2 = repo
                .narrow_mcp_tenants(mcp_id, &TenantSet::Only(kept), Utc::now())
                .await
                .unwrap();

            prop_assert!(!cascade1.is_empty(), "first narrow must affect grants");
            prop_assert!(
                cascade2.is_empty(),
                "second narrow with same target must be a no-op: got {:?}",
                cascade2
            );
            Ok(())
        }).unwrap();
    }
}

// ----------------------------------------------------------------------------
// Invariant 3 — no over-revocation
// ----------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig { cases: 24, ..ProptestConfig::default() })]

    #[test]
    fn grants_from_surviving_orgs_stay_live(
        org_count in 3usize..6,
        drop_indices in vec(any::<bool>(), 3..6),
    ) {
        // Align lengths — proptest may hand us a drop mask longer
        // or shorter than org_count; truncate to match.
        let drop_mask: Vec<bool> = drop_indices.into_iter().take(org_count).collect();
        prop_assume!(drop_mask.len() == org_count);
        prop_assume!(drop_mask.iter().any(|d| *d));   // at least one drop
        prop_assume!(drop_mask.iter().any(|d| !*d));  // at least one keep

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build runtime");

        rt.block_on(async move {
            let orgs = fresh_orgs(org_count);
            let repo = InMemoryRepository::new();
            let holder = AgentId::new();

            // Track which grant-ids descended from which org.
            let mut grants_by_org: std::collections::HashMap<OrgId, Vec<GrantId>> =
                std::collections::HashMap::new();

            for org in &orgs {
                let ar_id = AuthRequestId::new();
                repo.create_auth_request(&ar_for_org(ar_id, *org)).await.unwrap();
                let g = grant_for_ar(holder, ar_id);
                let gid = g.id;
                repo.create_grant(&g).await.unwrap();
                grants_by_org.entry(*org).or_default().push(gid);
            }

            let mcp_id = McpServerId::new();
            repo.put_mcp_server(&mcp_server(mcp_id, TenantSet::Only(orgs.clone())))
                .await
                .unwrap();

            let kept: Vec<OrgId> = orgs
                .iter()
                .zip(drop_mask.iter())
                .filter_map(|(o, drop)| if *drop { None } else { Some(*o) })
                .collect();

            repo.narrow_mcp_tenants(mcp_id, &TenantSet::Only(kept.clone()), Utc::now())
                .await
                .unwrap();

            // Every kept-org grant must remain live; every dropped-org grant must be revoked.
            let all_grants = repo
                .list_grants_for_principal(&PrincipalRef::Agent(holder))
                .await
                .unwrap();
            let live: std::collections::HashSet<GrantId> = all_grants
                .iter()
                .filter(|g| g.revoked_at.is_none())
                .map(|g| g.id)
                .collect();

            for (org, drop) in orgs.iter().zip(drop_mask.iter()) {
                for gid in grants_by_org.get(org).unwrap() {
                    if *drop {
                        prop_assert!(!live.contains(gid),
                            "grant from dropped org {} should be revoked", org);
                    } else {
                        prop_assert!(live.contains(gid),
                            "grant from kept org {} was wrongly revoked", org);
                    }
                }
            }
            Ok(())
        }).unwrap();
    }
}
