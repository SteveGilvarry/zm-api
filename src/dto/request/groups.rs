use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateGroupRequest {
    #[garde(length(chars, max = 64))]
    pub name: String,
    #[garde(skip)]
    pub parent_id: Option<u32>,
}
