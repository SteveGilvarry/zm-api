use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CreateStateRequest {
    pub name: String,
    pub definition: String,
    pub is_active: u8,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UpdateStateRequest {
    pub name: Option<String>,
    pub definition: Option<String>,
    pub is_active: Option<u8>,
}
