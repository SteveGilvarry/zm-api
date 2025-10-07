use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use sea_orm::prelude::DateTimeUtc;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CreateTagRequest {
    pub name: String,
    #[schema(value_type = Option<String>, example = "2025-01-15T10:30:00Z")]
    pub create_date: Option<DateTimeUtc>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UpdateTagRequest {
    pub name: Option<String>,
    #[schema(value_type = Option<String>, example = "2025-01-15T10:30:00Z")]
    pub create_date: Option<DateTimeUtc>,
}
