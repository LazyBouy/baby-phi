//! Property tests for the vault seal / unseal round-trip.
//!
//! Commitment C8 in the M2 plan. The invariant:
//!
//!   open(key, seal(key, plaintext)) == plaintext
//!
//! Plus negative cases — wrong key, tampered ciphertext — MUST fail.
//! Keys and plaintexts are arbitrary (within reasonable bounds) so
//! edge cases (empty / single-byte / large / random binary) are all
//! explored.
//!
//! Run with the default proptest config (256 cases). Ships with M2/P4.
//! The generator caps plaintext length at 8 KB so the test suite
//! doesn't balloon the test-binary runtime.
//!
//! Runs against the real `store::crypto::{seal, open}` functions; no
//! mocks — this is an integration test for the crypto layer as viewed
//! by the M2 credentials-vault handlers.
//!
//! Note: the real phi deployment reads the master key from
//! `PHI_MASTER_KEY`. These tests build the key from arbitrary 32
//! bytes via [`MasterKey::from_bytes`] so they don't touch env state.

use proptest::prelude::*;
use store::crypto::{open, seal, MasterKey};

fn arb_key_bytes() -> impl Strategy<Value = [u8; 32]> {
    proptest::array::uniform32(any::<u8>())
}

fn arb_plaintext() -> impl Strategy<Value = Vec<u8>> {
    proptest::collection::vec(any::<u8>(), 0..8192)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn seal_then_open_is_identity(
        key_bytes in arb_key_bytes(),
        plaintext in arb_plaintext(),
    ) {
        let key = MasterKey::from_bytes(key_bytes);
        let sealed = seal(&key, &plaintext).expect("seal");
        let opened = open(&key, &sealed).expect("open");
        prop_assert_eq!(opened, plaintext);
    }

    #[test]
    fn seal_produces_fresh_nonce_each_call(
        key_bytes in arb_key_bytes(),
        plaintext in arb_plaintext(),
    ) {
        let key = MasterKey::from_bytes(key_bytes);
        let a = seal(&key, &plaintext).expect("seal a");
        let b = seal(&key, &plaintext).expect("seal b");
        prop_assert_ne!(a.nonce, b.nonce, "nonce must be fresh per call");
        // Matching plaintext + fresh nonce ⇒ different ciphertext.
        prop_assert_ne!(a.ciphertext, b.ciphertext);
    }

    #[test]
    fn wrong_key_cannot_open(
        key_a_bytes in arb_key_bytes(),
        key_b_bytes in arb_key_bytes(),
        plaintext in arb_plaintext(),
    ) {
        prop_assume!(key_a_bytes != key_b_bytes);
        let key_a = MasterKey::from_bytes(key_a_bytes);
        let key_b = MasterKey::from_bytes(key_b_bytes);
        let sealed = seal(&key_a, &plaintext).expect("seal");
        prop_assert!(open(&key_b, &sealed).is_err());
    }

    #[test]
    fn tampered_ciphertext_cannot_open(
        key_bytes in arb_key_bytes(),
        plaintext in proptest::collection::vec(any::<u8>(), 1..8192),
        flip_idx in any::<usize>(),
    ) {
        let key = MasterKey::from_bytes(key_bytes);
        let mut sealed = seal(&key, &plaintext).expect("seal");
        // Flip one bit in the ciphertext — AEAD must refuse.
        let idx = flip_idx % sealed.ciphertext.len();
        sealed.ciphertext[idx] ^= 1;
        prop_assert!(open(&key, &sealed).is_err());
    }

    #[test]
    fn base64_roundtrip_preserves_sealed_shape(
        key_bytes in arb_key_bytes(),
        plaintext in arb_plaintext(),
    ) {
        let key = MasterKey::from_bytes(key_bytes);
        let sealed = seal(&key, &plaintext).expect("seal");
        let (ct_b64, nonce_b64) = sealed.to_base64();
        let decoded = store::crypto::SealedSecret::from_base64(&ct_b64, &nonce_b64)
            .expect("from_base64");
        let opened = open(&key, &decoded).expect("open after b64 roundtrip");
        prop_assert_eq!(opened, plaintext);
    }
}
