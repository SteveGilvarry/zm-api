use crate::dto::PaginatedResponse;
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

/// Paginated response for control presets
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PaginatedControlPresetsResponse {
    pub items: Vec<ControlPresetResponse>,
    pub total: u64,
    pub per_page: u64,
    pub current_page: u64,
    pub last_page: u64,
}

impl From<PaginatedResponse<ControlPresetResponse>> for PaginatedControlPresetsResponse {
    fn from(r: PaginatedResponse<ControlPresetResponse>) -> Self {
        Self {
            items: r.items,
            total: r.total,
            per_page: r.per_page,
            current_page: r.current_page,
            last_page: r.last_page,
        }
    }
}
