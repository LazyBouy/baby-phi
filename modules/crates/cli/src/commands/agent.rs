//! `phi agent` — agent-management subcommands.
//!
//! **Shape at M4/P4:**
//!
//! - `agent demo` — legacy phi-core agent-loop demo (unchanged from M1;
//!   consumes the pre-M1 `config.toml` and streams a single turn to
//!   stdout).
//! - `agent list` — page-08 roster lookup, wired at M4/P4. Supports
//!   `--role` and `--search` filters; renders as a human table or
//!   `--json` for scripting.
//! - `agent show` / `create` / `update` / `revert-limits` — scaffolds
//!   that still return [`EXIT_NOT_IMPLEMENTED`](crate::exit::EXIT_NOT_IMPLEMENTED)
//!   pending M4/P5.
//!
//! ## phi-core leverage
//!
//! Q1 **none** at the CLI tier for `list` — the roster response is
//! pure phi governance (no `AgentProfile` / `ExecutionLimits` in
//! the wire shape; those land with `agent show` at P5). The pre-M1
//! `demo` path still imports `phi_core::{agents_from_config, …}` as
//! before.

use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use phi_core::{
    agents_from_config, parse_config_file, save_session, AgentEvent, SessionRecorder,
    SessionRecorderConfig, StreamDelta,
};
use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use serde::Deserialize;
use server::config::ServerConfig;

use crate::exit::{
    EXIT_INTERNAL, EXIT_NOT_IMPLEMENTED, EXIT_OK, EXIT_PRECONDITION_FAILED, EXIT_REJECTED,
    EXIT_TRANSPORT,
};
use crate::session_store;
use crate::AgentCommand;

pub async fn run(server_url_override: Option<String>, cmd: AgentCommand) -> i32 {
    match cmd {
        AgentCommand::Demo { prompt } => demo(prompt).await,
        AgentCommand::List {
            org_id,
            role,
            search,
            json,
        } => {
            list_impl(
                server_url_override,
                &org_id,
                role.as_deref(),
                search.as_deref(),
                json,
            )
            .await
        }
        AgentCommand::Show { .. } => scaffold("agent show", "M4/P5b"),
        AgentCommand::Create {
            org_id,
            name,
            kind,
            role,
            model_id: _,
            system_prompt,
            parallelize,
            override_max_turns,
            override_max_tokens,
            override_max_duration_secs,
            override_max_cost,
            json,
        } => {
            create_impl(
                server_url_override,
                CreateCliInput {
                    org_id,
                    name,
                    kind,
                    role,
                    system_prompt,
                    parallelize,
                    override_max_turns,
                    override_max_tokens,
                    override_max_duration_secs,
                    override_max_cost,
                    json,
                },
            )
            .await
        }
        AgentCommand::Update {
            id,
            patch_json,
            model_config_id,
            json,
        } => update_impl(server_url_override, &id, patch_json, model_config_id, json).await,
        AgentCommand::RevertLimits { id, json } => {
            revert_limits_impl(server_url_override, &id, json).await
        }
    }
}

struct CreateCliInput {
    org_id: String,
    name: String,
    kind: String,
    role: String,
    system_prompt: Option<String>,
    parallelize: u32,
    override_max_turns: Option<usize>,
    override_max_tokens: Option<usize>,
    override_max_duration_secs: Option<u64>,
    override_max_cost: Option<f64>,
    json: bool,
}

fn scaffold(cmd: &str, target_milestone: &str) -> i32 {
    eprintln!(
        "`phi {cmd}` is scaffolded but not yet wired to the server. \
         Implementation lands at {target_milestone}. \
         Retry once the release notes mark {target_milestone} as shipped.",
    );
    EXIT_NOT_IMPLEMENTED
}

// ---------------------------------------------------------------------------
// `agent list` — M4/P4
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct ListResponseWire {
    #[allow(dead_code)]
    org_id: String,
    agents: Vec<AgentRosterItemWire>,
}

#[derive(Debug, Deserialize)]
struct AgentRosterItemWire {
    id: String,
    kind: String,
    display_name: String,
    #[serde(default)]
    role: Option<String>,
    created_at: String,
}

#[derive(Debug, Deserialize)]
struct ApiErrorWire {
    code: String,
    message: String,
}

