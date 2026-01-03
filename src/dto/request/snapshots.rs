use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CreateSnapshotRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub created_by: Option<i32>,
    #[schema(value_type = Option<String>, example = "2025-01-15T10:30:00")]
    pub created_on: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UpdateSnapshotRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub created_by: Option<i32>,
    #[schema(value_type = Option<String>, example = "2025-01-15T10:30:00")]
    pub created_on: Option<NaiveDateTime>,
}
