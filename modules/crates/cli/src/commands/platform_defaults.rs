//! `phi platform-defaults {get,put,factory}` subcommands.
//!
//! phi-core leverage:
//! - The wire struct is `PlatformDefaults` (via the `domain` crate)
//!   which wraps `phi_core::context::execution::ExecutionLimits`,
//!   `phi_core::agents::profile::AgentProfile`,
//!   `phi_core::context::config::ContextConfig`, and
//!   `phi_core::provider::retry::RetryConfig` as direct fields. When
//!   the CLI reads YAML/TOML input it deserialises straight into
//!   `PlatformDefaults` — phi-core's serde shapes the nested layout.
//! - phi-core's `parse_config` pipeline is scoped to `AgentConfig`;
//!   `PlatformDefaults` is a different envelope (a
//!   platform-governance container, not an agent blueprint), so the
//!   CLI does multi-format conversion directly through `serde_yaml` /
//!   `toml` / `serde_json` on the same struct. Reuse boundary is the
//!   embedded phi-core types — not the parser wrapper.

use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use chrono::Utc;
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
pub enum PlatformDefaultsCommand {
    /// Read the current platform defaults. Prints the persisted row
    /// (or the factory baseline when none is stored).
    Get {
        /// Render the factory baseline alongside the live row.
        #[arg(long)]
        include_factory: bool,
        /// Output format. Defaults to JSON (server wire format); YAML
        /// and TOML are generated client-side from the same struct.
        #[arg(long, value_enum, default_value = "json")]
        format: FormatArg,
    },
    /// Update the platform defaults. Reads a file containing the full
    /// `PlatformDefaults` struct in the supplied format; requires
    /// `--if-version` for optimistic concurrency.
    Put {
        /// Path to a `PlatformDefaults` serialisation. `-` reads from
        /// stdin.
        #[arg(long)]
        file: PathBuf,
        /// Input format. Auto-detected from file extension when
        /// unset (`.yaml`/`.yml`, `.toml`, `.json`, everything else →
        /// JSON).
        #[arg(long, value_enum)]
        format: Option<FormatArg>,
        /// Required optimistic-concurrency version. Fetch the
        /// current version via `get --format json` first. Use `0`
        /// for the very first write.
        #[arg(long = "if-version")]
        if_version: u64,
    },
    /// Print the factory baseline defaults without calling the
    /// server. Useful for seeding a first-write file or checking
    /// what `revert to factory` would produce.
    Factory {
        /// Output format.
        #[arg(long, value_enum, default_value = "json")]
        format: FormatArg,
    },
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum FormatArg {
    Json,
    Yaml,
    Toml,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub async fn run(server_url_override: Option<String>, cmd: PlatformDefaultsCommand) -> i32 {
    match cmd {
        PlatformDefaultsCommand::Factory { format } => factory(format),
        PlatformDefaultsCommand::Get {
            include_factory,
            format,
        } => {
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
            get(&client, &base, include_factory, format).await
        }
        PlatformDefaultsCommand::Put {
            file,
            format,
            if_version,
        } => {
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
            put(&client, &base, &file, format, if_version).await
        }
    }
}

// ---------------------------------------------------------------------------
// Wire shapes (mirror handlers/platform_defaults.rs)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize)]
struct GetWire {
    defaults: serde_json::Value,
    persisted: bool,
    factory: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct PutBody {
    if_version: u64,
    defaults: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct PutWire {
    new_version: u64,
    auth_request_id: String,
    audit_event_id: String,
}

#[derive(Debug, Deserialize)]
struct ApiErrorWire {
    code: String,
    message: String,
}

// ---------------------------------------------------------------------------
// Subcommand implementations
// ---------------------------------------------------------------------------

fn factory(format: FormatArg) -> i32 {
    // phi-core's ::default() for every wrapped field — the factory
    // baseline is defined in domain::PlatformDefaults::factory.
    let now = Utc::now();
    let factory = domain::model::PlatformDefaults::factory(now);
    let value = match serde_json::to_value(&factory) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("phi: failed to serialise factory defaults: {e}");
            return EXIT_INTERNAL;
        }
    };
    print_value(&value, format)
}

async fn get(
    client: &reqwest::Client,
    base: &str,
    include_factory: bool,
    format: FormatArg,
) -> i32 {
    let url = format!("{base}/api/v0/platform/defaults");
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
    let body: GetWire = match res.json().await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("phi: failed to decode get response: {e}");
            return EXIT_INTERNAL;
        }
    };
    if !body.persisted {
        eprintln!(
            "# note: no platform_defaults row persisted yet; showing factory baseline (version=0)"
        );
    }
    let out = if include_factory {
        serde_json::json!({
            "defaults": body.defaults,
            "persisted": body.persisted,
            "factory": body.factory,
        })
    } else {
        body.defaults
    };
    print_value(&out, format)
}

