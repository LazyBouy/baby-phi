//! SurrealDB-backed `impl Repository for SurrealStore`.
//!
//! Split out of `lib.rs` to keep the adapter small and the impl browsable.
//! Conventions:
//!
//! - **IDs are SurrealDB record ids.** We use `type::thing('table', $uuid)`
//!   so the row's SurrealDB-side id carries the same UUID the domain uses.
//! - **Domain structs round-trip through JSON.** On CREATE we
//!   `serde_json::to_value(struct)`, remove the `id` field, and pass the
//!   rest as `CONTENT $body`. On SELECT we `OMIT id` from the projection
//!   so the result has only the domain fields, then rehydrate by injecting
//!   the caller's typed ID.
//! - **"Rich" types** (Grant, AuthRequest) whose domain shape doesn't map
//!   1-to-1 to the flat schema get explicit translation helpers
//!   (`principal_to_kind_id`, `grant_to_row`, etc.).
//! - **`DateTime<Utc>` bindings use the SurrealQL `<datetime>` cast** — the
//!   driver serializes `DateTime<Utc>` as an RFC3339 string which
//!   `TYPE datetime` columns reject without the explicit cast.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use domain::audit::{AuditClass, AuditEvent};
use domain::model::ids::{
    AgentId, AuditEventId, AuthRequestId, EdgeId, GrantId, NodeId, OrgId, ProjectId, UserId,
};
use domain::model::nodes::{
    Agent, AgentProfile, AuthRequest, Channel, Consent, Grant, InboxObject, Memory, Organization,
    OutboxObject, PrincipalRef, ResourceRef, Template, ToolAuthorityManifest, User,
};
use domain::repository::{
    BootstrapClaim, BootstrapCredentialRow, Repository, RepositoryError, RepositoryResult,
};

use crate::SurrealStore;

use base64::engine::general_purpose::STANDARD_NO_PAD as BASE64_NOPAD;
use base64::Engine as _;

// ============================================================================
// Error plumbing
// ============================================================================

fn backend<E: std::fmt::Display>(e: E) -> RepositoryError {
    RepositoryError::Backend(e.to_string())
}

// ============================================================================
// Translation helpers — PrincipalRef ↔ (kind, id) and ResourceRef ↔ uri
// ============================================================================

fn principal_to_kind_id(p: &PrincipalRef) -> (&'static str, String) {
    match p {
        PrincipalRef::Agent(id) => ("agent", id.to_string()),
        PrincipalRef::User(id) => ("user", id.to_string()),
        PrincipalRef::Organization(id) => ("organization", id.to_string()),
        PrincipalRef::Project(id) => ("project", id.to_string()),
        PrincipalRef::System(s) => ("system", s.clone()),
    }
}

fn principal_from_kind_id(kind: &str, id: &str) -> RepositoryResult<PrincipalRef> {
    match kind {
        "agent" => Ok(PrincipalRef::Agent(AgentId::from_uuid(parse_uuid(id)?))),
        "user" => Ok(PrincipalRef::User(UserId::from_uuid(parse_uuid(id)?))),
        "organization" => Ok(PrincipalRef::Organization(OrgId::from_uuid(parse_uuid(
            id,
        )?))),
        "project" => Ok(PrincipalRef::Project(ProjectId::from_uuid(parse_uuid(id)?))),
        "system" => Ok(PrincipalRef::System(id.to_string())),
        other => Err(RepositoryError::Backend(format!(
            "unknown principal kind in row: {other}"
        ))),
    }
}

fn parse_uuid(s: &str) -> RepositoryResult<Uuid> {
    Uuid::parse_str(s).map_err(|e| RepositoryError::Backend(format!("invalid uuid `{s}`: {e}")))
}

// ============================================================================
// JSON hydration — strip id on write, inject id on read
// ============================================================================

fn strip_id(mut v: serde_json::Value) -> serde_json::Value {
    if let serde_json::Value::Object(ref mut map) = v {
        map.remove("id");
    }
    v
}

fn inject_id<T: Serialize>(mut v: serde_json::Value, id: T) -> RepositoryResult<serde_json::Value> {
    let id_val = serde_json::to_value(id).map_err(backend)?;
    if let serde_json::Value::Object(ref mut map) = v {
        map.insert("id".to_string(), id_val);
    }
    Ok(v)
}

async fn take_first_row(
    resp: &mut surrealdb::Response,
    idx: usize,
) -> RepositoryResult<Option<serde_json::Value>> {
    let rows: Vec<serde_json::Value> = resp.take(idx).map_err(backend)?;
    Ok(rows.into_iter().next())
}

// ============================================================================
// Grant row translator
// ============================================================================

#[derive(Serialize, Deserialize)]
struct GrantRow {
    holder_kind: String,
    holder_id: String,
    action: Vec<String>,
    resource_uri: String,
    /// Explicit fundamentals — serialized as a `Vec<&'static str>`
    /// since [`Fundamental`] already serdes as a snake_case string.
    /// `#[serde(default)]` means existing rows (pre-M2/P4.5) deserialize
    /// with an empty vec, which preserves legacy URI-derivation in
    /// `resolve_grant` (see its Case C fallback).
    #[serde(default)]
    fundamentals: Vec<domain::model::Fundamental>,
    descends_from: Option<String>,
    delegable: bool,
    issued_at: DateTime<Utc>,
    revoked_at: Option<DateTime<Utc>>,
}

impl GrantRow {
    fn from_domain(g: &Grant) -> Self {
        let (holder_kind, holder_id) = principal_to_kind_id(&g.holder);
        GrantRow {
            holder_kind: holder_kind.to_string(),
            holder_id,
            action: g.action.clone(),
            resource_uri: g.resource.uri.clone(),
            fundamentals: g.fundamentals.clone(),
            descends_from: g.descends_from.map(|a| a.to_string()),
            delegable: g.delegable,
            issued_at: g.issued_at,
            revoked_at: g.revoked_at,
        }
    }

