use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CreateUserPreferenceRequest {
    pub user_id: u32,
    pub name: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UpdateUserPreferenceRequest {
    pub user_id: Option<u32>,
    pub name: Option<String>,
    pub value: Option<String>,
}
