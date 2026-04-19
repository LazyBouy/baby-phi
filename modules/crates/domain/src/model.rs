//! Graph model — placeholder module.
//!
//! Full set of node and edge types lands in M1. See
//! `docs/specs/v0/concepts/ontology.md` for the target shape:
//! 9 fundamentals, 8 composites, 31 node kinds, 56+ edge kinds.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Opaque, DB-independent node identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub Uuid);

impl NodeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}