    fn into_domain(self, id: GrantId) -> RepositoryResult<Grant> {
        let holder = principal_from_kind_id(&self.holder_kind, &self.holder_id)?;
        let descends_from = self
            .descends_from
            .as_deref()
            .map(parse_uuid)
            .transpose()?
            .map(AuthRequestId::from_uuid);
        Ok(Grant {
            id,
            holder,
            action: self.action,
            resource: ResourceRef {
                uri: self.resource_uri,
            },
            fundamentals: self.fundamentals,
            descends_from,
            delegable: self.delegable,
            issued_at: self.issued_at,
            revoked_at: self.revoked_at,
        })
    }
}

// ============================================================================
// AuthRequest row translator — flattens `requestor: PrincipalRef` into
// `requestor_kind` + `requestor_id` to match the schema. `resource_slots`
// stays as a JSON `array<object>`.
// ============================================================================

#[derive(Serialize, Deserialize)]
struct AuthRequestRow {
    requestor_kind: String,
    requestor_id: String,
    kinds: Vec<String>,
    scope: Vec<String>,
    state: String,
    valid_until: Option<DateTime<Utc>>,
    submitted_at: DateTime<Utc>,
    resource_slots: serde_json::Value,
    justification: Option<String>,
    audit_class: String,
    terminal_state_entered_at: Option<DateTime<Utc>>,
    archived: bool,
    active_window_days: u32,
    provenance_template: Option<String>,
}

impl AuthRequestRow {
    fn from_domain(r: &AuthRequest) -> RepositoryResult<Self> {
        let (kind, id) = principal_to_kind_id(&r.requestor);
        Ok(AuthRequestRow {
            requestor_kind: kind.to_string(),
            requestor_id: id,
            kinds: r.kinds.clone(),
            scope: r.scope.clone(),
            state: auth_request_state_to_str(r.state).to_string(),
            valid_until: r.valid_until,
            submitted_at: r.submitted_at,
            resource_slots: serde_json::to_value(&r.resource_slots).map_err(backend)?,
            justification: r.justification.clone(),
            audit_class: audit_class_to_str(r.audit_class).to_string(),
            terminal_state_entered_at: r.terminal_state_entered_at,
            archived: r.archived,
            active_window_days: r.active_window_days,
            provenance_template: r.provenance_template.map(|t| t.to_string()),
        })
    }

    fn into_domain(self, id: AuthRequestId) -> RepositoryResult<AuthRequest> {
        let requestor = principal_from_kind_id(&self.requestor_kind, &self.requestor_id)?;
        let state = auth_request_state_from_str(&self.state)?;
        let audit_class = audit_class_from_str(&self.audit_class)?;
        let resource_slots = serde_json::from_value(self.resource_slots).map_err(backend)?;
        let provenance_template = self
            .provenance_template
            .as_deref()
            .map(parse_uuid)
            .transpose()?
            .map(domain::model::ids::TemplateId::from_uuid);
        Ok(AuthRequest {
            id,
            requestor,
            kinds: self.kinds,
            scope: self.scope,
            state,
            valid_until: self.valid_until,
            submitted_at: self.submitted_at,
            resource_slots,
            justification: self.justification,
            audit_class,
            terminal_state_entered_at: self.terminal_state_entered_at,
            archived: self.archived,
            active_window_days: self.active_window_days,
            provenance_template,
        })
    }
}

fn auth_request_state_to_str(s: domain::model::nodes::AuthRequestState) -> &'static str {
    use domain::model::nodes::AuthRequestState as S;
    match s {
        S::Draft => "draft",
        S::Pending => "pending",
        S::InProgress => "in_progress",
        S::Approved => "approved",
        S::Denied => "denied",
        S::Partial => "partial",
        S::Expired => "expired",
        S::Revoked => "revoked",
        S::Cancelled => "cancelled",
    }
}

fn auth_request_state_from_str(
    s: &str,
) -> RepositoryResult<domain::model::nodes::AuthRequestState> {
    use domain::model::nodes::AuthRequestState as S;
    Ok(match s {
        "draft" => S::Draft,
        "pending" => S::Pending,
        "in_progress" => S::InProgress,
        "approved" => S::Approved,
        "denied" => S::Denied,
        "partial" => S::Partial,
        "expired" => S::Expired,
        "revoked" => S::Revoked,
        "cancelled" => S::Cancelled,
        other => {
            return Err(RepositoryError::Backend(format!(
                "unknown auth_request state: {other}"
            )))
        }
    })
}

fn audit_class_to_str(c: domain::audit::AuditClass) -> &'static str {
    use domain::audit::AuditClass as C;
    match c {
        C::Silent => "silent",
        C::Logged => "logged",
        C::Alerted => "alerted",
    }
}

fn audit_class_from_str(s: &str) -> RepositoryResult<domain::audit::AuditClass> {
    use domain::audit::AuditClass as C;
    Ok(match s {
        "silent" => C::Silent,
        "logged" => C::Logged,
        "alerted" => C::Alerted,
        other => {
            return Err(RepositoryError::Backend(format!(
                "unknown audit_class: {other}"
            )))
        }
    })
}

// ============================================================================
// The Repository impl
// ============================================================================

#[async_trait]
impl Repository for SurrealStore {
    async fn ping(&self) -> RepositoryResult<()> {
        self.client().health().await.map_err(backend)
    }

    // ---- Node CRUD -------------------------------------------------------

