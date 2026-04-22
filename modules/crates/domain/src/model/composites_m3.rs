//! M3 composite-instance structs — the typed records admin pages 06–07
//! persist on top of M2's platform surfaces.
//!
//! Ontology source of truth: `docs/specs/v0/concepts/organization.md` +
//! `docs/specs/v0/concepts/permissions/01-resource-ontology.md`.
//!
//! phi-core leverage (mandatory per `phi/CLAUDE.md`): every
//! phi-core-overlapping field on [`OrganizationDefaultsSnapshot`] is a
//! direct wrap — `ExecutionLimits`, `AgentProfile`, `ContextConfig`,
//! `RetryConfig` — identical pattern to M2/P7's
//! [`crate::model::composites_m2::PlatformDefaults`]. The phi-only
//! types here ([`ConsentPolicy`], [`TokenBudgetPool`]) are governance
//! primitives phi-core has no counterpart for.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::ids::{NodeId, OrgId};

// ============================================================================
// ConsentPolicy — phi-only governance primitive.
// ============================================================================

/// Org-level consent policy governing how an org's members authorise
/// cross-session data access. Populated on every org at creation
/// time (M3/P4 wizard step 2) and frozen for the org's lifetime (the
/// non-retroactive invariant).
///
/// Variants mirror `docs/specs/v0/concepts/consent.md` §Policy Kinds:
///
/// - [`ConsentPolicy::Implicit`] — consent flows from org membership
///   alone. Operators in small/trusted teams usually start here.
/// - [`ConsentPolicy::OneTime`] — explicit consent prompt once per
///   consenter-principal pair; reused for all subsequent sessions.
/// - [`ConsentPolicy::PerSession`] — explicit consent required on
///   every session launch. The strictest mode; used by regulated /
///   privacy-focused orgs.
///
/// **phi-core leverage**: none — phi-core has no governance-consent
/// concept. This is a phi-only primitive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsentPolicy {
    Implicit,
    OneTime,
    PerSession,
}

impl ConsentPolicy {
    /// Stable wire-name for the variant (matches serde's snake_case
    /// rename). Used by the M3/P4 wizard's HTTP payload + the
    /// migration 0003 ASSERT clause on `organization.consent_policy`.
    pub fn as_wire(&self) -> &'static str {
        match self {
            ConsentPolicy::Implicit => "implicit",
            ConsentPolicy::OneTime => "one_time",
            ConsentPolicy::PerSession => "per_session",
        }
    }

    /// Every variant in declaration order. Used by tests + the CLI
    /// help-text generator to enumerate options without reflection.
    pub const ALL: [ConsentPolicy; 3] = [
        ConsentPolicy::Implicit,
        ConsentPolicy::OneTime,
        ConsentPolicy::PerSession,
    ];
}

// ============================================================================
// OrganizationDefaultsSnapshot — four phi-core wraps + two phi fields.
// ============================================================================

/// A per-org snapshot of the platform's default agent-loop + governance
/// settings, captured at org-creation time and never retroactively
/// updated. Embedded as a field on the [`crate::model::Organization`]
/// node (per M3 plan D1 — no sibling composite).
///
/// Four fields wrap phi-core types directly (no parallel phi
/// structs) so phi-core evolution flows through the snapshot without a
/// migration. The two phi-only fields ([`default_retention_days`]
/// and [`default_alert_channels`]) carry platform-governance concerns
/// phi-core does not model.
///
/// **Non-retroactive invariant** (ADR-0019, re-verified by the
/// `platform_defaults_non_retroactive_props` proptest in M2/P7): a
/// later `PlatformDefaults` PUT does NOT mutate any existing org's
/// snapshot. Each snapshot is a frozen copy at `created_at`.
///
/// [`default_retention_days`]: OrganizationDefaultsSnapshot::default_retention_days
/// [`default_alert_channels`]: OrganizationDefaultsSnapshot::default_alert_channels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationDefaultsSnapshot {
    /// phi-core agent-loop safety net (max turns / tokens / duration
    /// / cost). Captured from the platform defaults at creation time.
    pub execution_limits: phi_core::context::execution::ExecutionLimits,
    /// phi-core agent blueprint (system prompt, thinking level,
    /// skills). Applied to agents spawned in this org without a
    /// specific profile override.
    pub default_agent_profile: phi_core::agents::profile::AgentProfile,
    /// phi-core context-tracking config (compaction strategy, token
    /// budget).
    pub context_config: phi_core::context::config::ContextConfig,
    /// phi-core retry tuning (exponential backoff + jitter).
    pub retry_config: phi_core::provider::retry::RetryConfig,
    /// Audit-log retention in days (Silent tier baseline) for this
    /// org. Inherited from `PlatformDefaults.default_retention_days`
    /// at snapshot time.
    pub default_retention_days: u32,
    /// Alert-delivery channels for this org — handle strings
    /// (emails, webhook URLs) — matched against `Channel` records at
    /// alert time.
    pub default_alert_channels: Vec<String>,
}

