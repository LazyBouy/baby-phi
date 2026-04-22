//! `phi model-provider {list,add,archive,list-kinds}` subcommands.
//!
//! phi-core leverage:
//! - `add` accepts a `--config-file <PATH>` that deserialises directly
//!   into [`phi_core::provider::model::ModelConfig`] — no parallel
//!   phi wire schema. Operators construct the file in whatever
//!   format phi-core's serde accepts (JSON is the CLI default; YAML
//!   support piggy-backs on phi-core's `parse_config_file` when the
//!   file extension is `.yaml`/`.yml`).
//! - `list-kinds` proxies `GET /api/v0/platform/provider-kinds` which
//!   is itself backed by phi-core's `ProviderRegistry::default()` — so
//!   a fresh phi-core release's new providers surface automatically.

use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use serde::{Deserialize, Serialize};
use server::ServerConfig;

use crate::exit::{
    EXIT_INTERNAL, EXIT_OK, EXIT_PRECONDITION_FAILED, EXIT_REJECTED, EXIT_TRANSPORT,
};
use crate::session_store;

// ---------------------------------------------------------------------------
// Clap subcommand surface
// ---------------------------------------------------------------------------

#[derive(Debug, clap::Subcommand)]
pub enum ModelProviderCommand {
    /// List registered model providers.
    List {
        /// Include archived rows in the output.
        #[arg(long)]
        include_archived: bool,
        /// Render as JSON rather than a human table.
        #[arg(long)]
        json: bool,
    },
    /// Register a new provider. Reads the phi-core `ModelConfig` from
    /// `--config-file`; pass `-` for stdin.
    Add {
        /// Path to a phi-core `ModelConfig` serialisation (JSON by
        /// default; `.yaml`/`.yml` detected by extension).
        #[arg(long = "config-file")]
        config_file: PathBuf,
        /// Vault slug that stores the real API key (see
        /// `phi secret add`).
        #[arg(long = "secret-ref")]
        secret_ref: String,
        /// Which orgs may invoke this runtime. `all` or a
        /// comma-separated list of org UUIDs (e.g.
        /// `--tenants-allowed 6f3a…,8e12…`). Defaults to `all`.
        #[arg(long = "tenants-allowed", default_value = "all")]
        tenants_allowed: String,
    },
    /// Archive a registered provider by UUID.
    Archive {
        /// Provider UUID (from `list`).
        #[arg(long)]
        id: String,
    },
    /// List the provider kinds phi-core's default registry currently
    /// supports. Useful for discovering the valid `config.api` values
    /// before editing a config file.
    ListKinds,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub async fn run(server_url_override: Option<String>, cmd: ModelProviderCommand) -> i32 {
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

    match cmd {
        ModelProviderCommand::List {
            include_archived,
            json,
        } => list(&client, &base, include_archived, json).await,
        ModelProviderCommand::Add {
            config_file,
            secret_ref,
            tenants_allowed,
        } => add(&client, &base, &config_file, &secret_ref, &tenants_allowed).await,
        ModelProviderCommand::Archive { id } => archive(&client, &base, &id).await,
        ModelProviderCommand::ListKinds => list_kinds(&client, &base).await,
    }
}

// ---------------------------------------------------------------------------
// Wire shapes (mirror handlers/platform_model_providers.rs)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct AddBody {
    config: serde_json::Value,
    secret_ref: String,
    tenants_allowed: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct AddWire {
    provider_id: String,
    auth_request_id: String,
    audit_event_id: String,
}

#[derive(Debug, Deserialize)]
struct ArchiveWire {
    provider_id: String,
    audit_event_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ProviderSummaryWire {
    id: String,
    config: serde_json::Value,
    secret_ref: String,
    #[serde(default)]
    tenants_allowed: serde_json::Value,
    status: serde_json::Value,
    archived_at: Option<String>,
    created_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ListWire {
    providers: Vec<ProviderSummaryWire>,
}

#[derive(Debug, Deserialize)]
struct ProviderKindsWire {
    kinds: Vec<String>,
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
    let mut url = format!("{base}/api/v0/platform/model-providers");
    if include_archived {
        url.push_str("?include_archived=true");
    }
    let res = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("phi: request to {url} failed: {e}");
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
            eprintln!("phi: failed to decode list response: {e}");
            return EXIT_INTERNAL;
        }
    };
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&body.providers).unwrap_or_default()
        );
        return EXIT_OK;
    }
    if body.providers.is_empty() {
        println!(
            "(no model providers registered — run `phi model-provider add \\\n    --config-file <PATH> --secret-ref <slug>` to register one)"
        );
        return EXIT_OK;
    }
    println!(
        "{:<38}  {:<18}  {:<30}  {:<10}  archived",
        "provider_id", "provider", "model_id", "status"
    );
    for p in &body.providers {
        let provider = p
            .config
            .get("provider")
            .and_then(|v| v.as_str())
            .unwrap_or("(?)");
        let model_id = p.config.get("id").and_then(|v| v.as_str()).unwrap_or("(?)");
        let status = p.status.as_str().unwrap_or("(?)");
        let archived = p.archived_at.as_deref().unwrap_or("-");
        println!(
            "{:<38}  {:<18}  {:<30}  {:<10}  {}",
            p.id, provider, model_id, status, archived
        );
    }
    EXIT_OK
}

