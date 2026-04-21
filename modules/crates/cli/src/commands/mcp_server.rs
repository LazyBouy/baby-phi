//! `baby-phi mcp-server {list,add,patch-tenants,archive}` subcommands.
//!
//! phi-core leverage:
//! - The `--endpoint` value is phi-core's `McpClient::connect_stdio` /
//!   `connect_http` transport argument verbatim (`stdio:///cmd args…`
//!   or `http[s]://…`). The CLI never reinterprets it — the server's
//!   health-probe is the only code that parses the scheme.
//! - phi-core does not ship a "platform binding" wrapper around MCP;
//!   baby-phi's `ExternalService` composite carries platform-governance
//!   fields (tenants_allowed, secret_ref, status) around phi-core's
//!   transport config.

use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use serde::{Deserialize, Serialize};
use server::ServerConfig;

use crate::exit::{
    EXIT_CASCADE_ABORTED, EXIT_INTERNAL, EXIT_OK, EXIT_PRECONDITION_FAILED, EXIT_REJECTED,
    EXIT_TRANSPORT,
};
use crate::session_store;

// ---------------------------------------------------------------------------
// Clap subcommand surface
// ---------------------------------------------------------------------------

#[derive(Debug, clap::Subcommand)]
pub enum McpServerCommand {
    /// List registered MCP servers.
    List {
        /// Include archived rows in the output.
        #[arg(long)]
        include_archived: bool,
        /// Render as JSON rather than a human table.
        #[arg(long)]
        json: bool,
    },
    /// Register a new MCP server.
    Add {
        /// Operator-visible display name (e.g. `memory-mcp`).
        #[arg(long = "display-name")]
        display_name: String,
        /// External-service kind. M2 wires `mcp` only; `open-api`,
        /// `webhook`, `other` are reserved.
        #[arg(long, value_enum, default_value = "mcp")]
        kind: KindArg,
        /// phi-core transport argument. `stdio:///path/to/cmd args…`
        /// for a local process; `http://…` or `https://…` for a
        /// remote MCP server.
        #[arg(long)]
        endpoint: String,
        /// Vault slug holding the authentication material, if the
        /// server requires one. Omit for unauthenticated services.
        #[arg(long = "secret-ref")]
        secret_ref: Option<String>,
        /// Which orgs may invoke this server. `all` or a
        /// comma-separated list of org UUIDs. Defaults to `all`.
        #[arg(long = "tenants-allowed", default_value = "all")]
        tenants_allowed: String,
    },
    /// Update `tenants_allowed` for an MCP server. If the new set is a
    /// strict subset of the existing one, the server cascades grant
    /// revocations across every affected org. Requires
    /// `--confirm-cascade` to acknowledge the risk.
    PatchTenants {
        /// MCP-server UUID (from `list`).
        #[arg(long)]
        id: String,
        /// Tenants allowed: `all` or a comma-separated list of org
        /// UUIDs.
        #[arg(long = "tenants-allowed")]
        tenants_allowed: String,
        /// Explicit acknowledgement of the cascade risk. Required even
        /// when the PATCH widens (keeps muscle memory consistent).
        #[arg(long = "confirm-cascade")]
        confirm_cascade: bool,
    },
    /// Archive a registered MCP server by UUID.
    Archive {
        /// MCP-server UUID (from `list`).
        #[arg(long)]
        id: String,
    },
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum KindArg {
    Mcp,
    OpenApi,
    Webhook,
    Other,
}

impl KindArg {
    fn as_wire(self) -> &'static str {
        match self {
            KindArg::Mcp => "mcp",
            KindArg::OpenApi => "open_api",
            KindArg::Webhook => "webhook",
            KindArg::Other => "other",
        }
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub async fn run(server_url_override: Option<String>, cmd: McpServerCommand) -> i32 {
    let base = match resolve_base_url(server_url_override) {
        Ok(u) => u,
        Err(e) => {
            eprintln!("baby-phi: failed to resolve server URL: {e:#}");
            return EXIT_INTERNAL;
        }
    };
    let client = match build_authed_client() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("baby-phi: {e}");
            return EXIT_PRECONDITION_FAILED;
        }
    };

    match cmd {
        McpServerCommand::List {
            include_archived,
            json,
        } => list(&client, &base, include_archived, json).await,
        McpServerCommand::Add {
            display_name,
            kind,
            endpoint,
            secret_ref,
            tenants_allowed,
        } => {
            add(
                &client,
                &base,
                &display_name,
                kind,
                &endpoint,
                secret_ref.as_deref(),
                &tenants_allowed,
            )
            .await
        }
        McpServerCommand::PatchTenants {
            id,
            tenants_allowed,
            confirm_cascade,
        } => patch_tenants(&client, &base, &id, &tenants_allowed, confirm_cascade).await,
        McpServerCommand::Archive { id } => archive(&client, &base, &id).await,
    }
}

