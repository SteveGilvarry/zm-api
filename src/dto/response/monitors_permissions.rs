use crate::entity::monitors_permissions::Model as MonitorPermissionModel;
use crate::entity::sea_orm_active_enums::Permission;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
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
