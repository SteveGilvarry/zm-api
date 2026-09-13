use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateServerRequest {
    #[garde(length(chars, max = 64))]
    pub name: String,
    #[garde(skip)]
    pub hostname: Option<String>,
    #[garde(skip)]
    pub port: Option<u32>,
    #[garde(skip)]
    pub status: Option<String>,
}
