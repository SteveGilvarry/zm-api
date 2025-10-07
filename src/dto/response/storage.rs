use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StorageResponse {
    pub id: u16,
    pub name: String,
    pub path: String,
    pub r#type: String,
    pub enabled: i8,
}

impl From<&crate::entity::storage::Model> for StorageResponse {
    fn from(m: &crate::entity::storage::Model) -> Self {
        Self {
            id: m.id,
            name: m.name.clone(),
            path: m.path.clone(),
            r#type: m.r#type.to_string(),
            enabled: m.enabled,
        }
    }
}

