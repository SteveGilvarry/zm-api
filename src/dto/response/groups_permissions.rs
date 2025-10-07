use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::entity::groups_permissions::Model as GroupPermissionModel;
use crate::entity::sea_orm_active_enums::Permission;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GroupPermissionResponse {
    pub id: u32,
    pub group_id: u32,
    pub user_id: u32,
    pub permission: String,
}

impl From<&GroupPermissionModel> for GroupPermissionResponse {
    fn from(model: &GroupPermissionModel) -> Self {
        let permission_str = match model.permission {
            Permission::Inherit => "Inherit",
            Permission::None => "None",
            Permission::View => "View",
            Permission::Edit => "Edit",
        }.to_string();
        
        Self {
            id: model.id,
            group_id: model.group_id,
            user_id: model.user_id,
            permission: permission_str,
        }
    }
}
