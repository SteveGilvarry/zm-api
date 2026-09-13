use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Validate)]
pub struct CreateSessionRequest {
    #[garde(length(chars, max = 32))]
    pub id: String,
    #[garde(skip)]
    pub access: Option<u32>,
    #[garde(skip)]
    pub data: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Validate)]
pub struct UpdateSessionRequest {
    #[garde(skip)]
    pub access: Option<u32>,
    #[garde(skip)]
    pub data: Option<String>,
}
