//! The graph model — every type that populates the phi ontology.
//!
//! Sub-modules:
//! - [`ids`] — strongly-typed `Uuid` newtypes for every entity.
//! - [`fundamentals`] — 9 atomic resource classes.
//! - [`composites`] — 8 named bundles of fundamentals.
//! - [`nodes`] — 37 node types (structs + `NodeKind` inventory enum).
//! - [`edges`] — 67-variant `Edge` enum.
//!
//! Source of truth for the inventory: `docs/specs/v0/concepts/ontology.md` +
//! `docs/specs/v0/concepts/permissions/01-resource-ontology.md`.

pub mod composites;
pub mod composites_m2;
pub mod composites_m3;
pub mod composites_m4;
pub mod composites_m5;
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
pub use composites_m3::{ConsentPolicy, OrganizationDefaultsSnapshot, TokenBudgetPool};
pub use composites_m4::{
    AgentExecutionLimitsOverride, KeyResult, KeyResultStatus, MeasurementType, Objective,
    ObjectiveStatus, OkrValue, ResourceBoundaries,
};
pub use composites_m5::{
    AgentCatalogEntry, SessionDetail, ShapeBPendingProject, SystemAgentRuntimeStatus,
};
pub use edges::{Edge, EDGE_KIND_NAMES};
pub use fundamentals::Fundamental;
pub use ids::{
    AgentCatalogEntryId, AgentId, AuditEventId, AuthRequestId, ConsentId, EdgeId, GrantId, LoopId,
    McpServerId, MemoryId, ModelProviderId, NodeId, OrgId, ProjectId, SecretId, SessionId,
    SystemAgentRuntimeStatusId, TemplateId, TurnNodeId, UserId,
};
pub use nodes::{
    Agent, AgentKind, AgentProfile, AgentRole, ApproverSlot, ApproverSlotState, AuthRequest,
    AuthRequestState, Channel, ChannelKind, Consent, Grant, InboxObject, LoopRecordNode, Memory,
    NodeKind, Organization, OutboxObject, PrincipalRef, Project, ProjectShape, ProjectStatus,
    ResourceRef, ResourceSlot, ResourceSlotState, Session, SessionGovernanceState, Template,
    TemplateKind, ToolAuthorityManifest, TurnNode, User,
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
    fn ontology_has_sixty_nine_edge_kinds() {
        assert_eq!(EDGE_KIND_NAMES.len(), 69);
    }
}
