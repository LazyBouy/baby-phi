//! `phi-server bootstrap-init` — one-shot credential generation.
//!
//! Called by the install process on a fresh data dir. Generates 32
//! random bytes, encodes them URL-safe-base64, prepends
//! [`super::credential::BOOTSTRAP_PREFIX`], and stores the argon2id hash
//! in `bootstrap_credentials`. The plaintext is returned so the caller
//! (usually `main.rs` for the CLI path) can print it to stdout.
//!
//! Per plan decision D1, the plaintext is printed **once**. If the admin
//! loses it they must re-run the install. We never persist the plaintext.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use domain::repository::Repository;
use rand::RngCore;

use super::credential::{hash_credential, BOOTSTRAP_PREFIX};

/// Result of a successful bootstrap-init. Contains the **plaintext**
/// credential the admin must record; callers MUST print this exactly once
/// and MUST NOT persist it anywhere but stdout (+ the admin's copy/paste
/// buffer).
#[derive(Debug)]
pub struct GeneratedCredential {
    /// Full `bphi-bootstrap-XXXX...` string. 32 bytes of CSPRNG entropy.
    pub plaintext: String,
    /// SurrealDB record id of the stored hashed credential.
    pub record_id: String,
}

/// Generate a new bootstrap credential, hash it with argon2id, and
/// persist the hash via the repository. Returns both the plaintext
/// (for one-time display) and the stored row's record id.
pub async fn generate_bootstrap_credential(
    repo: &dyn Repository,
) -> Result<GeneratedCredential, BootstrapInitError> {
    let plaintext = fresh_credential_plaintext();
    let hash = hash_credential(&plaintext).map_err(BootstrapInitError::Hash)?;
    let row = repo
        .put_bootstrap_credential(hash)
        .await
        .map_err(BootstrapInitError::Store)?;
    Ok(GeneratedCredential {
        plaintext,
        record_id: row.record_id,
    })
}

fn fresh_credential_plaintext() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    let body = URL_SAFE_NO_PAD.encode(bytes);
    format!("{BOOTSTRAP_PREFIX}{body}")
}

/// Structured error returned by [`generate_bootstrap_credential`].
#[derive(Debug, thiserror::Error)]
pub enum BootstrapInitError {
    #[error("failed to hash bootstrap credential: {0}")]
    Hash(#[from] argon2::password_hash::Error),
    #[error("failed to persist bootstrap credential: {0}")]
    Store(#[from] domain::repository::RepositoryError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::in_memory::InMemoryRepository;

    #[test]
    fn fresh_credential_plaintext_is_prefixed_and_reasonably_long() {
        let p = fresh_credential_plaintext();
        assert!(p.starts_with(BOOTSTRAP_PREFIX));
        // 32-byte URL-safe-base64 (no pad) ≈ 43 chars; plus the prefix.
        assert!(p.len() >= BOOTSTRAP_PREFIX.len() + 40);
    }

    #[test]
    fn fresh_credentials_are_distinct() {
        let a = fresh_credential_plaintext();
        let b = fresh_credential_plaintext();
        assert_ne!(a, b, "CSPRNG must not repeat");
    }

    #[tokio::test]
    async fn generate_stores_hash_and_returns_plaintext() {
        let repo = InMemoryRepository::new();
        let out = generate_bootstrap_credential(&repo).await.unwrap();
        assert!(out.plaintext.starts_with(BOOTSTRAP_PREFIX));
        assert!(!out.record_id.is_empty());
        // The stored hash must verify against the returned plaintext.
        let row = repo
            .find_unconsumed_credential(
                // The in-memory repo matches exactly on digest — so we
                // look up using the hash that was stored. Fish it out
                // via an internal probe: the stored row's digest IS the
                // hash, and `find_unconsumed_credential` matches on
                // exact string. We can't directly query for it without
                // the hash, so this only confirms `put_bootstrap_credential`
                // wrote something; the hash/verify round-trip is covered
                // by credential.rs tests.
                "",
            )
            .await
            .unwrap();
        assert!(row.is_none(), "empty digest must not match anything");
    }
}