impl OrganizationDefaultsSnapshot {
    /// Snapshot the current platform defaults for a newly-created
    /// org. The resulting snapshot is frozen on the Organization
    /// node; later platform-defaults edits do not propagate (the
    /// non-retroactive invariant).
    pub fn from_platform_defaults(pd: &super::composites_m2::PlatformDefaults) -> Self {
        Self {
            execution_limits: pd.execution_limits.clone(),
            default_agent_profile: pd.default_agent_profile.clone(),
            context_config: pd.context_config.clone(),
            retry_config: pd.retry_config.clone(),
            default_retention_days: pd.default_retention_days,
            default_alert_channels: pd.default_alert_channels.clone(),
        }
    }
}

// ============================================================================
// TokenBudgetPool — phi-only economic-resource primitive.
// ============================================================================

/// Per-org token budget pool — tracks cumulative agent-loop token
/// consumption against an allocation ceiling. Pure phi: phi-core
/// has no budget-accounting container.
///
/// Lifecycle: created at M3/P4 org-creation time with
/// `used = 0` and `initial_allocation` set from the wizard payload.
/// M5+'s session-launch handler debits `used` at session end; pages
/// 07 (dashboard) and 14 (session-launch) both read `used / total`
/// for budget panels.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenBudgetPool {
    /// Node identity for this pool — lets audit events + catalogue
    /// entries reference it by stable id.
    pub id: NodeId,
    /// The org this pool belongs to. 1:1 with Organization in v0
    /// (each org gets exactly one pool at creation; M7+ may extend).
    pub owning_org: OrgId,
    /// Ceiling — the token allocation granted to the org at creation.
    /// Denominated in LLM input+output tokens (provider-agnostic;
    /// conversion at session-end happens via the provider's `usage`
    /// accounting).
    pub initial_allocation: u64,
    /// Cumulative tokens consumed to date. Must satisfy
    /// `used <= initial_allocation` — the invariant is enforced by
    /// [`TokenBudgetPool::debit`] at write time + by the migration
    /// 0003 ASSERT at the storage layer.
    pub used: u64,
    pub created_at: DateTime<Utc>,
}

impl TokenBudgetPool {
    /// Build a fresh pool for a newly-created org. `used = 0`;
    /// `created_at = now`.
    pub fn new(owning_org: OrgId, initial_allocation: u64, now: DateTime<Utc>) -> Self {
        Self {
            id: NodeId::new(),
            owning_org,
            initial_allocation,
            used: 0,
            created_at: now,
        }
    }

    /// Tokens remaining — `initial_allocation - used`. Saturates at 0
    /// when used exceeds allocation (the ASSERT at storage should
    /// prevent this, but saturating arithmetic keeps the API safe in
    /// the face of a future dirty-read).
    pub fn remaining(&self) -> u64 {
        self.initial_allocation.saturating_sub(self.used)
    }

    /// Record token consumption. Returns the new `used` total on
    /// success; returns `Err` when the debit would exceed the
    /// allocation — callers (M5's session-end handler) surface the
    /// error as `TOKEN_BUDGET_EXHAUSTED` to the operator.
    pub fn debit(&mut self, amount: u64) -> Result<u64, &'static str> {
        let new_used = self
            .used
            .checked_add(amount)
            .ok_or("token_budget_pool: usize overflow on debit")?;
        if new_used > self.initial_allocation {
            return Err("token_budget_pool: debit would exceed initial_allocation");
        }
        self.used = new_used;
        Ok(self.used)
    }
}

