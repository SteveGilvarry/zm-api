use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::entity::montage_layouts::Model as MontageLayoutModel;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct MontageLayoutResponse {
    pub id: u32,
    pub name: String,
    pub user_id: u32,
    pub positions: Option<String>,
}

impl From<&MontageLayoutModel> for MontageLayoutResponse {
    fn from(model: &MontageLayoutModel) -> Self {
        Self {
            id: model.id,
            name: model.name.clone(),
            user_id: model.user_id,
            positions: model.positions.clone(),
        }
    }
}
