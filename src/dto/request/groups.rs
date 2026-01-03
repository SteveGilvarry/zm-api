use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateGroupRequest {
    pub name: String,
    pub parent_id: Option<u32>,
}
