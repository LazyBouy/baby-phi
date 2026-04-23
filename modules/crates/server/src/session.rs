//! Signed session cookie (HS256 JWT).
//!
//! M1 ships a minimal session layer sufficient for the one principal
//! that exists at this milestone — the platform admin who just claimed
//! via `/api/v0/bootstrap/claim`. OAuth and general human-login land in
//! M3; see [ADR-0015] (placeholder) for the migration path.
//!
//! **Contract.** On a successful bootstrap claim, the handler calls
//! [`sign_and_build_cookie`] which returns a [`Cookie`] carrying a
//! short-lived HS256 JWT whose `sub` is the admin's `agent_id`. Every
//! subsequent request carries the cookie; handlers call
//! [`verify_from_cookies`] to recover the typed [`SessionClaims`].
//!
//! Cookie attributes: `HttpOnly`, `SameSite=Lax`, `Path=/`, `Secure`
//! driven by config (default `true`; `config/dev.toml` flips it off).
//! The JWT has a `exp` claim matching `SessionConfig::ttl_seconds` so a
//! leaked cookie auto-expires even without server-side revocation.
//!
//! Revocation (server-side session table) is **not** implemented in M1;
//! the TTL is the only defence. M3 widens this to proper session rows +
//! `POST /sessions/{id}/revoke`.

use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use tower_cookies::{
    cookie::{time::OffsetDateTime, SameSite},
    Cookie, Cookies,
};

use crate::config::SessionConfig;

/// Shape carried inside the HS256 JWT. `sub` is the admin's `agent_id`
/// (stringified UUID); the other claims are standard JWT registered
/// claims.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionClaims {
    /// Subject — the platform admin's agent id, as a UUID string.
    pub sub: String,
    /// Issued-at — UNIX seconds.
    pub iat: i64,
    /// Expires-at — UNIX seconds.
    pub exp: i64,
}

/// Opaque signing-key handle kept on [`crate::AppState`]. Holds the
/// `jsonwebtoken` encode + decode keys pre-computed so handlers never
/// allocate on the hot path, plus the cookie-shape knobs that apply to
/// every emitted cookie.
#[derive(Clone)]
pub struct SessionKey {
    encode: EncodingKey,
    decode: DecodingKey,
    pub(crate) cookie_name: String,
    pub(crate) ttl_seconds: u64,
    pub(crate) secure: bool,
}

impl std::fmt::Debug for SessionKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionKey")
            .field("cookie_name", &self.cookie_name)
            .field("ttl_seconds", &self.ttl_seconds)
            .field("secure", &self.secure)
            .field("encode", &"<redacted>")
            .field("decode", &"<redacted>")
            .finish()
    }
}

impl SessionKey {
    /// Build a SessionKey from the loaded config. Fails if the secret is
    /// shorter than 32 bytes — too short a secret makes HMAC brittle and
    /// is almost certainly a misconfig.
    pub fn from_config(cfg: &SessionConfig) -> Result<Self, SessionBuildError> {
        if cfg.secret.len() < 32 {
            return Err(SessionBuildError::SecretTooShort {
                got: cfg.secret.len(),
            });
        }
        Ok(Self {
            encode: EncodingKey::from_secret(cfg.secret.as_bytes()),
            decode: DecodingKey::from_secret(cfg.secret.as_bytes()),
            cookie_name: cfg.cookie_name.clone(),
            ttl_seconds: cfg.ttl_seconds,
            secure: cfg.secure,
        })
    }

