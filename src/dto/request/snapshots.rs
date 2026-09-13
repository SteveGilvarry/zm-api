use chrono::NaiveDateTime;
use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Validate)]
pub struct CreateSnapshotRequest {
    #[garde(inner(length(chars, max = 64)))]
    pub name: Option<String>,
    #[garde(skip)]
    pub description: Option<String>,
    #[garde(skip)]
    pub created_by: Option<i32>,
    #[schema(value_type = Option<String>, example = "2025-01-15T10:30:00")]
    #[garde(skip)]
    pub created_on: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Validate)]
pub struct UpdateSnapshotRequest {
    #[garde(inner(length(chars, max = 64)))]
    pub name: Option<String>,
    #[garde(skip)]
    pub description: Option<String>,
    #[garde(skip)]
    pub created_by: Option<i32>,
    #[schema(value_type = Option<String>, example = "2025-01-15T10:30:00")]
    #[garde(skip)]
    pub created_on: Option<NaiveDateTime>,
}