async fn put(
    client: &reqwest::Client,
    base: &str,
    file: &Path,
    format_arg: Option<FormatArg>,
    if_version: u64,
) -> i32 {
    // 1. Read + deserialise the struct.
    let raw = match read_file_or_stdin(file) {
        Ok(s) => s,
        Err(code) => return code,
    };
    let format = format_arg.unwrap_or_else(|| detect_format(file));
    let defaults: domain::model::PlatformDefaults = match format {
        FormatArg::Json => match serde_json::from_str(raw.trim()) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("phi: invalid JSON for PlatformDefaults: {e}");
                return EXIT_REJECTED;
            }
        },
        FormatArg::Yaml => match serde_yaml::from_str(&raw) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("phi: invalid YAML for PlatformDefaults: {e}");
                return EXIT_REJECTED;
            }
        },
        FormatArg::Toml => match toml::from_str(&raw) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("phi: invalid TOML for PlatformDefaults: {e}");
                return EXIT_REJECTED;
            }
        },
    };

    // 2. Convert to JSON for the server wire — the server is
    //    JSON-only; YAML / TOML support is purely a client-side
    //    convenience.
    let defaults_json = match serde_json::to_value(&defaults) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("phi: failed to serialise defaults for wire: {e}");
            return EXIT_INTERNAL;
        }
    };

    // 3. PUT.
    let body = PutBody {
        if_version,
        defaults: defaults_json,
    };
    let url = format!("{base}/api/v0/platform/defaults");
    let res = match client.put(&url).json(&body).send().await {
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
    let body: PutWire = match res.json().await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("phi: failed to decode put response: {e}");
            return EXIT_INTERNAL;
        }
    };
    println!("platform defaults updated");
    println!("  new_version:     {}", body.new_version);
    println!("  auth_request_id: {}", body.auth_request_id);
    println!("  audit_event_id:  {}", body.audit_event_id);
    EXIT_OK
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn read_file_or_stdin(path: &Path) -> Result<String, i32> {
    if path == Path::new("-") {
        let mut buf = String::new();
        use std::io::Read;
        if let Err(e) = std::io::stdin().read_to_string(&mut buf) {
            eprintln!("phi: failed to read from stdin: {e}");
            return Err(EXIT_INTERNAL);
        }
        Ok(buf)
    } else {
        std::fs::read_to_string(path).map_err(|e| {
            eprintln!("phi: failed to read {}: {e}", path.display());
            EXIT_PRECONDITION_FAILED
        })
    }
}

fn detect_format(path: &Path) -> FormatArg {
    match path.extension().and_then(|e| e.to_str()) {
        Some("yaml") | Some("yml") => FormatArg::Yaml,
        Some("toml") => FormatArg::Toml,
        _ => FormatArg::Json,
    }
}

fn print_value(value: &serde_json::Value, format: FormatArg) -> i32 {
    match format {
        FormatArg::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(value).unwrap_or_default()
            );
        }
        FormatArg::Yaml => match serde_yaml::to_string(value) {
            Ok(s) => print!("{s}"),
            Err(e) => {
                eprintln!("phi: failed to serialise as YAML: {e}");
                return EXIT_INTERNAL;
            }
        },
        FormatArg::Toml => match toml::to_string_pretty(value) {
            Ok(s) => print!("{s}"),
            Err(e) => {
                eprintln!("phi: failed to serialise as TOML: {e}");
                return EXIT_INTERNAL;
            }
        },
    }
    EXIT_OK
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
    fn detect_format_extensions() {
        assert!(matches!(
            detect_format(Path::new("file.yaml")),
            FormatArg::Yaml
        ));
        assert!(matches!(
            detect_format(Path::new("file.yml")),
            FormatArg::Yaml
        ));
        assert!(matches!(
            detect_format(Path::new("file.toml")),
            FormatArg::Toml
        ));
        assert!(matches!(
            detect_format(Path::new("file.json")),
            FormatArg::Json
        ));
        assert!(matches!(detect_format(Path::new("noext")), FormatArg::Json));
    }
}
