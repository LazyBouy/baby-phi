//! HTTP handlers.
//!
//! Each sub-module groups one resource family (bootstrap, orgs, grants,
//! …). M1/P6 ships only [`bootstrap`]; every M2+ milestone adds another
//! sibling module.

pub mod bootstrap;
pub mod platform_secrets;
