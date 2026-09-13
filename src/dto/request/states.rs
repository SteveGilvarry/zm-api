use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Validate)]
pub struct CreateStateRequest {
    #[garde(length(chars, max = 64))]
    pub name: String,
    #[garde(skip)]
    pub definition: String,
    #[garde(skip)]
    pub is_active: u8,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Validate)]
pub struct UpdateStateRequest {
    #[garde(inner(length(chars, max = 64)))]
    pub name: Option<String>,
    #[garde(skip)]
    pub definition: Option<String>,
    #[garde(skip)]
    pub is_active: Option<u8>,
}
