//! M5 / CH-14 additions to the SurrealDB-backed `Repository` impl —
//! the authority-chain walker and the recursive revocation cascade.
//!
//! These are **inherent methods** on [`SurrealStore`]; the thin trait
//! delegations live in [`super::repo_impl`] so the `Repository` trait
//! impl stays in one `impl` block. Helpers here use the `m5_` prefix
//! to mirror the existing `m2_` convention.
//!
//! Per ADR-0053 §D53.3 + §D53.4. The chain depth cap is
//! [`MAX_PROVENANCE_DEPTH`] (= 32 hops); exceeding it returns
//! [`RepositoryError::ProvenanceCycleDepthExceeded`].

use chrono::{DateTime, Utc};
use uuid::Uuid;

use domain::model::ids::{AuthRequestId, GrantId};
use domain::model::nodes::AuthRequest;
use domain::repository::{CascadeResult, RepositoryError, RepositoryResult, MAX_PROVENANCE_DEPTH};

use crate::repo_impl::{auth_request_row_into_domain_via, parse_grant_id};
use crate::SurrealStore;

impl SurrealStore {
    // ---- walk_provenance_chain ----------------------------------------

    /// CH-14 / ADR-0053 §D53.3 — leaf-to-root chain walk.
    ///
    /// Issues per-level SELECTs against the `grant` and `auth_request`
    /// tables. Typical chain depth in production is 1; the depth cap
    /// of 32 is a defensive guard.
    pub(crate) async fn m5_walk_provenance_chain(
        &self,
        grant: GrantId,
    ) -> RepositoryResult<Vec<AuthRequest>> {
        // Step 0: load the leaf grant's `descends_from` — String column
        // per migration 0001 (`grant.descends_from TYPE option<string>`).
        let mut current_ar_id_str = match self.fetch_grant_descends_from(grant).await? {
            Some(s) => s,
            None => return Ok(Vec::new()),
        };

        let mut leaf_to_root: Vec<AuthRequest> = Vec::new();
        for _ in 0..MAX_PROVENANCE_DEPTH {
            let ar = self
                .fetch_auth_request_by_id_str(&current_ar_id_str)
                .await?;
            let parent_grant_id = ar.descends_from_grant;
            let is_bootstrap = domain::permissions::axioms::is_bootstrap_ar(&ar);
            leaf_to_root.push(ar);
            if is_bootstrap {
                leaf_to_root.reverse();
                return Ok(leaf_to_root);
            }
            // AR.descends_from_grant -> Grant -> Grant.descends_from -> next AR.
            let parent_grant_id = match parent_grant_id {
                Some(g) => g,
                None => {
                    leaf_to_root.reverse();
                    return Ok(leaf_to_root);
                }
            };
            current_ar_id_str = match self.fetch_grant_descends_from(parent_grant_id).await? {
                Some(s) => s,
                None => {
                    leaf_to_root.reverse();
                    return Ok(leaf_to_root);
                }
            };
        }
        Err(RepositoryError::ProvenanceCycleDepthExceeded {
            depth_cap: MAX_PROVENANCE_DEPTH,
        })
    }

