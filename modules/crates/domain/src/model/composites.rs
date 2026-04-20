//! The 8 **Composite Classes** of the resource ontology.
//!
//! A composite is a named bundle of fundamentals. Every composite implicitly
//! pulls in the `tag` fundamental so that its `#kind:{composite_name}` tag is
//! queryable at Permission Check time. The runtime normalizes every composite
//! to its constituent fundamentals before grants are resolved.
//!
//! Source of truth: `docs/specs/v0/concepts/permissions/01-resource-ontology.md`
//! §Composite Classes.

use serde::{Deserialize, Serialize};

use super::fundamentals::Fundamental;

/// Every named bundle of fundamentals the v0 ontology defines.
///
/// Count: **8** (invariant asserted in `crate::model::tests`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Composite {
    /// MCP servers, webhooks, email APIs, Slack, etc.
    /// Expands to: `network_endpoint` + `secret_credential` + `tag`.
    ExternalServiceObject,
    /// LLM endpoints that also consume token budget.
    /// Expands to: `network_endpoint` + `secret_credential` + `economic_resource` + `tag`.
    ModelRuntimeObject,
    /// Managing the policy store (the catalogue, templates, etc.).
    /// Expands to: `data_object` + `identity_principal` + `tag`.
    ControlPlaneObject,
    /// An Agent's persistent knowledge store.
    /// Expands to: `data_object` + `tag`.
    MemoryObject,
    /// One logical task or conversation (a Session).
    /// Expands to: `data_object` + `tag`.
    SessionObject,
    /// First-class workflow node that mediates Grant creation.
    /// Expands to: `data_object` + `tag`.
    AuthRequestObject,
    /// An Agent's received-messages queue.
    /// Expands to: `data_object` + `tag`.
    InboxObject,
    /// An Agent's sent-messages log.
    /// Expands to: `data_object` + `tag`.
    OutboxObject,
}

impl Composite {
    /// Enumerate every variant in a stable order. Matches the concept doc's
    /// §Composite Classes table order.
    pub const ALL: [Composite; 8] = [
        Composite::ExternalServiceObject,
        Composite::ModelRuntimeObject,
        Composite::ControlPlaneObject,
        Composite::MemoryObject,
        Composite::SessionObject,
        Composite::AuthRequestObject,
        Composite::InboxObject,
        Composite::OutboxObject,
    ];

