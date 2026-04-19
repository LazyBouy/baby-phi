//! Permission Check engine — placeholder module.
//!
//! The full 6-step algorithm (Catalogue precondition → rule match → selector
//! match → scope intersection → parallelize → retention) lands in M1. See
//! `docs/specs/v0/concepts/permissions/04-manifest-and-resolution.md`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Decision {
    Allowed,
    Denied { failed_step: u8, reason: String },
}
