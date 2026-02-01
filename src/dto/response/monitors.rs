use crate::dto::response::MonitorResponse;
use crate::dto::PaginatedResponse;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Paginated response for monitors
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct PaginatedMonitorsResponse {
    pub items: Vec<MonitorResponse>,
    pub total: u64,
    pub per_page: u64,
    pub current_page: u64,
    pub last_page: u64,
}

impl From<PaginatedResponse<MonitorResponse>> for PaginatedMonitorsResponse {
    fn from(r: PaginatedResponse<MonitorResponse>) -> Self {
        Self {
            items: r.items,
            total: r.total,
            per_page: r.per_page,
            current_page: r.current_page,
            last_page: r.last_page,
        }
    }
}