    /// The canonical string form (e.g. `memory_object`).
    pub fn as_str(&self) -> &'static str {
        match self {
            Composite::ExternalServiceObject => "external_service_object",
            Composite::ModelRuntimeObject => "model_runtime_object",
            Composite::ControlPlaneObject => "control_plane_object",
            Composite::MemoryObject => "memory_object",
            Composite::SessionObject => "session_object",
            Composite::AuthRequestObject => "auth_request_object",
            Composite::InboxObject => "inbox_object",
            Composite::OutboxObject => "outbox_object",
        }
    }

    /// The `#kind:` tag every instance of this composite implicitly carries.
    ///
    /// Per `concepts/ontology.md` §Tag Conventions:
    /// > **Type-identity tags** — `#kind:{composite_name}` — auto-added at
    /// > creation; declares which composite type the instance belongs to
    /// > (e.g., `#kind:session`, `#kind:memory`, `#kind:auth_request`).
    pub fn kind_tag(&self) -> &'static str {
        match self {
            Composite::ExternalServiceObject => "#kind:external_service",
            Composite::ModelRuntimeObject => "#kind:model_runtime",
            Composite::ControlPlaneObject => "#kind:control_plane",
            Composite::MemoryObject => "#kind:memory",
            Composite::SessionObject => "#kind:session",
            Composite::AuthRequestObject => "#kind:auth_request",
            Composite::InboxObject => "#kind:inbox",
            Composite::OutboxObject => "#kind:outbox",
        }
    }

    /// Short kind name — the `{kind}` portion of the instance-identity tag
    /// `{kind}:{instance_id}` and of the `#kind:{kind}` type tag.
    /// E.g. `MemoryObject → "memory"`, `AuthRequestObject → "auth_request"`.
    /// The full composite class name (with the `_object` suffix) is
    /// available via [`as_str`]; this helper returns the short form used
    /// everywhere tags appear.
    pub fn kind_name(&self) -> &'static str {
        // kind_tag() is always `"#kind:{short_name}"`, so we can strip
        // the 6-char prefix and return a &'static &str slice.
        &self.kind_tag()["#kind:".len()..]
    }

    /// Build the pair of runtime-auto-added tags for a new instance of
    /// this composite: `[ "#kind:{kind}", "{kind}:{instance_id}" ]`.
    ///
    /// Per `concepts/ontology.md` §Tag Conventions:
    /// > Every composite instance carries a **self-identity tag** of the
    /// > form `{kind}:{instance_id}` in addition to its `#kind:{kind}`
    /// > type tag. The runtime auto-adds this tag at instance creation
    /// > time (just like the `#kind:` tag), and it cannot be set or
    /// > modified by agents or tools.
    ///
    /// Callers creating a new composite instance should always use this
    /// helper rather than hand-rolling the tag strings — it's the single
    /// source of truth for the two reserved tag namespaces.
    pub fn auto_tags(&self, instance_id: &str) -> [String; 2] {
        [
            self.kind_tag().to_string(),
            format!("{}:{}", self.kind_name(), instance_id),
        ]
    }

    /// Which fundamentals this composite expands to at Permission Check time.
    /// The `tag` fundamental is always implicitly included — it carries the
    /// `#kind:` identity tag — so it's in every list below.
    pub fn constituents(&self) -> &'static [Fundamental] {
        match self {
            Composite::ExternalServiceObject => &[
                Fundamental::NetworkEndpoint,
                Fundamental::SecretCredential,
                Fundamental::Tag,
            ],
            Composite::ModelRuntimeObject => &[
                Fundamental::NetworkEndpoint,
                Fundamental::SecretCredential,
                Fundamental::EconomicResource,
                Fundamental::Tag,
            ],
            Composite::ControlPlaneObject => &[
                Fundamental::DataObject,
                Fundamental::IdentityPrincipal,
                Fundamental::Tag,
            ],
            Composite::MemoryObject
            | Composite::SessionObject
            | Composite::AuthRequestObject
            | Composite::InboxObject
            | Composite::OutboxObject => &[Fundamental::DataObject, Fundamental::Tag],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn all_contains_exactly_eight() {
        assert_eq!(Composite::ALL.len(), 8);
    }

    #[test]
    fn all_variants_are_distinct() {
        let set: HashSet<_> = Composite::ALL.iter().collect();
        assert_eq!(set.len(), 8);
    }

    #[test]
    fn every_composite_includes_tag_fundamental() {
        for c in Composite::ALL {
            assert!(
                c.constituents().contains(&Fundamental::Tag),
                "{:?} must implicitly include Tag for its #kind:* identity tag",
                c
            );
        }
    }

    #[test]
    fn kind_tags_are_distinct() {
        let tags: HashSet<_> = Composite::ALL.iter().map(Composite::kind_tag).collect();
        assert_eq!(tags.len(), 8);
    }

    #[test]
    fn kind_name_strips_hashkind_prefix() {
        for c in Composite::ALL {
            let tag = c.kind_tag();
            assert_eq!(format!("#kind:{}", c.kind_name()), tag);
        }
    }

    #[test]
    fn auto_tags_emits_both_reserved_namespaces() {
        let c = Composite::MemoryObject;
        let tags = c.auto_tags("m-4581");
        assert_eq!(tags[0], "#kind:memory");
        assert_eq!(tags[1], "memory:m-4581");
    }

    #[test]
    fn auto_tags_work_for_every_composite() {
        for c in Composite::ALL {
            let tags = c.auto_tags("inst-1");
            // Type-identity tag = `#kind:{kind_name()}`
            assert_eq!(tags[0], format!("#kind:{}", c.kind_name()));
            // Instance-identity tag = `{kind_name()}:{instance_id}`
            assert_eq!(tags[1], format!("{}:inst-1", c.kind_name()));
        }
    }

    #[test]
    fn as_str_is_distinct_per_variant() {
        let strs: HashSet<_> = Composite::ALL.iter().map(Composite::as_str).collect();
        assert_eq!(strs.len(), 8);
    }

    #[test]
    fn serde_roundtrip_preserves_variant() {
        for c in Composite::ALL {
            let json = serde_json::to_string(&c).expect("serialize");
            let back: Composite = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(back, c);
        }
    }
}
