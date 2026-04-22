//! Audit-event builders for pages 06 (org creation) + 07 (org dashboard).
//!
//! Two event types ship with M3/P2:
//!
//! - `platform.organization.created` (Alerted) — a new org was just
//!   provisioned via the M3/P4 wizard compound transaction. Diff
//!   carries the org identity + the adopted-template summary + the
//!   CEO agent id + system agent ids so a reviewer can reconstruct
//!   the whole compound write from a single event.
//! - `authority_template.adopted` (Alerted) — per-template companion
//!   event emitted alongside `organization.created`. Each enabled
//!   template (A / B / C / D) fires one. Allows the M3/P5 dashboard's
//!   `AdoptedTemplates` panel to filter the audit log by
//!   `event_type = "authority_template.adopted"` without parsing the
//!   creation event's diff.
//!
//! ## Org-scope semantics
//!
//! Both events carry `org_scope = Some(org_id)` — the new org's id.
//! This is the **first** event in that org's hash chain. Subsequent
//! M3+ writes (future pages, M5 session launches, etc.) continue the
//! same per-org chain via `SurrealAuditEmitter`'s
//! `last_event_hash_for_org(Some(org_id))` lookup.
//!
//! ## phi-core leverage
//!
//! None — audit events are baby-phi's governance write log (see
//! `baby-phi/CLAUDE.md` §Orthogonal surfaces). `phi_core::AgentEvent`
//! is an agent-loop telemetry stream, not a hash-chained audit trail
//! — conflating the two is an orthogonal-surface mistake.

use chrono::{DateTime, Utc};

use crate::audit::{AuditClass, AuditEvent};
use crate::model::ids::{AgentId, AuditEventId, AuthRequestId, NodeId, OrgId};
use crate::model::nodes::{Organization, TemplateKind};

// ---------------------------------------------------------------------------
// Shared scaffolding
// ---------------------------------------------------------------------------

/// Every org-scoped builder emits the same audit-class default
/// (`Alerted`) because these events mark the creation of a
/// governance aggregate — losing one to Silent / Logged would break
/// compliance audit trails.
fn scaffold_org(
    event_type: &str,
    actor: AgentId,
    target: NodeId,
    org: OrgId,
    timestamp: DateTime<Utc>,
    diff: serde_json::Value,
    provenance_auth_request_id: Option<AuthRequestId>,
) -> AuditEvent {
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: event_type.to_string(),
        actor_agent_id: Some(actor),
        target_entity_id: Some(target),
        timestamp,
        diff,
        audit_class: AuditClass::Alerted,
        provenance_auth_request_id,
        // M3 is the first milestone to chain under Some(org_id).
        org_scope: Some(org),
        prev_event_hash: None,
    }
}

// ---------------------------------------------------------------------------
// Builders
// ---------------------------------------------------------------------------

/// `platform.organization.created` — Alerted.
///
/// Emitted by the M3/P4 org-creation handler immediately after
/// `Repository::apply_org_creation` commits. Carries a full
/// snapshot of the new org's governance surface: identity, consent
/// policy, audit class default, enabled templates, CEO agent id,
/// provisioned system agent ids. The M3/P5 dashboard reads this
/// event's diff to populate the `OrgHeader` + `AdoptedTemplates`
/// panels without reloading the Organization row.
pub fn organization_created(
    actor: AgentId,
    org: &Organization,
    ceo_agent_id: AgentId,
    provenance_auth_request_id: Option<AuthRequestId>,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    // Build the snapshot manually (rather than serialising the entire
    // Organization) because the embedded `defaults_snapshot` carries
    // phi-core wraps that are too verbose for the dashboard's
    // summary view. Individual phi-core fields are available via
    // `Repository::get_organization(id).defaults_snapshot` when a
    // reviewer needs them; the audit diff stays scannable.
    let diff = serde_json::json!({
        "before": serde_json::Value::Null,
        "after": {
            "org_id":                 org.id.to_string(),
            "display_name":           org.display_name,
            "vision":                 org.vision,
            "mission":                org.mission,
            "consent_policy":         org.consent_policy,
            "audit_class_default":    org.audit_class_default,
            "authority_templates_enabled":
                org.authority_templates_enabled
                    .iter()
                    .map(|k| k.as_str())
                    .collect::<Vec<_>>(),
            "ceo_agent_id":           ceo_agent_id.to_string(),
            "system_agents":          org.system_agents
                                        .iter()
                                        .map(|a| a.to_string())
                                        .collect::<Vec<_>>(),
            "default_model_provider": org.default_model_provider.map(|p| p.to_string()),
            "created_at":             org.created_at,
        },
    });
    scaffold_org(
        "platform.organization.created",
        actor,
        NodeId::from_uuid(*org.id.as_uuid()),
        org.id,
        timestamp,
        diff,
        provenance_auth_request_id,
    )
}

