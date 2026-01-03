use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ModelResponse {
    pub id: u32,
    pub name: String,
    pub manufacturer_id: Option<i32>,
}

impl From<&crate::entity::models::Model> for ModelResponse {
    fn from(m: &crate::entity::models::Model) -> Self {
        Self {
            id: m.id,
            name: m.name.clone(),
            manufacturer_id: m.manufacturer_id,
        }
    }
}
