//! `baby-phi bootstrap {status,claim}` HTTP clients.
//!
//! Thin wrappers around `reqwest` that hit the P6 endpoints, pretty-print
//! the result, and map HTTP status + error codes to CLI exit codes:
//!
//! - `0` — success.
//! - `1` — transport / IO failure (server unreachable, timeout).
//! - `2` — server returned 4xx with a known `code` (already-claimed,
//!   invalid credential, etc.). Stable machine-readable.
//! - `3` — server returned 5xx or otherwise-unexpected response.
//!
//! The exit-code split lets shell scripts distinguish "retry later"
//! (code 1) from "user-facing validation error, fix input" (code 2) from
//! "server broken, escalate" (code 3).

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use server::ServerConfig;

use crate::{BootstrapCommand, ChannelKindArg};

const EXIT_OK: i32 = 0;
const EXIT_TRANSPORT: i32 = 1;
const EXIT_REJECTED: i32 = 2;
const EXIT_INTERNAL: i32 = 3;

// ---- Wire types (match server/src/handlers/bootstrap.rs) -------------------

#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[allow(dead_code)]
enum StatusWire {
    Claimed {
        claimed: bool,
        admin_agent_id: String,
    },
    Unclaimed {
        claimed: bool,
        #[serde(default)]
        awaiting_credential: bool,
    },
}

#[derive(Debug, Serialize)]
struct ClaimBody<'a> {
    bootstrap_credential: &'a str,
    display_name: &'a str,
    channel: ChannelBody<'a>,
}

#[derive(Debug, Serialize)]
struct ChannelBody<'a> {
    kind: &'a str,
    handle: &'a str,
}

#[derive(Debug, Deserialize)]
struct ClaimSuccess {
    human_agent_id: String,
    inbox_id: String,
    outbox_id: String,
    grant_id: String,
    bootstrap_auth_request_id: String,
    audit_event_id: String,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    code: String,
    message: String,
}

// ---- Entry point -----------------------------------------------------------

pub async fn run(server_url_override: Option<String>, cmd: BootstrapCommand) -> i32 {
    let base = match resolve_base_url(server_url_override) {
        Ok(u) => u,
        Err(e) => {
            eprintln!("baby-phi: failed to resolve server URL: {e:#}");
            return EXIT_INTERNAL;
        }
    };
    match cmd {
        BootstrapCommand::Status => status(&base).await,
        BootstrapCommand::Claim {
            credential,
            display_name,
            channel_kind,
            channel_handle,
        } => {
            claim(
                &base,
                &credential,
                &display_name,
                channel_kind,
                &channel_handle,
            )
            .await
        }
    }
}

fn resolve_base_url(override_url: Option<String>) -> Result<String> {
    if let Some(u) = override_url {
        return Ok(strip_trailing_slash(u));
    }
    // Fall back to the layered ServerConfig. Scheme defaults to https if
    // TLS is configured, else http.
    let cfg = ServerConfig::load().context("loading ServerConfig for default server URL")?;
    let scheme = if cfg.server.tls.is_some() {
        "https"
    } else {
        "http"
    };
    let host = if cfg.server.host == "0.0.0.0" {
        // The server binds 0.0.0.0 by default, but the CLI can't call
        // a bind-address; 127.0.0.1 is the right client default.
        "127.0.0.1".to_string()
    } else {
        cfg.server.host.clone()
    };
    Ok(format!("{scheme}://{host}:{}", cfg.server.port))
}

fn strip_trailing_slash(mut u: String) -> String {
    while u.ends_with('/') {
        u.pop();
    }
    u
}

// ---- `baby-phi bootstrap status` -------------------------------------------

async fn status(base: &str) -> i32 {
    let url = format!("{base}/api/v0/bootstrap/status");
    let client = match reqwest::Client::builder().build() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("baby-phi: failed to build HTTP client: {e}");
            return EXIT_INTERNAL;
        }
    };
    let res = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("baby-phi: request to {url} failed: {e}");
            return EXIT_TRANSPORT;
        }
    };
    let status = res.status();
    if !status.is_success() {
        eprintln!("baby-phi: unexpected status {status} from {url}");
        return if status.is_server_error() {
            EXIT_INTERNAL
        } else {
            EXIT_REJECTED
        };
    }
    let body: StatusWire = match res.json().await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("baby-phi: failed to decode status body: {e}");
            return EXIT_INTERNAL;
        }
    };
    match body {
        StatusWire::Claimed { admin_agent_id, .. } => {
            println!("platform admin already claimed");
            println!("  admin_agent_id: {admin_agent_id}");
        }
        StatusWire::Unclaimed { .. } => {
            println!("platform admin NOT yet claimed");
            println!(
                "  next step: run `baby-phi bootstrap claim --credential bphi-bootstrap-… \\\n    --display-name '…' --channel-kind slack --channel-handle @you`"
            );
        }
    }
    EXIT_OK
}

// ---- `baby-phi bootstrap claim` --------------------------------------------

async fn claim(
    base: &str,
    credential: &str,
    display_name: &str,
    channel_kind: ChannelKindArg,
    channel_handle: &str,
) -> i32 {
    let url = format!("{base}/api/v0/bootstrap/claim");
    let body = ClaimBody {
        bootstrap_credential: credential,
        display_name,
        channel: ChannelBody {
            kind: channel_kind.as_wire(),
            handle: channel_handle,
        },
    };
    let client = match reqwest::Client::builder().build() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("baby-phi: failed to build HTTP client: {e}");
            return EXIT_INTERNAL;
        }
    };
    let res = match client.post(&url).json(&body).send().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("baby-phi: request to {url} failed: {e}");
            return EXIT_TRANSPORT;
        }
    };
    let status = res.status();
    if status.as_u16() == 201 {
        let success: ClaimSuccess = match res.json().await {
            Ok(b) => b,
            Err(e) => {
                eprintln!("baby-phi: failed to decode claim response: {e}");
                return EXIT_INTERNAL;
            }
        };
        println!("platform admin claimed successfully");
        println!("  human_agent_id:            {}", success.human_agent_id);
        println!("  inbox_id:                  {}", success.inbox_id);
        println!("  outbox_id:                 {}", success.outbox_id);
        println!("  grant_id:                  {}", success.grant_id);
        println!(
            "  bootstrap_auth_request_id: {}",
            success.bootstrap_auth_request_id
        );
        println!("  audit_event_id:            {}", success.audit_event_id);
        println!();
        println!(
            "Next step: continue to the M2 platform-admin journey (model-provider registration)."
        );
        return EXIT_OK;
    }

    // Error path — try to decode the server's `{code, message}` envelope.
    let code_num = status.as_u16();
    match res.json::<ApiError>().await {
        Ok(err) => {
            eprintln!("baby-phi: claim rejected ({}): {}", err.code, err.message);
            if status.is_server_error() {
                EXIT_INTERNAL
            } else {
                EXIT_REJECTED
            }
        }
        Err(e) => {
            eprintln!("baby-phi: claim failed with HTTP {code_num} and no error body: {e}");
            if status.is_server_error() {
                EXIT_INTERNAL
            } else {
                EXIT_REJECTED
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_trailing_slash_removes_single_slash() {
        assert_eq!(strip_trailing_slash("http://a/".into()), "http://a");
    }

    #[test]
    fn strip_trailing_slash_removes_multiple_slashes() {
        assert_eq!(strip_trailing_slash("http://a///".into()), "http://a");
    }

    #[test]
    fn strip_trailing_slash_leaves_clean_url_untouched() {
        assert_eq!(
            strip_trailing_slash("http://a:8080".into()),
            "http://a:8080"
        );
    }
}