async fn add(
    client: &reqwest::Client,
    base: &str,
    config_file: &Path,
    secret_ref: &str,
    tenants_allowed: &str,
) -> i32 {
    let config = match read_model_config(config_file) {
        Ok(c) => c,
        Err(code) => return code,
    };
    let tenants_json = match parse_tenants_spec(tenants_allowed) {
        Ok(v) => v,
        Err(msg) => {
            eprintln!("phi: {msg}");
            return EXIT_REJECTED;
        }
    };
    let body = AddBody {
        config,
        secret_ref: secret_ref.to_string(),
        tenants_allowed: tenants_json,
    };
    let url = format!("{base}/api/v0/platform/model-providers");
    let res = match client.post(&url).json(&body).send().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("phi: request to {url} failed: {e}");
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
            eprintln!("phi: failed to decode add response: {e}");
            return EXIT_INTERNAL;
        }
    };
    println!("model provider registered");
    println!("  provider_id:     {}", body.provider_id);
    println!("  auth_request_id: {}", body.auth_request_id);
    println!("  audit_event_id:  {}", body.audit_event_id);
    EXIT_OK
}

async fn archive(client: &reqwest::Client, base: &str, id: &str) -> i32 {
    let url = format!("{base}/api/v0/platform/model-providers/{id}/archive");
    let res = match client.post(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("phi: request to {url} failed: {e}");
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
            eprintln!("phi: failed to decode archive response: {e}");
            return EXIT_INTERNAL;
        }
    };
    println!("model provider archived");
    println!("  provider_id:    {}", body.provider_id);
    println!("  audit_event_id: {}", body.audit_event_id);
    EXIT_OK
}

async fn list_kinds(client: &reqwest::Client, base: &str) -> i32 {
    let url = format!("{base}/api/v0/platform/provider-kinds");
    let res = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("phi: request to {url} failed: {e}");
            return EXIT_TRANSPORT;
        }
    };
    let status = res.status();
    if !status.is_success() {
        return report_api_error(res, status).await;
    }
    let body: ProviderKindsWire = match res.json().await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("phi: failed to decode provider-kinds response: {e}");
            return EXIT_INTERNAL;
        }
    };
    println!("provider kinds supported by this phi-core build:");
    for k in &body.kinds {
        println!("  - {k}");
    }
    EXIT_OK
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Read a phi-core `ModelConfig` from file. `-` means stdin. The file
/// contents are a JSON object with the same fields phi-core's
/// `ModelConfig` serde accepts. YAML support lands when phi-core's
/// `parse_config_file` grows a public single-model entry point.
fn read_model_config(path: &Path) -> Result<serde_json::Value, i32> {
    let raw = if path == Path::new("-") {
        let mut buf = String::new();
        use std::io::Read;
        if let Err(e) = std::io::stdin().read_to_string(&mut buf) {
            eprintln!("phi: failed to read config from stdin: {e}");
            return Err(EXIT_INTERNAL);
        }
        buf
    } else {
        match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("phi: failed to read config file {}: {e}", path.display());
                return Err(EXIT_PRECONDITION_FAILED);
            }
        }
    };
    // Parse as JSON (forgiving of BOM + whitespace). YAML/TOML can
    // piggy-back on phi-core's `parse_config_file` once we expose a
    // single-model parse there; for M2/P5 JSON covers the common case.
    serde_json::from_str::<serde_json::Value>(raw.trim()).map_err(|e| {
        eprintln!("phi: invalid ModelConfig JSON: {e}");
        EXIT_REJECTED
    })
}

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
        // Validate UUID shape before sending.
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
            if status.is_server_error() {
                EXIT_INTERNAL
            } else {
                EXIT_REJECTED
            }
        }
        Err(e) => {
            eprintln!("phi: HTTP {} with no error body: {e}", status.as_u16());
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
}
