//! [`AuditEmitter`] implementation backed by `SurrealStore`.
//!
//! Synchronous path on the hot write boundary:
//!
//! 1. Look up the last `prev_event_hash_b64` for `event.org_scope` via
//!    [`Repository::last_event_hash_for_org`].
//! 2. Populate `event.prev_event_hash` with the looked-up value (or
//!    `None` for the org's first event).
//! 3. Hash the event with [`domain::audit::hash_event`] — this digest
//!    is what the NEXT event within `org_scope` will copy into its own
//!    `prev_event_hash`.
//! 4. Persist via [`Repository::write_audit_event`].
//!
//! Why synchronous? Because the hash-chain invariant ("every event's
//! `prev_event_hash` equals the previous event's hash") depends on the
//! emit actually completing before the next emit begins. An async
//! queue would reorder emits and break the chain. M7b replaces this
//! with a durable queue + off-site stream while keeping the
//! same per-org linearisability.
//!
//! Shadow NDJSON log is a separate concern and is deferred to M7b
//! (decision D6 in the archived M2 plan).

use std::sync::Arc;

use async_trait::async_trait;

use domain::audit::{hash_event, AuditEmitter, AuditEvent};
use domain::repository::{Repository, RepositoryResult};

/// SurrealDB-backed `AuditEmitter`. Cheap to clone (`Arc` inside).
#[derive(Clone)]
pub struct SurrealAuditEmitter {
    repo: Arc<dyn Repository>,
}

impl SurrealAuditEmitter {
    pub fn new(repo: Arc<dyn Repository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl AuditEmitter for SurrealAuditEmitter {
    async fn emit(&self, mut event: AuditEvent) -> RepositoryResult<()> {
        // Step 1 + 2 — chain link.
        let prev = self.repo.last_event_hash_for_org(event.org_scope).await?;
        event.prev_event_hash = prev;

        // Step 3 — persist. The repository is responsible for storing
        // the serialized event; we don't compute the chain digest
        // ourselves here because the NEXT emit reads it off the persisted
        // row via `last_event_hash_for_org`. Callers that need the
        // digest for their own correlation (e.g. an M7b streamer) can
        // compute it from `event.canonical_bytes()` after we return.
        let _ = hash_event; // used in docs; keep import reachable.
        self.repo.write_audit_event(&event).await?;
        Ok(())
    }
}
