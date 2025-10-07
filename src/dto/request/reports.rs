use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateReportRequest {
    #[schema(example = "Weekly Security Report")]
    pub name: Option<String>,
    #[schema(example = 1)]
    pub filter_id: Option<u32>,
    #[schema(value_type = String, example = "2025-01-01T00:00:00Z")]
    pub start_date_time: Option<String>,
    #[schema(value_type = String, example = "2025-01-08T00:00:00Z")]
    pub end_date_time: Option<String>,
    #[schema(example = 604800)]
    pub interval: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateReportRequest {
    #[schema(example = "Weekly Security Report")]
    pub name: Option<String>,
    #[schema(example = 1)]
    pub filter_id: Option<u32>,
    #[schema(value_type = String, example = "2025-01-01T00:00:00Z")]
    pub start_date_time: Option<String>,
    #[schema(value_type = String, example = "2025-01-08T00:00:00Z")]
    pub end_date_time: Option<String>,
    #[schema(example = 604800)]
    pub interval: Option<u32>,
}
