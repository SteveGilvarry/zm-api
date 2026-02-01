use crate::dto::PaginatedResponse;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
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

/// Paginated response for storage
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PaginatedStorageResponse {
    pub items: Vec<StorageResponse>,
    pub total: u64,
    pub per_page: u64,
    pub current_page: u64,
    pub last_page: u64,
}

impl From<PaginatedResponse<StorageResponse>> for PaginatedStorageResponse {
    fn from(r: PaginatedResponse<StorageResponse>) -> Self {
        Self {
            items: r.items,
            total: r.total,
            per_page: r.per_page,
            current_page: r.current_page,
            last_page: r.last_page,
        }
    }
}
