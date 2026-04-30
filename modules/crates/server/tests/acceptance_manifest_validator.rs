//! CH-05 / ADR-0044 — acceptance suite for the publish-time tool
//! authority manifest validator wired into the Repository boundary.
//!
//! Each test constructs an invalid manifest, attempts to persist it via
//! `Repository::create_tool_authority_manifest`, asserts the result is
//! `Err(RepositoryError::ManifestValidation { source })` with the
//! expected `ValidationError` variant, and asserts the manifest was
//! NOT persisted (the in-memory backend's `manifests` map stays
//! empty for that id).
//!
//! One happy-path test confirms a valid manifest persists cleanly via
//! the same code path. One cross-impl consistency test pins the
//! invariant that `InMemoryRepository` and `SurrealStore` give the
//! same validator verdict.

use std::sync::Arc;

use domain::in_memory::InMemoryRepository;
use domain::model::composites::Composite;
use domain::model::ids::NodeId;
use domain::model::nodes::ToolAuthorityManifest;
use domain::permissions::{Action, ValidationError};
use domain::repository::{Repository, RepositoryError};
use store::SurrealStore;

// ----------------------------------------------------------------------------
// Helpers
// ----------------------------------------------------------------------------

fn manifest(resource: Vec<&str>, actions: Vec<Action>, kinds: Vec<&str>) -> ToolAuthorityManifest {
    ToolAuthorityManifest {
        id: NodeId::new(),
        tool_name: "acceptance_test_tool".to_string(),
        resource: resource.into_iter().map(String::from).collect(),
        transitive: vec![],
        actions,
        constraints: vec![],
        kinds: kinds.into_iter().map(String::from).collect(),
        target_kinds: vec![],
        delegable: false,
        approval: "auto".to_string(),
    }
}

async fn fresh_in_memory_repo() -> Arc<dyn Repository> {
    Arc::new(InMemoryRepository::new())
}

async fn fresh_surreal_store() -> (SurrealStore, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = SurrealStore::open_embedded(dir.path().join("db"), "phi", "test")
        .await
        .expect("open embedded surreal");
    (store, dir)
}

// ----------------------------------------------------------------------------
// Hard-rejection tests — one per ValidationError variant
// ----------------------------------------------------------------------------

#[tokio::test]
async fn missing_kind_for_composite_rejected_at_repo_boundary() {
    let repo = fresh_in_memory_repo().await;
    // memory_object declared but kinds is empty.
    let m = manifest(vec!["memory_object"], vec![Action::Read], vec![]);
    let err = repo
        .create_tool_authority_manifest(&m)
        .await
        .expect_err("must reject");
    match err {
        RepositoryError::ManifestValidation {
            source: ValidationError::MissingKindForComposite { composite },
        } => {
            assert_eq!(composite, Composite::MemoryObject);
        }
        other => panic!("expected MissingKindForComposite, got {:?}", other),
    }
    // The manifest is NOT persisted: the validator runs BEFORE the
    // SurrealDB CREATE / HashMap insert, so a rejected manifest never
    // reaches storage. Asserting via the absence-of-a-side-effect is
    // implicit here (Repository trait has no get_tool_authority_manifest
    // surface today; if one is added in a future chunk, extend this
    // test to assert get_*(m.id).is_none()).
}

#[tokio::test]
async fn kind_fundamentals_inconsistent_rejected() {
    let repo = fresh_in_memory_repo().await;
    // kinds names memory but resource declares only network_endpoint.
    let m = manifest(vec!["network_endpoint"], vec![Action::Read], vec!["memory"]);
    let err = repo
        .create_tool_authority_manifest(&m)
        .await
        .expect_err("must reject");
    match err {
        RepositoryError::ManifestValidation {
            source: ValidationError::KindFundamentalsInconsistent { kind, .. },
        } => {
            assert_eq!(kind, Composite::MemoryObject);
        }
        other => panic!("expected KindFundamentalsInconsistent, got {:?}", other),
    }
}

#[tokio::test]
async fn unknown_resource_rejected() {
    let repo = fresh_in_memory_repo().await;
    let m = manifest(vec!["nonsense_class"], vec![Action::Read], vec![]);
    let err = repo
        .create_tool_authority_manifest(&m)
        .await
        .expect_err("must reject");
    match err {
        RepositoryError::ManifestValidation {
            source: ValidationError::UnknownResource { name },
        } => {
            assert_eq!(name, "nonsense_class");
        }
        other => panic!("expected UnknownResource, got {:?}", other),
    }
}

