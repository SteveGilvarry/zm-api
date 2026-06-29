//! Response DTO for the per-monitor zm-next processing-graph endpoints.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

use crate::entity::monitor_pipeline;

/// A monitor's stored zm-next processing plugin graph (the "free graph").
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MonitorPipelineResponse {
    pub monitor_id: u32,
    /// Document/schema version.
    pub version: u32,
    /// The processing plugin graph document: `{ "plugins": [ {id,kind,cfg,children}, ... ] }`.
    #[schema(value_type = Object)]
    pub graph: Value,
    /// RFC 3339 timestamps.
    pub created_at: String,
    pub updated_at: String,
}

impl From<monitor_pipeline::Model> for MonitorPipelineResponse {
    fn from(m: monitor_pipeline::Model) -> Self {
        // The stored graph is validated on write, so it parses; fall back to Null
        // only defensively (e.g. a hand-edited DB row).
        let graph = serde_json::from_str(&m.graph_json).unwrap_or(Value::Null);
        Self {
            monitor_id: m.monitor_id,
            version: m.version,
            graph,
            created_at: m.created_at.and_utc().to_rfc3339(),
            updated_at: m.updated_at.and_utc().to_rfc3339(),
        }
    }
}
