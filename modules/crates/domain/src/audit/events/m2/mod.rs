//! M2 audit-event builders — one module per admin page.
//!
//! - [`secrets`] — page 04 (credentials vault). Ships with M2/P4.
//! - [`providers`] — page 02 (model providers). Ships with M2/P5.
//! - [`mcp`] — page 03 (MCP / external services). Ships with M2/P6.
//! - [`defaults`] — page 05 (platform defaults). Ships with M2/P7.

pub mod defaults;
pub mod mcp;
pub mod providers;
pub mod secrets;
