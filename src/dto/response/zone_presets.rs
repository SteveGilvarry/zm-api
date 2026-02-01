use crate::dto::PaginatedResponse;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ZonePresetResponse {
    pub id: u32,
    pub name: String,
    pub r#type: String,
    pub units: String,
    pub check_method: String,
}

impl From<&crate::entity::zone_presets::Model> for ZonePresetResponse {
    fn from(m: &crate::entity::zone_presets::Model) -> Self {
        Self {
            id: m.id,
            name: m.name.clone(),
            r#type: format!("{:?}", m.r#type),
            units: format!("{:?}", m.units),
            check_method: format!("{:?}", m.check_method),
        }
    }
}

/// Paginated response for zone presets
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PaginatedZonePresetsResponse {
    pub items: Vec<ZonePresetResponse>,
    pub total: u64,
    pub per_page: u64,
    pub current_page: u64,
    pub last_page: u64,
}

impl From<PaginatedResponse<ZonePresetResponse>> for PaginatedZonePresetsResponse {
    fn from(r: PaginatedResponse<ZonePresetResponse>) -> Self {
        Self {
            items: r.items,
            total: r.total,
            per_page: r.per_page,
            current_page: r.current_page,
            last_page: r.last_page,
        }
    }
}