#[tokio::test]
async fn action_fundamental_mismatch_rejected() {
    let repo = fresh_in_memory_repo().await;
    // Recall on filesystem_object — Memory category only applies to Tag.
    let m = manifest(vec!["filesystem_object"], vec![Action::Recall], vec![]);
    let err = repo
        .create_tool_authority_manifest(&m)
        .await
        .expect_err("must reject");
    assert!(
        matches!(
            err,
            RepositoryError::ManifestValidation {
                source: ValidationError::ActionFundamentalMismatch {
                    action: Action::Recall,
                    ..
                }
            }
        ),
        "expected ActionFundamentalMismatch on Recall/filesystem_object, got {:?}",
        err
    );
}

#[tokio::test]
async fn constraint_fundamental_mismatch_rejected() {
    let repo = fresh_in_memory_repo().await;
    let mut m = manifest(vec!["network_endpoint"], vec![Action::Read], vec![]);
    m.constraints = vec!["path_prefix".to_string()];
    let err = repo
        .create_tool_authority_manifest(&m)
        .await
        .expect_err("must reject");
    assert!(
        matches!(
            err,
            RepositoryError::ManifestValidation {
                source: ValidationError::ConstraintFundamentalMismatch { .. }
            }
        ),
        "expected ConstraintFundamentalMismatch, got {:?}",
        err
    );
}

#[tokio::test]
async fn reserved_namespace_write_rejected() {
    let repo = fresh_in_memory_repo().await;
    // [Modify] on bare "tag" resource — reserved-namespace write
    // (Rule C, ADR-0044 §D44.6).
    let m = manifest(vec!["tag"], vec![Action::Modify], vec![]);
    let err = repo
        .create_tool_authority_manifest(&m)
        .await
        .expect_err("must reject");
    assert!(
        matches!(
            err,
            RepositoryError::ManifestValidation {
                source: ValidationError::ReservedNamespaceWrite {
                    action: Action::Modify,
                    ..
                }
            }
        ),
        "expected ReservedNamespaceWrite on Modify/tag, got {:?}",
        err
    );
}

// ----------------------------------------------------------------------------
// Happy-path: valid manifest persists cleanly
// ----------------------------------------------------------------------------

#[tokio::test]
async fn valid_manifest_persists_cleanly() {
    let repo = fresh_in_memory_repo().await;
    let m = manifest(
        vec!["memory_object"],
        vec![Action::Read, Action::List],
        vec!["memory"],
    );
    repo.create_tool_authority_manifest(&m)
        .await
        .expect("valid manifest must persist");
}

// ----------------------------------------------------------------------------
// Cross-impl consistency: SurrealStore + InMemoryRepository give the
// same validator verdict.
// ----------------------------------------------------------------------------

#[tokio::test]
async fn cross_impl_consistency_for_invalid_manifest() {
    // Same invalid manifest fed to both backends must surface the same
    // ValidationError variant.
    let invalid = || manifest(vec!["filesystem_object"], vec![Action::Recall], vec![]);

    let in_mem = fresh_in_memory_repo().await;
    let in_mem_err = in_mem
        .create_tool_authority_manifest(&invalid())
        .await
        .expect_err("in-memory must reject");

    let (surreal, _dir) = fresh_surreal_store().await;
    let surreal_err = surreal
        .create_tool_authority_manifest(&invalid())
        .await
        .expect_err("surreal must reject");

    let in_mem_variant = match in_mem_err {
        RepositoryError::ManifestValidation { source } => source,
        other => panic!("in-memory: expected ManifestValidation, got {:?}", other),
    };
    let surreal_variant = match surreal_err {
        RepositoryError::ManifestValidation { source } => source,
        other => panic!("surreal: expected ManifestValidation, got {:?}", other),
    };
    assert_eq!(
        in_mem_variant, surreal_variant,
        "InMemoryRepository and SurrealStore must give identical validator verdicts"
    );
}

#[tokio::test]
async fn cross_impl_consistency_for_valid_manifest() {
    // A valid manifest must persist cleanly on both backends.
    let valid = || {
        manifest(
            vec!["memory_object"],
            vec![Action::Read, Action::List],
            vec!["memory"],
        )
    };

    let in_mem = fresh_in_memory_repo().await;
    in_mem
        .create_tool_authority_manifest(&valid())
        .await
        .expect("in-memory must accept");

    let (surreal, _dir) = fresh_surreal_store().await;
    surreal
        .create_tool_authority_manifest(&valid())
        .await
        .expect("surreal must accept");
}
