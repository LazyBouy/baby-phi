//! `phi system-agent` — system-agent config subcommands (M5/P7).
//!
//! Surfaces the five page-13 routes from `/api/v0/orgs/:org_id/
//! system-agents/…` (D6.2 carryover):
//! - `list`    — `{standard, org_specific, recent_events}` buckets.
//! - `tune`    — adjust `parallelize`.
//! - `add`     — create a new org-specific system agent.
//! - `disable` — mark active=false (confirm required).
//! - `archive` — archive an org-specific system agent.
//!
//! Reuses M5/P6 wire contracts verbatim.
//!
//! ## phi-core leverage
//!
//! Q1 **none** at the CLI tier. The `AgentProfile` import lives in
//! `server::platform::system_agents::add` (M5/P6); the CLI is a thin
//! HTTP client.

use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use serde::Deserialize;
use server::config::ServerConfig;

use crate::exit::{
    EXIT_INTERNAL, EXIT_OK, EXIT_PRECONDITION_FAILED, EXIT_REJECTED, EXIT_TRANSPORT,
};
use crate::session_store;

#[derive(Debug, clap::Subcommand)]
pub enum SystemAgentCommand {
    /// List system agents with runtime-status tiles.
    List {
        #[arg(long = "org-id")]
        org_id: String,
        #[arg(long)]
        json: bool,
    },
    /// Tune a system agent's `parallelize` ceiling.
    Tune {
        #[arg(long = "org-id")]
        org_id: String,
        #[arg(long = "agent-id")]
        agent_id: String,
        #[arg(long)]
        parallelize: u32,
        #[arg(long)]
        json: bool,
    },
    /// Add a new org-specific system agent.
    Add {
        #[arg(long = "org-id")]
        org_id: String,
        #[arg(long = "display-name")]
        display_name: String,
        /// Profile-ref slug (maps to `AgentProfile.config_id`).
        #[arg(long = "profile-ref")]
        profile_ref: String,
        #[arg(long, default_value_t = 1)]
        parallelize: u32,
        /// Trigger: `session_end` / `edge_change` / `periodic` /
        /// `explicit` / `custom_event`.
        #[arg(long, default_value = "explicit")]
        trigger: String,
        #[arg(long)]
        json: bool,
    },
    /// Disable a system agent (confirm required).
    Disable {
        #[arg(long = "org-id")]
        org_id: String,
        #[arg(long = "agent-id")]
        agent_id: String,
        /// Must be supplied to complete the action (matches the
        /// server's `confirm: true` gate).
        #[arg(long)]
        confirm: bool,
        #[arg(long)]
        json: bool,
    },
    /// Archive an org-specific system agent. Platform-standard
    /// system agents cannot be archived; use `disable` instead.
    Archive {
        #[arg(long = "org-id")]
        org_id: String,
        #[arg(long = "agent-id")]
        agent_id: String,
        #[arg(long)]
        json: bool,
    },
}

pub async fn run(server_url_override: Option<String>, cmd: SystemAgentCommand) -> i32 {
    match cmd {
        SystemAgentCommand::List { org_id, json } => {
            list_impl(server_url_override, &org_id, json).await
        }
        SystemAgentCommand::Tune {
            org_id,
            agent_id,
            parallelize,
            json,
        } => tune_impl(server_url_override, &org_id, &agent_id, parallelize, json).await,
        SystemAgentCommand::Add {
            org_id,
            display_name,
            profile_ref,
            parallelize,
            trigger,
            json,
        } => {
            add_impl(
                server_url_override,
                &org_id,
                &display_name,
                &profile_ref,
                parallelize,
                &trigger,
                json,
            )
            .await
        }
        SystemAgentCommand::Disable {
            org_id,
            agent_id,
            confirm,
            json,
        } => disable_impl(server_url_override, &org_id, &agent_id, confirm, json).await,
        SystemAgentCommand::Archive {
            org_id,
            agent_id,
            json,
        } => archive_impl(server_url_override, &org_id, &agent_id, json).await,
    }
}

