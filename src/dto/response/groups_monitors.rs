use crate::entity::groups_monitors::Model as GroupMonitorModel;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GroupMonitorResponse {
    pub id: u32,
    pub group_id: u32,
    pub monitor_id: u32,
}

impl From<&GroupMonitorModel> for GroupMonitorResponse {
    fn from(model: &GroupMonitorModel) -> Self {
        Self {
            id: model.id,
            group_id: model.group_id,
            monitor_id: model.monitor_id,
        }
    }
}
