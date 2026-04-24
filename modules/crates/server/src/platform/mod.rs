//! Platform-admin surfaces — the M2 admin pages (02–05) mount their
//! business logic here. Each page gets its own submodule; handlers in
//! [`crate::handlers`] stay thin HTTP shims on top.

pub mod agents;
pub mod defaults;
pub mod mcp_servers;
pub mod model_providers;
pub mod orgs;
pub mod projects;
pub mod secrets;
pub mod sessions;
pub mod system_agents;
pub mod templates;
