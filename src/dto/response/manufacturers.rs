use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ManufacturerResponse {
    pub id: u32,
    pub name: String,
}

impl From<&crate::entity::manufacturers::Model> for ManufacturerResponse {
    fn from(m: &crate::entity::manufacturers::Model) -> Self {
        Self {
            id: m.id,
            name: m.name.clone(),
        }
    }
}
