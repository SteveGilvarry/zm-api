use crate::dto::PaginatedResponse;
use crate::entity::monitor_status::Model as MonitorStatusModel;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MonitorStatusResponse {
    pub monitor_id: u32,
    pub status: String,
    pub capture_fps: String,
    pub analysis_fps: String,
    pub capture_bandwidth: i32,
    pub updated_on: String,
    /// Live alarm state from the monitor's shared memory: `idle`,
    /// `prealarm`, `alarm` or `alert` (what `zmu -s` reports). Only on
    /// `GET /monitor-status/{id}`, and only while zmc is running.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alarm_state: Option<String>,
    /// Whether alarms are forced off (`zmu -n`, or `disable_alarms` on the
    /// alarm endpoint). Same availability as `alarm_state`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alarms_disabled: Option<bool>,
    /// Whether the monitor is analysing right now. Same availability.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub analysing: Option<bool>,
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
            alarm_state: None,
            alarms_disabled: None,
            analysing: None,
        }
    }
}

/// Paginated response for monitor statuses
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PaginatedMonitorStatusesResponse {
    pub items: Vec<MonitorStatusResponse>,
    pub total: u64,
    pub per_page: u64,
    pub current_page: u64,
    pub last_page: u64,
}

impl From<PaginatedResponse<MonitorStatusResponse>> for PaginatedMonitorStatusesResponse {
    fn from(r: PaginatedResponse<MonitorStatusResponse>) -> Self {
        Self {
            items: r.items,
            total: r.total,
            per_page: r.per_page,
            current_page: r.current_page,
            last_page: r.last_page,
        }
    }
}

/// A monitor's live audio level, for the monitor editor's meter (#180).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AudioLevelResponse {
    /// 0-100, the scale `audio_threshold` uses; null while zmc isn't running.
    pub level: Option<u8>,
    /// Whether the level is over the threshold; null while zmc isn't running.
    pub alarm: Option<bool>,
    /// The monitor's `AudioThreshold`.
    pub threshold: u8,
    /// Whether audio detection scores (`AudioDetection`).
    pub detection: bool,
}