async fn list_impl(
    server_url_override: Option<String>,
    org_id: &str,
    role: Option<&str>,
    search: Option<&str>,
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

    let mut url = format!("{base}/api/v0/orgs/{org_id}/agents");
    let mut first = true;
    if let Some(r) = role {
        url.push_str(if first { "?" } else { "&" });
        url.push_str(&format!("role={}", urlencode(r)));
        first = false;
    }
    if let Some(s) = search {
        url.push_str(if first { "?" } else { "&" });
        url.push_str(&format!("search={}", urlencode(s)));
    }

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
    let body: ListResponseWire = match res.json().await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("phi: decode response: {e}");
            return EXIT_INTERNAL;
        }
    };

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "org_id": org_id,
                "agents": body.agents.iter().map(|a| serde_json::json!({
                    "id": a.id,
                    "kind": a.kind,
                    "display_name": a.display_name,
                    "role": a.role,
                    "created_at": a.created_at,
                })).collect::<Vec<_>>()
            }))
            .unwrap()
        );
    } else if body.agents.is_empty() {
        println!("no agents match this filter");
    } else {
        println!(
            "{:<40} {:<28} {:<8} {:<10}",
            "id", "display_name", "kind", "role"
        );
        for a in &body.agents {
            println!(
                "{:<40} {:<28} {:<8} {:<10}",
                a.id,
                truncate(&a.display_name, 28),
                a.kind,
                a.role.as_deref().unwrap_or("-")
            );
        }
    }
    EXIT_OK
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let head: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{head}…")
    }
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
// `agent create` — M4/P5
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct CreateResponseWire {
    agent_id: String,
    owning_org_id: String,
    profile_id: Option<String>,
    execution_limits_override_id: Option<String>,
    audit_event_id: String,
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

    // Validate CLI-supplied kind + role against the server enum set
    // before round-tripping. The server re-checks — this is just a
    // friendlier early error.
    let kind_wire = match input.kind.as_str() {
        "human" => "human",
        "llm" => "llm",
        other => {
            eprintln!("phi: --kind must be `human` or `llm` (got `{other}`)");
            return EXIT_PRECONDITION_FAILED;
        }
    };
    let role_wire = match input.role.as_str() {
        "executive" | "admin" | "member" | "intern" | "contract" | "system" => input.role.as_str(),
        other => {
            eprintln!(
                "phi: --role must be one of executive|admin|member|intern|contract|system (got `{other}`)"
            );
            return EXIT_PRECONDITION_FAILED;
        }
    };

    // Blueprint — minimal seed from the CLI flags. The web wizard
    // fills a richer blueprint; CLI users typically refine via a
    // subsequent `phi agent update --patch-json`.
    let mut blueprint = serde_json::json!({});
    if let Some(sp) = &input.system_prompt {
        blueprint["system_prompt"] = serde_json::json!(sp);
    }

    let override_body = match (
        input.override_max_turns,
        input.override_max_tokens,
        input.override_max_duration_secs,
        input.override_max_cost,
    ) {
        (None, None, None, None) => serde_json::Value::Null,
        (t, tok, d, c) => {
            // phi-core::ExecutionLimits serde shape:
            // { max_turns, max_total_tokens, max_duration (Duration),
            //   max_cost }. We use sensible per-field defaults when a
            //   subset is supplied so the full struct round-trips.
            serde_json::json!({
                "max_turns": t.unwrap_or(50),
                "max_total_tokens": tok.unwrap_or(1_000_000),
                "max_duration": { "secs": d.unwrap_or(600), "nanos": 0 },
                "max_cost": c,
            })
        }
    };

    let body = serde_json::json!({
        "display_name": input.name,
        "kind": kind_wire,
        "role": role_wire,
        "blueprint": blueprint,
        "parallelize": input.parallelize,
        "initial_execution_limits_override": override_body,
    });

    let url = format!("{base}/api/v0/orgs/{}/agents", urlencode(&input.org_id));
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
    let body: serde_json::Value = match res.json().await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("phi: decode response: {e}");
            return EXIT_INTERNAL;
        }
    };
    if input.json {
        println!("{}", serde_json::to_string_pretty(&body).unwrap());
    } else {
        println!("agent created");
        if let Some(aid) = body["agent_id"].as_str() {
            println!("  id:               {aid}");
        }
        if let Some(oid) = body["execution_limits_override_id"].as_str() {
            println!("  override_id:      {oid}");
        } else {
            println!("  execution_limits: inherited from org snapshot");
        }
    }
    EXIT_OK
}

// ---------------------------------------------------------------------------
// `agent update` — M4/P5
// ---------------------------------------------------------------------------