// ---------------------------------------------------------------------------
// Wire shapes (mirror handlers/platform_mcp_servers.rs)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct AddBody {
    display_name: String,
    kind: String,
    endpoint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    secret_ref: Option<String>,
    tenants_allowed: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct PatchTenantsBody {
    tenants_allowed: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct AddWire {
    mcp_server_id: String,
    auth_request_id: String,
    audit_event_id: String,
}

#[derive(Debug, Deserialize)]
struct ArchiveWire {
    mcp_server_id: String,
    audit_event_id: String,
}

#[derive(Debug, Deserialize)]
struct PatchWire {
    mcp_server_id: String,
    #[serde(default)]
    cascade: Vec<TenantRevocationWire>,
    audit_event_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TenantRevocationWire {
    org: String,
    auth_request: String,
    revoked_grants: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ServerSummaryWire {
    id: String,
    display_name: String,
    kind: String,
    endpoint: String,
    secret_ref: Option<String>,
    tenants_allowed: serde_json::Value,
    status: serde_json::Value,
    archived_at: Option<String>,
    created_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ListWire {
    servers: Vec<ServerSummaryWire>,
}

#[derive(Debug, Deserialize)]
struct ApiErrorWire {
    code: String,
    message: String,
}

// ---------------------------------------------------------------------------
// Subcommand implementations
// ---------------------------------------------------------------------------

async fn list(client: &reqwest::Client, base: &str, include_archived: bool, json: bool) -> i32 {
    let mut url = format!("{base}/api/v0/platform/mcp-servers");
    if include_archived {
        url.push_str("?include_archived=true");
    }
    let res = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("baby-phi: request to {url} failed: {e}");
            return EXIT_TRANSPORT;
        }
    };
    let status = res.status();
    if !status.is_success() {
        return report_api_error(res, status).await;
    }
    let body: ListWire = match res.json().await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("baby-phi: failed to decode list response: {e}");
            return EXIT_INTERNAL;
        }
    };
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&body.servers).unwrap_or_default()
        );
        return EXIT_OK;
    }
    if body.servers.is_empty() {
        println!(
            "(no MCP servers registered — run `baby-phi mcp-server add \\\n    --display-name <NAME> --endpoint <ENDPOINT>` to register one)"
        );
        return EXIT_OK;
    }
    println!(
        "{:<38}  {:<18}  {:<6}  {:<10}  endpoint",
        "mcp_server_id", "display_name", "kind", "status"
    );
    for s in &body.servers {
        let status = s.status.as_str().unwrap_or("(?)");
        println!(
            "{:<38}  {:<18}  {:<6}  {:<10}  {}",
            s.id, s.display_name, s.kind, status, s.endpoint
        );
    }
    EXIT_OK
}

#[allow(clippy::too_many_arguments)]
async fn add(
    client: &reqwest::Client,
    base: &str,
    display_name: &str,
    kind: KindArg,
    endpoint: &str,
    secret_ref: Option<&str>,
    tenants_allowed: &str,
) -> i32 {
    let tenants_json = match parse_tenants_spec(tenants_allowed) {
        Ok(v) => v,
        Err(msg) => {
            eprintln!("baby-phi: {msg}");
            return EXIT_REJECTED;
        }
    };
    let body = AddBody {
        display_name: display_name.to_string(),
        kind: kind.as_wire().to_string(),
        endpoint: endpoint.to_string(),
        secret_ref: secret_ref.map(|s| s.to_string()),
        tenants_allowed: tenants_json,
    };
    let url = format!("{base}/api/v0/platform/mcp-servers");
    let res = match client.post(&url).json(&body).send().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("baby-phi: request to {url} failed: {e}");
            return EXIT_TRANSPORT;
        }
    };
    let status = res.status();
    if status.as_u16() != 201 {
        return report_api_error(res, status).await;
    }
    let body: AddWire = match res.json().await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("baby-phi: failed to decode add response: {e}");
            return EXIT_INTERNAL;
        }
    };
    println!("mcp server registered");
    println!("  mcp_server_id:   {}", body.mcp_server_id);
    println!("  auth_request_id: {}", body.auth_request_id);
    println!("  audit_event_id:  {}", body.audit_event_id);
    EXIT_OK
}