async fn list_impl(server_url_override: Option<String>, org_id: &str, json: bool) -> i32 {
    let (base, client) = match prepare(server_url_override) {
        Ok(x) => x,
        Err(code) => return code,
    };
    let url = format!("{base}/api/v0/orgs/{}/system-agents", urlencode(org_id));
    let res = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("phi: GET {url} failed: {e}");
            return EXIT_TRANSPORT;
        }
    };
    let status = res.status();
    if !status.is_success() {
        return report_api_error(res, status).await;
    }
    let out: serde_json::Value = match res.json().await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("phi: decode response: {e}");
            return EXIT_INTERNAL;
        }
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
    } else {
        for bucket in ["standard", "org_specific"] {
            let rows = out
                .get(bucket)
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            println!("{bucket} ({})", rows.len());
            for row in rows {
                let id = row.get("agent_id").and_then(|v| v.as_str()).unwrap_or("?");
                let name = row
                    .get("display_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");
                let parallelize = row.get("parallelize").and_then(|v| v.as_u64()).unwrap_or(0);
                println!("  {id}  {name}  parallelize={parallelize}");
            }
        }
        let events = out
            .get("recent_events")
            .and_then(|v| v.as_array())
            .map(|v| v.len())
            .unwrap_or(0);
        println!("recent events: {events}");
    }
    EXIT_OK
}

async fn tune_impl(
    server_url_override: Option<String>,
    org_id: &str,
    agent_id: &str,
    parallelize: u32,
    json: bool,
) -> i32 {
    let (base, client) = match prepare(server_url_override) {
        Ok(x) => x,
        Err(code) => return code,
    };
    let body = serde_json::json!({ "parallelize": parallelize });
    let url = format!(
        "{base}/api/v0/orgs/{}/system-agents/{}",
        urlencode(org_id),
        urlencode(agent_id),
    );
    let res = match client.patch(&url).json(&body).send().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("phi: PATCH {url} failed: {e}");
            return EXIT_TRANSPORT;
        }
    };
    let status = res.status();
    if !status.is_success() {
        return report_api_error(res, status).await;
    }
    let out: serde_json::Value = match res.json().await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("phi: decode response: {e}");
            return EXIT_INTERNAL;
        }
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
    } else {
        println!("system agent tuned");
        if let Some(id) = out.get("agent_id").and_then(|v| v.as_str()) {
            println!("  id:              {id}");
        }
        if let Some(event_id) = out.get("audit_event_id").and_then(|v| v.as_str()) {
            println!("  audit event:     {event_id}");
        }
    }
    EXIT_OK
}

async fn add_impl(
    server_url_override: Option<String>,
    org_id: &str,
    display_name: &str,
    profile_ref: &str,
    parallelize: u32,
    trigger: &str,
    json: bool,
) -> i32 {
    let (base, client) = match prepare(server_url_override) {
        Ok(x) => x,
        Err(code) => return code,
    };
    let body = serde_json::json!({
        "display_name": display_name,
        "profile_ref": profile_ref,
        "parallelize": parallelize,
        "trigger": trigger,
    });
    let url = format!("{base}/api/v0/orgs/{}/system-agents", urlencode(org_id));
    let res = match client.post(&url).json(&body).send().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("phi: POST {url} failed: {e}");
            return EXIT_TRANSPORT;
        }
    };
    let status = res.status();
    if !status.is_success() {
        return report_api_error(res, status).await;
    }
    let out: serde_json::Value = match res.json().await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("phi: decode response: {e}");
            return EXIT_INTERNAL;
        }
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
    } else {
        println!("system agent added");
        if let Some(id) = out.get("agent_id").and_then(|v| v.as_str()) {
            println!("  id:              {id}");
        }
        if let Some(event_id) = out.get("audit_event_id").and_then(|v| v.as_str()) {
            println!("  audit event:     {event_id}");
        }
    }
    EXIT_OK
}

async fn disable_impl(
    server_url_override: Option<String>,
    org_id: &str,
    agent_id: &str,
    confirm: bool,
    json: bool,
) -> i32 {
    let (base, client) = match prepare(server_url_override) {
        Ok(x) => x,
        Err(code) => return code,
    };
    let body = serde_json::json!({ "confirm": confirm });
    let url = format!(
        "{base}/api/v0/orgs/{}/system-agents/{}/disable",
        urlencode(org_id),
        urlencode(agent_id),
    );
    let res = match client.post(&url).json(&body).send().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("phi: POST {url} failed: {e}");
            return EXIT_TRANSPORT;
        }
    };
    let status = res.status();
    if !status.is_success() {
        return report_api_error(res, status).await;
    }
    let out: serde_json::Value = match res.json().await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("phi: decode response: {e}");
            return EXIT_INTERNAL;
        }
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
    } else {
        let was_standard = out
            .get("was_standard")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if was_standard {
            println!("system agent disabled (STANDARD — re-enable via add)");
        } else {
            println!("system agent disabled");
        }
        if let Some(id) = out.get("agent_id").and_then(|v| v.as_str()) {
            println!("  id:              {id}");
        }
    }
    EXIT_OK
}