    async fn create_agent(&self, agent: &Agent) -> RepositoryResult<()> {
        let body = strip_id(serde_json::to_value(agent).map_err(backend)?);
        self.client()
            .query(
                "CREATE type::thing('agent', $id) CONTENT $body \
                 RETURN NONE",
            )
            .bind(("id", agent.id.to_string()))
            .bind(("body", body))
            .await
            .map_err(backend)?
            .check()
            .map_err(backend)?;
        Ok(())
    }

    async fn get_agent(&self, id: AgentId) -> RepositoryResult<Option<Agent>> {
        let mut resp = self
            .client()
            .query("SELECT * OMIT id FROM type::thing('agent', $id)")
            .bind(("id", id.to_string()))
            .await
            .map_err(backend)?;
        let Some(row) = take_first_row(&mut resp, 0).await? else {
            return Ok(None);
        };
        let row = inject_id(row, id)?;
        Ok(Some(serde_json::from_value(row).map_err(backend)?))
    }

    async fn create_agent_profile(&self, profile: &AgentProfile) -> RepositoryResult<()> {
        let body = strip_id(serde_json::to_value(profile).map_err(backend)?);
        self.client()
            .query("CREATE type::thing('agent_profile', $id) CONTENT $body RETURN NONE")
            .bind(("id", profile.id.to_string()))
            .bind(("body", body))
            .await
            .map_err(backend)?
            .check()
            .map_err(backend)?;
        Ok(())
    }

    async fn create_user(&self, user: &User) -> RepositoryResult<()> {
        let body = strip_id(serde_json::to_value(user).map_err(backend)?);
        self.client()
            .query("CREATE type::thing('user', $id) CONTENT $body RETURN NONE")
            .bind(("id", user.id.to_string()))
            .bind(("body", body))
            .await
            .map_err(backend)?
            .check()
            .map_err(backend)?;
        Ok(())
    }

    async fn create_organization(&self, org: &Organization) -> RepositoryResult<()> {
        let body = strip_id(serde_json::to_value(org).map_err(backend)?);
        self.client()
            .query("CREATE type::thing('organization', $id) CONTENT $body RETURN NONE")
            .bind(("id", org.id.to_string()))
            .bind(("body", body))
            .await
            .map_err(backend)?
            .check()
            .map_err(backend)?;
        Ok(())
    }

    async fn get_organization(&self, id: OrgId) -> RepositoryResult<Option<Organization>> {
        let mut resp = self
            .client()
            .query("SELECT * OMIT id FROM type::thing('organization', $id)")
            .bind(("id", id.to_string()))
            .await
            .map_err(backend)?;
        let Some(row) = take_first_row(&mut resp, 0).await? else {
            return Ok(None);
        };
        let row = inject_id(row, id)?;
        Ok(Some(serde_json::from_value(row).map_err(backend)?))
    }

    async fn create_template(&self, template: &Template) -> RepositoryResult<()> {
        let body = strip_id(serde_json::to_value(template).map_err(backend)?);
        self.client()
            .query("CREATE type::thing('template', $id) CONTENT $body RETURN NONE")
            .bind(("id", template.id.to_string()))
            .bind(("body", body))
            .await
            .map_err(backend)?
            .check()
            .map_err(backend)?;
        Ok(())
    }

    async fn create_channel(&self, channel: &Channel) -> RepositoryResult<()> {
        let body = strip_id(serde_json::to_value(channel).map_err(backend)?);
        self.client()
            .query("CREATE type::thing('channel', $id) CONTENT $body RETURN NONE")
            .bind(("id", channel.id.to_string()))
            .bind(("body", body))
            .await
            .map_err(backend)?
            .check()
            .map_err(backend)?;
        Ok(())
    }

    async fn create_inbox(&self, inbox: &InboxObject) -> RepositoryResult<()> {
        let body = strip_id(serde_json::to_value(inbox).map_err(backend)?);
        self.client()
            .query("CREATE type::thing('inbox_object', $id) CONTENT $body RETURN NONE")
            .bind(("id", inbox.id.to_string()))
            .bind(("body", body))
            .await
            .map_err(backend)?
            .check()
            .map_err(backend)?;
        Ok(())
    }

    async fn create_outbox(&self, outbox: &OutboxObject) -> RepositoryResult<()> {
        let body = strip_id(serde_json::to_value(outbox).map_err(backend)?);
        self.client()
            .query("CREATE type::thing('outbox_object', $id) CONTENT $body RETURN NONE")
            .bind(("id", outbox.id.to_string()))
            .bind(("body", body))
            .await
            .map_err(backend)?
            .check()
            .map_err(backend)?;
        Ok(())
    }

    async fn create_memory(&self, memory: &Memory) -> RepositoryResult<()> {
        let body = strip_id(serde_json::to_value(memory).map_err(backend)?);
        self.client()
            .query("CREATE type::thing('memory', $id) CONTENT $body RETURN NONE")
            .bind(("id", memory.id.to_string()))
            .bind(("body", body))
            .await
            .map_err(backend)?
            .check()
            .map_err(backend)?;
        Ok(())
    }

    async fn create_consent(&self, consent: &Consent) -> RepositoryResult<()> {
        let body = strip_id(serde_json::to_value(consent).map_err(backend)?);
        self.client()
            .query("CREATE type::thing('consent', $id) CONTENT $body RETURN NONE")
            .bind(("id", consent.id.to_string()))
            .bind(("body", body))
            .await
            .map_err(backend)?
            .check()
            .map_err(backend)?;
        Ok(())
    }

    async fn create_tool_authority_manifest(
        &self,
        manifest: &ToolAuthorityManifest,
    ) -> RepositoryResult<()> {
        let body = strip_id(serde_json::to_value(manifest).map_err(backend)?);
        self.client()
            .query("CREATE type::thing('tool_authority_manifest', $id) CONTENT $body RETURN NONE")
            .bind(("id", manifest.id.to_string()))
            .bind(("body", body))
            .await
            .map_err(backend)?
            .check()
            .map_err(backend)?;
        Ok(())
    }

