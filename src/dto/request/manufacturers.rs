use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateManufacturerRequest {
    #[garde(length(min = 1, max = 64))]
    pub name: String,
}
