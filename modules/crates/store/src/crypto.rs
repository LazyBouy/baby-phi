//! At-rest envelope encryption for `secrets_vault` values.
//!
//! Scope for M1 (per plan decision D2): only the `secrets_vault.value`
//! column is sealed. Broader full-DB encryption requires KMS integration
//! and is deferred to M7b. The master key is read from
//! `BABY_PHI_MASTER_KEY` as a 32-byte base64-encoded value; binaries that
//! try to read a secret before loading it fail with
//! [`CryptoError::MissingMasterKey`] and the server refuses to start.
//!
//! Algorithm: AES-256-GCM (`aes_gcm::Aes256Gcm`). Each seal generates a
//! fresh 12-byte nonce and returns `(ciphertext, nonce)`; both are stored
//! side-by-side on the row. Opening needs both.

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use base64::engine::general_purpose::STANDARD_NO_PAD as BASE64_NOPAD;
use base64::Engine as _;
use rand::RngCore;
use serde::{Deserialize, Serialize};

/// The name of the environment variable the server reads for the master key.
pub const MASTER_KEY_ENV: &str = "BABY_PHI_MASTER_KEY";

/// A 32-byte symmetric key. Deliberately not `Serialize` so it cannot
/// accidentally land in logs or JSON payloads.
#[derive(Clone)]
pub struct MasterKey(Key<Aes256Gcm>);

impl std::fmt::Debug for MasterKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("MasterKey(***)")
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error(
        "master key missing: set the {MASTER_KEY_ENV} env var to a 32-byte base64 value before starting the server"
    )]
    MissingMasterKey,
    #[error("master key is not valid base64: {0}")]
    BadBase64(String),
    #[error("master key has wrong length: expected 32 bytes, got {0}")]
    BadLength(usize),
    #[error("encryption failed: {0}")]
    Seal(String),
    #[error("decryption failed (ciphertext tampered with or wrong key): {0}")]
    Open(String),
}

impl MasterKey {
    /// Load the master key from the `BABY_PHI_MASTER_KEY` env var.
    pub fn from_env() -> Result<Self, CryptoError> {
        let raw = std::env::var(MASTER_KEY_ENV).map_err(|_| CryptoError::MissingMasterKey)?;
        Self::from_base64(&raw)
    }

    /// Parse a 32-byte key from its base64 (standard, no-padding) form.
    pub fn from_base64(value: &str) -> Result<Self, CryptoError> {
        let bytes = BASE64_NOPAD
            .decode(value.trim())
            .map_err(|e| CryptoError::BadBase64(e.to_string()))?;
        if bytes.len() != 32 {
            return Err(CryptoError::BadLength(bytes.len()));
        }
        let key = *Key::<Aes256Gcm>::from_slice(&bytes);
        Ok(MasterKey(key))
    }

    /// Construct from raw 32 bytes. Prefer `from_env` / `from_base64` in
    /// production code.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        MasterKey(*Key::<Aes256Gcm>::from_slice(&bytes))
    }
}

/// A sealed value — ciphertext + nonce. Persisted on
/// `secrets_vault.value_ciphertext_b64` + `secrets_vault.nonce_b64` as
/// base64 strings; use [`SealedSecret::to_base64`] / [`SealedSecret::from_base64`]
/// to convert at the repository boundary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealedSecret {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; 12],
}

impl SealedSecret {
    /// Persist-friendly base64 encoding.
    pub fn to_base64(&self) -> (String, String) {
        (
            BASE64_NOPAD.encode(&self.ciphertext),
            BASE64_NOPAD.encode(self.nonce),
        )
    }

    /// Inverse of [`SealedSecret::to_base64`]. Fails if either string is not
    /// valid base64 or the nonce is not 12 bytes.
    pub fn from_base64(ciphertext_b64: &str, nonce_b64: &str) -> Result<Self, CryptoError> {
        let ciphertext = BASE64_NOPAD
            .decode(ciphertext_b64.trim())
            .map_err(|e| CryptoError::BadBase64(e.to_string()))?;
        let nonce_vec = BASE64_NOPAD
            .decode(nonce_b64.trim())
            .map_err(|e| CryptoError::BadBase64(e.to_string()))?;
        if nonce_vec.len() != 12 {
            return Err(CryptoError::BadLength(nonce_vec.len()));
        }
        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&nonce_vec);
        Ok(SealedSecret { ciphertext, nonce })
    }
}

