//! Integration test for the at-rest encryption path end-to-end against a
//! real SurrealDB: seal a plaintext, persist it into `secrets_vault`,
//! re-read it, and decrypt. Verifies C8 (at-rest encryption) from the M1
//! plan.

use store::{seal_plaintext, MasterKey, SealedSecret, SurrealStore};
use tempfile::tempdir;

#[tokio::test]
async fn seal_persist_read_open_roundtrip() {
    let dir = tempdir().expect("tempdir");
    let store = SurrealStore::open_embedded(dir.path().join("db"), "baby-phi", "test")
        .await
        .expect("open store");

    let key = MasterKey::from_bytes([0xAB; 32]);
    let sealed = seal_plaintext(&key, b"sk-live-xxxxxxxxxxxxxxxxxxxxxx").expect("seal");
    let (ct_b64, nonce_b64) = sealed.to_base64();

    store
        .client()
        .query(
            "CREATE secrets_vault SET \
                slug = $slug, \
                custodian_id = $custodian, \
                sensitive = true, \
                value_ciphertext_b64 = $ct, \
                nonce_b64 = $nonce, \
                created_at = time::now(), \
                last_rotated_at = NONE",
        )
        .bind(("slug", "test-openrouter"))
        .bind(("custodian", uuid::Uuid::new_v4().to_string()))
        .bind(("ct", ct_b64))
        .bind(("nonce", nonce_b64))
        .await
        .expect("persist secret")
        .check()
        .expect("check persist");

    // Read it back and rebuild the SealedSecret.
    let mut resp = store
        .client()
        .query("SELECT value_ciphertext_b64, nonce_b64 FROM secrets_vault WHERE slug = $slug")
        .bind(("slug", "test-openrouter"))
        .await
        .expect("query secret");

    let ct_strs: Vec<String> = resp.take((0, "value_ciphertext_b64")).expect("ciphertext");
    let nonce_strs: Vec<String> = resp.take((0, "nonce_b64")).expect("nonce");
    assert_eq!(ct_strs.len(), 1);
    assert_eq!(nonce_strs.len(), 1);

    let rehydrated =
        SealedSecret::from_base64(&ct_strs[0], &nonce_strs[0]).expect("base64 rehydrate");
    let plaintext = store::open_sealed(&key, &rehydrated).expect("open");
    assert_eq!(plaintext, b"sk-live-xxxxxxxxxxxxxxxxxxxxxx");
}
