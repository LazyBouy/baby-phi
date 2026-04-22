//! Credential hashing helpers — argon2id via the `argon2` crate.
//!
//! We hash the bootstrap credential (not the digest) with argon2id and
//! store the hash in `bootstrap_credentials.digest`. Verification uses
//! `argon2::PasswordVerifier` so we inherit the crate's constant-time
//! comparison.
//!
//! Per D1 in the M1 plan the plaintext is printed to stdout ONCE on
//! `phi-server bootstrap-init`; the hash is what the claim handler
//! looks up in [`Repository::find_unconsumed_credential`].

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

/// The literal prefix every generated bootstrap credential carries. The
/// claim handler strips this before hashing so a user who copies too much
/// (e.g. with a trailing newline from `echo`) still validates.
pub const BOOTSTRAP_PREFIX: &str = "bphi-bootstrap-";

/// Compute an argon2id hash of `credential` (with a fresh random salt).
///
/// The returned string is a PHC-encoded hash (includes algorithm, params,
/// salt, and digest) — exactly what we persist in
/// `bootstrap_credentials.digest`.
pub fn hash_credential(credential: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon = Argon2::default();
    let hash = argon.hash_password(credential.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

/// Verify a supplied `credential` against a stored PHC-encoded hash in
/// constant time. Returns `Ok(true)` on match, `Ok(false)` on mismatch,
/// `Err(_)` only if the stored hash is malformed (treated as an internal
/// error by callers).
pub fn verify_credential(
    credential: &str,
    hash: &str,
) -> Result<bool, argon2::password_hash::Error> {
    let parsed = PasswordHash::new(hash)?;
    match Argon2::default().verify_password(credential.as_bytes(), &parsed) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_prefix_namespaced() {
        let hash = hash_credential("hunter2").unwrap();
        assert!(hash.starts_with("$argon2id$"));
    }

    #[test]
    fn verify_accepts_original_credential() {
        let c = "bphi-bootstrap-correct-horse-battery-staple";
        let hash = hash_credential(c).unwrap();
        assert!(verify_credential(c, &hash).unwrap());
    }

    #[test]
    fn verify_rejects_wrong_credential() {
        let c = "bphi-bootstrap-correct-horse-battery-staple";
        let hash = hash_credential(c).unwrap();
        assert!(!verify_credential("bphi-bootstrap-wrong", &hash).unwrap());
    }

    #[test]
    fn two_hashes_of_same_credential_differ_but_both_verify() {
        let c = "bphi-bootstrap-xyz";
        let h1 = hash_credential(c).unwrap();
        let h2 = hash_credential(c).unwrap();
        // Different salts → different PHC strings.
        assert_ne!(h1, h2);
        // But both verify.
        assert!(verify_credential(c, &h1).unwrap());
        assert!(verify_credential(c, &h2).unwrap());
    }

    #[test]
    fn verify_rejects_malformed_hash() {
        assert!(verify_credential("anything", "not-a-phc-hash").is_err());
    }
}