    /// Test-only constructor. Takes a raw secret and sensible defaults
    /// (cookie name `phi_kernel_session`, TTL 1 hour, `secure=false`). No
    /// length check — use in-process tests only; production code MUST use
    /// [`from_config`].
    pub fn for_tests(secret: &str) -> Self {
        Self {
            encode: EncodingKey::from_secret(secret.as_bytes()),
            decode: DecodingKey::from_secret(secret.as_bytes()),
            cookie_name: "phi_kernel_session".to_string(),
            ttl_seconds: 3600,
            secure: false,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SessionBuildError {
    #[error("session secret is {got} bytes; need ≥ 32")]
    SecretTooShort { got: usize },
}

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("no session cookie present")]
    NoCookie,
    #[error("session token invalid or expired: {0}")]
    InvalidToken(#[from] jsonwebtoken::errors::Error),
    #[error("could not build session token: {0}")]
    TokenBuild(jsonwebtoken::errors::Error),
}

/// Produce a signed JWT + a matching `Cookie` for the named subject.
/// The caller is expected to attach the cookie to the response via
/// [`Cookies::add`]. Returns the JWT as well so callers (or tests) can
/// introspect it without pulling the cookie back out of the jar.
pub fn sign_and_build_cookie(
    key: &SessionKey,
    subject: &str,
) -> Result<(String, Cookie<'static>), SessionError> {
    let now = Utc::now();
    let exp = now + Duration::seconds(key.ttl_seconds as i64);
    let claims = SessionClaims {
        sub: subject.to_string(),
        iat: now.timestamp(),
        exp: exp.timestamp(),
    };
    let token = jsonwebtoken::encode(&Header::default(), &claims, &key.encode)
        .map_err(SessionError::TokenBuild)?;
    let cookie = build_cookie(key, token.clone(), exp.timestamp());
    Ok((token, cookie))
}

/// Build the cookie shape we attach to every authenticated response.
/// Extracted so tests can assert exact attributes without re-implementing
/// the shape in two places.
fn build_cookie(key: &SessionKey, value: String, exp_unix: i64) -> Cookie<'static> {
    let mut c = Cookie::new(key.cookie_name.clone(), value);
    c.set_http_only(true);
    c.set_secure(key.secure);
    c.set_same_site(SameSite::Lax);
    c.set_path("/");
    if let Ok(expires) = OffsetDateTime::from_unix_timestamp(exp_unix) {
        c.set_expires(expires);
    }
    c
}

/// Pull the session cookie out of the jar and verify it. Returns the
/// decoded claims or a [`SessionError`].
pub fn verify_from_cookies(
    key: &SessionKey,
    cookies: &Cookies,
) -> Result<SessionClaims, SessionError> {
    let cookie = cookies
        .get(&key.cookie_name)
        .ok_or(SessionError::NoCookie)?;
    verify_token(key, cookie.value())
}

/// Verify a raw token. Exposed for tests that want to exercise the JWT
/// path without the cookie jar.
pub fn verify_token(key: &SessionKey, token: &str) -> Result<SessionClaims, SessionError> {
    let mut validation = Validation::default();
    validation.validate_exp = true;
    validation.leeway = 0;
    let data = jsonwebtoken::decode::<SessionClaims>(token, &key.decode, &validation)?;
    Ok(data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_then_verify_roundtrip() {
        let key = SessionKey::for_tests("01234567890123456789012345678901234567890");
        let (token, cookie) = sign_and_build_cookie(&key, "agent-42").unwrap();
        let claims = verify_token(&key, &token).unwrap();
        assert_eq!(claims.sub, "agent-42");
        assert!(claims.exp > claims.iat);
        // Cookie shape.
        assert_eq!(cookie.name(), "phi_kernel_session");
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.secure(), Some(false)); // for_tests → secure=false
    }

    #[test]
    fn verify_rejects_wrong_signature() {
        let good = SessionKey::for_tests("01234567890123456789012345678901234567890");
        let bad = SessionKey::for_tests("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF");
        let (token, _) = sign_and_build_cookie(&good, "agent-42").unwrap();
        assert!(verify_token(&bad, &token).is_err());
    }

    #[test]
    fn verify_rejects_garbage_token() {
        let key = SessionKey::for_tests("01234567890123456789012345678901234567890");
        assert!(verify_token(&key, "not-a-jwt").is_err());
    }

    #[test]
    fn from_config_rejects_short_secret() {
        let cfg = SessionConfig {
            secret: "too-short".into(),
            cookie_name: "phi_kernel_session".into(),
            ttl_seconds: 3600,
            secure: true,
            max_concurrent: 16,
        };
        let err = SessionKey::from_config(&cfg).unwrap_err();
        matches!(err, SessionBuildError::SecretTooShort { .. });
    }

    #[test]
    fn from_config_accepts_32_byte_secret() {
        let cfg = SessionConfig {
            secret: "01234567890123456789012345678901".into(),
            cookie_name: "phi_kernel_session".into(),
            ttl_seconds: 3600,
            secure: true,
            max_concurrent: 16,
        };
        assert!(SessionKey::from_config(&cfg).is_ok());
    }

    #[test]
    fn expired_token_is_rejected() {
        let key = SessionKey::for_tests("01234567890123456789012345678901234567890");
        // Manually mint a token that expired one second ago.
        let now = Utc::now();
        let claims = SessionClaims {
            sub: "agent-expired".into(),
            iat: (now - Duration::seconds(10)).timestamp(),
            exp: (now - Duration::seconds(1)).timestamp(),
        };
        let token = jsonwebtoken::encode(&Header::default(), &claims, &key.encode).unwrap();
        assert!(verify_token(&key, &token).is_err());
    }
}
