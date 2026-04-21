//! `baby-phi secret {list,add,rotate,reveal,reassign}` subcommands.
//!
//! Every subcommand loads the saved session cookie (see
//! [`session_store`](crate::session_store)), builds a reqwest client that
//! auto-sends it, and calls the HTTP endpoint on the acceptance server.
//! Error handling mirrors [`super::bootstrap`] — stable exit codes live
//! in [`crate::exit`].
//!
//! Material I/O:
//! - `add` / `rotate` take `--material-file <PATH>`. `-` reads from stdin
//!   (the canonical way to pipe a secret without landing it on disk).
//! - `reveal` writes plaintext to stdout only when `--accept-audit` is
//!   passed — otherwise the command prints the audit-event id + reminds
//!   the operator that reveal always records to the alert channel.

use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use base64::engine::general_purpose::STANDARD_NO_PAD as BASE64_NOPAD;
use base64::Engine as _;
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
pub enum SecretCommand {
    /// List vault entries (metadata only — never plaintext).
    List {
        /// Render as JSON rather than a human table.
        #[arg(long)]
        json: bool,
    },
    /// Add a new vault entry.
    Add {
        /// Stable slug (lowercase, digits, dashes). Becomes `secret:<slug>`
        /// in the catalogue.
        #[arg(long)]
        slug: String,
        /// File containing the plaintext material. Pass `-` to read from
        /// stdin (preferred for secrets that should not hit disk).
        #[arg(long)]
        material_file: PathBuf,
        /// Mask the value in list views + audit diffs (default: true).
        #[arg(long, default_value_t = true)]
        sensitive: bool,
    },
    /// Rotate the sealed material on an existing entry.
    Rotate {
        #[arg(long)]
        slug: String,
        #[arg(long)]
        material_file: PathBuf,
    },
    /// Unseal + reveal plaintext. Always audited (Alerted class).
    Reveal {
        #[arg(long)]
        slug: String,
        /// Operator justification. Surfaced in the audit diff.
        #[arg(long)]
        purpose: String,
        /// Acknowledge that reveal is recorded to the alert channel.
        /// Without this flag, plaintext is NOT printed — the command
        /// reports the server-side event id + aborts.
        #[arg(long = "accept-audit", default_value_t = false)]
        accept_audit: bool,
    },
    /// Reassign custody of an entry to a different Agent.
    Reassign {
        #[arg(long)]
        slug: String,
        /// Target agent's UUID.
        #[arg(long = "new-custodian")]
        new_custodian: String,
    },
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub async fn run(server_url_override: Option<String>, cmd: SecretCommand) -> i32 {
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
        SecretCommand::List { json } => list(&client, &base, json).await,
        SecretCommand::Add {
            slug,
            material_file,
            sensitive,
        } => add(&client, &base, &slug, &material_file, sensitive).await,
        SecretCommand::Rotate {
            slug,
            material_file,
        } => rotate(&client, &base, &slug, &material_file).await,
        SecretCommand::Reveal {
            slug,
            purpose,
            accept_audit,
        } => reveal(&client, &base, &slug, &purpose, accept_audit).await,
        SecretCommand::Reassign {
            slug,
            new_custodian,
        } => reassign(&client, &base, &slug, &new_custodian).await,
    }
}

// ---------------------------------------------------------------------------
// Wire shapes (mirror `server/src/handlers/platform_secrets.rs`)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct AddBody<'a> {
    slug: &'a str,
    material_b64: String,
    sensitive: bool,
}

#[derive(Debug, Serialize)]
struct RotateBody {
    material_b64: String,
}

#[derive(Debug, Serialize)]
struct RevealBody<'a> {
    justification: &'a str,
}

#[derive(Debug, Serialize)]
struct ReassignBody<'a> {
    new_custodian_agent_id: &'a str,
}

#[derive(Debug, Deserialize)]
struct SecretSummaryWire {
    id: String,
    slug: String,
    custodian_id: String,
    sensitive: bool,
    last_rotated_at: Option<String>,
    created_at: String,
}

#[derive(Debug, Deserialize)]
struct ListWire {
    secrets: Vec<SecretSummaryWire>,
}

#[derive(Debug, Deserialize)]
struct AddWire {
    secret_id: String,
    slug: String,
    auth_request_id: String,
    audit_event_id: String,
}

#[derive(Debug, Deserialize)]
struct WriteWire {
    secret_id: String,
    slug: String,
    audit_event_id: String,
}

#[derive(Debug, Deserialize)]
struct RevealWire {
    secret_id: String,
    slug: String,
    material_b64: String,
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

async fn list(client: &reqwest::Client, base: &str, json: bool) -> i32 {
    let url = format!("{base}/api/v0/platform/secrets");
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
            serde_json::to_string_pretty(&serde_json::json!(
                { "secrets": body.secrets.iter().map(|s| serde_json::json!({
                    "id": s.id, "slug": s.slug, "custodian_id": s.custodian_id,
                    "sensitive": s.sensitive, "last_rotated_at": s.last_rotated_at,
                    "created_at": s.created_at,
                })).collect::<Vec<_>>() }
            ))
            .unwrap()
        );
    } else if body.secrets.is_empty() {
        println!("(vault is empty — run `baby-phi secret add --slug <…> --material-file <…>` to register one)");
    } else {
        println!(
            "{:<40}  {:<14}  {:<10}  custodian",
            "slug", "sensitive", "rotated"
        );
        for s in &body.secrets {
            let rotated = s.last_rotated_at.as_deref().unwrap_or("(never)");
            println!(
                "{:<40}  {:<14}  {:<10}  {}",
                s.slug,
                if s.sensitive { "sensitive" } else { "clear" },
                rotated,
                s.custodian_id,
            );
        }
    }
    EXIT_OK
}

