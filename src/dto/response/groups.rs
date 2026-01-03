use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GroupResponse {
    pub id: u32,
    pub name: String,
    pub parent_id: Option<u32>,
}

impl From<&crate::entity::groups::Model> for GroupResponse {
    fn from(m: &crate::entity::groups::Model) -> Self {
        Self {
            id: m.id,
            name: m.name.clone(),
            parent_id: m.parent_id,
        }
    }
}
