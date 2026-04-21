//! M2 additions to the SurrealDB-backed `Repository` impl.
//!
//! These are **inherent methods** on [`SurrealStore`]; the thin trait
//! delegations sit alongside the M1 methods in
//! [`super::repo_impl`] so the `Repository` trait impl stays in one
//! `impl` block (Rust requires this). The split keeps `repo_impl.rs`
//! bounded in line count — each milestone's surface gets its own file.
//!
//! Conventions (match M1's `repo_impl.rs`):
//! - Rows use `type::thing('table', $id)` so SurrealDB's record id
//!   carries the same UUID the domain uses.
//! - Domain structs round-trip through `serde_json::Value`; we strip
//!   the `id` field on write and inject it on read.
//! - Ciphertext bytes live in two `string` columns (`..._b64`) per
//!   ADR-0014 — the driver's `bytes` translation is non-trivial so we
//!   base64 everything at the domain/store boundary.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use domain::model::ids::{
    AgentId, AuthRequestId, GrantId, McpServerId, ModelProviderId, OrgId, SecretId,
};
use domain::model::nodes::PrincipalRef;
use domain::model::{
    Composite, ExternalService, ExternalServiceKind, ModelRuntime, PlatformDefaults, RuntimeStatus,
    SecretCredential, SecretRef, TenantSet,
};
use domain::repository::{RepositoryError, RepositoryResult, SealedBlob, TenantRevocation};

use crate::SurrealStore;

// ============================================================================
// Row translators — one per new composite. Each mirrors a table in
// `0001_initial.surql` (secrets_vault, extended in 0001) or
// `0002_platform_setup.surql` (model_runtime, platform_defaults, the
// mcp_server extension).
// ============================================================================

/// Wire shape for `secrets_vault`. Keeps the sealed bytes next to the
/// catalogue metadata so the reveal path does one row read.
#[derive(Debug, Serialize, Deserialize)]
struct SecretRow {
    slug: String,
    custodian_id: String,
    sensitive: bool,
    value_ciphertext_b64: String,
    nonce_b64: String,
    created_at: DateTime<Utc>,
    last_rotated_at: Option<DateTime<Utc>>,
}

impl SecretRow {
    fn into_domain(self, id: SecretId) -> RepositoryResult<(SecretCredential, SealedBlob)> {
        let custodian_uuid = uuid::Uuid::parse_str(&self.custodian_id)
            .map_err(|e| RepositoryError::Backend(format!("invalid custodian_id uuid: {e}")))?;
        let cred = SecretCredential {
            id,
            slug: SecretRef::new(self.slug),
            custodian: AgentId::from_uuid(custodian_uuid),
            last_rotated_at: self.last_rotated_at,
            sensitive: self.sensitive,
            created_at: self.created_at,
        };
        let sealed = SealedBlob {
            ciphertext_b64: self.value_ciphertext_b64,
            nonce_b64: self.nonce_b64,
        };
        Ok((cred, sealed))
    }
}