async fn update_impl(
    server_url_override: Option<String>,
    id: &str,
    patch_json: Option<String>,
    model_config_id: Option<String>,
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
    let body: serde_json::Value = match (patch_json.as_deref(), model_config_id.as_deref()) {
        (Some(_), Some(_)) => {
            eprintln!("phi: --patch-json and --model-config-id are mutually exclusive");
            return EXIT_PRECONDITION_FAILED;
        }
        (None, None) => {
            eprintln!("phi: exactly one of --patch-json / --model-config-id is required");
            return EXIT_PRECONDITION_FAILED;
        }
        (Some(raw), None) => match serde_json::from_str(raw) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("phi: --patch-json is not valid JSON: {e}");
                return EXIT_PRECONDITION_FAILED;
            }
        },
        (None, Some(mcid)) => serde_json::json!({ "model_config_id": mcid }),
    };
    let url = format!("{base}/api/v0/agents/{}/profile", urlencode(id));
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
        Ok(b) => b,
        Err(e) => {
            eprintln!("phi: decode response: {e}");
            return EXIT_INTERNAL;
        }
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
    } else {
        match out["audit_event_id"].as_str() {
            Some(event_id) => println!("agent updated (audit event: {event_id})"),
            None => println!("no-op — patch contained no changes"),
        }
        if let Some(src) = out["execution_limits_source"].as_str() {
            println!("  execution_limits: {src}");
        }
    }
    EXIT_OK
}

// ---------------------------------------------------------------------------
// `agent revert-limits` — M4/P5
// ---------------------------------------------------------------------------

async fn revert_limits_impl(server_url_override: Option<String>, id: &str, json: bool) -> i32 {
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
    let url = format!(
        "{base}/api/v0/agents/{}/execution-limits-override",
        urlencode(id)
    );
    let res = match client.delete(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("phi: DELETE {url} failed: {e}");
            return EXIT_TRANSPORT;
        }
    };
    let status = res.status();
    if !status.is_success() {
        return report_api_error(res, status).await;
    }
    let out: serde_json::Value = match res.json().await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("phi: decode response: {e}");
            return EXIT_INTERNAL;
        }
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
    } else {
        println!("execution_limits override reverted");
        if let Some(src) = out["execution_limits_source"].as_str() {
            println!("  agent now using: {src} (org snapshot)");
        }
    }
    EXIT_OK
}

// ---------------------------------------------------------------------------
// `agent demo` — pre-M1 legacy
// ---------------------------------------------------------------------------

async fn demo(prompt_override: Option<String>) -> i32 {
    let config = match parse_config_file(Path::new("config.toml")) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to parse config.toml: {e}");
            return 1;
        }
    };

    let agents = match agents_from_config(&config) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Failed to build agents: {e}");
            return 1;
        }
    };

    let (name, mut agent_arc) = match agents.into_iter().next() {
        Some(pair) => pair,
        None => {
            eprintln!("No agents configured");
            return 1;
        }
    };
    println!("Agent: {name}");

    let agent = match Arc::get_mut(&mut agent_arc) {
        Some(a) => a,
        None => {
            eprintln!("Failed to get mutable agent reference");
            return 1;
        }
    };

    let registry = phi_core::tools::ToolRegistry::new().with_defaults();
    let tools = registry.resolve(&config.tools.enabled);
    agent.set_tools(tools);

    let prompt = prompt_override.unwrap_or_else(|| {
        "Write a marketing email for our new AI consulting service that helps \
         mid-size companies automate their customer support with AI agents."
            .to_string()
    });

    println!("=== phi agent demo ===\n");
    println!("Prompt: {prompt}\n");
    println!("---\n");

    let mut recorder = SessionRecorder::new(SessionRecorderConfig::default());
    let mut rx = agent.prompt(prompt).await;

    while let Some(event) = rx.recv().await {
        recorder.on_event(event.clone());

        match &event {
            AgentEvent::MessageUpdate {
                delta: StreamDelta::Text { delta },
                ..
            } => {
                print!("{delta}");
                std::io::stdout().flush().ok();
            }
            AgentEvent::ToolExecutionStart { tool_name, .. } => {
                println!("\n[tool: {tool_name}]");
            }
            AgentEvent::ToolExecutionEnd {
                tool_name,
                is_error,
                ..
            } => {
                let status = if *is_error { "failed" } else { "done" };
                println!("[tool: {tool_name} — {status}]");
            }
            AgentEvent::AgentEnd { usage, .. } => {
                println!("\n--- Done ---");
                println!(
                    "Tokens: {} input, {} output, {} total",
                    usage.input, usage.output, usage.total_tokens
                );
            }
            _ => {}
        }
    }

    recorder.flush();
    let session_dir = Path::new("workspace/session");
    for session in recorder.drain_completed() {
        match save_session(&session, session_dir) {
            Ok(path) => println!("Session saved to: {}", path.display()),
            Err(e) => eprintln!("Failed to save session: {e}"),
        }
    }
    EXIT_OK
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urlencode_encodes_spaces_and_reserved() {
        assert_eq!(urlencode("alpha bot"), "alpha%20bot");
        assert_eq!(urlencode("search me?"), "search%20me%3F");
        assert_eq!(urlencode("A-Z_0.9~"), "A-Z_0.9~");
    }

    #[test]
    fn truncate_respects_max() {
        assert_eq!(truncate("hi", 10), "hi");
        assert_eq!(truncate("0123456789abc", 10), "012345678…");
    }
}
