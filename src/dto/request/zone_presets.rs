use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateZonePresetRequest {
    #[garde(length(chars, max = 64))]
    pub name: String,
    #[garde(skip)]
    pub r#type: String,
    #[garde(skip)]
    pub units: String,
    #[garde(skip)]
    pub check_method: String,
}
