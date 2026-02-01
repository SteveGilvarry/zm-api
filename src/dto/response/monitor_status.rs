use crate::dto::PaginatedResponse;
use crate::entity::monitor_status::Model as MonitorStatusModel;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MonitorStatusResponse {
    pub monitor_id: u32,
    pub status: String,
    pub capture_fps: String,
    pub analysis_fps: String,
    pub capture_bandwidth: i32,
    pub updated_on: String,
}

impl From<&MonitorStatusModel> for MonitorStatusResponse {
    fn from(model: &MonitorStatusModel) -> Self {
        Self {
            monitor_id: model.monitor_id,
            status: model.status.to_string(),
            capture_fps: model.capture_fps.to_string(),
            analysis_fps: model.analysis_fps.to_string(),
            capture_bandwidth: model.capture_bandwidth,
            updated_on: model.updated_on.to_rfc3339(),
        }
    }
}

/// Paginated response for monitor statuses
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PaginatedMonitorStatusesResponse {
    pub items: Vec<MonitorStatusResponse>,
    pub total: u64,
    pub per_page: u64,
    pub current_page: u64,
    pub last_page: u64,
}

impl From<PaginatedResponse<MonitorStatusResponse>> for PaginatedMonitorStatusesResponse {
    fn from(r: PaginatedResponse<MonitorStatusResponse>) -> Self {
        Self {
            items: r.items,
            total: r.total,
            per_page: r.per_page,
            current_page: r.current_page,
            last_page: r.last_page,
        }
    }
}
