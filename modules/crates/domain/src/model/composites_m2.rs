//! M2 composite-instance structs — the typed records admin pages 02–05
//! persist. Every phi-core-overlapping field is a **direct reuse or wrap**
//! per the phi-core leverage mandate in `CLAUDE.md` + §1.5 of the archived
//! M2 plan. Baby-phi-only types (no phi-core counterpart) — `SecretCredential`,
//! `TenantSet`, `RuntimeStatus`, `ExternalServiceKind`, `SecretRef`,
//! `PlatformDefaults`' top-level envelope — live alongside the wraps here
//! for cohesion.
//!
//! Ontology source of truth:
//! `docs/specs/v0/concepts/permissions/01-resource-ontology.md` §Composite
//! Classes. `model_runtime_object`, `external_service_object`, and
//! `secret_credential` (as a `#kind:secret_credential`-tagged fundamental
//! bundle) all live in that table.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::ids::{AgentId, McpServerId, ModelProviderId, OrgId, SecretId};

// ============================================================================
// Re-exports — phi-core is the single source of truth for provider kinds.
// ============================================================================

/// Which LLM API protocol a `ModelRuntime` speaks. Re-exports phi-core's
/// single source of truth so baby-phi never defines a parallel enum.
pub type ProviderKind = phi_core::provider::model::ApiProtocol;

// ============================================================================
// Baby-phi-only enums (no phi-core counterpart).
// ============================================================================

/// Which orgs are allowed to consume a shared platform resource.
///
/// `All` = every org currently on the platform; `Only(ids)` enumerates
/// explicit `OrgId`s. Narrowing `Only` (shrinking the set) triggers
/// cascading revocation of every tenant grant on the affected resource —
/// see the M2 plan's G2 / P6.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "mode", content = "orgs")]
pub enum TenantSet {
    /// Every org currently on the platform.
    All,
    /// Only the enumerated orgs.
    Only(Vec<OrgId>),
}

impl TenantSet {
    /// Whether `org` is permitted to consume the resource.
    pub fn contains(&self, org: OrgId) -> bool {
        match self {
            TenantSet::All => true,
            TenantSet::Only(ids) => ids.contains(&org),
        }
    }

    /// The org set as a `Vec` — empty for `All` (callers must handle the
    /// `All` case separately; this helper returns what was explicitly
    /// enumerated).
    pub fn explicit_orgs(&self) -> &[OrgId] {
        match self {
            TenantSet::All => &[],
            TenantSet::Only(ids) => ids,
        }
    }
}

/// Kind discriminator for [`ExternalService`] composite instances.
///
/// M2 only wires `Mcp`; the other variants carry the wire format forward
/// so the enum is stable when `OpenApi` / `Webhook` surfaces land.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalServiceKind {
    Mcp,
    OpenApi,
    Webhook,
    Other,
}

/// Runtime availability state for a platform resource.
///
/// Set by the (M7b) background health probe; M2 only populates `Ok` and
/// `Archived` at persist time — `Probing` / `Degraded` / `Error` are
/// reserved for the probe's writeback path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeStatus {
    Ok,
    Probing,
    Degraded,
    Error,
    Archived,
}

// ============================================================================
// Secret reference — the stable human-readable slug into the vault.
// ============================================================================

/// Human-readable slug referring to a vault entry (e.g.
/// `"anthropic-api-key"`). Stable across rotations; paired with
/// [`SecretId`] for referential integrity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SecretRef(pub String);

