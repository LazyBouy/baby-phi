//! Targeted integration test for migration `0014_authority_chain.surql`.
//!
//! Verifies the CH-14 / ADR-0053 §D53.5 schema addition is queryable
//! against a live SurrealDB instance — `auth_request.descends_from_grant`
//! is `option<string>` (accepts both NONE and a UUID-shaped string) and
//! pre-CH-14 rows round-trip cleanly via the `#[serde(default)]` shield
//! on `AuthRequest.descends_from_grant`.

use store::SurrealStore;
use tempfile::tempdir;

async fn fresh_store() -> SurrealStore {
    let dir = tempdir().expect("tempdir");
    SurrealStore::open_embedded(dir.path().join("db"), "phi", "test")
        .await
        .expect("open store")
}

/// `descends_from_grant` is `option<string>` — accepts NONE on insert.
///
/// We CREATE a row with the minimal required field set per migration
/// 0001's `auth_request` schema (`requestor_kind` + `requestor_id` +
/// `kinds` + `scope` + `state` + `submitted_at` + `resource_slots` +
/// `audit_class`) and OMIT `descends_from_grant`. The new optional
/// column must accept the absent value and read back as None.
#[tokio::test]
async fn auth_request_descends_from_grant_accepts_none() {
    let store = fresh_store().await;
    store
        .client()
        .query(
            "CREATE type::thing('auth_request', 'ar-no-parent') SET \
             requestor_kind = 'system', \
             requestor_id = 'system:genesis', \
             kinds = ['control_plane_object'], \
             scope = ['allocate'], \
             state = 'approved', \
             submitted_at = time::now(), \
             resource_slots = [], \
             audit_class = 'silent'",
        )
        .await
        .expect("insert without descends_from_grant")
        .check()
        .expect("option<string> accepts absence");

    let mut resp = store
        .client()
        .query("SELECT descends_from_grant FROM type::thing('auth_request', 'ar-no-parent')")
        .await
        .expect("select descends_from_grant");
    let parents: Vec<Option<String>> = resp
        .take((0, "descends_from_grant"))
        .expect("take descends_from_grant");
    assert_eq!(parents, vec![None], "missing column must read back as None");
}

/// `descends_from_grant` accepts an explicit UUID-shaped string (the
/// adoption-AR-side wiring path that successor chunks will populate).
#[tokio::test]
async fn auth_request_descends_from_grant_accepts_uuid_string() {
    let store = fresh_store().await;
    let parent_grant = "11111111-1111-4111-8111-111111111111";
    store
        .client()
        .query(
            "CREATE type::thing('auth_request', 'ar-with-parent') SET \
             requestor_kind = 'system', \
             requestor_id = 'system:genesis', \
             kinds = ['control_plane_object'], \
             scope = ['allocate'], \
             state = 'approved', \
             submitted_at = time::now(), \
             resource_slots = [], \
             audit_class = 'silent', \
             descends_from_grant = $g",
        )
        .bind(("g", parent_grant))
        .await
        .expect("insert with descends_from_grant")
        .check()
        .expect("option<string> accepts a UUID-shaped value");

    let mut resp = store
        .client()
        .query("SELECT descends_from_grant FROM type::thing('auth_request', 'ar-with-parent')")
        .await
        .expect("select descends_from_grant");
    let parents: Vec<Option<String>> = resp
        .take((0, "descends_from_grant"))
        .expect("take descends_from_grant");
    assert_eq!(parents, vec![Some(parent_grant.to_string())]);
}
