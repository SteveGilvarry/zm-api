use crate::dto::PaginatedResponse;
use crate::entity::groups_monitors::Model as GroupMonitorModel;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GroupMonitorResponse {
    pub id: u32,
    pub group_id: u32,
    pub monitor_id: u32,
}

impl From<&GroupMonitorModel> for GroupMonitorResponse {
    fn from(model: &GroupMonitorModel) -> Self {
        Self {
            id: model.id,
            group_id: model.group_id,
            monitor_id: model.monitor_id,
        }
    }
}

/// Paginated response for group-monitor associations
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PaginatedGroupMonitorsResponse {
    pub items: Vec<GroupMonitorResponse>,
    pub total: u64,
    pub per_page: u64,
    pub current_page: u64,
    pub last_page: u64,
}

impl From<PaginatedResponse<GroupMonitorResponse>> for PaginatedGroupMonitorsResponse {
    fn from(r: PaginatedResponse<GroupMonitorResponse>) -> Self {
        Self {
            items: r.items,
            total: r.total,
            per_page: r.per_page,
            current_page: r.current_page,
            last_page: r.last_page,
        }
    }
}