async fn patch_tenants(
    client: &reqwest::Client,
    base: &str,
    id: &str,
    tenants_allowed: &str,
    confirm_cascade: bool,
) -> i32 {
    if !confirm_cascade {
        eprintln!(
            "baby-phi: --confirm-cascade is required for patch-tenants\n\
             (a narrowing PATCH revokes every grant descending from an AR\n\
             requested by a now-excluded org; pass the flag to acknowledge)"
        );
        return EXIT_CASCADE_ABORTED;
    }
    let tenants_json = match parse_tenants_spec(tenants_allowed) {
        Ok(v) => v,
        Err(msg) => {
            eprintln!("baby-phi: {msg}");
            return EXIT_REJECTED;
        }
    };
    let body = PatchTenantsBody {
        tenants_allowed: tenants_json,
    };
    let url = format!("{base}/api/v0/platform/mcp-servers/{id}/tenants");
    let res = match client.patch(&url).json(&body).send().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("baby-phi: request to {url} failed: {e}");
            return EXIT_TRANSPORT;
        }
    };
    let status = res.status();
    if !status.is_success() {
        return report_api_error(res, status).await;
    }
    let body: PatchWire = match res.json().await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("baby-phi: failed to decode patch response: {e}");
            return EXIT_INTERNAL;
        }
    };
    let total_grants: usize = body.cascade.iter().map(|r| r.revoked_grants.len()).sum();
    println!("mcp server tenants patched");
    println!("  mcp_server_id:    {}", body.mcp_server_id);
    println!("  cascade_ar_count: {}", body.cascade.len());
    println!("  revoked_grants:   {}", total_grants);
    if let Some(aid) = body.audit_event_id {
        println!("  audit_event_id:   {}", aid);
    } else {
        println!("  audit_event_id:   (no cascade — no summary event emitted)");
    }
    if !body.cascade.is_empty() {
        println!();
        println!("cascade detail:");
        for rev in &body.cascade {
            println!(
                "  org {} via AR {} — revoked {} grant(s)",
                rev.org,
                rev.auth_request,
                rev.revoked_grants.len()
            );
        }
    }
    EXIT_OK
}

async fn archive(client: &reqwest::Client, base: &str, id: &str) -> i32 {
    let url = format!("{base}/api/v0/platform/mcp-servers/{id}/archive");
    let res = match client.post(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("baby-phi: request to {url} failed: {e}");
            return EXIT_TRANSPORT;
        }
    };
    let status = res.status();
    if !status.is_success() {
        return report_api_error(res, status).await;
    }
    let body: ArchiveWire = match res.json().await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("baby-phi: failed to decode archive response: {e}");
            return EXIT_INTERNAL;
        }
    };
    println!("mcp server archived");
    println!("  mcp_server_id:  {}", body.mcp_server_id);
    println!("  audit_event_id: {}", body.audit_event_id);
    EXIT_OK
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// `all` → `{"mode": "all"}`; comma-separated UUIDs →
/// `{"mode": "only", "orgs": [...]}`.
fn parse_tenants_spec(spec: &str) -> Result<serde_json::Value, String> {
    if spec.eq_ignore_ascii_case("all") {
        return Ok(serde_json::json!({ "mode": "all" }));
    }
    let mut orgs: Vec<String> = Vec::new();
    for token in spec.split(',') {
        let token = token.trim();
        if token.is_empty() {
            continue;
        }
        uuid::Uuid::parse_str(token)
            .map_err(|_| format!("`{token}` is not a valid UUID in --tenants-allowed"))?;
        orgs.push(token.to_string());
    }
    Ok(serde_json::json!({ "mode": "only", "orgs": orgs }))
}

fn build_authed_client() -> Result<reqwest::Client> {
    let path = session_store::default_session_path().context("resolve session-store path")?;
    let session = match session_store::load(&path) {
        Ok(s) => s,
        Err(session_store::SessionStoreError::NotFound { .. }) => {
            anyhow::bail!(
                "no saved session at {} — run `baby-phi bootstrap claim --credential <…>` first",
                path.display()
            );
        }
        Err(e) => anyhow::bail!("failed to load saved session: {e}"),
    };
    let mut headers = HeaderMap::new();
    let cookie = format!("baby_phi_session={}", session.cookie_value);
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
            eprintln!("baby-phi: rejected ({}): {}", err.code, err.message);
            if status.is_server_error() {
                EXIT_INTERNAL
            } else {
                EXIT_REJECTED
            }
        }
        Err(e) => {
            eprintln!("baby-phi: HTTP {} with no error body: {e}", status.as_u16());
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
    fn tenants_spec_all() {
        let v = parse_tenants_spec("all").unwrap();
        assert_eq!(v["mode"], "all");
    }

    #[test]
    fn tenants_spec_only_list() {
        let uuid = uuid::Uuid::new_v4().to_string();
        let v = parse_tenants_spec(&uuid).unwrap();
        assert_eq!(v["mode"], "only");
        assert_eq!(v["orgs"][0], uuid);
    }

    #[test]
    fn tenants_spec_rejects_non_uuid() {
        assert!(parse_tenants_spec("not-a-uuid").is_err());
    }

    #[test]
    fn kind_arg_wires_to_snake_case() {
        assert_eq!(KindArg::Mcp.as_wire(), "mcp");
        assert_eq!(KindArg::OpenApi.as_wire(), "open_api");
    }
}
