use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateFilterRequest {
    pub name: String,
    pub query_json: String,
    pub user_id: Option<u32>,
    pub execute_interval: Option<u32>,
    pub email_format: Option<String>,
}
