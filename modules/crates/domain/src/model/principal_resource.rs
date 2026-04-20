//! Sealed marker traits for the **Principal** and **Resource** type unions.
//!
//! The concept doc defines:
//!
//! - **Principal** — any entity that can hold authority: Agent, User,
//!   Organization, Project (plus the `system:` axioms, which are represented
//!   by string-keyed `PrincipalRef::System`, not an ID newtype).
//! - **Resource** — any entity that can be owned: every node that can end
//!   up on the receiving end of an `OWNED_BY` edge. In v0.1 this is
//!   effectively every node kind; the marker trait is therefore broadly
//!   implemented on each `*Id` newtype plus the generic [`NodeId`].
//!
//! ## Why both layers exist
//!
//! SurrealDB cannot enforce the Resource/Principal union at the schema
//! layer — the three relations (`owned_by`, `created`, `allocated_to`) are
//! declared without typed `FROM`/`TO` endpoints because partitioning into
//! concrete-pair tables would balloon to ~150 relations (see ADR-0015 +
//! ADR-0009). Instead, **Rust gives us compile-time enforcement** via:
//!
//! 1. The [`Principal`] and [`Resource`] marker traits declared here
//!    (sealed so downstream crates cannot add rogue impls).
//! 2. Typed `Edge::new_owned_by` / `new_created` / `new_allocated_to`
//!    constructors in [`crate::model::edges`] that accept only
//!    `&impl Principal` / `&impl Resource` arguments.
//! 3. Typed repository helpers (`upsert_ownership` / `upsert_creation` /
//!    `upsert_allocation`) in [`crate::repository`] that do the same at the
//!    persistence boundary.
//!
//! A caller who tries to pass, say, a `ConsentId` where a `Principal` is
//! required fails the build with a clear `trait bound not satisfied` error.
//!
//! ## Sealing pattern
//!
//! Both traits extend a crate-private [`sealed::Sealed`] supertrait. Rust's
//! coherence rules then forbid any external crate from adding its own
//! `Principal`/`Resource` impls, because implementing `sealed::Sealed` on a
//! foreign type is impossible.

use super::ids::{
    AgentId, AuthRequestId, ConsentId, GrantId, MemoryId, NodeId, OrgId, ProjectId, SessionId,
    TemplateId, UserId,
};

pub(crate) mod sealed {
    /// Crate-private supertrait used to seal [`super::Principal`] and
    /// [`super::Resource`]. External crates cannot implement this.
    pub trait Sealed {}
}

/// Types that can hold authority — the concept doc's **Principal** union.
///
/// Implementers in v0.1 are: [`AgentId`], [`UserId`], [`OrgId`],
/// [`ProjectId`]. A [`NodeId`] is NOT a Principal (it's too generic —
/// accept it and you lose the whole point of the trait).
pub trait Principal: sealed::Sealed {
    /// The underlying generic [`NodeId`] for persistence. Implementations
    /// are one-liners that wrap the inner `Uuid` in a `NodeId`.
    fn node_id(&self) -> NodeId;
}

/// Types that can be owned — the concept doc's **Resource** union.
///
/// v0.1 treats every node kind as potentially ownable: Agents (dual-role
/// as Principal + Resource), Memories, Sessions, AuthRequests, etc., plus
/// the generic [`NodeId`] for everything else. Growing this list is the
/// normal way to widen the set of things that can participate in
/// ownership edges.
pub trait Resource: sealed::Sealed {
    /// The underlying generic [`NodeId`] for persistence.
    fn node_id(&self) -> NodeId;
}

// ----------------------------------------------------------------------------
// Sealed impls. One per ID newtype that participates.
// ----------------------------------------------------------------------------

macro_rules! seal {
    ($ty:ty) => {
        impl sealed::Sealed for $ty {}
    };
}

// Every ID newtype that participates in either trait (or both) must `seal!`
// itself once. Keeping the list in one place makes it easy to audit that
// the intended universe is correct.
seal!(NodeId);
seal!(AgentId);
seal!(UserId);
seal!(OrgId);
seal!(ProjectId);
seal!(SessionId);
seal!(MemoryId);
seal!(AuthRequestId);
seal!(GrantId);
seal!(TemplateId);
seal!(ConsentId);

