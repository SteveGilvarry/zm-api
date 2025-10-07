use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::entity::states::Model as StateModel;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct StateResponse {
    pub id: u32,
    pub name: String,
    pub definition: String,
    pub is_active: u8,
}

impl From<&StateModel> for StateResponse {
    fn from(model: &StateModel) -> Self {
        Self {
            id: model.id,
            name: model.name.clone(),
            definition: model.definition.clone(),
            is_active: model.is_active,
        }
    }
}
