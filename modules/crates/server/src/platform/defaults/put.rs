//! `put_platform_defaults` — update the singleton with optimistic
//! concurrency.
//!
//! Flow (plan §P7 + D5):
//! 1. Load current. Absence → current_version = 0.
//! 2. Check `input.if_version == current_version`. Mismatch →
//!    [`DefaultsError::StaleWrite { current_version }`].
//! 3. Validate the incoming struct (light sanity — detailed validation
//!    lives in phi-core's serde types themselves).
//! 4. Mint a Template E AR — self-approved platform-admin write.
//! 5. Seed the catalogue entry at `platform_defaults:singleton` so
//!    Permission-Check Step 0 resolves on the URI (idempotent — every
//!    PUT re-seeds; seed is a no-op on existing rows).
//! 6. Build the new row with `version = current_version + 1`,
//!    `updated_at = now`.
//! 7. Persist.
//! 8. Emit `platform.defaults.updated` (Alerted) — diff carries the
//!    old + new snapshot (embedded phi-core serde is the single
//!    source of truth for field layouts).

use std::sync::Arc;

use chrono::{DateTime, Utc};
use domain::audit::events::m2::defaults as defaults_events;
use domain::audit::{AuditClass, AuditEmitter};
use domain::model::ids::AgentId;
use domain::model::nodes::{PrincipalRef, ResourceRef};
use domain::model::{Composite, PlatformDefaults};
use domain::repository::Repository;
use domain::templates::e::{build_auto_approved_request, BuildArgs};

use super::{defaults_uri, DefaultsError, PutOutcome, KIND_TAG};

pub struct PutInput {
    pub if_version: u64,
    pub defaults: PlatformDefaults,
    pub actor: AgentId,
    pub now: DateTime<Utc>,
}

pub async fn put_platform_defaults(
    repo: Arc<dyn Repository>,
    audit: Arc<dyn AuditEmitter>,
    input: PutInput,
) -> Result<PutOutcome, DefaultsError> {
    // 1 + 2. Load current; stale-check.
    let current = repo.get_platform_defaults().await?;
    let current_version = current.as_ref().map(|c| c.version).unwrap_or(0);
    if input.if_version != current_version {
        return Err(DefaultsError::StaleWrite { current_version });
    }

    // 3. Light validation — zero-value execution limits are usually
    //    operator errors, but we don't second-guess phi-core's
    //    domain; just block the obvious foot-guns.
    if input.defaults.execution_limits.max_turns == 0 {
        return Err(DefaultsError::Validation(
            "execution_limits.max_turns must be > 0".into(),
        ));
    }
    if input.defaults.execution_limits.max_total_tokens == 0 {
        return Err(DefaultsError::Validation(
            "execution_limits.max_total_tokens must be > 0".into(),
        ));
    }
    if input.defaults.retry_config.max_retries > 100 {
        return Err(DefaultsError::Validation(
            "retry_config.max_retries must be <= 100 (sanity bound)".into(),
        ));
    }

    // 4. Template E AR — self-approved platform-admin write.
    let uri = defaults_uri().to_string();
    let ar = build_auto_approved_request(BuildArgs {
        requestor_and_approver: PrincipalRef::Agent(input.actor),
        resource: ResourceRef { uri: uri.clone() },
        kinds: vec![KIND_TAG.to_string()],
        scope: vec![uri.clone()],
        justification: Some(format!(
            "self-approved platform-admin write: update platform defaults (v{} → v{})",
            current_version,
            current_version + 1,
        )),
        audit_class: AuditClass::Alerted,
        now: input.now,
    });
    let auth_request_id = ar.id;
    repo.create_auth_request(&ar).await?;

    // 5. Catalogue seed — idempotent. ControlPlaneObject is the
    //    composite class for the platform-defaults singleton.
    repo.seed_catalogue_entry_for_composite(None, &uri, Composite::ControlPlaneObject)
        .await?;

    // 6. Build the new row. Version MUST be bumped here (BEFORE the
    //    persist call on the next line) — a retry after a transient
    //    repo error would otherwise leave `version` unchanged, and
    //    the next successful write would hand the same version back
    //    to two different clients. Version monotonicity is the whole
    //    point of the OCC contract; moving this assignment below the
    //    `put_platform_defaults` call would break it silently.
    let mut next = input.defaults;
    next.singleton = 1;
    next.version = current_version + 1;
    next.updated_at = input.now;

    // 7. Persist.
    repo.put_platform_defaults(&next).await?;

    // 8. Audit.
    let event = defaults_events::platform_defaults_updated(
        input.actor,
        current.as_ref(),
        &next,
        Some(auth_request_id),
        input.now,
    );
    let audit_event_id = event.event_id;
    audit
        .emit(event)
        .await
        .map_err(|e| DefaultsError::AuditEmit(e.to_string()))?;

    Ok(PutOutcome {
        new_version: next.version,
        auth_request_id,
        audit_event_id,
    })
}
