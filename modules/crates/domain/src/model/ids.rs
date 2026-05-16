//! Strongly-typed identifiers for every entity in the graph model.
//!
//! Each ID is a `Uuid` newtype so the compiler catches accidental crosses
//! (e.g., passing an `AgentId` where an `OrgId` is required). All IDs derive
//! `Serialize`/`Deserialize` with a transparent representation so wire format
//! is a plain UUID string.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Helper macro: defines a `Uuid` newtype with `new()` + `Default` + Display.
macro_rules! id_newtype {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            pub fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }

            pub fn as_uuid(&self) -> &Uuid {
                &self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

id_newtype!(
    /// Opaque identifier for any node in the graph.
    NodeId
);
id_newtype!(
    /// Opaque identifier for any edge in the graph.
    EdgeId
);
id_newtype!(
    /// Identifier of an `Organization` node.
    OrgId
);
id_newtype!(
    /// Identifier of an `Agent` node (human or LLM).
    AgentId
);
id_newtype!(
    /// Identifier of a `User` node.
    UserId
);
id_newtype!(
    /// Identifier of a `Project` node.
    ProjectId
);
id_newtype!(
    /// Identifier of a `Grant` node.
    GrantId
);
id_newtype!(
    /// Identifier of an `AuthRequest` node.
    AuthRequestId
);
id_newtype!(
    /// Identifier of a `Template` node.
    TemplateId
);
id_newtype!(
    /// Identifier of a `Consent` node.
    ConsentId
);
id_newtype!(
    /// Identifier of a `Session` node.
    SessionId
);
id_newtype!(
    /// Identifier of a `Memory` node.
    MemoryId
);
id_newtype!(
    /// Identifier of an `AuditEvent`.
    AuditEventId
);
id_newtype!(
    /// Identifier of a secret entry in the credentials vault (M2 page 04).
    ///
    /// Distinct from [`SecretRef`] (the human-readable slug, e.g.
    /// `anthropic-api-key`). `SecretId` is the opaque UUID used for
    /// referential integrity; `SecretRef` is what operators type.
    SecretId
);
id_newtype!(
    /// Identifier of a `ModelRuntime` composite instance (M2 page 02).
    ModelProviderId
);
id_newtype!(
    /// Identifier of an `ExternalService` composite instance (M2 page 03).
    McpServerId
);
id_newtype!(
    /// Identifier of a `LoopRecord` governance node (M5 — wraps
    /// `phi_core::session::model::LoopRecord`). Distinct from phi-core's
    /// `LoopRecord.loop_id: String` — the latter lives on `inner`.
    LoopId
);
id_newtype!(
    /// Identifier of a `Turn` governance node (M5 — wraps
    /// `phi_core::session::model::Turn`). Distinct from phi-core's
    /// `Turn.turn_id: TurnId { loop_id, turn_index }` — the latter lives
    /// on `inner`.
    TurnNodeId
);
id_newtype!(
    /// Identifier of an `AgentCatalogEntry` composite instance (M5 page
    /// 13 / s03 catalogue cache).
    AgentCatalogEntryId
);
id_newtype!(
    /// Identifier of a `SystemAgentRuntimeStatus` composite instance (M5
    /// page 13 live-status tile).
    SystemAgentRuntimeStatusId
);

/// Typed payload-end discriminator for the [`Edge::Owns`] variant
/// (CH-25 / ADR-0060 §D60.1).
///
/// Carries the Agent→Org/Project ownership target as a typed enum so
/// the variant payload can name the concrete ID kind at compile time
/// (vs erasing to `NodeId`). The two-variant closed-set mirrors the
/// concept-doc claim *"Agent owns Organization"* + *"Agent create
/// Projects"* (core-philosophy.md:9, 23, 24).
///
/// Note (F1.b user-lock vs F1.a planner-recommendation): under F1.b
/// (NEW [`Edge::Owns`] variant) Org/Project STAY Principal-only per the
/// v0 ontology invariant at `principal_resource.rs:182-186`. This enum
/// is the resource-end carrier, NOT a `Resource`-trait relaxation. See
/// `m5_3/decisions/0060-agent-as-creator-and-owner.md` §D60.1 for the
/// full rationale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OwnedResourceId {
    Org(OrgId),
    Project(ProjectId),
}

impl OwnedResourceId {
    /// Convert the typed payload into a `NodeId` for graph-shape consumers
    /// that don't care which sub-variant is in play (e.g., wire-format
    /// serialization). Read-only — the typed kind is preserved on the
    /// owning [`Edge::Owns`] variant payload.
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::Org(id) => NodeId::from_uuid(*id.as_uuid()),
            Self::Project(id) => NodeId::from_uuid(*id.as_uuid()),
        }
    }

    /// Stable canonical-name string for the resource kind ("org" /
    /// "project"). Used by the synth-grant URI builder in the Permission
    /// Check engine (CH-25 / ADR-0060 §D60.3).
    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::Org(_) => "org",
            Self::Project(_) => "project",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_by_default() {
        let a = AgentId::new();
        let b = AgentId::new();
        assert_ne!(a, b, "freshly generated IDs must differ");
    }

    #[test]
    fn ids_serialize_as_plain_uuid() {
        let id = AgentId::new();
        let json = serde_json::to_string(&id).expect("serialize");
        // Transparent repr: the JSON is just a quoted UUID string.
        assert_eq!(json, format!("\"{}\"", id.0));
    }

    #[test]
    fn owned_resource_id_org_round_trips_node_id() {
        // CH-25 / ADR-0060 §D60.1 — typed payload carrier for
        // `Edge::Owns`. `node_id()` is the lossy conversion for
        // graph-shape consumers; the typed kind is preserved on the
        // owning Edge variant.
        let org = OrgId::new();
        let owned = OwnedResourceId::Org(org);
        let node = owned.node_id();
        assert_eq!(node.as_uuid(), org.as_uuid());
        assert_eq!(owned.kind_name(), "org");
    }

    #[test]
    fn owned_resource_id_project_round_trips_node_id() {
        let project = ProjectId::new();
        let owned = OwnedResourceId::Project(project);
        let node = owned.node_id();
        assert_eq!(node.as_uuid(), project.as_uuid());
        assert_eq!(owned.kind_name(), "project");
    }

    #[test]
    fn owned_resource_id_serde_round_trips() {
        // Default external-tagging — wire form is `{"Org": "<uuid>"}` or
        // `{"Project": "<uuid>"}`. Round-trips losslessly across the
        // serde adapter.
        let project = ProjectId::new();
        let owned = OwnedResourceId::Project(project);
        let json = serde_json::to_string(&owned).expect("serialize");
        let back: OwnedResourceId = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, owned);
    }
}