/// Wire shape for `model_runtime`. The embedded `config` column is a
/// flexible object so phi-core's `ModelConfig` field evolution does not
/// force a baby-phi migration.
#[derive(Debug, Serialize, Deserialize)]
struct ModelRuntimeRow {
    config: serde_json::Value,
    secret_ref: String,
    tenants_allowed: serde_json::Value,
    status: String,
    archived_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl ModelRuntimeRow {
    fn into_domain(self, id: ModelProviderId) -> RepositoryResult<ModelRuntime> {
        let config = serde_json::from_value(self.config)
            .map_err(|e| RepositoryError::Backend(format!("config deserialize: {e}")))?;
        let tenants_allowed = serde_json::from_value(self.tenants_allowed)
            .map_err(|e| RepositoryError::Backend(format!("tenants deserialize: {e}")))?;
        Ok(ModelRuntime {
            id,
            config,
            secret_ref: SecretRef::new(self.secret_ref),
            tenants_allowed,
            status: runtime_status_from_wire(&self.status)?,
            archived_at: self.archived_at,
            created_at: self.created_at,
        })
    }
}

/// Wire shape for `mcp_server` (the M2 extension of the M1 scaffolded
/// table — see `0002_platform_setup.surql`).
#[derive(Debug, Serialize, Deserialize)]
struct McpServerRow {
    display_name: String,
    kind: String,
    endpoint: String,
    secret_ref: Option<String>,
    tenants_allowed: serde_json::Value,
    status: String,
    archived_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl McpServerRow {
    fn into_domain(self, id: McpServerId) -> RepositoryResult<ExternalService> {
        let tenants_allowed = serde_json::from_value(self.tenants_allowed)
            .map_err(|e| RepositoryError::Backend(format!("tenants deserialize: {e}")))?;
        Ok(ExternalService {
            id,
            display_name: self.display_name,
            kind: external_service_kind_from_wire(&self.kind)?,
            endpoint: self.endpoint,
            secret_ref: self.secret_ref.map(SecretRef::new),
            tenants_allowed,
            status: runtime_status_from_wire(&self.status)?,
            archived_at: self.archived_at,
            created_at: self.created_at,
        })
    }
}

fn runtime_status_as_wire(s: RuntimeStatus) -> &'static str {
    match s {
        RuntimeStatus::Ok => "ok",
        RuntimeStatus::Probing => "probing",
        RuntimeStatus::Degraded => "degraded",
        RuntimeStatus::Error => "error",
        RuntimeStatus::Archived => "archived",
    }
}

fn runtime_status_from_wire(s: &str) -> RepositoryResult<RuntimeStatus> {
    match s {
        "ok" => Ok(RuntimeStatus::Ok),
        "probing" => Ok(RuntimeStatus::Probing),
        "degraded" => Ok(RuntimeStatus::Degraded),
        "error" => Ok(RuntimeStatus::Error),
        "archived" => Ok(RuntimeStatus::Archived),
        other => Err(RepositoryError::Backend(format!(
            "unknown runtime status: {other}"
        ))),
    }
}

fn external_service_kind_as_wire(k: ExternalServiceKind) -> &'static str {
    match k {
        ExternalServiceKind::Mcp => "mcp",
        ExternalServiceKind::OpenApi => "open_api",
        ExternalServiceKind::Webhook => "webhook",
        ExternalServiceKind::Other => "other",
    }
}

fn external_service_kind_from_wire(s: &str) -> RepositoryResult<ExternalServiceKind> {
    match s {
        "mcp" => Ok(ExternalServiceKind::Mcp),
        "open_api" => Ok(ExternalServiceKind::OpenApi),
        "webhook" => Ok(ExternalServiceKind::Webhook),
        "other" => Ok(ExternalServiceKind::Other),
        other => Err(RepositoryError::Backend(format!(
            "unknown external service kind: {other}"
        ))),
    }
}

// ============================================================================
// Inherent methods — called from the trait impl block in `repo_impl.rs`.
// ============================================================================

impl SurrealStore {
    fn m2_backend<E: std::fmt::Display>(e: E) -> RepositoryError {
        RepositoryError::Backend(e.to_string())
    }

    // ---- Secrets --------------------------------------------------------

    pub(crate) async fn m2_put_secret(
        &self,
        credential: &SecretCredential,
        sealed: &SealedBlob,
    ) -> RepositoryResult<()> {
        let sql = if credential.last_rotated_at.is_some() {
            "CREATE type::thing('secrets_vault', $id) SET \
             slug = $slug, custodian_id = $custodian, sensitive = $sensitive, \
             value_ciphertext_b64 = $ct, nonce_b64 = $nonce, \
             created_at = $created_at, last_rotated_at = $last_rotated_at \
             RETURN NONE"
        } else {
            "CREATE type::thing('secrets_vault', $id) SET \
             slug = $slug, custodian_id = $custodian, sensitive = $sensitive, \
             value_ciphertext_b64 = $ct, nonce_b64 = $nonce, \
             created_at = $created_at, last_rotated_at = NONE \
             RETURN NONE"
        };
        let mut q = self
            .client()
            .query(sql)
            .bind(("id", credential.id.to_string()))
            .bind(("slug", credential.slug.as_str().to_string()))
            .bind(("custodian", credential.custodian.to_string()))
            .bind(("sensitive", credential.sensitive))
            .bind(("ct", sealed.ciphertext_b64.clone()))
            .bind(("nonce", sealed.nonce_b64.clone()))
            .bind(("created_at", credential.created_at.to_rfc3339()));
        if let Some(at) = credential.last_rotated_at {
            q = q.bind(("last_rotated_at", at.to_rfc3339()));
        }
        let result = q.await;
        let resp = match result {
            Ok(r) => r,
            Err(e) => return Err(map_put_secret_err(e.to_string(), &credential.slug)),
        };
        resp.check()
            .map_err(|e| map_put_secret_err(e.to_string(), &credential.slug))?;
        Ok(())
    }

