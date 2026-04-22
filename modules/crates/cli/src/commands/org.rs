//! `baby-phi org {create, list, show, dashboard}` subcommands.
//!
//! Create / list / show are wired at M3/P4 against the HTTP surface
//! under `/api/v0/orgs`. Dashboard stays stubbed until M3/P5.
//!
//! ## phi-core leverage
//!
//! Q1 **none** at the CLI tier — no `use phi_core::…` imports.
//! Q2 **yes, via serde**: when the operator supplies
//! `--from-layout <ref>` the fixture YAML may include a
//! `defaults_snapshot_override` field that carries phi-core-wrapping
//! types. We deserialise to `serde_json::Value` and forward verbatim,
//! so field evolution in phi-core never forces a CLI migration.
//! Q3 considered-and-rejected: phi-core has no CLI layer; nothing to
//! reuse.

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

#[derive(Debug, clap::Subcommand)]
pub enum OrgCommand {
    /// Create an organization via the 8-step wizard payload. **[M3/P4]**
    /// Accepts direct flags OR `--from-layout <path>` to seed from a
    /// reference-layout YAML fixture (minimal-startup /
    /// mid-product-team / regulated-enterprise).
    Create {
        /// Operator-facing display name.
        #[arg(long)]
        name: Option<String>,
        /// Optional vision + mission strings (wizard step 1).
        #[arg(long)]
        vision: Option<String>,
        #[arg(long)]
        mission: Option<String>,
        /// Consent policy: implicit / one_time / per_session.
        #[arg(long = "consent-policy", value_enum)]
        consent_policy: Option<ConsentPolicyArg>,
        /// Default audit tier for non-Alerted events.
        #[arg(long = "audit-class-default", value_enum)]
        audit_class_default: Option<AuditClassArg>,
        /// Comma-separated template kinds to adopt at creation time
        /// (subset of `a, b, c, d`).
        #[arg(long = "templates-enabled")]
        templates_enabled: Option<String>,
        #[arg(long = "ceo-display-name")]
        ceo_display_name: Option<String>,
        #[arg(long = "ceo-channel-kind", value_enum)]
        ceo_channel_kind: Option<CeoChannelKindArg>,
        #[arg(long = "ceo-channel-handle")]
        ceo_channel_handle: Option<String>,
        /// Token budget pool ceiling in tokens.
        #[arg(long = "initial-token-allocation")]
        initial_token_allocation: Option<u64>,
        /// Path to a reference-layout YAML fixture (takes precedence
        /// over individual flags when supplied). Ships with
        /// `cli/fixtures/reference_layouts/{minimal-startup,mid-product-team,regulated-enterprise}.yaml`.
        #[arg(long = "from-layout")]
        from_layout: Option<PathBuf>,
        /// Render response as JSON (scripting-friendly) rather than
        /// a human-readable summary.
        #[arg(long)]
        json: bool,
    },
    /// List orgs visible to the calling admin. **[M3/P4]**
    List {
        /// Render as JSON rather than a human table.
        #[arg(long)]
        json: bool,
    },
    /// Show a single org's detail (members, projects, pending ARs,
    /// token budget, etc.). **[M3/P4]**
    Show {
        /// Org UUID.
        #[arg(long)]
        id: String,
        /// Render as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Fetch the consolidated org dashboard payload. **[M3/P5]**
    Dashboard {
        /// Org UUID.
        #[arg(long)]
        id: String,
        /// Render as JSON (scripting-friendly); default human table.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum ConsentPolicyArg {
    Implicit,
    OneTime,
    PerSession,
}

impl ConsentPolicyArg {
    fn as_wire(&self) -> &'static str {
        match self {
            ConsentPolicyArg::Implicit => "implicit",
            ConsentPolicyArg::OneTime => "one_time",
            ConsentPolicyArg::PerSession => "per_session",
        }
    }
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum AuditClassArg {
    Silent,
    Logged,
    Alerted,
}

impl AuditClassArg {
    fn as_wire(&self) -> &'static str {
        match self {
            AuditClassArg::Silent => "silent",
            AuditClassArg::Logged => "logged",
            AuditClassArg::Alerted => "alerted",
        }
    }
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum CeoChannelKindArg {
    Slack,
    Email,
    Web,
}

impl CeoChannelKindArg {
    fn as_wire(&self) -> &'static str {
        match self {
            CeoChannelKindArg::Slack => "slack",
            CeoChannelKindArg::Email => "email",
            CeoChannelKindArg::Web => "web",
        }
    }
}

/// Reserved for the dashboard subcommand; P5 flips it back to 7.
pub const EXIT_NOT_IMPLEMENTED: i32 = 7;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub async fn run(server_url_override: Option<String>, cmd: OrgCommand) -> i32 {
    match cmd {
        OrgCommand::Dashboard { .. } => {
            eprintln!(
                "baby-phi: `org dashboard` is scaffolded but the HTTP \
                 wiring lands in M3/P5. Completion scripts know the \
                 flag surface today."
            );
            EXIT_NOT_IMPLEMENTED
        }
        cmd => {
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
                OrgCommand::Create { .. } => create_impl(&client, &base, cmd).await,
                OrgCommand::List { json } => list_impl(&client, &base, json).await,
                OrgCommand::Show { id, json } => show_impl(&client, &base, &id, json).await,
                OrgCommand::Dashboard { .. } => unreachable!(),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Wire shapes
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct CreateResponseWire {
    org_id: String,
    ceo_agent_id: String,
    #[serde(default)]
    #[allow(dead_code)]
    system_agent_ids: Vec<String>,
    #[serde(default)]
    #[allow(dead_code)]
    adoption_auth_request_ids: Vec<String>,
    #[serde(default)]
    #[allow(dead_code)]
    audit_event_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ListResponseWire {
    orgs: Vec<OrgListItemWire>,
}

#[derive(Debug, Deserialize)]
struct OrgListItemWire {
    id: String,
    display_name: String,
    member_count: usize,
}

#[derive(Debug, Deserialize)]
struct ApiErrorWire {
    code: String,
    message: String,
}

// ---------------------------------------------------------------------------
// Subcommand impls
// ---------------------------------------------------------------------------

async fn create_impl(client: &reqwest::Client, base: &str, cmd: OrgCommand) -> i32 {
    let OrgCommand::Create {
        name,
        vision,
        mission,
        consent_policy,
        audit_class_default,
        templates_enabled,
        ceo_display_name,
        ceo_channel_kind,
        ceo_channel_handle,
        initial_token_allocation,
        from_layout,
        json,
    } = cmd
    else {
        unreachable!()
    };

    // Build request body. If `--from-layout` is supplied, load + use
    // verbatim (phi-core fields transit via serde_yaml → serde_json
    // round-trip). Otherwise assemble from individual flags.
    let body = match from_layout {
        Some(path) => match load_layout(&path) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("baby-phi: failed to load layout: {e:#}");
                return EXIT_INTERNAL;
            }
        },
        None => {
            let Some(name) = name else {
                eprintln!("baby-phi: --name is required (or use --from-layout)");
                return EXIT_PRECONDITION_FAILED;
            };
            let Some(ceo_display_name) = ceo_display_name else {
                eprintln!("baby-phi: --ceo-display-name is required (or use --from-layout)");
                return EXIT_PRECONDITION_FAILED;
            };
            let Some(ceo_channel_handle) = ceo_channel_handle else {
                eprintln!("baby-phi: --ceo-channel-handle is required (or use --from-layout)");
                return EXIT_PRECONDITION_FAILED;
            };
            let Some(initial_token_allocation) = initial_token_allocation else {
                eprintln!(
                    "baby-phi: --initial-token-allocation is required (or use --from-layout)"
                );
                return EXIT_PRECONDITION_FAILED;
            };
            let templates: Vec<String> = templates_enabled
                .as_deref()
                .unwrap_or("")
                .split(',')
                .map(|s| s.trim().to_ascii_lowercase())
                .filter(|s| !s.is_empty())
                .collect();
            serde_json::json!({
                "display_name": name,
                "vision": vision,
                "mission": mission,
                "consent_policy": consent_policy.map(|c| c.as_wire()).unwrap_or("implicit"),
                "audit_class_default": audit_class_default.map(|a| a.as_wire()).unwrap_or("logged"),
                "authority_templates_enabled": templates,
                "default_model_provider": serde_json::Value::Null,
                "ceo_display_name": ceo_display_name,
                "ceo_channel_kind": ceo_channel_kind.map(|c| c.as_wire()).unwrap_or("email"),
                "ceo_channel_handle": ceo_channel_handle,
                "token_budget": initial_token_allocation,
            })
        }
    };

    let url = format!("{base}/api/v0/orgs");
    let res = match client.post(&url).json(&body).send().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("baby-phi: POST {url} failed: {e}");
            return EXIT_TRANSPORT;
        }
    };
    let status = res.status();
    if !status.is_success() {
        return report_api_error(res, status).await;
    }
    let created: CreateResponseWire = match res.json().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("baby-phi: decode response: {e}");
            return EXIT_INTERNAL;
        }
    };
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "org_id": created.org_id,
                "ceo_agent_id": created.ceo_agent_id,
                "system_agent_ids": created.system_agent_ids,
                "adoption_auth_request_ids": created.adoption_auth_request_ids,
                "audit_event_ids": created.audit_event_ids,
            }))
            .unwrap()
        );
    } else {
        println!("org created");
        println!("  id:                 {}", created.org_id);
        println!("  ceo_agent:          {}", created.ceo_agent_id);
        println!(
            "  system_agents:      {} provisioned",
            created.system_agent_ids.len()
        );
        println!(
            "  adoption ARs:       {}",
            created.adoption_auth_request_ids.len()
        );
    }
    EXIT_OK
}

