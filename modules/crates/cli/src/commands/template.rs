//! `phi template` — authority-template adoption subcommands (M5/P7).
//!
//! Surfaces the five page-12 routes from `/api/v0/orgs/:org_id/
//! authority-templates/…` as first-class CLI:
//! - `list`   — bucketed listing (pending / active / revoked / available).
//! - `approve`/`deny` — slot-level decisions on pending adoption ARs.
//! - `adopt`  — inline adoption (auto-approved Template-E shape).
//! - `revoke` — forward-only cascade revoke of all descending grants.
//!
//! Reuses M5/P5 wire contracts verbatim (D5.1 carryover).
//!
//! ## phi-core leverage
//!
//! Q1 **none**. Authority templates + grants are pure phi governance.

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
pub enum TemplateCommand {
    /// List authority-template adoptions, bucketed by state.
    List {
        #[arg(long = "org-id")]
        org_id: String,
        #[arg(long)]
        json: bool,
    },
    /// Approve the pending adoption AR for `--kind`.
    Approve {
        #[arg(long = "org-id")]
        org_id: String,
        /// Template kind (`a`/`b`/`c`/`d`).
        #[arg(long)]
        kind: String,
        #[arg(long)]
        json: bool,
    },
    /// Deny the pending adoption AR for `--kind`.
    Deny {
        #[arg(long = "org-id")]
        org_id: String,
        #[arg(long)]
        kind: String,
        #[arg(long, default_value = "operator denied")]
        reason: String,
        #[arg(long)]
        json: bool,
    },
    /// Adopt a template inline (auto-approved).
    Adopt {
        #[arg(long = "org-id")]
        org_id: String,
        #[arg(long)]
        kind: String,
        #[arg(long)]
        json: bool,
    },
    /// Revoke a previously adopted template + all descending grants.
    Revoke {
        #[arg(long = "org-id")]
        org_id: String,
        #[arg(long)]
        kind: String,
        #[arg(long, default_value = "operator revoked")]
        reason: String,
        #[arg(long)]
        json: bool,
    },
}

pub async fn run(server_url_override: Option<String>, cmd: TemplateCommand) -> i32 {
    match cmd {
        TemplateCommand::List { org_id, json } => {
            list_impl(server_url_override, &org_id, json).await
        }
        TemplateCommand::Approve { org_id, kind, json } => {
            action_impl(server_url_override, &org_id, &kind, "approve", None, json).await
        }
        TemplateCommand::Deny {
            org_id,
            kind,
            reason,
            json,
        } => {
            action_impl(
                server_url_override,
                &org_id,
                &kind,
                "deny",
                Some(serde_json::json!({ "reason": reason })),
                json,
            )
            .await
        }
        TemplateCommand::Adopt { org_id, kind, json } => {
            action_impl(server_url_override, &org_id, &kind, "adopt", None, json).await
        }
        TemplateCommand::Revoke {
            org_id,
            kind,
            reason,
            json,
        } => {
            action_impl(
                server_url_override,
                &org_id,
                &kind,
                "revoke",
                Some(serde_json::json!({ "reason": reason })),
                json,
            )
            .await
        }
    }
}

async fn list_impl(server_url_override: Option<String>, org_id: &str, json: bool) -> i32 {
    let (base, client) = match prepare(server_url_override) {
        Ok(x) => x,
        Err(code) => return code,
    };
    let url = format!(
        "{base}/api/v0/orgs/{}/authority-templates",
        urlencode(org_id)
    );
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
        for bucket in ["pending", "active", "revoked", "available"] {
            let rows = out
                .get(bucket)
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            println!("{bucket} ({})", rows.len());
            for row in rows {
                let kind = row
                    .get("kind")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?")
                    .to_string();
                let summary = row
                    .get("summary")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                println!("  {kind}  {summary}");
            }
        }
    }
    EXIT_OK
}

async fn action_impl(
    server_url_override: Option<String>,
    org_id: &str,
    kind: &str,
    action: &str,
    body: Option<serde_json::Value>,
    json: bool,
) -> i32 {
    let (base, client) = match prepare(server_url_override) {
        Ok(x) => x,
        Err(code) => return code,
    };
    let url = format!(
        "{base}/api/v0/orgs/{}/authority-templates/{}/{}",
        urlencode(org_id),
        urlencode(kind),
        action,
    );
    let mut req = client.post(&url);
    if let Some(body) = body {
        req = req.json(&body);
    } else {
        req = req.json(&serde_json::json!({}));
    }
    let res = match req.send().await {
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
        match action {
            "approve" => println!("template approved"),
            "deny" => println!("template denied"),
            "adopt" => println!("template adopted"),
            "revoke" => {
                let n = out
                    .get("grant_count_revoked")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                println!("template revoked — {n} grant(s) cascaded");
            }
            _ => {}
        }
        if let Some(ar_id) = out.get("adoption_auth_request_id").and_then(|v| v.as_str()) {
            println!("  auth_request:    {ar_id}");
        }
        if let Some(state) = out
            .get("new_state")
            .or_else(|| out.get("state"))
            .and_then(|v| v.as_str())
        {
            println!("  state:           {state}");
        }
        if let Some(event_id) = out.get("audit_event_id").and_then(|v| v.as_str()) {
            println!("  audit event:     {event_id}");
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
