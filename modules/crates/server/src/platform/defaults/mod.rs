// `DefaultsError`-style error enum carries strings and an ApiError —
// the `api_error` path is allocation-heavy but only lives on the cold
// denial path. Matches the vault + model-providers + mcp-servers
// modules' allow.
#![allow(clippy::result_large_err)]

//! Page 05 — Platform Defaults business logic.
//!
//! phi-core leverage (mandatory per `CLAUDE.md`):
//! - The persisted `PlatformDefaults.execution_limits /
//!   .default_agent_profile / .context_config / .retry_config` fields
//!   ARE the phi-core types directly. No parallel phi shape.
//! - The wire body for PUT deserialises into `PlatformDefaults` via
//!   serde, so phi-core's field evolution never forces a phi
//!   migration (the store layer's `FLEXIBLE TYPE object` columns
//!   absorb added fields too).
//! - YAML / TOML format conversions for import/export are **CLI-side
//!   concerns** — the server surface is JSON-only. The CLI uses
//!   `serde_yaml` / `toml` directly on the same `PlatformDefaults`
//!   struct, so no parallel parser logic is needed on the server.
//!
//! Optimistic concurrency (plan §P7 + D5):
//! - Every PUT carries an `if_version`. The handler reads the current
//!   row, checks `current.version == if_version`; mismatch → 409.
//! - On match, the handler bumps `version + 1`, stamps
//!   `updated_at = now`, and persists.
//! - First write (no row yet): handler treats `current_version = 0`
//!   and the incoming `if_version` must be `0`.
//!
//! Non-retroactive invariant (plan §G4 + ADR-0019):
//! - Writing `PlatformDefaults` mutates **only** the singleton
//!   `platform_defaults` row. It does NOT touch any per-org state.
//!   M3's org-creation wizard snapshots `PlatformDefaults` at creation
//!   time; existing orgs keep their snapshot untouched. The invariant
//!   is pinned by `platform_defaults_non_retroactive_props`.

pub mod get;
pub mod put;

use domain::audit::events::m2::defaults::PLATFORM_DEFAULTS_URI;
use domain::model::ids::{AuditEventId, AuthRequestId};

/// The stable resource URI for the platform-defaults singleton —
/// matches the catalogue seed done by [`put::put_platform_defaults`]
/// and the `target_uri` every audit event targeting it uses.
///
/// Re-exported here so handler code paths in this module don't need
/// to reach into `domain::audit::events::m2::defaults` for the
/// constant.
pub fn defaults_uri() -> &'static str {
    PLATFORM_DEFAULTS_URI
}

/// Tag placed on the platform-defaults catalogue entry so
/// Permission-Check Step 3 can match `#kind:control_plane` manifests
/// targeting it.
///
/// `PlatformDefaults` classifies as `Composite::ControlPlaneObject` in
/// the v0 ontology (`concepts/permissions/01-resource-ontology.md`
/// §Composite Classes) — a platform-governance data object.
pub const KIND_TAG: &str = "#kind:control_plane";

/// Outcome returned by [`put::put_platform_defaults`].
#[derive(Debug, Clone)]
pub struct PutOutcome {
    pub new_version: u64,
    pub auth_request_id: AuthRequestId,
    pub audit_event_id: AuditEventId,
}

/// Outcome returned by [`get::get_platform_defaults`]. Always carries
/// a `defaults` payload — on a fresh install the handler returns
/// `PlatformDefaults::factory(now)` with `persisted = false` so the
/// web UI can render "using factory defaults" messaging without a
/// null-check.
#[derive(Debug, Clone)]
pub struct GetOutcome {
    /// The persisted defaults when one exists, else the factory
    /// baseline. Callers inspect `persisted` to disambiguate.
    pub defaults: domain::model::PlatformDefaults,
    /// `true` when a row exists in the store; `false` when the
    /// response was synthesised from [`PlatformDefaults::factory`].
    pub persisted: bool,
    /// Always the factory baseline — the web UI renders this
    /// alongside the editable form.
    pub factory: domain::model::PlatformDefaults,
}

/// Business-logic error vocabulary. Mirrors the
/// model-providers / mcp-servers / vault modules' conventions so
/// `error_to_api_error` translations stay symmetric.
#[derive(Debug)]
pub enum DefaultsError {
    /// Input failed shape validation (bad numeric bound, non-empty
    /// requirement, malformed field, etc.).
    Validation(String),
    /// Stale `if_version` — the client's view of the row is out of
    /// date. Carries the current server-side version so the client
    /// can re-read + retry.
    StaleWrite { current_version: u64 },
    /// Repository returned an error.
    Repository(String),
    /// Audit emitter returned an error.
    AuditEmit(String),
}

impl std::fmt::Display for DefaultsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DefaultsError::Validation(m) => write!(f, "validation failed: {m}"),
            DefaultsError::StaleWrite { current_version } => write!(
                f,
                "stale write: client if_version out of date; current is {current_version}"
            ),
            DefaultsError::Repository(m) => write!(f, "repository: {m}"),
            DefaultsError::AuditEmit(m) => write!(f, "audit emit: {m}"),
        }
    }
}

impl std::error::Error for DefaultsError {}

impl From<domain::repository::RepositoryError> for DefaultsError {
    fn from(e: domain::repository::RepositoryError) -> Self {
        DefaultsError::Repository(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_uri_is_stable() {
        assert_eq!(defaults_uri(), "platform_defaults:singleton");
    }

    #[test]
    fn kind_tag_matches_control_plane_composite() {
        // The tag MUST match Composite::ControlPlaneObject.kind_tag()
        // because Permission-Check Step 3 uses it to resolve `#kind:`
        // filters on manifests targeting the singleton.
        assert_eq!(
            KIND_TAG,
            domain::model::Composite::ControlPlaneObject.kind_tag()
        );
    }
}