impl SecretRef {
    pub fn new(slug: impl Into<String>) -> Self {
        Self(slug.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SecretRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<&str> for SecretRef {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for SecretRef {
    fn from(s: String) -> Self {
        Self(s)
    }
}

// ============================================================================
// Composite-instance records (what admin page handlers persist).
// ============================================================================

/// A `model_runtime_object` composite instance — a bound LLM provider
/// endpoint plus governance metadata. **Wraps [`phi_core::provider::model::ModelConfig`].**
///
/// Every field that overlaps with phi-core (API protocol, base URL,
/// API-key placeholder, cost / cache config, thinking level, headers,
/// compat flags) lives on `config` — the embedded phi-core struct is
/// the single source of truth. baby-phi only adds platform-governance
/// fields (`secret_ref`, `tenants_allowed`, `status`, timestamps).
///
/// At invocation time (M5+), handlers resolve `secret_ref` against the
/// vault and splice the plaintext into `config.api_key` before passing
/// the `ModelConfig` to phi-core's `ProviderRegistry`. The record itself
/// never stores plaintext material.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRuntime {
    pub id: ModelProviderId,
    /// The phi-core provider binding — source of truth for
    /// `api`, `base_url`, `api_key` (placeholder), `cost`, `headers`,
    /// `compat`.
    pub config: phi_core::provider::model::ModelConfig,
    /// The vault entry holding the real API-key material. `config.api_key`
    /// MUST remain a sentinel (empty string) in the stored record;
    /// handlers splice the plaintext at call time.
    pub secret_ref: SecretRef,
    /// Which orgs may invoke this runtime.
    pub tenants_allowed: TenantSet,
    /// Health status; `Ok` at persist time, updated by the probe.
    pub status: RuntimeStatus,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// An `external_service_object` composite instance — MCP server, OpenAPI
/// spec, webhook, etc. Pure baby-phi: phi-core has no equivalent
/// container. The live `McpClient` (phi-core's) is instantiated **on
/// demand** from this record at probe/invocation time; never stored.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalService {
    pub id: McpServerId,
    pub display_name: String,
    pub kind: ExternalServiceKind,
    /// Transport endpoint (stdio command, HTTP URL, etc.). Schema
    /// depends on `kind`; validation lives in M2/P6 handlers.
    pub endpoint: String,
    /// Optional auth secret. `None` for unauthenticated services.
    pub secret_ref: Option<SecretRef>,
    pub tenants_allowed: TenantSet,
    pub status: RuntimeStatus,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// A `secret_credential` composite-instance catalogue entry in the vault.
/// Pure baby-phi — phi-core does not ship a secrets vault.
///
/// The sealed bytes live in the `secrets_vault` SurrealDB table; this
/// struct is the domain-layer "catalogue" entry pointing at that row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretCredential {
    pub id: SecretId,
    /// Human-readable slug (e.g. `anthropic-api-key`). Stable across
    /// rotations; how operators refer to the secret on the CLI / UI.
    pub slug: SecretRef,
    /// Agent currently authorized to rotate + reveal this secret.
    pub custodian: AgentId,
    pub last_rotated_at: Option<DateTime<Utc>>,
    /// `true` = mask the value in audit diffs and list views. The
    /// vault always masks plaintext regardless; this is an extra
    /// opt-in for diffs.
    pub sensitive: bool,
    pub created_at: DateTime<Utc>,
}

/// Platform-wide defaults applied at org-creation time.
///
/// **Not retroactive** — changing a field here does NOT mutate existing
/// orgs; it only affects orgs spawned afterwards (the invariant landed
/// in M2/P7 via `platform_defaults_non_retroactive_props`).
///
/// Every phi-core-overlapping field is a direct wrap — `ExecutionLimits`,
/// `AgentProfile`, `ContextConfig`, `RetryConfig` all imported directly
/// from phi-core. The baby-phi-only fields (retention, alert channels)
/// are platform-governance additions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformDefaults {
    /// Sentinel value (always `1`) enforced by a `UNIQUE INDEX` on the
    /// `platform_defaults` table — guarantees at most one row ever.
    pub singleton: u8,
    /// phi-core agent-loop safety net (max turns / tokens / duration /
    /// cost). Single source of truth.
    pub execution_limits: phi_core::context::execution::ExecutionLimits,
    /// phi-core agent blueprint (system prompt, thinking level, skills).
    /// Applied to agents spawned in orgs without their own override.
    pub default_agent_profile: phi_core::agents::profile::AgentProfile,
    /// phi-core context-tracking config (compaction strategy, token budget).
    pub context_config: phi_core::context::config::ContextConfig,
    /// phi-core retry tuning (exponential backoff + jitter parameters).
    pub retry_config: phi_core::provider::retry::RetryConfig,
    /// Default audit-log retention in days (Silent tier baseline).
    pub default_retention_days: u32,
    /// Default alert-delivery channels — handle strings (emails,
    /// webhook URLs) — matched against `Channel` records at alert time.
    pub default_alert_channels: Vec<String>,
    pub updated_at: DateTime<Utc>,
    /// Monotonic revision counter — incremented on every PUT; drives
    /// optimistic concurrency control in P7.
    pub version: u64,
}

// ============================================================================
// Tests — shape + serde round-trip for every new type.
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tenant_set_contains_all_matches_every_org() {
        let ts = TenantSet::All;
        assert!(ts.contains(OrgId::new()));
    }

    #[test]
    fn tenant_set_only_matches_enumerated_orgs() {
        let org_a = OrgId::new();
        let org_b = OrgId::new();
        let ts = TenantSet::Only(vec![org_a]);
        assert!(ts.contains(org_a));
        assert!(!ts.contains(org_b));
    }

    #[test]
    fn tenant_set_serde_roundtrip() {
        let org = OrgId::new();
        for ts in [TenantSet::All, TenantSet::Only(vec![org])] {
            let json = serde_json::to_string(&ts).expect("serialize");
            let back: TenantSet = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(back, ts);
        }
    }

    #[test]
    fn external_service_kind_serde_roundtrip() {
        for k in [
            ExternalServiceKind::Mcp,
            ExternalServiceKind::OpenApi,
            ExternalServiceKind::Webhook,
            ExternalServiceKind::Other,
        ] {
            let json = serde_json::to_string(&k).expect("serialize");
            let back: ExternalServiceKind = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(back, k);
        }
    }

    #[test]
    fn runtime_status_serde_roundtrip() {
        for s in [
            RuntimeStatus::Ok,
            RuntimeStatus::Probing,
            RuntimeStatus::Degraded,
            RuntimeStatus::Error,
            RuntimeStatus::Archived,
        ] {
            let json = serde_json::to_string(&s).expect("serialize");
            let back: RuntimeStatus = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(back, s);
        }
    }

    #[test]
    fn secret_ref_is_transparent_wire_format() {
        let r = SecretRef::new("anthropic-api-key");
        let json = serde_json::to_string(&r).expect("serialize");
        assert_eq!(json, "\"anthropic-api-key\"");
        let back: SecretRef = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, r);
    }

