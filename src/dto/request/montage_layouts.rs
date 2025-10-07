use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CreateMontageLayoutRequest {
    pub name: String,
    pub user_id: u32,
    pub positions: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UpdateMontageLayoutRequest {
    pub name: Option<String>,
    pub user_id: Option<u32>,
    pub positions: Option<String>,
}