/// `authority_template.adopted` — Alerted, per-template companion
/// event.
///
/// Emitted once per adopted template (subset of A / B / C / D) in
/// the same `emit_audit_batch` as `organization.created`. The
/// `adoption_auth_request_id` is the id of the Template-E-shaped
/// adoption AR minted by [`crate::templates::adoption::build_adoption_request`].
pub fn authority_template_adopted(
    actor: AgentId,
    org_id: OrgId,
    template_kind: TemplateKind,
    adoption_auth_request_id: AuthRequestId,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    let diff = serde_json::json!({
        "before": serde_json::Value::Null,
        "after": {
            "org_id":                    org_id.to_string(),
            "template_kind":             template_kind.as_str(),
            "adoption_auth_request_id":  adoption_auth_request_id.to_string(),
        },
    });
    scaffold_org(
        "authority_template.adopted",
        actor,
        NodeId::from_uuid(*adoption_auth_request_id.as_uuid()),
        org_id,
        timestamp,
        diff,
        Some(adoption_auth_request_id),
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::AuditClass;
    use crate::model::composites_m3::ConsentPolicy;
    use crate::model::ids::OrgId;

    fn sample_org() -> Organization {
        Organization {
            id: OrgId::new(),
            display_name: "Acme Research".to_string(),
            vision: Some("Memory-first autonomous research".to_string()),
            mission: Some("Automate literature review".to_string()),
            consent_policy: ConsentPolicy::OneTime,
            audit_class_default: AuditClass::Logged,
            authority_templates_enabled: vec![TemplateKind::A, TemplateKind::B],
            defaults_snapshot: None,
            default_model_provider: None,
            system_agents: vec![AgentId::new(), AgentId::new()],
            created_at: Utc::now(),
        }
    }

    #[test]
    fn organization_created_is_alerted_and_org_scoped() {
        let org = sample_org();
        let actor = AgentId::new();
        let ceo = AgentId::new();
        let ev = organization_created(actor, &org, ceo, None, Utc::now());
        assert_eq!(ev.event_type, "platform.organization.created");
        assert_eq!(ev.audit_class, AuditClass::Alerted);
        // The org-scope hash chain invariant: M3 events open a new
        // per-org chain, not the platform root chain M2 writes use.
        assert_eq!(ev.org_scope, Some(org.id));
        assert_eq!(ev.actor_agent_id, Some(actor));
    }

    #[test]
    fn organization_created_diff_includes_template_kinds_as_wire_strings() {
        let org = sample_org();
        let ev = organization_created(AgentId::new(), &org, AgentId::new(), None, Utc::now());
        let kinds = ev.diff["after"]["authority_templates_enabled"]
            .as_array()
            .expect("templates_enabled serialises as an array");
        // `TemplateKind::as_str()` — serde-stable wire names
        // (`"a"`/`"b"` etc.). The dashboard relies on this for
        // filter-by-kind.
        let kind_strs: Vec<&str> = kinds.iter().filter_map(|v| v.as_str()).collect();
        assert_eq!(kind_strs, vec!["a", "b"]);
    }

    #[test]
    fn organization_created_captures_ceo_and_system_agents() {
        let org = sample_org();
        let ceo = AgentId::new();
        let ev = organization_created(AgentId::new(), &org, ceo, None, Utc::now());
        assert_eq!(ev.diff["after"]["ceo_agent_id"], ceo.to_string());
        let sa = ev.diff["after"]["system_agents"]
            .as_array()
            .expect("system_agents is an array");
        assert_eq!(sa.len(), 2);
    }

    #[test]
    fn authority_template_adopted_shape_and_class() {
        let org_id = OrgId::new();
        let ar_id = AuthRequestId::new();
        let ev =
            authority_template_adopted(AgentId::new(), org_id, TemplateKind::C, ar_id, Utc::now());
        assert_eq!(ev.event_type, "authority_template.adopted");
        assert_eq!(ev.audit_class, AuditClass::Alerted);
        assert_eq!(ev.org_scope, Some(org_id));
        assert_eq!(ev.provenance_auth_request_id, Some(ar_id));
        assert_eq!(ev.diff["after"]["template_kind"], "c");
        assert_eq!(
            ev.diff["after"]["adoption_auth_request_id"],
            ar_id.to_string()
        );
    }

    #[test]
    fn builders_leave_prev_event_hash_unset() {
        // SurrealAuditEmitter fills prev_event_hash at persist time by
        // looking up last_event_hash_for_org(event.org_scope). A
        // builder that pre-fills it would break the chain.
        let org = sample_org();
        let oid = org.id;
        let a = organization_created(AgentId::new(), &org, AgentId::new(), None, Utc::now());
        let b = authority_template_adopted(
            AgentId::new(),
            oid,
            TemplateKind::A,
            AuthRequestId::new(),
            Utc::now(),
        );
        assert!(a.prev_event_hash.is_none());
        assert!(b.prev_event_hash.is_none());
    }
}
