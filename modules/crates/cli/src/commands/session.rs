//! `phi session` — first-session-launch subcommands (M5).
//!
//! **Shape at M5/P7:**
//!
//! - `phi session launch` — POST
//!   `/api/v0/orgs/:org_id/projects/:project_id/sessions`; returns
//!   immediately (server spawns the replay). `--detach` returns the
//!   JSON receipt; the default shows human-friendly receipt lines.
//!   (ADR-0031 live-tail is not wired — see drift D4.2 + D7 live tail
//!   deferral; `phi session show` surfaces terminal state.)
//! - `phi session show` — GET `/api/v0/sessions/:id`; drills the
//!   full `SessionDetail` down to loops / turns.
//! - `phi session terminate` — POST
//!   `/api/v0/sessions/:id/terminate`.
//! - `phi session list` — GET
//!   `/api/v0/projects/:project_id/sessions`.
//!
//! Binary prefix is `phi` (never `baby-phi`) per the M2+ CLI naming
//! discipline.
//!
//! ## phi-core leverage
//!
//! Q1 at M5/P7: **none**. The plan's Part 1.5 prediction of an
//! `AgentEvent` import for SSE tail rendering did not land — the
//! live-tail SSE endpoint is itself deferred (plan drift D4.2
//! already documents that the real `agent_loop` call is M7+).
//! Revisit at M7 when SSE lands; at that point a single
//! `use phi_core::types::event::AgentEvent;` will deserialise the
//! tail payload.

use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use serde::Deserialize;
use server::config::ServerConfig;

use crate::exit::{
    EXIT_INTERNAL, EXIT_OK, EXIT_PRECONDITION_FAILED, EXIT_REJECTED, EXIT_TRANSPORT,
};
use crate::session_store;

/// Clap subcommand surface for `phi session`.
#[derive(Debug, clap::Subcommand)]
pub enum SessionCommand {
    /// Launch a first session. **[M5/P7]**
    ///
    /// Submits the launch request and surfaces the receipt. The
    /// live-tail SSE path is deferred to M7 alongside the real
    /// `agent_loop` invocation (plan drift D4.2). Use `phi session
    /// show --id <id>` to inspect terminal state once the synthetic
    /// replay completes.
    Launch {
        #[arg(long = "org-id")]
        org_id: String,
        #[arg(long = "project-id")]
        project_id: String,
        #[arg(long = "agent-id")]
        agent_id: String,
        /// Initial prompt the session runs against.
        #[arg(long)]
        prompt: String,
        /// Return the JSON receipt without any human-readable
        /// rendering. At M5/P7 every launch is effectively
        /// detached (no live tail yet); the flag persists so the
        /// wire is stable for M7 SSE integration.
        #[arg(long)]
        detach: bool,
        #[arg(long)]
        json: bool,
    },
    /// Show a single session drill-down by id. **[M5/P7]**
    Show {
        #[arg(long)]
        id: String,
        #[arg(long)]
        json: bool,
    },
    /// Terminate a running session. **[M5/P7]**
    Terminate {
        #[arg(long)]
        id: String,
        /// Reason — surfaced on the audit event + governance event.
        #[arg(long, default_value = "operator requested")]
        reason: String,
        #[arg(long)]
        json: bool,
    },
    /// List sessions in a project. **[M5/P7]**
    List {
        #[arg(long = "project-id")]
        project_id: String,
        /// Show only sessions with `governance_state = running`.
        #[arg(long = "active-only")]
        active_only: bool,
        #[arg(long)]
        json: bool,
    },
    /// Preview the Permission Check trace that the launch wire
    /// evaluates, without spawning a session. **[M5/P7]**
    Preview {
        #[arg(long = "org-id")]
        org_id: String,
        #[arg(long = "project-id")]
        project_id: String,
        #[arg(long = "agent-id")]
        agent_id: String,
        #[arg(long)]
        json: bool,
    },
}

