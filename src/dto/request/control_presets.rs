use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Validate)]
#[garde(allow_unvalidated)]
pub struct CreateControlPresetRequest {
    pub monitor_id: u32,
    pub preset: u32,
    #[garde(length(min = 1, max = 64))]
    pub label: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Validate)]
#[garde(allow_unvalidated)]
pub struct UpdateControlPresetRequest {
    #[garde(inner(length(min = 1, max = 64)))]
    pub label: Option<String>,
}
