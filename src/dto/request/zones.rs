use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
#[garde(allow_unvalidated)]
pub struct CreateZoneRequest {
    #[garde(length(min = 1, max = 64))]
    pub name: String,
    #[garde(length(max = 32))]
    pub r#type: String,
    #[garde(length(max = 32))]
    pub units: String,
    // Polygon coords serialised as a string. The underlying column is
    // TINYTEXT (max 255 bytes), so anything beyond that would 500 at insert
    // time — match the DB cap exactly.
    #[garde(length(max = 255))]
    pub coords: String,
    pub num_coords: u8,
    #[garde(inner(length(max = 32)))]
    pub check_method: Option<String>,
}
