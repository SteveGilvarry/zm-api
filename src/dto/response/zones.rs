use crate::dto::PaginatedResponse;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ZoneResponse {
    pub id: u32,
    pub monitor_id: u32,
    pub name: String,
    pub r#type: String,
    pub units: String,
    pub num_coords: u8,
    pub coords: String,
}

impl From<&crate::entity::zones::Model> for ZoneResponse {
    fn from(m: &crate::entity::zones::Model) -> Self {
        Self {
            id: m.id,
            monitor_id: m.monitor_id,
            name: m.name.clone(),
            r#type: m.r#type.to_string(),
            units: m.units.to_string(),
            num_coords: m.num_coords,
            coords: m.coords.clone(),
        }
    }
}

/// Paginated response for zones
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PaginatedZonesResponse {
    pub items: Vec<ZoneResponse>,
    pub total: u64,
    pub per_page: u64,
    pub current_page: u64,
    pub last_page: u64,
}

impl From<PaginatedResponse<ZoneResponse>> for PaginatedZonesResponse {
    fn from(r: PaginatedResponse<ZoneResponse>) -> Self {
        Self {
            items: r.items,
            total: r.total,
            per_page: r.per_page,
            current_page: r.current_page,
            last_page: r.last_page,
        }
    }
}