async fn archive_impl(
    server_url_override: Option<String>,
    org_id: &str,
    agent_id: &str,
    json: bool,
) -> i32 {
    let (base, client) = match prepare(server_url_override) {
        Ok(x) => x,
        Err(code) => return code,
    };
    let url = format!(
        "{base}/api/v0/orgs/{}/system-agents/{}/archive",
        urlencode(org_id),
        urlencode(agent_id),
    );
    let res = match client.post(&url).json(&serde_json::json!({})).send().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("phi: POST {url} failed: {e}");
            return EXIT_TRANSPORT;
        }
    };
    let status = res.status();
    if !status.is_success() {
        return report_api_error(res, status).await;
    }
    let out: serde_json::Value = match res.json().await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("phi: decode response: {e}");
            return EXIT_INTERNAL;
        }
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
    } else {
        println!("system agent archived");
        if let Some(id) = out.get("agent_id").and_then(|v| v.as_str()) {
            println!("  id:              {id}");
        }
    }
    EXIT_OK
}

// ---------------------------------------------------------------------------
// Shared plumbing
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct ApiErrorWire {
    code: String,
    message: String,
}

fn prepare(override_url: Option<String>) -> std::result::Result<(String, reqwest::Client), i32> {
    let base = match resolve_base_url(override_url) {
        Ok(u) => u,
        Err(e) => {
            eprintln!("phi: failed to resolve server URL: {e:#}");
            return Err(EXIT_INTERNAL);
        }
    };
    let client = match build_authed_client() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("phi: {e}");
            return Err(EXIT_PRECONDITION_FAILED);
        }
    };
    Ok((base, client))
}

fn build_authed_client() -> Result<reqwest::Client> {
    let path = session_store::default_session_path().context("resolve session-store path")?;
    let session = match session_store::load(&path) {
        Ok(s) => s,
        Err(session_store::SessionStoreError::NotFound { .. }) => {
            anyhow::bail!(
                "no saved session at {} — run `phi bootstrap claim --credential <…>` first",
                path.display()
            );
        }
        Err(e) => anyhow::bail!("failed to load saved session: {e}"),
    };
    let mut headers = HeaderMap::new();
    let cookie = format!("phi_kernel_session={}", session.cookie_value);
    headers.insert(
        COOKIE,
        HeaderValue::from_str(&cookie).context("cookie value is not a valid header")?,
    );
    reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .default_headers(headers)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .context("build reqwest client")
}

fn resolve_base_url(override_url: Option<String>) -> Result<String> {
    if let Some(u) = override_url {
        let mut u = u;
        while u.ends_with('/') {
            u.pop();
        }
        return Ok(u);
    }
    let cfg = ServerConfig::load().context("loading ServerConfig for default server URL")?;
    let scheme = if cfg.server.tls.is_some() {
        "https"
    } else {
        "http"
    };
    let host = if cfg.server.host == "0.0.0.0" {
        "127.0.0.1".to_string()
    } else {
        cfg.server.host.clone()
    };
    Ok(format!("{scheme}://{host}:{}", cfg.server.port))
}

fn urlencode(raw: &str) -> String {
    raw.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => {
                let mut buf = [0; 4];
                let bytes = c.encode_utf8(&mut buf).as_bytes().to_vec();
                bytes
                    .into_iter()
                    .map(|b| format!("%{b:02X}"))
                    .collect::<Vec<_>>()
                    .join("")
            }
        })
        .collect()
}

async fn report_api_error(res: reqwest::Response, status: reqwest::StatusCode) -> i32 {
    match res.json::<ApiErrorWire>().await {
        Ok(err) => {
            eprintln!("phi: rejected ({}): {}", err.code, err.message);
            EXIT_REJECTED
        }
        Err(_) => {
            eprintln!("phi: rejected (HTTP {status}) — non-JSON body");
            EXIT_REJECTED
        }
    }
}