    async fn get_admin_agent(&self) -> RepositoryResult<Option<Agent>> {
        let mut resp = self
            .client()
            .query(
                "SELECT *, record::id(id) AS _rid OMIT id FROM agent WHERE kind = 'human' LIMIT 1",
            )
            .await
            .map_err(backend)?;
        let rows: Vec<serde_json::Value> = resp.take(0).map_err(backend)?;
        let Some(mut row) = rows.into_iter().next() else {
            return Ok(None);
        };
        // Extract the rid, which we wrote as a string via record::id, then
        // turn it into an AgentId and inject back as `id`.
        let rid = row
            .as_object_mut()
            .and_then(|m| m.remove("_rid"))
            .and_then(|v| v.as_str().map(str::to_string))
            .ok_or_else(|| RepositoryError::Backend("admin row missing _rid".into()))?;
        let agent_id = AgentId::from_uuid(parse_uuid(&rid)?);
        let row = inject_id(row, agent_id)?;
        Ok(Some(serde_json::from_value(row).map_err(backend)?))
    }

    // ---- Grants ----------------------------------------------------------

    async fn create_grant(&self, grant: &Grant) -> RepositoryResult<()> {
        let row = GrantRow::from_domain(grant);
        let body = serde_json::to_value(&row).map_err(backend)?;
        self.client()
            .query("CREATE type::thing('grant', $id) CONTENT $body RETURN NONE")
            .bind(("id", grant.id.to_string()))
            .bind(("body", body))
            .await
            .map_err(backend)?
            .check()
            .map_err(backend)?;
        Ok(())
    }

    async fn get_grant(&self, id: GrantId) -> RepositoryResult<Option<Grant>> {
        let mut resp = self
            .client()
            .query("SELECT * OMIT id FROM type::thing('grant', $id)")
            .bind(("id", id.to_string()))
            .await
            .map_err(backend)?;
        let rows: Vec<GrantRow> = resp.take(0).map_err(backend)?;
        let Some(row) = rows.into_iter().next() else {
            return Ok(None);
        };
        Ok(Some(row.into_domain(id)?))
    }

    async fn revoke_grant(&self, id: GrantId, revoked_at: DateTime<Utc>) -> RepositoryResult<()> {
        self.client()
            .query(
                "UPDATE type::thing('grant', $id) \
                 SET revoked_at = <datetime> $at \
                 RETURN NONE",
            )
            .bind(("id", id.to_string()))
            .bind(("at", revoked_at.to_rfc3339()))
            .await
            .map_err(backend)?
            .check()
            .map_err(backend)?;
        Ok(())
    }

    async fn list_grants_for_principal(
        &self,
        principal: &PrincipalRef,
    ) -> RepositoryResult<Vec<Grant>> {
        let (kind, id) = principal_to_kind_id(principal);
        let mut resp = self
            .client()
            .query(
                "SELECT *, record::id(id) AS _rid OMIT id \
                 FROM grant \
                 WHERE holder_kind = $kind AND holder_id = $id",
            )
            .bind(("kind", kind.to_string()))
            .bind(("id", id))
            .await
            .map_err(backend)?;
        let rows: Vec<serde_json::Value> = resp.take(0).map_err(backend)?;
        let mut out = Vec::with_capacity(rows.len());
        for mut row in rows {
            let rid = row
                .as_object_mut()
                .and_then(|m| m.remove("_rid"))
                .and_then(|v| v.as_str().map(str::to_string))
                .ok_or_else(|| RepositoryError::Backend("grant row missing _rid".into()))?;
            let grant_id = GrantId::from_uuid(parse_uuid(&rid)?);
            let grow: GrantRow = serde_json::from_value(row).map_err(backend)?;
            out.push(grow.into_domain(grant_id)?);
        }
        Ok(out)
    }

    // ---- Auth Requests ---------------------------------------------------

    async fn create_auth_request(&self, req: &AuthRequest) -> RepositoryResult<()> {
        let row = AuthRequestRow::from_domain(req)?;
        let body = serde_json::to_value(&row).map_err(backend)?;
        self.client()
            .query("CREATE type::thing('auth_request', $id) CONTENT $body RETURN NONE")
            .bind(("id", req.id.to_string()))
            .bind(("body", body))
            .await
            .map_err(backend)?
            .check()
            .map_err(backend)?;
        Ok(())
    }

    async fn get_auth_request(&self, id: AuthRequestId) -> RepositoryResult<Option<AuthRequest>> {
        let mut resp = self
            .client()
            .query("SELECT * OMIT id FROM type::thing('auth_request', $id)")
            .bind(("id", id.to_string()))
            .await
            .map_err(backend)?;
        let rows: Vec<AuthRequestRow> = resp.take(0).map_err(backend)?;
        let Some(row) = rows.into_iter().next() else {
            return Ok(None);
        };
        Ok(Some(row.into_domain(id)?))
    }

    async fn update_auth_request(&self, req: &AuthRequest) -> RepositoryResult<()> {
        let row = AuthRequestRow::from_domain(req)?;
        let body = serde_json::to_value(&row).map_err(backend)?;
        self.client()
            .query("UPDATE type::thing('auth_request', $id) CONTENT $body RETURN NONE")
            .bind(("id", req.id.to_string()))
            .bind(("body", body))
            .await
            .map_err(backend)?
            .check()
            .map_err(backend)?;
        Ok(())
    }

