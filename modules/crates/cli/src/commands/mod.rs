//! Subcommand implementations. Each module returns a `i32` exit code so
//! `main.rs` can pass it to `std::process::exit`.

pub mod agent;
pub mod bootstrap;
pub mod completion;
pub mod login;
pub mod mcp_server;
pub mod model_provider;
pub mod platform_defaults;
pub mod secrets;
