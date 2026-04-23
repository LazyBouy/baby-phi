//! `phi project` — project-management subcommands (M4).
//!
//! **Shape at M4/P6:**
//!
//! - `phi project create` — **wired**. Ships Shape A (immediate
//!   materialisation → 201) + Shape B submit (creates a 2-approver
//!   pending AR → 202). Callers drive Shape B approvals via `phi
//!   project approve-pending --ar-id --approver-id [--deny]`.
//! - `phi project approve-pending` — **wired**. Drives one slot on a
//!   Shape B pending AR.
//! - `phi project list` / `show` / `update-okrs` — scaffolded; wired at M4/P7.
//!
//! ## phi-core leverage
//!
//! Q1 **none** — this module introduces no `use phi_core::…` imports.
//! The project surface is pure phi governance (OKRs, project shape,
//! resource boundaries) with zero phi-core types in transit.

use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use serde::Deserialize;
use server::config::ServerConfig;

use crate::exit::{
    EXIT_INTERNAL, EXIT_OK, EXIT_PRECONDITION_FAILED, EXIT_REJECTED, EXIT_TRANSPORT,
};
use crate::session_store;

/// Clap subcommand surface for `phi project`.
///
/// The `Create` variant is large (~300 bytes) because clap expects
/// inline fields for rich `--help` rendering; boxing individual
/// strings would hurt readability without meaningful runtime benefit
/// (this enum is held for microseconds during CLI dispatch).
#[allow(clippy::large_enum_variant)]
#[derive(Debug, clap::Subcommand)]
pub enum ProjectCommand {
    /// List projects in an org. **[M4/P7]**
    List {
        #[arg(long = "org-id")]
        org_id: String,
        /// Optional shape filter — `shape_a` or `shape_b`.
        #[arg(long)]
        shape: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Show a single project by id. **[M4/P7]**
    Show {
        #[arg(long)]
        id: String,
        #[arg(long)]
        json: bool,
    },
    /// Create a project (Shape A immediate or Shape B pending-approval). **[M4/P6]**
    Create {
        #[arg(long = "org-id")]
        org_id: String,
        /// Project id (UUID). Auto-generated if omitted.
        #[arg(long = "project-id")]
        project_id: Option<String>,
        #[arg(long)]
        name: String,
        #[arg(long, default_value = "")]
        description: String,
        #[arg(long)]
        goal: Option<String>,
        /// `shape_a` (single-org immediate) or `shape_b` (co-owned
        /// two-approver). Defaults to `shape_a`.
        #[arg(long, default_value = "shape_a")]
        shape: String,
        #[arg(long = "co-owner-org-id")]
        co_owner_org_id: Option<String>,
        #[arg(long = "lead-agent-id")]
        lead_agent_id: String,
        /// Comma-separated list of additional member agent ids.
        #[arg(long = "member-ids")]
        member_ids: Option<String>,
        /// Comma-separated list of sponsor agent ids.
        #[arg(long = "sponsor-ids")]
        sponsor_ids: Option<String>,
        #[arg(long = "token-budget")]
        token_budget: Option<u64>,
        /// Optional path to an OKR JSON file (matches the domain
        /// Objective + KeyResult shape; validated server-side).
        #[arg(long = "okrs-file")]
        okrs_file: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    /// Drive one slot on a Shape B pending AR (approve or deny). **[M4/P6]**
    ApprovePending {
        #[arg(long = "ar-id")]
        ar_id: String,
        #[arg(long = "approver-id")]
        approver_id: String,
        /// Supply `--deny` to reject; default is approve.
        #[arg(long)]
        deny: bool,
        #[arg(long)]
        json: bool,
    },
    /// Apply an OKR patch (create / update / delete Objectives and
    /// KeyResults in place). **[M4/P7]**
    UpdateOkrs {
        #[arg(long)]
        id: String,
        /// JSON patch payload — a `[{kind,op,payload}]` array matching
        /// the M4/P7 `PATCH /api/v0/projects/:id/okrs` contract.
        #[arg(long = "patch-json")]
        patch_json: String,
        #[arg(long)]
        json: bool,
    },
}

pub async fn run(server_url_override: Option<String>, cmd: ProjectCommand) -> i32 {
    match cmd {
        ProjectCommand::List { .. } => scaffold("project list", "M4/P8"),
        ProjectCommand::Show { id, json } => show_impl(server_url_override, &id, json).await,
        ProjectCommand::Create {
            org_id,
            project_id,
            name,
            description,
            goal,
            shape,
            co_owner_org_id,
            lead_agent_id,
            member_ids,
            sponsor_ids,
            token_budget,
            okrs_file,
            json,
        } => {
            create_impl(
                server_url_override,
                CreateCliInput {
                    org_id,
                    project_id,
                    name,
                    description,
                    goal,
                    shape,
                    co_owner_org_id,
                    lead_agent_id,
                    member_ids,
                    sponsor_ids,
                    token_budget,
                    okrs_file,
                    json,
                },
            )
            .await
        }
        ProjectCommand::ApprovePending {
            ar_id,
            approver_id,
            deny,
            json,
        } => approve_pending_impl(server_url_override, &ar_id, &approver_id, !deny, json).await,
        ProjectCommand::UpdateOkrs {
            id,
            patch_json,
            json,
        } => update_okrs_impl(server_url_override, &id, &patch_json, json).await,
    }
}

fn scaffold(cmd: &str, target_milestone: &str) -> i32 {
    eprintln!(
        "`phi {cmd}` is scaffolded but not yet wired to the server. \
         Implementation lands at {target_milestone}. \
         Retry once the release notes mark {target_milestone} as shipped.",
    );
    crate::exit::EXIT_NOT_IMPLEMENTED
}

// ---------------------------------------------------------------------------
// `phi project create` — M4/P6
// ---------------------------------------------------------------------------

struct CreateCliInput {
    org_id: String,
    project_id: Option<String>,
    name: String,
    description: String,
    goal: Option<String>,
    shape: String,
    co_owner_org_id: Option<String>,
    lead_agent_id: String,
    member_ids: Option<String>,
    sponsor_ids: Option<String>,
    token_budget: Option<u64>,
    okrs_file: Option<PathBuf>,
    json: bool,
}

#[derive(Debug, Deserialize)]
struct ApiErrorWire {
    code: String,
    message: String,
}

fn parse_csv(raw: &Option<String>) -> Vec<String> {
    raw.as_deref()
        .unwrap_or("")
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

async fn create_impl(server_url_override: Option<String>, input: CreateCliInput) -> i32 {
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

    let shape_wire = match input.shape.as_str() {
        "shape_a" | "a" | "A" => "shape_a",
        "shape_b" | "b" | "B" => "shape_b",
        other => {
            eprintln!("phi: --shape must be `shape_a` or `shape_b` (got `{other}`)");
            return EXIT_PRECONDITION_FAILED;
        }
    };
    let project_id = input
        .project_id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    // OKR file loading (optional) — expects a `{objectives, key_results}`
    // JSON object. The server re-validates measurement-type shapes.
    let (objectives, key_results) = match input.okrs_file {
        None => (serde_json::json!([]), serde_json::json!([])),
        Some(path) => match std::fs::read_to_string(&path) {
            Ok(s) => match serde_json::from_str::<serde_json::Value>(&s) {
                Ok(v) => {
                    let o = v
                        .get("objectives")
                        .cloned()
                        .unwrap_or(serde_json::json!([]));
                    let k = v
                        .get("key_results")
                        .cloned()
                        .unwrap_or(serde_json::json!([]));
                    (o, k)
                }
                Err(e) => {
                    eprintln!("phi: failed to parse --okrs-file as JSON: {e}");
                    return EXIT_PRECONDITION_FAILED;
                }
            },
            Err(e) => {
                eprintln!("phi: failed to read --okrs-file {}: {e}", path.display());
                return EXIT_PRECONDITION_FAILED;
            }
        },
    };

    let body = serde_json::json!({
        "project_id": project_id,
        "name": input.name,
        "description": input.description,
        "goal": input.goal,
        "shape": shape_wire,
        "co_owner_org_id": input.co_owner_org_id,
        "lead_agent_id": input.lead_agent_id,
        "member_agent_ids": parse_csv(&input.member_ids),
        "sponsor_agent_ids": parse_csv(&input.sponsor_ids),
        "token_budget": input.token_budget,
        "objectives": objectives,
        "key_results": key_results,
    });

    let url = format!("{base}/api/v0/orgs/{}/projects", urlencode(&input.org_id));
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

    if input.json {
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
    } else {
        match out["outcome"].as_str() {
            Some("materialised") => {
                println!("project created (Shape A)");
                if let Some(pid) = out["project_id"].as_str() {
                    println!("  id:            {pid}");
                }
                if let Some(lead) = out["lead_agent_id"].as_str() {
                    println!("  lead:          {lead}");
                }
            }
            Some("pending") => {
                println!("project pending co-owner approval (Shape B)");
                if let Some(ar) = out["pending_ar_id"].as_str() {
                    println!("  auth_request:  {ar}");
                }
                if let Some(approvers) = out["approver_ids"].as_array() {
                    let names: Vec<&str> = approvers.iter().filter_map(|a| a.as_str()).collect();
                    println!("  approvers:     {}", names.join(", "));
                }
                println!("  next step:     `phi project approve-pending --ar-id <id> --approver-id <id>`");
            }
            _ => {
                eprintln!("phi: unknown outcome in response — falling back to JSON");
                println!("{}", serde_json::to_string_pretty(&out).unwrap());
            }
        }
    }
    EXIT_OK
}

// ---------------------------------------------------------------------------
// `phi project approve-pending` — M4/P6
// ---------------------------------------------------------------------------

async fn approve_pending_impl(
    server_url_override: Option<String>,
    ar_id: &str,
    approver_id: &str,
    approve: bool,
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
        "approver_id": approver_id,
        "approve": approve,
    });
    let url = format!(
        "{base}/api/v0/projects/_pending/{}/approve",
        urlencode(ar_id)
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
        match out["outcome"].as_str() {
            Some("still_pending") => {
                println!(
                    "{} recorded; still waiting on the other approver",
                    if approve { "approval" } else { "denial" }
                );
            }
            Some("terminal") => {
                let state = out["state"].as_str().unwrap_or("unknown");
                println!("auth request reached terminal state: {state}");
                if let Some(pid) = out["project_id"].as_str() {
                    println!("  project materialised: {pid}");
                } else {
                    println!(
                        "  (no project materialised — materialisation-after-approve ships at M5, see C-M5-6)"
                    );
                }
            }
            _ => {
                println!("{}", serde_json::to_string_pretty(&out).unwrap());
            }
        }
    }
    EXIT_OK
}

// ---------------------------------------------------------------------------
// Shared CLI plumbing (mirrors the org/agent modules)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// `phi project show` — M4/P7
// ---------------------------------------------------------------------------

async fn show_impl(server_url_override: Option<String>, project_id: &str, json: bool) -> i32 {
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

    let url = format!("{base}/api/v0/projects/{}", urlencode(project_id));
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
        let project = &out["project"];
        println!(
            "project {}",
            project.get("id").and_then(|v| v.as_str()).unwrap_or("?")
        );
        if let Some(name) = project.get("name").and_then(|v| v.as_str()) {
            println!("  name:          {name}");
        }
        if let Some(shape) = project.get("shape").and_then(|v| v.as_str()) {
            println!("  shape:         {shape}");
        }
        if let Some(status) = project.get("status").and_then(|v| v.as_str()) {
            println!("  status:        {status}");
        }
        if let Some(lead) = out.get("lead_agent_id").and_then(|v| v.as_str()) {
            println!("  lead:          {lead}");
        }
        if let Some(orgs) = out.get("owning_org_ids").and_then(|v| v.as_array()) {
            let ids: Vec<&str> = orgs.iter().filter_map(|v| v.as_str()).collect();
            println!("  owning orgs:   {}", ids.join(", "));
        }
        let objectives = project.get("objectives").and_then(|v| v.as_array());
        let key_results = project.get("key_results").and_then(|v| v.as_array());
        println!(
            "  objectives:    {}",
            objectives.map(|v| v.len()).unwrap_or(0)
        );
        println!(
            "  key results:   {}",
            key_results.map(|v| v.len()).unwrap_or(0)
        );
        let roster = out
            .get("roster")
            .and_then(|v| v.as_array())
            .map(|v| v.len())
            .unwrap_or(0);
        println!("  roster size:   {roster}");
        let sessions = out
            .get("recent_sessions")
            .and_then(|v| v.as_array())
            .map(|v| v.len())
            .unwrap_or(0);
        if sessions == 0 {
            println!("  recent sessions: (none — session persistence ships at M5; C-M5-3)");
        } else {
            println!("  recent sessions: {sessions}");
        }
    }
    EXIT_OK
}