    async fn fetch_grant_descends_from(&self, grant: GrantId) -> RepositoryResult<Option<String>> {
        // SELECT a single string column from a single row. The row may
        // be absent (NotFound) or the column may be NONE (legacy
        // grant); both surface as `Ok(None)`.
        let mut resp = self
            .client()
            .query("SELECT descends_from FROM type::thing('grant', $id)")
            .bind(("id", grant.to_string()))
            .await
            .map_err(backend)?;
        let rows: Vec<serde_json::Value> = resp.take(0).map_err(backend)?;
        let row = match rows.into_iter().next() {
            Some(v) => v,
            None => return Err(RepositoryError::NotFound),
        };
        Ok(row
            .get("descends_from")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()))
    }

    async fn fetch_auth_request_by_id_str(&self, id_str: &str) -> RepositoryResult<AuthRequest> {
        let id = AuthRequestId::from_uuid(uuid::Uuid::parse_str(id_str).map_err(backend)?);
        // Mirrors `Repository::get_auth_request` in `repo_impl.rs`:
        // `SELECT * OMIT id FROM type::thing('auth_request', $id)`.
        let mut resp = self
            .client()
            .query("SELECT * OMIT id FROM type::thing('auth_request', $id)")
            .bind(("id", id.to_string()))
            .await
            .map_err(backend)?;
        let rows: Vec<serde_json::Value> = resp.take(0).map_err(backend)?;
        let row = rows.into_iter().next().ok_or(RepositoryError::NotFound)?;
        auth_request_row_into_domain_via(row, id)
    }

    // ---- recursive revocation cascade ---------------------------------

    /// CH-14 / ADR-0053 §D53.4 + §D53.7 — BFS from `ar` through the
    /// authority subtree.
    ///
    /// Each level: (a) UPDATE every live grant whose `descends_from`
    /// is in the current frontier → newly_revoked. (b) SELECT every
    /// AR whose `descends_from_grant` is in `newly_revoked` → next
    /// frontier (also recorded in `cascaded_ars`). Loop until no new
    /// revocations or depth cap.
    ///
    /// Returns a [`CascadeResult`] with both axes — `revoked_grants`
    /// for the `template.revoked` summary event and `cascaded_ars`
    /// for the per-AR `auth_request.revoked` events emitted by the
    /// handler layer (`server::platform::templates::revoke`). This
    /// method is pure storage; audit emission stays in the handler
    /// per the single-writer pattern.
    pub(crate) async fn m5_revoke_grants_by_descends_from_recursive(
        &self,
        ar: AuthRequestId,
        at: DateTime<Utc>,
    ) -> RepositoryResult<CascadeResult> {
        let mut revoked_grants: Vec<GrantId> = Vec::new();
        let mut cascaded_ars: Vec<AuthRequestId> = Vec::new();
        let mut frontier_ars: Vec<String> = vec![ar.to_string()];
        for _ in 0..MAX_PROVENANCE_DEPTH {
            if frontier_ars.is_empty() {
                return Ok(CascadeResult {
                    revoked_grants,
                    cascaded_ars,
                });
            }
            // (a) Revoke every live grant whose descends_from is in
            //     `frontier_ars`. The IN-clause uses bind variables
            //     for safety. Surreal accepts an array binding.
            let mut resp = self
                .client()
                .query(
                    "UPDATE grant SET revoked_at = $at \
                     WHERE descends_from INSIDE $frontier \
                     AND revoked_at IS NONE \
                     RETURN record::id(id) AS __id",
                )
                .bind(("at", at.to_rfc3339()))
                .bind(("frontier", frontier_ars.clone()))
                .await
                .map_err(backend)?;
            let rows: Vec<serde_json::Value> = resp.take(0).map_err(backend)?;
            let mut newly_revoked: Vec<GrantId> = Vec::with_capacity(rows.len());
            for row in rows {
                let id_str = row.get("__id").and_then(|v| v.as_str()).ok_or_else(|| {
                    RepositoryError::Backend("grant cascade revoke missing __id".into())
                })?;
                newly_revoked.push(parse_grant_id(id_str)?);
            }
            if newly_revoked.is_empty() {
                return Ok(CascadeResult {
                    revoked_grants,
                    cascaded_ars,
                });
            }
            revoked_grants.extend(newly_revoked.iter().copied());
            // (b) Find the next-level ARs whose descends_from_grant
            //     is one of the just-revoked grants. Each row collected
            //     here is a level-≥1 cascaded AR per ADR-0053 §D53.7.
            let revoked_strs: Vec<String> = newly_revoked.iter().map(|g| g.to_string()).collect();
            let mut resp = self
                .client()
                .query(
                    "SELECT record::id(id) AS __id FROM auth_request \
                     WHERE descends_from_grant INSIDE $revoked",
                )
                .bind(("revoked", revoked_strs))
                .await
                .map_err(backend)?;
            let rows: Vec<serde_json::Value> = resp.take(0).map_err(backend)?;
            let mut next_frontier_strs: Vec<String> = Vec::with_capacity(rows.len());
            let mut next_frontier_ids: Vec<AuthRequestId> = Vec::with_capacity(rows.len());
            for row in rows {
                let id_str = row.get("__id").and_then(|v| v.as_str()).ok_or_else(|| {
                    RepositoryError::Backend("ar cascade scan missing __id".into())
                })?;
                if !next_frontier_strs.iter().any(|s| s == id_str) {
                    let ar_uuid = Uuid::parse_str(id_str).map_err(|e| {
                        RepositoryError::Backend(format!("ar cascade scan __id is not a UUID: {e}"))
                    })?;
                    let ar_id = AuthRequestId::from_uuid(ar_uuid);
                    next_frontier_strs.push(id_str.to_string());
                    next_frontier_ids.push(ar_id);
                }
            }
            for ar_id in &next_frontier_ids {
                if !cascaded_ars.contains(ar_id) {
                    cascaded_ars.push(*ar_id);
                }
            }
            frontier_ars = next_frontier_strs;
        }
        Err(RepositoryError::ProvenanceCycleDepthExceeded {
            depth_cap: MAX_PROVENANCE_DEPTH,
        })
    }
}

fn backend(e: impl std::fmt::Display) -> RepositoryError {
    RepositoryError::Backend(e.to_string())
}
