use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateMonitorStatusRequest {
    #[schema(example = "Connected")]
    pub status: Option<String>,
    #[schema(value_type = String, example = "15.5")]
    pub capture_fps: Option<String>,
    #[schema(value_type = String, example = "14.8")]
    pub analysis_fps: Option<String>,
    #[schema(example = 1024000)]
    pub capture_bandwidth: Option<i32>,
}
