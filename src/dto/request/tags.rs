use sea_orm::prelude::DateTimeUtc;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

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

/// Query params for tag detail with paginated events
#[derive(Debug, Deserialize, IntoParams)]
pub struct TagDetailQuery {
    /// Page number (1-indexed)
    #[param(minimum = 1)]
    pub page: Option<u64>,
    /// Items per page
    #[param(minimum = 1, maximum = 100)]
    pub page_size: Option<u64>,
}
