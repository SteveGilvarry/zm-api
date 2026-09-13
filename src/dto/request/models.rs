use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateModelRequest {
    #[garde(length(chars, max = 64))]
    pub name: String,
    #[garde(skip)]
    pub manufacturer_id: Option<i32>,
}
