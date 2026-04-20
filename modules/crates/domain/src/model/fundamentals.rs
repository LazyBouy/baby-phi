//! The 9 **Fundamental Classes** of the resource ontology.
//!
//! Fundamentals are atomic capabilities; they cannot be decomposed further.
//! Permission Checks are ultimately resolved against fundamentals — every
//! composite (see [`crate::model::composites`]) normalizes to a subset of
//! these 9 before the engine runs.
//!
//! Source of truth: `docs/specs/v0/concepts/permissions/01-resource-ontology.md`
//! §Fundamental Classes.

use serde::{Deserialize, Serialize};

/// Every atomic resource class the system exposes.
///
/// Count: **9** (invariant asserted in `crate::model::tests`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Fundamental {
    /// Files, directories, paths on disk.
    FilesystemObject,
    /// Spawning processes, running binaries/scripts.
    ProcessExecObject,
    /// Outbound network traffic to hosts/ports.
    NetworkEndpoint,
    /// API keys, tokens, env vars holding secrets.
    SecretCredential,
    /// Token budgets, spend, quotas.
    EconomicResource,
    /// CPU, memory, wall clock, concurrency.
    TimeComputeResource,
    /// Generic structured data access (graph nodes, tables, vectors).
    DataObject,
    /// Tag-predicate access grammar (`contains` / `intersects` / `subset_of`)
    /// plus the structural substrate that carries every composite's
    /// `#kind:` identity tag.
    Tag,
    /// Agents, users, roles — the "who" axis.
    IdentityPrincipal,
}

impl Fundamental {
    /// Enumerate every variant in a stable order. The order matches the
    /// concept doc's §Fundamental Classes table.
    pub const ALL: [Fundamental; 9] = [
        Fundamental::FilesystemObject,
        Fundamental::ProcessExecObject,
        Fundamental::NetworkEndpoint,
        Fundamental::SecretCredential,
        Fundamental::EconomicResource,
        Fundamental::TimeComputeResource,
        Fundamental::DataObject,
        Fundamental::Tag,
        Fundamental::IdentityPrincipal,
    ];

    /// The canonical string form used in grants, tags, and wire protocols.
    pub fn as_str(&self) -> &'static str {
        match self {
            Fundamental::FilesystemObject => "filesystem_object",
            Fundamental::ProcessExecObject => "process_exec_object",
            Fundamental::NetworkEndpoint => "network_endpoint",
            Fundamental::SecretCredential => "secret_credential",
            Fundamental::EconomicResource => "economic_resource",
            Fundamental::TimeComputeResource => "time_compute_resource",
            Fundamental::DataObject => "data_object",
            Fundamental::Tag => "tag",
            Fundamental::IdentityPrincipal => "identity_principal",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn all_contains_exactly_nine() {
        assert_eq!(Fundamental::ALL.len(), 9);
    }

    #[test]
    fn all_variants_are_distinct() {
        let set: HashSet<_> = Fundamental::ALL.iter().collect();
        assert_eq!(set.len(), 9);
    }

    #[test]
    fn as_str_is_distinct_per_variant() {
        let strs: HashSet<_> = Fundamental::ALL.iter().map(Fundamental::as_str).collect();
        assert_eq!(strs.len(), 9);
    }

    #[test]
    fn serde_roundtrip_preserves_variant() {
        for f in Fundamental::ALL {
            let json = serde_json::to_string(&f).expect("serialize");
            let back: Fundamental = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(back, f);
        }
    }
}
