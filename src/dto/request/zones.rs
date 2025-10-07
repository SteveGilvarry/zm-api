use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateZoneRequest {
    pub name: String,
    pub r#type: String,
    pub units: String,
    pub coords: String,
    pub num_coords: u8,
    pub check_method: Option<String>,
}
