use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateServerRequest {
    pub name: String,
    pub hostname: Option<String>,
    pub port: Option<u32>,
    pub status: Option<String>,
}