    async fn list_active_auth_requests_for_resource(
        &self,
        resource: &ResourceRef,
    ) -> RepositoryResult<Vec<AuthRequest>> {
        // v0 impl: select non-archived requests whose resource_slots contains
        // a slot with a matching resource uri. Uses SurrealDB's `$this`-style
        // nested filtering.
        let mut resp = self
            .client()
            .query(
                "SELECT *, record::id(id) AS _rid OMIT id \
                 FROM auth_request \
                 WHERE archived = false \
                   AND resource_slots[WHERE resource.uri = $uri] != []",
            )
            .bind(("uri", resource.uri.clone()))
            .await
            .map_err(backend)?;
        let rows: Vec<serde_json::Value> = resp.take(0).map_err(backend)?;
        let mut out = Vec::with_capacity(rows.len());
        for mut row in rows {
            let rid = row
                .as_object_mut()
                .and_then(|m| m.remove("_rid"))
                .and_then(|v| v.as_str().map(str::to_string))
                .ok_or_else(|| RepositoryError::Backend("auth_request row missing _rid".into()))?;
            let arid = AuthRequestId::from_uuid(parse_uuid(&rid)?);
            let arow: AuthRequestRow = serde_json::from_value(row).map_err(backend)?;
            out.push(arow.into_domain(arid)?);
        }
        Ok(out)
    }

    // ---- Ownership edges (raw) ------------------------------------------

    async fn upsert_ownership_raw(
        &self,
        resource_id: NodeId,
        owner_id: NodeId,
        auth_request: Option<AuthRequestId>,
    ) -> RepositoryResult<EdgeId> {
        let edge_id = EdgeId::new();
        // The three ownership relations have no typed FROM/TO in the schema
        // (ADR-0015). We use a neutral 'node' table prefix so the source and
        // target can be any of the 37 node tables. SurrealDB's RELATE parser
        // does not accept `type::thing(...)` directly in the FROM/TO slots,
        // so we bind the record refs via `LET` statements first.
        self.client()
            .query(
                "LET $f = type::thing('node', $from); \
                 LET $t = type::thing('node', $to); \
                 RELATE $f -> owned_by -> $t \
                    SET id = type::thing('owned_by', $edge), \
                        auth_request = $auth_request \
                    RETURN NONE",
            )
            .bind(("from", resource_id.to_string()))
            .bind(("to", owner_id.to_string()))
            .bind(("edge", edge_id.to_string()))
            .bind(("auth_request", auth_request.map(|a| a.to_string())))
            .await
            .map_err(backend)?
            .check()
            .map_err(backend)?;
        Ok(edge_id)
    }

    async fn upsert_creation_raw(
        &self,
        creator_id: NodeId,
        resource_id: NodeId,
    ) -> RepositoryResult<EdgeId> {
        let edge_id = EdgeId::new();
        self.client()
            .query(
                "LET $f = type::thing('node', $from); \
                 LET $t = type::thing('node', $to); \
                 RELATE $f -> created -> $t \
                    SET id = type::thing('created', $edge) \
                    RETURN NONE",
            )
            .bind(("from", creator_id.to_string()))
            .bind(("to", resource_id.to_string()))
            .bind(("edge", edge_id.to_string()))
            .await
            .map_err(backend)?
            .check()
            .map_err(backend)?;
        Ok(edge_id)
    }

    async fn upsert_allocation_raw(
        &self,
        from_id: NodeId,
        to_id: NodeId,
        resource: &ResourceRef,
        auth_request: AuthRequestId,
    ) -> RepositoryResult<EdgeId> {
        let edge_id = EdgeId::new();
        self.client()
            .query(
                "LET $f = type::thing('node', $from); \
                 LET $t = type::thing('node', $to); \
                 RELATE $f -> allocated_to -> $t \
                    SET id = type::thing('allocated_to', $edge), \
                        resource_uri = $uri, \
                        auth_request = $auth_request \
                    RETURN NONE",
            )
            .bind(("from", from_id.to_string()))
            .bind(("to", to_id.to_string()))
            .bind(("edge", edge_id.to_string()))
            .bind(("uri", resource.uri.clone()))
            .bind(("auth_request", auth_request.to_string()))
            .await
            .map_err(backend)?
            .check()
            .map_err(backend)?;
        Ok(edge_id)
    }

    // ---- Bootstrap credentials -------------------------------------------

    async fn put_bootstrap_credential(
        &self,
        digest: String,
    ) -> RepositoryResult<BootstrapCredentialRow> {
        let mut resp = self
            .client()
            .query(
                "CREATE bootstrap_credentials SET digest = $digest, \
                 created_at = $created_at, consumed_at = NONE \
                 RETURN record::id(id) AS _rid, digest, created_at, consumed_at",
            )
            .bind(("digest", digest))
            .bind(("created_at", chrono::Utc::now().to_rfc3339()))
            .await
            .map_err(backend)?;
        let rows: Vec<BootstrapRow> = resp.take(0).map_err(backend)?;
        let row = rows
            .into_iter()
            .next()
            .ok_or_else(|| RepositoryError::Backend("CREATE returned no rows".into()))?;
        Ok(row.into_domain())
    }

    async fn find_unconsumed_credential(
        &self,
        digest: &str,
    ) -> RepositoryResult<Option<BootstrapCredentialRow>> {
        let mut resp = self
            .client()
            .query(
                "SELECT record::id(id) AS _rid, digest, created_at, consumed_at \
                 FROM bootstrap_credentials \
                 WHERE digest = $digest AND consumed_at IS NONE LIMIT 1",
            )
            .bind(("digest", digest.to_string()))
            .await
            .map_err(backend)?;
        let rows: Vec<BootstrapRow> = resp.take(0).map_err(backend)?;
        Ok(rows.into_iter().next().map(BootstrapRow::into_domain))
    }

