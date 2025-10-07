use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CreateSessionRequest {
    pub id: String,
    pub access: Option<u32>,
    pub data: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UpdateSessionRequest {
    pub access: Option<u32>,
    pub data: Option<String>,
}
