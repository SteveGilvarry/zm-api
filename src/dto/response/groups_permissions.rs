use crate::dto::PaginatedResponse;
use crate::entity::groups_permissions::Model as GroupPermissionModel;
use crate::entity::sea_orm_active_enums::Permission;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
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
        }
        .to_string();

        Self {
            id: model.id,
            group_id: model.group_id,
            user_id: model.user_id,
            permission: permission_str,
        }
    }
}

/// Paginated response for group permissions
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PaginatedGroupPermissionsResponse {
    pub items: Vec<GroupPermissionResponse>,
    pub total: u64,
    pub per_page: u64,
    pub current_page: u64,
    pub last_page: u64,
}

impl From<PaginatedResponse<GroupPermissionResponse>> for PaginatedGroupPermissionsResponse {
    fn from(r: PaginatedResponse<GroupPermissionResponse>) -> Self {
        Self {
            items: r.items,
            total: r.total,
            per_page: r.per_page,
            current_page: r.current_page,
            last_page: r.last_page,
        }
    }
}
