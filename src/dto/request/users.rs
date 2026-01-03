use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub email: String,
    pub name: Option<String>,
    pub phone: Option<String>,
    pub enabled: Option<u8>,
}
