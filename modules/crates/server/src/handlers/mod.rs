//! HTTP handlers.
//!
//! Each sub-module groups one resource family (bootstrap, orgs, grants,
//! …). M1/P6 ships only [`bootstrap`]; every M2+ milestone adds another
//! sibling module.

pub mod bootstrap;
pub mod platform_defaults;
pub mod platform_mcp_servers;
pub mod platform_model_providers;
pub mod platform_secrets;