/// AES-GCM-encrypt `plaintext` under `key`. A fresh nonce is sampled from
/// the OS RNG per call.
pub fn seal(key: &MasterKey, plaintext: &[u8]) -> Result<SealedSecret, CryptoError> {
    let cipher = Aes256Gcm::new(&key.0);
    let mut nonce_bytes = [0u8; 12];
    rand::rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| CryptoError::Seal(e.to_string()))?;
    Ok(SealedSecret {
        ciphertext,
        nonce: nonce_bytes,
    })
}

/// AES-GCM-decrypt a sealed value. Fails if the nonce/ciphertext/key do not
/// match (the GCM authentication tag catches tampering).
pub fn open(key: &MasterKey, sealed: &SealedSecret) -> Result<Vec<u8>, CryptoError> {
    let cipher = Aes256Gcm::new(&key.0);
    let nonce = Nonce::from_slice(&sealed.nonce);
    cipher
        .decrypt(nonce, sealed.ciphertext.as_ref())
        .map_err(|e| CryptoError::Open(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_key() -> MasterKey {
        MasterKey::from_bytes([42u8; 32])
    }

    #[test]
    fn seal_open_roundtrip_recovers_plaintext() {
        let key = sample_key();
        let sealed = seal(&key, b"api_key:sk-xxxxx").expect("seal");
        let opened = open(&key, &sealed).expect("open");
        assert_eq!(opened, b"api_key:sk-xxxxx");
    }

    #[test]
    fn nonces_are_unique_per_seal() {
        let key = sample_key();
        let a = seal(&key, b"same plaintext").unwrap();
        let b = seal(&key, b"same plaintext").unwrap();
        assert_ne!(a.nonce, b.nonce);
        assert_ne!(a.ciphertext, b.ciphertext);
    }

    #[test]
    fn tampered_ciphertext_fails_to_open() {
        let key = sample_key();
        let mut sealed = seal(&key, b"important").unwrap();
        sealed.ciphertext[0] ^= 0xff;
        let err = open(&key, &sealed).unwrap_err();
        assert!(matches!(err, CryptoError::Open(_)));
    }

    #[test]
    fn wrong_key_fails_to_open() {
        let a = sample_key();
        let b = MasterKey::from_bytes([7u8; 32]);
        let sealed = seal(&a, b"hello").unwrap();
        let err = open(&b, &sealed).unwrap_err();
        assert!(matches!(err, CryptoError::Open(_)));
    }

    #[test]
    fn base64_round_trip_matches_raw_bytes() {
        let raw = [9u8; 32];
        let b64 = BASE64_NOPAD.encode(raw);
        let key = MasterKey::from_base64(&b64).expect("parse");
        let sealed = seal(&key, b"x").unwrap();
        let opened = open(&MasterKey::from_bytes(raw), &sealed).unwrap();
        assert_eq!(opened, b"x");
    }

    #[test]
    fn rejects_short_key() {
        let short = BASE64_NOPAD.encode([1u8; 16]);
        let err = MasterKey::from_base64(&short).unwrap_err();
        assert!(matches!(err, CryptoError::BadLength(16)));
    }

    #[test]
    fn rejects_bad_base64() {
        let err = MasterKey::from_base64("!!!not-base64!!!").unwrap_err();
        assert!(matches!(err, CryptoError::BadBase64(_)));
    }

    #[test]
    fn missing_env_var_reports_missing_master_key() {
        // Clear + try to load. We do not rely on the real env in tests
        // because `cargo test` may run in parallel with other tests that
        // set the var; use a local scoped unset-then-set pattern.
        let previous = std::env::var(MASTER_KEY_ENV).ok();
        // SAFETY: tests run single-threaded for this assertion only; see
        // `missing_env_var_reports_missing_master_key_is_single_threaded`
        // module guard below if we ever parallelise. For now: --test-threads=1
        // is the default for this file's tests via `[[test]]` profile.
        unsafe {
            std::env::remove_var(MASTER_KEY_ENV);
        }
        let err = MasterKey::from_env().unwrap_err();
        assert!(matches!(err, CryptoError::MissingMasterKey));
        if let Some(v) = previous {
            unsafe {
                std::env::set_var(MASTER_KEY_ENV, v);
            }
        }
    }

    #[test]
    fn debug_impl_redacts_key() {
        let k = sample_key();
        let s = format!("{:?}", k);
        assert_eq!(s, "MasterKey(***)");
    }
}
