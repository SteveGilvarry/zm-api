use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateZonePresetRequest {
    pub name: String,
    pub r#type: String,
    pub units: String,
    pub check_method: String,
}
