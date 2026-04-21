//! Persistent CLI session-cookie store.
//!
//! The CLI's write commands (M2/P4+ `secret`, `model-provider`,
//! `mcp-server`, `platform-defaults`) need the admin's signed
//! session cookie on every request. Rather than re-prompting for the
//! bootstrap credential on each invocation (which is a **single-use**
//! secret, so it can't be reused anyway), the cookie is saved to a
//! file that is mode `0600` — owner read/write only — under the
//! platform's standard XDG config directory.
//!
//! Location precedence (matches the XDG Base Directory spec):
//!   1. `$XDG_CONFIG_HOME/baby-phi/session`
//!   2. `~/.config/baby-phi/session`  (fallback when `$XDG_CONFIG_HOME` is unset)
//!
//! M2 ships only a single-slot store (one session per `$HOME`). OAuth
//! lands in M3 + that milestone upgrades this to a keyring-backed
//! storage per decision D14 of the M2 plan.
//!
//! Only baby-phi Unix targets are wired for the `0600` permission
//! check; on non-Unix (Windows) the file is written with the
//! platform's default perms and a warning is emitted. The CLI is not
//! expected to be installed on Windows in M2.

use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

/// Token format written to / read from the session file. Kept as a
/// struct (not a bare string) so M3 can extend it with the refresh
/// token + OAuth metadata without a disk-format migration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct SavedSession {
    /// The signed session-cookie value (JWT string). Opaque to the
    /// CLI; the server verifies it.
    pub cookie_value: String,
    /// When the cookie was minted (RFC3339). Helpful in `--debug` output.
    pub issued_at: String,
    /// The logged-in admin's agent_id (UUID string). Redundant with
    /// the cookie claim, but lets the CLI print "logged in as …"
    /// without hitting the server.
    pub agent_id: String,
}

/// Session-store I/O failures. All variants wrap the underlying error
/// so callers can decide whether to surface `EXIT_TRANSPORT` /
/// `EXIT_INTERNAL` / `EXIT_PRECONDITION_FAILED`.
#[derive(Debug, thiserror::Error)]
pub enum SessionStoreError {
    #[error("could not resolve session-store path: {0}")]
    PathResolution(String),
    #[error("no saved session found at {path} — run `baby-phi login` first")]
    NotFound { path: PathBuf },
    #[error("failed to read session file at {path}: {source}")]
    Read { path: PathBuf, source: io::Error },
    #[error("failed to write session file at {path}: {source}")]
    Write { path: PathBuf, source: io::Error },
    #[error("saved session at {path} is malformed: {reason}")]
    Malformed { path: PathBuf, reason: String },
}

/// Resolve `$XDG_CONFIG_HOME/baby-phi/session` (or the fallback). Does
/// NOT create the parent directory; callers must call `save()` to do
/// that at write time.
pub fn default_session_path() -> Result<PathBuf, SessionStoreError> {
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        let mut p = PathBuf::from(xdg);
        p.push("baby-phi");
        p.push("session");
        return Ok(p);
    }
    if let Some(home) = std::env::var_os("HOME") {
        let mut p = PathBuf::from(home);
        p.push(".config");
        p.push("baby-phi");
        p.push("session");
        return Ok(p);
    }
    Err(SessionStoreError::PathResolution(
        "neither $XDG_CONFIG_HOME nor $HOME is set".to_string(),
    ))
}

/// Save a session to `path` with `0600` permissions on Unix.
///
/// Overwrites any existing file atomically (write-to-temp + rename).
pub fn save(path: &std::path::Path, session: &SavedSession) -> Result<(), SessionStoreError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| SessionStoreError::Write {
            path: path.to_path_buf(),
            source,
        })?;
    }

    let tmp = path.with_extension("tmp");
    {
        let mut f = fs::File::create(&tmp).map_err(|source| SessionStoreError::Write {
            path: tmp.clone(),
            source,
        })?;

        // Apply restrictive perms BEFORE the cookie material hits disk.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            f.set_permissions(fs::Permissions::from_mode(0o600))
                .map_err(|source| SessionStoreError::Write {
                    path: tmp.clone(),
                    source,
                })?;
        }

        let body =
            serde_json::to_vec_pretty(session).map_err(|e| SessionStoreError::Malformed {
                path: tmp.clone(),
                reason: e.to_string(),
            })?;
        f.write_all(&body)
            .map_err(|source| SessionStoreError::Write {
                path: tmp.clone(),
                source,
            })?;
        f.sync_all().map_err(|source| SessionStoreError::Write {
            path: tmp.clone(),
            source,
        })?;
    }

    fs::rename(&tmp, path).map_err(|source| SessionStoreError::Write {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(())
}

/// Load the saved session from `path`. Returns `NotFound` when the
/// file does not exist (the idiomatic "logged out" signal).
pub fn load(path: &std::path::Path) -> Result<SavedSession, SessionStoreError> {
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            return Err(SessionStoreError::NotFound {
                path: path.to_path_buf(),
            });
        }
        Err(source) => {
            return Err(SessionStoreError::Read {
                path: path.to_path_buf(),
                source,
            });
        }
    };
    serde_json::from_slice(&bytes).map_err(|e| SessionStoreError::Malformed {
        path: path.to_path_buf(),
        reason: e.to_string(),
    })
}

/// Remove the saved session. Idempotent — missing file is success.
pub fn clear(path: &std::path::Path) -> Result<(), SessionStoreError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(SessionStoreError::Write {
            path: path.to_path_buf(),
            source,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn sample() -> SavedSession {
        SavedSession {
            cookie_value: "abc.def.ghi".to_string(),
            issued_at: "2026-04-21T12:00:00Z".to_string(),
            agent_id: "00000000-0000-0000-0000-000000000001".to_string(),
        }
    }

    #[test]
    fn save_then_load_round_trips() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("session");
        let s = sample();
        save(&path, &s).expect("save");
        let back = load(&path).expect("load");
        assert_eq!(back, s);
    }

    #[test]
    fn load_missing_file_returns_not_found() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("does-not-exist");
        match load(&path) {
            Err(SessionStoreError::NotFound { .. }) => {}
            other => panic!("expected NotFound, got {:?}", other),
        }
    }

    #[test]
    fn clear_is_idempotent_when_absent() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("does-not-exist");
        clear(&path).expect("idempotent clear");
    }

    #[test]
    fn clear_removes_an_existing_session() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("session");
        save(&path, &sample()).expect("save");
        assert!(path.exists());
        clear(&path).expect("clear");
        assert!(!path.exists());
    }

    #[test]
    fn malformed_file_surfaces_malformed_error() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("session");
        fs::write(&path, "this is not json").expect("write garbage");
        match load(&path) {
            Err(SessionStoreError::Malformed { .. }) => {}
            other => panic!("expected Malformed, got {:?}", other),
        }
    }

    #[cfg(unix)]
    #[test]
    fn save_applies_0600_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("session");
        save(&path, &sample()).expect("save");
        let meta = fs::metadata(&path).expect("metadata");
        let mode = meta.permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "session file must be owner-only");
    }
}
