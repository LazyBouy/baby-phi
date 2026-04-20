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
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
