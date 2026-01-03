use crate::entity::event_data::Model as EventDataModel;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EventDataResponse {
    pub id: u64,
    pub event_id: Option<u64>,
    pub monitor_id: Option<u32>,
    pub frame_id: Option<u32>,
    pub timestamp: Option<String>,
    pub data: Option<String>,
}

impl From<&EventDataModel> for EventDataResponse {
    fn from(model: &EventDataModel) -> Self {
        Self {
            id: model.id,
            event_id: model.event_id,
            monitor_id: model.monitor_id,
            frame_id: model.frame_id,
            timestamp: model.timestamp.map(|dt| dt.to_rfc3339()),
            data: model.data.clone(),
        }
    }
}