// ---------------------------------------------------------------------------
// `phi project update-okrs` — M4/P7
// ---------------------------------------------------------------------------

async fn update_okrs_impl(
    server_url_override: Option<String>,
    project_id: &str,
    patch_json: &str,
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

    // Expect a bare array of patch entries; wrap into the {patches: [...]}
    // request shape so the CLI surface matches the wire body.
    let patches: serde_json::Value = match serde_json::from_str(patch_json) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("phi: failed to parse --patch-json: {e}");
            return EXIT_PRECONDITION_FAILED;
        }
    };
    if !patches.is_array() {
        eprintln!("phi: --patch-json must be a JSON array of patch entries");
        return EXIT_PRECONDITION_FAILED;
    }
    let body = serde_json::json!({ "patches": patches });

    let url = format!("{base}/api/v0/projects/{}/okrs", urlencode(project_id));
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
        let applied = out
            .get("audit_event_ids")
            .and_then(|v| v.as_array())
            .map(|v| v.len())
            .unwrap_or(0);
        println!("OKR patch applied: {applied} mutation(s)");
        let objectives = out
            .get("objectives")
            .and_then(|v| v.as_array())
            .map(|v| v.len())
            .unwrap_or(0);
        let key_results = out
            .get("key_results")
            .and_then(|v| v.as_array())
            .map(|v| v.len())
            .unwrap_or(0);
        println!("  objectives:  {objectives}");
        println!("  key results: {key_results}");
    }
    EXIT_OK
}