    async fn consume_bootstrap_credential(&self, record_id: &str) -> RepositoryResult<()> {
        self.client()
            .query(
                "UPDATE type::thing('bootstrap_credentials', $id) \
                 SET consumed_at = $at RETURN NONE",
            )
            .bind(("id", record_id.to_string()))
            .bind(("at", chrono::Utc::now().to_rfc3339()))
            .await
            .map_err(backend)?
            .check()
            .map_err(backend)?;
        Ok(())
    }

    async fn list_bootstrap_credentials(
        &self,
        unconsumed_only: bool,
    ) -> RepositoryResult<Vec<BootstrapCredentialRow>> {
        let q = if unconsumed_only {
            "SELECT record::id(id) AS _rid, digest, created_at, consumed_at \
             FROM bootstrap_credentials WHERE consumed_at IS NONE"
        } else {
            "SELECT record::id(id) AS _rid, digest, created_at, consumed_at \
             FROM bootstrap_credentials"
        };
        let mut resp = self.client().query(q).await.map_err(backend)?;
        let rows: Vec<BootstrapRow> = resp.take(0).map_err(backend)?;
        Ok(rows.into_iter().map(BootstrapRow::into_domain).collect())
    }

    // ---- Resources Catalogue --------------------------------------------

    async fn seed_catalogue_entry(
        &self,
        owning_org: Option<OrgId>,
        resource_uri: &str,
        kind: &str,
    ) -> RepositoryResult<()> {
        self.client()
            .query(
                "CREATE resources_catalogue SET \
                 owning_org = $org, resource_uri = $uri, kind = $kind, \
                 added_at = $added_at RETURN NONE",
            )
            .bind(("org", owning_org.map(|o| o.to_string())))
            .bind(("uri", resource_uri.to_string()))
            .bind(("kind", kind.to_string()))
            .bind(("added_at", chrono::Utc::now().to_rfc3339()))
            .await
            .map_err(backend)?
            .check()
            .map_err(backend)?;
        Ok(())
    }

    async fn catalogue_contains(
        &self,
        owning_org: Option<OrgId>,
        resource_uri: &str,
    ) -> RepositoryResult<bool> {
        let mut resp = self
            .client()
            .query(
                "SELECT count() FROM resources_catalogue \
                 WHERE owning_org = $org AND resource_uri = $uri GROUP ALL",
            )
            .bind(("org", owning_org.map(|o| o.to_string())))
            .bind(("uri", resource_uri.to_string()))
            .await
            .map_err(backend)?;
        let counts: Vec<i64> = resp.take((0, "count")).map_err(backend)?;
        Ok(counts.into_iter().next().unwrap_or(0) > 0)
    }

    // ---- Audit ----------------------------------------------------------

    async fn write_audit_event(&self, event: &AuditEvent) -> RepositoryResult<()> {
        let row = AuditEventRow::from_domain(event);
        let body = serde_json::to_value(&row).map_err(backend)?;
        self.client()
            .query("CREATE type::thing('audit_events', $id) CONTENT $body RETURN NONE")
            .bind(("id", event.event_id.to_string()))
            .bind(("body", body))
            .await
            .map_err(backend)?
            .check()
            .map_err(backend)?;
        Ok(())
    }

    async fn get_audit_event(&self, id: AuditEventId) -> RepositoryResult<Option<AuditEvent>> {
        let mut resp = self
            .client()
            .query("SELECT * OMIT id FROM type::thing('audit_events', $id)")
            .bind(("id", id.to_string()))
            .await
            .map_err(backend)?;
        let rows: Vec<AuditEventRow> = resp.take(0).map_err(backend)?;
        match rows.into_iter().next() {
            Some(row) => Ok(Some(row.into_domain(id)?)),
            None => Ok(None),
        }
    }

    async fn apply_bootstrap_claim(&self, claim: &BootstrapClaim) -> RepositoryResult<()> {
        // All s01 writes happen inside a single SurrealDB transaction. If
        // any query errors, SurrealDB rolls the whole batch back — the
        // credential stays unconsumed and the admin may retry.
        //
        // We serialize each entity once, up-front, so a serde error surfaces
        // before we open the transaction.
        let agent_body = strip_id(serde_json::to_value(&claim.human_agent).map_err(backend)?);
        let channel_body = strip_id(serde_json::to_value(&claim.channel).map_err(backend)?);
        let inbox_body = strip_id(serde_json::to_value(&claim.inbox).map_err(backend)?);
        let outbox_body = strip_id(serde_json::to_value(&claim.outbox).map_err(backend)?);
        let auth_request_body =
            serde_json::to_value(AuthRequestRow::from_domain(&claim.auth_request)?)
                .map_err(backend)?;
        let grant_body =
            serde_json::to_value(GrantRow::from_domain(&claim.grant)).map_err(backend)?;
        let audit_body = serde_json::to_value(AuditEventRow::from_domain(&claim.audit_event))
            .map_err(backend)?;

        // Catalogue entries — we emit one CREATE per entry inside the
        // transaction. SurrealDB 2.x permits multiple statements separated
        // by `;` in a single `query(...)` call.
        let mut q = String::from(
            "BEGIN TRANSACTION;\n\
             CREATE type::thing('agent', $agent_id) CONTENT $agent_body RETURN NONE;\n\
             CREATE type::thing('channel', $channel_id) CONTENT $channel_body RETURN NONE;\n\
             CREATE type::thing('inbox_object', $inbox_id) CONTENT $inbox_body RETURN NONE;\n\
             CREATE type::thing('outbox_object', $outbox_id) CONTENT $outbox_body RETURN NONE;\n\
             CREATE type::thing('auth_request', $ar_id) CONTENT $auth_request_body RETURN NONE;\n\
             CREATE type::thing('grant', $grant_id) CONTENT $grant_body RETURN NONE;\n\
             CREATE type::thing('audit_events', $audit_id) CONTENT $audit_body RETURN NONE;\n",
        );
        // One CREATE per catalogue entry.
        for i in 0..claim.catalogue_entries.len() {
            q.push_str(&format!(
                "CREATE resources_catalogue SET owning_org = NONE, \
                 resource_uri = $cat_uri_{i}, kind = $cat_kind_{i}, \
                 added_at = $now RETURN NONE;\n"
            ));
        }
        q.push_str(
            "UPDATE type::thing('bootstrap_credentials', $cred_record_id) \
             SET consumed_at = $now RETURN NONE;\n\
             COMMIT TRANSACTION;",
        );

        let now = chrono::Utc::now().to_rfc3339();
        let channel_id = claim.channel.id.to_string();
        let inbox_id = claim.inbox.id.to_string();
        let outbox_id = claim.outbox.id.to_string();

        let mut binder = self
            .client()
            .query(q)
            .bind(("agent_id", claim.human_agent.id.to_string()))
            .bind(("agent_body", agent_body))
            .bind(("channel_id", channel_id))
            .bind(("channel_body", channel_body))
            .bind(("inbox_id", inbox_id))
            .bind(("inbox_body", inbox_body))
            .bind(("outbox_id", outbox_id))
            .bind(("outbox_body", outbox_body))
            .bind(("ar_id", claim.auth_request.id.to_string()))
            .bind(("auth_request_body", auth_request_body))
            .bind(("grant_id", claim.grant.id.to_string()))
            .bind(("grant_body", grant_body))
            .bind(("audit_id", claim.audit_event.event_id.to_string()))
            .bind(("audit_body", audit_body))
            .bind(("cred_record_id", claim.credential_record_id.clone()))
            .bind(("now", now));
        for (i, (uri, kind)) in claim.catalogue_entries.iter().enumerate() {
            binder = binder
                .bind((format!("cat_uri_{i}"), uri.clone()))
                .bind((format!("cat_kind_{i}"), kind.clone()));
        }
        binder.await.map_err(backend)?.check().map_err(backend)?;
        Ok(())
    }

