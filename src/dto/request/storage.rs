use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateStorageRequest {
    pub name: String,
    pub path: String,
    pub r#type: String,
    pub enabled: i8,
    pub scheme: Option<String>,
    pub server_id: Option<u32>,
    pub url: Option<String>,
}