async fn list_impl(client: &reqwest::Client, base: &str, json: bool) -> i32 {
    let url = format!("{base}/api/v0/orgs");
    let res = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("baby-phi: GET {url} failed: {e}");
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
            eprintln!("baby-phi: decode response: {e}");
            return EXIT_INTERNAL;
        }
    };
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({ "orgs":
                body.orgs.iter().map(|o| serde_json::json!({
                    "id": o.id,
                    "display_name": o.display_name,
                    "member_count": o.member_count,
                })).collect::<Vec<_>>()
            }))
            .unwrap()
        );
    } else if body.orgs.is_empty() {
        println!("no organizations yet");
    } else {
        println!("{:<40} {:<30} members", "id", "display_name");
        for org in &body.orgs {
            println!(
                "{:<40} {:<30} {}",
                org.id, org.display_name, org.member_count
            );
        }
    }
    EXIT_OK
}

async fn show_impl(client: &reqwest::Client, base: &str, id: &str, json: bool) -> i32 {
    let url = format!("{base}/api/v0/orgs/{id}");
    let res = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("baby-phi: GET {url} failed: {e}");
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
            eprintln!("baby-phi: decode response: {e}");
            return EXIT_INTERNAL;
        }
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&body).unwrap());
    } else {
        let org = &body["organization"];
        println!(
            "{} ({})",
            org["display_name"].as_str().unwrap_or("?"),
            org["id"].as_str().unwrap_or("?")
        );
        println!(
            "  members: {}  projects: {}  adopted_templates: {}",
            body["member_count"].as_u64().unwrap_or(0),
            body["project_count"].as_u64().unwrap_or(0),
            body["adopted_template_count"].as_u64().unwrap_or(0),
        );
    }
    EXIT_OK
}