    async fn last_event_hash_for_org(
        &self,
        org: Option<OrgId>,
    ) -> RepositoryResult<Option<[u8; 32]>> {
        // Read the full last event within `org_scope` and hash its
        // canonical bytes. The next event's `prev_event_hash` copies
        // this digest — that's what makes the chain a chain:
        //
        //   event_n.prev_event_hash = hash_event(event_{n-1})
        //
        // Returning the stored `prev_event_hash_b64` column would
        // propagate the SECOND-to-last event's hash, not the last —
        // `prev_event_hash(n) = prev_event_hash(n-1)` is a constant,
        // not a chain. We pull the full row + id so we can rebuild the
        // domain struct exactly and call `hash_event`.
        let mut resp = self
            .client()
            .query(
                "SELECT *, record::id(id) AS __id OMIT id FROM audit_events \
                 WHERE org_scope = $org ORDER BY timestamp DESC LIMIT 1",
            )
            .bind(("org", org.map(|o| o.to_string())))
            .await
            .map_err(backend)?;
        let raw: Vec<serde_json::Value> = resp.take(0).map_err(backend)?;
        let Some(mut row) = raw.into_iter().next() else {
            return Ok(None);
        };
        let id_str = row
            .get("__id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| RepositoryError::Backend("last audit event missing __id".into()))?
            .to_string();
        if let serde_json::Value::Object(ref mut map) = row {
            map.remove("__id");
        }
        let parsed: AuditEventRow = serde_json::from_value(row).map_err(backend)?;
        let id = AuditEventId::from_uuid(parse_uuid(&id_str)?);
        let event = parsed.into_domain(id)?;
        Ok(Some(domain::audit::hash_event(&event)))
    }

    // ================================================================
    // M2 trait delegations — bodies live in `repo_impl_m2.rs` to keep
    // this file bounded. See that module for row translators + the
    // full SurrealQL for each surface.
    // ================================================================

    async fn put_secret(
        &self,
        credential: &domain::model::SecretCredential,
        sealed: &domain::repository::SealedBlob,
    ) -> RepositoryResult<()> {
        self.m2_put_secret(credential, sealed).await
    }

    async fn get_secret_by_slug(
        &self,
        slug: &domain::model::SecretRef,
    ) -> RepositoryResult<
        Option<(
            domain::model::SecretCredential,
            domain::repository::SealedBlob,
        )>,
    > {
        self.m2_get_secret_by_slug(slug).await
    }

    async fn list_secrets(&self) -> RepositoryResult<Vec<domain::model::SecretCredential>> {
        self.m2_list_secrets().await
    }

    async fn rotate_secret(
        &self,
        id: domain::model::ids::SecretId,
        new_sealed: &domain::repository::SealedBlob,
        at: DateTime<Utc>,
    ) -> RepositoryResult<()> {
        self.m2_rotate_secret(id, new_sealed, at).await
    }

    async fn reassign_secret_custodian(
        &self,
        id: domain::model::ids::SecretId,
        new_custodian: AgentId,
    ) -> RepositoryResult<()> {
        self.m2_reassign_secret_custodian(id, new_custodian).await
    }

    async fn put_model_provider(
        &self,
        provider: &domain::model::ModelRuntime,
    ) -> RepositoryResult<()> {
        self.m2_put_model_provider(provider).await
    }

    async fn list_model_providers(
        &self,
        include_archived: bool,
    ) -> RepositoryResult<Vec<domain::model::ModelRuntime>> {
        self.m2_list_model_providers(include_archived).await
    }

    async fn archive_model_provider(
        &self,
        id: domain::model::ids::ModelProviderId,
        at: DateTime<Utc>,
    ) -> RepositoryResult<()> {
        self.m2_archive_model_provider(id, at).await
    }

    async fn put_mcp_server(
        &self,
        server: &domain::model::ExternalService,
    ) -> RepositoryResult<()> {
        self.m2_put_mcp_server(server).await
    }

    async fn list_mcp_servers(
        &self,
        include_archived: bool,
    ) -> RepositoryResult<Vec<domain::model::ExternalService>> {
        self.m2_list_mcp_servers(include_archived).await
    }

    async fn patch_mcp_tenants(
        &self,
        id: domain::model::ids::McpServerId,
        new_allowed: &domain::model::TenantSet,
    ) -> RepositoryResult<()> {
        self.m2_patch_mcp_tenants(id, new_allowed).await
    }

    async fn archive_mcp_server(
        &self,
        id: domain::model::ids::McpServerId,
        at: DateTime<Utc>,
    ) -> RepositoryResult<()> {
        self.m2_archive_mcp_server(id, at).await
    }

    async fn get_platform_defaults(
        &self,
    ) -> RepositoryResult<Option<domain::model::PlatformDefaults>> {
        self.m2_get_platform_defaults().await
    }

    async fn put_platform_defaults(
        &self,
        defaults: &domain::model::PlatformDefaults,
    ) -> RepositoryResult<()> {
        self.m2_put_platform_defaults(defaults).await
    }

    async fn narrow_mcp_tenants(
        &self,
        id: domain::model::ids::McpServerId,
        new_allowed: &domain::model::TenantSet,
        at: DateTime<Utc>,
    ) -> RepositoryResult<Vec<domain::repository::TenantRevocation>> {
        self.m2_narrow_mcp_tenants(id, new_allowed, at).await
    }

    async fn revoke_grants_by_descends_from(
        &self,
        ar: domain::model::ids::AuthRequestId,
        at: DateTime<Utc>,
    ) -> RepositoryResult<Vec<GrantId>> {
        self.m2_revoke_grants_by_descends_from(ar, at).await
    }

    async fn seed_catalogue_entry_for_composite(
        &self,
        owning_org: Option<OrgId>,
        resource_uri: &str,
        composite: domain::model::Composite,
    ) -> RepositoryResult<()> {
        self.m2_seed_catalogue_entry_for_composite(owning_org, resource_uri, composite)
            .await
    }
}

// ============================================================================
// Internal row types — only reachable via this module
// ============================================================================

#[derive(Deserialize)]
struct BootstrapRow {
    _rid: String,
    digest: String,
    created_at: DateTime<Utc>,
    consumed_at: Option<DateTime<Utc>>,
}

impl BootstrapRow {
    fn into_domain(self) -> BootstrapCredentialRow {
        BootstrapCredentialRow {
            record_id: self._rid,
            digest: self.digest,
            created_at: self.created_at,
            consumed_at: self.consumed_at,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct AuditEventRow {
    event_type: String,
    actor_agent_id: Option<String>,
    target_entity_id: Option<String>,
    timestamp: DateTime<Utc>,
    diff: serde_json::Value,
    audit_class: String,
    provenance_auth_request_id: Option<String>,
    org_scope: Option<String>,
    prev_event_hash_b64: Option<String>,
}

impl AuditEventRow {
    fn from_domain(ev: &AuditEvent) -> Self {
        AuditEventRow {
            event_type: ev.event_type.clone(),
            actor_agent_id: ev.actor_agent_id.map(|a| a.to_string()),
            target_entity_id: ev.target_entity_id.map(|n| n.to_string()),
            timestamp: ev.timestamp,
            diff: ev.diff.clone(),
            audit_class: audit_class_to_str(ev.audit_class).to_string(),
            provenance_auth_request_id: ev.provenance_auth_request_id.map(|a| a.to_string()),
            org_scope: ev.org_scope.map(|o| o.to_string()),
            prev_event_hash_b64: ev.prev_event_hash.map(|h| BASE64_NOPAD.encode(h)),
        }
    }

    fn into_domain(self, event_id: AuditEventId) -> RepositoryResult<AuditEvent> {
        let actor_agent_id = match self.actor_agent_id {
            Some(s) => Some(AgentId::from_uuid(
                s.parse::<uuid::Uuid>().map_err(backend)?,
            )),
            None => None,
        };
        let target_entity_id = match self.target_entity_id {
            Some(s) => Some(NodeId::from_uuid(s.parse::<uuid::Uuid>().map_err(backend)?)),
            None => None,
        };
        let provenance_auth_request_id = match self.provenance_auth_request_id {
            Some(s) => Some(AuthRequestId::from_uuid(
                s.parse::<uuid::Uuid>().map_err(backend)?,
            )),
            None => None,
        };
        let org_scope = match self.org_scope {
            Some(s) => Some(OrgId::from_uuid(s.parse::<uuid::Uuid>().map_err(backend)?)),
            None => None,
        };
        let prev_event_hash = match self.prev_event_hash_b64 {
            Some(b64) => {
                let bytes = BASE64_NOPAD.decode(&b64).map_err(backend)?;
                if bytes.len() != 32 {
                    return Err(RepositoryError::Backend(format!(
                        "prev_event_hash_b64 decoded length != 32 ({} bytes)",
                        bytes.len()
                    )));
                }
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                Some(arr)
            }
            None => None,
        };
        let audit_class = match self.audit_class.as_str() {
            "silent" => AuditClass::Silent,
            "logged" => AuditClass::Logged,
            "alerted" => AuditClass::Alerted,
            other => {
                return Err(RepositoryError::Backend(format!(
                    "unknown audit_class string: {other}"
                )))
            }
        };
        Ok(AuditEvent {
            event_id,
            event_type: self.event_type,
            actor_agent_id,
            target_entity_id,
            timestamp: self.timestamp,
            diff: self.diff,
            audit_class,
            provenance_auth_request_id,
            org_scope,
            prev_event_hash,
        })
    }
}