pub async fn run(server_url_override: Option<String>, cmd: SessionCommand) -> i32 {
    match cmd {
        SessionCommand::Launch {
            org_id,
            project_id,
            agent_id,
            prompt,
            detach,
            json,
        } => {
            launch_impl(
                server_url_override,
                &org_id,
                &project_id,
                &agent_id,
                &prompt,
                detach,
                json,
            )
            .await
        }
        SessionCommand::Show { id, json } => show_impl(server_url_override, &id, json).await,
        SessionCommand::Terminate { id, reason, json } => {
            terminate_impl(server_url_override, &id, &reason, json).await
        }
        SessionCommand::List {
            project_id,
            active_only,
            json,
        } => list_impl(server_url_override, &project_id, active_only, json).await,
        SessionCommand::Preview {
            org_id,
            project_id,
            agent_id,
            json,
        } => preview_impl(server_url_override, &org_id, &project_id, &agent_id, json).await,
    }
}

// ---------------------------------------------------------------------------
// `phi session launch`
// ---------------------------------------------------------------------------

async fn launch_impl(
    server_url_override: Option<String>,
    org_id: &str,
    project_id: &str,
    agent_id: &str,
    prompt: &str,
    detach: bool,
    json: bool,
) -> i32 {
    let base = match resolve_base_url(server_url_override) {
        Ok(u) => u,
        Err(e) => {
            eprintln!("phi: failed to resolve server URL: {e:#}");
            return EXIT_INTERNAL;
        }
    };
    let client = match build_authed_client() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("phi: {e}");
            return EXIT_PRECONDITION_FAILED;
        }
    };
    let body = serde_json::json!({
        "agent_id": agent_id,
        "prompt": prompt,
    });
    let url = format!(
        "{base}/api/v0/orgs/{}/projects/{}/sessions",
        urlencode(org_id),
        urlencode(project_id),
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
    if json || detach {
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
    } else {
        println!("session launched");
        if let Some(sid) = out["session_id"].as_str() {
            println!("  id:              {sid}");
        }
        if let Some(loop_id) = out["first_loop_id"].as_str() {
            println!("  first loop:      {loop_id}");
        }
        if let Some(event_id) = out["session_started_event_id"].as_str() {
            println!("  audit event:     {event_id}");
        }
        if let Some(decision) = out["permission_check"].as_object() {
            let outcome = decision
                .get("decision")
                .and_then(|v| v.as_str())
                .or_else(|| decision.get("outcome").and_then(|v| v.as_str()))
                .unwrap_or("unknown");
            println!("  permission:      {outcome}");
        }
        println!(
            "  (live tail deferred to M7 — `phi session show --id <id>` inspects terminal state)",
        );
    }
    EXIT_OK
}

// ---------------------------------------------------------------------------
// `phi session show`
// ---------------------------------------------------------------------------

async fn show_impl(server_url_override: Option<String>, session_id: &str, json: bool) -> i32 {
    let base = match resolve_base_url(server_url_override) {
        Ok(u) => u,
        Err(e) => {
            eprintln!("phi: failed to resolve server URL: {e:#}");
            return EXIT_INTERNAL;
        }
    };
    let client = match build_authed_client() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("phi: {e}");
            return EXIT_PRECONDITION_FAILED;
        }
    };
    let url = format!("{base}/api/v0/sessions/{}", urlencode(session_id));
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
        let session = &out["session"];
        println!(
            "session {}",
            session.get("id").and_then(|v| v.as_str()).unwrap_or("?")
        );
        if let Some(state) = session.get("governance_state").and_then(|v| v.as_str()) {
            println!("  state:           {state}");
        }
        if let Some(started_by) = session.get("started_by").and_then(|v| v.as_str()) {
            println!("  started by:      {started_by}");
        }
        if let Some(started_at) = session.get("started_at").and_then(|v| v.as_str()) {
            println!("  started at:      {started_at}");
        }
        if let Some(ended_at) = session.get("ended_at").and_then(|v| v.as_str()) {
            println!("  ended at:        {ended_at}");
        }
        let loops = out.get("loops").and_then(|v| v.as_array());
        println!("  loops:           {}", loops.map(|v| v.len()).unwrap_or(0));
        let turns = out
            .get("turns_by_loop")
            .and_then(|v| v.as_object())
            .map(|m| {
                m.values()
                    .filter_map(|v| v.as_array())
                    .map(|a| a.len())
                    .sum::<usize>()
            })
            .unwrap_or(0);
        println!("  turns:           {turns}");
    }
    EXIT_OK
}

