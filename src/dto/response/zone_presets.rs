use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
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