async fn add(
    client: &reqwest::Client,
    base: &str,
    slug: &str,
    material_file: &Path,
    sensitive: bool,
) -> i32 {
    let material = match read_material(material_file) {
        Ok(m) => m,
        Err(code) => return code,
    };
    let body = AddBody {
        slug,
        material_b64: BASE64_NOPAD.encode(&material),
        sensitive,
    };
    let url = format!("{base}/api/v0/platform/secrets");
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
    println!("secret added");
    println!("  secret_id:        {}", body.secret_id);
    println!("  slug:             {}", body.slug);
    println!("  auth_request_id:  {}", body.auth_request_id);
    println!("  audit_event_id:   {}", body.audit_event_id);
    EXIT_OK
}

async fn rotate(client: &reqwest::Client, base: &str, slug: &str, material_file: &Path) -> i32 {
    let material = match read_material(material_file) {
        Ok(m) => m,
        Err(code) => return code,
    };
    let body = RotateBody {
        material_b64: BASE64_NOPAD.encode(&material),
    };
    let url = format!("{base}/api/v0/platform/secrets/{slug}/rotate");
    let res = match client.post(&url).json(&body).send().await {
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
    let body: WriteWire = match res.json().await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("baby-phi: failed to decode rotate response: {e}");
            return EXIT_INTERNAL;
        }
    };
    println!("secret rotated");
    println!("  secret_id:       {}", body.secret_id);
    println!("  slug:            {}", body.slug);
    println!("  audit_event_id:  {}", body.audit_event_id);
    EXIT_OK
}

async fn reveal(
    client: &reqwest::Client,
    base: &str,
    slug: &str,
    purpose: &str,
    accept_audit: bool,
) -> i32 {
    if !accept_audit {
        eprintln!(
            "baby-phi: reveal is always recorded to the alert channel (Alerted audit class)."
        );
        eprintln!(
            "  re-run with `--accept-audit` to confirm you understand the audit implications."
        );
        return EXIT_PRECONDITION_FAILED;
    }
    let body = RevealBody {
        justification: purpose,
    };
    let url = format!("{base}/api/v0/platform/secrets/{slug}/reveal");
    let res = match client.post(&url).json(&body).send().await {
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
    let body: RevealWire = match res.json().await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("baby-phi: failed to decode reveal response: {e}");
            return EXIT_INTERNAL;
        }
    };
    let plaintext = match BASE64_NOPAD.decode(body.material_b64.as_bytes()) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("baby-phi: reveal response carried invalid base64: {e}");
            return EXIT_INTERNAL;
        }
    };
    // Header goes to stderr so operators can pipe stdout into another process
    // without the annotation getting in the way.
    eprintln!(
        "secret revealed (audit_event_id = {}); plaintext on stdout:",
        body.audit_event_id
    );
    use std::io::Write;
    if let Err(e) = std::io::stdout().write_all(&plaintext) {
        eprintln!("baby-phi: failed to write plaintext to stdout: {e}");
        return EXIT_INTERNAL;
    }
    // Trailing newline so pipes terminate cleanly.
    let _ = std::io::stdout().write_all(b"\n");
    // Best-effort purge so the Vec doesn't live in CLI memory longer than needed.
    drop(plaintext);
    drop(body.slug);
    drop(body.secret_id);
    EXIT_OK
}

async fn reassign(client: &reqwest::Client, base: &str, slug: &str, new_custodian: &str) -> i32 {
    let body = ReassignBody {
        new_custodian_agent_id: new_custodian,
    };
    let url = format!("{base}/api/v0/platform/secrets/{slug}/reassign-custody");
    let res = match client.post(&url).json(&body).send().await {
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
    let body: WriteWire = match res.json().await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("baby-phi: failed to decode reassign response: {e}");
            return EXIT_INTERNAL;
        }
    };
    println!("custody reassigned");
    println!("  secret_id:       {}", body.secret_id);
    println!("  slug:            {}", body.slug);
    println!("  audit_event_id:  {}", body.audit_event_id);
    EXIT_OK
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn read_material(path: &Path) -> Result<Vec<u8>, i32> {
    if path == Path::new("-") {
        let mut buf = Vec::new();
        if let Err(e) = std::io::stdin().read_to_end(&mut buf) {
            eprintln!("baby-phi: failed to read material from stdin: {e}");
            return Err(EXIT_INTERNAL);
        }
        if buf.is_empty() {
            eprintln!("baby-phi: material read from stdin was empty");
            return Err(EXIT_REJECTED);
        }
        Ok(buf)
    } else {
        match std::fs::read(path) {
            Ok(buf) if buf.is_empty() => {
                eprintln!("baby-phi: material file {} was empty", path.display());
                Err(EXIT_REJECTED)
            }
            Ok(buf) => Ok(buf),
            Err(e) => {
                eprintln!(
                    "baby-phi: failed to read material from {}: {e}",
                    path.display()
                );
                Err(EXIT_PRECONDITION_FAILED)
            }
        }
    }
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
    fn resolve_base_url_strips_trailing_slashes() {
        let u = resolve_base_url(Some("http://example:8080///".to_string())).unwrap();
        assert_eq!(u, "http://example:8080");
    }
}
