//! MCP health-probe — thin wrapper around phi-core's [`McpClient`].
//!
//! phi-core leverage (§1.5): `McpClient::connect_stdio`,
//! `McpClient::connect_http`, `McpClient::list_tools()` are the single
//! source of truth for MCP transport. phi adds only a
//! timeout+retry envelope (phi-core has no probe abstraction of its
//! own; §1.5 🚫 — this is the only hand-rolled MCP code in M2).
//!
//! **M2 scope**: shape-only. The [`probe_mcp_health`] fn is wired so
//! unit tests can exercise the timeout/error path, but no handler or
//! scheduled task calls it in production. The real scheduled probe
//! lands in M7b per plan §G5.
//!
//! Endpoint parsing — the MCP `endpoint` string follows phi-core's
//! transport-argument convention:
//!
//! - `stdio:///path/to/server [arg1 arg2 …]` — the `stdio:///` prefix
//!   is a phi convention to disambiguate from HTTP URLs; after
//!   stripping the prefix, the remainder is whitespace-split to
//!   `(command, args)` and passed to [`McpClient::connect_stdio`].
//! - `http://…` / `https://…` — passed verbatim to
//!   [`McpClient::connect_http`].

use std::time::Duration;

use phi_core::mcp::client::McpClient;
use tokio::time::timeout;
use tracing::{debug, warn};

/// Outcome of a single probe attempt. Wire shape mirrors
/// [`domain::model::RuntimeStatus`] semantics.
#[derive(Debug, Clone)]
pub enum ProbeResult {
    /// `list_tools()` returned the tool catalogue. Carries the tool
    /// count for the summary.
    Ok { tool_count: usize },
    /// The probe timed out or returned an error. Carries a short
    /// reason string for the audit event.
    Degraded { reason: String },
}

/// Probe timeout per attempt. 2 s matches the plan spec §P6.
pub const PROBE_TIMEOUT: Duration = Duration::from_secs(2);

/// Probe an MCP server by constructing a [`McpClient`] from the stored
/// `endpoint` string and calling `list_tools()` with a 2 s timeout.
///
/// M2 makes **no retry attempts** — the single-attempt contract keeps
/// the unit tests deterministic. M7b's scheduled probe wraps this in
/// exponential backoff + the 3-retry budget from plan §P6.
pub async fn probe_mcp_health(endpoint: &str) -> ProbeResult {
    let client_result = timeout(PROBE_TIMEOUT, connect(endpoint)).await;
    let client = match client_result {
        Ok(Ok(c)) => c,
        Ok(Err(e)) => {
            warn!(error = %e, "mcp probe: connect failed");
            return ProbeResult::Degraded {
                reason: format!("connect failed: {e}"),
            };
        }
        Err(_) => {
            warn!("mcp probe: connect timed out");
            return ProbeResult::Degraded {
                reason: "connect timed out".into(),
            };
        }
    };
    match timeout(PROBE_TIMEOUT, client.list_tools()).await {
        Ok(Ok(tools)) => {
            debug!(tool_count = tools.len(), "mcp probe: ok");
            ProbeResult::Ok {
                tool_count: tools.len(),
            }
        }
        Ok(Err(e)) => {
            warn!(error = %e, "mcp probe: list_tools failed");
            ProbeResult::Degraded {
                reason: format!("list_tools failed: {e}"),
            }
        }
        Err(_) => {
            warn!("mcp probe: list_tools timed out");
            ProbeResult::Degraded {
                reason: "list_tools timed out".into(),
            }
        }
    }
}

async fn connect(endpoint: &str) -> Result<McpClient, String> {
    if let Some(rest) = endpoint.strip_prefix("stdio:///") {
        let mut parts = rest.split_whitespace();
        let command = parts
            .next()
            .ok_or_else(|| "stdio endpoint missing command".to_string())?;
        let args: Vec<&str> = parts.collect();
        McpClient::connect_stdio(command, &args, None)
            .await
            .map_err(|e| e.to_string())
    } else if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
        McpClient::connect_http(endpoint)
            .await
            .map_err(|e| e.to_string())
    } else {
        Err(format!(
            "unsupported endpoint scheme (expected stdio:///… or http[s]://…): `{endpoint}`"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn unsupported_scheme_is_degraded() {
        let r = probe_mcp_health("tcp://example.com:1234").await;
        assert!(matches!(r, ProbeResult::Degraded { .. }));
    }

    #[tokio::test]
    async fn stdio_missing_command_is_degraded() {
        let r = probe_mcp_health("stdio:///").await;
        assert!(matches!(r, ProbeResult::Degraded { .. }));
    }

    #[tokio::test]
    async fn http_to_nonexistent_host_degrades_within_timeout() {
        // 10.255.255.1 is non-routable on most hosts — connect will
        // either time out within PROBE_TIMEOUT or error fast; either
        // way it must classify as Degraded.
        let r = probe_mcp_health("http://10.255.255.1:1/").await;
        assert!(matches!(r, ProbeResult::Degraded { .. }));
    }
}