// --- Principal ---------------------------------------------------------------
//
// Matches the concept doc's §Governance Wiring: "A Principal is any entity
// that can hold authority: Agent, Project, Organization, or User."

impl Principal for AgentId {
    fn node_id(&self) -> NodeId {
        NodeId::from_uuid(*self.as_uuid())
    }
}

impl Principal for UserId {
    fn node_id(&self) -> NodeId {
        NodeId::from_uuid(*self.as_uuid())
    }
}

impl Principal for OrgId {
    fn node_id(&self) -> NodeId {
        NodeId::from_uuid(*self.as_uuid())
    }
}

impl Principal for ProjectId {
    fn node_id(&self) -> NodeId {
        NodeId::from_uuid(*self.as_uuid())
    }
}

// --- Resource ----------------------------------------------------------------
//
// Every ownable node kind. Agents are dual-role (both Principal and
// Resource) — this matches the concept doc's note that agents are "both
// principals and resources".

impl Resource for NodeId {
    fn node_id(&self) -> NodeId {
        *self
    }
}

impl Resource for AgentId {
    fn node_id(&self) -> NodeId {
        NodeId::from_uuid(*self.as_uuid())
    }
}

impl Resource for SessionId {
    fn node_id(&self) -> NodeId {
        NodeId::from_uuid(*self.as_uuid())
    }
}

impl Resource for MemoryId {
    fn node_id(&self) -> NodeId {
        NodeId::from_uuid(*self.as_uuid())
    }
}

impl Resource for AuthRequestId {
    fn node_id(&self) -> NodeId {
        NodeId::from_uuid(*self.as_uuid())
    }
}

impl Resource for GrantId {
    fn node_id(&self) -> NodeId {
        NodeId::from_uuid(*self.as_uuid())
    }
}

impl Resource for TemplateId {
    fn node_id(&self) -> NodeId {
        NodeId::from_uuid(*self.as_uuid())
    }
}

impl Resource for ConsentId {
    fn node_id(&self) -> NodeId {
        NodeId::from_uuid(*self.as_uuid())
    }
}

// NOTE: OrgId, UserId, ProjectId are Principals only — they are never the
// target of an OWNED_BY edge in the v0 ontology. Any caller who tries to
// pass them where a Resource is required gets a compile-time error.
// If a future milestone needs org-owned-by-org or similar, we add
// `impl Resource for OrgId` at that time and note the relaxation here.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn principal_node_id_round_trips_uuid() {
        let agent = AgentId::new();
        let node = <AgentId as Principal>::node_id(&agent);
        assert_eq!(agent.as_uuid(), node.as_uuid());
    }

    #[test]
    fn resource_node_id_round_trips_uuid() {
        let mem = MemoryId::new();
        let node = <MemoryId as Resource>::node_id(&mem);
        assert_eq!(mem.as_uuid(), node.as_uuid());
    }

    #[test]
    fn agent_is_both_principal_and_resource() {
        // Compile-time check: `AgentId` implements both traits. If either
        // impl is ever removed, this test stops compiling, which catches
        // a very specific regression (the dual-role property is a
        // concept-doc invariant).
        fn assert_both<T: Principal + Resource>() {}
        assert_both::<AgentId>();
    }

    #[test]
    fn node_id_is_resource_only() {
        // Compile-time check: a generic `NodeId` is a Resource (catch-all
        // for ownable things) but NOT a Principal (would defeat the
        // type-safety of ownership edges).
        fn assert_resource<T: Resource>() {}
        assert_resource::<NodeId>();
        // The negative assertion — that `NodeId: !Principal` — cannot be
        // expressed as a stable unit test in current Rust (negative bounds
        // are unstable). It is instead covered by the `trybuild`
        // compile-fail fixture `node_id_as_principal_fails.rs`.
    }

    #[test]
    fn org_user_project_are_principal_only() {
        // Compile-time assertion: the three pure-Principal ID types
        // implement Principal. Absence of a Resource impl is again covered
        // by trybuild in P2.d.
        fn assert_principal<T: Principal>() {}
        assert_principal::<OrgId>();
        assert_principal::<UserId>();
        assert_principal::<ProjectId>();
    }
}
