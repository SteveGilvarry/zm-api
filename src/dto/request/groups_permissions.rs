use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateGroupPermissionRequest {
    #[schema(example = 1)]
    pub group_id: u32,
    #[schema(example = 1)]
    pub user_id: u32,
    #[schema(example = "View")]
    pub permission: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateGroupPermissionRequest {
    #[schema(example = "Edit")]
    pub permission: Option<String>,
}