    #[test]
    fn model_runtime_wraps_phi_core_model_config() {
        // Build a ModelRuntime with phi-core's anthropic factory — proves
        // the wrap composes and serdes cleanly.
        let config = phi_core::provider::model::ModelConfig::anthropic(
            "claude-sonnet",
            "claude-sonnet-4-6",
            "__placeholder__",
        );
        let rt = ModelRuntime {
            id: ModelProviderId::new(),
            config,
            secret_ref: SecretRef::new("anthropic-api-key"),
            tenants_allowed: TenantSet::All,
            status: RuntimeStatus::Ok,
            archived_at: None,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&rt).expect("serialize");
        assert!(json.contains("claude-sonnet"));
        assert!(json.contains("anthropic-api-key"));
        let back: ModelRuntime = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.id, rt.id);
        assert_eq!(back.config.id, rt.config.id);
        assert_eq!(back.secret_ref, rt.secret_ref);
    }

    #[test]
    fn platform_defaults_wraps_phi_core_types() {
        // Build PlatformDefaults with phi-core defaults — proves the
        // four wrapped phi-core types all compose + serde round-trip.
        let pd = PlatformDefaults {
            singleton: 1,
            execution_limits: phi_core::context::execution::ExecutionLimits::default(),
            default_agent_profile: phi_core::agents::profile::AgentProfile::default(),
            context_config: phi_core::context::config::ContextConfig::default(),
            retry_config: phi_core::provider::retry::RetryConfig::default(),
            default_retention_days: 30,
            default_alert_channels: vec!["ops@example.com".to_string()],
            updated_at: Utc::now(),
            version: 0,
        };
        let json = serde_json::to_string(&pd).expect("serialize");
        let back: PlatformDefaults = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.singleton, 1);
        assert_eq!(back.default_retention_days, 30);
        assert_eq!(back.version, 0);
    }

    #[test]
    fn external_service_roundtrips() {
        let svc = ExternalService {
            id: McpServerId::new(),
            display_name: "memory-mcp".to_string(),
            kind: ExternalServiceKind::Mcp,
            endpoint: "stdio:///usr/local/bin/memory-mcp".to_string(),
            secret_ref: None,
            tenants_allowed: TenantSet::All,
            status: RuntimeStatus::Ok,
            archived_at: None,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&svc).expect("serialize");
        let back: ExternalService = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.id, svc.id);
        assert_eq!(back.kind, ExternalServiceKind::Mcp);
    }

    #[test]
    fn secret_credential_roundtrips() {
        let sc = SecretCredential {
            id: SecretId::new(),
            slug: SecretRef::new("anthropic-api-key"),
            custodian: AgentId::new(),
            last_rotated_at: None,
            sensitive: true,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&sc).expect("serialize");
        let back: SecretCredential = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.id, sc.id);
        assert_eq!(back.slug, sc.slug);
        assert!(back.sensitive);
    }
}