// ============================================================================
// Tests — shape + serde round-trip for every new type.
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consent_policy_all_has_three_variants() {
        assert_eq!(ConsentPolicy::ALL.len(), 3);
    }

    #[test]
    fn consent_policy_serde_roundtrip() {
        for p in ConsentPolicy::ALL {
            let json = serde_json::to_string(&p).expect("serialize");
            let back: ConsentPolicy = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(back, p);
        }
    }

    #[test]
    fn consent_policy_wire_names_match_serde() {
        // Wire names must match serde's snake_case rendering — the
        // migration 0003 ASSERT clause on `organization.consent_policy`
        // uses the serde representation, so drift between
        // `as_wire()` and serde would silently break INSERTs.
        for p in ConsentPolicy::ALL {
            let serde_form = serde_json::to_value(p)
                .ok()
                .and_then(|v| v.as_str().map(str::to_string))
                .expect("ConsentPolicy serialises as a JSON string");
            assert_eq!(p.as_wire(), serde_form);
        }
    }

    #[test]
    fn organization_defaults_snapshot_wraps_phi_core_types() {
        // Proves the four phi-core wraps compose + serde round-trip
        // without a parallel phi layer.
        let snap = OrganizationDefaultsSnapshot {
            execution_limits: phi_core::context::execution::ExecutionLimits::default(),
            default_agent_profile: phi_core::agents::profile::AgentProfile::default(),
            context_config: phi_core::context::config::ContextConfig::default(),
            retry_config: phi_core::provider::retry::RetryConfig::default(),
            default_retention_days: 30,
            default_alert_channels: vec!["ops@example.com".into()],
        };
        let json = serde_json::to_string(&snap).expect("serialize");
        let back: OrganizationDefaultsSnapshot = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.default_retention_days, 30);
        assert_eq!(
            back.default_alert_channels,
            vec!["ops@example.com".to_string()]
        );
    }

    #[test]
    fn organization_defaults_snapshot_from_platform_defaults_is_identity() {
        // from_platform_defaults must copy every field verbatim — the
        // non-retroactive invariant depends on the snapshot matching
        // the platform defaults at snapshot time.
        use super::super::composites_m2::PlatformDefaults;

        let now = Utc::now();
        let mut pd = PlatformDefaults::factory(now);
        pd.default_retention_days = 45;
        pd.default_alert_channels = vec!["alerts@acme.com".into()];

        let snap = OrganizationDefaultsSnapshot::from_platform_defaults(&pd);
        assert_eq!(snap.default_retention_days, 45);
        assert_eq!(
            snap.default_alert_channels,
            vec!["alerts@acme.com".to_string()]
        );
        // phi-core-wrapped fields come through untouched — field-level
        // comparison is expensive, so serialise both sides and diff.
        let expected = serde_json::to_value(&pd.execution_limits).unwrap();
        let got = serde_json::to_value(&snap.execution_limits).unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn token_budget_pool_new_starts_at_zero_used() {
        let p = TokenBudgetPool::new(OrgId::new(), 1_000_000, Utc::now());
        assert_eq!(p.used, 0);
        assert_eq!(p.remaining(), 1_000_000);
    }

    #[test]
    fn token_budget_pool_debit_updates_used_and_remaining() {
        let mut p = TokenBudgetPool::new(OrgId::new(), 1_000, Utc::now());
        let after = p.debit(300).expect("debit within budget");
        assert_eq!(after, 300);
        assert_eq!(p.remaining(), 700);
        p.debit(500).expect("second debit within budget");
        assert_eq!(p.used, 800);
    }

    #[test]
    fn token_budget_pool_debit_rejects_overflow_budget() {
        let mut p = TokenBudgetPool::new(OrgId::new(), 100, Utc::now());
        assert!(
            p.debit(101).is_err(),
            "debit must reject exceeding allocation"
        );
        // used stays at 0 — debit is all-or-nothing.
        assert_eq!(p.used, 0);
    }

    #[test]
    fn token_budget_pool_serde_roundtrip() {
        let p = TokenBudgetPool::new(OrgId::new(), 1_000_000, Utc::now());
        let json = serde_json::to_string(&p).expect("serialize");
        let back: TokenBudgetPool = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, p);
    }
}
