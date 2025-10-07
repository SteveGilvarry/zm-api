use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CreateControlPresetRequest {
    pub monitor_id: u32,
    pub preset: u32,
    pub label: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UpdateControlPresetRequest {
    pub label: Option<String>,
}
