//! `list_mcp_servers` — catalogue read over external-service rows.
//!
//! Returns every field, including the stored `endpoint` string (the
//! phi-core transport argument). No audit event — list ops are routine.

use std::sync::Arc;

use domain::model::ExternalService;
use domain::repository::Repository;

use super::McpError;

pub struct ListInput {
    pub include_archived: bool,
}

pub struct ListOutcome {
    pub servers: Vec<ExternalService>,
}

pub async fn list_mcp_servers(
    repo: Arc<dyn Repository>,
    input: ListInput,
) -> Result<ListOutcome, McpError> {
    let servers = repo.list_mcp_servers(input.include_archived).await?;
    Ok(ListOutcome { servers })
}
