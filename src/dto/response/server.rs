use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MessageResponse {
    message: String
}

impl MessageResponse {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ServiceStatusResponse {
    status: String
}

impl ServiceStatusResponse {
    pub fn new(status: &str) -> Self {
        Self {
            status: status.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct VersionResponse {
    /// ZoneMinder version
    pub version: String,
    /// API version
    pub api_version: String,
    /// Database version
    pub db_version: String,
}