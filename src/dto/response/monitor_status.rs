use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::entity::monitor_status::Model as MonitorStatusModel;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MonitorStatusResponse {
    pub monitor_id: u32,
    pub status: String,
    pub capture_fps: String,
    pub analysis_fps: String,
    pub capture_bandwidth: i32,
    pub updated_on: String,
}

impl From<&MonitorStatusModel> for MonitorStatusResponse {
    fn from(model: &MonitorStatusModel) -> Self {
        Self {
            monitor_id: model.monitor_id,
            status: model.status.to_string(),
            capture_fps: model.capture_fps.to_string(),
            analysis_fps: model.analysis_fps.to_string(),
            capture_bandwidth: model.capture_bandwidth,
            updated_on: model.updated_on.to_rfc3339(),
        }
    }
}