// ---------------------------------------------------------------------------
// Helpers — `--from-layout` YAML + HTTP plumbing + error mapping
// ---------------------------------------------------------------------------

/// Load a reference-layout YAML file and deserialise to JSON. Lets
/// the wizard payload carry phi-core-wrapped fields transparently
/// (serde_yaml → serde_json round-trip is lossless for our field
/// shapes).
fn load_layout(path: &std::path::Path) -> Result<serde_json::Value> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("read layout file {}", path.display()))?;
    // Deserialise through serde_yaml's Value then re-encode as JSON —
    // preserves the shape, normalises numeric / string literals.
    let yaml: serde_yaml::Value =
        serde_yaml::from_str(&text).with_context(|| "parse layout as YAML")?;
    let json = serde_json::to_value(yaml).context("convert YAML to JSON")?;
    Ok(json)
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
            EXIT_REJECTED
        }
        Err(_) => {
            eprintln!("baby-phi: rejected (HTTP {}) — non-JSON body", status);
            EXIT_REJECTED
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exit::{EXIT_INTERNAL, EXIT_OK};

    #[test]
    fn exit_not_implemented_is_distinct_from_other_codes() {
        assert_ne!(EXIT_NOT_IMPLEMENTED, EXIT_OK);
        assert_ne!(EXIT_NOT_IMPLEMENTED, EXIT_INTERNAL);
        assert_eq!(EXIT_NOT_IMPLEMENTED, 7);
    }

    #[test]
    fn load_layout_reads_minimal_startup_fixture() {
        // The fixture must parse + carry the required fields.
        let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures/reference_layouts/minimal-startup.yaml");
        let val = load_layout(&fixture_path).expect("parse minimal-startup fixture");
        assert!(val.get("display_name").is_some());
        assert!(val.get("ceo_display_name").is_some());
        assert!(val.get("token_budget").is_some());
    }

    #[test]
    fn all_three_reference_layouts_parse() {
        // Positive fidelity assertion for P4 acceptance — each of the
        // 3 reference layouts must be syntactically valid YAML and
        // carry every field the POST /orgs wire shape requires.
        for name in &[
            "minimal-startup",
            "mid-product-team",
            "regulated-enterprise",
        ] {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(format!("fixtures/reference_layouts/{name}.yaml"));
            let val = load_layout(&path).unwrap_or_else(|e| {
                panic!("parse {name}: {e:#}");
            });
            for required in &[
                "display_name",
                "consent_policy",
                "audit_class_default",
                "authority_templates_enabled",
                "ceo_display_name",
                "ceo_channel_kind",
                "ceo_channel_handle",
                "token_budget",
            ] {
                assert!(
                    val.get(required).is_some(),
                    "{name}: missing required field `{required}`"
                );
            }
        }
    }
}
