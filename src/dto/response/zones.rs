use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
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

