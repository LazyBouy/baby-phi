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
}