// ---------------------------------------------------------------------------
// `phi session terminate`
// ---------------------------------------------------------------------------

async fn terminate_impl(
    server_url_override: Option<String>,
    session_id: &str,
    reason: &str,
    json: bool,
) -> i32 {
    let base = match resolve_base_url(server_url_override) {
        Ok(u) => u,
        Err(e) => {
            eprintln!("phi: failed to resolve server URL: {e:#}");
            return EXIT_INTERNAL;
        }
    };
    let client = match build_authed_client() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("phi: {e}");
            return EXIT_PRECONDITION_FAILED;
        }
    };
    let body = serde_json::json!({ "reason": reason });
    let url = format!("{base}/api/v0/sessions/{}/terminate", urlencode(session_id));
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
        println!("session terminated");
        if let Some(final_state) = out["final_state"].as_str() {
            println!("  final state:     {final_state}");
        }
        if let Some(event_id) = out["event_id"].as_str() {
            println!("  audit event:     {event_id}");
        }
    }
    EXIT_OK
}

// ---------------------------------------------------------------------------
// `phi session list`
// ---------------------------------------------------------------------------

async fn list_impl(
    server_url_override: Option<String>,
    project_id: &str,
    active_only: bool,
    json: bool,
) -> i32 {
    let base = match resolve_base_url(server_url_override) {
        Ok(u) => u,
        Err(e) => {
            eprintln!("phi: failed to resolve server URL: {e:#}");
            return EXIT_INTERNAL;
        }
    };
    let client = match build_authed_client() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("phi: {e}");
            return EXIT_PRECONDITION_FAILED;
        }
    };
    let url = format!("{base}/api/v0/projects/{}/sessions", urlencode(project_id));
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
    let items: Vec<serde_json::Value> = match &out {
        serde_json::Value::Array(a) => a.clone(),
        v => v
            .get("sessions")
            .and_then(|s| s.as_array())
            .cloned()
            .unwrap_or_default(),
    };
    let filtered: Vec<&serde_json::Value> = items
        .iter()
        .filter(|s| {
            if !active_only {
                return true;
            }
            matches!(
                s.get("governance_state").and_then(|v| v.as_str()),
                Some("running")
            )
        })
        .collect();
    if json {
        println!("{}", serde_json::to_string_pretty(&filtered).unwrap());
    } else if filtered.is_empty() {
        println!("no sessions");
    } else {
        println!("sessions ({})", filtered.len());
        for s in filtered {
            let id = s.get("id").and_then(|v| v.as_str()).unwrap_or("?");
            let state = s
                .get("governance_state")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            let agent = s.get("started_by").and_then(|v| v.as_str()).unwrap_or("?");
            println!("  {id}  [{state}]  started-by={agent}");
        }
    }
    EXIT_OK
}

// ---------------------------------------------------------------------------
// `phi session preview`
// ---------------------------------------------------------------------------

async fn preview_impl(
    server_url_override: Option<String>,
    org_id: &str,
    project_id: &str,
    agent_id: &str,
    json: bool,
) -> i32 {
    let base = match resolve_base_url(server_url_override) {
        Ok(u) => u,
        Err(e) => {
            eprintln!("phi: failed to resolve server URL: {e:#}");
            return EXIT_INTERNAL;
        }
    };
    let client = match build_authed_client() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("phi: {e}");
            return EXIT_PRECONDITION_FAILED;
        }
    };
    let body = serde_json::json!({ "agent_id": agent_id });
    let url = format!(
        "{base}/api/v0/orgs/{}/projects/{}/sessions/preview",
        urlencode(org_id),
        urlencode(project_id),
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
        println!("permission-check preview");
        if let Some(decision) = out["decision"].as_object() {
            let outcome = decision
                .get("decision")
                .and_then(|v| v.as_str())
                .or_else(|| decision.get("outcome").and_then(|v| v.as_str()))
                .unwrap_or("?");
            println!("  outcome:         {outcome}");
        }
    }
    EXIT_OK
}

// ---------------------------------------------------------------------------
// Shared CLI plumbing (mirrors the project/agent modules)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct ApiErrorWire {
    code: String,
    message: String,
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
        .timeout(Duration::from_secs(15))
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