    pub(crate) async fn m2_get_secret_by_slug(
        &self,
        slug: &SecretRef,
    ) -> RepositoryResult<Option<(SecretCredential, SealedBlob)>> {
        let mut resp = self
            .client()
            .query(
                "SELECT slug, custodian_id, sensitive, value_ciphertext_b64, \
                 nonce_b64, created_at, last_rotated_at, \
                 record::id(id) AS __id OMIT id \
                 FROM secrets_vault WHERE slug = $slug",
            )
            .bind(("slug", slug.as_str().to_string()))
            .await
            .map_err(Self::m2_backend)?;
        // `__id` alias gives us the record-id string; SELECT * inlines the
        // rest of the row.
        let rows: Vec<serde_json::Value> = resp.take(0).map_err(Self::m2_backend)?;
        let row = match rows.into_iter().next() {
            Some(v) => v,
            None => return Ok(None),
        };
        let id_str = row
            .get("__id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| RepositoryError::Backend("secret row missing __id".into()))?;
        let id = SecretId::from_uuid(
            uuid::Uuid::parse_str(id_str)
                .map_err(|e| RepositoryError::Backend(format!("invalid secret uuid: {e}")))?,
        );
        // Strip the helper alias before deserializing.
        let mut row = row;
        if let serde_json::Value::Object(ref mut map) = row {
            map.remove("__id");
            map.remove("id");
        }
        let parsed: SecretRow = serde_json::from_value(row).map_err(Self::m2_backend)?;
        Ok(Some(parsed.into_domain(id)?))
    }

    pub(crate) async fn m2_list_secrets(&self) -> RepositoryResult<Vec<SecretCredential>> {
        let mut resp = self
            .client()
            .query(
                "SELECT slug, custodian_id, sensitive, created_at, last_rotated_at, \
                 record::id(id) AS __id OMIT id FROM secrets_vault",
            )
            .await
            .map_err(Self::m2_backend)?;
        let rows: Vec<serde_json::Value> = resp.take(0).map_err(Self::m2_backend)?;
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            let id_str = row
                .get("__id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RepositoryError::Backend("secret list row missing __id".into()))?;
            let id = SecretId::from_uuid(uuid::Uuid::parse_str(id_str).map_err(Self::m2_backend)?);
            let slug = row
                .get("slug")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RepositoryError::Backend("secret row missing slug".into()))?;
            let custodian_id = row
                .get("custodian_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    RepositoryError::Backend("secret row missing custodian_id".into())
                })?;
            let sensitive = row
                .get("sensitive")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let created_at: DateTime<Utc> = row
                .get("created_at")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RepositoryError::Backend("secret row missing created_at".into()))
                .and_then(|s| {
                    DateTime::parse_from_rfc3339(s)
                        .map(|dt| dt.with_timezone(&Utc))
                        .map_err(Self::m2_backend)
                })?;
            let last_rotated_at = row
                .get("last_rotated_at")
                .and_then(|v| v.as_str())
                .map(|s| {
                    DateTime::parse_from_rfc3339(s)
                        .map(|dt| dt.with_timezone(&Utc))
                        .map_err(Self::m2_backend)
                })
                .transpose()?;
            let custodian =
                AgentId::from_uuid(uuid::Uuid::parse_str(custodian_id).map_err(Self::m2_backend)?);
            out.push(SecretCredential {
                id,
                slug: SecretRef::new(slug),
                custodian,
                last_rotated_at,
                sensitive,
                created_at,
            });
        }
        Ok(out)
    }

    pub(crate) async fn m2_rotate_secret(
        &self,
        id: SecretId,
        new_sealed: &SealedBlob,
        at: DateTime<Utc>,
    ) -> RepositoryResult<()> {
        // `RETURN record::id(id) AS __id` returns a plain string rather
        // than SurrealDB's record-id enum (which serde_json::Value
        // can't deserialize). Same reason M1 uses `OMIT id` on SELECTs.
        let mut resp = self
            .client()
            .query(
                "UPDATE type::thing('secrets_vault', $id) SET \
                 value_ciphertext_b64 = $ct, nonce_b64 = $nonce, \
                 last_rotated_at = $at \
                 RETURN record::id(id) AS __id",
            )
            .bind(("id", id.to_string()))
            .bind(("ct", new_sealed.ciphertext_b64.clone()))
            .bind(("nonce", new_sealed.nonce_b64.clone()))
            .bind(("at", at.to_rfc3339()))
            .await
            .map_err(Self::m2_backend)?;
        let rows: Vec<serde_json::Value> = resp.take(0).map_err(Self::m2_backend)?;
        if rows.is_empty() {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }

    pub(crate) async fn m2_reassign_secret_custodian(
        &self,
        id: SecretId,
        new_custodian: AgentId,
    ) -> RepositoryResult<()> {
        let mut resp = self
            .client()
            .query(
                "UPDATE type::thing('secrets_vault', $id) \
                 SET custodian_id = $cust \
                 RETURN record::id(id) AS __id",
            )
            .bind(("id", id.to_string()))
            .bind(("cust", new_custodian.to_string()))
            .await
            .map_err(Self::m2_backend)?;
        let rows: Vec<serde_json::Value> = resp.take(0).map_err(Self::m2_backend)?;
        if rows.is_empty() {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }

    // ---- Model providers ------------------------------------------------

    pub(crate) async fn m2_put_model_provider(
        &self,
        provider: &ModelRuntime,
    ) -> RepositoryResult<()> {
        let config = serde_json::to_value(&provider.config).map_err(Self::m2_backend)?;
        let tenants = serde_json::to_value(&provider.tenants_allowed).map_err(Self::m2_backend)?;
        self.client()
            .query(
                "CREATE type::thing('model_runtime', $id) SET \
                 config = $config, \
                 secret_ref = $secret_ref, \
                 tenants_allowed = $tenants, \
                 status = $status, \
                 archived_at = $archived_at, \
                 created_at = $created_at \
                 RETURN NONE",
            )
            .bind(("id", provider.id.to_string()))
            .bind(("config", config))
            .bind(("secret_ref", provider.secret_ref.as_str().to_string()))
            .bind(("tenants", tenants))
            .bind((
                "status",
                runtime_status_as_wire(provider.status).to_string(),
            ))
            .bind(("archived_at", provider.archived_at.map(|d| d.to_rfc3339())))
            .bind(("created_at", provider.created_at.to_rfc3339()))
            .await
            .map_err(Self::m2_backend)?
            .check()
            .map_err(Self::m2_backend)?;
        Ok(())
    }

    pub(crate) async fn m2_list_model_providers(
        &self,
        include_archived: bool,
    ) -> RepositoryResult<Vec<ModelRuntime>> {
        let sql = if include_archived {
            "SELECT *, record::id(id) AS __id OMIT id FROM model_runtime"
        } else {
            "SELECT *, record::id(id) AS __id OMIT id FROM model_runtime WHERE archived_at IS NONE"
        };
        let mut resp = self.client().query(sql).await.map_err(Self::m2_backend)?;
        let rows: Vec<serde_json::Value> = resp.take(0).map_err(Self::m2_backend)?;
        let mut out = Vec::with_capacity(rows.len());
        for mut row in rows {
            let id_str = row
                .get("__id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RepositoryError::Backend("model_runtime row missing __id".into()))?;
            let id = ModelProviderId::from_uuid(
                uuid::Uuid::parse_str(id_str).map_err(Self::m2_backend)?,
            );
            if let serde_json::Value::Object(ref mut map) = row {
                map.remove("__id");
                map.remove("id");
            }
            let parsed: ModelRuntimeRow = serde_json::from_value(row).map_err(Self::m2_backend)?;
            out.push(parsed.into_domain(id)?);
        }
        Ok(out)
    }

    pub(crate) async fn m2_archive_model_provider(
        &self,
        id: ModelProviderId,
        at: DateTime<Utc>,
    ) -> RepositoryResult<()> {
        let mut resp = self
            .client()
            .query(
                "UPDATE type::thing('model_runtime', $id) SET \
                 archived_at = $at, status = 'archived' RETURN record::id(id) AS __id",
            )
            .bind(("id", id.to_string()))
            .bind(("at", at.to_rfc3339()))
            .await
            .map_err(Self::m2_backend)?;
        let rows: Vec<serde_json::Value> = resp.take(0).map_err(Self::m2_backend)?;
        if rows.is_empty() {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }

    // ---- MCP servers ----------------------------------------------------

    pub(crate) async fn m2_put_mcp_server(&self, server: &ExternalService) -> RepositoryResult<()> {
        let tenants = serde_json::to_value(&server.tenants_allowed).map_err(Self::m2_backend)?;
        self.client()
            .query(
                "CREATE type::thing('mcp_server', $id) SET \
                 display_name = $name, \
                 kind = $kind, \
                 endpoint = $endpoint, \
                 secret_ref = $secret_ref, \
                 tenants_allowed = $tenants, \
                 status = $status, \
                 archived_at = $archived_at, \
                 created_at = $created_at \
                 RETURN NONE",
            )
            .bind(("id", server.id.to_string()))
            .bind(("name", server.display_name.clone()))
            .bind((
                "kind",
                external_service_kind_as_wire(server.kind).to_string(),
            ))
            .bind(("endpoint", server.endpoint.clone()))
            .bind((
                "secret_ref",
                server.secret_ref.as_ref().map(|r| r.as_str().to_string()),
            ))
            .bind(("tenants", tenants))
            .bind(("status", runtime_status_as_wire(server.status).to_string()))
            .bind(("archived_at", server.archived_at.map(|d| d.to_rfc3339())))
            .bind(("created_at", server.created_at.to_rfc3339()))
            .await
            .map_err(Self::m2_backend)?
            .check()
            .map_err(Self::m2_backend)?;
        Ok(())
    }

    pub(crate) async fn m2_list_mcp_servers(
        &self,
        include_archived: bool,
    ) -> RepositoryResult<Vec<ExternalService>> {
        let sql = if include_archived {
            "SELECT *, record::id(id) AS __id OMIT id FROM mcp_server"
        } else {
            "SELECT *, record::id(id) AS __id OMIT id FROM mcp_server WHERE archived_at IS NONE"
        };
        let mut resp = self.client().query(sql).await.map_err(Self::m2_backend)?;
        let rows: Vec<serde_json::Value> = resp.take(0).map_err(Self::m2_backend)?;
        let mut out = Vec::with_capacity(rows.len());
        for mut row in rows {
            let id_str = row
                .get("__id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RepositoryError::Backend("mcp_server row missing __id".into()))?;
            let id =
                McpServerId::from_uuid(uuid::Uuid::parse_str(id_str).map_err(Self::m2_backend)?);
            if let serde_json::Value::Object(ref mut map) = row {
                map.remove("__id");
                map.remove("id");
            }
            let parsed: McpServerRow = serde_json::from_value(row).map_err(Self::m2_backend)?;
            out.push(parsed.into_domain(id)?);
        }
        Ok(out)
    }

    pub(crate) async fn m2_patch_mcp_tenants(
        &self,
        id: McpServerId,
        new_allowed: &TenantSet,
    ) -> RepositoryResult<()> {
        let tenants = serde_json::to_value(new_allowed).map_err(Self::m2_backend)?;
        let mut resp = self
            .client()
            .query(
                "UPDATE type::thing('mcp_server', $id) \
                 SET tenants_allowed = $tenants \
                 RETURN record::id(id) AS __id",
            )
            .bind(("id", id.to_string()))
            .bind(("tenants", tenants))
            .await
            .map_err(Self::m2_backend)?;
        let rows: Vec<serde_json::Value> = resp.take(0).map_err(Self::m2_backend)?;
        if rows.is_empty() {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }

    pub(crate) async fn m2_archive_mcp_server(
        &self,
        id: McpServerId,
        at: DateTime<Utc>,
    ) -> RepositoryResult<()> {
        let mut resp = self
            .client()
            .query(
                "UPDATE type::thing('mcp_server', $id) SET \
                 archived_at = $at, status = 'archived' RETURN record::id(id) AS __id",
            )
            .bind(("id", id.to_string()))
            .bind(("at", at.to_rfc3339()))
            .await
            .map_err(Self::m2_backend)?;
        let rows: Vec<serde_json::Value> = resp.take(0).map_err(Self::m2_backend)?;
        if rows.is_empty() {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }

    // ---- Platform defaults ---------------------------------------------

    pub(crate) async fn m2_get_platform_defaults(
        &self,
    ) -> RepositoryResult<Option<PlatformDefaults>> {
        let mut resp = self
            .client()
            .query("SELECT * OMIT id FROM platform_defaults WHERE singleton = 1 LIMIT 1")
            .await
            .map_err(Self::m2_backend)?;
        let rows: Vec<serde_json::Value> = resp.take(0).map_err(Self::m2_backend)?;
        let row = match rows.into_iter().next() {
            Some(v) => v,
            None => return Ok(None),
        };
        let mut row = row;
        if let serde_json::Value::Object(ref mut map) = row {
            map.remove("id");
        }
        let parsed: PlatformDefaults = serde_json::from_value(row).map_err(Self::m2_backend)?;
        Ok(Some(parsed))
    }

    pub(crate) async fn m2_put_platform_defaults(
        &self,
        defaults: &PlatformDefaults,
    ) -> RepositoryResult<()> {
        let exec = serde_json::to_value(&defaults.execution_limits).map_err(Self::m2_backend)?;
        let profile =
            serde_json::to_value(&defaults.default_agent_profile).map_err(Self::m2_backend)?;
        let ctx = serde_json::to_value(&defaults.context_config).map_err(Self::m2_backend)?;
        let retry = serde_json::to_value(&defaults.retry_config).map_err(Self::m2_backend)?;
        // Upsert on the deterministic `platform_defaults:singleton`
        // record id. The UNIQUE INDEX on `singleton = 1` rejects any
        // attempt to create a second row. `UPSERT` (SurrealDB 2.x) both
        // creates the row on first call and updates it on subsequent
        // calls.
        self.client()
            .query(
                "UPSERT type::thing('platform_defaults', 'singleton') SET \
                 singleton = 1, \
                 execution_limits = $exec, \
                 default_agent_profile = $profile, \
                 context_config = $ctx, \
                 retry_config = $retry, \
                 default_retention_days = $retention, \
                 default_alert_channels = $channels, \
                 updated_at = $updated_at, \
                 version = $version \
                 RETURN NONE",
            )
            .bind(("exec", exec))
            .bind(("profile", profile))
            .bind(("ctx", ctx))
            .bind(("retry", retry))
            .bind(("retention", defaults.default_retention_days as i64))
            .bind(("channels", defaults.default_alert_channels.clone()))
            .bind(("updated_at", defaults.updated_at.to_rfc3339()))
            .bind(("version", defaults.version as i64))
            .await
            .map_err(Self::m2_backend)?
            .check()
            .map_err(Self::m2_backend)?;
        Ok(())
    }

    // ---- Cascade --------------------------------------------------------

    pub(crate) async fn m2_narrow_mcp_tenants(
        &self,
        id: McpServerId,
        new_allowed: &TenantSet,
        at: DateTime<Utc>,
    ) -> RepositoryResult<Vec<TenantRevocation>> {
        // Step 1 — read the current tenants_allowed so we can compute
        // the diff. SurrealDB does not offer an atomic "compare and
        // swap" for flexible objects, so we lean on the single-writer
        // embedded RocksDB backend for M2; the server crate's request
        // layer serialises writes per handler. True cross-process
        // atomicity lands with the standalone SurrealDB cluster at
        // M7b+.
        let mut resp = self
            .client()
            .query("SELECT tenants_allowed OMIT id FROM type::thing('mcp_server', $id)")
            .bind(("id", id.to_string()))
            .await
            .map_err(Self::m2_backend)?;
        let rows: Vec<serde_json::Value> = resp.take(0).map_err(Self::m2_backend)?;
        let row = rows.into_iter().next().ok_or(RepositoryError::NotFound)?;
        let old_raw = row.get("tenants_allowed").cloned().ok_or_else(|| {
            RepositoryError::Backend("mcp_server row missing tenants_allowed".into())
        })?;
        let old: TenantSet = serde_json::from_value(old_raw).map_err(Self::m2_backend)?;

        let dropped = dropped_orgs(&old, new_allowed);

        // Step 2 — overwrite tenants_allowed.
        let tenants = serde_json::to_value(new_allowed).map_err(Self::m2_backend)?;
        self.client()
            .query(
                "UPDATE type::thing('mcp_server', $id) \
                 SET tenants_allowed = $tenants RETURN NONE",
            )
            .bind(("id", id.to_string()))
            .bind(("tenants", tenants))
            .await
            .map_err(Self::m2_backend)?
            .check()
            .map_err(Self::m2_backend)?;

        // No orgs dropped → nothing to cascade.
        if dropped.is_empty() {
            return Ok(vec![]);
        }

        // Step 3 — per dropped org, find every live AR whose requestor
        // was that org, then revoke every grant descending from it.
        let mut out: Vec<TenantRevocation> = Vec::new();
        for org in dropped {
            let mut resp = self
                .client()
                .query(
                    "SELECT record::id(id) AS __id OMIT id FROM auth_request \
                     WHERE requestor_kind = 'organization' AND requestor_id = $org \
                       AND archived = false",
                )
                .bind(("org", org.to_string()))
                .await
                .map_err(Self::m2_backend)?;
            let ar_rows: Vec<serde_json::Value> = resp.take(0).map_err(Self::m2_backend)?;
            for ar_row in ar_rows {
                let ar_id_str = ar_row.get("__id").and_then(|v| v.as_str()).ok_or_else(|| {
                    RepositoryError::Backend("auth_request row missing __id".into())
                })?;
                let ar = AuthRequestId::from_uuid(
                    uuid::Uuid::parse_str(ar_id_str).map_err(Self::m2_backend)?,
                );
                let revoked = self.m2_revoke_grants_by_descends_from(ar, at).await?;
                if !revoked.is_empty() {
                    out.push(TenantRevocation {
                        org,
                        auth_request: ar,
                        revoked_grants: revoked,
                    });
                }
            }
        }
        Ok(out)
    }

    pub(crate) async fn m2_revoke_grants_by_descends_from(
        &self,
        ar: AuthRequestId,
        at: DateTime<Utc>,
    ) -> RepositoryResult<Vec<GrantId>> {
        let mut resp = self
            .client()
            .query(
                "UPDATE grant SET revoked_at = $at \
                 WHERE descends_from = $ar AND revoked_at IS NONE \
                 RETURN record::id(id) AS __id",
            )
            .bind(("ar", ar.to_string()))
            .bind(("at", at.to_rfc3339()))
            .await
            .map_err(Self::m2_backend)?;
        let rows: Vec<serde_json::Value> = resp.take(0).map_err(Self::m2_backend)?;
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            let id_str = row
                .get("__id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RepositoryError::Backend("grant update missing __id".into()))?;
            out.push(GrantId::from_uuid(
                uuid::Uuid::parse_str(id_str).map_err(Self::m2_backend)?,
            ));
        }
        Ok(out)
    }

    // ---- Catalogue ------------------------------------------------------

    pub(crate) async fn m2_seed_catalogue_entry_for_composite(
        &self,
        owning_org: Option<OrgId>,
        resource_uri: &str,
        composite: Composite,
    ) -> RepositoryResult<()> {
        // Thin wrapper over the M1 method — delegates to the existing
        // Repository impl so behaviour (idempotency, etc.) is shared.
        use domain::repository::Repository;
        self.seed_catalogue_entry(owning_org, resource_uri, composite.kind_name())
            .await
    }
}

fn map_put_secret_err(msg: String, slug: &SecretRef) -> RepositoryError {
    if msg.contains("already contains")
        || msg.contains("Database index")
        || msg.contains("duplicate")
        || msg.contains("unique")
    {
        RepositoryError::Conflict(format!("vault slug already in use: {slug}"))
    } else {
        RepositoryError::Backend(msg)
    }
}

/// Dropped-orgs helper — mirrors `in_memory::dropped_orgs`. Kept next
/// to the SurrealStore impl so cascades see identical semantics.
fn dropped_orgs(old: &TenantSet, new: &TenantSet) -> Vec<OrgId> {
    match (old, new) {
        (TenantSet::Only(old_ids), TenantSet::Only(new_ids)) => old_ids
            .iter()
            .filter(|o| !new_ids.contains(o))
            .copied()
            .collect(),
        (TenantSet::Only(_), TenantSet::All) => vec![],
        (TenantSet::All, TenantSet::All) => vec![],
        (TenantSet::All, TenantSet::Only(_)) => vec![],
    }
}

// Silence unused warnings on helper traits we use for test access.
#[allow(dead_code)]
fn _ensure_principalref_in_scope(_: PrincipalRef) {}
