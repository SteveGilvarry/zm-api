use crate::entity::control_presets::Model as ControlPresetModel;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ControlPresetResponse {
    pub monitor_id: u32,
    pub preset: u32,
    pub label: String,
}

impl From<&ControlPresetModel> for ControlPresetResponse {
    fn from(model: &ControlPresetModel) -> Self {
        Self {
            monitor_id: model.monitor_id,
            preset: model.preset,
            label: model.label.clone(),
        }
    }
}
