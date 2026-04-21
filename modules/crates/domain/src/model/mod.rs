//! The graph model — every type that populates the baby-phi ontology.
//!
//! Sub-modules:
//! - [`ids`] — strongly-typed `Uuid` newtypes for every entity.
//! - [`fundamentals`] — 9 atomic resource classes.
//! - [`composites`] — 8 named bundles of fundamentals.
//! - [`nodes`] — 37 node types (structs + `NodeKind` inventory enum).
//! - [`edges`] — 66-variant `Edge` enum.
//!
//! Source of truth for the inventory: `docs/specs/v0/concepts/ontology.md` +
//! `docs/specs/v0/concepts/permissions/01-resource-ontology.md`.

pub mod composites;
pub mod composites_m2;
pub mod edges;
pub mod fundamentals;
pub mod ids;
pub mod nodes;
pub mod principal_resource;

pub use composites::Composite;
pub use composites_m2::{
    ExternalService, ExternalServiceKind, ModelRuntime, PlatformDefaults, ProviderKind,
    RuntimeStatus, SecretCredential, SecretRef, TenantSet,
};
pub use edges::{Edge, EDGE_KIND_NAMES};
pub use fundamentals::Fundamental;
pub use ids::{
    AgentId, AuditEventId, AuthRequestId, ConsentId, EdgeId, GrantId, McpServerId, MemoryId,
    ModelProviderId, NodeId, OrgId, ProjectId, SecretId, SessionId, TemplateId, UserId,
};
pub use nodes::{
    Agent, AgentKind, AgentProfile, ApproverSlot, ApproverSlotState, AuthRequest, AuthRequestState,
    Channel, ChannelKind, Consent, Grant, InboxObject, Memory, NodeKind, Organization,
    OutboxObject, PrincipalRef, ResourceRef, ResourceSlot, ResourceSlotState, Template,
    TemplateKind, ToolAuthorityManifest, User,
};
pub use principal_resource::{Principal, Resource};

#[cfg(test)]
mod tests {
    //! Cross-cutting counts — the invariants that the plan's verification
    //! matrix (C1) asserts must hold.

    use super::*;

    #[test]
    fn ontology_has_nine_fundamentals() {
        assert_eq!(Fundamental::ALL.len(), 9);
    }

    #[test]
    fn ontology_has_eight_composites() {
        assert_eq!(Composite::ALL.len(), 8);
    }

    #[test]
    fn ontology_has_thirty_seven_node_kinds() {
        assert_eq!(NodeKind::ALL.len(), 37);
    }

    #[test]
    fn ontology_has_sixty_six_edge_kinds() {
        assert_eq!(EDGE_KIND_NAMES.len(), 66);
    }
}
