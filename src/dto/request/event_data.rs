use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateEventDataRequest {
    #[schema(example = 1)]
    pub event_id: Option<u64>,
    #[schema(example = 1)]
    pub monitor_id: Option<u32>,
    #[schema(example = 1)]
    pub frame_id: Option<u32>,
    #[schema(value_type = String, example = "2025-01-01T00:00:00Z")]
    pub timestamp: Option<String>,
    #[schema(example = "{\"detections\": []}")]
    pub data: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateEventDataRequest {
    #[schema(example = "{\"detections\": []}")]
    pub data: Option<String>,
}
