use crate::dto::PaginatedResponse;
use crate::entity::monitors_permissions::Model as MonitorPermissionModel;
use crate::entity::sea_orm_active_enums::Permission;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MonitorPermissionResponse {
    pub id: u32,
    pub monitor_id: u32,
    pub user_id: u32,
    pub permission: String,
}

impl From<&MonitorPermissionModel> for MonitorPermissionResponse {
    fn from(model: &MonitorPermissionModel) -> Self {
        let permission_str = match model.permission {
            Permission::Inherit => "Inherit",
            Permission::None => "None",
            Permission::View => "View",
            Permission::Edit => "Edit",
        }
        .to_string();

        Self {
            id: model.id,
            monitor_id: model.monitor_id,
            user_id: model.user_id,
            permission: permission_str,
        }
    }
}

/// Paginated response for monitor permissions
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PaginatedMonitorPermissionsResponse {
    pub items: Vec<MonitorPermissionResponse>,
    pub total: u64,
    pub per_page: u64,
    pub current_page: u64,
    pub last_page: u64,
}

impl From<PaginatedResponse<MonitorPermissionResponse>> for PaginatedMonitorPermissionsResponse {
    fn from(r: PaginatedResponse<MonitorPermissionResponse>) -> Self {
        Self {
            items: r.items,
            total: r.total,
            per_page: r.per_page,
            current_page: r.current_page,
            last_page: r.last_page,
        }
    }
}
