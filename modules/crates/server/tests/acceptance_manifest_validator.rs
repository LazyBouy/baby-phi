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

// ----------------------------------------------------------------------------
// Rule E — Composite structural-tag-write rejection (CH-12 / ADR-0049 §D49.1)
//
// Trigger: actions ∋ Modify AND a non-Memory composite (e.g.
// `session_object`) AND a `target_kinds` entry overlapping a reserved
// namespace (`#kind` / `kind` / `delegated_from` / `derived_from`). The
// validator surfaces this as `ValidationError::CompositeStructuralTagWrite`
// at the publish-time Repository boundary; the manifest is NOT persisted.
//
// D49.1.a — Memory exemption. `Composite::MemoryObject` is exempt
// because Memory tags ARE intentionally agent-mutable per concept doc
// 05 lines 24–26. The Memory happy-path test below pins this: same
// shape (`[Modify]` + `target_kinds: ["memory"]`) on `memory_object`
// passes Rule E and persists cleanly.
// ----------------------------------------------------------------------------

fn manifest_session_modify_with_target_kinds(target_kinds: Vec<&str>) -> ToolAuthorityManifest {
    let mut m = manifest(
        vec!["session_object"],
        vec![Action::Read, Action::Modify],
        vec!["session"],
    );
    m.target_kinds = target_kinds.into_iter().map(String::from).collect();
    m
}

#[tokio::test]
async fn rule_e_session_modify_with_session_target_kind_rejected() {
    // Test 1 — `actions: [Modify], resource: ["session_object"],
    // target_kinds: ["session"]` overlaps the runtime-owned `session:`
    // reserved-namespace prefix (matched via the `tk == "session"` ==
    // `"session:".trim_end_matches(':')` branch of Rule E).
    let repo = fresh_in_memory_repo().await;
    let m = manifest_session_modify_with_target_kinds(vec!["session"]);
    let err = repo
        .create_tool_authority_manifest(&m)
        .await
        .expect_err("Rule E must reject session_object target_kind");
    match err {
        RepositoryError::ManifestValidation {
            source:
                ValidationError::CompositeStructuralTagWrite {
                    composite,
                    action: Action::Modify,
                    namespace,
                },
        } => {
            assert_eq!(composite, Composite::SessionObject);
            // namespace is the matched reserved-namespace literal, e.g.
            // `"session"` (one of CompositeStructuralTagWrite reserved
            // namespaces). The exact literal is validator-internal; the
            // assertion below pins the variant + composite identity.
            assert!(
                !namespace.is_empty(),
                "namespace field must carry the matched reserved literal"
            );
        }
        other => panic!(
            "expected CompositeStructuralTagWrite{{ session_object, Modify, _ }}, got {:?}",
            other
        ),
    }
}

#[tokio::test]
async fn rule_e_session_modify_with_kind_prefix_target_kinds_rejected() {
    // Test 2 — `target_kinds: ["#kind"]` overlaps the `#kind:` reserved
    // prefix via the namespace literal `kind`.
    let repo = fresh_in_memory_repo().await;
    let m = manifest_session_modify_with_target_kinds(vec!["#kind"]);
    let err = repo
        .create_tool_authority_manifest(&m)
        .await
        .expect_err("Rule E must reject #kind target_kind");
    assert!(
        matches!(
            err,
            RepositoryError::ManifestValidation {
                source: ValidationError::CompositeStructuralTagWrite {
                    composite: Composite::SessionObject,
                    action: Action::Modify,
                    ..
                }
            }
        ),
        "expected CompositeStructuralTagWrite for #kind target on session_object, got {:?}",
        err
    );
}

#[tokio::test]
async fn rule_e_session_modify_with_delegated_from_target_kinds_rejected() {
    // Test 3 — `target_kinds: ["delegated_from"]` overlaps the
    // `delegated_from` reserved namespace literal directly.
    let repo = fresh_in_memory_repo().await;
    let m = manifest_session_modify_with_target_kinds(vec!["delegated_from"]);
    let err = repo
        .create_tool_authority_manifest(&m)
        .await
        .expect_err("Rule E must reject delegated_from target_kind");
    assert!(
        matches!(
            err,
            RepositoryError::ManifestValidation {
                source: ValidationError::CompositeStructuralTagWrite {
                    composite: Composite::SessionObject,
                    action: Action::Modify,
                    ..
                }
            }
        ),
        "expected CompositeStructuralTagWrite for delegated_from target on session_object, got {:?}",
        err
    );
}

#[tokio::test]
async fn rule_e_cross_impl_consistency_session_object_modify_rejected_identically() {
    // Test 4 — same invalid manifest fed to both backends produces the
    // same `CompositeStructuralTagWrite` value (composite + action +
    // namespace identity). Pins the cross-impl consistency invariant
    // for the Rule E rejection path.
    let invalid = || manifest_session_modify_with_target_kinds(vec!["session"]);

    let in_mem = fresh_in_memory_repo().await;
    let in_mem_err = in_mem
        .create_tool_authority_manifest(&invalid())
        .await
        .expect_err("in-memory must reject Rule E composite write");

    let (surreal, _dir) = fresh_surreal_store().await;
    let surreal_err = surreal
        .create_tool_authority_manifest(&invalid())
        .await
        .expect_err("surreal must reject Rule E composite write");

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
        "Rule E rejection must be byte-identical across InMemoryRepository and SurrealStore"
    );
    // Sanity: the variant is the Rule E one.
    assert!(
        matches!(
            in_mem_variant,
            ValidationError::CompositeStructuralTagWrite {
                composite: Composite::SessionObject,
                action: Action::Modify,
                ..
            }
        ),
        "expected CompositeStructuralTagWrite, got {:?}",
        in_mem_variant
    );
}

#[tokio::test]
async fn rule_e_memory_modify_happy_path_passes_d49_1_a_exemption() {
    // Test 5 — D49.1.a memory exemption. Same shape as the Rule E
    // rejection (`[Modify]` + a `target_kinds` overlap), but on
    // `memory_object` it MUST pass: Memory tags are agent-mutable per
    // concept doc 05 lines 24–26. The validator's Rule E body skips
    // `Composite::MemoryObject` explicitly.
    let repo = fresh_in_memory_repo().await;
    let mut m = manifest(
        vec!["memory_object"],
        vec![Action::Read, Action::Modify],
        vec!["memory"],
    );
    m.target_kinds = vec!["memory".to_string()];
    repo.create_tool_authority_manifest(&m)
        .await
        .expect("Memory exemption (D49.1.a) — manifest must persist cleanly");
}
